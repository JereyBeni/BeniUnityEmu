//! Input abstraction (keyboard/mouse → touch later).

pub struct InputBackend;

impl InputBackend {
    pub fn initialize() -> Result<(), String> {
        println!("[INPUT] Backend initialized (keyboard/mouse stub)");
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
