//! APK loader – ZIP-based inspection and safe extraction.

use std::fs::File;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::ziputil::{
    extract_file_to, extract_prefix_to, has_entry, list_entries, open_archive, read_entry_bytes,
};

#[derive(Debug, Clone, Default)]
pub struct ApkSummary {
    pub file_count: usize,
    pub preferred_arch: Option<String>,
    pub has_libunity: bool,
    pub has_libmain: bool,
    pub has_mono: bool,
    pub native_libs: Vec<String>,
    pub entries: Vec<String>,
}

pub struct ApkLoader {
    path: PathBuf,
    archive: ZipArchive<File>,
}

impl ApkLoader {
    pub fn open(path: &Path) -> Result<Self, String> {
        let archive = open_archive(path)?;
        Ok(ApkLoader {
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

    pub fn summarize(&mut self) -> ApkSummary {
        let entries = list_entries(&mut self.archive);
        let file_count = entries.len();

        let mut native_libs = Vec::new();
        let mut has_libunity = false;
        let mut has_libmain = false;
        let mut has_mono = false;
        let mut archs: Vec<String> = Vec::new();

        for name in &entries {
            let lower = name.to_lowercase();
            if lower.contains("libunity") {
                has_libunity = true;
            }
            if lower.contains("libmain") || lower.ends_with("/libmain.so") {
                has_libmain = true;
            }
            if lower.contains("libmono") || (lower.contains("mono") && lower.ends_with(".so")) {
                has_mono = true;
            }
            if name.starts_with("lib/") && name.ends_with(".so") {
                native_libs.push(name.clone());
                if let Some(rest) = name.strip_prefix("lib/") {
                    if let Some(abi) = rest.split('/').next() {
                        if !abi.is_empty() && !archs.iter().any(|a| a == abi) {
                            archs.push(abi.to_string());
                        }
                    }
                }
            }
        }

        let preferred_arch = prefer_android_arch(&archs);

        ApkSummary {
            file_count,
            preferred_arch,
            has_libunity,
            has_libmain,
            has_mono,
            native_libs,
            entries,
        }
    }

    pub fn print_structure(&mut self) {
        let summary = self.summarize();
        println!("[APK] Structure of {:?}", self.path);
        println!("  Total entries: {}", summary.file_count);
        if let Some(ref arch) = summary.preferred_arch {
            println!("  Preferred ABI: {}", arch);
        }
        println!("  libunity: {}", summary.has_libunity);
        println!("  libmain:  {}", summary.has_libmain);
        println!("  mono:     {}", summary.has_mono);
        for e in summary.entries.iter().take(40) {
            println!("    {}", e);
        }
        if summary.entries.len() > 40 {
            println!("    ... ({} more)", summary.entries.len() - 40);
        }
    }

    pub fn read_bytes(&mut self, name: &str) -> Result<Vec<u8>, String> {
        read_entry_bytes(&mut self.archive, name)
    }
}

fn prefer_android_arch(archs: &[String]) -> Option<String> {
    const ORDER: &[&str] = &["arm64-v8a", "armeabi-v7a", "armeabi", "x86_64", "x86"];
    for pref in ORDER {
        if archs.iter().any(|a| a == *pref) {
            return Some(pref.to_string());
        }
    }
    archs.first().cloned()
}
