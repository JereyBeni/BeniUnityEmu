//! Touch input model aligned with TouchScript-style contact semantics.
//!
//! Reference: https://github.com/TouchScript/TouchScript
//! Host-side only: queues events for a future managed/Unity guest bridge.

use std::collections::{HashMap, VecDeque};

/// Phase of a touch contact (TouchScript / Unity TouchPhase compatible naming).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchPhase {
    Began,
    Moved,
    Stationary,
    Ended,
    Canceled,
}

impl TouchPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            TouchPhase::Began => "Began",
            TouchPhase::Moved => "Moved",
            TouchPhase::Stationary => "Stationary",
            TouchPhase::Ended => "Ended",
            TouchPhase::Canceled => "Canceled",
        }
    }
}

/// One contact point with a stable id for the lifetime of the contact.
#[derive(Debug, Clone, PartialEq)]
pub struct TouchContact {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub phase: TouchPhase,
    pub pressure: f32,
    /// Host timestamp in milliseconds (monotonic preferred).
    pub timestamp_ms: u64,
}

/// Discrete input event delivered to the guest in order.
#[derive(Debug, Clone, PartialEq)]
pub struct TouchInputEvent {
    pub contact: TouchContact,
    pub frame: u64,
}

/// Ordered queue consumed by the guest update cycle.
#[derive(Debug, Default)]
pub struct TouchInputQueue {
    events: VecDeque<TouchInputEvent>,
    frame: u64,
}

impl TouchInputQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, mut contact: TouchContact) {
        contact.timestamp_ms = contact.timestamp_ms;
        self.events.push_back(TouchInputEvent {
            contact,
            frame: self.frame,
        });
    }

    pub fn pop(&mut self) -> Option<TouchInputEvent> {
        self.events.pop_front()
    }

    pub fn peek(&self) -> Option<&TouchInputEvent> {
        self.events.front()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Advance frame counter (call once per host/guest frame).
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn drain(&mut self) -> Vec<TouchInputEvent> {
        self.events.drain(..).collect()
    }
}

/// Tracks active contacts and assigns stable ids.
pub struct TouchInputProvider {
    next_id: u64,
    active: HashMap<u64, TouchContact>,
    /// Optional external pointer id → contact id (for multi-touch hosts).
    pointer_map: HashMap<u64, u64>,
    viewport_w: f32,
    viewport_h: f32,
}

impl Default for TouchInputProvider {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

impl TouchInputProvider {
    pub fn new(viewport_w: f32, viewport_h: f32) -> Self {
        TouchInputProvider {
            next_id: 1,
            active: HashMap::new(),
            pointer_map: HashMap::new(),
            viewport_w: viewport_w.max(1.0),
            viewport_h: viewport_h.max(1.0),
        }
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.viewport_w = w.max(1.0);
        self.viewport_h = h.max(1.0);
    }

    pub fn viewport(&self) -> (f32, f32) {
        (self.viewport_w, self.viewport_h)
    }

    /// Normalize host pixel coords into viewport space (origin top-left).
    pub fn scale_point(&self, x: f32, y: f32) -> (f32, f32) {
        (x.clamp(0.0, self.viewport_w), y.clamp(0.0, self.viewport_h))
    }

    pub fn active_contacts(&self) -> impl Iterator<Item = &TouchContact> {
        self.active.values()
    }

    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// Begin a contact; returns the stable contact id.
    pub fn begin(
        &mut self,
        pointer_id: u64,
        x: f32,
        y: f32,
        timestamp_ms: u64,
        queue: &mut TouchInputQueue,
    ) -> u64 {
        let (x, y) = self.scale_point(x, y);
        let id = self.alloc_id();
        self.pointer_map.insert(pointer_id, id);
        let contact = TouchContact {
            id,
            x,
            y,
            phase: TouchPhase::Began,
            pressure: 1.0,
            timestamp_ms,
        };
        self.active.insert(id, contact.clone());
        queue.push(contact);
        id
    }

    pub fn move_contact(
        &mut self,
        pointer_id: u64,
        x: f32,
        y: f32,
        timestamp_ms: u64,
        queue: &mut TouchInputQueue,
    ) {
        let (x, y) = self.scale_point(x, y);
        let Some(&id) = self.pointer_map.get(&pointer_id) else {
            return;
        };
        if let Some(c) = self.active.get_mut(&id) {
            c.x = x;
            c.y = y;
            c.phase = TouchPhase::Moved;
            c.timestamp_ms = timestamp_ms;
            queue.push(c.clone());
        }
    }

    pub fn end(
        &mut self,
        pointer_id: u64,
        timestamp_ms: u64,
        queue: &mut TouchInputQueue,
    ) {
        let Some(id) = self.pointer_map.remove(&pointer_id) else {
            return;
        };
        if let Some(mut c) = self.active.remove(&id) {
            c.phase = TouchPhase::Ended;
            c.timestamp_ms = timestamp_ms;
            queue.push(c);
        }
    }

    pub fn cancel(
        &mut self,
        pointer_id: u64,
        timestamp_ms: u64,
        queue: &mut TouchInputQueue,
    ) {
        let Some(id) = self.pointer_map.remove(&pointer_id) else {
            return;
        };
        if let Some(mut c) = self.active.remove(&id) {
            c.phase = TouchPhase::Canceled;
            c.timestamp_ms = timestamp_ms;
            queue.push(c);
        }
    }

    /// Cancel all active contacts (window blur / focus loss).
    pub fn cancel_all(&mut self, timestamp_ms: u64, queue: &mut TouchInputQueue) {
        let ids: Vec<u64> = self.pointer_map.keys().copied().collect();
        for pid in ids {
            self.cancel(pid, timestamp_ms, queue);
        }
    }
}

/// Maps primary mouse button to a single touch contact (desktop testing).
pub struct MouseTouchMapper {
    pointer_id: u64,
    down: bool,
}

impl Default for MouseTouchMapper {
    fn default() -> Self {
        Self {
            pointer_id: 0,
            down: false,
        }
    }
}

impl MouseTouchMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_button_down(
        &mut self,
        x: f32,
        y: f32,
        timestamp_ms: u64,
        provider: &mut TouchInputProvider,
        queue: &mut TouchInputQueue,
    ) {
        if self.down {
            return;
        }
        self.down = true;
        provider.begin(self.pointer_id, x, y, timestamp_ms, queue);
    }

    pub fn on_move(
        &mut self,
        x: f32,
        y: f32,
        timestamp_ms: u64,
        provider: &mut TouchInputProvider,
        queue: &mut TouchInputQueue,
    ) {
        if self.down {
            provider.move_contact(self.pointer_id, x, y, timestamp_ms, queue);
        }
    }

    pub fn on_button_up(
        &mut self,
        timestamp_ms: u64,
        provider: &mut TouchInputProvider,
        queue: &mut TouchInputQueue,
    ) {
        if !self.down {
            return;
        }
        self.down = false;
        provider.end(self.pointer_id, timestamp_ms, queue);
    }

    pub fn is_down(&self) -> bool {
        self.down
    }
}

/// Routes host events into the queue; optional sink for future managed bridge.
pub struct TouchInputDispatcher {
    pub provider: TouchInputProvider,
    pub queue: TouchInputQueue,
    pub mouse: MouseTouchMapper,
}

impl TouchInputDispatcher {
    pub fn new(viewport_w: f32, viewport_h: f32) -> Self {
        TouchInputDispatcher {
            provider: TouchInputProvider::new(viewport_w, viewport_h),
            queue: TouchInputQueue::new(),
            mouse: MouseTouchMapper::new(),
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.provider.set_viewport(w, h);
    }

    pub fn begin_frame(&mut self) {
        self.queue.begin_frame();
    }

    /// Consume queued events (guest update).
    pub fn take_events(&mut self) -> Vec<TouchInputEvent> {
        self.queue.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_to_touch_begin_move_end() {
        let mut d = TouchInputDispatcher::new(800.0, 600.0);
        d.mouse
            .on_button_down(10.0, 20.0, 1, &mut d.provider, &mut d.queue);
        d.mouse
            .on_move(15.0, 25.0, 2, &mut d.provider, &mut d.queue);
        d.mouse
            .on_button_up(3, &mut d.provider, &mut d.queue);

        let events = d.take_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].contact.phase, TouchPhase::Began);
        assert_eq!(events[1].contact.phase, TouchPhase::Moved);
        assert_eq!(events[2].contact.phase, TouchPhase::Ended);
        // Stable id across the contact lifetime
        assert_eq!(events[0].contact.id, events[1].contact.id);
        assert_eq!(events[1].contact.id, events[2].contact.id);
        assert!((events[1].contact.x - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multi_contact_ids_stable_and_distinct() {
        let mut p = TouchInputProvider::new(100.0, 100.0);
        let mut q = TouchInputQueue::new();
        let a = p.begin(10, 1.0, 1.0, 0, &mut q);
        let b = p.begin(11, 2.0, 2.0, 1, &mut q);
        assert_ne!(a, b);
        p.move_contact(10, 3.0, 3.0, 2, &mut q);
        p.end(11, 3, &mut q);
        let ev = q.drain();
        assert_eq!(ev[0].contact.id, a);
        assert_eq!(ev[1].contact.id, b);
        assert_eq!(ev[2].contact.id, a);
        assert_eq!(ev[3].contact.phase, TouchPhase::Ended);
        assert_eq!(ev[3].contact.id, b);
        assert_eq!(p.active_count(), 1);
    }

    #[test]
    fn cancel_all_on_blur() {
        let mut p = TouchInputProvider::new(100.0, 100.0);
        let mut q = TouchInputQueue::new();
        p.begin(1, 0.0, 0.0, 0, &mut q);
        p.begin(2, 1.0, 1.0, 0, &mut q);
        p.cancel_all(5, &mut q);
        assert_eq!(p.active_count(), 0);
        let ev = q.drain();
        assert!(ev.iter().any(|e| e.contact.phase == TouchPhase::Canceled));
    }

    #[test]
    fn viewport_clamp() {
        let p = TouchInputProvider::new(100.0, 50.0);
        let (x, y) = p.scale_point(-10.0, 999.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 50.0);
    }

    #[test]
    fn ordered_frame_delivery() {
        let mut q = TouchInputQueue::new();
        q.begin_frame();
        q.push(TouchContact {
            id: 1,
            x: 0.0,
            y: 0.0,
            phase: TouchPhase::Began,
            pressure: 1.0,
            timestamp_ms: 0,
        });
        q.begin_frame();
        q.push(TouchContact {
            id: 1,
            x: 1.0,
            y: 1.0,
            phase: TouchPhase::Moved,
            pressure: 1.0,
            timestamp_ms: 1,
        });
        let ev = q.drain();
        assert_eq!(ev[0].frame, 1);
        assert_eq!(ev[1].frame, 2);
    }
}
