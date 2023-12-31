#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
#[cfg(feature = "alloc")]
extern crate alloc;
pub mod de {
    use crate::{
        error::{NoRWError, UnexpectedTag},
        read::Read, tag::Tag,
    };
    use serde::de::Visitor;
    use serde::{de, serde_if_integer128};
    pub type Error<Re = NoRWError> = crate::error::DeError<Re>;
    pub type Result<T, Re = NoRWError> = core::result::Result<T, Error<Re>>;
    pub struct Deserializer<R> {
        reader: R,
        peeked_tag: Option<Tag>,
    }
    impl<'de, R: Read<'de>> Deserializer<R> {
        fn pop_tag(&mut self) -> Result<Tag, R::Error> {
            if let Some(tag) = self.peeked_tag.take() {
                Ok(tag)
            } else {
                let byte = self.reader.read_byte()?;
                let tag = byte.try_into()?;
                Ok(tag)
            }
        }
        fn peek_tag(&mut self) -> Result<Tag, R::Error> {
            if let Some(tag) = self.peeked_tag {
                Ok(tag)
            } else {
                let tag = self.pop_tag()?;
                self.peeked_tag = Some(tag);
                Ok(tag)
            }
        }
        fn pop_n<const N: usize>(&mut self) -> Result<[u8; N], R::Error> {
            let mut buff = [0; N];
            self.reader.read_bytes(&mut buff)?;
            Ok(buff)
        }
        fn pop_len(&mut self) -> Result<usize, R::Error> {
            let len = self.parse_u64()?;
            len.try_into().map_err(|_| Error::InvalidLen(len))
        }
        fn pop_variant(&mut self) -> Result<u32, R::Error> {
            self.parse_u32()
        }
        fn parse_u64(&mut self) -> Result<u64, R::Error> {
            match self.peek_tag()? {
                Tag::U64 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(u64::from_be_bytes(bytes))
                }
                Tag::U32 => self.parse_u32().map(u64::from),
                Tag::U16 => self.parse_u16().map(u64::from),
                Tag::U8 => self.parse_u8().map(u64::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::U64,
                        Tag::U32,
                        Tag::U16,
                        Tag::U8,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_u32(&mut self) -> Result<u32, R::Error> {
            match self.peek_tag()? {
                Tag::U32 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(u32::from_be_bytes(bytes))
                }
                Tag::U16 => self.parse_u16().map(u32::from),
                Tag::U8 => self.parse_u8().map(u32::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::U32, Tag::U16, Tag::U8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_u16(&mut self) -> Result<u16, R::Error> {
            match self.peek_tag()? {
                Tag::U16 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(u16::from_be_bytes(bytes))
                }
                Tag::U8 => self.parse_u8().map(u16::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::U16, Tag::U8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_u8(&mut self) -> Result<u8, R::Error> {
            match self.pop_tag()? {
                Tag::U8 => {
                    let bytes = self.pop_n()?;
                    Ok(u8::from_be_bytes(bytes))
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::U8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_i64(&mut self) -> Result<i64, R::Error> {
            match self.peek_tag()? {
                Tag::I64 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(i64::from_be_bytes(bytes))
                }
                Tag::I32 => self.parse_i32().map(i64::from),
                Tag::I16 => self.parse_i16().map(i64::from),
                Tag::I8 => self.parse_i8().map(i64::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::I64,
                        Tag::I32,
                        Tag::I16,
                        Tag::I8,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_i32(&mut self) -> Result<i32, R::Error> {
            match self.peek_tag()? {
                Tag::I32 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(i32::from_be_bytes(bytes))
                }
                Tag::I16 => self.parse_i16().map(i32::from),
                Tag::I8 => self.parse_i8().map(i32::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::I32, Tag::I16, Tag::I8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_i16(&mut self) -> Result<i16, R::Error> {
            match self.peek_tag()? {
                Tag::I16 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(i16::from_be_bytes(bytes))
                }
                Tag::I8 => self.parse_i8().map(i16::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::I16, Tag::I8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_i8(&mut self) -> Result<i8, R::Error> {
            match self.pop_tag()? {
                Tag::I8 => {
                    let bytes = self.pop_n()?;
                    Ok(i8::from_be_bytes(bytes))
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::I8];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_f64(&mut self) -> Result<f64, R::Error> {
            match self.peek_tag()? {
                Tag::F64 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(f64::from_be_bytes(bytes))
                }
                Tag::F32 => self.parse_f32().map(f64::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::F64, Tag::F32];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_f32(&mut self) -> Result<f32, R::Error> {
            match self.pop_tag()? {
                Tag::F32 => {
                    let bytes = self.pop_n()?;
                    Ok(f32::from_be_bytes(bytes))
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::F32];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_u128(&mut self) -> Result<u128, R::Error> {
            match self.peek_tag()? {
                Tag::U128 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(u128::from_be_bytes(bytes))
                }
                Tag::U64 => self.parse_u64().map(u128::from),
                Tag::U32 => self.parse_u32().map(u128::from),
                Tag::U16 => self.parse_u16().map(u128::from),
                Tag::U8 => self.parse_u8().map(u128::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::U128,
                        Tag::U64,
                        Tag::U32,
                        Tag::U16,
                        Tag::U8,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn parse_i128(&mut self) -> Result<i128, R::Error> {
            match self.peek_tag()? {
                Tag::I128 => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok(i128::from_be_bytes(bytes))
                }
                Tag::I64 => self.parse_i64().map(i128::from),
                Tag::I32 => self.parse_i32().map(i128::from),
                Tag::I16 => self.parse_i16().map(i128::from),
                Tag::I8 => self.parse_i8().map(i128::from),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::I128,
                        Tag::I64,
                        Tag::I32,
                        Tag::I16,
                        Tag::I8,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
    }
    impl<'a, 'de, R: Read<'de>> de::Deserializer<'de> for &'a mut Deserializer<R> {
        type Error = Error<R::Error>;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.peek_tag()? {
                Tag::None | Tag::Some => self.deserialize_option(visitor),
                Tag::BoolFalse | Tag::BoolTrue => self.deserialize_bool(visitor),
                Tag::I8 => self.deserialize_i8(visitor),
                Tag::I16 => self.deserialize_i16(visitor),
                Tag::I32 => self.deserialize_i32(visitor),
                Tag::I64 => self.deserialize_i64(visitor),
                Tag::U8 => self.deserialize_u8(visitor),
                Tag::U16 => self.deserialize_u16(visitor),
                Tag::U32 => self.deserialize_u32(visitor),
                Tag::U64 => self.deserialize_u64(visitor),
                Tag::F32 => self.deserialize_f32(visitor),
                Tag::F64 => self.deserialize_f64(visitor),
                Tag::Char1 | Tag::Char2 | Tag::Char3 | Tag::Char4 => {
                    self.deserialize_char(visitor)
                }
                Tag::String | Tag::MarkerTerminatedString => {
                    self.deserialize_str(visitor)
                }
                Tag::Bytes => self.deserialize_bytes(visitor),
                Tag::Unit => self.deserialize_unit(visitor),
                Tag::UnitStruct => self.deserialize_unit_struct("", visitor),
                Tag::NewTypeStruct => self.deserialize_newtype_struct("", visitor),
                Tag::Seq | Tag::UnsizedSeq | Tag::Tuple | Tag::TupleStruct => {
                    self.deserialize_seq(visitor)
                }
                Tag::UnitVariant
                | Tag::NewTypeVariant
                | Tag::TupleVariant
                | Tag::StructVariant => self.deserialize_enum("", &[], visitor),
                Tag::Map | Tag::UnsizedMap | Tag::Struct => self.deserialize_map(visitor),
                #[cfg(not(no_integer128))]
                Tag::I128 => self.deserialize_i128(visitor),
                #[cfg(not(no_integer128))]
                Tag::U128 => self.deserialize_u128(visitor),
                got @ Tag::UnsizedSeqEnd => {
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            expected: &[],
                            got,
                        }),
                    )
                }
            }
        }
        fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::BoolFalse => visitor.visit_bool(false),
                Tag::BoolTrue => visitor.visit_bool(true),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::BoolFalse, Tag::BoolTrue];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_i8()?;
            visitor.visit_i8(num)
        }
        fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_i16()?;
            visitor.visit_i16(num)
        }
        fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_i32()?;
            visitor.visit_i32(num)
        }
        fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_i64()?;
            visitor.visit_i64(num)
        }
        fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_u8()?;
            visitor.visit_u8(num)
        }
        fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_u16()?;
            visitor.visit_u16(num)
        }
        fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_u32()?;
            visitor.visit_u32(num)
        }
        fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_u64()?;
            visitor.visit_u64(num)
        }
        fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_f32()?;
            visitor.visit_f32(num)
        }
        fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_f64()?;
            visitor.visit_f64(num)
        }
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_i128()?;
            visitor.visit_i128(num)
        }
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.parse_u128()?;
            visitor.visit_u128(num)
        }
        fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            fn inner<'de, const N: usize, R: Read<'de>>(
                de: &mut Deserializer<R>,
            ) -> Result<char, R::Error> {
                let bytes = de.pop_n::<N>()?;
                let c = core::str::from_utf8(&bytes)
                    .map_err(Error::Utf8Error)?
                    .chars()
                    .next();
                Ok(c.unwrap_or_default())
            }
            let c = match self.pop_tag()? {
                Tag::Char1 => inner::<1, R>(self),
                Tag::Char2 => inner::<2, R>(self),
                Tag::Char3 => inner::<3, R>(self),
                Tag::Char4 => inner::<4, R>(self),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::Char1,
                        Tag::Char2,
                        Tag::Char3,
                        Tag::Char4,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }?;
            visitor.visit_char(c)
        }
        fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::String => {
                    let len = self.pop_len()?;
                    self.reader.deserialize_str(len, visitor)
                }
                Tag::MarkerTerminatedString => {
                    self.reader
                        .deserialize_unknown_len_str(visitor, crate::tag::end_of_str)
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::String,
                        Tag::MarkerTerminatedString,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_str(visitor)
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::Bytes => {
                    let len = self.pop_len()?;
                    self.reader.deserialize_bytes(len, visitor)
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::Bytes];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_bytes(visitor)
        }
        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::None => visitor.visit_none(),
                Tag::Some => visitor.visit_some(self),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::None, Tag::Some];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::Unit => visitor.visit_unit(),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::Unit];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_unit_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::UnitStruct => visitor.visit_unit(),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::UnitStruct];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_newtype_struct<V>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::NewTypeStruct => visitor.visit_newtype_struct(self),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::NewTypeStruct];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            let len = match self.pop_tag()? {
                Tag::Seq | Tag::Tuple | Tag::TupleStruct => self.pop_len().map(Some),
                Tag::UnsizedSeq => Ok(None),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::Seq,
                        Tag::Tuple,
                        Tag::TupleStruct,
                        Tag::UnsizedSeq,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }?;
            visitor.visit_seq(SeqDeserializer::new(self, len))
        }
        fn deserialize_tuple<V>(
            self,
            _len: usize,
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }
        fn deserialize_tuple_struct<V>(
            self,
            _name: &'static str,
            _len: usize,
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }
        fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            let len = match self.pop_tag()? {
                Tag::Map | Tag::Struct => self.pop_len().map(Some),
                Tag::UnsizedMap => Ok(None),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::Map,
                        Tag::Struct,
                        Tag::UnsizedMap,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }?;
            visitor.visit_map(SeqDeserializer::new(self, len))
        }
        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_map(visitor)
        }
        fn deserialize_enum<V>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_enum(self)
        }
        fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            let tag = self.pop_tag()?;
            match tag {
                Tag::UnitVariant
                | Tag::NewTypeVariant
                | Tag::TupleVariant
                | Tag::StructVariant => {
                    let variant_index = self.pop_variant()?;
                    let value = visitor.visit_u32::<Self::Error>(variant_index)?;
                    self.peeked_tag = Some(tag);
                    Ok(value)
                }
                Tag::String => {
                    let len = self.pop_len()?;
                    self.reader.deserialize_str(len, visitor)
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[
                        Tag::UnitVariant,
                        Tag::NewTypeVariant,
                        Tag::TupleVariant,
                        Tag::StructVariant,
                        Tag::String,
                    ];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_any(visitor)
        }
    }
    struct SeqDeserializer<'a, R> {
        de: &'a mut Deserializer<R>,
        remaining: Option<usize>,
    }
    impl<'a, 'de: 'a, R: Read<'de>> SeqDeserializer<'a, R> {
        fn new(de: &'a mut Deserializer<R>, len: Option<usize>) -> Self {
            SeqDeserializer {
                de,
                remaining: len,
            }
        }
        fn parse_next<T>(&mut self, seed: T) -> Result<Option<T::Value>, R::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            if let Some(remaining) = self.remaining.as_mut() {
                if *remaining == 0 {
                    return Ok(None);
                }
                *remaining -= 1;
            } else if self.de.peek_tag()? == Tag::UnsizedSeqEnd {
                self.de.pop_tag()?;
                return Ok(None);
            }
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
    impl<'a, 'de: 'a, R: Read<'de>> de::SeqAccess<'de> for SeqDeserializer<'a, R> {
        type Error = Error<R::Error>;
        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, R::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            self.parse_next(seed)
        }
        fn size_hint(&self) -> Option<usize> {
            self.remaining
        }
    }
    impl<'a, 'de: 'a, R: Read<'de>> de::MapAccess<'de> for SeqDeserializer<'a, R> {
        type Error = Error<R::Error>;
        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, R::Error>
        where
            K: de::DeserializeSeed<'de>,
        {
            self.parse_next(seed)
        }
        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, R::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            seed.deserialize(&mut *self.de)
        }
        fn size_hint(&self) -> Option<usize> {
            self.remaining
        }
    }
    impl<'a, 'de: 'a, R: Read<'de>> de::EnumAccess<'de> for &'a mut Deserializer<R> {
        type Error = Error<R::Error>;
        type Variant = Self;
        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), R::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            let val = seed.deserialize(&mut *self)?;
            Ok((val, self))
        }
    }
    impl<'a, 'de: 'a, R: Read<'de>> de::VariantAccess<'de> for &'a mut Deserializer<R> {
        type Error = Error<R::Error>;
        fn unit_variant(self) -> Result<(), R::Error> {
            match self.pop_tag()? {
                Tag::UnitVariant => Ok(()),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::UnitVariant];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, R::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.pop_tag()? {
                Tag::NewTypeVariant => seed.deserialize(self),
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::NewTypeVariant];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::TupleVariant => {
                    visitor.visit_seq(SeqDeserializer::new(self, Some(len)))
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::TupleVariant];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
        fn struct_variant<V>(
            self,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            match self.pop_tag()? {
                Tag::StructVariant => {
                    visitor.visit_seq(SeqDeserializer::new(self, Some(fields.len())))
                }
                tag => {
                    const EXPECTED_TAGS: &[Tag] = &[Tag::StructVariant];
                    Err(
                        Error::UnexpectedTag(UnexpectedTag {
                            got: tag,
                            expected: EXPECTED_TAGS,
                        }),
                    )
                }
            }
        }
    }
}
pub mod error {
    use core::fmt::{self, Debug, Display};
    use core::str::Utf8Error;
    use serde::{de, ser};
    #[cfg(feature = "std")]
    pub(crate) use std::error::Error as RWError;
    #[cfg(feature = "alloc")]
    use alloc::string::{String, ToString};
    use crate::tag::{Tag, TagParsingError};
    pub enum NoRWError {}
    #[automatically_derived]
    impl ::core::fmt::Debug for NoRWError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for NoRWError {
        #[inline]
        fn clone(&self) -> NoRWError {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for NoRWError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for NoRWError {
        #[inline]
        fn eq(&self, other: &NoRWError) -> bool {
            unsafe { ::core::intrinsics::unreachable() }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for NoRWError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for NoRWError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl Display for NoRWError {
        fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            ::core::panicking::panic("internal error: entered unreachable code")
        }
    }
    #[cfg(feature = "std")]
    impl std::error::Error for NoRWError {}
    pub enum SerError<We> {
        WriteError(We),
        #[cfg(feature = "alloc")]
        Custom(String),
    }
    #[automatically_derived]
    impl<We: ::core::fmt::Debug> ::core::fmt::Debug for SerError<We> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                SerError::WriteError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "WriteError",
                        &__self_0,
                    )
                }
                SerError::Custom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Custom",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl<We: ::core::clone::Clone> ::core::clone::Clone for SerError<We> {
        #[inline]
        fn clone(&self) -> SerError<We> {
            match self {
                SerError::WriteError(__self_0) => {
                    SerError::WriteError(::core::clone::Clone::clone(__self_0))
                }
                SerError::Custom(__self_0) => {
                    SerError::Custom(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl<We> ::core::marker::StructuralPartialEq for SerError<We> {}
    #[automatically_derived]
    impl<We: ::core::cmp::PartialEq> ::core::cmp::PartialEq for SerError<We> {
        #[inline]
        fn eq(&self, other: &SerError<We>) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (SerError::WriteError(__self_0), SerError::WriteError(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (SerError::Custom(__self_0), SerError::Custom(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl<We> ::core::marker::StructuralEq for SerError<We> {}
    #[automatically_derived]
    impl<We: ::core::cmp::Eq> ::core::cmp::Eq for SerError<We> {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<We>;
            let _: ::core::cmp::AssertParamIsEq<String>;
        }
    }
    impl<We: RWError> Display for SerError<We> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                SerError::WriteError(err) => Display::fmt(err, f),
                #[cfg(feature = "alloc")]
                SerError::Custom(err) => Display::fmt(err, f),
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
    }
    impl<We: RWError> From<We> for SerError<We> {
        fn from(value: We) -> Self {
            SerError::WriteError(value)
        }
    }
    pub enum DeError<E> {
        ReaderError(E),
        TagParsingError(TagParsingError),
        Utf8Error(Utf8Error),
        InvalidLen(u64),
        UnexpectedTag(UnexpectedTag),
        #[cfg(feature = "alloc")]
        Custom(String),
    }
    #[automatically_derived]
    impl<E: ::core::fmt::Debug> ::core::fmt::Debug for DeError<E> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                DeError::ReaderError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ReaderError",
                        &__self_0,
                    )
                }
                DeError::TagParsingError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "TagParsingError",
                        &__self_0,
                    )
                }
                DeError::Utf8Error(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Utf8Error",
                        &__self_0,
                    )
                }
                DeError::InvalidLen(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InvalidLen",
                        &__self_0,
                    )
                }
                DeError::UnexpectedTag(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UnexpectedTag",
                        &__self_0,
                    )
                }
                DeError::Custom(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Custom",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl<E: ::core::clone::Clone> ::core::clone::Clone for DeError<E> {
        #[inline]
        fn clone(&self) -> DeError<E> {
            match self {
                DeError::ReaderError(__self_0) => {
                    DeError::ReaderError(::core::clone::Clone::clone(__self_0))
                }
                DeError::TagParsingError(__self_0) => {
                    DeError::TagParsingError(::core::clone::Clone::clone(__self_0))
                }
                DeError::Utf8Error(__self_0) => {
                    DeError::Utf8Error(::core::clone::Clone::clone(__self_0))
                }
                DeError::InvalidLen(__self_0) => {
                    DeError::InvalidLen(::core::clone::Clone::clone(__self_0))
                }
                DeError::UnexpectedTag(__self_0) => {
                    DeError::UnexpectedTag(::core::clone::Clone::clone(__self_0))
                }
                DeError::Custom(__self_0) => {
                    DeError::Custom(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    #[automatically_derived]
    impl<E> ::core::marker::StructuralPartialEq for DeError<E> {}
    #[automatically_derived]
    impl<E: ::core::cmp::PartialEq> ::core::cmp::PartialEq for DeError<E> {
        #[inline]
        fn eq(&self, other: &DeError<E>) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (DeError::ReaderError(__self_0), DeError::ReaderError(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        DeError::TagParsingError(__self_0),
                        DeError::TagParsingError(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (DeError::Utf8Error(__self_0), DeError::Utf8Error(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (DeError::InvalidLen(__self_0), DeError::InvalidLen(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    (
                        DeError::UnexpectedTag(__self_0),
                        DeError::UnexpectedTag(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (DeError::Custom(__self_0), DeError::Custom(__arg1_0)) => {
                        *__self_0 == *__arg1_0
                    }
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl<E> ::core::marker::StructuralEq for DeError<E> {}
    #[automatically_derived]
    impl<E: ::core::cmp::Eq> ::core::cmp::Eq for DeError<E> {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<E>;
            let _: ::core::cmp::AssertParamIsEq<TagParsingError>;
            let _: ::core::cmp::AssertParamIsEq<Utf8Error>;
            let _: ::core::cmp::AssertParamIsEq<u64>;
            let _: ::core::cmp::AssertParamIsEq<UnexpectedTag>;
            let _: ::core::cmp::AssertParamIsEq<String>;
        }
    }
    impl<E: RWError> Display for DeError<E> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DeError::ReaderError(err) => Display::fmt(err, f),
                DeError::TagParsingError(err) => Display::fmt(err, f),
                DeError::Utf8Error(err) => Display::fmt(err, f),
                #[cfg(feature = "alloc")]
                DeError::Custom(err) => Display::fmt(err, f),
                DeError::UnexpectedTag(err) => Display::fmt(err, f),
                DeError::InvalidLen(len) => {
                    f.write_fmt(format_args!("Sequence len is too big: {0}", len))
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
    pub struct UnexpectedTag {
        pub expected: &'static [Tag],
        pub got: Tag,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnexpectedTag {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnexpectedTag",
                "expected",
                &self.expected,
                "got",
                &&self.got,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UnexpectedTag {
        #[inline]
        fn clone(&self) -> UnexpectedTag {
            let _: ::core::clone::AssertParamIsClone<&'static [Tag]>;
            let _: ::core::clone::AssertParamIsClone<Tag>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnexpectedTag {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UnexpectedTag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UnexpectedTag {
        #[inline]
        fn eq(&self, other: &UnexpectedTag) -> bool {
            self.expected == other.expected && self.got == other.got
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for UnexpectedTag {}
    #[automatically_derived]
    impl ::core::cmp::Eq for UnexpectedTag {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<&'static [Tag]>;
            let _: ::core::cmp::AssertParamIsEq<Tag>;
        }
    }
    impl Display for UnexpectedTag {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(
                format_args!(
                    "Unexpected tag, got {0:?} but expected {1:?}", self.got, self
                    .expected
                ),
            )
        }
    }
    pub struct EndOfBuff;
    #[automatically_derived]
    impl ::core::fmt::Debug for EndOfBuff {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "EndOfBuff")
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for EndOfBuff {
        #[inline]
        fn clone(&self) -> EndOfBuff {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for EndOfBuff {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for EndOfBuff {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for EndOfBuff {
        #[inline]
        fn eq(&self, other: &EndOfBuff) -> bool {
            true
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for EndOfBuff {}
    #[automatically_derived]
    impl ::core::cmp::Eq for EndOfBuff {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for EndOfBuff {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {}
    }
    impl Display for EndOfBuff {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("Reached end of buffer before end of serialization.")
        }
    }
    #[cfg(feature = "std")]
    impl std::error::Error for EndOfBuff {}
}
pub mod ser {
    use crate::error::{EndOfBuff, NoRWError};
    use crate::tag::{Tag, UNSIZED_STRING_END_MARKER};
    use crate::utils::write::{BuffWriter, DummyWriter, Write};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;
    use core::fmt;
    use serde::{ser, serde_if_integer128, Serialize};
    #[cfg(feature = "std")]
    use std::io;
    pub type Error<We = NoRWError> = crate::error::SerError<We>;
    pub type Result<T, We = NoRWError> = core::result::Result<T, Error<We>>;
    pub struct Serializer<W> {
        writer: W,
    }
    impl<W: Write> Serializer<W> {
        pub fn new(writer: W) -> Self {
            Serializer { writer }
        }
        pub fn to_writer<T: ?Sized>(value: &T, writer: W) -> Result<usize, W::Error>
        where
            T: Serialize,
        {
            let mut serializer = Serializer::new(writer);
            value.serialize(&mut serializer)
        }
        fn write_byte(&mut self, byte: u8) -> Result<usize, W::Error> {
            self.writer.write_byte(byte).map_err(Into::into)
        }
        fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, W::Error> {
            self.writer.write_bytes(bytes).map_err(Into::into)
        }
        fn write_tag(&mut self, tag: Tag) -> Result<usize, W::Error> {
            self.write_byte(tag.into())
        }
        fn write_tag_then_serialize<T: ?Sized>(
            &mut self,
            tag: Tag,
            value: &T,
        ) -> Result<usize, W::Error>
        where
            T: Serialize,
        {
            let mut wb = self.write_tag(tag)?;
            wb += value.serialize(self)?;
            Ok(wb)
        }
        fn write_tag_then_bytes(
            &mut self,
            tag: Tag,
            bytes: &[u8],
        ) -> Result<usize, W::Error> {
            let mut wb = self.write_byte(tag.into())?;
            wb += self.write_bytes(bytes)?;
            Ok(wb)
        }
        fn write_tag_then_len(
            &mut self,
            tag: Tag,
            len: usize,
        ) -> Result<usize, W::Error> {
            self.write_tag_then_serialize(tag, &len)
        }
        fn write_tag_then_seq(
            &mut self,
            tag: Tag,
            bytes: &[u8],
        ) -> Result<usize, W::Error> {
            let mut wb = self.write_tag_then_len(tag, bytes.len())?;
            wb += self.write_bytes(bytes)?;
            Ok(wb)
        }
        fn write_tag_then_variant(
            &mut self,
            tag: Tag,
            variant_index: u32,
        ) -> Result<usize, W::Error> {
            self.write_tag_then_serialize(tag, &variant_index)
        }
    }
    pub fn to_writer<W, T: ?Sized>(value: &T, writer: W) -> Result<usize, W::Error>
    where
        T: Serialize,
        W: Write,
    {
        Serializer::to_writer(value, writer)
    }
    #[cfg(feature = "std")]
    pub fn to_bytes<T: ?Sized>(value: &T) -> Result<Vec<u8>, io::Error>
    where
        T: Serialize,
    {
        let mut output = Vec::new();
        Serializer::to_writer(value, &mut output)?;
        Ok(output)
    }
    pub fn to_buff<'a, T: ?Sized>(
        value: &T,
        buff: &'a mut [u8],
    ) -> Result<BuffWriter<'a>, EndOfBuff>
    where
        T: Serialize,
    {
        let mut buff_writer = BuffWriter::new(buff);
        Serializer::to_writer(value, &mut buff_writer)?;
        Ok(buff_writer)
    }
    pub fn get_serialized_size<T: ?Sized>(value: &T) -> Result<usize>
    where
        T: Serialize,
    {
        Serializer::to_writer(value, DummyWriter)
    }
    impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        type SerializeSeq = SeqSerializer<'a, W>;
        type SerializeTuple = SeqSerializer<'a, W>;
        type SerializeTupleStruct = SeqSerializer<'a, W>;
        type SerializeTupleVariant = SeqSerializer<'a, W>;
        type SerializeMap = SeqSerializer<'a, W>;
        type SerializeStruct = SeqSerializer<'a, W>;
        type SerializeStructVariant = SeqSerializer<'a, W>;
        fn serialize_bool(self, v: bool) -> Result<Self::Ok, W::Error> {
            let tag = if v { Tag::BoolTrue } else { Tag::BoolFalse };
            self.write_tag(tag)
        }
        fn serialize_i8(self, value: i8) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::I8, &value.to_be_bytes())
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_i16(self, value: i16) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <i8>::try_from(value) {
                self.serialize_i8(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_i32(self, value: i32) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <i16>::try_from(value) {
                self.serialize_i16(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_i64(self, value: i64) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <i32>::try_from(value) {
                self.serialize_i32(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        fn serialize_u8(self, value: u8) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::U8, &value.to_be_bytes())
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_u16(self, value: u16) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <u8>::try_from(value) {
                self.serialize_u8(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_u32(self, value: u32) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <u16>::try_from(value) {
                self.serialize_u16(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        #[cfg(feature = "compact-nums")]
        fn serialize_u64(self, value: u64) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <u32>::try_from(value) {
                self.serialize_u32(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }
        fn serialize_f32(self, value: f32) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::F32, &value.to_be_bytes())
        }
        fn serialize_f64(self, value: f64) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::F64, &value.to_be_bytes())
        }
        fn serialize_i128(self, value: i128) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::I128, &value.to_be_bytes())
        }
        fn serialize_u128(self, value: u128) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes(Tag::U128, &value.to_be_bytes())
        }
        fn serialize_char(self, v: char) -> Result<Self::Ok, W::Error> {
            let mut buff = [0; 4];
            let (tag, bytes) = Tag::encode_char(v, &mut buff);
            self.write_tag_then_bytes(tag, bytes)
        }
        fn serialize_str(self, v: &str) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_seq(Tag::String, v.as_bytes())
        }
        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_seq(Tag::Bytes, v)
        }
        fn serialize_none(self) -> Result<Self::Ok, W::Error> {
            self.write_tag(Tag::None)
        }
        fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, W::Error>
        where
            T: Serialize,
        {
            self.write_tag_then_serialize(Tag::Some, value)
        }
        fn serialize_unit(self) -> Result<Self::Ok, W::Error> {
            self.write_tag(Tag::Unit)
        }
        fn serialize_unit_struct(
            self,
            _name: &'static str,
        ) -> Result<Self::Ok, W::Error> {
            self.write_tag(Tag::UnitStruct)
        }
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_variant(Tag::UnitVariant, variant_index)
        }
        fn serialize_newtype_struct<T: ?Sized>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Self::Ok, W::Error>
        where
            T: Serialize,
        {
            self.write_tag_then_serialize(Tag::NewTypeStruct, value)
        }
        fn serialize_newtype_variant<T: ?Sized>(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            value: &T,
        ) -> Result<Self::Ok, W::Error>
        where
            T: Serialize,
        {
            let mut wb = self
                .write_tag_then_variant(Tag::NewTypeVariant, variant_index)?;
            wb += value.serialize(self)?;
            Ok(wb)
        }
        fn serialize_seq(
            self,
            len: Option<usize>,
        ) -> Result<Self::SerializeSeq, W::Error> {
            match len {
                Some(len) => {
                    let written_bytes = self.write_tag_then_len(Tag::Seq, len)?;
                    Ok(SeqSerializer::new(self, written_bytes, true))
                }
                None => {
                    let written_bytes = self.write_tag(Tag::UnsizedSeq)?;
                    Ok(SeqSerializer::new(self, written_bytes, false))
                }
            }
        }
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, W::Error> {
            let wb = self.write_tag_then_len(Tag::Tuple, len)?;
            Ok(SeqSerializer::new(self, wb, true))
        }
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, W::Error> {
            let wb = self.write_tag_then_len(Tag::TupleStruct, len)?;
            Ok(SeqSerializer::new(self, wb, true))
        }
        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, W::Error> {
            let wb = self.write_tag_then_variant(Tag::TupleVariant, variant_index)?;
            Ok(SeqSerializer::new(self, wb, true))
        }
        fn serialize_map(
            self,
            len: Option<usize>,
        ) -> Result<Self::SerializeMap, W::Error> {
            match len {
                Some(len) => {
                    let wb = self.write_tag_then_len(Tag::Map, len)?;
                    Ok(SeqSerializer::new(self, wb, true))
                }
                None => {
                    let written_bytes = self.write_tag(Tag::UnsizedMap)?;
                    Ok(SeqSerializer::new(self, written_bytes, false))
                }
            }
        }
        fn serialize_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStruct, W::Error> {
            let wb = self.write_tag_then_len(Tag::Struct, len)?;
            Ok(SeqSerializer::new(self, wb, true))
        }
        fn serialize_struct_variant(
            self,
            _name: &'static str,
            variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, W::Error> {
            let wb = self.write_tag_then_variant(Tag::StructVariant, variant_index)?;
            Ok(SeqSerializer::new(self, wb, true))
        }
        fn is_human_readable(&self) -> bool {
            false
        }
        fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, W::Error>
        where
            T: fmt::Display,
        {
            use ser::Error as _;
            let mut wb = self.write_tag(Tag::MarkerTerminatedString)?;
            let mut collector = StrCollector::new(&mut self.writer);
            if fmt::write(&mut collector, format_args!("{0}", value)).is_err() {
                let err = match collector.error {
                    Some(err) => Error::WriteError(err),
                    None => Error::custom("Something went really wrong."),
                };
                return Err(err);
            }
            wb += collector.written_bytes;
            wb += self.writer.write_bytes(&UNSIZED_STRING_END_MARKER)?;
            Ok(wb)
        }
    }
    pub struct SeqSerializer<'a, W> {
        serializer: &'a mut Serializer<W>,
        written_bytes_count: usize,
        known_size: bool,
    }
    impl<'a, W: Write> SeqSerializer<'a, W> {
        pub fn new(
            serializer: &'a mut Serializer<W>,
            written_bytes: usize,
            known_size: bool,
        ) -> Self {
            Self {
                serializer,
                written_bytes_count: written_bytes,
                known_size,
            }
        }
        pub fn ser_value<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.written_bytes_count += value.serialize(&mut *self.serializer)?;
            Ok(())
        }
        pub fn finish(mut self) -> Result<usize, W::Error> {
            if !self.known_size {
                self.written_bytes_count
                    += self.serializer.write_tag(Tag::UnsizedSeqEnd)?;
            }
            Ok(self.written_bytes_count)
        }
    }
    impl<'a, W: Write> ser::SerializeSeq for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeTuple for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeTupleStruct for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeTupleVariant for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeMap for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(key)
        }
        fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            self.ser_value(value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeStruct for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_field<T: ?Sized>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            use ser::SerializeMap;
            self.serialize_entry(key, value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    impl<'a, W: Write> ser::SerializeStructVariant for SeqSerializer<'a, W> {
        type Ok = usize;
        type Error = Error<W::Error>;
        fn serialize_field<T: ?Sized>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), W::Error>
        where
            T: Serialize,
        {
            use ser::SerializeMap;
            self.serialize_entry(key, value)
        }
        fn end(self) -> Result<Self::Ok, W::Error> {
            self.finish()
        }
    }
    struct StrCollector<'a, W: Write> {
        writer: &'a mut W,
        written_bytes: usize,
        error: Option<W::Error>,
    }
    impl<'a, W: Write> StrCollector<'a, W> {
        pub fn new(writer: &'a mut W) -> Self {
            StrCollector {
                writer,
                written_bytes: 0,
                error: None,
            }
        }
    }
    impl<'a, W: Write> fmt::Write for StrCollector<'a, W> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            match self.writer.write_bytes(s.as_bytes()) {
                Ok(written_bytes) => {
                    self.written_bytes += written_bytes;
                    Ok(())
                }
                Err(err) => {
                    self.error = Some(err);
                    Err(fmt::Error)
                }
            }
        }
    }
}
mod tag {
    use core::fmt::Display;
    pub const UNSIZED_STRING_END_MARKER: [u8; 2] = [0xD8, 0x00];
    pub fn end_of_str(bytes: &[u8; 2]) -> bool {
        bytes == &UNSIZED_STRING_END_MARKER
    }
    #[repr(u8)]
    pub enum Tag {
        None = 0,
        Some = 1,
        BoolFalse = 2,
        BoolTrue = 3,
        I8 = 4,
        I16 = 5,
        I32 = 6,
        I64 = 7,
        U8 = 8,
        U16 = 9,
        U32 = 10,
        U64 = 11,
        F32 = 12,
        F64 = 13,
        Char1 = 14,
        Char2 = 15,
        Char3 = 16,
        Char4 = 17,
        String = 18,
        MarkerTerminatedString = 19,
        Bytes = 20,
        Unit = 21,
        UnitStruct = 22,
        UnitVariant = 23,
        NewTypeStruct = 24,
        NewTypeVariant = 25,
        Seq = 26,
        UnsizedSeq = 27,
        UnsizedSeqEnd = 28,
        Tuple = 29,
        TupleStruct = 30,
        TupleVariant = 31,
        Map = 32,
        UnsizedMap = 33,
        Struct = 34,
        StructVariant = 35,
        #[cfg(not(no_integer128))]
        I128 = 36,
        #[cfg(not(no_integer128))]
        U128 = 37,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Tag {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Tag::None => "None",
                    Tag::Some => "Some",
                    Tag::BoolFalse => "BoolFalse",
                    Tag::BoolTrue => "BoolTrue",
                    Tag::I8 => "I8",
                    Tag::I16 => "I16",
                    Tag::I32 => "I32",
                    Tag::I64 => "I64",
                    Tag::U8 => "U8",
                    Tag::U16 => "U16",
                    Tag::U32 => "U32",
                    Tag::U64 => "U64",
                    Tag::F32 => "F32",
                    Tag::F64 => "F64",
                    Tag::Char1 => "Char1",
                    Tag::Char2 => "Char2",
                    Tag::Char3 => "Char3",
                    Tag::Char4 => "Char4",
                    Tag::String => "String",
                    Tag::MarkerTerminatedString => "MarkerTerminatedString",
                    Tag::Bytes => "Bytes",
                    Tag::Unit => "Unit",
                    Tag::UnitStruct => "UnitStruct",
                    Tag::UnitVariant => "UnitVariant",
                    Tag::NewTypeStruct => "NewTypeStruct",
                    Tag::NewTypeVariant => "NewTypeVariant",
                    Tag::Seq => "Seq",
                    Tag::UnsizedSeq => "UnsizedSeq",
                    Tag::UnsizedSeqEnd => "UnsizedSeqEnd",
                    Tag::Tuple => "Tuple",
                    Tag::TupleStruct => "TupleStruct",
                    Tag::TupleVariant => "TupleVariant",
                    Tag::Map => "Map",
                    Tag::UnsizedMap => "UnsizedMap",
                    Tag::Struct => "Struct",
                    Tag::StructVariant => "StructVariant",
                    Tag::I128 => "I128",
                    Tag::U128 => "U128",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Tag {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Tag {
        #[inline]
        fn eq(&self, other: &Tag) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for Tag {}
    #[automatically_derived]
    impl ::core::cmp::Eq for Tag {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Tag {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Tag {
        #[inline]
        fn clone(&self) -> Tag {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Tag {}
    impl Tag {
        pub fn encode_char(c: char, buff: &mut [u8]) -> (Self, &[u8]) {
            let bytes = c.encode_utf8(buff).as_bytes();
            let tag = match bytes.len() {
                1 => Tag::Char1,
                2 => Tag::Char2,
                3 => Tag::Char3,
                4 => Tag::Char4,
                _ => ::core::panicking::panic("internal error: entered unreachable code"),
            };
            (tag, bytes)
        }
    }
    pub enum TagParsingError {
        InvalidTag(u8),
        UnexpectedTag { expected: &'static str, got: Tag },
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TagParsingError {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                TagParsingError::InvalidTag(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InvalidTag",
                        &__self_0,
                    )
                }
                TagParsingError::UnexpectedTag { expected: __self_0, got: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "UnexpectedTag",
                        "expected",
                        __self_0,
                        "got",
                        &__self_1,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for TagParsingError {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for TagParsingError {
        #[inline]
        fn eq(&self, other: &TagParsingError) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
                && match (self, other) {
                    (
                        TagParsingError::InvalidTag(__self_0),
                        TagParsingError::InvalidTag(__arg1_0),
                    ) => *__self_0 == *__arg1_0,
                    (
                        TagParsingError::UnexpectedTag {
                            expected: __self_0,
                            got: __self_1,
                        },
                        TagParsingError::UnexpectedTag {
                            expected: __arg1_0,
                            got: __arg1_1,
                        },
                    ) => *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1,
                    _ => unsafe { ::core::intrinsics::unreachable() }
                }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralEq for TagParsingError {}
    #[automatically_derived]
    impl ::core::cmp::Eq for TagParsingError {
        #[inline]
        #[doc(hidden)]
        #[no_coverage]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<u8>;
            let _: ::core::cmp::AssertParamIsEq<&'static str>;
            let _: ::core::cmp::AssertParamIsEq<Tag>;
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for TagParsingError {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state);
            match self {
                TagParsingError::InvalidTag(__self_0) => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                TagParsingError::UnexpectedTag { expected: __self_0, got: __self_1 } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state)
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TagParsingError {
        #[inline]
        fn clone(&self) -> TagParsingError {
            let _: ::core::clone::AssertParamIsClone<u8>;
            let _: ::core::clone::AssertParamIsClone<&'static str>;
            let _: ::core::clone::AssertParamIsClone<Tag>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for TagParsingError {}
    impl TagParsingError {
        pub fn unexpected(expected: &'static str, got: Tag) -> Self {
            Self::UnexpectedTag {
                expected,
                got,
            }
        }
    }
    impl Display for TagParsingError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                TagParsingError::InvalidTag(tag) => {
                    f.write_fmt(
                        format_args!(
                            "Invalid tag for data type: expected byte beetween 0 and 31 included, got {0}",
                            tag
                        ),
                    )
                }
                TagParsingError::UnexpectedTag { expected, got } => {
                    f.write_fmt(
                        format_args!("Expected {0} but got {1:?}", expected, got),
                    )
                }
            }
        }
    }
    impl TryFrom<u8> for Tag {
        type Error = TagParsingError;
        fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
            match value {
                0 => Ok(Tag::None),
                1 => Ok(Tag::Some),
                2 => Ok(Tag::BoolFalse),
                3 => Ok(Tag::BoolTrue),
                4 => Ok(Tag::I8),
                5 => Ok(Tag::I16),
                6 => Ok(Tag::I32),
                7 => Ok(Tag::I64),
                8 => Ok(Tag::U8),
                9 => Ok(Tag::U16),
                10 => Ok(Tag::U32),
                11 => Ok(Tag::U64),
                12 => Ok(Tag::F32),
                13 => Ok(Tag::F64),
                14 => Ok(Tag::Char1),
                15 => Ok(Tag::Char2),
                16 => Ok(Tag::Char3),
                17 => Ok(Tag::Char4),
                18 => Ok(Tag::String),
                19 => Ok(Tag::MarkerTerminatedString),
                20 => Ok(Tag::Bytes),
                21 => Ok(Tag::Unit),
                22 => Ok(Tag::UnitStruct),
                23 => Ok(Tag::UnitVariant),
                24 => Ok(Tag::NewTypeStruct),
                25 => Ok(Tag::NewTypeVariant),
                26 => Ok(Tag::Seq),
                27 => Ok(Tag::UnsizedSeq),
                28 => Ok(Tag::UnsizedSeqEnd),
                29 => Ok(Tag::Tuple),
                30 => Ok(Tag::TupleStruct),
                31 => Ok(Tag::TupleVariant),
                32 => Ok(Tag::Map),
                33 => Ok(Tag::UnsizedMap),
                34 => Ok(Tag::Struct),
                35 => Ok(Tag::StructVariant),
                #[cfg(not(no_integer128))]
                36 => Ok(Tag::I128),
                #[cfg(not(no_integer128))]
                37 => Ok(Tag::U128),
                tag => Err(TagParsingError::InvalidTag(tag)),
            }
        }
    }
    impl From<Tag> for u8 {
        fn from(value: Tag) -> Self {
            value as u8
        }
    }
}
mod utils {
    pub mod read {
        use serde::de::Visitor;
        use crate::error::{DeError, EndOfBuff, RWError};
        pub trait Read<'de> {
            type Error: RWError;
            fn read_bytes(&mut self, buff: &mut [u8]) -> Result<(), Self::Error>;
            fn read_byte(&mut self) -> Result<u8, Self::Error> {
                let mut byte = 0;
                self.read_bytes(core::slice::from_mut(&mut byte))?;
                Ok(byte)
            }
            fn deserialize_str<V: Visitor<'de>>(
                &mut self,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, DeError<Self::Error>>;
            fn deserialize_bytes<V: Visitor<'de>>(
                &mut self,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, DeError<Self::Error>>;
            fn deserialize_unknown_len_str<V, F>(
                &mut self,
                visitor: V,
                end_of_str: F,
            ) -> Result<V::Value, DeError<Self::Error>>
            where
                V: Visitor<'de>,
                F: Fn(&[u8; 2]) -> bool;
        }
        pub struct BuffReader<'de> {
            buff: &'de [u8],
        }
        impl<'de> BuffReader<'de> {
            pub fn new(buff: &'de [u8]) -> Self {
                BuffReader { buff }
            }
            pub fn pop_slice(&mut self, len: usize) -> Result<&'de [u8], EndOfBuff> {
                if self.buff.len() < len {
                    Err(EndOfBuff)
                } else {
                    let (popped, rest) = self.buff.split_at(len);
                    self.buff = rest;
                    Ok(popped)
                }
            }
        }
        impl<'de> Read<'de> for BuffReader<'de> {
            type Error = EndOfBuff;
            fn read_byte(&mut self) -> Result<u8, Self::Error> {
                let (first, rest) = self.buff.split_first().ok_or(EndOfBuff)?;
                self.buff = rest;
                Ok(*first)
            }
            fn read_bytes(&mut self, buff: &mut [u8]) -> Result<(), Self::Error> {
                let to_copy = self.pop_slice(buff.len())?;
                buff.copy_from_slice(to_copy);
                Ok(())
            }
            fn deserialize_str<V: Visitor<'de>>(
                &mut self,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, DeError<Self::Error>> {
                let str = self.pop_slice(len)?;
                let str = core::str::from_utf8(str).map_err(DeError::Utf8Error)?;
                visitor.visit_borrowed_str(str)
            }
            fn deserialize_bytes<V: Visitor<'de>>(
                &mut self,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, DeError<Self::Error>> {
                let bytes = self.pop_slice(len)?;
                visitor.visit_borrowed_bytes(bytes)
            }
            fn deserialize_unknown_len_str<V, F>(
                &mut self,
                visitor: V,
                end_of_str: F,
            ) -> Result<V::Value, DeError<Self::Error>>
            where
                V: Visitor<'de>,
                F: Fn(&[u8; 2]) -> bool,
            {
                let len = self
                    .buff
                    .windows(2)
                    .position(|bytes| end_of_str(bytes.try_into().unwrap()))
                    .ok_or(EndOfBuff)?;
                self.deserialize_str(len, visitor)
            }
        }
    }
    pub mod write {
        use core::ops::{Deref, DerefMut};
        #[cfg(feature = "alloc")]
        extern crate alloc;
        #[cfg(feature = "std")]
        use std::io;
        use crate::error::{EndOfBuff, NoRWError, RWError};
        pub trait Write {
            type Error: RWError;
            fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
            fn write_byte(&mut self, byte: u8) -> Result<usize, Self::Error> {
                self.write_bytes(core::slice::from_ref(&byte))
            }
        }
        #[cfg(feature = "std")]
        impl<W: io::Write> Write for W {
            type Error = io::Error;
            fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
                self.write_all(bytes)?;
                Ok(bytes.len())
            }
        }
        pub struct BuffWriter<'a> {
            buff: &'a mut [u8],
            head: usize,
        }
        impl<'a> BuffWriter<'a> {
            pub fn new(buff: &'a mut [u8]) -> Self {
                BuffWriter { buff, head: 0 }
            }
            pub fn unwrap(self) -> (usize, &'a mut [u8]) {
                (self.head, self.buff)
            }
            pub fn len(&self) -> usize {
                self.head
            }
            pub fn is_empty(&self) -> bool {
                self.head == 0
            }
            pub fn get(&self) -> &[u8] {
                &self.buff[..self.head]
            }
            pub fn get_mut(&mut self) -> &mut [u8] {
                &mut self.buff[..self.head]
            }
        }
        impl<'a> Deref for BuffWriter<'a> {
            type Target = [u8];
            fn deref(&self) -> &Self::Target {
                self.get()
            }
        }
        impl<'a> DerefMut for BuffWriter<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.get_mut()
            }
        }
        impl<'a, 'b> Write for &'a mut BuffWriter<'b> {
            type Error = EndOfBuff;
            fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
                let spot = self
                    .buff
                    .get_mut(self.head..self.head + bytes.len())
                    .ok_or(EndOfBuff)?;
                spot.copy_from_slice(bytes);
                Ok(bytes.len())
            }
        }
        pub struct DummyWriter;
        impl Write for DummyWriter {
            type Error = NoRWError;
            fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
                Ok(bytes.len())
            }
        }
    }
}
pub use error::{DeError, NoRWError, SerError};
#[cfg(feature = "alloc")]
pub use ser::to_bytes;
pub use ser::{get_serialized_size, to_buff, to_writer, Serializer};
pub use utils::read;
pub use utils::write;
mod test {
    use serde::{Deserialize, Serialize};
    enum Test {
        Unit,
        NewType(u8),
        Struct { field: u8 },
        Tuple(u8, u8),
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for Test {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> _serde::__private::Result<Self, __D::Error>
            where
                __D: _serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                    __field3,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "variant identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            3u64 => _serde::__private::Ok(__Field::__field3),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"variant index 0 <= i < 4",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "Unit" => _serde::__private::Ok(__Field::__field0),
                            "NewType" => _serde::__private::Ok(__Field::__field1),
                            "Struct" => _serde::__private::Ok(__Field::__field2),
                            "Tuple" => _serde::__private::Ok(__Field::__field3),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"Unit" => _serde::__private::Ok(__Field::__field0),
                            b"NewType" => _serde::__private::Ok(__Field::__field1),
                            b"Struct" => _serde::__private::Ok(__Field::__field2),
                            b"Tuple" => _serde::__private::Ok(__Field::__field3),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_variant(__value, VARIANTS),
                                )
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de> {
                    marker: _serde::__private::PhantomData<Test>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                    type Value = Test;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(__formatter, "enum Test")
                    }
                    fn visit_enum<__A>(
                        self,
                        __data: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::EnumAccess<'de>,
                    {
                        match match _serde::de::EnumAccess::variant(__data) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            (__Field::__field0, __variant) => {
                                match _serde::de::VariantAccess::unit_variant(__variant) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                                _serde::__private::Ok(Test::Unit)
                            }
                            (__Field::__field1, __variant) => {
                                _serde::__private::Result::map(
                                    _serde::de::VariantAccess::newtype_variant::<u8>(__variant),
                                    Test::NewType,
                                )
                            }
                            (__Field::__field2, __variant) => {
                                #[allow(non_camel_case_types)]
                                #[doc(hidden)]
                                enum __Field {
                                    __field0,
                                    __ignore,
                                }
                                #[doc(hidden)]
                                struct __FieldVisitor;
                                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                                    type Value = __Field;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "field identifier",
                                        )
                                    }
                                    fn visit_u64<__E>(
                                        self,
                                        __value: u64,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            0u64 => _serde::__private::Ok(__Field::__field0),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_str<__E>(
                                        self,
                                        __value: &str,
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            "field" => _serde::__private::Ok(__Field::__field0),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                    fn visit_bytes<__E>(
                                        self,
                                        __value: &[u8],
                                    ) -> _serde::__private::Result<Self::Value, __E>
                                    where
                                        __E: _serde::de::Error,
                                    {
                                        match __value {
                                            b"field" => _serde::__private::Ok(__Field::__field0),
                                            _ => _serde::__private::Ok(__Field::__ignore),
                                        }
                                    }
                                }
                                impl<'de> _serde::Deserialize<'de> for __Field {
                                    #[inline]
                                    fn deserialize<__D>(
                                        __deserializer: __D,
                                    ) -> _serde::__private::Result<Self, __D::Error>
                                    where
                                        __D: _serde::Deserializer<'de>,
                                    {
                                        _serde::Deserializer::deserialize_identifier(
                                            __deserializer,
                                            __FieldVisitor,
                                        )
                                    }
                                }
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<Test>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = Test;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "struct variant Test::Struct",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 = match match _serde::de::SeqAccess::next_element::<
                                            u8,
                                        >(&mut __seq) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        } {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        0usize,
                                                        &"struct variant Test::Struct with 1 element",
                                                    ),
                                                );
                                            }
                                        };
                                        _serde::__private::Ok(Test::Struct { field: __field0 })
                                    }
                                    #[inline]
                                    fn visit_map<__A>(
                                        self,
                                        mut __map: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::MapAccess<'de>,
                                    {
                                        let mut __field0: _serde::__private::Option<u8> = _serde::__private::None;
                                        while let _serde::__private::Some(__key)
                                            = match _serde::de::MapAccess::next_key::<
                                                __Field,
                                            >(&mut __map) {
                                                _serde::__private::Ok(__val) => __val,
                                                _serde::__private::Err(__err) => {
                                                    return _serde::__private::Err(__err);
                                                }
                                            } {
                                            match __key {
                                                __Field::__field0 => {
                                                    if _serde::__private::Option::is_some(&__field0) {
                                                        return _serde::__private::Err(
                                                            <__A::Error as _serde::de::Error>::duplicate_field("field"),
                                                        );
                                                    }
                                                    __field0 = _serde::__private::Some(
                                                        match _serde::de::MapAccess::next_value::<u8>(&mut __map) {
                                                            _serde::__private::Ok(__val) => __val,
                                                            _serde::__private::Err(__err) => {
                                                                return _serde::__private::Err(__err);
                                                            }
                                                        },
                                                    );
                                                }
                                                _ => {
                                                    let _ = match _serde::de::MapAccess::next_value::<
                                                        _serde::de::IgnoredAny,
                                                    >(&mut __map) {
                                                        _serde::__private::Ok(__val) => __val,
                                                        _serde::__private::Err(__err) => {
                                                            return _serde::__private::Err(__err);
                                                        }
                                                    };
                                                }
                                            }
                                        }
                                        let __field0 = match __field0 {
                                            _serde::__private::Some(__field0) => __field0,
                                            _serde::__private::None => {
                                                match _serde::__private::de::missing_field("field") {
                                                    _serde::__private::Ok(__val) => __val,
                                                    _serde::__private::Err(__err) => {
                                                        return _serde::__private::Err(__err);
                                                    }
                                                }
                                            }
                                        };
                                        _serde::__private::Ok(Test::Struct { field: __field0 })
                                    }
                                }
                                #[doc(hidden)]
                                const FIELDS: &'static [&'static str] = &["field"];
                                _serde::de::VariantAccess::struct_variant(
                                    __variant,
                                    FIELDS,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<Test>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                            (__Field::__field3, __variant) => {
                                #[doc(hidden)]
                                struct __Visitor<'de> {
                                    marker: _serde::__private::PhantomData<Test>,
                                    lifetime: _serde::__private::PhantomData<&'de ()>,
                                }
                                impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                                    type Value = Test;
                                    fn expecting(
                                        &self,
                                        __formatter: &mut _serde::__private::Formatter,
                                    ) -> _serde::__private::fmt::Result {
                                        _serde::__private::Formatter::write_str(
                                            __formatter,
                                            "tuple variant Test::Tuple",
                                        )
                                    }
                                    #[inline]
                                    fn visit_seq<__A>(
                                        self,
                                        mut __seq: __A,
                                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                                    where
                                        __A: _serde::de::SeqAccess<'de>,
                                    {
                                        let __field0 = match match _serde::de::SeqAccess::next_element::<
                                            u8,
                                        >(&mut __seq) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        } {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        0usize,
                                                        &"tuple variant Test::Tuple with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        let __field1 = match match _serde::de::SeqAccess::next_element::<
                                            u8,
                                        >(&mut __seq) {
                                            _serde::__private::Ok(__val) => __val,
                                            _serde::__private::Err(__err) => {
                                                return _serde::__private::Err(__err);
                                            }
                                        } {
                                            _serde::__private::Some(__value) => __value,
                                            _serde::__private::None => {
                                                return _serde::__private::Err(
                                                    _serde::de::Error::invalid_length(
                                                        1usize,
                                                        &"tuple variant Test::Tuple with 2 elements",
                                                    ),
                                                );
                                            }
                                        };
                                        _serde::__private::Ok(Test::Tuple(__field0, __field1))
                                    }
                                }
                                _serde::de::VariantAccess::tuple_variant(
                                    __variant,
                                    2usize,
                                    __Visitor {
                                        marker: _serde::__private::PhantomData::<Test>,
                                        lifetime: _serde::__private::PhantomData,
                                    },
                                )
                            }
                        }
                    }
                }
                #[doc(hidden)]
                const VARIANTS: &'static [&'static str] = &[
                    "Unit",
                    "NewType",
                    "Struct",
                    "Tuple",
                ];
                _serde::Deserializer::deserialize_enum(
                    __deserializer,
                    "Test",
                    VARIANTS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<Test>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Serialize for Test {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> _serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: _serde::Serializer,
            {
                match *self {
                    Test::Unit => {
                        _serde::Serializer::serialize_unit_variant(
                            __serializer,
                            "Test",
                            0u32,
                            "Unit",
                        )
                    }
                    Test::NewType(ref __field0) => {
                        _serde::Serializer::serialize_newtype_variant(
                            __serializer,
                            "Test",
                            1u32,
                            "NewType",
                            __field0,
                        )
                    }
                    Test::Struct { ref field } => {
                        let mut __serde_state = match _serde::Serializer::serialize_struct_variant(
                            __serializer,
                            "Test",
                            2u32,
                            "Struct",
                            0 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStructVariant::serialize_field(
                            &mut __serde_state,
                            "field",
                            field,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStructVariant::end(__serde_state)
                    }
                    Test::Tuple(ref __field0, ref __field1) => {
                        let mut __serde_state = match _serde::Serializer::serialize_tuple_variant(
                            __serializer,
                            "Test",
                            3u32,
                            "Tuple",
                            0 + 1 + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field0,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeTupleVariant::serialize_field(
                            &mut __serde_state,
                            __field1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeTupleVariant::end(__serde_state)
                    }
                }
            }
        }
    };
}
