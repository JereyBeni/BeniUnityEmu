//! Game session lifecycle management.
//! Tracks real launch stages. Never claims guest code ran unless a backend executed it.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::android::runtime::AndroidRuntime;
use crate::ios::runtime::IosRuntime;
use crate::ipa::loader::IpaLoader;
use crate::unity::runtime::{UnityRuntime, UnityState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Extracting,
    Inspecting,
    Loading,
    Running,
    Stopped,
    Failed,
    UnsupportedRuntime,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Created => "Created",
            SessionState::Extracting => "Extracting",
            SessionState::Inspecting => "Inspecting",
            SessionState::Loading => "Loading",
            SessionState::Running => "Running",
            SessionState::Stopped => "Stopped",
            SessionState::Failed => "Failed",
            SessionState::UnsupportedRuntime => "UnsupportedRuntime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchResult {
    UnsupportedRuntime {
        reason: String,
        architecture: Option<String>,
        executable: Option<String>,
    },
    Failed {
        stage: String,
        error: String,
    },
    Executed {
        note: String,
    },
}

impl LaunchResult {
    pub fn is_executed(&self) -> bool {
        matches!(self, LaunchResult::Executed { .. })
    }
}

pub struct GameSession {
    pub id: String,
    pub app_name: String,
    pub app_path: PathBuf,
    pub platform: String,
    pub state: SessionState,
    pub last_message: String,
    pub log: String,
    pub launch_result: Option<LaunchResult>,
    pub unity: UnityRuntime,
    android: Option<AndroidRuntime>,
    ios: Option<IosRuntime>,
    work_dir: PathBuf,
}

impl GameSession {
    pub fn new(app_name: &str, app_path: &Path, platform: &str) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = format!("sess-{}-{}", sanitize_id(app_name), ts);
        let work_dir = PathBuf::from("cache")
            .join("sessions")
            .join(sanitize_id(&id));
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
            last_message: format!("[Launch] {}", app_name),
            log: format!("[Launch] {}\n", app_name),
            launch_result: None,
            unity,
            android,
            ios,
            work_dir,
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    fn append_log(&mut self, line: &str) {
        self.log.push_str(line);
        if !line.ends_with('\n') {
            self.log.push('\n');
        }
        self.last_message = line.trim_end().to_string();
        println!("{}", line.trim_end());
    }

    pub fn run(&mut self) -> Result<(), String> {
        match self.platform.as_str() {
            "ios" => self.run_ios_pipeline(),
            "android" => self.run_android_pipeline(),
            other => {
                self.state = SessionState::Failed;
                let msg = format!("[Result] Failed — unknown platform: {}", other);
                self.append_log(&msg);
                self.launch_result = Some(LaunchResult::Failed {
                    stage: "platform".into(),
                    error: format!("unknown platform: {}", other),
                });
                Err(msg)
            }
        }
    }

    fn run_ios_pipeline(&mut self) -> Result<(), String> {
        self.state = SessionState::Extracting;
        self.append_log("[Extract] Starting IPA extraction…");

        if let Err(e) = fs::create_dir_all(&self.work_dir) {
            self.state = SessionState::Failed;
            let msg = format!("[Extract] Failed to create work dir: {}", e);
            self.append_log(&msg);
            self.launch_result = Some(LaunchResult::Failed {
                stage: "extract".into(),
                error: e.to_string(),
            });
            return Err(msg);
        }

        let loader = match IpaLoader::open(&self.app_path) {
            Ok(l) => l,
            Err(e) => {
                self.state = SessionState::Failed;
                let msg = format!("[Extract] Cannot open IPA: {}", e);
                self.append_log(&msg);
                self.launch_result = Some(LaunchResult::Failed {
                    stage: "extract".into(),
                    error: e,
                });
                return Err(msg);
            }
        };

        let payload_dest = self.work_dir.join("Payload");
        match loader.extract_prefix("Payload/", &payload_dest) {
            Ok(n) => self.append_log(&format!("[Extract] Completed ({} files under Payload/)", n)),
            Err(e) => self.append_log(&format!("[Extract] Partial/failed: {}", e)),
        }

        self.state = SessionState::Inspecting;
        let summary = loader.summarize();

        if let Some(ref bundle) = summary.app_bundle {
            self.append_log(&format!("[Inspect] App bundle: {}", bundle.trim_end_matches('/')));
        } else {
            self.append_log("[Inspect] Warning: Payload/*.app not located");
        }
        if let Some(ref bid) = summary.bundle_id {
            self.append_log(&format!("[Inspect] Bundle ID: {}", bid));
        }
        if let Some(ref ver) = summary.bundle_version {
            self.append_log(&format!("[Inspect] Version: {}", ver));
        }

        let exe_name = summary.executable.clone();
        match &exe_name {
            Some(name) => self.append_log(&format!("[Inspect] Mach-O executable found: {}", name)),
            None => self.append_log("[Inspect] CFBundleExecutable not found in Info.plist"),
        }

        let arch = summary.preferred_arch.clone();
        self.append_log(&format!(
            "[Inspect] Architecture: {}",
            arch.as_deref().unwrap_or("unknown")
        ));

        // --- Stage F analysis ---
        if summary.has_unity {
            self.append_log("[Analyze] Unity game / framework markers detected");
        } else {
            self.append_log("[Analyze] Unity markers: not detected (may still be native UIKit)");
        }
        let backend = if summary.has_il2cpp {
            "IL2CPP (heuristic)"
        } else if summary.has_mono_assemblies {
            "Mono / managed assemblies (heuristic)"
        } else {
            "unknown / native-only (heuristic)"
        };
        self.append_log(&format!("[Analyze] Scripting backend: {}", backend));
        if summary.has_touchscript {
            self.append_log("[Analyze] TouchScript dependency: detected in package paths");
        } else {
            self.append_log("[Analyze] TouchScript dependency: not detected in package paths");
        }
        self.append_log("[Input] Host TouchScript-oriented input queue: available");
        self.append_log("[Input] TouchScript guest C# compatibility: not tested (no managed runtime)");

        if !summary.native_libs.is_empty() {
            self.append_log(&format!(
                "[Inspect] Native libraries: {} (.dylib/.so)",
                summary.native_libs.len()
            ));
        }

        self.state = SessionState::Loading;
        if let Some(ref mut rt) = self.ios {
            rt.initialize();
        }

        let missing = "Native iOS runtime unavailable: Mach-O/dyld loader, ARM64 guest CPU, \
Apple frameworks (UIKit/Foundation), and iOS syscalls are not implemented. \
Windows cannot execute iOS IPA binaries as PE images. \
Mono/.NET does not make a native IPA runnable. \
TouchScript host input exists but cannot reach guest scripts without a managed/native runtime.";

        self.append_log("[Runtime] Managed runtime: unavailable");
        self.append_log("[Runtime] Native iOS runtime: unavailable");
        self.append_log(&format!("[Runtime] {}", missing));

        self.state = SessionState::UnsupportedRuntime;
        self.unity.set_state(UnityState::Stopped);

        self.append_log("[Result] UnsupportedRuntime");
        self.append_log("[Details] Game scripts were not executed.");
        self.append_log(
            "[Details] The executable was inspected but no guest code was executed.",
        );

        self.launch_result = Some(LaunchResult::UnsupportedRuntime {
            reason: missing.to_string(),
            architecture: arch,
            executable: exe_name,
        });
        Ok(())
    }

    fn run_android_pipeline(&mut self) -> Result<(), String> {
        self.state = SessionState::Extracting;
        self.append_log("[Extract] Android/APK path — capability check only");
        self.state = SessionState::Inspecting;
        self.append_log("[Inspect] APK launch uses Android compatibility stubs only");
        self.state = SessionState::Loading;
        if let Some(ref mut rt) = self.android {
            rt.initialize();
        }
        self.unity.set_state(UnityState::Loading);
        self.unity.set_state(UnityState::NativeLoaded);

        let missing = "Android/native guest runtime unavailable: ART/JNI and ARM guest execution are not implemented.";
        self.append_log("[Runtime] Android guest runtime: unavailable");
        self.append_log(&format!("[Runtime] {}", missing));
        self.state = SessionState::UnsupportedRuntime;
        self.unity.set_state(UnityState::Stopped);
        self.append_log("[Result] UnsupportedRuntime");
        self.append_log("[Details] No guest code was executed.");

        self.launch_result = Some(LaunchResult::UnsupportedRuntime {
            reason: missing.to_string(),
            architecture: None,
            executable: None,
        });
        Ok(())
    }

    pub fn stop(&mut self) {
        self.state = SessionState::Stopped;
        self.unity.set_state(UnityState::Stopped);
        self.append_log(&format!("[Session] Stopped by request ({})", self.id));
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(80)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use zip::write::FileOptions;
    use zip::ZipWriter;

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn make_minimal_ipa() -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("beni_sess_ipa_{}_{}", std::process::id(), n));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("My Talking Tom 1.0.ipa");
        let file = fs::File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("Payload/MyTalkingTom.app/Info.plist", opts)
            .unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.outfit7.mytalkingtom</string>
<key>CFBundleShortVersionString</key><string>1.0</string>
<key>CFBundleExecutable</key><string>MyTalkingTom</string>
<key>MinimumOSVersion</key><string>7.0</string>
</dict></plist>"#,
        )
        .unwrap();
        let mut macho = vec![0u8; 32];
        macho[0..4].copy_from_slice(&0xfeed_facfu32.to_le_bytes());
        macho[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
        zip.start_file("Payload/MyTalkingTom.app/MyTalkingTom", opts)
            .unwrap();
        zip.write_all(&macho).unwrap();
        zip.finish().unwrap();
        path
    }

    #[test]
    fn ipa_launch_does_not_claim_execution() {
        let path = make_minimal_ipa();
        let mut s = GameSession::new("My Talking Tom 1.0", &path, "ios");
        assert!(s.run().is_ok());
        assert_eq!(s.state(), SessionState::UnsupportedRuntime);
        assert!(!matches!(s.launch_result, Some(LaunchResult::Executed { .. })));
        assert!(s.log.contains("[Analyze]"));
        assert!(s.log.contains("[Input] Host TouchScript-oriented input queue: available"));
        assert!(s.log.contains("[Result] UnsupportedRuntime"));
        assert!(!s.log.contains("Architecture is ready for Stage E integration"));
    }

    #[test]
    fn ipa_filename_with_spaces_and_brackets() {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("beni_sess_ipa2_{}_{}", std::process::id(), n));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(
            "My Learn Go [dRed Technologies Limited] (v1.0 iPad os42)-JCTOTGaming.rc336_935.ipa",
        );
        let src = make_minimal_ipa();
        fs::copy(&src, &path).unwrap();
        let mut s = GameSession::new(
            path.file_stem().unwrap().to_str().unwrap(),
            &path,
            "ios",
        );
        assert!(s.run().is_ok());
        assert_eq!(s.state(), SessionState::UnsupportedRuntime);
    }

    #[test]
    fn missing_ipa_file_fails_at_extract() {
        let mut s = GameSession::new("missing", Path::new("/nonexistent/no.ipa"), "ios");
        assert!(s.run().is_err());
        assert_eq!(s.state(), SessionState::Failed);
    }

    #[test]
    fn running_state_not_reached_without_backend() {
        let path = make_minimal_ipa();
        let mut s = GameSession::new("t", &path, "ios");
        let _ = s.run();
        assert_ne!(s.state(), SessionState::Running);
    }

    #[test]
    fn android_also_unsupported() {
        let mut s = GameSession::new("testapp", Path::new("/tmp/test.apk"), "android");
        assert!(s.run().is_ok());
        assert_eq!(s.state(), SessionState::UnsupportedRuntime);
    }

    #[test]
    fn unknown_platform_fails() {
        let mut s = GameSession::new("x", Path::new("/tmp/x"), "amiga");
        assert!(s.run().is_err());
        assert_eq!(s.state(), SessionState::Failed);
    }
}
