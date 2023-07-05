use core::fmt::Debug;
use std::collections::HashMap;

use serde::{
    de::{VariantAccess, Visitor},
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Serialize,
};

use crate::{error::EndOfBuff, DeError, SerError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum EnumVariantKind {
    Unit,
    NewType,
    Struct,
    Tuple,
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Option(Option<Box<Self>>),
    Bool(bool),
    Number(Number),
    Char(char),
    BorrowedString(&'a str),
    String(String),
    BorrowedBytes(&'a [u8]),
    Bytes(Vec<u8>),
    Unit,
    #[allow(unused)]
    UnitStruct,
    UnitVariant,
    NewTypeStruct(Box<Self>),
    NewTypeVariant(Box<Self>),
    Seq(Vec<Self>),
    UnsizedSeq(Vec<Self>),
    #[allow(unused)]
    Tuple(Vec<Self>),
    #[allow(unused)]
    TupleStruct(Vec<Self>),
    TupleVariant(Vec<Self>),
    Map(Vec<(Self, Self)>),
    UnsizedMap(Vec<(Self, Self)>),
    Struct(HashMap<&'static str, Self>),
    StructVariant(HashMap<&'static str, Self>),
    #[allow(unused)]
    StructVariantDeserialized(Vec<(Self, Self)>),
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
}

//////////////////////////////////////////////////////////////////////////
///                    Util functions                                  ///
//////////////////////////////////////////////////////////////////////////

impl From<Number> for Token<'_> {
    fn from(value: Number) -> Self {
        Token::Number(value)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            // signed
            (Self::I8(l0), Self::I8(r0)) => l0 == r0,
            (Self::I8(l0), Self::I16(r0)) => i16::from(l0) == r0,
            (Self::I8(l0), Self::I32(r0)) => i32::from(l0) == r0,
            (Self::I8(l0), Self::I64(r0)) => i64::from(l0) == r0,
            (Self::I8(l0), Self::I128(r0)) => i128::from(l0) == r0,
            (Self::I16(l0), Self::I8(r0)) => l0 == i16::from(r0),
            (Self::I16(l0), Self::I16(r0)) => l0 == r0,
            (Self::I16(l0), Self::I32(r0)) => i32::from(l0) == r0,
            (Self::I16(l0), Self::I64(r0)) => i64::from(l0) == r0,
            (Self::I16(l0), Self::I128(r0)) => i128::from(l0) == r0,
            (Self::I32(l0), Self::I8(r0)) => l0 == i32::from(r0),
            (Self::I32(l0), Self::I16(r0)) => l0 == i32::from(r0),
            (Self::I32(l0), Self::I32(r0)) => l0 == r0,
            (Self::I32(l0), Self::I64(r0)) => i64::from(l0) == r0,
            (Self::I32(l0), Self::I128(r0)) => i128::from(l0) == r0,
            (Self::I64(l0), Self::I8(r0)) => l0 == i64::from(r0),
            (Self::I64(l0), Self::I16(r0)) => l0 == i64::from(r0),
            (Self::I64(l0), Self::I32(r0)) => l0 == i64::from(r0),
            (Self::I64(l0), Self::I64(r0)) => l0 == r0,
            (Self::I64(l0), Self::I128(r0)) => i128::from(l0) == r0,
            (Self::I128(l0), Self::I8(r0)) => l0 == i128::from(r0),
            (Self::I128(l0), Self::I16(r0)) => l0 == i128::from(r0),
            (Self::I128(l0), Self::I32(r0)) => l0 == i128::from(r0),
            (Self::I128(l0), Self::I64(r0)) => l0 == i128::from(r0),
            (Self::I128(l0), Self::I128(r0)) => l0 == r0,
            // unsigned
            (Self::U8(l0), Self::U8(r0)) => l0 == r0,
            (Self::U8(l0), Self::U16(r0)) => u16::from(l0) == r0,
            (Self::U8(l0), Self::U32(r0)) => u32::from(l0) == r0,
            (Self::U8(l0), Self::U64(r0)) => u64::from(l0) == r0,
            (Self::U8(l0), Self::U128(r0)) => u128::from(l0) == r0,
            (Self::U16(l0), Self::U8(r0)) => l0 == u16::from(r0),
            (Self::U16(l0), Self::U16(r0)) => l0 == r0,
            (Self::U16(l0), Self::U32(r0)) => u32::from(l0) == r0,
            (Self::U16(l0), Self::U64(r0)) => u64::from(l0) == r0,
            (Self::U16(l0), Self::U128(r0)) => u128::from(l0) == r0,
            (Self::U32(l0), Self::U8(r0)) => l0 == u32::from(r0),
            (Self::U32(l0), Self::U16(r0)) => l0 == u32::from(r0),
            (Self::U32(l0), Self::U32(r0)) => l0 == r0,
            (Self::U32(l0), Self::U64(r0)) => u64::from(l0) == r0,
            (Self::U32(l0), Self::U128(r0)) => u128::from(l0) == r0,
            (Self::U64(l0), Self::U8(r0)) => l0 == u64::from(r0),
            (Self::U64(l0), Self::U16(r0)) => l0 == u64::from(r0),
            (Self::U64(l0), Self::U32(r0)) => l0 == u64::from(r0),
            (Self::U64(l0), Self::U64(r0)) => l0 == r0,
            (Self::U64(l0), Self::U128(r0)) => u128::from(l0) == r0,
            (Self::U128(l0), Self::U8(r0)) => l0 == u128::from(r0),
            (Self::U128(l0), Self::U16(r0)) => l0 == u128::from(r0),
            (Self::U128(l0), Self::U32(r0)) => l0 == u128::from(r0),
            (Self::U128(l0), Self::U64(r0)) => l0 == u128::from(r0),
            (Self::U128(l0), Self::U128(r0)) => l0 == r0,
            // floats
            (Self::F32(l0), Self::F32(r0)) => l0 == r0,
            (Self::F32(l0), Self::F64(r0)) => f64::from(l0) == r0,
            (Self::F64(l0), Self::F32(r0)) => l0 == f64::from(r0),
            (Self::F64(l0), Self::F64(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<'a, 'b> PartialEq<Token<'a>> for Token<'b> {
    fn eq(&self, other: &Token<'a>) -> bool {
        fn compare_struct_map(s: &HashMap<&str, Token>, map: &[(Token, Token)]) -> bool {
            if s.len() != map.len() {
                return false;
            }
            let check = |key: &str, value: &Token| s.get(key).is_some_and(|v| v == value);
            for (key, value) in map {
                match key {
                    Token::BorrowedString(key) => {
                        let is_same = check(key, value);
                        if !is_same {
                            return false;
                        }
                    }
                    Token::String(key) => {
                        let is_same = check(key, value);
                        if !is_same {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
            true
        }

        match (self, other) {
            // 1 to 1 variants
            (Token::Option(l0), Token::Option(r0)) => l0 == r0,
            (Token::Bool(l0), Token::Bool(r0)) => l0 == r0,
            (Token::Number(l0), Token::Number(r0)) => l0 == r0,
            (Token::Char(l0), Token::Char(r0)) => l0 == r0,
            (Token::BorrowedString(l0), Token::BorrowedString(r0)) => l0 == r0,
            (Token::String(l0), Token::String(r0)) => l0 == r0,
            (Token::BorrowedBytes(l0), Token::BorrowedBytes(r0)) => l0 == r0,
            (Token::Bytes(l0), Token::Bytes(r0)) => l0 == r0,
            (Token::Unit, Token::Unit) => true,
            (Token::UnitStruct, Token::UnitStruct) => true,
            (Token::UnitVariant, Token::UnitVariant) => true,
            (Token::NewTypeStruct(l0), Token::NewTypeStruct(r0)) => l0 == r0,
            (Token::NewTypeVariant(l0), Token::NewTypeVariant(r0)) => l0 == r0,
            (Token::Seq(l0), Token::Seq(r0)) => l0 == r0,
            (Token::UnsizedSeq(l0), Token::UnsizedSeq(r0)) => l0 == r0,
            (Token::Tuple(l0), Token::Tuple(r0)) => l0 == r0,
            (Token::TupleStruct(l0), Token::TupleStruct(r0)) => l0 == r0,
            (Token::TupleVariant(l0), Token::TupleVariant(r0)) => l0 == r0,
            (Token::Map(l0), Token::Map(r0)) => l0 == r0,
            (Token::UnsizedMap(l0), Token::UnsizedMap(r0)) => l0 == r0,
            (Token::Struct(l0), Token::Struct(r0)) => l0 == r0,
            (Token::StructVariant(l0), Token::StructVariant(r0)) => l0 == r0,
            // tokens that can change types in round trip
            (Token::BorrowedString(l0), Token::String(r0)) => l0 == r0,
            (Token::String(l0), Token::BorrowedString(r0)) => l0 == r0,
            (Token::BorrowedBytes(l0), Token::Bytes(r0)) => l0 == r0,
            (Token::Bytes(l0), Token::BorrowedBytes(r0)) => l0 == r0,
            (Token::Unit, Token::UnitStruct) => true,
            (Token::UnitStruct, Token::Unit) => true,
            (Token::Tuple(l0), Token::Seq(r0)) => l0 == r0,
            (Token::Seq(l0), Token::Tuple(r0)) => l0 == r0,
            (Token::TupleStruct(l0), Token::Seq(r0)) => l0 == r0,
            (Token::Seq(l0), Token::TupleStruct(r0)) => l0 == r0,
            (Token::Struct(l0), Token::Map(r0)) => compare_struct_map(l0, r0),
            (Token::Map(l0), Token::Struct(r0)) => compare_struct_map(r0, l0),
            (Token::StructVariant(l0), Token::StructVariantDeserialized(r0)) => {
                compare_struct_map(l0, r0)
            }
            (Token::StructVariantDeserialized(l0), Token::StructVariant(r0)) => {
                compare_struct_map(r0, l0)
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum CheckError {
    SerializationError(SerError<std::io::Error>),
    DeserializationError(DeError<EndOfBuff>),
}

impl From<SerError<std::io::Error>> for CheckError {
    fn from(value: SerError<std::io::Error>) -> Self {
        CheckError::SerializationError(value)
    }
}
impl From<DeError<EndOfBuff>> for CheckError {
    fn from(value: DeError<EndOfBuff>) -> Self {
        CheckError::DeserializationError(value)
    }
}

pub fn check_ser<T: ?Sized>(value: &T, token_tree: &Token) -> Result<(), CheckError>
where
    T: Serialize,
{
    let bytes = crate::ser::to_bytes(value)?;
    let tt: Token = crate::de::from_bytes(&bytes)?;
    assert_eq!(token_tree, &tt);
    Ok(())
}

pub fn check_de<T: ?Sized>(value: &T, token_tree: &Token) -> Result<(), CheckError>
where
    T: for<'de> Deserialize<'de> + PartialEq + Debug,
{
    let bytes = crate::ser::to_bytes(token_tree)?;
    let deserialized_value: T = crate::de::from_bytes(&bytes)?;
    assert_eq!(value, &deserialized_value);
    Ok(())
}

pub fn check_round_trip<T: ?Sized>(value: &T, token_tree: &Token) -> Result<(), CheckError>
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
{
    check_ser(value, token_tree)?;
    check_de(value, token_tree)?;
    let bytes = crate::ser::to_bytes(value)?;
    let deserialized_value = crate::de::from_bytes(&bytes)?;
    assert_eq!(value, &deserialized_value);
    Ok(())
}

#[macro_export]
macro_rules! token_struct {
    ($($field:expr => $value: expr),*) => {
        Token::Struct({
            let mut map = std::collections::HashMap::new();
            {
                $(
                    if map.insert($field, $value).is_some() {
                        panic!("field {} already present.", $field);
                    }
                )*
            }
            map
        })

    };
}

//////////////////////////////////////////////////////////////////////////
///                    Deserialization                                 ///
//////////////////////////////////////////////////////////////////////////

struct TokenVisitor;

macro_rules! impl_visit_num {
    ($fn_name:ident, $t:ty, $num_variant:ident) => {
        fn $fn_name<E>(self, v: $t) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Token::Number(Number::$num_variant(v)))
        }
    };
}

struct EnumVariantKindVisitor;

impl<'de> Deserialize<'de> for EnumVariantKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(EnumVariantKindVisitor)
    }
}

impl<'de> Visitor<'de> for EnumVariantKindVisitor {
    type Value = EnumVariantKind;

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            0 => Ok(EnumVariantKind::Unit),
            1 => Ok(EnumVariantKind::NewType),
            2 => Ok(EnumVariantKind::Struct),
            3 => Ok(EnumVariantKind::Tuple),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(v),
                &"variant index 0 <= i < 4",
            )),
        }
    }

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        write!(formatter, "an enum kind identifier (0..=3)")
    }
}

impl<'de> Visitor<'de> for TokenVisitor {
    type Value = Token<'de>;

    impl_visit_num!(visit_i8, i8, I8);
    impl_visit_num!(visit_i16, i16, I16);
    impl_visit_num!(visit_i32, i32, I32);
    impl_visit_num!(visit_i64, i64, I64);
    impl_visit_num!(visit_i128, i128, I128);
    impl_visit_num!(visit_u8, u8, U8);
    impl_visit_num!(visit_u16, u16, U16);
    impl_visit_num!(visit_u32, u32, U32);
    impl_visit_num!(visit_u64, u64, U64);
    impl_visit_num!(visit_u128, u128, U128);
    impl_visit_num!(visit_f32, f32, F32);
    impl_visit_num!(visit_f64, f64, F64);

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Bool(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Char(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::String(v.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::BorrowedString(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Bytes(v.to_vec()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::BorrowedBytes(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Bytes(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Option(None))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token = deserializer.deserialize_any(self)?;
        Ok(Token::Option(Some(token.into())))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Token::Unit)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token = deserializer.deserialize_any(self)?;
        Ok(Token::NewTypeStruct(token.into()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let len = seq.size_hint();
        let mut sequence = Vec::with_capacity(len.unwrap_or_default());
        while let Some(token) = seq.next_element()? {
            sequence.push(token)
        }
        if len.is_some() {
            Ok(Token::Seq(sequence))
        } else {
            Ok(Token::UnsizedSeq(sequence))
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let len = map.size_hint();
        let mut token_map = Vec::with_capacity(len.unwrap_or_default());
        while let Some(entry) = map.next_entry()? {
            token_map.push(entry);
        }
        if len.is_some() {
            Ok(Token::Map(token_map))
        } else {
            Ok(Token::UnsizedMap(token_map))
        }
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let (variant_kind, enum_access) = data.variant()?;
        match variant_kind {
            EnumVariantKind::Unit => {
                enum_access.unit_variant()?;
                Ok(Token::UnitVariant)
            }
            EnumVariantKind::NewType => {
                let token = enum_access.newtype_variant()?;
                Ok(Token::NewTypeVariant(token))
            }
            EnumVariantKind::Struct => {
                let Token::Struct(map) = enum_access.struct_variant(&[], self)? else {
                    todo!()
                };
                Ok(Token::StructVariant(map))
            }
            EnumVariantKind::Tuple => {
                let Token::Tuple(tuple) = enum_access.tuple_variant(0, self)? else {
                    todo!()
                };
                Ok(Token::TupleVariant(tuple))
            }
        }
    }

    fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        let _ = formatter;
        todo!()
    }
}

impl<'de> Deserialize<'de> for Token<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TokenVisitor)
    }
}

//////////////////////////////////////////////////////////////////////////
///                    Serialization                                   ///
//////////////////////////////////////////////////////////////////////////

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Number::I8(v) => serializer.serialize_i8(v),
            Number::I16(v) => serializer.serialize_i16(v),
            Number::I32(v) => serializer.serialize_i32(v),
            Number::I64(v) => serializer.serialize_i64(v),
            Number::I128(v) => serializer.serialize_i128(v),
            Number::U8(v) => serializer.serialize_u8(v),
            Number::U16(v) => serializer.serialize_u16(v),
            Number::U32(v) => serializer.serialize_u32(v),
            Number::U64(v) => serializer.serialize_u64(v),
            Number::U128(v) => serializer.serialize_u128(v),
            Number::F32(v) => serializer.serialize_f32(v),
            Number::F64(v) => serializer.serialize_f64(v),
        }
    }
}

impl Serialize for Token<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Token::Option(None) => serializer.serialize_none(),
            Token::Option(Some(value)) => serializer.serialize_some(value),
            Token::Bool(value) => serializer.serialize_bool(*value),
            Token::Number(num) => num.serialize(serializer),
            Token::Char(value) => serializer.serialize_char(*value),
            Token::BorrowedString(value) => serializer.serialize_str(value),
            Token::String(value) => serializer.serialize_str(value),
            Token::BorrowedBytes(value) => serializer.serialize_bytes(value),
            Token::Bytes(value) => serializer.serialize_bytes(value),
            Token::Unit => serializer.serialize_unit(),
            Token::UnitStruct => serializer.serialize_unit_struct(""),
            Token::UnitVariant => {
                serializer.serialize_unit_variant("", EnumVariantKind::Unit as u32, "")
            }
            Token::NewTypeStruct(value) => serializer.serialize_newtype_struct("", value),
            Token::NewTypeVariant(value) => {
                serializer.serialize_newtype_variant("", EnumVariantKind::NewType as u32, "", value)
            }
            Token::Seq(values) => {
                let mut seq_ser = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq_ser.serialize_element(value)?;
                }
                seq_ser.end()
            }
            Token::UnsizedSeq(values) => {
                let mut seq_ser = serializer.serialize_seq(None)?;
                for value in values {
                    seq_ser.serialize_element(value)?;
                }
                seq_ser.end()
            }
            Token::Tuple(values) => {
                let mut seq_ser = serializer.serialize_tuple(values.len())?;
                for value in values {
                    seq_ser.serialize_element(value)?;
                }
                seq_ser.end()
            }
            Token::TupleStruct(values) => {
                let mut seq_ser = serializer.serialize_tuple_struct("", values.len())?;
                for value in values {
                    seq_ser.serialize_field(value)?;
                }
                seq_ser.end()
            }
            Token::TupleVariant(values) => {
                let mut seq_ser = serializer.serialize_tuple_variant(
                    "",
                    EnumVariantKind::Tuple as u32,
                    "",
                    values.len(),
                )?;
                for value in values {
                    seq_ser.serialize_field(value)?;
                }
                seq_ser.end()
            }
            Token::Map(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                for (key, value) in map {
                    map_ser.serialize_entry(key, value)?;
                }
                map_ser.end()
            }
            Token::UnsizedMap(map) => {
                let mut map_ser = serializer.serialize_map(None)?;
                for (key, value) in map {
                    map_ser.serialize_entry(key, value)?;
                }
                map_ser.end()
            }
            Token::Struct(map) => {
                let mut struct_ser = serializer.serialize_struct("", map.len())?;
                for (key, value) in map {
                    struct_ser.serialize_field(key, value)?;
                }
                struct_ser.end()
            }
            Token::StructVariant(map) => {
                let mut struct_ser = serializer.serialize_struct_variant(
                    "",
                    EnumVariantKind::Struct as u32,
                    "",
                    map.len(),
                )?;
                for (key, value) in map {
                    struct_ser.serialize_field(key, value)?;
                }
                struct_ser.end()
            }
            Token::StructVariantDeserialized(_) => {
                unimplemented!()
            }
        }
    }
}
