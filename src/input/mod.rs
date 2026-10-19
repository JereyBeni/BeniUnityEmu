//! Host input abstraction.
//!
//! Provides a platform-neutral touch model suitable for mapping mouse/touch
//! into a TouchScript-compatible guest input layer once a managed runtime exists.
//!
//! This module does **not** claim that TouchScript C# assemblies are loaded.

pub mod touch;

pub use touch::{
    MouseTouchMapper, TouchContact, TouchInputDispatcher, TouchInputEvent, TouchInputProvider,
    TouchInputQueue, TouchPhase,
};

/// Legacy stub kept for existing call sites.
pub struct InputBackend;

impl InputBackend {
    pub fn initialize() -> Result<(), String> {
        println!("[INPUT] Host input backend ready (touch queue available)");
        Ok(())
    }

    pub fn init() {
        let _ = Self::initialize();
    }

    pub fn poll(&self) {}

    pub fn shutdown(&self) {
        println!("[INPUT] Shutdown");
    }
}
