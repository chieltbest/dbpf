// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;
use std::iter;
use std::sync::Arc;

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
use futures::channel::oneshot;
use futures::TryFutureExt;
use itertools::Either;
use rfd::FileHandle;
use thiserror::Error;
use tracing::{debug, error, span, warn, Level};

use crate::editor::resource_collection::geometric_data_container::GMDCEditorCreationError::{
	Create, GetAttrib, GetUniform, GetUniformBlock, NoContext, ProgramLink, ShaderCompile,
};
use crate::{async_execute, editor::Editor};

const VERTEX_SHADER_SOURCE: &str = include_str!("shaders/main.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/main.frag");

const OFFSCREEN_RENDER_VERTEX_SHADER_SOURCE: &str = include_str!("shaders/blit.vert");
const OFFSCREEN_RENDER_FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/blit.frag");

#[derive(Clone, Debug)]
struct GlMesh {
	vao: glow::VertexArray,
	primitive_type: u32,
	indices: glow::Buffer,
	num_indices: usize,
}

#[derive(Clone, Debug)]
struct SharedGlState {
	program: glow::Program,

	attribute_locations: BTreeMap<&'static str, u32>,
	uniform_locations: BTreeMap<&'static str, glow::UniformLocation>,
	uniform_block_locations: BTreeMap<&'static str, u32>,

	blend_values_buffer: glow::Buffer,
	bones_buffer: glow::Buffer,

	subsets: Vec<(glow::VertexArray, glow::Buffer, glow::Buffer, usize, usize)>,

	buffers: Vec<glow::Buffer>,
	attribute_objects: Vec<glow::VertexArray>,
	meshes: Vec<GlMesh>,

	offscreen_render_program: glow::Program,
	offscreen_render_vao: glow::VertexArray,
}

#[derive(Clone, Debug)]
struct GlState {
	// gl is not sync, so this has to be separate from the other state
	gl: Arc<Context>,

	data: Arc<SharedGlState>,
}

#[derive(Clone, Debug)]
struct DisplayState {
	subsets_visible: Vec<bool>,
	meshes_visible: Vec<bool>,

	blend_values: [f32; 256],

	camera_angle: (f32, f32),
	camera_position: Vertex,
	camera_distance: f32,

	display_mode: i32, // TODO enum
}

#[derive(Clone, Debug)]
pub struct GMDCEditorStateData {
	gl_state: Arc<GlState>,

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
	#[error("OpenGL error: {0}")]
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
					let shader = gl.create_shader(*shader_type).map_err(Create)?;
					gl.shader_source(shader, shader_source);
					gl.compile_shader(shader);

					if gl.get_shader_compile_status(shader) {
						Ok(shader)
					} else {
						Err(ShaderCompile(gl.get_shader_info_log(shader)))
					}
				})
				.collect::<Result<Vec<_>, _>>()?;

			let main_program = gl.create_program().map_err(Create)?;

			for shader in &shaders[0..2] {
				gl.attach_shader(main_program, *shader);
			}

			gl.link_program(main_program);
			if !gl.get_program_link_status(main_program) {
				return Err(ProgramLink(gl.get_program_info_log(main_program)));
			}

			for shader in &shaders[0..2] {
				gl.detach_shader(main_program, *shader);
				gl.delete_shader(*shader);
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
				"in_blend_weights",
				"in_bone_keys",
				"in_bone_weights",
			]
			.into_iter()
			.map(|name| {
				gl.get_attrib_location(main_program, name)
					.map(|loc| (name, loc))
					.ok_or(GetAttrib(name))
			})
			.collect::<Result<BTreeMap<_, _>, _>>()?;

			let uniform_locations = ["view_matrix", "display_mode", "dark_mode"]
				.into_iter()
				.map(|name| {
					gl.get_uniform_location(main_program, name)
						.map(|loc| (name, loc))
						.ok_or(GetUniform(name))
				})
				.collect::<Result<BTreeMap<_, _>, _>>()?;

			let uniform_block_locations = ["BlendValues", "Bones"]
				.into_iter()
				.map(|name| {
					gl.get_uniform_block_index(main_program, name)
						.ok_or(GetUniformBlock(name))
						.map(|loc| (name, loc))
				})
				.collect::<Result<BTreeMap<_, _>, _>>()?;

			gl.uniform_block_binding(main_program, uniform_block_locations["BlendValues"], 0);
			gl.uniform_block_binding(main_program, uniform_block_locations["Bones"], 1);

			let blend_values_buffer = gl.create_buffer().map_err(Create)?;
			gl.bind_buffer(glow::UNIFORM_BUFFER, Some(blend_values_buffer));
			/*gl.buffer_storage(
				glow::UNIFORM_BUFFER,
				(size_of::<f32>() * 256) as i32,
				None,
				glow::DYNAMIC_STORAGE_BIT | glow::MAP_WRITE_BIT,
			);*/
			gl.buffer_data_size(
				glow::UNIFORM_BUFFER,
				// padded to vec4 as in std140
				(size_of::<f32>() * 256 * 4) as i32,
				glow::DYNAMIC_DRAW,
			);

			let bones_buffer = gl.create_buffer().map_err(Create)?;
			gl.bind_buffer(glow::UNIFORM_BUFFER, Some(bones_buffer));
			/*gl.buffer_storage(
				glow::UNIFORM_BUFFER,
				(size_of::<f32>() * 256 * 16) as i32,
				None,
				glow::DYNAMIC_STORAGE_BIT | glow::MAP_WRITE_BIT,
			);*/
			gl.buffer_data_size(
				glow::UNIFORM_BUFFER,
				(size_of::<f32>() * 256 * 16) as i32,
				glow::DYNAMIC_DRAW,
			);

			gl.bind_buffer(glow::UNIFORM_BUFFER, None);

			gl.bind_buffer_base(glow::UNIFORM_BUFFER, 0, Some(blend_values_buffer));
			gl.bind_buffer_base(glow::UNIFORM_BUFFER, 1, Some(bones_buffer));

			debug!(
				GL_ACTIVE_ATOMIC_COUNTER_BUFFERS =
					gl.get_program_parameter_i32(main_program, glow::ACTIVE_ATOMIC_COUNTER_BUFFERS)
			);
			debug!(
				GL_ACTIVE_ATTRIBUTES =
					gl.get_program_parameter_i32(main_program, glow::ACTIVE_ATTRIBUTES)
			);
			debug!(
				GL_ACTIVE_ATTRIBUTE_MAX_LENGTH =
					gl.get_program_parameter_i32(main_program, glow::ACTIVE_ATTRIBUTE_MAX_LENGTH)
			);
			debug!(
				GL_ACTIVE_UNIFORMS =
					gl.get_program_parameter_i32(main_program, glow::ACTIVE_UNIFORMS)
			);
			debug!(
				GL_ACTIVE_UNIFORM_MAX_LENGTH =
					gl.get_program_parameter_i32(main_program, glow::ACTIVE_UNIFORM_MAX_LENGTH)
			);
			debug!(
				GL_PROGRAM_BINARY_LENGTH =
					gl.get_program_parameter_i32(main_program, glow::PROGRAM_BINARY_LENGTH)
			);
			debug!(
				GL_COMPUTE_WORK_GROUP_SIZE =
					gl.get_program_parameter_i32(main_program, glow::COMPUTE_WORK_GROUP_SIZE)
			);

			let active_attributes = gl.get_active_attributes(main_program);
			for attribute in 0..active_attributes {
				if let Some(attr) = gl.get_active_attribute(main_program, attribute) {
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

			let active_uniforms = gl.get_active_uniforms(main_program);
			for uniform in 0..active_uniforms {
				if let Some(uni) = gl.get_active_uniform(main_program, uniform) {
					debug!(uniform, name = uni.name, utype = uni.utype, size = uni.size);
				} else {
					debug!(uniform);
				}
			}

			let or_program = gl.create_program().map_err(Create)?;

			for shader in &shaders[2..4] {
				gl.attach_shader(or_program, *shader);
			}

			gl.link_program(or_program);
			if !gl.get_program_link_status(main_program) {
				return Err(ProgramLink(gl.get_program_info_log(or_program)));
			}

			for shader in &shaders[2..4] {
				gl.detach_shader(or_program, *shader);
				gl.delete_shader(*shader);
			}

			let subsets = std::iter::once(&gmdc.bounding_mesh)
				.chain(gmdc.dynamic_bounding_mesh.data.iter())
				.map(|subset| {
					let vao = gl.create_vertex_array().map_err(Create)?;
					gl.bind_vertex_array(Some(vao));

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

					let vbo = gl.create_buffer().map_err(Create)?;
					gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
					gl.buffer_data_u8_slice(
						glow::ARRAY_BUFFER,
						&subset_vertex_data,
						glow::STATIC_DRAW,
					);
					gl.vertex_attrib_pointer_f32(
						attribute_locations["in_position"],
						3,
						glow::FLOAT,
						false,
						3 * 4,
						0,
					);
					gl.enable_vertex_attrib_array(attribute_locations["in_position"]);
					gl.bind_buffer(glow::ARRAY_BUFFER, None);

					let veo = gl.create_buffer().map_err(Create)?;
					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(veo));
					gl.buffer_data_u8_slice(
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

			let mut buffers = vec![];

			let attribute_objects = gmdc
				.attribute_groups
				.iter()
				.map(|group| {
					let vao = gl.create_vertex_array().map_err(Create)?;
					gl.bind_vertex_array(Some(vao));

					span!(Level::DEBUG, "group");

					debug!(?group.number_elements);
					debug!(?group.referenced_active);

					debug!(?group.vertex_indices.data);
					debug!(?group.normal_indices.data);
					debug!(?group.uv_indices.data);

					let (buffer_it, stride, attributes) = group.construct_interleaved(gmdc);
					let buffer = buffer_it.flatten().copied().collect::<Vec<_>>();

					let vbo = gl.create_buffer().map_err(Create)?;
					buffers.push(vbo); // TODO just return
					gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
					gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &buffer, glow::STATIC_DRAW);

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
							AttributeType::TangentDeltas => ("in_tangent_delta", glow::FLOAT),
							AttributeType::BlendValues1 => ("in_blend_indices", glow::FLOAT),
							AttributeType::BoneValues => ("in_blend_weights", glow::FLOAT),
							AttributeType::BlendKeys => ("in_blend_keys", glow::UNSIGNED_BYTE),
							AttributeType::BoneWeights => ("in_bone_weights", glow::FLOAT),
							AttributeType::BoneKeys => ("in_bone_keys", glow::UNSIGNED_BYTE),
							AttributeType::BlendValues2 => {
								("in_target_indices", glow::UNSIGNED_BYTE)
							}
							AttributeType::VertexID => ("in_vertex_id", glow::UNSIGNED_BYTE),
							AttributeType::RegionMask => ("in_region_mask", glow::UNSIGNED_BYTE),
							AttributeType::DeformMask => ("in_deform_mask", glow::UNSIGNED_BYTE),
						};
						let attr_binding = attribute_locations[attr_binding_name];
						let num_components = attr.block_format.num_components();

						gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
						gl.vertex_attrib_pointer_f32(
							attr_binding,
							num_components as i32,
							attr_type,
							false,
							stride as i32,
							offset as i32,
						);
						gl.enable_vertex_attrib_array(attr_binding);
						gl.bind_buffer(glow::ARRAY_BUFFER, None);
					}

					gl.bind_buffer(glow::ARRAY_BUFFER, None);

					Ok(vao)
				})
				.collect::<Result<Vec<_>, _>>()?;

			let (meshes, meshes_visible): (Vec<_>, Vec<_>) = gmdc
				.meshes
				.iter()
				.map(|mesh| {
					let vao = attribute_objects[mesh.attribute_group_index as usize];
					let indices = gl.create_buffer().map_err(Create)?;

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

					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices));
					gl.buffer_data_u8_slice(
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

			let or_vao = gl.create_vertex_array().map_err(Create)?;

			gl.bind_vertex_array(None);

			Ok(GMDCEditorStateData {
				gl_state: Arc::new(GlState {
					gl,

					data: Arc::new(SharedGlState {
						program: main_program,

						attribute_locations,
						uniform_locations,
						uniform_block_locations,

						blend_values_buffer,
						bones_buffer,

						subsets,

						buffers,
						attribute_objects,
						meshes,

						offscreen_render_program: or_program,
						offscreen_render_vao: or_vao,
					}),
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

					display_mode: 0,
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
				// TODO standard display
				for (i, name) in ["normals", "tangents", "uv", "depth", "wireframe"]
					.into_iter()
					.enumerate()
				{
					ui.radio_value(&mut state_data.display_state.display_mode, i as i32, name);
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
							let Some(gl_state) = gl_state_ptr.upgrade() else {
								return;
							};
							// let viewport = info.viewport_in_pixels();
							// let clip = info.clip_rect_in_pixels();
							let [width, height] = info.screen_size_px.map(|u| u as i32);
							// let width = viewport.width_px;
							// let height = viewport.height_px;
							unsafe {
								gl.use_program(Some(gl_state.program));

								// TODO opengl error handling
								let err = gl.get_error();
								if err != glow::NO_ERROR {
									eprintln!("s {err:?}");
								}

								// TODO retain fbo across frames
								let fbo = gl.create_framebuffer().unwrap();
								gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

								gl.active_texture(glow::TEXTURE0);
								let ctex = gl.create_texture().unwrap();
								gl.bind_texture(glow::TEXTURE_2D, Some(ctex));
								gl.tex_storage_2d(glow::TEXTURE_2D, 1, glow::RGBA8, width, height);

								gl.framebuffer_texture_2d(
									glow::FRAMEBUFFER,
									glow::COLOR_ATTACHMENT0,
									glow::TEXTURE_2D,
									Some(ctex),
									0,
								);

								/*let fbstatus = gl.check_framebuffer_status(glow::FRAMEBUFFER);
								if fbstatus != glow::FRAMEBUFFER_COMPLETE {
									error!(
										"framebuffer is incomplete, OpenGL error code: {}",
										fbstatus
									);
								}*/

								let rbd = gl.create_renderbuffer().unwrap();
								gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbd));
								gl.renderbuffer_storage(
									glow::RENDERBUFFER,
									glow::DEPTH_COMPONENT32F,
									width,
									height,
								);
								gl.framebuffer_renderbuffer(
									glow::FRAMEBUFFER,
									glow::DEPTH_ATTACHMENT,
									glow::RENDERBUFFER,
									Some(rbd),
								);

								gl.bind_renderbuffer(glow::RENDERBUFFER, None);

								gl.clear_color(0.0, 0.0, 0.0, 0.0);
								gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

								gl.enable(glow::DEPTH_TEST);
								gl.depth_func(glow::LESS);

								// gl.viewport(viewport.left_px, viewport.from_bottom_px, viewport.width_px, viewport.height_px);
								// gl.scissor(clip.left_px, clip.from_bottom_px, clip.width_px, clip.height_px);

								/*eprintln!("{} {} {} {} ({})", viewport.left_px, viewport.top_px, viewport.width_px, viewport.height_px, viewport.from_bottom_px);
								eprintln!("{} {} {} {} ({})", clip.left_px, clip.top_px, clip.width_px, clip.height_px, clip.from_bottom_px);
								eprintln!("{} {}", width, height);*/

								let model_mat =
									Mat4::projection(0.1, 1.0 / info.viewport.aspect_ratio())
										* Mat4::translation(Vertex {
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
									gl.bind_vertex_array(Some(*vao));

									let object_mat = model_mat * Mat4::transform(transform.inverse());
									gl.uniform_matrix_4_f32_slice(
										gl.get_uniform_location(state.program, "camera").as_ref(),
										false,
										&object_mat.0,
									);

									gl.draw_elements(glow::TRIANGLES, *num_faces as i32, glow::UNSIGNED_INT, 0);
								}*/

								let blend_values_data = display_data
									.blend_values
									.iter()
									.flat_map(|f| {
										// pad to vec4 as specified in std140 layout
										f.to_ne_bytes().into_iter().chain(iter::repeat_n(0, 12))
									})
									.collect::<Vec<_>>();
								gl.bind_buffer(
									glow::UNIFORM_BUFFER,
									Some(gl_state.blend_values_buffer),
								);
								gl.buffer_sub_data_u8_slice(
									glow::UNIFORM_BUFFER,
									0,
									&blend_values_data,
								);

								let identity_bones = iter::repeat_n(Mat4::identity(), 256)
									.flat_map(|eye| eye.0.map(|f| f.to_ne_bytes()))
									.flatten()
									.collect::<Vec<_>>();
								gl.bind_buffer(glow::UNIFORM_BUFFER, Some(gl_state.bones_buffer));
								gl.buffer_sub_data_u8_slice(
									glow::UNIFORM_BUFFER,
									0,
									&identity_bones,
								);

								gl.bind_buffer(glow::UNIFORM_BUFFER, None);

								let display_mode = if display_data.display_mode <= 3 {
									gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
									display_data.display_mode
								} else {
									gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
									3
								};

								gl.uniform_1_i32(
									Some(&gl_state.uniform_locations["display_mode"]),
									display_mode,
								);

								gl.uniform_1_i32(
									Some(&gl_state.uniform_locations["dark_mode"]),
									dark_mode as i32,
								);

								for mesh in gl_state
									.meshes
									.iter()
									.zip(&display_data.meshes_visible)
									.filter_map(|(m, visible)| visible.then_some(m))
								{
									// TODO bone bindings

									gl.bind_vertex_array(Some(mesh.vao));

									gl.vertex_attrib_4_f32(
										gl_state.attribute_locations["in_blend_weights"],
										1.0,
										1.0,
										1.0,
										1.0,
									);

									gl.uniform_matrix_4_f32_slice(
										Some(&gl_state.uniform_locations["view_matrix"]),
										false,
										&model_mat.transpose().0,
									);

									gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(mesh.indices));

									gl.draw_elements(
										mesh.primitive_type,
										mesh.num_indices as i32,
										glow::UNSIGNED_INT,
										0,
									);
								}

								// render the texture to the main buffer target
								gl.use_program(Some(gl_state.offscreen_render_program));
								gl.bind_vertex_array(Some(gl_state.offscreen_render_vao));
								gl.bind_framebuffer(glow::FRAMEBUFFER, painter.intermediate_fbo());

								gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);

								gl.viewport(0, 0, width, height);

								gl.uniform_1_i32(
									// TODO store
									gl.get_uniform_location(gl_state.offscreen_render_program, "t")
										.as_ref(),
									0,
								);

								gl.draw_arrays(glow::TRIANGLES, 0, 3);

								gl.bind_texture(glow::TEXTURE_2D, None);
								gl.delete_framebuffer(fbo);
								gl.delete_texture(ctex);
								gl.delete_renderbuffer(rbd);
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
		let SharedGlState {
			program,
			subsets,
			buffers,
			attribute_objects,
			meshes,
			offscreen_render_program,
			offscreen_render_vao,
			..
		} = &**data;
		unsafe {
			warn!("delete");

			gl.delete_program(*program);

			for (vao, vertices, indices, ..) in subsets {
				gl.delete_vertex_array(*vao);
				gl.delete_buffer(*vertices);
				gl.delete_buffer(*indices);
			}

			for buffer in buffers {
				gl.delete_buffer(*buffer);
			}

			for vao in attribute_objects {
				gl.delete_vertex_array(*vao);
			}

			for mesh in meshes {
				gl.delete_vertex_array(mesh.vao);
				gl.delete_buffer(mesh.indices);
			}

			gl.delete_program(*offscreen_render_program);
			gl.delete_vertex_array(*offscreen_render_vao);
		}
	}
}
