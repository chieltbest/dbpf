use std::io::{Read, Seek, Write};
use binrw::{args, binrw, BinRead, BinResult, BinWrite, Endian, NamedArgs};
use crate::common::BigString;
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

#[binrw]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[binrw]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reference(pub u32);

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

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ElementIdentity {
    BlendIndices = 0x1C4AFC56,
    BlendWeights = 0x5C4AFC5C,
    TargetIndices = 0x7C4DEE82,
    NormalDeltas = 0xCB6F3A6A,
    #[default]
    Vertices = 0x5B830781,
    VertexDeltas = 0x5CF2CFE1,
    VertexMap = 0xDCF2CFDC,
    VertexID = 0x114113C3,
    RegionMask = 0x114113CD,
    UV = 0xBB8307AB,
    UVDeltas = 0xDB830795,
    Binormals = 0x9BB38AFB,
    BoneWeights = 0x3BD70105,
    BoneAssignments = 0xFBD70111,
    BumpMapNormals = 0x89D92BA0,
    BumpMapNormalDeltas = 0x69D92B93,
    Unknown = 0x3B83078B,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BlockFormat {
    #[default]
    Float1 = 0,
    Float2 = 1,
    Float3 = 2,
    Byte = 4,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SetFormat {
    #[default]
    Main = 0,
    Norms = 1,
    UV = 2,
    Secondary = 3,
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
pub struct Element {
    pub reference_array_size: u32,
    pub element_identity: ElementIdentity,

    pub identity_num: u32,

    pub block_format: BlockFormat,
    pub set_format: SetFormat,

    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    data_size: u32,
    #[br(count = data_size)]
    pub data: Vec<u8>,

    #[br(temp)]
    #[bw(calc = references.len() as u32)]
    reference_count: u32,
    #[br(args { count: reference_count as usize, inner: args! { version } })]
    #[bw(args { version })]
    pub references: Vec<Reference>,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Linkage {
    #[br(temp)]
    #[bw(calc = indices.len() as u32)]
    index_count: u32,
    #[br(args{ count: index_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub indices: Vec<Reference>,

    pub referenced_array_size: u32, // TODO wtf does this do
    pub referenced_active: u32,

    #[br(temp)]
    #[bw(calc = vertices.len() as u32)]
    vertex_count: u32,
    #[br(args{ count: vertex_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub vertices: Vec<Reference>,

    #[br(temp)]
    #[bw(calc = normals.len() as u32)]
    normal_count: u32,
    #[br(args{ count: normal_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub normals: Vec<Reference>,

    #[br(temp)]
    #[bw(calc = uvs.len() as u32)]
    uv_count: u32,
    #[br(args{ count: uv_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub uvs: Vec<Reference>,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub primitive_type: PrimitiveType,
    pub link_index: u32,

    pub name: BigString,

    #[br(temp)]
    #[bw(calc = faces.len() as u32)]
    face_count: u32,
    #[br(args{ count: face_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub faces: Vec<Reference>,

    pub opacity: u32,

    #[br(temp, if(version > ResourceBlockVersion::V1))]
    #[bw(calc = subsets.len() as u32, if(version > ResourceBlockVersion::V1))]
    subset_count: u32,
    #[br(args{ count: subset_count as usize, inner: args! { version } }, if(version > ResourceBlockVersion::V1))]
    #[bw(args{ version }, if(version > ResourceBlockVersion::V1))]
    pub subsets: Vec<Reference>,
}

#[binrw]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub rotation: Quaternion,
    pub translation: Vertex,
}

#[binrw]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendGroupBinding {
    pub blend_group: BigString,
    pub element: BigString,
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Subset {
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

    #[br(temp)]
    #[bw(calc = elements.len() as u32)]
    element_count: u32,
    #[br(args{ count: element_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub elements: Vec<Element>,

    #[br(temp)]
    #[bw(calc = linkages.len() as u32)]
    linkage_count: u32,
    #[br(args{ count: linkage_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub linkages: Vec<Linkage>,

    #[br(temp)]
    #[bw(calc = groups.len() as u32)]
    group_count: u32,
    #[br(args{ count: group_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub groups: Vec<Group>,

    #[br(temp)]
    #[bw(calc = transforms.len() as u32)]
    transform_count: u32,
    #[br(count = transform_count)]
    pub transforms: Vec<Transform>,

    #[br(temp)]
    #[bw(calc = blend_group_bindings.len() as u32)]
    blend_group_binding_count: u32,
    #[br(count = blend_group_binding_count)]
    pub blend_group_bindings: Vec<BlendGroupBinding>,

    #[brw(args{ version })]
    pub main_subset: Subset,

    #[br(temp)]
    #[bw(calc = subsets.len() as u32)]
    subset_count: u32,
    #[br(args{ count: subset_count as usize, inner: args! { version } })]
    #[bw(args{ version })]
    pub subsets: Vec<Subset>,
}
