//! Graphics abstraction for BeniUnityEmu.
//!
//! Stage C1 adds an optional **desktop OpenGL** path (`feature = "opengl"`)
//! that opens a real window and draws a diagnostic triangle.
//!
//! This module does **not** execute Unity guest code.

pub mod capabilities;

#[cfg(feature = "opengl")]
pub mod opengl;

#[cfg(feature = "opengl")]
pub mod runtime;

pub use capabilities::{CapabilityStatus, RendererCapabilities};

/// Window / diagnostics settings for Stage C1 (active options only).
#[derive(Debug, Clone)]
pub struct GraphicsSettings {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub show_renderer_info: bool,
    pub show_fps: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        GraphicsSettings {
            title: "BeniUnityEmu - OpenGL Runtime [PARTIAL]".into(),
            width: 1280,
            height: 720,
            resizable: true,
            show_renderer_info: true,
            show_fps: true,
        }
    }
}

impl GraphicsSettings {
    pub fn validated(mut self) -> Self {
        if self.width < 320 {
            self.width = 320;
        }
        if self.width > 7680 {
            self.width = 7680;
        }
        if self.height < 240 {
            self.height = 240;
        }
        if self.height > 4320 {
            self.height = 4320;
        }
        if self.title.trim().is_empty() {
            self.title = Self::default().title;
        }
        self
    }

    /// Load from config.yml `graphics:` keys when present; otherwise defaults.
    pub fn load() -> Self {
        let path = std::env::var("BENI_CONFIG").unwrap_or_else(|_| "config.yml".into());
        let text = std::fs::read_to_string(&path).ok();
        let Some(text) = text else {
            return Self::default().validated();
        };
        let mut s = Self::default();
        if let Some(v) = yaml_u32(&text, "width") {
            s.width = v;
        }
        if let Some(v) = yaml_u32(&text, "height") {
            s.height = v;
        }
        if let Some(v) = yaml_bool(&text, "resizable") {
            s.resizable = v;
        }
        if let Some(v) = yaml_bool(&text, "show_renderer_info") {
            s.show_renderer_info = v;
        }
        if let Some(v) = yaml_bool(&text, "show_fps") {
            s.show_fps = v;
        }
        if let Some(v) = yaml_string(&text, "title") {
            s.title = v;
        }
        s.validated()
    }
}

fn yaml_u32(text: &str, key: &str) -> Option<u32> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&format!("{}:", key)) {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn yaml_bool(text: &str, key: &str) -> Option<bool> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&format!("{}:", key)) {
            return match rest.trim() {
                "true" | "True" | "yes" | "1" => Some(true),
                "false" | "False" | "no" | "0" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn yaml_string(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(&format!("{}:", key)) {
            let v = rest.trim().trim_matches('"').trim_matches('\'');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub struct GraphicsBackend;

impl GraphicsBackend {
    pub fn initialize() -> Result<(), String> {
        #[cfg(feature = "opengl")]
        {
            println!(
                "[GRAPHICS] OpenGL feature enabled — use BeniOpenGL binary for Stage C1 window"
            );
            return Ok(());
        }
        #[cfg(not(feature = "opengl"))]
        {
            println!(
                "[GRAPHICS] Backend stub (compile with --features opengl for Stage C1)"
            );
            Ok(())
        }
    }

    pub fn init() {
        let _ = Self::initialize();
    }

    pub fn create_context(&self) -> Result<(), String> {
        Err(
            "[GRAPHICS] create_context: use BeniOpenGL / graphics::runtime for a real context"
                .into(),
        )
    }

    pub fn create_texture(&self) -> Result<(), String> {
        Err(
            "[GRAPHICS] create_texture not implemented (Stage C1: PARTIAL capability only)".into(),
        )
    }

    pub fn create_buffer(&self) -> Result<(), String> {
        Err("[GRAPHICS] create_buffer not implemented".into())
    }

    pub fn draw(&self) -> Result<(), String> {
        Err("[GRAPHICS] draw not implemented on stub backend".into())
    }

    pub fn present(&self) -> Result<(), String> {
        Err("[GRAPHICS] present not implemented on stub backend".into())
    }

    pub fn shutdown(&self) {
        println!("[GRAPHICS] Shutdown");
    }

    pub fn capabilities() -> RendererCapabilities {
        #[cfg(feature = "opengl")]
        {
            RendererCapabilities::stage_c1_opengl()
        }
        #[cfg(not(feature = "opengl"))]
        {
            RendererCapabilities::stub()
        }
    }
}

#[cfg(feature = "opengl")]
pub fn run_opengl_runtime_from_config() -> Result<(), String> {
    let settings = GraphicsSettings::load();
    runtime::run_partial_opengl_runtime(&settings)
}

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn validates_dimensions() {
        let s = GraphicsSettings {
            width: 10,
            height: 10,
            title: String::new(),
            ..GraphicsSettings::default()
        }
        .validated();
        assert_eq!(s.width, 320);
        assert_eq!(s.height, 240);
        assert!(!s.title.is_empty());
    }
}
