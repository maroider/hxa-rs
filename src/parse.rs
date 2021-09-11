use alloc::borrow::{Cow, ToOwned};
use alloc::str;
use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};
use core::mem;
use core::slice;

use crate::{
    error::{InternalError, InternalErrorKind},
    Hxa, HxaError, HxaResult, ImageType, Layer, LayerData, LayerDataType, LayerStack, Meta,
    MetaValue, MetadataType, Node, NodeContent, NodeGeometry, NodeImage, NodeType,
};

trait FromData: Sized {
    const SIZE: usize = 0;

    fn parse(data: &mut Cursor<'_>) -> HxaResult<Self> {
        Self::_parse(data.take_bytes(Self::SIZE)?)
    }

    fn _parse(data: &[u8]) -> HxaResult<Self>;
}

impl FromData for u8 {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.get(0).copied().ok_or(HxaError::UnexpectedEndOfData)
    }
}

impl FromData for i8 {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.get(0)
            .copied()
            .ok_or(HxaError::UnexpectedEndOfData)
            .map(|n| n as i8)
    }
}

impl FromData for u16 {
    const SIZE: usize = 2;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(u16::from_le_bytes)
    }
}

impl FromData for i16 {
    const SIZE: usize = 2;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(i16::from_le_bytes)
    }
}

impl FromData for u32 {
    const SIZE: usize = 4;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(u32::from_le_bytes)
    }
}

impl FromData for i32 {
    const SIZE: usize = 4;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(i32::from_le_bytes)
    }
}

impl FromData for u64 {
    const SIZE: usize = 8;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(u64::from_le_bytes)
    }
}

impl FromData for i64 {
    const SIZE: usize = 8;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(i64::from_le_bytes)
    }
}

impl FromData for f32 {
    const SIZE: usize = 4;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(f32::from_le_bytes)
    }
}
impl FromData for f64 {
    const SIZE: usize = 8;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        data.try_into()
            .map_err(|err| {
                HxaError::InternalError(InternalError::new(InternalErrorKind::TryFromSlice(err)))
            })
            .map(f64::from_le_bytes)
    }
}

impl FromData for NodeType {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        u8::_parse(data).and_then(|n| match n {
            0 => Ok(Self::Meta),
            1 => Ok(Self::Geometry),
            2 => Ok(Self::Image),
            3.. => Err(HxaError::UnexpectedNodeType(n)),
        })
    }
}

impl FromData for LayerDataType {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> HxaResult<Self> {
        u8::_parse(data).and_then(|n| match n {
            0 => Ok(Self::Uint8),
            1 => Ok(Self::Int32),
            2 => Ok(Self::Float),
            3 => Ok(Self::Double),
            4.. => Err(HxaError::UnexpectedLayerDataType(n)),
        })
    }
}

impl FromData for ImageType {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> Result<ImageType, HxaError> {
        u8::_parse(data).and_then(|n| match n {
            0 => Ok(Self::ImageCube),
            1 => Ok(Self::Image1D),
            2 => Ok(Self::Image2D),
            3 => Ok(Self::Image3D),
            4.. => Err(HxaError::UnexpectedImageType(n)),
        })
    }
}

impl FromData for MetadataType {
    const SIZE: usize = 1;

    fn _parse(data: &[u8]) -> Result<MetadataType, HxaError> {
        u8::_parse(data).and_then(|n| match n {
            0 => Ok(Self::Int64),
            1 => Ok(Self::Double),
            2 => Ok(Self::Node),
            3 => Ok(Self::Text),
            4 => Ok(Self::Binary),
            5 => Ok(Self::Meta),
            6.. => Err(HxaError::UnexpectedMetadataType(n)),
        })
    }
}

struct Cursor<'a> {
    // base_addr: usize,
    data: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            // base_addr: data.as_ptr() as usize,
            data,
        }
    }

    // fn offset(&self) -> usize {
    //     self.data.as_ptr() as usize - self.base_addr
    // }

    fn take_bytes(&mut self, size: usize) -> HxaResult<&'a [u8]> {
        // trace!("offset is {}, taking {} bytes", self.offset(), size);

        let bytes = self
            .data
            .get(0..size)
            .ok_or(HxaError::UnexpectedEndOfData)?;
        self.data = &self.data[size..];
        Ok(bytes)
    }
}

impl<'a> Hxa<'a> {
    pub(crate) fn parse(data: &'a [u8]) -> HxaResult<Self> {
        let mut cursor = Cursor::new(data);

        let magic_number = u32::parse(&mut cursor)?;
        let reference = u32::from_ne_bytes(*b"HxA\0");
        if magic_number != reference {
            return Err(HxaError::InvalidMagicNumber(magic_number));
        }

        let version = u32::parse(&mut cursor)? as u8;
        let node_count = u32::parse(&mut cursor)?.try_into().unwrap();
        let mut nodes = Vec::with_capacity(mem::size_of::<Node>() * node_count);
        for _ in 0..node_count {
            nodes.push(Node::new(&mut cursor, version)?);
        }

        debug_assert!(cursor.data.is_empty());

        Ok(Self { version, nodes })
    }
}

impl<'a> Node<'a> {
    fn new<'c>(cursor: &'c mut Cursor<'a>, version: u8) -> HxaResult<Self> {
        let type_ = NodeType::parse(cursor)?;
        let metadata_count = u32::parse(cursor)?.try_into().unwrap();
        let metadata = Meta::load(cursor, metadata_count)?;

        let content = match type_ {
            NodeType::Geometry => {
                let vertex_count = u32::parse(cursor)?.try_into().unwrap();
                let vertex_stack = LayerStack::new(cursor, vertex_count)?;
                let edge_corner_count = u32::parse(cursor)?.try_into().unwrap();
                let corner_stack = LayerStack::new(cursor, edge_corner_count)?;
                let edge_stack = if version > 2 {
                    LayerStack::new(cursor, edge_corner_count)?
                } else {
                    LayerStack::empty()
                };
                let face_count = u32::parse(cursor)?.try_into().unwrap();
                let face_stack = LayerStack::new(cursor, face_count)?;

                Some(NodeContent::Geometry(NodeGeometry {
                    vertex_stack,
                    corner_stack,
                    edge_stack,
                    face_stack,
                }))
            }
            NodeType::Image => {
                let type_ = ImageType::parse(cursor)?;
                let dimensions = match type_ {
                    ImageType::ImageCube => 2,
                    ImageType::Image1D => 1,
                    ImageType::Image2D => 2,
                    ImageType::Image3D => 3,
                };
                #[rustfmt::skip]
                let resolution = [
                    u32::parse(cursor)?,
                    if dimensions >= 2 { u32::parse(cursor)? } else { 1 },
                    if dimensions >= 3 { u32::parse(cursor)? } else { 1 },
                ];
                #[rustfmt::skip]
                let mut size =
                      usize::try_from(resolution[0]).unwrap()
                    * usize::try_from(resolution[1]).unwrap()
                    * usize::try_from(resolution[2]).unwrap();
                if type_ == ImageType::ImageCube {
                    size *= 6;
                }
                let image_stack = LayerStack::new(cursor, size)?;

                Some(NodeContent::Image(NodeImage {
                    type_,
                    resolution,
                    image_stack,
                }))
            }
            NodeType::Meta => None,
        };

        Ok(Self {
            type_,
            metadata,
            content,
        })
    }
}

impl<'a> LayerStack<'a> {
    fn new<'c>(cursor: &'c mut Cursor<'a>, length: usize) -> HxaResult<Self> {
        let stack_count = u32::parse(cursor)?.try_into().unwrap();
        let mut layers = Vec::with_capacity(mem::size_of::<Layer>() * stack_count);
        for _ in 0..stack_count {
            layers.push(Layer::new(cursor, length)?);
        }
        Ok(Self { layers })
    }

    fn empty() -> Self {
        Self { layers: Vec::new() }
    }
}

impl<'a> Layer<'a> {
    fn new<'c>(cursor: &'c mut Cursor<'a>, length: usize) -> HxaResult<Self> {
        let name = load_name(cursor)?;
        let component_count = u8::parse(cursor)?;
        let type_ = LayerDataType::parse(cursor)?;
        let len = component_count as usize * length;
        let data = match type_ {
            LayerDataType::Uint8 => {
                let data = cursor.take_bytes(len)?;
                LayerData::Uint8(Cow::Borrowed(data))
            }
            LayerDataType::Int32 => {
                let data = load_slice(cursor, len)?;
                LayerData::Int32(data)
            }
            LayerDataType::Float => {
                let data = load_slice(cursor, len)?;
                LayerData::Float(data)
            }
            LayerDataType::Double => {
                let data = load_slice(cursor, len)?;
                LayerData::Double(data)
            }
        };
        Ok(Self {
            name: Cow::Borrowed(name),
            component_count,
            type_,
            data,
        })
    }
}

impl<'a> Meta<'a> {
    fn load<'c>(cursor: &'c mut Cursor<'a>, count: usize) -> HxaResult<Vec<Self>> {
        let mut metadata = Vec::with_capacity(mem::size_of::<Meta>() * count);
        for _ in 0..count {
            metadata.push(Meta::new(cursor)?)
        }
        Ok(metadata)
    }

    fn new<'c>(cursor: &'c mut Cursor<'a>) -> HxaResult<Self> {
        let name = load_name(cursor)?;
        let type_ = MetadataType::parse(cursor)?;
        let array_length = u32::parse(cursor)?.try_into().unwrap();
        let value = match type_ {
            MetadataType::Int64 => {
                let data = load_slice(cursor, array_length)?;
                MetaValue::Int64(data)
            }
            MetadataType::Double => {
                let data = load_slice(cursor, array_length)?;
                MetaValue::Double(data)
            }
            MetadataType::Node => {
                let data = load_slice(cursor, array_length)?;
                MetaValue::Node(data)
            }
            MetadataType::Text => {
                let data = cursor.take_bytes(array_length)?;
                let string = str::from_utf8(data).map_err(HxaError::InvalidUtf8)?;
                MetaValue::Text(Cow::Borrowed(string))
            }
            MetadataType::Binary => {
                let data = cursor.take_bytes(array_length)?;
                MetaValue::Bin(data.into())
            }
            MetadataType::Meta => MetaValue::Meta(Meta::load(cursor, array_length)?),
        };

        Ok(Self {
            name: name.into(),
            type_,
            value,
        })
    }
}

fn load_name<'c, 'a>(cursor: &'c mut Cursor<'a>) -> HxaResult<&'a str> {
    let length = u8::parse(cursor)?;
    core::str::from_utf8(cursor.take_bytes(length as usize)?).map_err(HxaError::InvalidUtf8)
}

fn load_slice<'c, 'a, T>(cursor: &'c mut Cursor<'a>, length: usize) -> HxaResult<Cow<'a, [T]>>
where
    T: Pod + FromData,
    [T]: ToOwned<Owned = Vec<T>>,
{
    let len = mem::size_of::<T>() * length;

    #[cfg(target_endian = "little")]
    {
        let ptr = cursor.take_bytes(len)?.as_ptr();
        let slice = unsafe { slice::from_raw_parts(ptr.cast(), len / mem::size_of::<T>()) };
        Ok(Cow::Borrowed(slice))
    }
    #[cfg(target_endian = "big")]
    {
        let data = cursor.take_bytes(len)?;
        let data = data
            .iter()
            .chunks(mem::size_of::<T>())
            .map(|data| T::_parse(data).unwrap())
            .collect();
        Ok(Cow::Owned(data))
    }
}

trait Pod {}
impl Pod for u8 {}
impl Pod for i8 {}
impl Pod for u16 {}
impl Pod for i16 {}
impl Pod for u32 {}
impl Pod for i32 {}
impl Pod for u64 {}
impl Pod for i64 {}
impl Pod for u128 {}
impl Pod for i128 {}
impl Pod for f32 {}
impl Pod for f64 {}
