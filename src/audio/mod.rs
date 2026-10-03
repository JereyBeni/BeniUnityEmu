//! Audio abstraction.

pub struct AudioBackend;

impl AudioBackend {
    pub fn initialize() -> Result<(), String> {
        println!("[AUDIO] Backend initialized (stub)");
        Ok(())
    }

    pub fn init() {
        let _ = Self::initialize();
    }

    pub fn load_audio(&self, _path: &str) -> Result<(), String> {
        Err("[AUDIO] load_audio not implemented".into())
    }

    pub fn play(&self) -> Result<(), String> {
        Err("[AUDIO] play not implemented".into())
    }

    pub fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn set_volume(&self, _v: f32) -> Result<(), String> {
        Ok(())
    }

    pub fn shutdown(&self) {
        println!("[AUDIO] Shutdown");
    }
}
