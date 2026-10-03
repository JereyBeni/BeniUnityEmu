//! Android compatibility layer (stage 2).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidState {
    Created,
    Loading,
    Initialized,
    Running,
    Stopped,
    Error,
}

pub struct AndroidRuntime {
    pub state: AndroidState,
    pub filesystem: bool,
    pub opengl_es: bool,
    pub audio: bool,
    pub input: bool,
    pub activity: bool,
    pub lifecycle: bool,
    pub jni: bool,
}

impl AndroidRuntime {
    pub fn new() -> Self {
        AndroidRuntime {
            state: AndroidState::Created,
            filesystem: true,
            opengl_es: true,
            audio: true,
            input: true,
            activity: true,
            lifecycle: true,
            jni: true,
        }
    }

    pub fn initialize(&mut self) {
        self.state = AndroidState::Initialized;
        self.print_status();
    }

    pub fn print_status(&self) {
        println!("[ANDROID] Compatibility layer initialized");
        println!("[ANDROID] Activity API: stub");
        println!("[ANDROID] Context API: stub");
        println!("[ANDROID] AssetManager API: stub");
        println!("[ANDROID] OpenGL ES API: stub");
        println!("[ANDROID] Audio API: stub");
        println!("[ANDROID] Input API: stub");
        println!("[ANDROID] Filesystem API: stub");
        println!("[ANDROID] Lifecycle API: stub");
        println!("[ANDROID] JNI bridge: stub");
    }
}

impl Default for AndroidRuntime {
    fn default() -> Self { Self::new() }
}
