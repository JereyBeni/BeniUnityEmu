//! AndroidManifest.xml handling.
//! Binary XML parsing is complex; for now we only provide a stub interface.

#[derive(Debug, Default, Clone)]
pub struct ManifestInfo {
    pub package_name: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<i32>,
    pub min_sdk: Option<i32>,
    pub target_sdk: Option<i32>,
    pub application_label: Option<String>,
}

impl ManifestInfo {
    pub fn parse_stub(_data: &[u8]) -> Self {
        println!("[Manifest] Parsing not implemented yet");
        ManifestInfo::default()
    }
}
