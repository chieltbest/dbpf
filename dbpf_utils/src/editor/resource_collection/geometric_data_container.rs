use std::sync::Arc;
use eframe::egui::{Response, Ui};
use eframe::{egui, egui_glow, glow};
use eframe::glow::{Context, HasContext};
use tracing::error;
use dbpf::internal_file::resource_collection::geometric_data_container::GeometricDataContainer;
use crate::editor::Editor;

const VERTEX_SHADER_SOURCE: &str = r#"#version 330
    in vec3 in_position;
    const vec4 colors[3] = vec4[3](
        vec4(1.0, 0.0, 0.0, 1.0),
        vec4(0.0, 1.0, 0.0, 1.0),
        vec4(0.0, 0.0, 1.0, 1.0)
    );
    out vec4 v_color;
    void main() {
        v_color = colors[gl_VertexID % 3];
        gl_Position = vec4(in_position / 5, 1.0);
    }"#;
const FRAGMENT_SHADER_SOURCE: &str = r#"#version 330
    precision mediump float;
    in vec4 v_color;
    out vec4 out_color;
    void main() {
        out_color = v_color;
    }"#;

#[repr(u32)]
enum ShaderAttribute {
    POSITION = 0,
}

#[derive(Clone, Debug)]
pub struct GMDCEditorStateData {
    // gl: Arc<Context>,
    program: glow::Program,
    main_subset_vao: glow::NativeVertexArray,
    main_subset_vertices: glow::NativeBuffer,
    main_subset_indices: glow::NativeBuffer,
    main_subset_num_faces: usize,
}

impl Editor for GeometricDataContainer {
    type EditorState = Option<GMDCEditorStateData>;

    fn new_editor(&self, _context: &eframe::egui::Context, gl_context: &Option<Arc<Context>>) -> Self::EditorState {
        if let Some(gl) = gl_context {
            let gl = gl.clone();
            unsafe {
                let program = gl.create_program().expect("Cannot create program");

                let shader_sources = [
                    (glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
                    (glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
                ];

                let shaders: Vec<_> = shader_sources
                    .iter()
                    .map(|(shader_type, shader_source)| {
                        let shader = gl
                            .create_shader(*shader_type)
                            .expect("Cannot create shader");
                        gl.shader_source(
                            shader,
                            shader_source,
                        );
                        gl.compile_shader(shader);
                        assert!(
                            gl.get_shader_compile_status(shader),
                            "Failed to compile custom_3d_glow {shader_type}: {}",
                            gl.get_shader_info_log(shader)
                        );

                        gl.attach_shader(program, shader);
                        shader
                    })
                    .collect();

                gl.link_program(program);
                assert!(
                    gl.get_program_link_status(program),
                    "{}",
                    gl.get_program_info_log(program)
                );

                for shader in shaders {
                    gl.detach_shader(program, shader);
                    gl.delete_shader(shader);
                }


                let vao = gl.create_vertex_array()
                    .inspect_err(|err| error!(?err))
                    .ok()?;
                gl.bind_vertex_array(Some(vao));

                let main_subset_vertex_data: Vec<u8> = self.main_subset.vertices.iter()
                    .flat_map(|v| [v.x.to_le_bytes(), v.y.to_le_bytes(), v.z.to_le_bytes()])
                    .flatten()
                    .collect();
                let main_subset_index_data: Vec<u8> = self.main_subset.faces.iter()
                    .flat_map(|f| f.0.to_le_bytes())
                    .collect();
                let main_subset_num_faces = self.main_subset.faces.len();

                let vbo = gl.create_buffer()
                    .inspect_err(|err| error!(?err))
                    .ok()?;
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &main_subset_vertex_data, glow::STATIC_DRAW);
                gl.vertex_attrib_pointer_f32(ShaderAttribute::POSITION as u32,
                                             3,
                                             glow::FLOAT,
                                             false,
                                             3 * 4,
                                             0);
                gl.enable_vertex_attrib_array(ShaderAttribute::POSITION as u32);
                gl.bind_buffer(glow::ARRAY_BUFFER, None);

                let veo = gl.create_buffer()
                    .inspect_err(|err| error!(?err))
                    .ok()?;
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(veo));
                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &main_subset_index_data, glow::STATIC_DRAW);

                gl.bind_vertex_array(None);

                Some(GMDCEditorStateData {
                    // gl,
                    program,
                    main_subset_vao: vao,
                    main_subset_vertices: vbo,
                    main_subset_indices: veo,
                    main_subset_num_faces,
                })
            }
        } else {
            None
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        if let Some(state) = state {
            egui::Frame::canvas(ui.style())
                .show(ui, |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(ui.available_size_before_wrap(), egui::Sense::drag());

                    let state = state.clone();

                    let cb = egui_glow::CallbackFn::new(move |_info, painter| {
                        let gl = painter.gl();
                        unsafe {
                            gl.use_program(Some(state.program));
                            gl.bind_vertex_array(Some(state.main_subset_vao));
                            gl.draw_elements(glow::TRIANGLES, state.main_subset_num_faces as i32, glow::UNSIGNED_INT, 0);
                        }
                    });

                    let callback = egui::PaintCallback {
                        rect,
                        callback: Arc::new(cb),
                    };

                    ui.painter().add(callback);
                }).response
        } else {
            ui.label("Something went wrong, does this device have OpenGL support?")
        }
    }
}

/*impl Drop for GMDCEditorStateData {
    fn drop(&mut self) {
        let Self {
            gl,
            program,
            main_subset_vao: main_vao,
            // main_subset_vertices,
            // main_subset_indices,
            ..
        } = self;
        unsafe {
            gl.delete_program(*program);
            gl.delete_vertex_array(*main_vao);
            // gl.delete_buffer(*main_subset_vertices);
            // gl.delete_buffer(*main_subset_indices);
        }
    }
}*/
