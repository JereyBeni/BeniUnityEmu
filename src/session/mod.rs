//! Game session lifecycle management.
//! Tracks state without claiming that a real game has executed.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::android::runtime::AndroidRuntime;
use crate::ios::runtime::IosRuntime;
use crate::unity::runtime::{UnityRuntime, UnityState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Preparing,
    PlatformInit,
    UnityInit,
    Ready,
    Running,
    Stopped,
    Failed,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Created => "Created",
            SessionState::Preparing => "Preparing",
            SessionState::PlatformInit => "PlatformInit",
            SessionState::UnityInit => "UnityInit",
            SessionState::Ready => "Ready",
            SessionState::Running => "Running",
            SessionState::Stopped => "Stopped",
            SessionState::Failed => "Failed",
        }
    }
}

pub struct GameSession {
    pub id: String,
    pub app_name: String,
    pub app_path: PathBuf,
    pub platform: String,
    pub state: SessionState,
    pub last_message: String,
    pub unity: UnityRuntime,
    android: Option<AndroidRuntime>,
    ios: Option<IosRuntime>,
}

impl GameSession {
    pub fn new(app_name: &str, app_path: &Path, platform: &str) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = format!("sess-{}-{}", app_name, ts);
        let unity = UnityRuntime::new(app_path, None);
        let (android, ios) = match platform {
            "android" => (Some(AndroidRuntime::new()), None),
            "ios" => (None, Some(IosRuntime::new())),
            _ => (None, None),
        };
        GameSession {
            id: id.clone(),
            app_name: app_name.to_string(),
            app_path: app_path.to_path_buf(),
            platform: platform.to_string(),
            state: SessionState::Created,
            last_message: format!("Session {} created for {}", id, app_name),
            unity,
            android,
            ios,
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.state = SessionState::Preparing;
        self.last_message = format!("[{}] Preparing session", self.id);
        println!("{}", self.last_message);

        self.state = SessionState::PlatformInit;
        match self.platform.as_str() {
            "android" => {
                if let Some(ref mut rt) = self.android {
                    rt.initialize();
                }
                self.last_message =
                    format!("[{}] Android compatibility layer ready (stub)", self.id);
            }
            "ios" => {
                if let Some(ref mut rt) = self.ios {
                    rt.initialize();
                }
                self.last_message =
                    format!("[{}] iOS compatibility layer ready (stub)", self.id);
            }
            other => {
                self.state = SessionState::Failed;
                self.last_message = format!("[{}] Unknown platform: {}", self.id, other);
                return Err(self.last_message.clone());
            }
        }
        println!("{}", self.last_message);

        self.state = SessionState::UnityInit;
        self.unity.set_state(UnityState::Loading);
        self.unity.set_state(UnityState::NativeLoaded);
        self.unity.set_state(UnityState::EngineInitialized);
        self.last_message = format!(
            "[{}] Unity runtime state machine advanced to EngineInitialized (no execution backend)",
            self.id
        );
        println!("{}", self.last_message);

        self.state = SessionState::Ready;
        self.last_message = format!(
            "[{}] Session ready. Execution backend is not available yet – game was NOT run.",
            self.id
        );
        println!("{}", self.last_message);

        self.state = SessionState::Stopped;
        self.unity.set_state(UnityState::Stopped);
        self.last_message = format!(
            "[{}] Session stopped. No guest code was executed. Architecture is ready for Stage E integration.",
            self.id
        );
        Ok(())
    }

    pub fn stop(&mut self) {
        self.state = SessionState::Stopped;
        self.unity.set_state(UnityState::Stopped);
        self.last_message = format!("[{}] Session stopped by request", self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn session_lifecycle() {
        let mut s = GameSession::new("testapp", Path::new("/tmp/test.apk"), "android");
        assert_eq!(s.state(), SessionState::Created);
        assert!(s.id.starts_with("sess-testapp-"));
        let r = s.run();
        assert!(r.is_ok());
        assert_eq!(s.state(), SessionState::Stopped);
        assert!(
            s.last_message.contains("No guest code was executed")
                || s.last_message.contains("stopped")
        );
    }

    #[test]
    fn unknown_platform_fails() {
        let mut s = GameSession::new("x", Path::new("/tmp/x"), "amiga");
        let r = s.run();
        assert!(r.is_err());
        assert_eq!(s.state(), SessionState::Failed);
    }
}
