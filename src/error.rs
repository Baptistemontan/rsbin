use core::fmt::{self, Debug, Display};
use core::str::Utf8Error;
use serde::{de, ser};

#[cfg(feature = "std")]
pub(crate) use std::error::Error as RWError;
#[cfg(not(feature = "std"))]
pub trait RWError: Display + Debug {}
#[cfg(not(feature = "std"))]
impl<T: Display + Debug> RWError for T {}

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

use crate::tag::{Tag, TagParsingError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoRWError {}

impl Display for NoRWError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NoWritterError is an enum with no variant, it can't be created.
        // So calling this function is impossible
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NoRWError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerError<We> {
    WriteError(We),
    #[cfg(feature = "alloc")]
    Custom(String),
    #[cfg(not(feature = "alloc"))]
    Custom,
}

impl<We: RWError> Display for SerError<We> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerError::WriteError(err) => Display::fmt(err, f),
            #[cfg(feature = "alloc")]
            SerError::Custom(err) => Display::fmt(err, f),
            #[cfg(not(feature = "alloc"))]
            SerError::Custom => f.write_str("An error occured."),
        }
    }
}

#[cfg(feature = "std")]
impl<We: RWError> std::error::Error for SerError<We> {
    fn cause(&self) -> Option<&dyn RWError> {
        match self {
            SerError::WriteError(err) => Some(err),
            _ => None,
        }
    }
}

impl<We: RWError> ser::Error for SerError<We> {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        SerError::Custom(msg.to_string())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        SerError::Custom
    }
}

impl<We: RWError> From<We> for SerError<We> {
    fn from(value: We) -> Self {
        SerError::WriteError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeError<E> {
    ReaderError(E),
    TagParsingError(TagParsingError),
    Utf8Error(Utf8Error),
    InvalidLen(u64),
    UnexpectedTag(UnexpectedTag),
    #[cfg(feature = "alloc")]
    Custom(String),
    #[cfg(not(feature = "alloc"))]
    Custom,
}

impl<E: RWError> Display for DeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeError::ReaderError(err) => Display::fmt(err, f),
            DeError::TagParsingError(err) => Display::fmt(err, f),
            DeError::Utf8Error(err) => Display::fmt(err, f),
            #[cfg(feature = "alloc")]
            DeError::Custom(err) => Display::fmt(err, f),
            #[cfg(not(feature = "alloc"))]
            DeError::Custom => f.write_str("An error occured."),
            DeError::UnexpectedTag(err) => Display::fmt(err, f),
            DeError::InvalidLen(len) => {
                f.write_fmt(format_args!("Sequence len is too big: {}", len))
            }
        }
    }
}

#[cfg(feature = "std")]
impl<E: RWError> std::error::Error for DeError<E> {}

impl<E: RWError> de::Error for DeError<E> {
    #[cfg(feature = "alloc")]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        DeError::Custom(msg.to_string())
    }

    #[cfg(not(feature = "alloc"))]
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        DeError::Custom
    }
}

impl<E: RWError> From<TagParsingError> for DeError<E> {
    fn from(value: TagParsingError) -> Self {
        DeError::TagParsingError(value)
    }
}

impl<E: RWError> From<UnexpectedTag> for DeError<E> {
    fn from(value: UnexpectedTag) -> Self {
        DeError::UnexpectedTag(value)
    }
}

impl<E: RWError> From<E> for DeError<E> {
    fn from(value: E) -> Self {
        DeError::ReaderError(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnexpectedTag {
    pub expected: &'static [Tag],
    pub got: Tag,
}

impl Display for UnexpectedTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Unexpected tag, got {:?} but expected {:?}",
            self.got, self.expected
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndOfBuff;

impl Display for EndOfBuff {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Reached end of buffer before end of serialization.")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EndOfBuff {}
