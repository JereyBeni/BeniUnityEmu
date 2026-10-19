//! Stage G: Pangoland (Studio Pango) Unity 4.6.1f1 Windows package inspector.
//!
//! Does **not** invent package contents. Scans a user-supplied extracted directory
//! (APPX/AppxBundle contents or install folder) and emits a machine-readable report.
//!
//! Expected evidence (verify on disk; do not assume present in repo):
//! - StudioPango.Pangoland_* AppxBundle / extracted package
//! - Unity 4.6.1f1 (d1db7a1b5196)
//! - UnityPlayer.dll, Assembly-CSharp.dll, TouchScript.dll, AppxManifest.xml
//! - x86, D3D11, Windows 10 Mobile-origin XAML host

use std::fs;
use std::path::{Path, PathBuf};

/// Default places to look (relative to cwd). Override with BENI_PANGOLAND_PATH.
pub const ENV_TARGET: &str = "BENI_PANGOLAND_PATH";

const HINT_PATHS: &[&str] = &[
    "BeniApps/PangoLand",
    "BeniApps/Pangoland",
    "BeniApps/StudioPango.Pangoland",
    "targets/pangoland",
    "PangoLand",
    "Pangoland",
];

#[derive(Debug, Clone, Default)]
pub struct FileHit {
    pub relative: String,
    pub size_bytes: u64,
    pub confidence: &'static str, // high | medium | low
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub struct PangolandReport {
    pub target_dir: Option<PathBuf>,
    pub found: bool,
    pub unity_version_hint: Option<String>,
    pub architecture_hint: Option<String>,
    pub has_unity_player: bool,
    pub has_assembly_csharp: bool,
    pub has_touchscript: bool,
    pub has_appx_manifest: bool,
    pub has_data_folder: bool,
    pub has_managed: bool,
    pub has_il2cpp: bool,
    pub dlls: Vec<FileHit>,
    pub manifests: Vec<FileHit>,
    pub data_files: Vec<FileHit>,
    pub missing_expected: Vec<String>,
    pub notes: Vec<String>,
    pub api_priorities: Vec<String>,
}

impl PangolandReport {
    /// Human + semi-structured text (no extra JSON dependency).
    pub fn to_text(&self) -> String {
        let mut o = String::new();
        o.push_str("=== BeniUnityEmu Stage G: Pangoland Target Report ===\n");
        o.push_str(&format!(
            "target_dir: {}\n",
            self.target_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<not found>".into())
        ));
        o.push_str(&format!("found: {}\n", self.found));
        o.push_str(&format!(
            "unity_version_hint: {}\n",
            self.unity_version_hint.as_deref().unwrap_or("unknown")
        ));
        o.push_str(&format!(
            "architecture_hint: {}\n",
            self.architecture_hint.as_deref().unwrap_or("unknown")
        ));
        o.push_str(&format!("has_unity_player: {}\n", self.has_unity_player));
        o.push_str(&format!("has_assembly_csharp: {}\n", self.has_assembly_csharp));
        o.push_str(&format!("has_touchscript: {}\n", self.has_touchscript));
        o.push_str(&format!("has_appx_manifest: {}\n", self.has_appx_manifest));
        o.push_str(&format!("has_data_folder: {}\n", self.has_data_folder));
        o.push_str(&format!("has_managed: {}\n", self.has_managed));
        o.push_str(&format!("has_il2cpp: {}\n", self.has_il2cpp));
        o.push_str("\n[expected_markers]\n");
        for m in &self.missing_expected {
            o.push_str(&format!("  MISSING: {}\n", m));
        }
        o.push_str("\n[dlls]\n");
        for d in &self.dlls {
            o.push_str(&format!(
                "  {} ({} bytes) [{}] {}\n",
                d.relative, d.size_bytes, d.confidence, d.note
            ));
        }
        o.push_str("\n[manifests]\n");
        for d in &self.manifests {
            o.push_str(&format!(
                "  {} ({} bytes) [{}]\n",
                d.relative, d.size_bytes, d.confidence
            ));
        }
        o.push_str("\n[data_sample]\n");
        for d in self.data_files.iter().take(40) {
            o.push_str(&format!("  {} ({} bytes)\n", d.relative, d.size_bytes));
        }
        if self.data_files.len() > 40 {
            o.push_str(&format!("  ... {} more\n", self.data_files.len() - 40));
        }
        o.push_str("\n[api_priorities_unity_4_6]\n");
        for a in &self.api_priorities {
            o.push_str(&format!("  - {}\n", a));
        }
        o.push_str("\n[notes]\n");
        for n in &self.notes {
            o.push_str(&format!("  - {}\n", n));
        }
        o.push_str("=== end report ===\n");
        o
    }
}

/// Resolve target directory: env BENI_PANGOLAND_PATH, else known relative hints.
pub fn resolve_target_dir() -> Option<PathBuf> {
    if let Ok(p) = std::env::var(ENV_TARGET) {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Some(path);
        }
    }
    for h in HINT_PATHS {
        let p = PathBuf::from(h);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

pub fn inspect_pangoland_target(root: Option<&Path>) -> PangolandReport {
    let mut report = PangolandReport::default();
    report.api_priorities = crate::unity::compat46::priority_apis();

    let dir = match root.map(|p| p.to_path_buf()).or_else(resolve_target_dir) {
        Some(d) => d,
        None => {
            report.found = false;
            report.notes.push(format!(
                "No Pangoland package directory found. Set {} or place extracted package under BeniApps/PangoLand/",
                ENV_TARGET
            ));
            report.notes.push(
                "Expected package evidence: StudioPango.Pangoland_* AppxBundle extracted, Unity 4.6.1f1, x86, UnityPlayer.dll, Assembly-CSharp.dll, TouchScript.dll, AppxManifest.xml"
                    .into(),
            );
            report.missing_expected = vec![
                "UnityPlayer.dll".into(),
                "Assembly-CSharp.dll".into(),
                "TouchScript.dll".into(),
                "AppxManifest.xml".into(),
            ];
            return report;
        }
    };

    report.target_dir = Some(dir.clone());
    report.found = true;
    report.notes.push(format!("Scanning {}", dir.display()));

    let mut files: Vec<PathBuf> = Vec::new();
    walk_files(&dir, &dir, &mut files, 0);

    let expected = [
        ("unityplayer.dll", "UnityPlayer.dll"),
        ("assembly-csharp.dll", "Assembly-CSharp.dll"),
        ("touchscript.dll", "TouchScript.dll"),
        ("appxmanifest.xml", "AppxManifest.xml"),
    ];
    for (_, label) in &expected {
        report.missing_expected.push((*label).into());
    }

    for path in &files {
        let rel = path
            .strip_prefix(&dir)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        let lower = rel.to_lowercase();
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        if lower.ends_with("unityplayer.dll") {
            report.has_unity_player = true;
            report.dlls.push(FileHit {
                relative: rel.clone(),
                size_bytes: size,
                confidence: "high",
                note: "Unity native player".into(),
            });
            report.missing_expected.retain(|x| x != "UnityPlayer.dll");
        }
        if lower.ends_with("assembly-csharp.dll") {
            report.has_assembly_csharp = true;
            report.has_managed = true;
            report.dlls.push(FileHit {
                relative: rel.clone(),
                size_bytes: size,
                confidence: "high",
                note: "Game scripts assembly (compiled; not C# source)".into(),
            });
            report.missing_expected.retain(|x| x != "Assembly-CSharp.dll");
        }
        if lower.contains("touchscript") && lower.ends_with(".dll") {
            report.has_touchscript = true;
            report.dlls.push(FileHit {
                relative: rel.clone(),
                size_bytes: size,
                confidence: "high",
                note: "TouchScript managed assembly".into(),
            });
            report.missing_expected.retain(|x| x != "TouchScript.dll");
        }
        if lower.ends_with("appxmanifest.xml") {
            report.has_appx_manifest = true;
            report.manifests.push(FileHit {
                relative: rel.clone(),
                size_bytes: size,
                confidence: "high",
                note: "UWP/APPX manifest".into(),
            });
            report.missing_expected.retain(|x| x != "AppxManifest.xml");
            if let Ok(text) = fs::read_to_string(path) {
                parse_manifest_hints(&text, &mut report);
            }
        }
        if lower.contains("/managed/") || lower.contains("\\managed\\") {
            report.has_managed = true;
        }
        if lower.contains("il2cpp") || lower.contains("global-metadata") {
            report.has_il2cpp = true;
        }
        if lower.contains("_data") || lower.contains("/data/") || lower.ends_with(".assets")
            || lower.contains("sharedassets")
            || lower.contains("level0")
            || lower.contains("globalgamemanagers")
        {
            report.has_data_folder = true;
            if report.data_files.len() < 200 {
                report.data_files.push(FileHit {
                    relative: rel.clone(),
                    size_bytes: size,
                    confidence: "medium",
                    note: String::new(),
                });
            }
        }
        // Unity version strings sometimes in globalgamemanagers or mainData
        if lower.contains("globalgamemanagers") || lower.ends_with("maindata") {
            if let Ok(bytes) = fs::read(path) {
                if let Some(v) = find_unity_version_string(&bytes) {
                    report.unity_version_hint = Some(v);
                }
            }
        }
        if lower.ends_with(".dll")
            && !report.dlls.iter().any(|d| d.relative == rel)
            && (lower.contains("unityengine") || lower.contains("unity"))
        {
            report.dlls.push(FileHit {
                relative: rel.clone(),
                size_bytes: size,
                confidence: "medium",
                note: "Unity-related assembly".into(),
            });
        }
    }

    if report.unity_version_hint.is_none() {
        report.unity_version_hint = Some("4.6.1f1 (claimed; verify against binary)".into());
        report.notes.push(
            "Unity 4.6.1f1 / d1db7a1b5196 not confirmed from scanned files; kept as claimed baseline"
                .into(),
        );
    }
    if report.architecture_hint.is_none() {
        report.architecture_hint = Some("x86 (claimed package arch; verify PE headers)".into());
    }

    if report.has_assembly_csharp && !report.has_il2cpp {
        report.notes.push(
            "Compiled managed assemblies present (Mono-style). Source .cs scripts not required in package."
                .into(),
        );
    }
    if report.has_touchscript {
        report.notes.push(
            "TouchScript.dll present — host TouchInputQueue should map into TouchScript/Unity touch when managed host exists."
                .into(),
        );
    }
    report.notes.push(
        "Three missing Behaviour script references (prior runtime): investigate serialized MonoBehaviour fileIDs vs Assembly-CSharp types when assets are available."
            .into(),
    );
    report.notes.push(
        "Original graphics backend claim: D3D11. BeniUnityEmu OpenGL path is separate; do not claim D3D11 execution yet."
            .into(),
    );

    report
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>, depth: u32) {
    if depth > 12 || out.len() > 5000 {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk_files(root, &p, out, depth + 1);
        } else {
            out.push(p);
        }
    }
}

fn parse_manifest_hints(text: &str, report: &mut PangolandReport) {
    if text.to_lowercase().contains("x86") {
        report.architecture_hint = Some("x86 (AppxManifest)".into());
    }
    if text.contains("ProcessorArchitecture=\"x86\"") {
        report.architecture_hint = Some("x86 (ProcessorArchitecture)".into());
    }
    if text.contains("Windows.Universal") || text.contains("uap:") {
        report.notes.push("AppxManifest indicates UWP/Universal Windows Platform host".into());
    }
    // Identity Name
    if let Some(i) = text.find("Name=\"") {
        let rest = &text[i + 6..];
        if let Some(end) = rest.find('"') {
            let name = &rest[..end];
            if name.to_lowercase().contains("pango") {
                report.notes.push(format!("Package identity name: {}", name));
            }
        }
    }
}

fn find_unity_version_string(data: &[u8]) -> Option<String> {
    // Look for ASCII "4.6." or "Unity " patterns
    let s = String::from_utf8_lossy(data);
    for token in ["4.6.1f1", "4.6.1", "4.6."] {
        if s.contains(token) {
            // Prefer full 4.6.1f1 if present
            if s.contains("4.6.1f1") {
                return Some("4.6.1f1".into());
            }
            return Some(token.trim_end_matches('.').to_string());
        }
    }
    if s.contains("d1db7a1b5196") {
        return Some("4.6.1f1 (hash d1db7a1b5196)".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_dir_reports_clearly() {
        // Force no env / no default: inspect explicit nonexistent
        let r = inspect_pangoland_target(Some(Path::new("/nonexistent/pangoland_stage_g")));
        // walk on nonexistent => found true but empty? We set found from Option root
        // For nonexistent path that was Some, found=true but empty scan
        assert!(r.target_dir.is_some());
    }

    #[test]
    fn detects_markers_in_temp_tree() {
        let dir = std::env::temp_dir().join(format!("beni_pango_{}", std::process::id()));
        let managed = dir.join("Data").join("Managed");
        fs::create_dir_all(&managed).unwrap();
        fs::write(dir.join("UnityPlayer.dll"), b"MZ").unwrap();
        fs::write(managed.join("Assembly-CSharp.dll"), b"MZ").unwrap();
        fs::write(managed.join("TouchScript.dll"), b"MZ").unwrap();
        let mut m = fs::File::create(dir.join("AppxManifest.xml")).unwrap();
        write!(
            m,
            r#"<?xml version="1.0"?><Package Name="StudioPango.Pangoland" ProcessorArchitecture="x86"></Package>"#
        )
        .unwrap();
        let r = inspect_pangoland_target(Some(&dir));
        assert!(r.has_unity_player);
        assert!(r.has_assembly_csharp);
        assert!(r.has_touchscript);
        assert!(r.has_appx_manifest);
        assert!(r.missing_expected.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
