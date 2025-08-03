use std::fmt::{Debug, Formatter};
use crate::internal_file::resource_collection::geometric_data_container::math::{Vertex, Transform};
use std::io::{Read, Seek, Write};
use binrw::{args, binrw, BinRead, BinResult, BinWrite, Endian, NamedArgs};
use itertools::Either::{Left, Right};
use crate::common::{BigString, SizedVec};
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

pub mod math;
pub mod gltf;

#[derive(Copy, Clone, Default, PartialEq)]
pub struct Reference(
    // either u16 or u32 depending on the resource version
    pub u32
);

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum AttributeType {
    #[default]
    /// normally F32 Vec3
    Positions = 0x5B830781,
    /// normally F32 Vec3
    PositionDeltas = 0x5CF2CFE1,
    /// normally F32 Vec3
    Normals = 0x3B83078B,
    /// normally F32 Vec3
    NormalDeltas = 0xCB6F3A6A,
    /// the handedness is assumed to be universal
    /// normally F32 Vec3
    Tangents = 0x89D92BA0,
    /// normally F32 Vec3
    TangentDeltas = 0x69D92B93,

    /// normally F32 Vec2
    TexCoords = 0xBB8307AB,

    /// morph target ids in groups of 4; in the vertex shader these components are read as (x, y, z, w)
    /// these ids are used to index the blend weights bindings,
    /// after which the weights are multiplied by the [Position|Normals|Tangents]Deltas
    /// to get the corresponding final attributes
    /// normally U8 Vec4
    BlendKeys = 0xDCF2CFDC,

    /// specifies the weight of specific bones for this vertex
    /// a bind slot can have up to tree (four?) values per vertex
    /// normally F32 Vec1/2/3
    BoneWeights = 0x3BD70105,
    /// bone ids in groups of 4; in the vertex shader these components are read as (x, y, z, w)
    /// id 0xff is interpreted as no binding
    /// these ids are then used to index into the bone weight bindings
    /// and finally multiplied by the bone matrix bindings
    /// normally U8 Vec4
    BoneKeys = 0xFBD70111,

    /// one binding per morph
    /// most likely the original position deltas per morph
    /// normally F32 Vec3
    BlendValues1 = 0x1C4AFC56,
    /// one binding per morph
    /// most likely the original normal deltas per morph
    /// normally F32 Vec3
    BlendValues2 = 0x7C4DEE82,
    /// bone weights???
    /// one binding per bone
    /// normally F32 Scalar
    BoneValues = 0x5C4AFC5C,

    /// observed values are 51, 76, 89, 101, 102, 255
    /// some sort of flags?
    /// normally U8 Vec4
    DeformMask = 0xDB830795,

    /// related to RegionMask?
    /// normally U8 Vec4
    VertexID = 0x114113C3,
    /// related to VertexID?
    /// normally U8 Vec4
    RegionMask = 0x114113CD,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct AttributeBinding {
    pub binding_type: AttributeType,
    /// the shader bind slot
    /// some attributes (like position or normals) cannot have more than one bind slot,
    /// so they should always have a bind slot of 0
    pub binding_slot: u32,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BlockFormat {
    #[default]
    F32Scalar = 0,
    F32Vec2 = 1,
    F32Vec3 = 2,
    // TODO F32Vec4 = 3 ?
    U8Vec4 = 4,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum IndexSet {
    #[default]
    Main = 0,
    Norms = 1,
    UV = 2,
    None = 3,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum PrimitiveType {
    Points = 0,
    Lines = 1,
    #[default]
    Triangles = 2,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttributeBuffer {
    /// the number of elements this buffer contains
    /// can be derived independently with `data.len() / block_format.size()`
    pub number_elements: u32,

    pub binding: AttributeBinding,

    /// the data type and size for this attribute buffer
    pub block_format: BlockFormat,

    /// the indices to use while parsing this buffer
    pub index_set: IndexSet,

    pub data: SizedVec<u32, u8>,

    #[brw(args{ version })]
    pub references: SizedVec<u32, Reference>,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttributeGroup {
    /// the list of attribute buffers that belong to this binding group
    #[brw(args{ version })]
    pub attributes: SizedVec<u32, Reference>,

    /// number of elements that can be referenced in the indices
    /// all referenced attributes should have this number of elements, either directly or through indexing
    pub number_elements: u32,
    // number of attributes active?
    pub referenced_active: u32,

    #[brw(args{ version })]
    pub vertex_indices: SizedVec<u32, Reference>,
    #[brw(args{ version })]
    pub normal_indices: SizedVec<u32, Reference>,
    #[brw(args{ version })]
    pub uv_indices: SizedVec<u32, Reference>,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub primitive_type: PrimitiveType,

    /// index of the set of attribute bindings used by this mesh
    /// multiple meshes can use the same attribute groups, which is why it is not directly included
    pub attribute_group_index: u32,

    pub name: BigString,

    /// the list that would be referred to as "vertex element object" in opengl
    /// used as indices/elements in "glDrawElements*"
    #[brw(args{ version })]
    pub indices: SizedVec<u32, Reference>,

    /// sets the order that transparent objects are rendered in
    pub opacity: i32,

    /// used to bind bone matrices to the vertex shader
    /// each element refers to a bone that will be bound in the shader at that element's index
    /// meaning if for instance there is an array [1, 2, 4, 0]
    /// the second (1) bone will be bound to slot 0,
    /// the third (2) to slot 1,
    /// the fifth (4) to slot 2,
    /// the first (0) to slot 3.
    #[brw(args{ version }, if(version > ResourceBlockVersion::V1))]
    pub bone_references: SizedVec<u32, Reference>,
}

#[binrw]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendGroupBinding {
    pub blend_group: BigString,
    pub element: BigString,
}

/// A simple mesh used for selection testing.
/// This mesh does not have any attributes as its only purpose is to test if a model is being hovered.
/// This bounding mesh normally does not include visual effects such as shadows or particles.
/// For very large background objects (such s grass) it may be desirable to have a smaller bounding mesh.
#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoundingMesh {
    #[br(temp)]
    #[bw(calc = vertices.len() as u32)]
    vertex_count: u32,

    #[br(temp, if(vertex_count > 0))]
    #[bw(calc = faces.len() as u32, if(!vertices.is_empty()))]
    face_count: u32,

    #[br(count = vertex_count)]
    pub vertices: Vec<Vertex>,

    #[br(args{ count: face_count as usize, inner: args! { version } }, if(vertex_count > 0))]
    #[bw(args{ version }, if(!vertices.is_empty()))]
    pub faces: Vec<Reference>,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometricDataContainer {
    pub file_name: FileName,

    /// the raw data buffers for storing vertex attributes
    /// corresponds to "Buffer Object" in OpenGL
    #[brw(args{ version })]
    pub attribute_buffers: SizedVec<u32, AttributeBuffer>,

    /// the collection of buffers that make up a collection of vertex attributes
    /// corresponds to "Vertex Array Object" in OpenGL (minus element array)
    #[brw(args{ version })]
    pub attribute_groups: SizedVec<u32, AttributeGroup>,

    /// the set of models in this mesh
    /// every model corresponds to a single "glDrawElements" call in OpenGL
    #[brw(args{ version })]
    pub meshes: SizedVec<u32, Mesh>,

    pub bones: SizedVec<u32, Transform>,

    pub blend_group_bindings: SizedVec<u32, BlendGroupBinding>,

    /// the mesh used in the selection test
    /// to determine what object is being hovered the bounding meshes of all objects
    /// are rendered instead of the full mesh, then the hovered pixel is examined to get
    /// the corresponding object
    #[brw(args{ version })]
    pub bounding_mesh: BoundingMesh,

    /// same as [bounding_mesh](Self::bounding_mesh),
    /// but these meshes move with the [bones](Self::bones) in the model
    #[brw(args{ version })]
    pub dynamic_bounding_mesh: SizedVec<u32, BoundingMesh>,
}


#[derive(NamedArgs, Clone, Debug, Default)]
pub struct ReferenceBinArgs {
    pub version: ResourceBlockVersion,
}

impl BinRead for Reference {
    type Args<'a> = ReferenceBinArgs;

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        if args.version < ResourceBlockVersion::V4 {
            Ok(Self(u32::read_le(reader)?))
        } else {
            Ok(Self(u16::read_le(reader)? as u32))
        }
    }
}

impl BinWrite for Reference {
    type Args<'a> = ReferenceBinArgs;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        if args.version < ResourceBlockVersion::V4 {
            self.0.write_le(writer)
        } else {
            (self.0 as u16).write_le(writer)
        }
    }
}

impl Debug for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl BlockFormat {
    pub fn num_components(&self) -> usize {
        match self {
            BlockFormat::F32Scalar => 1,
            BlockFormat::F32Vec2 => 2,
            BlockFormat::F32Vec3 => 3,
            BlockFormat::U8Vec4 => 4,
        }
    }

    pub fn component_size(&self) -> usize {
        match self {
            BlockFormat::F32Scalar |
            BlockFormat::F32Vec2 |
            BlockFormat::F32Vec3 => size_of::<f32>(),
            BlockFormat::U8Vec4 => size_of::<u8>(),
        }
    }

    pub fn element_size(&self) -> usize {
        let components = self.num_components();
        let component_size = self.component_size();
        components * component_size

    }
}

impl AttributeBuffer {
    pub fn element_size(&self) -> usize {
        self.block_format.element_size()
    }

    pub fn elements(&self) -> impl Iterator<Item=&[u8]> {
        self.data.chunks_exact(self.element_size())
    }

    pub fn elements_indexed<'a, 'b, T: Iterator<Item=&'b Reference>>(&'a self, index: T)
        -> impl Iterator<Item=&'a [u8]> + use < 'a, 'b, T > {
        index.map(|i| {
            let element_size = self.element_size();
            let base = (i.0 as usize) * element_size;
            &self.data[base..base + element_size]
        })
    }
}

struct MultiZip<T: Iterator> {
    iters: Vec<T>,
    i: usize,
}

impl<T: Iterator> MultiZip<T> {
    fn new(iters: Vec<T>) -> Self {
        Self {
            iters,
            i: 0,
        }
    }
}

impl<T: Iterator> Iterator for MultiZip<T> {
    type Item = <T as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iters[self.i].next();
        self.i = (self.i + 1) % self.iters.len();
        next
    }
}

impl AttributeGroup {
    pub fn get_buffer_data<'a, 'b>(&'a self, attribute: &'b AttributeBuffer)
        -> impl Iterator<Item=&'b [u8]> + use < 'a, 'b > {
        let indices = match attribute.index_set {
            IndexSet::Main => Some(&self.vertex_indices),
            IndexSet::Norms => Some(&self.normal_indices),
            IndexSet::UV => Some(&self.uv_indices),
            IndexSet::None => None,
        }
            // some implementations write data that has a fixed index set but empty indices, handle this case
            .filter(|indices| !indices.is_empty());

        if let Some(indices) = indices {
            Left(attribute.elements_indexed(indices.iter()))
        } else {
            Right(attribute.elements())
        }
    }

    /// returns the data, stride, and offset + size + components for each attribute
    pub fn construct_interleaved<'a, 'b>(&'a self, gmdc: &'b GeometricDataContainer)
        -> (impl Iterator<Item=&'b [u8]> + use < 'a, 'b >, usize, Vec<(&'b AttributeBuffer, usize)>) {
        let mut attribute_offset = 0;
        let attributes = self.attributes.iter()
            .filter_map(|attr_idx| {
                let attr = &gmdc.attribute_buffers[attr_idx.0 as usize];
                match attr.binding.binding_type {
                    AttributeType::BlendValues2 |
                    AttributeType::BlendValues1 |
                    AttributeType::BoneValues |
                    AttributeType::DeformMask |
                    AttributeType::VertexID |
                    AttributeType::RegionMask => None,
                    _ => {
                        let element_size = attr.element_size();
                        let num_components = attr.block_format.num_components();
                        let offset = attribute_offset;
                        attribute_offset += element_size;
                        Some((attr, offset))
                    }
                }
            })
            .collect::<Vec<_>>();

        let interleaved = MultiZip::new(attributes.iter()
            .map(|(attr, _)| {
                self.get_buffer_data(attr)
            })
            .collect());

        let stride = attribute_offset;

        (interleaved, stride, attributes)
    }
}

impl Mesh {
    pub fn poly_count(&self) -> usize {
        match self.primitive_type {
            // every point is rendered as 2 triangles
            PrimitiveType::Points => self.indices.len() * 2,
            // same as for lines, but they require 2 indices each
            PrimitiveType::Lines => self.indices.len(),
            // triangles have 3 corners (indices)
            PrimitiveType::Triangles => self.indices.len() / 3,
        }
    }
}
