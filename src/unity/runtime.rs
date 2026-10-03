//! Unity runtime state machine (shared by Android & iOS).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnityState {
    Created,
    Loading,
    NativeLoaded,
    EngineInitialized,
    GraphicsInitialized,
    AudioInitialized,
    InputInitialized,
    Running,
    Stopped,
    Error,
}

pub struct UnityRuntime {
    pub apk_path: PathBuf,
    pub extracted_dir: Option<PathBuf>,
    pub architecture: Option<String>,
    pub state: UnityState,
    pub native_libs: Vec<String>,
}

impl UnityRuntime {
    pub fn new(apk_path: &Path, architecture: Option<String>) -> Self {
        UnityRuntime {
            apk_path: apk_path.to_path_buf(),
            extracted_dir: None,
            architecture,
            state: UnityState::Created,
            native_libs: Vec::new(),
        }
    }

    pub fn state(&self) -> UnityState {
        self.state
    }

    pub fn set_state(&mut self, state: UnityState) {
        self.state = state;
    }

    pub fn set_extracted_dir(&mut self, dir: PathBuf) {
        self.extracted_dir = Some(dir);
    }

    pub fn add_native_lib(&mut self, name: &str) {
        if !self.native_libs.iter().any(|l| l == name) {
            self.native_libs.push(name.to_string());
        }
    }

    pub fn try_run(&mut self) -> Result<(), String> {
        if self.state != UnityState::EngineInitialized
            && self.state != UnityState::GraphicsInitialized
            && self.state != UnityState::InputInitialized
        {
            return Err("[UNITY] Runtime not fully initialized".to_string());
        }
        self.state = UnityState::Error;
        Err("[UNITY] Execution backend unavailable.".to_string())
    }
}
