//! Explicit renderer capability statuses for Stage C1.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    Implemented,
    Partial,
    Unsupported,
    NotTested,
    NotImplemented,
}

impl CapabilityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityStatus::Implemented => "IMPLEMENTED",
            CapabilityStatus::Partial => "PARTIAL",
            CapabilityStatus::Unsupported => "UNSUPPORTED",
            CapabilityStatus::NotTested => "NOT_TESTED",
            CapabilityStatus::NotImplemented => "NOT_IMPLEMENTED",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RendererCapabilities {
    pub open_window: CapabilityStatus,
    pub opengl_context: CapabilityStatus,
    pub clear_framebuffer: CapabilityStatus,
    pub basic_triangle: CapabilityStatus,
    pub resize_viewport: CapabilityStatus,
    pub shader_compile_report: CapabilityStatus,
    pub texture_loading: CapabilityStatus,
    pub blending: CapabilityStatus,
    pub depth_testing: CapabilityStatus,
    pub framebuffer_objects: CapabilityStatus,
    pub gles_compatibility: CapabilityStatus,
    pub unity_guest_runtime: CapabilityStatus,
}

impl RendererCapabilities {
    pub fn stage_c1_opengl() -> Self {
        RendererCapabilities {
            open_window: CapabilityStatus::Implemented,
            opengl_context: CapabilityStatus::Implemented,
            clear_framebuffer: CapabilityStatus::Implemented,
            basic_triangle: CapabilityStatus::Implemented,
            resize_viewport: CapabilityStatus::Implemented,
            shader_compile_report: CapabilityStatus::Implemented,
            texture_loading: CapabilityStatus::Partial,
            blending: CapabilityStatus::NotTested,
            depth_testing: CapabilityStatus::NotTested,
            framebuffer_objects: CapabilityStatus::Unsupported,
            gles_compatibility: CapabilityStatus::Partial,
            unity_guest_runtime: CapabilityStatus::NotImplemented,
        }
    }

    pub fn stub() -> Self {
        RendererCapabilities {
            open_window: CapabilityStatus::NotImplemented,
            opengl_context: CapabilityStatus::NotImplemented,
            clear_framebuffer: CapabilityStatus::NotImplemented,
            basic_triangle: CapabilityStatus::NotImplemented,
            resize_viewport: CapabilityStatus::NotImplemented,
            shader_compile_report: CapabilityStatus::NotImplemented,
            texture_loading: CapabilityStatus::NotImplemented,
            blending: CapabilityStatus::NotImplemented,
            depth_testing: CapabilityStatus::NotImplemented,
            framebuffer_objects: CapabilityStatus::NotImplemented,
            gles_compatibility: CapabilityStatus::NotImplemented,
            unity_guest_runtime: CapabilityStatus::NotImplemented,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("Open Window:              {}", self.open_window.as_str()),
            format!("OpenGL Context:           {}", self.opengl_context.as_str()),
            format!("Clear Framebuffer:        {}", self.clear_framebuffer.as_str()),
            format!("Basic Triangle:           {}", self.basic_triangle.as_str()),
            format!("Resize Viewport:          {}", self.resize_viewport.as_str()),
            format!("Shader Compile Report:    {}", self.shader_compile_report.as_str()),
            format!("Texture Loading:          {}", self.texture_loading.as_str()),
            format!("Blending:                 {}", self.blending.as_str()),
            format!("Depth Testing:            {}", self.depth_testing.as_str()),
            format!("Framebuffer Objects:      {}", self.framebuffer_objects.as_str()),
            format!("GLES Compatibility:       {}", self.gles_compatibility.as_str()),
            format!("Unity Guest Runtime:      {}", self.unity_guest_runtime.as_str()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_c1_marks_triangle_implemented() {
        let c = RendererCapabilities::stage_c1_opengl();
        assert_eq!(c.basic_triangle, CapabilityStatus::Implemented);
        assert_eq!(c.unity_guest_runtime, CapabilityStatus::NotImplemented);
        assert_eq!(c.texture_loading, CapabilityStatus::Partial);
    }

    #[test]
    fn status_strings_are_stable() {
        assert_eq!(CapabilityStatus::Implemented.as_str(), "IMPLEMENTED");
        assert_eq!(CapabilityStatus::NotImplemented.as_str(), "NOT_IMPLEMENTED");
    }

    #[test]
    fn lines_include_warning_capability() {
        let lines = RendererCapabilities::stage_c1_opengl().lines();
        assert!(lines.iter().any(|l| l.contains("Unity Guest Runtime")));
        assert!(lines.iter().any(|l| l.contains("NOT_IMPLEMENTED")));
    }
}
