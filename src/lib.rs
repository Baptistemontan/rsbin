// Enforce #![no_std] when the feature "std" is'nt active
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod de;
mod error;
mod ser;
mod utils;
