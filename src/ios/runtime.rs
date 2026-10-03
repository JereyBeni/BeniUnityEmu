//! iOS compatibility layer (stage 2).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IosState {
    Created,
    Loading,
    Initialized,
    Running,
    Stopped,
    Error,
}

pub struct IosRuntime {
    pub state: IosState,
    pub initialized: bool,
}

impl IosRuntime {
    pub fn new() -> Self {
        IosRuntime {
            state: IosState::Created,
            initialized: true,
        }
    }

    pub fn initialize(&mut self) {
        self.state = IosState::Initialized;
        self.print_status();
    }

    pub fn print_status(&self) {
        println!("[IOS] Compatibility layer initialized");
        println!("[IOS] NSBundle / resources: stub");
        println!("[IOS] UIKit / Foundation API: stub");
        println!("[IOS] Metal / OpenGL ES API: stub");
        println!("[IOS] Audio API: stub");
        println!("[IOS] Input API: stub");
        println!("[IOS] Application lifecycle: stub");
        println!("[IOS] Note: Mach-O loading and dyld simulation not implemented");
    }

    pub fn unsupported(api: &str) {
        println!("[IOS] Unsupported API: {}", api);
    }
}

impl Default for IosRuntime {
    fn default() -> Self { Self::new() }
}
