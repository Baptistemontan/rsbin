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

#[cfg(all(test, feature = "test-utils"))]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::{
        token_struct,
        utils::token::{check_round_trip, CheckError, Number, Token},
    };

    pub type Result<T = (), E = CheckError> = core::result::Result<T, E>;

    #[derive(Deserialize, Serialize)]
    enum Test {
        Unit,
        NewType(u8),
        Struct { field: u8 },
        Tuple(u8, u8),
    }

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    struct MyStruct {
        foo: i32,
        bar: Option<u8>,
        foobar: bool,
    }

    #[test]
    fn test() -> Result {
        let my_struct = MyStruct {
            foo: 12,
            bar: Some(56),
            foobar: false,
        };

        check_round_trip(
            &my_struct,
            &token_struct! {
                "foo" => Token::Number(Number::I32(12)),
                "bar" => Token::Option(Some(Token::Number(Number::U8(56)).into())),
                "foobar" => Token::Bool(false)
            },
        )
    }
}
