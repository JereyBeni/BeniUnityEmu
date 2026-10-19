//! Partial desktop OpenGL renderer (Stage C1).
//! Documented target: **OpenGL 3.3 core**. NOT GLES. NOT Unity guest renderer.

#![cfg(feature = "opengl")]

use std::sync::Arc;

use glow::{Context as GlowContext, HasContext};
use glutin::context::PossiblyCurrentContext;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};

use super::capabilities::RendererCapabilities;

const VERT_SRC: &str = r#"#version 330 core
layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec3 a_color;
out vec3 v_color;
void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
    v_color = a_color;
}
"#;

const FRAG_SRC: &str = r#"#version 330 core
in vec3 v_color;
out vec4 frag_color;
void main() {
    frag_color = vec4(v_color, 1.0);
}
"#;

#[derive(Debug, Clone, Default)]
pub struct GlDiagnostics {
    pub version: String,
    pub vendor: String,
    pub renderer: String,
    pub shading_language: String,
    pub context_ready: bool,
    pub last_shader_log: String,
    pub fps: f32,
    pub frame: u64,
}

pub struct PartialGlRenderer {
    pub gl: Arc<GlowContext>,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    pub width: u32,
    pub height: u32,
    pub diagnostics: GlDiagnostics,
    pub capabilities: RendererCapabilities,
}

impl PartialGlRenderer {
    pub unsafe fn new(gl: Arc<GlowContext>, width: u32, height: u32) -> Result<Self, String> {
        let mut diagnostics = GlDiagnostics {
            context_ready: true,
            ..Default::default()
        };
        diagnostics.version = gl.get_parameter_string(glow::VERSION);
        diagnostics.vendor = gl.get_parameter_string(glow::VENDOR);
        diagnostics.renderer = gl.get_parameter_string(glow::RENDERER);
        diagnostics.shading_language = gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION);

        let program = compile_program(&gl, VERT_SRC, FRAG_SRC, &mut diagnostics)?;

        let vertices: [f32; 15] = [
            0.0, 0.6, 0.95, 0.35, 0.25,
            -0.6, -0.5, 0.25, 0.75, 0.45,
            0.6, -0.5, 0.30, 0.45, 0.95,
        ];

        let vao = gl.create_vertex_array().map_err(|e| format!("create_vertex_array: {}", e))?;
        let vbo = gl.create_buffer().map_err(|e| format!("create_buffer: {}", e))?;

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, f32_slice_as_bytes(&vertices), glow::STATIC_DRAW);

        let stride = (5 * std::mem::size_of::<f32>()) as i32;
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, (2 * std::mem::size_of::<f32>()) as i32);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);
        gl.viewport(0, 0, width as i32, height as i32);

        Ok(PartialGlRenderer {
            gl,
            program,
            vao,
            vbo,
            width,
            height,
            diagnostics,
            capabilities: RendererCapabilities::stage_c1_opengl(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        self.width = width;
        self.height = height;
        unsafe { self.gl.viewport(0, 0, width as i32, height as i32); }
    }

    pub fn draw_frame(&mut self, dt: f32) {
        unsafe {
            self.gl.clear_color(0.08, 0.09, 0.12, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.use_program(Some(self.program));
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
            self.gl.bind_vertex_array(None);
            self.gl.use_program(None);
        }
        self.diagnostics.frame += 1;
        if dt > 0.0 {
            let inst = 1.0 / dt;
            if self.diagnostics.fps <= 0.0 {
                self.diagnostics.fps = inst;
            } else {
                self.diagnostics.fps = self.diagnostics.fps * 0.9 + inst * 0.1;
            }
        }
    }

    pub fn diagnostics_report(&self) -> String {
        let mut out = String::new();
        out.push_str("=== BeniUnityEmu OpenGL Runtime [PARTIAL] ===\n");
        out.push_str("WARNING: Unity guest execution is NOT implemented.\n");
        out.push_str("Documented GL target: OpenGL 3.3 core (driver may expose higher).\n\n");
        out.push_str(&format!("OpenGL Context: {}\n", if self.diagnostics.context_ready { "READY" } else { "FAILED" }));
        out.push_str(&format!("OpenGL Version:  {}\n", self.diagnostics.version));
        out.push_str(&format!("GLSL Version:    {}\n", self.diagnostics.shading_language));
        out.push_str(&format!("GPU Vendor:      {}\n", self.diagnostics.vendor));
        out.push_str(&format!("GPU Renderer:    {}\n", self.diagnostics.renderer));
        out.push_str(&format!("Window Size:     {}x{}\n", self.width, self.height));
        out.push_str(&format!("Frame:           {}\n", self.diagnostics.frame));
        out.push_str(&format!("FPS:             {:.1}\n", self.diagnostics.fps));
        if !self.diagnostics.last_shader_log.is_empty() {
            out.push_str(&format!("Shader Log:      {}\n", self.diagnostics.last_shader_log));
        }
        out.push_str("\n--- Capability table ---\n");
        for line in self.capabilities.lines() {
            out.push_str(&line);
            out.push('\n');
        }
        out
    }
}

impl Drop for PartialGlRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

unsafe fn compile_program(gl: &GlowContext, vert_src: &str, frag_src: &str, diag: &mut GlDiagnostics) -> Result<glow::Program, String> {
    let program = gl.create_program().map_err(|e| format!("create_program: {}", e))?;
    let vs = compile_shader(gl, glow::VERTEX_SHADER, vert_src)?;
    let fs = compile_shader(gl, glow::FRAGMENT_SHADER, frag_src)?;
    gl.attach_shader(program, vs);
    gl.attach_shader(program, fs);
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        let log = gl.get_program_info_log(program);
        diag.last_shader_log = log.clone();
        gl.detach_shader(program, vs);
        gl.detach_shader(program, fs);
        gl.delete_shader(vs);
        gl.delete_shader(fs);
        gl.delete_program(program);
        return Err(format!("Program link failed: {}", log));
    }
    let log = gl.get_program_info_log(program);
    if !log.trim().is_empty() { diag.last_shader_log = log; }
    gl.detach_shader(program, vs);
    gl.detach_shader(program, fs);
    gl.delete_shader(vs);
    gl.delete_shader(fs);
    Ok(program)
}

unsafe fn compile_shader(gl: &GlowContext, kind: u32, src: &str) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(kind).map_err(|e| format!("create_shader: {}", e))?;
    gl.shader_source(shader, src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        let log = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        return Err(format!("Shader compile failed: {}", log));
    }
    Ok(shader)
}

fn f32_slice_as_bytes(data: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * std::mem::size_of::<f32>()) }
}

pub fn present(surface: &Surface<WindowSurface>, context: &PossiblyCurrentContext) -> Result<(), String> {
    surface.swap_buffers(context).map_err(|e| format!("swap_buffers failed: {}", e))
}
