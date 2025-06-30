use std::sync::Arc;
use eframe::egui::{Response, Ui};
use eframe::glow;
use eframe::glow::{Context, HasContext};
use tracing::error;
use dbpf::internal_file::resource_collection::geometric_data_container::GeometricDataContainer;
use crate::editor::Editor;

pub struct GMDCEditorState {
    gl: Arc<Context>,
    main_subset_vertices: glow::VertexArray,
}

impl Editor for GeometricDataContainer {
    type EditorState = Option<GMDCEditorState>;

    fn new_editor(&self, _context: &eframe::egui::Context, gl_context: &Option<Arc<Context>>) -> Self::EditorState {
        if let Some(gl) = gl_context {
            unsafe {
                let main_subset: Vec<u8> = self.main_subset.vertices.iter()
                    .flat_map(|v| [v.x.to_le_bytes(), v.y.to_le_bytes(), v.z.to_le_bytes()])
                    .flatten()
                    .collect();
                let vbo = gl.create_buffer()
                    .inspect_err(|err| error!(?err))
                    .ok()?;
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_u8_slice(vbo.0.into(), &main_subset, glow::STATIC_DRAW);
            }

            todo!()
        } else {
            None
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        if let Some(state) = state {
            unsafe {

            }
        }
        todo!()
    }
}

impl Drop for GMDCEditorState {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.main_subset_vertices);
        }
    }
}
