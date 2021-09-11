use core::array::TryFromSliceError;
use core::fmt;
use core::str::Utf8Error;

#[cfg(feature = "std")]
use std::error::Error;

pub type HxaResult<T> = Result<T, HxaError>;

#[derive(Debug)]
pub enum HxaError {
    InvalidMagicNumber(u32),
    UnexpectedEndOfData,
    UnexpectedNodeType(u8),
    UnexpectedLayerDataType(u8),
    UnexpectedImageType(u8),
    UnexpectedMetadataType(u8),
    InvalidUtf8(Utf8Error),
    InternalError(InternalError),
}

impl fmt::Display for HxaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HxaError::InvalidMagicNumber(n) => write!(f, "Invalid magic numer: {}", n),
            HxaError::UnexpectedEndOfData => {
                write!(
                    f,
                    "The parser unexpectedly reached the end of the data stream"
                )
            }
            HxaError::UnexpectedNodeType(n) => write!(f, "Unexpected node type {} encountered", n),
            HxaError::UnexpectedLayerDataType(n) => {
                write!(f, "Unexpected layer data type encountered: {}", n)
            }
            HxaError::UnexpectedImageType(n) => {
                write!(f, "Unexpected image type encountered: {}", n)
            }
            HxaError::UnexpectedMetadataType(n) => {
                write!(f, "Unexpected metadata type encountered: {}", n)
            }
            HxaError::InvalidUtf8(inner) => inner.fmt(f),
            HxaError::InternalError(inner) => write!(f, "Internal parser error: {}", inner),
        }
    }
}

#[cfg(feature = "std")]
impl Error for HxaError {}

#[derive(Debug)]
pub struct InternalError {
    kind: InternalErrorKind,
}

impl InternalError {
    pub(crate) fn new(kind: InternalErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            InternalErrorKind::TryFromSlice(inner) => inner.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl Error for InternalError {}

#[derive(Debug)]
pub(crate) enum InternalErrorKind {
    TryFromSlice(TryFromSliceError),
}
