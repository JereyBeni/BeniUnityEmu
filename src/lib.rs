//! BeniUnityEmu library – shared runtime for CLI and App Picker.

pub mod ziputil;
pub mod memory;
pub mod session;
pub mod jni;
pub mod apk;
pub mod ipa;
pub mod cache;
pub mod cpu;
pub mod elf;
pub mod android;
pub mod ios;
pub mod unity;
pub mod graphics;
pub mod audio;
pub mod input;
pub mod app_picker;

use std::fs;
use std::path::{Path, PathBuf};

use apk::loader::ApkLoader;
use cache::manager::CacheManager;
use ipa::loader::IpaLoader;
use session::GameSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFormat {
    Apk,
    Ipa,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: PathBuf,
    pub format: AppFormat,
    pub package: Option<String>,
    pub version: Option<String>,
    pub architecture: Option<String>,
    pub has_libunity: bool,
    pub has_libmain: bool,
    pub has_mono: bool,
    pub native_libs: Vec<String>,
    pub size_bytes: u64,
}

impl AppInfo {
    pub fn format_label(&self) -> &'static str {
        match self.format {
            AppFormat::Apk => "APK",
            AppFormat::Ipa => "IPA",
            AppFormat::Unknown => "???",
        }
    }

    pub fn size_label(&self) -> String {
        let b = self.size_bytes as f64;
        if b >= 1_073_741_824.0 {
            format!("{:.2} GB", b / 1_073_741_824.0)
        } else if b >= 1_048_576.0 {
            format!("{:.1} MB", b / 1_048_576.0)
        } else if b >= 1024.0 {
            format!("{:.0} KB", b / 1024.0)
        } else {
            format!("{} B", self.size_bytes)
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.format {
            AppFormat::Apk => {
                if self.has_libunity || self.has_libmain {
                    "Ready"
                } else if self.architecture.is_some() {
                    "Unsupported"
                } else {
                    "Ready"
                }
            }
            AppFormat::Ipa => "Ready",
            AppFormat::Unknown => "Error",
        }
    }
}

pub fn ensure_directories() {
    let dirs = [
        "BeniApps",
        "BeniAPK_Apps",
        "BeniIOS_Apps",
        "BeniFonts",
        "BeniSO",
        "BeniDyLIB",
        "cache",
    ];
    for d in dirs {
        let path = PathBuf::from(d);
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(&path) {
                eprintln!("[ERROR] Failed to create directory {}: {}", d, e);
            } else {
                println!("[INFO] Created directory: {}", d);
            }
        }
    }
}

pub fn scan_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let roots: &[(&str, Option<AppFormat>)] = &[
        ("BeniApps", None),
        ("BeniAPK_Apps", Some(AppFormat::Apk)),
        ("BeniIOS_Apps", Some(AppFormat::Ipa)),
    ];

    for (dir, forced) in roots {
        let dir_path = PathBuf::from(dir);
        if !dir_path.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let format = match (forced, ext.as_str()) {
                    (_, "apk") => AppFormat::Apk,
                    (_, "ipa") => AppFormat::Ipa,
                    (Some(f), _) => *f,
                    _ => continue,
                };
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                if !seen.insert(name.clone()) {
                    continue;
                }
                let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                apps.push(AppInfo {
                    name,
                    path,
                    format,
                    package: None,
                    version: None,
                    architecture: None,
                    has_libunity: false,
                    has_libmain: false,
                    has_mono: false,
                    native_libs: Vec::new(),
                    size_bytes,
                });
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

pub fn find_app(name: &str) -> Option<AppInfo> {
    scan_apps().into_iter().find(|a| {
        a.name == name
            || a.name == format!("{}.apk", name)
            || a.name == format!("{}.ipa", name)
    })
}

pub fn enrich_app_info(info: &mut AppInfo) {
    match info.format {
        AppFormat::Apk => match ApkLoader::open(&info.path) {
            Ok(loader) => {
                let summary = loader.summarize();
                info.has_libunity = summary.has_libunity;
                info.has_libmain = summary.has_libmain;
                info.has_mono = summary.has_mono;
                info.native_libs = summary.native_libs;
                info.architecture = summary.preferred_arch;
            }
            Err(e) => eprintln!("[ERROR] Failed to open APK: {}", e),
        },
        AppFormat::Ipa => match IpaLoader::open(&info.path) {
            Ok(loader) => {
                let summary = loader.summarize();
                info.native_libs = summary.native_libs;
                info.architecture = summary.preferred_arch;
            }
            Err(e) => eprintln!("[ERROR] Failed to open IPA: {}", e),
        },
        AppFormat::Unknown => {}
    }
}

pub fn extract_app(app: &AppInfo) -> Result<String, String> {
    let cache_mgr = CacheManager::new("cache");
    let app_name = app
        .name
        .trim_end_matches(".apk")
        .trim_end_matches(".ipa");

    if cache_mgr.has_valid_cache(app_name) {
        return Ok(format!("[CACHE] Existing cache found for {}", app_name));
    }

    match app.format {
        AppFormat::Apk => {
            let loader = ApkLoader::open(&app.path)?;
            let meta = cache_mgr.extract_apk(app_name, &loader)?;
            Ok(format!(
                "[CACHE] Extracted {} (arch={:?}, libunity={})",
                meta.name, meta.architecture, meta.has_libunity
            ))
        }
        AppFormat::Ipa => {
            let loader = IpaLoader::open(&app.path)?;
            let _meta = cache_mgr.extract_ipa(app_name, &loader)?;
            Ok(format!("[CACHE] IPA cache created for {}", app_name))
        }
        AppFormat::Unknown => Err("Unknown format".into()),
    }
}

pub fn inspect_app(app: &AppInfo) -> String {
    let mut out = String::new();
    match app.format {
        AppFormat::Apk => match ApkLoader::open(&app.path) {
            Ok(loader) => {
                let summary = loader.summarize();
                out.push_str(&format!("[APK] {}\n", app.name));
                out.push_str(&format!("  Files: {}\n", summary.file_count));
                out.push_str(&format!(
                    "  Architecture: {}\n",
                    summary.preferred_arch.as_deref().unwrap_or("unknown")
                ));
                out.push_str(&format!("  libunity: {}\n", summary.has_libunity));
                out.push_str(&format!("  libmain: {}\n", summary.has_libmain));
                out.push_str(&format!("  mono: {}\n", summary.has_mono));
                out.push_str(&format!("  Native libs: {}\n", summary.native_libs.len()));
                for lib in &summary.native_libs {
                    out.push_str(&format!("    - {}\n", lib));
                }
                loader.print_structure();
            }
            Err(e) => out.push_str(&format!("[ERROR] {}\n", e)),
        },
        AppFormat::Ipa => match IpaLoader::open(&app.path) {
            Ok(loader) => {
                let summary = loader.summarize();
                out.push_str(&format!("[IPA] {}\n", app.name));
                out.push_str(&format!("  Files: {}\n", summary.file_count));
                out.push_str(&format!(
                    "  Architecture: {}\n",
                    summary.preferred_arch.as_deref().unwrap_or("arm64")
                ));
                loader.print_structure();
            }
            Err(e) => out.push_str(&format!("[ERROR] {}\n", e)),
        },
        AppFormat::Unknown => out.push_str("[ERROR] Unknown format\n"),
    }
    out
}

pub fn run_app(app: &AppInfo) -> Result<String, String> {
    let app_name = app
        .name
        .trim_end_matches(".apk")
        .trim_end_matches(".ipa");
    let platform = match app.format {
        AppFormat::Apk => "android",
        AppFormat::Ipa => "ios",
        AppFormat::Unknown => return Err("Unknown format".into()),
    };

    let cache_mgr = CacheManager::new("cache");
    if !cache_mgr.has_valid_cache(app_name) {
        let _ = extract_app(app);
    }

    let mut session = GameSession::new(app_name, &app.path, platform);
    session.run()?;
    Ok(session.last_message)
}

pub fn path_exists(p: &str) -> bool {
    Path::new(p).exists()
}
