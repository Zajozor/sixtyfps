// Copyright © SixtyFPS GmbH <info@sixtyfps.io>
// SPDX-License-Identifier: (GPL-3.0-only OR LicenseRef-SixtyFPS-commercial)

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(all(target_family = "unix", not(target_os = "macos")), link(name = "EGL"))]
extern "C" {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    fn eglGetProcAddress(name: *const std::os::raw::c_void) -> *const std::os::raw::c_void;
}

#[cfg(all(target_family = "unix", not(target_os = "macos")))]
unsafe fn egl_get_proc_address(name: &str) -> *const std::os::raw::c_void {
    eglGetProcAddress(name as *const str as *const std::os::raw::c_void)
}

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;

#[cfg(target_os = "macos")]
unsafe fn egl_get_proc_address(name: &str) -> *const std::os::raw::c_void {
    use std::str::FromStr;
    let symbol_name: CFString = FromStr::from_str(name).unwrap();
    let framework_name: CFString = FromStr::from_str("com.apple.opengl").unwrap();
    let framework = CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef());
    let symbol = CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef());
    symbol as *const _
}

sixtyfps::include_modules!();

use glow::HasContext;

struct EGLUnderlay {
    gl: glow::Context,
    program: glow::Program,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
}

impl Default for EGLUnderlay {
    fn default() -> Self {
        unsafe {
            let gl = glow::Context::from_loader_function(|s| egl_get_proc_address(s) as *const _);

            let program = gl.create_program().expect("Cannot create program");

            let (vertex_shader_source, fragment_shader_source) = (
                r#"#version 100
            attribute vec2 position;
            varying vec2 frag_position;
            void main() {
                frag_position = position;
                gl_Position = vec4(position - 0.5, 0.0, 1.0);
            }"#,
                r#"#version 100
            precision mediump float;
            varying vec2 frag_position;
            void main() {
                gl_FragColor = vec4(frag_position, 0.0, 1.0);
            }"#,
            );

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];

            let mut shaders = Vec::with_capacity(shader_sources.len());

            for (shader_type, shader_source) in shader_sources.iter() {
                let shader = gl.create_shader(*shader_type).expect("Cannot create shader");
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let position_location = gl.get_attrib_location(program, "position").unwrap();

            let vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let vertices = [0.5f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32];

            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices.align_to().1, glow::STATIC_DRAW);

            let vao = gl.create_vertex_array().expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(position_location);
            gl.vertex_attrib_pointer_f32(position_location, 2, glow::FLOAT, false, 8, 0);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Self { gl, program, vbo, vao }
        }
    }
}

impl Drop for EGLUnderlay {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
        }
    }
}

impl EGLUnderlay {
    fn render(&mut self) {
        unsafe {
            let gl = &self.gl;

            gl.use_program(Some(self.program));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            gl.bind_vertex_array(Some(self.vao));

            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let app = App::new();

    let mut underlay = None;

    app.window().set_rendering_notifier(move |state| {
        // eprintln!("rendering state {:#?}", state);

        match state {
            sixtyfps::RenderingState::RenderingSetup => {
                underlay = Some(EGLUnderlay::default());
            }
            sixtyfps::RenderingState::BeforeRendering => {
                if let Some(underlay) = underlay.as_mut() {
                    underlay.render();
                }
            }
            sixtyfps::RenderingState::AfterRendering => {}
            sixtyfps::RenderingState::RenderingTeardown => {
                drop(underlay.take());
            }
        }
    });

    app.run();
}
