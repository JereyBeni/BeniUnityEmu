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
}

impl ApkLoader {
    pub fn open(path: &Path) -> Result<Self, String> {
        // Validate archive opens
        let _ = open_archive(path)?;
        Ok(ApkLoader {
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

    pub fn summarize(&self) -> ApkSummary {
        let mut summary = ApkSummary::default();
        let Ok(entries) = self.with_archive(|a| list_entries(a)) else {
            return summary;
        };
        summary.file_count = entries.len();
        summary.entries = entries.clone();

        let mut archs: Vec<String> = Vec::new();
        for name in &entries {
            let lower = name.to_lowercase();
            if lower.contains("libunity") {
                summary.has_libunity = true;
            }
            if lower.contains("libmain") || lower.ends_with("/libmain.so") {
                summary.has_libmain = true;
            }
            if lower.contains("libmono") || (lower.contains("mono") && lower.ends_with(".so")) {
                summary.has_mono = true;
            }
            if name.starts_with("lib/") && name.ends_with(".so") {
                summary.native_libs.push(name.clone());
                if let Some(rest) = name.strip_prefix("lib/") {
                    if let Some(abi) = rest.split('/').next() {
                        if !abi.is_empty() && !archs.iter().any(|a| a == abi) {
                            archs.push(abi.to_string());
                        }
                    }
                }
            }
        }
        summary.preferred_arch = prefer_android_arch(&archs);
        summary
    }

    pub fn print_structure(&self) {
        let summary = self.summarize();
        println!("[APK] structure for {}", self.path.display());
        println!("  files: {}", summary.file_count);
        println!("  arch:  {:?}", summary.preferred_arch);
        for e in summary.entries.iter().take(40) {
            println!("    {}", e);
        }
    }

    pub fn read_bytes(&self, name: &str) -> Result<Vec<u8>, String> {
        self.with_archive(|a| read_entry_bytes(a, name))?
    }
}

fn prefer_android_arch(archs: &[String]) -> Option<String> {
    const ORDER: &[&str] = &["arm64-v8a", "armeabi-v7a", "armeabi", "x86_64", "x86"];
    for pref in ORDER {
        if archs.iter().any(|a| a == *pref) {
            return Some((*pref).to_string());
        }
    }
    archs.first().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    fn make_test_apk() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("beni_apk_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.apk");
        let file = File::create(&path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("AndroidManifest.xml", opts).unwrap();
        zip.write_all(b"<manifest/>").unwrap();
        zip.start_file("lib/arm64-v8a/libunity.so", opts).unwrap();
        zip.write_all(&[0u8; 16]).unwrap();
        zip.start_file("lib/armeabi-v7a/libmain.so", opts).unwrap();
        zip.write_all(&[0u8; 8]).unwrap();
        zip.finish().unwrap();
        path
    }

    #[test]
    fn open_and_summarize() {
        let path = make_test_apk();
        let loader = ApkLoader::open(&path).unwrap();
        let s = loader.summarize();
        assert!(s.file_count >= 3);
        assert!(s.has_libunity);
        assert!(s.has_libmain);
        assert_eq!(s.preferred_arch.as_deref(), Some("arm64-v8a"));
    }

    #[test]
    fn rejects_nonexistent() {
        assert!(ApkLoader::open(Path::new("/nonexistent/no.apk")).is_err());
    }
}
