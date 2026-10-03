//! Cache manager - per-app directories under ./cache/

use std::fs;
use std::path::PathBuf;

use crate::apk::extractor;
use crate::apk::loader::ApkLoader;
use crate::ipa::loader::IpaLoader;

#[derive(Debug)]
pub struct CacheMetadata {
    pub name: String,
    pub package: Option<String>,
    pub version: Option<String>,
    pub architecture: Option<String>,
    pub has_libunity: bool,
    pub has_libmain: bool,
    pub has_mono: bool,
    pub format: String,
}

impl CacheMetadata {
    fn to_json(&self) -> String {
        format!(
            r#"{{
  "name": "{}",
  "package": {},
  "version": {},
  "architecture": {},
  "has_libunity": {},
  "has_libmain": {},
  "has_mono": {},
  "format": "{}"
}}"#,
            self.name,
            opt_str(&self.package),
            opt_str(&self.version),
            opt_str(&self.architecture),
            self.has_libunity,
            self.has_libmain,
            self.has_mono,
            self.format
        )
    }
}

fn opt_str(v: &Option<String>) -> String {
    match v {
        Some(s) => format!("\"{}\"", s),
        None => "null".to_string(),
    }
}

pub struct CacheManager {
    root: PathBuf,
}

impl CacheManager {
    pub fn new(root: &str) -> Self {
        let root = PathBuf::from(root);
        if !root.exists() {
            let _ = fs::create_dir_all(&root);
        }
        CacheManager { root }
    }

    pub fn app_dir(&self, app_name: &str) -> PathBuf {
        self.root.join(app_name)
    }

    pub fn has_valid_cache(&self, app_name: &str) -> bool {
        let dir = self.app_dir(app_name);
        let meta = dir.join("metadata.json");
        let extracted = dir.join("extracted");
        meta.exists() && extracted.exists()
    }

    pub fn extract_apk(&self, app_name: &str, loader: &ApkLoader) -> Result<CacheMetadata, String> {
        let app_dir = self.app_dir(app_name);
        let extracted = app_dir.join("extracted");
        fs::create_dir_all(&extracted).map_err(|e| format!("Cannot create cache dir: {}", e))?;

        let count = extractor::extract_required(loader, &extracted)?;
        println!("[CACHE] Total extracted files: {}", count);

        let summary = loader.summarize();

        let meta = CacheMetadata {
            name: app_name.to_string(),
            package: None,
            version: None,
            architecture: summary.preferred_arch.clone(),
            has_libunity: summary.has_libunity,
            has_libmain: summary.has_libmain,
            has_mono: summary.has_mono,
            format: "apk".to_string(),
        };

        let meta_path = app_dir.join("metadata.json");
        fs::write(&meta_path, meta.to_json()).map_err(|e| format!("Cannot write metadata: {}", e))?;

        Ok(meta)
    }

    pub fn extract_ipa(&self, app_name: &str, loader: &IpaLoader) -> Result<CacheMetadata, String> {
        let app_dir = self.app_dir(app_name);
        let extracted = app_dir.join("extracted");
        fs::create_dir_all(&extracted).map_err(|e| format!("Cannot create cache dir: {}", e))?;

        match loader.extract_prefix("Payload/", &extracted.join("Payload")) {
            Ok(n) => println!("[CACHE] Extracted {} files from Payload/", n),
            Err(e) => println!("[CACHE] Payload extraction note: {}", e),
        }

        let summary = loader.summarize();

        let meta = CacheMetadata {
            name: app_name.to_string(),
            package: None,
            version: None,
            architecture: summary.preferred_arch.clone(),
            has_libunity: summary.native_libs.iter().any(|l| l.contains("unity")),
            has_libmain: false,
            has_mono: false,
            format: "ipa".to_string(),
        };

        let meta_path = app_dir.join("metadata.json");
        fs::write(&meta_path, meta.to_json()).map_err(|e| format!("Cannot write metadata: {}", e))?;

        Ok(meta)
    }
}
