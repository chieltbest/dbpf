// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

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
use rfd::FileHandle;
use tracing::{debug, error, span, Level};

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
	visible: bool,
}

#[derive(Clone, Debug)]
pub struct GMDCEditorStateData {
	// gl: Arc<Context>,
	program: glow::Program,
	subsets: Vec<(
		glow::VertexArray,
		glow::Buffer,
		glow::Buffer,
		usize,
		usize,
		bool,
	)>,

	buffers: Vec<glow::Buffer>,
	attribute_objects: Vec<glow::VertexArray>,
	meshes: Vec<GlMesh>,

	blend_values: [f32; 256],

	offscreen_render_program: glow::Program,
	offscreen_render_vao: glow::VertexArray,

	camera_angle: (f32, f32),
	camera_position: Vertex,
	camera_distance: f32,

	display_mode: i32, // TODO enum
}

#[derive(Debug, Default)]
pub struct GMDCEditorState {
	data: Option<GMDCEditorStateData>,

	save_file_picker: Option<oneshot::Receiver<Option<FileHandle>>>,
}

impl Editor for GeometricDataContainer {
	type EditorState = GMDCEditorState;

	fn new_editor(
		&self,
		_context: &egui::Context,
		gl_context: &Option<Arc<Context>>,
	) -> Self::EditorState {
		if let Some(gl) = gl_context {
			let gl = gl.clone();
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

				let shaders: Vec<_> = shader_sources
					.iter()
					.map(|(shader_type, shader_source)| {
						let shader = gl
							.create_shader(*shader_type)
							.expect("Cannot create shader");
						gl.shader_source(shader, shader_source);
						gl.compile_shader(shader);
						assert!(
							gl.get_shader_compile_status(shader),
							"Failed to compile custom_3d_glow {shader_source}: {}",
							gl.get_shader_info_log(shader)
						);

						shader
					})
					.collect();

				let main_program = gl.create_program().expect("Cannot create program");

				for shader in &shaders[0..2] {
					gl.attach_shader(main_program, *shader);
				}

				gl.link_program(main_program);
				assert!(
					gl.get_program_link_status(main_program),
					"{}",
					gl.get_program_info_log(main_program)
				);

				for shader in &shaders[0..2] {
					gl.detach_shader(main_program, *shader);
					gl.delete_shader(*shader);
				}

				let or_program = gl.create_program().expect("Cannot create program");

				for shader in &shaders[2..4] {
					gl.attach_shader(or_program, *shader);
				}

				gl.link_program(or_program);
				assert!(
					gl.get_program_link_status(or_program),
					"{}",
					gl.get_program_info_log(or_program)
				);

				for shader in &shaders[2..4] {
					gl.detach_shader(or_program, *shader);
					gl.delete_shader(*shader);
				}

				let Some(subsets) = std::iter::once(&self.bounding_mesh)
					.chain(self.dynamic_bounding_mesh.data.iter())
					.map(|subset| {
						let vao = gl
							.create_vertex_array()
							.inspect_err(|err| error!(?err))
							.ok()?;
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

						let vbo = gl.create_buffer().inspect_err(|err| error!(?err)).ok()?;
						gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
						gl.buffer_data_u8_slice(
							glow::ARRAY_BUFFER,
							&subset_vertex_data,
							glow::STATIC_DRAW,
						);
						let position_location =
							gl.get_attrib_location(main_program, "in_position").unwrap(); // TODO unwrap
						gl.vertex_attrib_pointer_f32(
							position_location,
							3,
							glow::FLOAT,
							false,
							3 * 4,
							0,
						);
						gl.enable_vertex_attrib_array(position_location);
						gl.bind_buffer(glow::ARRAY_BUFFER, None);

						let veo = gl.create_buffer().inspect_err(|err| error!(?err)).ok()?;
						gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(veo));
						gl.buffer_data_u8_slice(
							glow::ELEMENT_ARRAY_BUFFER,
							&subset_index_data,
							glow::STATIC_DRAW,
						);

						Some((vao, vbo, veo, subset_num_faces, subset_num_vertices, false))
					})
					.collect::<Option<Vec<_>>>()
				else {
					return GMDCEditorState {
						data: None,
						save_file_picker: None,
					};
				};

				let mut buffers = vec![];

				/*for attr in self.attribute_buffers.iter() {
					if attr.binding.binding_type == AttributeType::DeformMask {
						debug!(?attr.binding);
						debug!(?attr.block_format);
						debug!(?attr.index_set);
						debug!(attr.data.len = attr.data.len() / attr.element_size());
						debug!(?attr.number_elements);
						debug!(attr.references.len = attr.references.len());

						/*let data = attr.data.data.chunks_exact(4)
							.map(|c| f32::from_le_bytes(c.try_into().unwrap()))
							.collect::<Vec<_>>();
						let data_chunks = data.chunks_exact(3)
							.collect::<Vec<_>>();*/
						eprintln!("attr.data.data = {:?}", attr.data.data.chunks_exact(4).collect::<Vec<_>>());
					}
				}*/

				let attribute_objects = self
					.attribute_groups
					.iter()
					.map(|group| {
						let vao = gl
							.create_vertex_array()
							.inspect_err(|err| error!(?err))
							.ok()?;
						gl.bind_vertex_array(Some(vao));

						span!(Level::DEBUG, "group");

						debug!(?group.number_elements);
						debug!(?group.referenced_active);

						debug!(?group.vertex_indices.data);
						debug!(?group.normal_indices.data);
						debug!(?group.uv_indices.data);

						let (buffer_it, stride, attributes) = group.construct_interleaved(self);
						let buffer = buffer_it.flatten().copied().collect::<Vec<_>>();

						let vbo = gl.create_buffer().inspect_err(|err| error!(?err)).ok()?;
						buffers.push(vbo);
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
								AttributeType::RegionMask => {
									("in_region_mask", glow::UNSIGNED_BYTE)
								}
								AttributeType::DeformMask => {
									("in_deform_mask", glow::UNSIGNED_BYTE)
								}
							};
							let location = gl.get_attrib_location(main_program, attr_binding_name);
							if let Some(location) = location {
								// let attr_binding = location + attr.binding.binding_slot;
								let attr_binding = location;
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
						}

						gl.bind_buffer(glow::ARRAY_BUFFER, None);

						Some(vao)
					})
					.collect::<Vec<_>>();

				let meshes = self
					.meshes
					.iter()
					.map(|mesh| {
						attribute_objects[mesh.attribute_group_index as usize].and_then(|vao| {
							let indices =
								gl.create_buffer().inspect_err(|err| error!(?err)).ok()?;

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

							Some(GlMesh {
								vao,
								primitive_type,
								indices,
								num_indices,
								visible: true,
							})
						})
					})
					.collect::<Vec<_>>();

				let Ok(or_vao) = gl.create_vertex_array().inspect_err(|err| error!(?err)) else {
					return GMDCEditorState {
						data: None,
						save_file_picker: None,
					};
				};

				gl.bind_vertex_array(None);

				GMDCEditorState {
					data: Some(GMDCEditorStateData {
						// gl,
						program: main_program,
						subsets,

						buffers,
						attribute_objects: attribute_objects.into_iter().flatten().collect(),
						meshes: meshes.into_iter().flatten().collect(),

						blend_values: [0.0; 256],

						offscreen_render_program: or_program,
						offscreen_render_vao: or_vao,

						camera_angle: (std::f32::consts::PI, 0.0),
						camera_position: Vertex {
							x: 0.0,
							y: -1.0,
							z: 0.0,
						},
						camera_distance: 1.0,

						display_mode: 0,
					}),

					save_file_picker: None,
				}
			}
		} else {
			GMDCEditorState {
				data: None,
				save_file_picker: None,
			}
		}
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		if let Some(picker) = &mut state.save_file_picker {
			if let Ok(Some(handle)) = picker.try_recv() {
				state.save_file_picker = None;
				if let Some(handle) = handle {
					// let mut cur = Cursor::new(vec![]);
					let gltf = self.export_gltf();
					if let Some(gltf) = gltf {
						// gltf.write_file(handle.path()).unwrap(); // TODO proper error
						let res = futures::executor::block_on(handle.write(&gltf.0));
						if let Err(e) = res {
							error!(?e);
						}
					}
				}
			}
		}

		if let Some(gl_state) = &mut state.data {
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
								for (mesh, gl_mesh) in
									self.meshes.iter_mut().zip(gl_state.meshes.iter_mut())
								{
									ui.horizontal(|ui| {
										ui.add(egui::Checkbox::without_text(&mut gl_mesh.visible));

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
												let attr =
													&self.attribute_buffers[attr_idx.0 as usize];
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
									for (name, value) in self
										.blend_group_bindings
										.iter_mut()
										.zip(&mut gl_state.blend_values)
									{
										ui.horizontal(|ui| {
											ui.add(
												egui::Slider::new(value, 0.0..=1.0)
													.clamping(SliderClamping::Never),
											);
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

				for (i, name) in ["normals", "tangents", "uv", "depth", "wireframe"]
					.into_iter()
					.enumerate()
				{
					ui.radio_value(&mut gl_state.display_mode, i as i32, name);
				}
			});

			egui::Frame::canvas(ui.style())
				.show(ui, |ui| {
					let (rect, response) = ui
						.allocate_exact_size(ui.available_size_before_wrap(), egui::Sense::drag());

					let inverse_orientation = Mat4::rotation_y(-gl_state.camera_angle.0)
						* Mat4::rotation_x(-gl_state.camera_angle.1);

					let drag_delta = response.drag_delta() / rect.height() * 2.0;
					if ui.input(|i| i.pointer.button_down(PointerButton::Primary)) {
						gl_state.camera_angle.0 += drag_delta.x * std::f32::consts::PI;
						gl_state.camera_angle.1 -= drag_delta.y * std::f32::consts::FRAC_PI_2;
					}
					if ui.input(|i| i.pointer.button_down(PointerButton::Secondary)) {
						gl_state.camera_position += inverse_orientation
							* Vertex {
								x: drag_delta.x,
								y: -drag_delta.y,
								z: 0.0,
							};
					}
					if ui.input(|i| i.pointer.button_down(PointerButton::Middle)) {
						gl_state.camera_position += inverse_orientation
							* Vertex {
								x: drag_delta.x,
								y: 0.0,
								z: -drag_delta.y,
							};
					}

					if response.hovered() {
						let scroll_delta =
							ui.input(|i| i.smooth_scroll_delta) / rect.height() * 2.0;
						gl_state.camera_position += inverse_orientation
							* Vertex {
								x: scroll_delta.x,
								y: 0.0,
								z: -scroll_delta.y,
							};
					}

					let state = gl_state.clone();
					// let transforms = self.bones.clone();
					let dark_mode = ui.style().visuals.dark_mode;

					let cb = egui_glow::CallbackFn::new(move |info, painter| {
						let gl = painter.gl();
						// let viewport = info.viewport_in_pixels();
						// let clip = info.clip_rect_in_pixels();
						let [width, height] = info.screen_size_px.map(|u| u as i32);
						// let width = viewport.width_px;
						// let height = viewport.height_px;
						unsafe {
							gl.use_program(Some(state.program));

							/*// TODO opengl error handling
							let err = gl.get_error();
							if err != glow::NO_ERROR {
								eprintln!("s {err:?}");
							}*/

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
										z: state.camera_distance,
									}) * Mat4::rotation_x(state.camera_angle.1)
									* Mat4::rotation_y(state.camera_angle.0)
									* Mat4::translation(state.camera_position)
									* Mat4::identity().swap_axes(1, 2);

							// let ident_transform = Transform::identity();

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

							gl.uniform_1_f32_slice(
								gl.get_uniform_location(state.program, "blend_values")
									.as_ref(),
								&state.blend_values,
							);

							let identity_bones = std::iter::repeat_n(Mat4::identity(), 256)
								.flat_map(|eye| eye.0)
								.collect::<Vec<_>>();

							gl.uniform_matrix_4_f32_slice(
								gl.get_uniform_location(state.program, "bones").as_ref(),
								false,
								&identity_bones,
							);

							let display_mode = if state.display_mode <= 3 {
								gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
								state.display_mode
							} else {
								gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
								3
							};

							gl.uniform_1_i32(
								gl.get_uniform_location(state.program, "display_mode")
									.as_ref(),
								display_mode,
							);

							gl.uniform_1_i32(
								gl.get_uniform_location(state.program, "dark_mode").as_ref(),
								dark_mode as i32,
							);

							for mesh in state.meshes.iter().filter(|m| m.visible) {
								// TODO bone bindings

								gl.bind_vertex_array(Some(mesh.vao));

								gl.vertex_attrib_4_f32(
									gl.get_attrib_location(state.program, "in_blend_weights")
										.unwrap(),
									1.0,
									1.0,
									1.0,
									1.0,
								);

								gl.uniform_matrix_4_f32_slice(
									gl.get_uniform_location(state.program, "view_matrix")
										.as_ref(),
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
							gl.use_program(Some(state.offscreen_render_program));
							gl.bind_vertex_array(Some(state.offscreen_render_vao));
							gl.bind_framebuffer(glow::FRAMEBUFFER, painter.intermediate_fbo());

							gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);

							gl.viewport(0, 0, width, height);

							gl.uniform_1_i32(
								gl.get_uniform_location(state.offscreen_render_program, "t")
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
		} else {
			ui.label("Something went wrong in OpenGL initialization, does this device have OpenGL support?")
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
