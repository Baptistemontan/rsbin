use core::fmt::{self, Debug, Display};
use serde::{de, ser};

#[cfg(feature = "std")]
pub(crate) use std::error::Error as WriterError;
#[cfg(not(feature = "std"))]
pub trait WriterError: Display + Debug {}
#[cfg(not(feature = "std"))]
impl<T: Display + Debug> WriterError for T {}

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

use crate::tag::TagParsingError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoWriterError {}

impl Display for NoWriterError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NoWritterError is an enum with no variant, it can't be created.
        // So calling this function is impossible
        unreachable!()
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NoWriterError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerError<We> {
    WriterError(We),
    #[cfg(feature = "alloc")]
    Custom(String),
    #[cfg(not(feature = "alloc"))]
    Custom,
}

impl<We: WriterError> Display for SerError<We> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerError::WriterError(err) => Display::fmt(err, f),
            #[cfg(feature = "alloc")]
            SerError::Custom(err) => Display::fmt(err, f),
            #[cfg(not(feature = "alloc"))]
            SerError::Custom => f.write_str("An error occured."),
        }
    }
}

#[cfg(feature = "std")]
impl<We: WriterError> std::error::Error for SerError<We> {
    fn cause(&self) -> Option<&dyn WriterError> {
        match self {
            SerError::WriterError(err) => Some(err),
            _ => None,
        }
    }
}

impl<We: WriterError> ser::Error for SerError<We> {
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

impl<We: WriterError> From<We> for SerError<We> {
    fn from(value: We) -> Self {
        SerError::WriterError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeError {
    TagParsingError(TagParsingError),
    #[cfg(feature = "alloc")]
    Custom(String),
    #[cfg(not(feature = "alloc"))]
    Custom,
}

impl Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeError::TagParsingError(err) => Display::fmt(err, f),
            #[cfg(feature = "alloc")]
            DeError::Custom(err) => Display::fmt(err, f),
            #[cfg(not(feature = "alloc"))]
            DeError::Custom => f.write_str("An error occured."),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DeError {}

impl de::Error for DeError {
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

impl From<TagParsingError> for DeError {
    fn from(value: TagParsingError) -> Self {
        DeError::TagParsingError(value)
    }
}
