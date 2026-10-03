//! Graphics abstraction: Unity / GLES → host backend.

pub struct GraphicsBackend;

impl GraphicsBackend {
    pub fn initialize() -> Result<(), String> {
        println!("[GRAPHICS] Backend initialized (Windows host stub)");
        Ok(())
    }

    pub fn init() {
        let _ = Self::initialize();
    }

    pub fn create_context(&self) -> Result<(), String> {
        Err("[GRAPHICS] create_context not implemented".into())
    }

    pub fn create_texture(&self) -> Result<(), String> {
        Err("[GRAPHICS] create_texture not implemented".into())
    }

    pub fn create_buffer(&self) -> Result<(), String> {
        Err("[GRAPHICS] create_buffer not implemented".into())
    }

    pub fn draw(&self) -> Result<(), String> {
        Err("[GRAPHICS] draw not implemented".into())
    }

    pub fn present(&self) -> Result<(), String> {
        Err("[GRAPHICS] present not implemented".into())
    }

    pub fn shutdown(&self) {
        println!("[GRAPHICS] Shutdown");
    }
}
