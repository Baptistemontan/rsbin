// Enforce #![no_std] when the feature "std" is'nt active
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod de;
pub mod error;
pub mod ser;
mod tag;
mod utils;

pub use error::{DeError, NoRWError, SerError};
#[cfg(feature = "alloc")]
pub use ser::to_bytes;
pub use ser::{get_serialized_size, to_buff, to_writer, Serializer};

pub use utils::read;
pub use utils::write;

mod pub_ser {}
