//! IPA loader – ZIP-based inspection of iOS application bundles.

use std::fs::File;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::ipa::macho::{inspect_macho_arches, MachOArch};
use crate::ziputil::{
    extract_file_to, extract_prefix_to, has_entry, list_entries, open_archive, read_entry_bytes,
    sanitize_entry_name,
};

#[derive(Debug, Clone, Default)]
pub struct IpaSummary {
    pub file_count: usize,
    pub preferred_arch: Option<String>,
    pub bundle_id: Option<String>,
    pub bundle_version: Option<String>,
    pub executable: Option<String>,
    pub min_ios: Option<String>,
    pub app_bundle_path: Option<String>,
    pub native_libs: Vec<String>,
    pub has_unity: bool,
    pub entries: Vec<String>,
}

pub struct IpaLoader {
    path: PathBuf,
    archive: ZipArchive<File>,
}

impl IpaLoader {
    pub fn open(path: &Path) -> Result<Self, String> {
        let archive = open_archive(path)?;
        Ok(IpaLoader {
            path: path.to_path_buf(),
            archive,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn has_file(&mut self, name: &str) -> bool {
        has_entry(&mut self.archive, name)
    }

    pub fn extract_file(&mut self, name: &str, dest: &Path) -> Result<(), String> {
        extract_file_to(&mut self.archive, name, dest)
    }

    pub fn extract_prefix(&mut self, prefix: &str, dest: &Path) -> Result<usize, String> {
        extract_prefix_to(&mut self.archive, prefix, dest)
    }

    pub fn find_app_bundle(&mut self) -> Option<String> {
        let entries = list_entries(&mut self.archive);
        for name in entries {
            if let Some(rest) = name.strip_prefix("Payload/") {
                if rest.ends_with(".app/") || (rest.ends_with(".app") && !rest.contains('/')) {
                    let bundle = if rest.ends_with('/') {
                        rest.to_string()
                    } else {
                        format!("{}/", rest)
                    };
                    return Some(format!("Payload/{}", bundle));
                }
                if let Some(app_part) = rest.split('/').next() {
                    if app_part.ends_with(".app") {
                        return Some(format!("Payload/{}/", app_part));
                    }
                }
            }
        }
        None
    }

    pub fn summarize(&mut self) -> IpaSummary {
        let entries = list_entries(&mut self.archive);
        let file_count = entries.len();
        let app_bundle = self.find_app_bundle();

        let mut native_libs = Vec::new();
        let mut has_unity = false;
        let mut arches: Vec<MachOArch> = Vec::new();

        for name in &entries {
            let lower = name.to_lowercase();
            if lower.contains("unity")
                && (lower.ends_with(".dylib") || lower.contains("framework"))
            {
                has_unity = true;
            }
            if name.ends_with(".dylib") || name.contains(".framework/") {
                if sanitize_entry_name(name).is_some() {
                    native_libs.push(name.clone());
                }
            }
        }

        let mut bundle_id = None;
        let mut bundle_version = None;
        let mut executable = None;
        let mut min_ios = None;

        if let Some(ref bundle) = app_bundle {
            let plist_path = format!("{}Info.plist", bundle);
            if let Ok(data) = read_entry_bytes(&mut self.archive, &plist_path) {
                if let Some(s) = extract_plist_string(&data, "CFBundleIdentifier") {
                    bundle_id = Some(s);
                }
                if let Some(s) = extract_plist_string(&data, "CFBundleShortVersionString") {
                    bundle_version = Some(s);
                } else if let Some(s) = extract_plist_string(&data, "CFBundleVersion") {
                    bundle_version = Some(s);
                }
                if let Some(s) = extract_plist_string(&data, "CFBundleExecutable") {
                    executable = Some(s);
                }
                if let Some(s) = extract_plist_string(&data, "MinimumOSVersion") {
                    min_ios = Some(s);
                }
            }

            if let Some(ref exe) = executable {
                let exe_path = format!("{}{}", bundle, exe);
                if let Ok(data) = read_entry_bytes(&mut self.archive, &exe_path) {
                    if let Ok(found) = inspect_macho_arches(&data) {
                        for a in found {
                            if !arches.contains(&a) {
                                arches.push(a);
                            }
                        }
                    }
                }
            }
        }

        let preferred_arch = prefer_ios_arch(&arches);

        IpaSummary {
            file_count,
            preferred_arch,
            bundle_id,
            bundle_version,
            executable,
            min_ios,
            app_bundle_path: app_bundle,
            native_libs,
            has_unity,
            entries,
        }
    }

    pub fn print_structure(&mut self) {
        let summary = self.summarize();
        println!("[IPA] Structure of {:?}", self.path);
        println!("  Total entries: {}", summary.file_count);
        if let Some(ref b) = summary.app_bundle_path {
            println!("  App bundle: {}", b);
        }
        if let Some(ref id) = summary.bundle_id {
            println!("  Bundle ID: {}", id);
        }
        if let Some(ref a) = summary.preferred_arch {
            println!("  Preferred arch: {}", a);
        }
        println!("  Unity-related: {}", summary.has_unity);
    }
}

fn prefer_ios_arch(arches: &[MachOArch]) -> Option<String> {
    if arches.iter().any(|a| matches!(a, MachOArch::Arm64)) {
        return Some("arm64".into());
    }
    if arches.iter().any(|a| matches!(a, MachOArch::Arm)) {
        return Some("armv7".into());
    }
    arches.first().map(|a| a.as_str().to_string())
}

fn extract_plist_string(data: &[u8], key: &str) -> Option<String> {
    if data.starts_with(b"<?xml")
        || data.starts_with(b"<plist")
        || data.starts_with(b"\xef\xbb\xbf<?xml")
    {
        let text = String::from_utf8_lossy(data);
        let key_tag = format!("<key>{}</key>", key);
        if let Some(pos) = text.find(&key_tag) {
            let after = &text[pos + key_tag.len()..];
            if let Some(start) = after.find("<string>") {
                let rest = &after[start + 8..];
                if let Some(end) = rest.find("</string>") {
                    return Some(rest[..end].trim().to_string());
                }
            }
        }
    }
    None
}
