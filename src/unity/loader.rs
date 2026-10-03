//! Future: load Unity data files, assets, and map native entry points.

pub struct UnityLoader;

impl UnityLoader {
    pub fn detect_unity_version(_assets_path: &std::path::Path) -> Option<String> {
        None
    }
}
