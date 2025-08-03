use std::collections::{BTreeMap, BTreeSet};
use std::f32::consts::FRAC_1_SQRT_2;
use std::iter;
use gltf_kun::extensions::DefaultExtensions;
use gltf_kun::graph::gltf::{Accessor, GltfDocument, Node};
use gltf_kun::graph::{Graph, GraphNodeWeight};
use gltf_kun::graph::gltf::accessor::{ComponentType, Type};
use gltf_kun::graph::gltf::accessor::iter::AccessorIter;
use gltf_kun::graph::gltf::node::{Quat, Vec3};
use gltf_kun::graph::gltf::primitive::{Mode, Semantic};
use gltf_kun::io::format::glb::{GlbExport, GlbFormat};
use serde_json::{Number, Value};
use itertools::Itertools;
use crate::internal_file::resource_collection::geometric_data_container::{BlockFormat, AttributeType, GeometricDataContainer, PrimitiveType, BoundingMesh, AttributeBuffer};
use crate::internal_file::resource_collection::geometric_data_container::math::Mat4;

impl GeometricDataContainer {
    pub fn export_gltf(&self) -> Option<GlbFormat> {
        eprintln!("self.file_name = {:#?}", self.file_name);

        let mut graph = Graph::default();

        let doc = GltfDocument::new(&mut graph);

        let accessors: Vec<_> = self.attribute_buffers.iter().map(|element| {
            // eprintln!("element = {:#?}", element);

            if element.data.is_empty() {
                return None;
            }

            let mut gltf_accessor = match element.binding.binding_type {
                AttributeType::BoneWeights => {
                    // expand weights to vec4
                    Accessor::from_iter(&mut graph, AccessorIter::new(
                        &element.data
                            .chunks_exact(4 * element.block_format.num_components())
                            .flat_map(|e| {
                                e.iter()
                                    .copied()
                                    .chain(iter::repeat(0.0f32.to_le_bytes())
                                        .flatten())
                                    .take(4 * size_of::<f32>())
                            })
                            .collect::<Vec<_>>(),
                        ComponentType::F32,
                        Type::Vec4,
                        false,
                    ).unwrap()) // TODO error handling
                }
                AttributeType::BoneKeys => {
                    // bone indices have to be rebuilt using the mesh's bone bindings
                    return None;
                }
                AttributeType::Tangents | AttributeType::TangentDeltas => {
                    // tangents need their handedness specified in the w component
                    // gmdc meshes only have one handedness
                    debug_assert!(element.block_format == BlockFormat::F32Vec3);

                    Accessor::from_iter(&mut graph, AccessorIter::new(
                        &element.data
                            .chunks_exact(4 * 3)
                            .flat_map(|e| {
                                e.iter()
                                    .copied()
                                    .chain(if element.binding.binding_type == AttributeType::Tangents {
                                        1.0f32
                                    } else {
                                        0.0f32
                                    }.to_le_bytes().into_iter())
                            })
                            .collect::<Vec<_>>(),
                        ComponentType::F32,
                        Type::Vec4,
                        false,
                    ).unwrap()) // TODO error handling
                }
                _ => {
                    let (component_type, element_type) = match element.block_format {
                        BlockFormat::F32Scalar => (ComponentType::F32, Type::Scalar),
                        BlockFormat::F32Vec2 => (ComponentType::F32, Type::Vec2),
                        BlockFormat::F32Vec3 => (ComponentType::F32, Type::Vec3),
                        BlockFormat::U8Vec4 => (ComponentType::U8, Type::Vec4),
                    };

                    Accessor::from_iter(&mut graph, AccessorIter::new(
                        &element.data,
                        component_type,
                        element_type,
                        false,
                    ).unwrap()) // TODO error handling
                }
            };

            doc.add_accessor(&mut graph, gltf_accessor);

            let accessor_w = gltf_accessor.get_mut(&mut graph);
            accessor_w.name = Some(format!(
                "{:?}_{}: {:?} ({:?})",
                element.binding.binding_type,
                element.binding.binding_slot,
                element.block_format,
                element.index_set,
            ));

            Some(gltf_accessor)
        }).collect();


        let model_name = String::from_utf8_lossy(&self.file_name.name.0).to_string();

        let mut gltf_scene = doc.create_scene(&mut graph);
        doc.set_default_scene(&mut graph, Some(gltf_scene));

        let scene_w = gltf_scene.get_mut(&mut graph);
        scene_w.name = Some(model_name.clone());

        let mut gltf_base_node = doc.create_node(&mut graph);
        let base_node_w = gltf_base_node.get_mut(&mut graph);

        base_node_w.name = Some(model_name);
        base_node_w.rotation = Quat::from_xyzw(0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0);

        gltf_scene.add_node(&mut graph, gltf_base_node);

        fn make_bounding_mesh(doc: &GltfDocument, graph: &mut Graph, b_mesh: &BoundingMesh, name: String) -> Node {
            let mut gltf_node = doc.create_node(graph);
            let node_w = gltf_node.get_mut(graph);

            node_w.name = Some(name.clone());

            if !b_mesh.vertices.is_empty() {
                let mut gltf_mesh = doc.create_mesh(graph);
                let mesh_w = gltf_mesh.get_mut(graph);

                mesh_w.name = Some(name);

                let mut gltf_primitive = gltf_mesh.create_primitive(graph);
                let primitive_w = gltf_primitive.get_mut(graph);

                primitive_w.mode = Mode::Triangles;

                let vertices_accessor = Accessor::from_iter(graph, AccessorIter::new(
                    &b_mesh.vertices
                        .iter()
                        .flat_map(|v| [
                            v.x.to_le_bytes(),
                            v.y.to_le_bytes(),
                            v.z.to_le_bytes(),
                        ])
                        .flatten()
                        .collect::<Vec<_>>(),
                    ComponentType::F32,
                    Type::Vec3,
                    false,
                ).unwrap()); // TODO error handling
                doc.add_accessor(graph, vertices_accessor);
                gltf_primitive.set_attribute(graph, Semantic::Positions, Some(vertices_accessor));

                let indices_accessor = Accessor::from_iter(graph, AccessorIter::new(
                    &b_mesh.faces
                        .iter()
                        .flat_map(|i| i.0.to_le_bytes())
                        .collect::<Vec<_>>(),
                    ComponentType::U32,
                    Type::Scalar,
                    false,
                ).unwrap()); // TODO error handling
                doc.add_accessor(graph, indices_accessor);
                gltf_primitive.set_indices(graph, Some(indices_accessor));

                gltf_node.set_mesh(graph, Some(gltf_mesh));
            }

            gltf_node
        }

        let mut bone_nodes = vec![];

        if !self.bounding_mesh.vertices.is_empty() || !self.dynamic_bounding_mesh.is_empty() {
            let main_bounding_mesh = make_bounding_mesh(
                &doc,
                &mut graph,
                &self.bounding_mesh,
                "b_mesh".to_string());
            gltf_base_node.add_child(&mut graph, &main_bounding_mesh);

            bone_nodes = self.dynamic_bounding_mesh.iter().enumerate().map(|(bone_idx, b_mesh)| {
                let mut bone_bounding_mesh = make_bounding_mesh(
                    &doc,
                    &mut graph,
                    b_mesh,
                    format!("bone#{bone_idx}"));

                main_bounding_mesh.add_child(&mut graph, &bone_bounding_mesh);

                if let Some(bone) = self.bones.get(bone_idx) {
                    let inverse = bone.inverse();
                    let r = inverse.rotation;
                    let t = inverse.translation;

                    let node_w = bone_bounding_mesh.get_mut(&mut graph);
                    node_w.rotation = Quat::from_xyzw(r.x, r.y, r.z, r.w);
                    node_w.translation = Vec3::new(t.x, t.y, t.z);
                }

                bone_bounding_mesh
            }).collect();
        }
        // TODO add nodes for remaining bone translations

        let gltf_skin = if !bone_nodes.is_empty() {
            let mut gltf_skin = doc.create_skin(&mut graph);
            let skin_w = gltf_skin.get_mut(&mut graph);

            skin_w.name = Some("Armature".to_string());

            for (idx, bone) in bone_nodes.iter().enumerate() {
                gltf_skin.add_joint(&mut graph, bone, idx);
            }

            let inverse_transforms_accessor = Accessor::from_iter(
                &mut graph,
                AccessorIter::new(
                    &self.bones
                        .iter()
                        .map(|bone|
                            Mat4::transform(*bone)
                                .transpose() // gltf matrices are column-major
                                .0)
                        .flat_map(|mat| {
                            mat.into_iter()
                                .flat_map(|f| f.to_le_bytes())
                        }).collect::<Vec<_>>(),
                    ComponentType::F32,
                    Type::Mat4,
                    false,
                ).unwrap()); // TODO error handling
            doc.add_accessor(&mut graph, inverse_transforms_accessor);
            gltf_skin.set_inverse_bind_matrices(&mut graph, Some(inverse_transforms_accessor));

            Some(gltf_skin)
        } else {
            None
        };


        for group in self.meshes.iter() {
            let group_name = String::from_utf8_lossy(&group.name.0).to_string();

            eprintln!("group_name = {:?}", group_name);

            let mut gltf_mesh = doc.create_mesh(&mut graph);

            let mut gltf_primitive = gltf_mesh.create_primitive(&mut graph);
            let primitive_w = gltf_primitive.get_mut(&mut graph);

            primitive_w.mode = match group.primitive_type {
                PrimitiveType::Points => Mode::Points,
                PrimitiveType::Lines => Mode::Lines,
                PrimitiveType::Triangles => Mode::Triangles,
            };

            let indices_accessor = Accessor::from_iter(&mut graph, AccessorIter::new(
                &group.indices
                    .iter()
                    .flat_map(|i| i.0.to_le_bytes())
                    .collect::<Vec<_>>(),
                ComponentType::U32,
                Type::Scalar,
                false,
            ).unwrap()); // TODO error handling
            doc.add_accessor(&mut graph, indices_accessor);
            gltf_primitive.set_indices(&mut graph, Some(indices_accessor));

            let link = &self.attribute_groups[group.attribute_group_index as usize];

            let blend_keys = link.attributes.iter()
                .map(|idx| &self.attribute_buffers[idx.0 as usize])
                .filter(|attr|
                    attr.binding.binding_type == AttributeType::BlendKeys)
                .collect::<Vec<_>>();
            let active_morphs = blend_keys
                .iter()
                .flat_map(|keys|
                    keys.data.iter()
                        .copied())
                .collect::<BTreeSet<_>>();
            let mut morph_to_buffer = active_morphs.iter()
                .copied()
                .enumerate()
                .map(|(i, m)| (m, (i,
                                   gltf_primitive.create_morph_target(&mut graph, i),
                                   BTreeMap::<AttributeType, Vec<u8>>::new())))
                .collect::<BTreeMap<_, _>>();

            let mut process_delta = |delta: &AttributeBuffer| {
                let b_type = delta.binding.binding_type;
                let b_slot = delta.binding.binding_slot;

                let delta_data_chunk_size = delta.block_format.num_components() * size_of::<f32>();

                // find the blend keys object that corresponds to this buffer
                let keys = blend_keys.iter().rev()
                    .find(|keys| keys.binding.binding_slot == b_slot / 4);
                // we can't make sense of morph deltas that don't have blend keys
                if let Some(keys) = keys {
                    let keys = keys.data
                        .chunks_exact(4)
                        .map(|chunk| chunk[(b_slot % 4) as usize]);

                    // go through all morph targets and update them with new data
                    for (buffer_key, (_, _, buffer)) in morph_to_buffer.iter_mut() {
                        if keys.clone().contains(buffer_key) {
                            // this buffer needs to get the new data, see if a buffer already exists
                            if !buffer.contains_key(&b_type) {
                                buffer.insert(b_type,
                                              iter::repeat(0.0f32.to_le_bytes())
                                                  .flatten()
                                                  .take(delta.data.len())
                                                  .collect());
                            }
                            keys.clone().zip(delta.data.chunks_exact(delta_data_chunk_size))
                                .zip(buffer.get_mut(&b_type).unwrap().chunks_exact_mut(delta_data_chunk_size))
                                .for_each(|((key, delta), buf)| {
                                    if key == *buffer_key {
                                        buf.copy_from_slice(delta);
                                    }
                                });
                        }
                    }
                }
            };

            for attr_idx in link.attributes.iter() {
                let attr = &self.attribute_buffers[attr_idx.0 as usize];
                eprintln!("attr = {:#?}", attr);
                let binding_idx = attr.binding.binding_slot;

                let identity = match attr.binding.binding_type {
                    AttributeType::Positions => Semantic::Positions,
                    AttributeType::Normals => Semantic::Normals,
                    AttributeType::TexCoords => Semantic::TexCoords(binding_idx),
                    AttributeType::Tangents => Semantic::Tangents,

                    AttributeType::BlendValues2 => Semantic::Extras(format!("TargetIndices_{binding_idx}")),
                    AttributeType::VertexID => Semantic::Extras(format!("VertexID_{binding_idx}")),
                    AttributeType::RegionMask => Semantic::Extras(format!("RegionMask_{binding_idx}")),

                    // skins/bones
                    AttributeType::BoneWeights => Semantic::Weights(binding_idx),
                    AttributeType::BoneKeys => {
                        // rebuild the bone keys using this mesh's bone bindings
                        let bone_keys_accessor = Accessor::from_iter(&mut graph, AccessorIter::new(
                            &attr.data
                                .iter()
                                .flat_map(|i| if *i == 0xff {
                                    0u16.to_le_bytes()
                                } else {
                                    // TODO handle bone idx out of range
                                    (group.bone_references[*i as usize].0 as u16).to_le_bytes()
                                })
                                .collect::<Vec<_>>(),
                            ComponentType::U16,
                            Type::Vec4,
                            false,
                        ).unwrap()); // TODO error handling
                        doc.add_accessor(&mut graph, bone_keys_accessor);

                        gltf_primitive.set_attribute(&mut graph, Semantic::Joints(binding_idx), Some(bone_keys_accessor));

                        continue;
                    }

                    // morph targets/blends
                    // TODO store blend data in morph targets array
                    // AttributeType::BlendIndices => Semantic::Extras(format!("BlendIndices_{binding_idx}")),
                    // AttributeType::BlendWeights => Semantic::Extras(format!("BlendWeights_{binding_idx}")),
                    // AttributeType::DeformMask => Semantic::Extras(format!("DeformMask_{binding_idx}")),
                    _ => {
                        match attr.binding.binding_type {
                            AttributeType::PositionDeltas |
                            AttributeType::NormalDeltas |
                            AttributeType::TangentDeltas => {
                                process_delta(attr);
                            }
                            _ => {}
                        }
                        continue;
                    }
                };

                gltf_primitive.set_attribute(&mut graph, identity, accessors[attr_idx.0 as usize]);
            }

            let num_morphs = morph_to_buffer.len();
            let morph_names = morph_to_buffer.iter()
                .map(|(id, (i, _, _))| {
                    let name = self.blend_group_bindings.get(*id as usize)
                        .map(|binding| format!("{}::{}", binding.blend_group, binding.element))
                        .unwrap_or_else(|| "".to_string());

                    (i, Value::String(name))
                })
                .sorted_by_key(|(i, _)| *i)
                .map(|(_, v)| v)
                .collect();

            for (key, (weight_idx, morph, attributes)) in morph_to_buffer {
                for (attr_type, buf) in attributes {
                    let (identity, components) = match attr_type {
                        AttributeType::PositionDeltas => (Semantic::Positions, Type::Vec3),
                        AttributeType::NormalDeltas => (Semantic::Normals, Type::Vec3),
                        AttributeType::TangentDeltas => (Semantic::Tangents, Type::Vec3),
                        _ => { continue; }
                    };

                    let accessor = Accessor::from_iter(&mut graph, AccessorIter::new(
                        &buf,
                        ComponentType::F32,
                        components,
                        false,
                    ).unwrap()); // TODO error handling
                    doc.add_accessor(&mut graph, accessor);

                    morph.set_attribute(&mut graph, identity, Some(accessor));
                }
            }

            let mesh_w = gltf_mesh.get_mut(&mut graph);

            mesh_w.name = Some(group_name.clone());
            mesh_w.weights = vec![0.0; num_morphs];
            let _ = mesh_w.extras.insert(*Box::new(
                serde_json::value::to_raw_value(&Value::Object(
                    [("targetNames".to_string(), Value::Array(morph_names)),
                        ("opacity".to_string(), Value::Number(Number::from(group.opacity)))]
                        .into_iter().collect())).unwrap_or_default()));

            let mut gltf_node = doc.create_node(&mut graph);
            let node_w = gltf_node.get_mut(&mut graph);

            node_w.name = Some(group_name);

            gltf_node.set_mesh(&mut graph, Some(gltf_mesh));
            gltf_node.set_skin(&mut graph, gltf_skin);

            gltf_base_node.add_child(&mut graph, &gltf_node);
        }

        GlbExport::<DefaultExtensions>::export(&mut graph, &doc).ok()
    }
}
