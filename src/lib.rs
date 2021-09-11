#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::vec::Vec;

mod error;
mod parse;

pub use error::{HxaError, HxaResult};

#[derive(Clone, Debug)]
pub struct Hxa<'a> {
    pub version: u8,
    pub nodes: Vec<Node<'a>>,
}

impl<'a> Hxa<'a> {
    pub fn new(data: &'a [u8]) -> HxaResult<Self> {
        Self::parse(data)
    }
}

#[derive(Clone, Debug)]
pub struct Node<'a> {
    pub type_: NodeType,
    pub metadata: Vec<Meta<'a>>,
    pub content: Option<NodeContent<'a>>,
}

#[derive(Clone, Debug)]
pub enum NodeContent<'a> {
    Geometry(NodeGeometry<'a>),
    Image(NodeImage<'a>),
}

#[derive(Clone, Debug)]
pub struct NodeGeometry<'a> {
    pub vertex_stack: LayerStack<'a>,
    pub corner_stack: LayerStack<'a>,
    pub edge_stack: LayerStack<'a>,
    pub face_stack: LayerStack<'a>,
}

#[derive(Clone, Debug)]
pub struct NodeImage<'a> {
    pub type_: ImageType,
    pub resolution: [u32; 3],
    pub image_stack: LayerStack<'a>,
}

#[derive(Clone, Debug)]
pub struct LayerStack<'a> {
    pub layers: Vec<Layer<'a>>,
}

#[derive(Clone, Debug)]
pub struct Layer<'a> {
    pub name: Cow<'a, str>,
    pub component_count: u8,
    pub type_: LayerDataType,
    pub data: LayerData<'a>,
}

#[derive(Clone, Debug)]
pub enum LayerData<'a> {
    Uint8(Cow<'a, [u8]>),
    Int32(Cow<'a, [i32]>),
    Float(Cow<'a, [f32]>),
    Double(Cow<'a, [f64]>),
}

#[derive(Clone, Debug)]
pub struct Meta<'a> {
    pub name: Cow<'a, str>,
    pub type_: MetadataType,
    pub value: MetaValue<'a>,
}

#[derive(Clone, Debug)]
pub enum MetaValue<'a> {
    Int64(Cow<'a, [i64]>),
    Double(Cow<'a, [f64]>),
    Node(Cow<'a, [u32]>),
    Text(Cow<'a, str>),
    Bin(Cow<'a, [u8]>),
    Meta(Vec<Meta<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NodeType {
    Meta = 0,
    Geometry = 1,
    Image = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LayerDataType {
    Uint8 = 0,
    Int32 = 1,
    Float = 2,
    Double = 3,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ImageType {
    ImageCube = 0,
    Image1D = 1,
    Image2D = 2,
    Image3D = 3,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MetadataType {
    Int64 = 0,
    Double = 1,
    Node = 2,
    Text = 3,
    Binary = 4,
    Meta = 5,
}

// Hard conventions

pub const HC_BASE_VERTEX_LAYER_NAME: &str = "vertex";
pub const HC_BASE_VERTEX_LAYER_ID: usize = 0;
pub const HC_BASE_VERTEX_LAYER_COMPONENTS: u8 = 3;
pub const HC_BASE_CORNER_LAYER_NAME: &str = "reference";
pub const HC_BASE_CORNER_LAYER_ID: usize = 0;
pub const HC_BASE_CORNER_LAYER_COMPONENTS: u8 = 1;
pub const HC_BASE_CORNER_LAYER_TYPE: LayerDataType = LayerDataType::Int32;
pub const HC_EDGE_NEIGHBOUR_LAYER_NAME: &str = "neighbour";
pub const HC_EDGE_NEIGHBOUR_LAYER_TYPE: LayerDataType = LayerDataType::Int32;

// Soft conventions

pub const SC_LAYER_SEQUENCE0: &str = "sequence";
pub const SC_LAYER_NAME_UV0: &str = "uv";
pub const SC_LAYER_NORMALS: &str = "normal";
pub const SC_LAYER_BINORMAL: &str = "binormal";
pub const SC_LAYER_TANGENT: &str = "tangent";
pub const SC_LAYER_COLOR: &str = "color";
pub const SC_LAYER_CREASES: &str = "creases";
pub const SC_LAYER_SELECTION: &str = "select";
pub const SC_LAYER_SKIN_WEIGHT: &str = "skining_weight";
pub const SC_LAYER_SKIN_REFERENCE: &str = "skining_reference";
pub const SC_LAYER_BLENDSHAPE: &str = "blendshape";
pub const SC_LAYER_ADD_BLENDSHAPE: &str = "addblendshape";
pub const SC_LAYER_MATERIAL_ID: &str = "material";

// Image layers

pub const SC_ALBEDO: &str = "albedo";
pub const SC_LIGHT: &str = "light";
pub const SC_DISPLACEMENT: &str = "displacement";
pub const SC_DISTORTION: &str = "distortion";
pub const SC_AMBIENT_OCCLUSION: &str = "ambient_occlusion";

// Tag layers

pub const SC_NAME: &str = "name";
pub const SC_TRANSFORM: &str = "transform";
