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
use unity::pangoland::inspect_pangoland_target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFormat {
    Apk,
    Ipa,
    /// Extracted Unity / APPX / standalone game directory (e.g. Pango Zoo, Pangoland).
    UnityFolder,
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
            AppFormat::UnityFolder => "Unity",
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
            AppFormat::Ipa => "Inspectable",
            AppFormat::UnityFolder => {
                if self.has_libunity || self.has_mono {
                    "Inspectable"
                } else {
                    "Folder"
                }
            }
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

/// Heuristic: directory looks like an extracted Unity / APPX / UWP game package.
fn looks_like_unity_folder(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    let markers = [
        "UnityPlayer.dll",
        "AppxManifest.xml",
        "globalgamemanagers",
        "mainData",
    ];
    for m in markers {
        if dir.join(m).exists() {
            return true;
        }
    }
    // Nested Managed / Data common layouts
    if dir.join("Data").join("Managed").is_dir() {
        return true;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for ent in entries.flatten() {
            let p = ent.path();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name.ends_with("_data") && p.is_dir() {
                return true;
            }
            if name == "managed" && p.is_dir() {
                return true;
            }
            if name == "unityplayer.dll" || name == "appxmanifest.xml" {
                return true;
            }
        }
    }
    false
}

fn dir_size_approx(dir: &Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    let mut n = 0usize;
    while let Some(d) = stack.pop() {
        if n > 2000 {
            break;
        }
        if let Ok(rd) = fs::read_dir(&d) {
            for ent in rd.flatten() {
                n += 1;
                let p = ent.path();
                if p.is_dir() {
                    stack.push(p);
                } else if let Ok(m) = ent.metadata() {
                    total = total.saturating_add(m.len());
                }
            }
        }
    }
    total
}

fn display_name(raw: &str) -> String {
    // Explorer sometimes shows URL-encoded folder names literally
    raw.replace("%20", " ")
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

                // Unity / APPX extracted folders
                if path.is_dir() && forced.is_none() && looks_like_unity_folder(&path) {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let name = display_name(&name);
                    if !seen.insert(name.clone()) {
                        continue;
                    }
                    let size_bytes = dir_size_approx(&path);
                    apps.push(AppInfo {
                        name,
                        path,
                        format: AppFormat::UnityFolder,
                        package: None,
                        version: None,
                        architecture: None,
                        has_libunity: false,
                        has_libmain: false,
                        has_mono: false,
                        native_libs: Vec::new(),
                        size_bytes,
                    });
                    continue;
                }

                if !path.is_file() {
                    continue;
                }

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let format = match (forced, ext.as_str()) {
                    (_, "apk") => AppFormat::Apk,
                    (_, "ipa") => AppFormat::Ipa,
                    (_, "appx") | (_, "msix") | (_, "appxbundle") => AppFormat::UnityFolder,
                    (Some(f), _) if path.is_file() => *f,
                    _ => continue,
                };
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let name = display_name(&name);
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
                info.package = summary.bundle_id;
                info.version = summary.bundle_version;
            }
            Err(e) => eprintln!("[ERROR] Failed to open IPA: {}", e),
        },
        AppFormat::UnityFolder => {
            let report = inspect_pangoland_target(Some(&info.path));
            info.has_libunity = report.has_unity_player;
            info.has_mono = report.has_assembly_csharp || report.has_managed;
            info.architecture = report.architecture_hint;
            info.version = report.unity_version_hint;
            if report.has_touchscript {
                info.native_libs.push("TouchScript.dll".into());
            }
            if report.has_unity_player {
                info.native_libs.push("UnityPlayer.dll".into());
            }
        }
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
        AppFormat::UnityFolder => {
            if app.path.is_dir() {
                Ok(format!(
                    "[CACHE] Unity/APPX folder already extracted on disk: {}",
                    app.path.display()
                ))
            } else {
                Err("Unity package file extract not implemented — extract AppxBundle manually first".into())
            }
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
                if let Some(ref b) = summary.bundle_id {
                    out.push_str(&format!("  Bundle ID: {}\n", b));
                }
                if let Some(ref e) = summary.executable {
                    out.push_str(&format!("  Executable: {}\n", e));
                }
                out.push_str(
                    "  Note: Inspect only — Launch will report UnsupportedRuntime until a native iOS guest backend exists.\n",
                );
                loader.print_structure();
            }
            Err(e) => out.push_str(&format!("[ERROR] {}\n", e)),
        },
        AppFormat::UnityFolder => {
            out.push_str(&format!("[Unity folder] {}\n", app.name));
            out.push_str(&format!("  Path: {}\n", app.path.display()));
            let report = inspect_pangoland_target(Some(&app.path));
            out.push_str(&report.to_text());
        }
        AppFormat::Unknown => out.push_str("[ERROR] Unknown format\n"),
    }
    out
}

pub fn run_app(app: &AppInfo) -> Result<String, String> {
    let app_name = app
        .name
        .trim_end_matches(".apk")
        .trim_end_matches(".ipa");

    match app.format {
        AppFormat::UnityFolder => {
            let report = inspect_pangoland_target(Some(&app.path));
            let mut log = format!("[Launch] {}\n", app.name);
            log.push_str("[Analyze] Unity/APPX folder target\n");
            log.push_str(&format!(
                "[Analyze] UnityPlayer: {} | Assembly-CSharp: {} | TouchScript: {}\n",
                report.has_unity_player, report.has_assembly_csharp, report.has_touchscript
            ));
            log.push_str("[Runtime] Unity guest execution for Windows APPX/folder: not implemented yet\n");
            log.push_str("[Result] UnsupportedRuntime\n");
            log.push_str("[Details] Package was inspected; no guest code was executed.\n");
            Ok(log)
        }
        AppFormat::Unknown => return Err("Unknown format".into()),
        AppFormat::Apk | AppFormat::Ipa => {
            let platform = match app.format {
                AppFormat::Apk => "android",
                AppFormat::Ipa => "ios",
                _ => unreachable!(),
            };
            let cache_mgr = CacheManager::new("cache");
            if !cache_mgr.has_valid_cache(app_name) {
                let _ = extract_app(app);
            }
            let mut session = GameSession::new(app_name, &app.path, platform);
            match session.run() {
                Ok(()) => Ok(session.log),
                Err(e) => {
                    if session.log.is_empty() {
                        Err(e)
                    } else {
                        Ok(format!("{}\n{}", session.log.trim_end(), e))
                    }
                }
            }
        }
    }
}

pub fn path_exists(p: &str) -> bool {
    Path::new(p).exists()
}
