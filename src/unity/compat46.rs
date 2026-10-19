//! Unity 4.6.1f1 API compatibility inventory (Pangoland-oriented).
//!
//! Baseline docs: Unity 4.3–4.6 manual era. Do **not** substitute modern Unity APIs.
//! Status values: not_implemented | partial | detected_in_target | verified

#[derive(Debug, Clone)]
pub struct ApiEntry {
    pub name: &'static str,
    pub assembly: &'static str,
    pub status: &'static str,
    pub notes: &'static str,
}

/// Priority APIs for Pangoland / Unity 4.6.1 guest work.
pub fn priority_apis() -> Vec<String> {
    inventory()
        .iter()
        .map(|e| {
            format!(
                "{} [{}] status={} — {}",
                e.name, e.assembly, e.status, e.notes
            )
        })
        .collect()
}

pub fn inventory() -> &'static [ApiEntry] {
    &[
        ApiEntry {
            name: "MonoBehaviour.Awake/Start/Update/FixedUpdate/OnDestroy",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Lifecycle required for any guest script execution",
        },
        ApiEntry {
            name: "GameObject / Transform (position, rotation, parent)",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Core scene graph",
        },
        ApiEntry {
            name: "Object.Instantiate / Destroy",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Runtime object management",
        },
        ApiEntry {
            name: "Input.touches / Touch / TouchPhase",
            assembly: "UnityEngine",
            status: "partial",
            notes: "Host TouchInputQueue exists; guest Input bridge missing",
        },
        ApiEntry {
            name: "Input.GetAxis / GetMouseButton / mousePosition",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Desktop fallback used by many 4.x titles",
        },
        ApiEntry {
            name: "Camera.main / ScreenPointToRay / WorldToScreenPoint",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Needed for touch → world interaction",
        },
        ApiEntry {
            name: "Mesh / MeshFilter / MeshRenderer / Material / Texture2D",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Rendering vertical slice",
        },
        ApiEntry {
            name: "Application / Time.deltaTime / Time.time",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "Frame timing",
        },
        ApiEntry {
            name: "AudioSource / AudioClip",
            assembly: "UnityEngine",
            status: "not_implemented",
            notes: "May be deferred",
        },
        ApiEntry {
            name: "TouchScript.TouchManager / Gestures / Pointers",
            assembly: "TouchScript",
            status: "detected_in_target",
            notes: "DLL claimed in Pangoland; needs managed load + host pointer feed",
        },
        ApiEntry {
            name: "TouchScript.InputSources.StandardInput",
            assembly: "TouchScript",
            status: "not_implemented",
            notes: "Replace or feed via custom IInputSource from host queue",
        },
        ApiEntry {
            name: "Serialized MonoBehaviour script references (fileID/guid)",
            assembly: "assets",
            status: "not_implemented",
            notes: "Three missing Behaviour refs from prior inspection — resolve against Assembly-CSharp",
        },
        ApiEntry {
            name: "UnityPlayer.dll + XAML/UWP lifecycle",
            assembly: "native",
            status: "not_implemented",
            notes: "Windows APPX host; not the same as standalone player main()",
        },
        ApiEntry {
            name: "Direct3D 11 device/swap chain",
            assembly: "native",
            status: "not_implemented",
            notes: "Original backend claim; OpenGL path is separate experiment",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_non_empty() {
        assert!(!inventory().is_empty());
        assert!(!priority_apis().is_empty());
    }
}
