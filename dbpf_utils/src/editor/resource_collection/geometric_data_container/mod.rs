// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;
use std::iter;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::editor::resource_collection::geometric_data_container::GMDCEditorCreationError::{
	Create, GetAttrib, GetUniform, GetUniformBlock, IncompleteFramebuffer, NoContext, OpenGL,
	ProgramLink, ShaderCompile,
};
use crate::{async_execute, editor::Editor};
use dbpf::internal_file::resource_collection::geometric_data_container::{
	math::{Mat4, Vertex},
	AttributeType, GeometricDataContainer, PrimitiveType,
};
use eframe::{
	egui,
	egui::{PointerButton, Response, SliderClamping, Ui},
	egui_glow, glow,
	glow::{Context, HasContext},
};
use enum_iterator::{all, Sequence};
use futures::channel::oneshot;
use itertools::Either;
use rfd::FileHandle;
use thiserror::Error;
use tracing::{debug, error, span, Level};

const VERTEX_SHADER_SOURCE: &str = include_str!("shaders/main.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/main.frag");

const OFFSCREEN_RENDER_VERTEX_SHADER_SOURCE: &str = include_str!("shaders/blit.vert");
const OFFSCREEN_RENDER_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/blit.frag");

macro_rules! gl_check {
	($gl:expr) => {
		if cfg!(debug_assertions) {
			let err = $gl.get_error();
			if err != glow::NO_ERROR {
				error!(error = %OpenGL(err));
			}
		}
	};
	($gl:expr, $func:ident) => {
		if cfg!(debug_assertions) {
			let err = $gl.get_error();
			let func_name = stringify!($func);
			if err != glow::NO_ERROR {
				error!(error = %OpenGL(err), function = func_name);
			}
		}
	};
}

macro_rules! gl {
	($gl:expr, $func:ident $(, $params:expr)* $(,)?) => {
		{
			let res = $gl.$func($($params),*);
			gl_check!($gl, $func);
			res
		}
	};
}

#[derive(Clone, Debug)]
struct GlMesh {
	// non-owning reference
	vao: glow::VertexArray,
	primitive_type: u32,
	indices: glow::Buffer,
	num_indices: usize,
}

#[derive(Clone, Debug)]
struct SharedGlState {
	program: glow::Program,

	attribute_locations: BTreeMap<&'static str, u32>,
	uniform_block_locations: BTreeMap<&'static str, u32>,

	blend_values_buffer: glow::Buffer,
	bones_buffer: glow::Buffer,

	subsets: Vec<(glow::VertexArray, glow::Buffer, glow::Buffer, usize, usize)>,

	groups: Vec<(glow::VertexArray, glow::Buffer)>,
	meshes: Vec<GlMesh>,

	fbo: Option<Fbo>,

	offscreen_render_program: glow::Program,
	offscreen_render_vao: glow::VertexArray,
}

#[derive(Clone, Debug)]
struct Fbo {
	width: i32,
	height: i32,

	fbo: glow::Framebuffer,
	color_tex: glow::Texture,
	depth_buf: glow::Renderbuffer,
}

#[derive(Clone, Debug)]
struct GlState {
	// gl is not sync, so this has to be separate from the other state
	gl: Arc<Context>,

	data: Arc<RwLock<SharedGlState>>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug, Default, Sequence)]
#[repr(i32)]
enum DisplayMode {
	#[default]
	Standard = 0,
	Normals,
	Tangents,
	Uv,
	Depth,
	Wireframe,
}

#[derive(Clone, Debug)]
struct DisplayState {
	subsets_visible: Vec<bool>,
	meshes_visible: Vec<bool>,

	blend_values: [f32; 256],

	camera_angle: (f32, f32),
	camera_position: Vertex,
	camera_distance: f32,

	display_mode: DisplayMode,
}

#[derive(Clone, Debug)]
pub struct GMDCEditorStateData {
	gl_state: Rc<GlState>,

	display_state: DisplayState,
}

#[derive(Debug)]
pub struct GMDCEditorState {
	data: Result<GMDCEditorStateData, GMDCEditorCreationError>,

	save_file_picker: Option<oneshot::Receiver<Option<FileHandle>>>,

	total_polys: usize,
	total_memory: usize,
}

impl Default for GMDCEditorState {
	fn default() -> Self {
		Self {
			data: Err(NoContext),
			save_file_picker: None,
			total_polys: 0,
			total_memory: 0,
		}
	}
}

#[derive(Clone, Debug, Error)]
pub enum GMDCEditorCreationError {
	#[error("Could not get a OpenGL context, does this device support OpenGL?")]
	NoContext,
	#[error("OpenGL error: 0x{0:x}")]
	OpenGL(u32),
	#[error("Failed to compile OpenGL shader: {0}")]
	ShaderCompile(String),
	#[error("Failed to link OpenGL program: {0}")]
	ProgramLink(String),
	#[error("Failed to create OpenGL resource: {0}")]
	Create(String),
	#[error("Could not get attribute location for OpenGL attribute {0}")]
	GetAttrib(&'static str),
	#[error("Could not get uniform location for OpenGL uniform {0}")]
	GetUniform(&'static str),
	#[error("Could not get uniform block index for OpenGL uniform block {0}")]
	GetUniformBlock(&'static str),
	#[error("framebuffer is incomplete, OpenGL error code: 0x{0}")]
	IncompleteFramebuffer(u32),
}

impl Fbo {
	/// create fbo and associated color and depth buffer
	/// the framebuffer will be bound, and the color texture will be bound to TEXTURE0
	unsafe fn new(
		width: i32,
		height: i32,
		gl: &Arc<Context>,
	) -> Result<Self, GMDCEditorCreationError> {
		// TODO make fbo smaller
		let fbo = gl!(gl, create_framebuffer).map_err(Create)?;
		gl!(gl, bind_framebuffer, glow::FRAMEBUFFER, Some(fbo));

		gl!(gl, active_texture, glow::TEXTURE0);
		let color_tex = gl!(gl, create_texture).map_err(Create)?;
		gl!(gl, bind_texture, glow::TEXTURE_2D, Some(color_tex));
		gl!(
			gl,
			tex_storage_2d,
			glow::TEXTURE_2D,
			1,
			glow::RGBA8,
			width,
			height
		);

		gl!(
			gl,
			framebuffer_texture_2d,
			glow::FRAMEBUFFER,
			glow::COLOR_ATTACHMENT0,
			glow::TEXTURE_2D,
			Some(color_tex),
			0,
		);

		let depth_buf = gl!(gl, create_renderbuffer).unwrap();
		gl!(gl, bind_renderbuffer, glow::RENDERBUFFER, Some(depth_buf));
		gl!(
			gl,
			renderbuffer_storage,
			glow::RENDERBUFFER,
			glow::DEPTH_COMPONENT32F,
			width,
			height
		);
		gl!(
			gl,
			framebuffer_renderbuffer,
			glow::FRAMEBUFFER,
			glow::DEPTH_ATTACHMENT,
			glow::RENDERBUFFER,
			Some(depth_buf),
		);

		gl!(gl, bind_renderbuffer, glow::RENDERBUFFER, None);

		let fbstatus = gl!(gl, check_framebuffer_status, glow::FRAMEBUFFER);
		if fbstatus != glow::FRAMEBUFFER_COMPLETE {
			return Err(IncompleteFramebuffer(fbstatus));
		}

		Ok(Self {
			width,
			height,

			fbo,
			color_tex,
			depth_buf,
		})
	}

	unsafe fn bind(&self, gl: &Arc<Context>) {
		gl!(gl, bind_framebuffer, glow::FRAMEBUFFER, Some(self.fbo));
		gl!(gl, active_texture, glow::TEXTURE0);
		gl!(gl, bind_texture, glow::TEXTURE_2D, Some(self.color_tex));
	}

	unsafe fn drop(&self, gl: &Arc<Context>) {
		gl!(gl, delete_framebuffer, self.fbo);
		gl!(gl, delete_texture, self.color_tex);
		gl!(gl, delete_renderbuffer, self.depth_buf);
	}
}

impl GMDCEditorStateData {
	fn new(
		gmdc: &GeometricDataContainer,
		gl: Arc<Context>,
	) -> Result<Self, GMDCEditorCreationError> {
		unsafe {
			let shader_sources = [
				(glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
				(glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
				(glow::VERTEX_SHADER, OFFSCREEN_RENDER_VERTEX_SHADER_SOURCE),
				(
					glow::FRAGMENT_SHADER,
					OFFSCREEN_RENDER_FRAGMENT_SHADER_SOURCE,
				),
			];

			let shaders = shader_sources
				.iter()
				.map(|(shader_type, shader_source)| {
					let shader = gl!(gl, create_shader, *shader_type).map_err(Create)?;
					gl!(gl, shader_source, shader, shader_source);
					gl!(gl, compile_shader, shader);

					if gl!(gl, get_shader_compile_status, shader) {
						Ok(shader)
					} else {
						Err(ShaderCompile(gl!(gl, get_shader_info_log, shader)))
					}
				})
				.collect::<Result<Vec<_>, _>>()?;

			let main_program = gl!(gl, create_program).map_err(Create)?;

			for shader in &shaders[0..2] {
				gl!(gl, attach_shader, main_program, *shader);
			}

			gl!(gl, link_program, main_program);
			if !gl!(gl, get_program_link_status, main_program) {
				return Err(ProgramLink(gl!(gl, get_program_info_log, main_program)));
			}

			for shader in &shaders[0..2] {
				gl!(gl, detach_shader, main_program, *shader);
				gl!(gl, delete_shader, *shader);
			}

			let attribute_locations = [
				"in_position",
				"in_normal",
				"in_texcoord",
				"in_tangent",
				"in_position_delta_0",
				"in_position_delta_1",
				"in_position_delta_2",
				"in_position_delta_3",
				"in_normal_delta_0",
				"in_normal_delta_1",
				"in_normal_delta_2",
				"in_normal_delta_3",
				"in_blend_keys",
				"in_bone_keys",
				"in_bone_weights",
			]
			.into_iter()
			.map(|name| {
				gl!(gl, get_attrib_location, main_program, name)
					.map(|loc| (name, loc))
					.ok_or(GetAttrib(name))
			})
			.collect::<Result<BTreeMap<_, _>, _>>()?;

			let uniform_block_locations = ["BlendValues", "Bones"]
				.into_iter()
				.map(|name| {
					gl!(gl, get_uniform_block_index, main_program, name)
						.ok_or(GetUniformBlock(name))
						.map(|loc| (name, loc))
				})
				.collect::<Result<BTreeMap<_, _>, _>>()?;

			gl!(
				gl,
				uniform_block_binding,
				main_program,
				uniform_block_locations["BlendValues"],
				0
			);
			gl!(
				gl,
				uniform_block_binding,
				main_program,
				uniform_block_locations["Bones"],
				1
			);

			let blend_values_buffer = gl!(gl, create_buffer).map_err(Create)?;
			gl!(
				gl,
				bind_buffer,
				glow::UNIFORM_BUFFER,
				Some(blend_values_buffer)
			);
			gl!(
				gl,
				buffer_data_size,
				glow::UNIFORM_BUFFER,
				// padded to vec4 as in std140
				(size_of::<f32>() * 256 * 4) as i32,
				glow::STREAM_DRAW,
			);

			let bones_buffer = gl!(gl, create_buffer).map_err(Create)?;
			gl!(gl, bind_buffer, glow::UNIFORM_BUFFER, Some(bones_buffer));
			gl!(
				gl,
				buffer_data_size,
				glow::UNIFORM_BUFFER,
				(size_of::<f32>() * 256 * 16) as i32,
				glow::STREAM_DRAW,
			);

			gl!(gl, bind_buffer, glow::UNIFORM_BUFFER, None);

			gl!(
				gl,
				bind_buffer_base,
				glow::UNIFORM_BUFFER,
				0,
				Some(blend_values_buffer)
			);
			gl!(
				gl,
				bind_buffer_base,
				glow::UNIFORM_BUFFER,
				1,
				Some(bones_buffer)
			);

			debug!(
				GL_ACTIVE_ATOMIC_COUNTER_BUFFERS = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::ACTIVE_ATOMIC_COUNTER_BUFFERS
				)
			);
			debug!(
				GL_ACTIVE_ATTRIBUTES = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::ACTIVE_ATTRIBUTES
				)
			);
			debug!(
				GL_ACTIVE_ATTRIBUTE_MAX_LENGTH = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::ACTIVE_ATTRIBUTE_MAX_LENGTH
				)
			);
			debug!(
				GL_ACTIVE_UNIFORMS = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::ACTIVE_UNIFORMS
				)
			);
			debug!(
				GL_ACTIVE_UNIFORM_MAX_LENGTH = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::ACTIVE_UNIFORM_MAX_LENGTH
				)
			);
			debug!(
				GL_PROGRAM_BINARY_LENGTH = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::PROGRAM_BINARY_LENGTH
				)
			);
			debug!(
				GL_COMPUTE_WORK_GROUP_SIZE = gl!(
					gl,
					get_program_parameter_i32,
					main_program,
					glow::COMPUTE_WORK_GROUP_SIZE
				)
			);

			let active_attributes = gl!(gl, get_active_attributes, main_program);
			for attribute in 0..active_attributes {
				if let Some(attr) = gl!(gl, get_active_attribute, main_program, attribute) {
					debug!(
						attribute,
						name = attr.name,
						atype = attr.atype,
						size = attr.size
					);
				} else {
					debug!(attribute)
				}
			}

			let active_uniforms = gl!(gl, get_active_uniforms, main_program);
			for uniform in 0..active_uniforms {
				if let Some(uni) = gl!(gl, get_active_uniform, main_program, uniform) {
					debug!(uniform, name = uni.name, utype = uni.utype, size = uni.size);
				} else {
					debug!(uniform);
				}
			}

			let or_program = gl!(gl, create_program).map_err(Create)?;

			for shader in &shaders[2..4] {
				gl!(gl, attach_shader, or_program, *shader);
			}

			gl!(gl, link_program, or_program);
			if !gl!(gl, get_program_link_status, main_program) {
				return Err(ProgramLink(gl!(gl, get_program_info_log, or_program)));
			}

			for shader in &shaders[2..4] {
				gl!(gl, detach_shader, or_program, *shader);
				gl!(gl, delete_shader, *shader);
			}

			let subsets = iter::once(&gmdc.bounding_mesh)
				.chain(gmdc.dynamic_bounding_mesh.data.iter())
				.map(|subset| {
					let vao = gl!(gl, create_vertex_array).map_err(Create)?;
					gl!(gl, bind_vertex_array, Some(vao));

					let subset_vertex_data: Vec<u8> = subset
						.vertices
						.iter()
						.flat_map(|v| [v.x.to_le_bytes(), v.y.to_le_bytes(), v.z.to_le_bytes()])
						.flatten()
						.collect();
					let subset_num_vertices = subset.vertices.len();
					let subset_index_data: Vec<u8> = subset
						.faces
						.iter()
						.flat_map(|f| f.0.to_le_bytes())
						.collect();
					let subset_num_faces = subset.faces.len();

					let vbo = gl!(gl, create_buffer).map_err(Create)?;
					gl!(gl, bind_buffer, glow::ARRAY_BUFFER, Some(vbo));
					gl!(
						gl,
						buffer_data_u8_slice,
						glow::ARRAY_BUFFER,
						&subset_vertex_data,
						glow::STATIC_DRAW,
					);
					gl!(
						gl,
						vertex_attrib_pointer_f32,
						attribute_locations["in_position"],
						3,
						glow::FLOAT,
						false,
						3 * 4,
						0,
					);
					gl!(
						gl,
						enable_vertex_attrib_array,
						attribute_locations["in_position"]
					);
					gl!(gl, bind_buffer, glow::ARRAY_BUFFER, None);

					let veo = gl!(gl, create_buffer).map_err(Create)?;
					gl!(gl, bind_buffer, glow::ELEMENT_ARRAY_BUFFER, Some(veo));
					gl!(
						gl,
						buffer_data_u8_slice,
						glow::ELEMENT_ARRAY_BUFFER,
						&subset_index_data,
						glow::STATIC_DRAW,
					);

					Ok((
						(vao, vbo, veo, subset_num_faces, subset_num_vertices),
						false,
					))
				})
				.collect::<Result<Vec<_>, _>>()?;

			let (subsets, subsets_visible) = subsets.into_iter().unzip();

			let groups = gmdc
				.attribute_groups
				.iter()
				.map(|group| {
					let vao = gl!(gl, create_vertex_array).map_err(Create)?;
					gl!(gl, bind_vertex_array, Some(vao));

					span!(Level::DEBUG, "group");

					debug!(?group.number_elements);
					debug!(?group.referenced_active);

					debug!(?group.vertex_indices.data);
					debug!(?group.normal_indices.data);
					debug!(?group.uv_indices.data);

					let (buffer_it, stride, attributes) = group.construct_interleaved(gmdc);
					let buffer = buffer_it.flatten().copied().collect::<Vec<_>>();

					let vbo = gl!(gl, create_buffer).map_err(Create)?;
					// buffers.push(vbo); // TODO just return
					gl!(gl, bind_buffer, glow::ARRAY_BUFFER, Some(vbo));
					gl!(
						gl,
						buffer_data_u8_slice,
						glow::ARRAY_BUFFER,
						&buffer,
						glow::STATIC_DRAW
					);

					debug!(group.buffer_len = buffer.len());
					debug!(group.stride = stride);
					debug!(group.buffer_vertices = buffer.len() / stride);

					for (attr, offset) in attributes {
						span!(Level::DEBUG, "attribute");

						debug!(attr.offset = offset);
						debug!(?attr.binding);
						debug!(?attr.block_format);
						debug!(?attr.index_set);
						debug!(attr.data.len = attr.data.len() / attr.element_size());
						debug!(?attr.number_elements);
						debug!(attr.references.len = attr.references.len());

						let (attr_binding_name, attr_type) = match attr.binding.binding_type {
							AttributeType::Positions => ("in_position", glow::FLOAT),
							AttributeType::PositionDeltas => (
								[
									"in_position_delta_0",
									"in_position_delta_1",
									"in_position_delta_2",
									"in_position_delta_3",
								][attr.binding.binding_slot as usize],
								glow::FLOAT,
							),
							AttributeType::Normals => ("in_normal", glow::FLOAT),
							AttributeType::NormalDeltas => (
								[
									"in_normal_delta_0",
									"in_normal_delta_1",
									"in_normal_delta_2",
									"in_normal_delta_3",
								][attr.binding.binding_slot as usize],
								glow::FLOAT,
							),
							AttributeType::TexCoords => ("in_texcoord", glow::FLOAT),
							AttributeType::Tangents => ("in_tangent", glow::FLOAT),
							AttributeType::BlendKeys => ("in_blend_keys", glow::UNSIGNED_BYTE),
							AttributeType::BoneWeights => ("in_bone_weights", glow::FLOAT),
							AttributeType::BoneKeys => ("in_bone_keys", glow::UNSIGNED_BYTE),
							_ => continue,
						};
						let attr_binding = attribute_locations[attr_binding_name];
						let num_components = attr.block_format.num_components();

						gl!(gl, bind_buffer, glow::ARRAY_BUFFER, Some(vbo));
						gl!(
							gl,
							vertex_attrib_pointer_f32,
							attr_binding,
							num_components as i32,
							attr_type,
							false,
							stride as i32,
							offset as i32,
						);
						gl!(gl, enable_vertex_attrib_array, attr_binding);
						gl!(gl, bind_buffer, glow::ARRAY_BUFFER, None);
					}

					gl!(gl, bind_buffer, glow::ARRAY_BUFFER, None);

					Ok((vao, vbo))
				})
				.collect::<Result<Vec<_>, _>>()?;

			let (meshes, meshes_visible): (Vec<_>, Vec<_>) = gmdc
				.meshes
				.iter()
				.map(|mesh| {
					let vao = groups[mesh.attribute_group_index as usize].0;
					let indices = gl!(gl, create_buffer).map_err(Create)?;

					let mesh_index_data: Vec<u8> = mesh
						.indices
						.iter()
						.flat_map(|f| f.0.to_le_bytes())
						.collect();
					let num_indices = mesh.indices.len();

					let primitive_type = match mesh.primitive_type {
						PrimitiveType::Points => glow::POINTS,
						PrimitiveType::Lines => glow::LINES,
						PrimitiveType::Triangles => glow::TRIANGLES,
					};

					gl!(gl, bind_buffer, glow::ELEMENT_ARRAY_BUFFER, Some(indices));
					gl!(
						gl,
						buffer_data_u8_slice,
						glow::ELEMENT_ARRAY_BUFFER,
						&mesh_index_data,
						glow::STATIC_DRAW,
					);

					Ok((
						GlMesh {
							vao,
							primitive_type,
							indices,
							num_indices,
						},
						true,
					))
				})
				.collect::<Result<Vec<_>, _>>()?
				.into_iter()
				.unzip();

			let or_vao = gl!(gl, create_vertex_array).map_err(Create)?;

			gl!(gl, bind_vertex_array, None);

			Ok(GMDCEditorStateData {
				gl_state: Rc::new(GlState {
					gl,

					data: Arc::new(RwLock::new(SharedGlState {
						program: main_program,

						attribute_locations,
						// uniform_locations,
						uniform_block_locations,

						blend_values_buffer,
						bones_buffer,

						subsets,

						// buffers,
						// attribute_objects,
						groups,
						meshes,

						fbo: None,

						offscreen_render_program: or_program,
						offscreen_render_vao: or_vao,
					})),
				}),

				display_state: DisplayState {
					subsets_visible,
					meshes_visible,
					blend_values: [0.0; 256],

					camera_angle: (std::f32::consts::PI, 0.0),
					camera_position: Vertex {
						x: 0.0,
						y: -1.0,
						z: 0.0,
					},
					camera_distance: 1.0,

					display_mode: DisplayMode::default(),
				},
			})
		}
	}
}

impl Editor for GeometricDataContainer {
	type EditorState = GMDCEditorState;

	fn new_editor(
		&self,
		_context: &egui::Context,
		gl_context: &Option<Arc<Context>>,
	) -> Self::EditorState {
		let data = if let Some(gl) = gl_context {
			GMDCEditorStateData::new(self, gl.clone())
		} else {
			Err(NoContext)
		};

		let total_memory = self
			.attribute_buffers
			.iter()
			.map(|buf| buf.data.len())
			.sum::<usize>()
			+ self
				.meshes
				.iter()
				.map(|m| {
					m.indices.len() * 2 // assume that all indices are u16
				})
				.sum::<usize>();
		let total_polys = self.meshes.iter().map(|m| m.poly_count()).sum();

		GMDCEditorState {
			data,
			save_file_picker: None,
			total_memory,
			total_polys,
		}
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		if let Some(picker) = &mut state.save_file_picker {
			if let Ok(Some(handle)) = picker.try_recv() {
				state.save_file_picker = None;
				if let Some(handle) = handle {
					let gltf = self.export_gltf();
					if let Some(gltf) = gltf {
						let res = futures::executor::block_on(handle.write(&gltf.0));
						if let Err(e) = res {
							error!(?e);
						}
					}
				}
			}
		}

		self.file_name.name.show_editor(&mut 300.0, ui);

		ui.label(format!("poly: {} triangles", state.total_polys));
		ui.label(format!(
			"memory: {}",
			humansize::format_size(state.total_memory, humansize::DECIMAL)
		));
		ui.label(format!(
			"draw calls: {}",
			self.meshes.len()
				+ if !self.bounding_mesh.faces.is_empty()
					|| self
						.dynamic_bounding_mesh
						.iter()
						.any(|d| !d.faces.is_empty())
				{
					1
				} else {
					0
				}
		));

		/*ui.horizontal_wrapped(|ui| {
			for (i, (_, _, _, num_indices, num_vertices, subset_enabled)) in gl_state.subsets
				.iter_mut().enumerate() {
				ui.checkbox(subset_enabled, if i == 0 {
					format!("main: {}v, {}i", num_vertices, num_indices)
				} else {
					format!("{}: {}v, {}i", i - 1, num_vertices, num_indices)
				});
			}
		});*/

		let available = ui.available_size_before_wrap();

		egui::ScrollArea::vertical()
			.auto_shrink([false, true])
			.max_height(available.y / 3.0)
			.show(ui, |ui| {
				ui.horizontal_wrapped(|ui| {
					// TODO name in tooltip
					ui.group(|ui| {
						ui.vertical(|ui| {
							for (mesh, visible) in self.meshes.iter_mut().zip(
								state
									.data
									.as_mut()
									.map(|state| {
										Either::Left(state.display_state.meshes_visible.iter_mut())
									})
									.unwrap_or(Either::Right(iter::repeat(())))
									.factor_into_iter(),
							) {
								ui.horizontal(|ui| {
									if let Either::Left(visible) = visible {
										ui.add(egui::Checkbox::without_text(visible));
									}

									mesh.name.show_editor(&mut 100.0, ui).on_hover_ui(|ui| {
										ui.label(format!(
											"{} {:?}",
											mesh.poly_count(),
											mesh.primitive_type,
										));

										let bones = mesh
											.bone_references
											.iter()
											.map(|r| r.0)
											.collect::<Vec<_>>();
										ui.label(format!("Bones: {:?}", bones));

										ui.label("Attributes:");
										let attribute_group = &self.attribute_groups
											[mesh.attribute_group_index as usize];
										for attr_idx in attribute_group.attributes.iter() {
											let attr = &self.attribute_buffers[attr_idx.0 as usize];
											ui.label(format!(
												"{:?}#{}",
												attr.binding.binding_type,
												attr.binding.binding_slot
											));
										}
									});

									ui.add(egui::DragValue::new(&mut mesh.opacity))
										.on_hover_text("opacity");
								});
							}
						});
					});
					// TODO name in tooltip
					if !self.blend_group_bindings.is_empty() {
						ui.group(|ui| {
							ui.vertical(|ui| {
								for (name, value) in self.blend_group_bindings.iter_mut().zip(
									state
										.data
										.as_mut()
										.map(|state| {
											Either::Left(
												state.display_state.blend_values.iter_mut(),
											)
										})
										.unwrap_or(Either::Right(iter::repeat(())))
										.factor_into_iter(),
								) {
									ui.horizontal(|ui| {
										if let Either::Left(value) = value {
											ui.add(
												egui::Slider::new(value, 0.0..=1.0)
													.clamping(SliderClamping::Never),
											);
										}
										name.blend_group
											.show_editor(&mut 70.0, ui)
											.on_hover_text("Blend group");
										name.element
											.show_editor(&mut 70.0, ui)
											.on_hover_text("Element");
									});
								}
							});
						});
					}
				});
			});

		ui.horizontal_wrapped(|ui| {
			if ui
				.button("Export glTF")
				.on_hover_text("export the mesh to a .gltf file")
				.clicked() && state.save_file_picker.is_none()
			{
				let (tx, rx) = oneshot::channel();
				let dialog = rfd::AsyncFileDialog::new()
					.set_file_name(format!(
						"{}.glb",
						String::from_utf8_lossy(&self.file_name.name.0.data)
					))
					.add_filter("OpenGL Transfer Format", &["gltf", "glb"]);
				// TODO global options open file path set directory
				let dialog = dialog.save_file();
				async_execute(async move {
					let file = dialog.await;
					let _ = if let Some(handle) = file {
						tx.send(Some(handle))
					} else {
						tx.send(None)
					};
				});
				state.save_file_picker = Some(rx);
			}

			if let Ok(state_data) = &mut state.data {
				for mode in all::<DisplayMode>() {
					ui.radio_value(
						&mut state_data.display_state.display_mode,
						mode,
						format!("{:?}", mode).to_lowercase(),
					);
				}
			}
		});

		match &mut state.data {
			Ok(state_data) => {
				let display_data = &mut state_data.display_state;

				egui::Frame::canvas(ui.style())
					.show(ui, |ui| {
						let (rect, response) = ui.allocate_exact_size(
							ui.available_size_before_wrap(),
							egui::Sense::drag(),
						);

						let inverse_orientation = Mat4::rotation_y(-display_data.camera_angle.0)
							* Mat4::rotation_x(-display_data.camera_angle.1);

						let drag_delta = response.drag_delta() / rect.height() * 2.0;
						if ui.input(|i| i.pointer.button_down(PointerButton::Primary)) {
							display_data.camera_angle.0 += drag_delta.x * std::f32::consts::PI;
							display_data.camera_angle.1 -=
								drag_delta.y * std::f32::consts::FRAC_PI_2;
						}
						if ui.input(|i| i.pointer.button_down(PointerButton::Secondary)) {
							display_data.camera_position += inverse_orientation
								* Vertex {
									x: drag_delta.x,
									y: -drag_delta.y,
									z: 0.0,
								};
						}
						if ui.input(|i| i.pointer.button_down(PointerButton::Middle)) {
							display_data.camera_position += inverse_orientation
								* Vertex {
									x: drag_delta.x,
									y: 0.0,
									z: -drag_delta.y,
								};
						}

						if response.hovered() {
							let scroll_delta =
								ui.input(|i| i.smooth_scroll_delta) / rect.height() * 2.0;
							display_data.camera_position += inverse_orientation
								* Vertex {
									x: scroll_delta.x,
									y: 0.0,
									z: -scroll_delta.y,
								};
						}

						let gl_state_ptr = Arc::downgrade(&state_data.gl_state.data);
						let display_data = display_data.clone();
						// let transforms = self.bones.clone();
						let dark_mode = ui.style().visuals.dark_mode;

						let cb = egui_glow::CallbackFn::new(move |info, painter| {
							let gl = painter.gl();
							let Some(ptr) = gl_state_ptr.upgrade() else {
								return;
							};
							let Ok(mut gl_state) = ptr.write() else {
								return;
							};

							let viewport = info.viewport_in_pixels();
							let clip = info.clip_rect_in_pixels();

							unsafe {
								gl!(gl, use_program, Some(gl_state.program));

								if let Some(fbo) = &mut gl_state.fbo {
									if fbo.width != viewport.width_px
										|| fbo.height != viewport.height_px
									{
										fbo.drop(gl);
										gl_state.fbo = None;
									}
								}

								match &mut gl_state.fbo {
									Some(fbo) => {
										fbo.bind(gl);
									}
									fbo => {
										if let Ok(new_fbo) =
											Fbo::new(viewport.width_px, viewport.height_px, gl)
										{
											*fbo = Some(new_fbo);
										} else {
											return;
										};
									}
								};

								gl!(gl, viewport, 0, 0, viewport.width_px, viewport.height_px);
								gl!(gl, scissor, 0, 0, viewport.width_px, viewport.height_px);

								gl!(gl, clear_color, 0.0, 0.0, 0.0, 0.0);
								gl!(gl, clear_depth, 0.0);
								gl!(gl, clear, glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

								gl!(gl, enable, glow::DEPTH_TEST);
								gl!(gl, depth_func, glow::GREATER);

								let projection_mat =
									Mat4::projection(0.1, 1.0 / info.viewport.aspect_ratio());

								let model_mat = Mat4::translation(Vertex {
									x: 0.0,
									y: 0.0,
									z: display_data.camera_distance,
								}) * Mat4::rotation_x(display_data.camera_angle.1)
									* Mat4::rotation_y(display_data.camera_angle.0)
									* Mat4::translation(display_data.camera_position)
									* Mat4::identity().swap_axes(1, 2);

								/*for ((vao, _, _, num_faces, _, _), transform) in state.subsets.iter()
									.zip(iter::once(ident_transform)
										.chain(transforms.data.clone().into_iter())
										.chain(iter::repeat(ident_transform)))
									.filter(|((_, _, _, _, _, enabled), _)| *enabled) {
									gl!(gl, bind_vertex_array, Some(*vao));

									let object_mat = model_mat * Mat4::transform(transform.inverse());
									gl!(gl, uniform_matrix_4_f32_slice,
										gl!(gl, get_uniform_location, state.program, "camera").as_ref(),
										false,
										&object_mat.0,
									);

									gl!(gl, draw_elements, glow::TRIANGLES, *num_faces as i32, glow::UNSIGNED_INT, 0);
								}*/

								let blend_values_data = display_data
									.blend_values
									.iter()
									.flat_map(|f| {
										// pad to vec4 as specified in std140 layout
										f.to_ne_bytes().into_iter().chain(iter::repeat_n(0, 12))
									})
									.collect::<Vec<_>>();
								gl!(
									gl,
									bind_buffer,
									glow::UNIFORM_BUFFER,
									Some(gl_state.blend_values_buffer),
								);
								gl!(
									gl,
									buffer_sub_data_u8_slice,
									glow::UNIFORM_BUFFER,
									0,
									&blend_values_data,
								);

								let identity_bones = iter::repeat_n(Mat4::identity(), 256)
									.flat_map(|eye| eye.0.map(|f| f.to_ne_bytes()))
									.flatten()
									.collect::<Vec<_>>();
								gl!(
									gl,
									bind_buffer,
									glow::UNIFORM_BUFFER,
									Some(gl_state.bones_buffer),
								);
								gl!(
									gl,
									buffer_sub_data_u8_slice,
									glow::UNIFORM_BUFFER,
									0,
									&identity_bones,
								);

								gl!(gl, bind_buffer, glow::UNIFORM_BUFFER, None);

								let display_mode = match display_data.display_mode {
									DisplayMode::Wireframe => {
										gl!(gl, polygon_mode, glow::FRONT_AND_BACK, glow::LINE);
										DisplayMode::Depth as i32
									}
									mode => {
										gl!(gl, polygon_mode, glow::FRONT_AND_BACK, glow::FILL);
										mode as i32
									}
								};

								gl!(
									gl,
									uniform_1_i32,
									gl!(gl, get_uniform_location, gl_state.program, "display_mode")
										.as_ref(),
									display_mode,
								);

								gl!(
									gl,
									uniform_1_i32,
									gl!(gl, get_uniform_location, gl_state.program, "dark_mode")
										.as_ref(),
									dark_mode as i32,
								);

								for mesh in gl_state
									.meshes
									.iter()
									.zip(&display_data.meshes_visible)
									.filter_map(|(m, visible)| visible.then_some(m))
								{
									// TODO bone bindings

									gl!(gl, bind_vertex_array, Some(mesh.vao));

									gl!(
										gl,
										uniform_matrix_4_f32_slice,
										gl!(
											gl,
											get_uniform_location,
											gl_state.program,
											"projection_matrix",
										)
										.as_ref(),
										false,
										&projection_mat.transpose().0,
									);

									gl!(
										gl,
										uniform_matrix_4_f32_slice,
										gl!(
											gl,
											get_uniform_location,
											gl_state.program,
											"view_matrix",
										)
										.as_ref(),
										false,
										&model_mat.transpose().0,
									);

									gl!(
										gl,
										bind_buffer,
										glow::ELEMENT_ARRAY_BUFFER,
										Some(mesh.indices),
									);

									gl!(
										gl,
										draw_elements,
										mesh.primitive_type,
										mesh.num_indices as i32,
										glow::UNSIGNED_INT,
										0,
									);
								}

								// render the texture to the main buffer target
								gl!(gl, use_program, Some(gl_state.offscreen_render_program));
								gl!(gl, bind_vertex_array, Some(gl_state.offscreen_render_vao));
								gl!(
									gl,
									bind_framebuffer,
									glow::FRAMEBUFFER,
									painter.intermediate_fbo(),
								);

								gl!(gl, disable, glow::DEPTH_TEST);
								gl!(gl, depth_func, glow::LESS);
								gl!(gl, polygon_mode, glow::FRONT_AND_BACK, glow::FILL);

								gl!(
									gl,
									viewport,
									viewport.left_px,
									viewport.from_bottom_px,
									viewport.width_px,
									viewport.height_px
								);
								gl!(
									gl,
									scissor,
									clip.left_px,
									clip.from_bottom_px,
									clip.width_px,
									clip.height_px
								);

								gl!(
									gl,
									uniform_1_i32,
									// TODO store
									gl!(
										gl,
										get_uniform_location,
										gl_state.offscreen_render_program,
										"t",
									)
									.as_ref(),
									0,
								);

								gl!(gl, draw_arrays, glow::TRIANGLES, 0, 3);

								gl!(gl, bind_texture, glow::TEXTURE_2D, None);
							}
						});

						let callback = egui::PaintCallback {
							rect,
							callback: Arc::new(cb),
						};

						ui.painter().add(callback);
					})
					.response
			}
			Err(err) => ui.label(format!("{err}")),
		}
	}
}

impl Drop for GlState {
	fn drop(&mut self) {
		let Self { gl, data } = self;
		if let Ok(guard) = data.read() {
			let SharedGlState {
				program,
				subsets,
				groups,
				meshes,
				offscreen_render_program,
				offscreen_render_vao,
				fbo,
				..
			} = guard.deref();
			unsafe {
				gl!(gl, delete_program, *program);

				for (vao, vertices, indices, ..) in subsets {
					gl!(gl, delete_vertex_array, *vao);
					gl!(gl, delete_buffer, *vertices);
					gl!(gl, delete_buffer, *indices);
				}

				for (vao, buffer) in groups {
					gl!(gl, delete_vertex_array, *vao);
					gl!(gl, delete_buffer, *buffer);
				}

				for mesh in meshes {
					gl!(gl, delete_buffer, mesh.indices);
				}

				gl!(gl, delete_program, *offscreen_render_program);
				gl!(gl, delete_vertex_array, *offscreen_render_vao);

				if let Some(fbo) = fbo {
					fbo.drop(gl);
				}
			}
		}
	}
}
