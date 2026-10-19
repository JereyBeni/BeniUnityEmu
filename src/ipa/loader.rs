//! IPA loader – ZIP-based inspection of Payload/*.app bundles.

use std::fs::File;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::ipa::macho::{inspect_macho_arches, MachOArch};
use crate::ziputil::{
    extract_file_to, extract_prefix_to, has_entry, list_entries, open_archive, read_entry_bytes,
};

#[derive(Debug, Clone, Default)]
pub struct IpaSummary {
    pub file_count: usize,
    pub preferred_arch: Option<String>,
    pub bundle_id: Option<String>,
    pub bundle_version: Option<String>,
    pub executable: Option<String>,
    pub min_ios: Option<String>,
    pub has_unity: bool,
    /// Heuristic: path contains TouchScript
    pub has_touchscript: bool,
    /// Heuristic: il2cpp / global-metadata
    pub has_il2cpp: bool,
    /// Heuristic: Assembly-CSharp / Managed folder (Mono-style)
    pub has_mono_assemblies: bool,
    pub native_libs: Vec<String>,
    pub entries: Vec<String>,
    pub app_bundle: Option<String>,
}

pub struct IpaLoader {
    path: PathBuf,
}

impl IpaLoader {
    pub fn open(path: &Path) -> Result<Self, String> {
        let _ = open_archive(path)?;
        Ok(IpaLoader {
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn with_archive<T>(&self, f: impl FnOnce(&mut ZipArchive<File>) -> T) -> Result<T, String> {
        let mut archive = open_archive(&self.path)?;
        Ok(f(&mut archive))
    }

    pub fn has_file(&self, name: &str) -> bool {
        self.with_archive(|a| has_entry(a, name)).unwrap_or(false)
    }

    pub fn extract_file(&self, name: &str, dest: &Path) -> Result<(), String> {
        self.with_archive(|a| extract_file_to(a, name, dest))?
    }

    pub fn extract_prefix(&self, prefix: &str, dest: &Path) -> Result<usize, String> {
        self.with_archive(|a| extract_prefix_to(a, prefix, dest))?
    }

    pub fn find_app_bundle(&self) -> Option<String> {
        let entries = self.with_archive(|a| list_entries(a)).ok()?;
        for name in entries {
            if name.starts_with("Payload/") && name.ends_with(".app/") {
                return Some(name);
            }
            if name.starts_with("Payload/") && name.ends_with(".app") {
                return Some(format!("{}/", name));
            }
            if name.starts_with("Payload/") && name.ends_with("/Info.plist") {
                if let Some(prefix) = name.strip_suffix("Info.plist") {
                    return Some(prefix.to_string());
                }
            }
        }
        None
    }

    pub fn summarize(&self) -> IpaSummary {
        let mut summary = IpaSummary::default();
        let Ok(entries) = self.with_archive(|a| list_entries(a)) else {
            return summary;
        };
        summary.file_count = entries.len();
        summary.entries = entries.clone();
        summary.app_bundle = self.find_app_bundle();

        for name in &entries {
            let lower = name.to_lowercase();
            if lower.contains("unity")
                && (lower.ends_with(".dylib")
                    || lower.ends_with(".framework/")
                    || lower.contains("unityframework")
                    || lower.contains("data/raw")
                    || lower.contains("data.unity3d")
                    || lower.contains("globalgamemanagers"))
            {
                summary.has_unity = true;
            }
            if lower.contains("touchscript") {
                summary.has_touchscript = true;
                summary.has_unity = true;
            }
            if lower.contains("il2cpp")
                || lower.contains("global-metadata.dat")
                || lower.contains("libil2cpp")
            {
                summary.has_il2cpp = true;
            }
            if lower.contains("assembly-csharp")
                || lower.ends_with(".dll")
                    && (lower.contains("/managed/") || lower.contains("data/managed"))
            {
                summary.has_mono_assemblies = true;
            }
            if lower.ends_with(".dylib") || lower.ends_with(".so") {
                summary.native_libs.push(name.clone());
            }
        }

        if let Some(ref bundle) = summary.app_bundle {
            let plist_name = format!("{}Info.plist", bundle);
            if let Ok(Ok(bytes)) = self.with_archive(|a| read_entry_bytes(a, &plist_name)) {
                if let Ok(text) = String::from_utf8(bytes) {
                    summary.bundle_id = plist_string_value(&text, "CFBundleIdentifier");
                    summary.bundle_version = plist_string_value(&text, "CFBundleShortVersionString")
                        .or_else(|| plist_string_value(&text, "CFBundleVersion"));
                    summary.executable = plist_string_value(&text, "CFBundleExecutable");
                    summary.min_ios = plist_string_value(&text, "MinimumOSVersion");
                }
            }
            if let Some(ref exe) = summary.executable {
                let exe_path = format!("{}{}", bundle, exe);
                if let Ok(Ok(bytes)) = self.with_archive(|a| read_entry_bytes(a, &exe_path)) {
                    if let Ok(arches) = inspect_macho_arches(&bytes) {
                        summary.preferred_arch = prefer_macho_arch(&arches);
                    }
                }
            }
        }
        if summary.preferred_arch.is_none() {
            summary.preferred_arch = Some("arm64".into());
        }
        summary
    }

    pub fn print_structure(&self) {
        let summary = self.summarize();
        println!("[IPA] structure for {}", self.path.display());
        println!("  files:  {}", summary.file_count);
        println!("  bundle: {:?}", summary.app_bundle);
        println!("  arch:   {:?}", summary.preferred_arch);
        println!("  unity:  {}", summary.has_unity);
        println!("  touchscript: {}", summary.has_touchscript);
        println!("  il2cpp: {}", summary.has_il2cpp);
        println!("  mono assemblies: {}", summary.has_mono_assemblies);
        for e in summary.entries.iter().take(40) {
            println!("    {}", e);
        }
    }
}

fn plist_string_value(text: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    let idx = text.find(&key_tag)?;
    let after = &text[idx + key_tag.len()..];
    let s_idx = after.find("<string>")?;
    let rest = &after[s_idx + 8..];
    let end = rest.find("</string>")?;
    let val = rest[..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

fn prefer_macho_arch(arches: &[MachOArch]) -> Option<String> {
    for a in arches {
        if matches!(a, MachOArch::Arm64) {
            return Some("arm64".into());
        }
    }
    for a in arches {
        if matches!(a, MachOArch::Arm) {
            return Some("armv7".into());
        }
    }
    arches.first().map(|a| a.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ziputil::sanitize_entry_name;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use zip::write::FileOptions;
    use zip::ZipWriter;

    static IPA_TEST_SEQ: AtomicU64 = AtomicU64::new(0);

    fn make_test_ipa() -> PathBuf {
        let n = IPA_TEST_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "beni_ipa_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.ipa");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("Payload/TestApp.app/Info.plist", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.example.testapp</string>
<key>CFBundleShortVersionString</key><string>1.2.3</string>
<key>CFBundleExecutable</key><string>TestApp</string>
<key>MinimumOSVersion</key><string>9.0</string>
</dict></plist>"#,
        )
        .unwrap();
        let mut macho = vec![0u8; 32];
        macho[0..4].copy_from_slice(&0xfeed_facfu32.to_le_bytes());
        macho[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
        zip.start_file("Payload/TestApp.app/TestApp", opts).unwrap();
        zip.write_all(&macho).unwrap();
        zip.start_file(
            "Payload/TestApp.app/Data/Managed/TouchScript.dll",
            opts,
        )
        .unwrap();
        zip.write_all(b"MZ-fake").unwrap();
        zip.finish().unwrap();
        path
    }

    #[test]
    fn open_and_find_bundle() {
        let path = make_test_ipa();
        let loader = IpaLoader::open(&path).unwrap();
        assert!(loader.find_app_bundle().is_some());
    }

    #[test]
    fn summarize_reads_plist() {
        let path = make_test_ipa();
        let loader = IpaLoader::open(&path).unwrap();
        let s = loader.summarize();
        assert!(s.file_count >= 2);
        assert_eq!(s.bundle_id.as_deref(), Some("com.example.testapp"));
        assert_eq!(s.bundle_version.as_deref(), Some("1.2.3"));
        assert_eq!(s.executable.as_deref(), Some("TestApp"));
        assert_eq!(s.min_ios.as_deref(), Some("9.0"));
        assert_eq!(s.preferred_arch.as_deref(), Some("arm64"));
        assert!(s.has_touchscript);
        assert!(s.has_mono_assemblies);
    }

    #[test]
    fn path_traversal_rejected() {
        let path = make_test_ipa();
        let loader = IpaLoader::open(&path);
        assert!(loader.is_ok());
        assert!(sanitize_entry_name("../evil").is_none());
    }
}
