//! Desktop window + OpenGL context bootstrap (Stage C1).
//! Failure to create a real OpenGL context is fatal (no silent fake backend).

#![cfg(feature = "opengl")]

use std::ffi::CString;
use std::num::NonZeroU32;
use std::time::Instant;

use glutin::config::{Config, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasRawWindowHandle;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use super::opengl::{present, PartialGlRenderer};
use super::GraphicsSettings;

pub fn run_partial_opengl_runtime(cfg: &GraphicsSettings) -> Result<(), String> {
    let title = if cfg.title.trim().is_empty() {
        "BeniUnityEmu - OpenGL Runtime [PARTIAL]".to_string()
    } else {
        cfg.title.clone()
    };
    let width = cfg.width.max(320);
    let height = cfg.height.max(240);

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title(&title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_resizable(cfg.resizable);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

    let (window, gl_config) = display_builder
        .build(&event_loop, template, gl_config_picker)
        .map_err(|e| format!("Failed to build display/window: {}", e))?;

    let window = window.ok_or_else(|| {
        "Window was not created. OpenGL runtime requires a real desktop window.".to_string()
    })?;

    let raw_handle = window.raw_window_handle();
    let gl_display = gl_config.display();
    let context = create_gl_context(&gl_display, &gl_config, Some(raw_handle))?;

    let attrs = window.build_surface_attributes(SurfaceAttributesBuilder::<WindowSurface>::new());
    let surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &attrs)
            .map_err(|e| format!("create_window_surface: {}", e))?
    };

    let context = context
        .make_current(&surface)
        .map_err(|e| format!("make_current failed (no fake fallback): {}", e))?;

    let gl = std::sync::Arc::new(unsafe {
        glow::Context::from_loader_function(|symbol| {
            let c = CString::new(symbol).unwrap_or_default();
            gl_display.get_proc_address(&c).cast()
        })
    });

    let size = window.inner_size();
    let mut renderer = unsafe {
        PartialGlRenderer::new(gl, size.width.max(1), size.height.max(1))
            .map_err(|e| format!("Renderer init failed: {}", e))?
    };

    println!("{}", renderer.diagnostics_report());
    if cfg.show_renderer_info {
        eprintln!("[OpenGL] Context READY — triangle path IMPLEMENTED");
        eprintln!("[OpenGL] Unity guest runtime: NOT IMPLEMENTED");
    }

    let mut app = RuntimeState {
        window,
        surface,
        context,
        renderer,
        last_frame: Instant::now(),
        show_fps: cfg.show_fps,
        show_info: cfg.show_renderer_info,
        frames_since_log: 0,
    };

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("[OpenGL] Close requested — releasing resources");
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        app.surface.resize(
                            &app.context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                        app.renderer.resize(size.width, size.height);
                    }
                }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let dt = now.duration_since(app.last_frame).as_secs_f32();
                    app.last_frame = now;
                    app.renderer.draw_frame(dt);
                    if let Err(e) = present(&app.surface, &app.context) {
                        eprintln!("[OpenGL] present error: {}", e);
                    }
                    app.frames_since_log += 1;
                    if app.show_fps && app.frames_since_log >= 120 {
                        app.frames_since_log = 0;
                        if app.show_info {
                            eprintln!(
                                "[OpenGL] frame={} fps={:.1} size={}x{}",
                                app.renderer.diagnostics.frame,
                                app.renderer.diagnostics.fps,
                                app.renderer.width,
                                app.renderer.height
                            );
                        }
                    }
                    app.window.request_redraw();
                }
                _ => {}
            },
            Event::AboutToWait => {
                app.window.request_redraw();
            }
            _ => {}
        }
    });

    Ok(())
}

struct RuntimeState {
    window: Window,
    surface: glutin::surface::Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    renderer: PartialGlRenderer,
    last_frame: Instant,
    show_fps: bool,
    show_info: bool,
    frames_since_log: u32,
}

fn gl_config_picker<'a>(configs: Box<dyn Iterator<Item = Config> + 'a>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);
            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .expect("No GL configs found")
}

fn create_gl_context(
    gl_display: &glutin::display::Display,
    gl_config: &Config,
    raw_window_handle: Option<raw_window_handle::RawWindowHandle>,
) -> Result<NotCurrentContext, String> {
    let attrs_33 = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .build(raw_window_handle);

    match unsafe { gl_display.create_context(gl_config, &attrs_33) } {
        Ok(ctx) => return Ok(ctx),
        Err(e33) => {
            eprintln!(
                "[OpenGL] OpenGL 3.3 core context failed ({}); trying default OpenGL",
                e33
            );
        }
    }

    let attrs_default = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(None))
        .build(raw_window_handle);

    unsafe { gl_display.create_context(gl_config, &attrs_default) }.map_err(|e| {
        format!(
            "Failed to create OpenGL context (no software fake fallback): {}",
            e
        )
    })
}
