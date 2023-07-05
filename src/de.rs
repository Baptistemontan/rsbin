use crate::{
    error::{EndOfBuff, NoRWError, UnexpectedTag},
    read::{BuffReader, Read},
    tag::Tag,
};
use serde::{de, serde_if_integer128};
use serde::{de::Visitor, Deserialize};

pub type Error<Re = NoRWError> = crate::error::DeError<Re>;
pub type Result<T, Re = NoRWError> = core::result::Result<T, Error<Re>>;

pub struct Deserializer<R> {
    reader: R,
    peeked_tag: Option<Tag>,
}

pub fn from_bytes<'de, T: ?Sized>(bytes: &'de [u8]) -> core::result::Result<T, Error<EndOfBuff>>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::new(BuffReader::new(bytes));
    T::deserialize(&mut de)
}

macro_rules! match_tag {
    ($tag:expr, $($($pat:path)|+ => $body:expr),+) => {
        match $tag {
            $($($pat)|+ => $body,)+
            tag => {
                const EXPECTED_TAGS: &[Tag] = &[$($($pat),+),+];
                Err(Error::UnexpectedTag(UnexpectedTag { got: tag, expected: EXPECTED_TAGS }))
            }
        }
    };
}

macro_rules! implement_number_parsing {
    ($fn_name:ident, $t:ident, $expected_tag:path, $($tag:path, $small_fn:ident),+) => {
        fn $fn_name(&mut self) -> Result<$t, R::Error> {
            match_tag! { self.peek_tag()?,
                $expected_tag => {
                    self.pop_tag()?;
                    let bytes = self.pop_n()?;
                    Ok($t::from_be_bytes(bytes))
                },
                $($tag => self.$small_fn().map($t::from)),+
            }
        }
    };
    ($fn_name:ident, $t:ident, $expected_tag:path) => {
        fn $fn_name(&mut self) -> Result<$t, R::Error> {
            match_tag! { self.pop_tag()?,
                $expected_tag => {
                    let bytes = self.pop_n()?;
                    Ok($t::from_be_bytes(bytes))
                }
            }
        }
    };

    ($fn_name:ident, $t:ident, $expected_tag:path, $($next_fn_name:ident, $next_t:ident, $next_expected_tag:path),+) => {
        implement_number_parsing!($fn_name, $t, $expected_tag, $($next_expected_tag, $next_fn_name),+);
        implement_number_parsing!($($next_fn_name, $next_t, $next_expected_tag),+);
    }
}

macro_rules! implement_number {
    ($fn_name:ident, $parse_fn_name:ident, $visitor_fn_name:ident) => {
        fn $fn_name<V>(self, visitor: V) -> Result<V::Value, R::Error>
        where
            V: Visitor<'de>,
        {
            let num = self.$parse_fn_name()?;
            visitor.$visitor_fn_name(num)
        }
    };
}

impl<'de, R: Read<'de>> Deserializer<R> {
    fn new(reader: R) -> Self {
        Deserializer {
            reader,
            peeked_tag: None,
        }
    }

    fn pop_tag(&mut self) -> Result<Tag, R::Error> {
        if let Some(tag) = self.peeked_tag.take() {
            Ok(tag)
        } else {
            let byte = self.reader.read_byte()?;
            byte.try_into().map_err(Error::TagParsingError)
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
        self.reader.read_to_buff(&mut buff)?;
        Ok(buff)
    }

    #[cfg(feature = "alloc")]
    fn convert_bytes_cow_to_str(
        bytes: alloc::borrow::Cow<'_, [u8]>,
    ) -> core::result::Result<alloc::borrow::Cow<'_, str>, core::str::Utf8Error> {
        use alloc::borrow::Cow;
        match bytes {
            Cow::Borrowed(bytes) => core::str::from_utf8(bytes).map(Cow::Borrowed),
            Cow::Owned(bytes) => alloc::string::String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|err| err.utf8_error()),
        }
    }

    #[cfg(feature = "alloc")]
    fn pop_str(&mut self, len: usize) -> Result<alloc::borrow::Cow<'de, str>, R::Error> {
        let bytes = self.reader.read_bytes(len)?;
        Self::convert_bytes_cow_to_str(bytes).map_err(Error::Utf8Error)
    }

    #[cfg(not(feature = "alloc"))]
    fn pop_str(&mut self, len: usize) -> Result<&'de str, R::Error> {
        let bytes = self.reader.read_bytes(len)?;
        core::str::from_utf8(bytes).map_err(Error::Utf8Error)
    }

    #[cfg(feature = "alloc")]
    fn pop_unsized_str(&mut self) -> Result<alloc::borrow::Cow<'de, str>, R::Error> {
        use alloc::borrow::Cow;
        let bytes = self.reader.read_bytes_until(crate::tag::end_of_str)?;
        // Read::read_bytes_until contract states it has to end with the 2 bytes passed to the callback, so bytes.len() >= 2
        let len = bytes.len() - 2;
        // this part remove the 2 bytes of the string end marker.
        let bytes = match bytes {
            Cow::Owned(mut bytes) => {
                bytes.truncate(len);
                Cow::Owned(bytes)
            }
            Cow::Borrowed(bytes) => Cow::Borrowed(&bytes[..len]),
        };
        Self::convert_bytes_cow_to_str(bytes).map_err(Error::Utf8Error)
    }

    #[cfg(not(feature = "alloc"))]
    fn pop_unsized_str(&mut self) -> Result<&'de str, R::Error> {
        let bytes = self.reader.read_bytes_until(crate::tag::end_of_str)?;
        // Read::read_bytes_until contract states it has to end with the 2 bytes passed to the callback, so bytes.len() >= 2
        let len = bytes.len() - 2;
        // this part remove the 2 bytes of the string end marker.
        let bytes = &bytes[..len];
        core::str::from_utf8(bytes).map_err(Error::Utf8Error)
    }

    fn pop_len(&mut self) -> Result<usize, R::Error> {
        let len = self.parse_u64()?;
        len.try_into().map_err(|_| Error::InvalidLen(len))
    }

    fn pop_variant(&mut self) -> Result<u32, R::Error> {
        self.parse_u32()
    }

    implement_number_parsing!(
        parse_u64,
        u64,
        Tag::U64,
        parse_u32,
        u32,
        Tag::U32,
        parse_u16,
        u16,
        Tag::U16,
        parse_u8,
        u8,
        Tag::U8
    );

    implement_number_parsing!(
        parse_i64,
        i64,
        Tag::I64,
        parse_i32,
        i32,
        Tag::I32,
        parse_i16,
        i16,
        Tag::I16,
        parse_i8,
        i8,
        Tag::I8
    );

    implement_number_parsing!(parse_f64, f64, Tag::F64, parse_f32, f32, Tag::F32);

    serde_if_integer128! {
        implement_number_parsing!(
            parse_u128,
            u128,
            Tag::U128,
            Tag::U64,
            parse_u64,
            Tag::U32,
            parse_u32,
            Tag::U16,
            parse_u16,
            Tag::U8,
            parse_u8
        );
        implement_number_parsing!(
            parse_i128,
            i128,
            Tag::I128,
            Tag::I64,
            parse_i64,
            Tag::I32,
            parse_i32,
            Tag::I16,
            parse_i16,
            Tag::I8,
            parse_i8
        );
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
            Tag::Char1 | Tag::Char2 | Tag::Char3 | Tag::Char4 => self.deserialize_char(visitor),
            Tag::String | Tag::MarkerTerminatedString => self.deserialize_str(visitor),
            Tag::Bytes => self.deserialize_bytes(visitor),
            Tag::Unit => self.deserialize_unit(visitor),
            Tag::UnitStruct => self.deserialize_unit_struct("", visitor),
            Tag::NewTypeStruct => self.deserialize_newtype_struct("", visitor),
            Tag::Seq | Tag::UnsizedSeq | Tag::Tuple | Tag::TupleStruct => {
                self.deserialize_seq(visitor)
            }
            Tag::UnitVariant | Tag::NewTypeVariant | Tag::TupleVariant | Tag::StructVariant => {
                self.deserialize_enum("", &[], visitor)
            }
            Tag::Map | Tag::UnsizedMap | Tag::Struct => self.deserialize_map(visitor),
            #[cfg(not(no_integer128))]
            Tag::I128 => self.deserialize_i128(visitor),
            #[cfg(not(no_integer128))]
            Tag::U128 => self.deserialize_u128(visitor),
            got @ Tag::UnsizedSeqEnd => {
                Err(Error::UnexpectedTag(UnexpectedTag { expected: &[], got }))
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, R::Error>
    where
        V: de::Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?,
            Tag::BoolFalse => visitor.visit_bool(false),
            Tag::BoolTrue => visitor.visit_bool(true)
        }
    }

    implement_number!(deserialize_i8, parse_i8, visit_i8);
    implement_number!(deserialize_i16, parse_i16, visit_i16);
    implement_number!(deserialize_i32, parse_i32, visit_i32);
    implement_number!(deserialize_i64, parse_i64, visit_i64);
    implement_number!(deserialize_u8, parse_u8, visit_u8);
    implement_number!(deserialize_u16, parse_u16, visit_u16);
    implement_number!(deserialize_u32, parse_u32, visit_u32);
    implement_number!(deserialize_u64, parse_u64, visit_u64);
    implement_number!(deserialize_f32, parse_f32, visit_f32);
    implement_number!(deserialize_f64, parse_f64, visit_f64);

    serde_if_integer128! {
        implement_number!(deserialize_i128, parse_i128, visit_i128);
        implement_number!(deserialize_u128, parse_u128, visit_u128);
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
        let c = match_tag! {
            self.pop_tag()?,
            Tag::Char1 => inner::<1, R>(self),
            Tag::Char2 => inner::<2, R>(self),
            Tag::Char3 => inner::<3, R>(self),
            Tag::Char4 => inner::<4, R>(self)
        }?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, R::Error>
    where
        V: de::Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?,
            Tag::String => {
                let len = self.pop_len()?;
                let str = self.pop_str(len)?;
                #[cfg(feature = "alloc")]
                match str {
                    alloc::borrow::Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
                    alloc::borrow::Cow::Owned(string) => visitor.visit_string(string)
                }
                #[cfg(not(feature = "alloc"))]
                visitor.visit_borrowed_str(str)

            },
            Tag::MarkerTerminatedString => {
                let str = self.pop_unsized_str()?;
                #[cfg(feature = "alloc")]
                match str {
                    alloc::borrow::Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
                    alloc::borrow::Cow::Owned(string) => visitor.visit_string(string)
                }
                #[cfg(not(feature = "alloc"))]
                visitor.visit_borrowed_str(str)
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
        match_tag! {
            self.pop_tag()?,
            Tag::Bytes => {
                let len = self.pop_len()?;
                let bytes = self.reader.read_bytes(len)?;
                #[cfg(feature = "alloc")]
                match bytes {
                    alloc::borrow::Cow::Borrowed(bytes) => visitor.visit_borrowed_bytes(bytes),
                    alloc::borrow::Cow::Owned(bytes) => visitor.visit_byte_buf(bytes)
                }
                #[cfg(not(feature = "alloc"))]
                visitor.visit_borrowed_bytes(bytes)
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
        match_tag! {
            self.pop_tag()?,
            Tag::None => visitor.visit_none(),
            Tag::Some => visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, R::Error>
    where
        V: de::Visitor<'de>,
    {
        match_tag! {
            self.pop_tag()?,
            Tag::Unit => visitor.visit_unit()
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
        match_tag! {
            self.pop_tag()?,
            Tag::UnitStruct => visitor.visit_unit()
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
        match_tag! {
            self.pop_tag()?,
            Tag::NewTypeStruct => visitor.visit_newtype_struct(self)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, R::Error>
    where
        V: de::Visitor<'de>,
    {
        let len = match_tag! {
            self.pop_tag()?,
            Tag::Seq | Tag::Tuple | Tag::TupleStruct => {
                self.pop_len().map(Some)
            },
            Tag::UnsizedSeq => {
                Ok(None)
            }
        }?;

        visitor.visit_seq(SeqDeserializer::new(self, len))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, R::Error>
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
        let len = match_tag! {
            self.pop_tag()?,
            Tag::Map | Tag::Struct => {
                self.pop_len().map(Some)
            },
            Tag::UnsizedMap => {
                Ok(None)
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
        match_tag! {
            tag,
            Tag::UnitVariant | Tag::NewTypeVariant | Tag::TupleVariant | Tag::StructVariant => {
                let variant_index = self.pop_variant()?;
                let value = visitor.visit_u32::<Self::Error>(variant_index)?;
                // carry tag to check de::VariantAccess impl
                self.peeked_tag = Some(tag);
                Ok(value)
            },
            Tag::String => {
                let len = self.pop_len()?;
                let str = self.pop_str(len)?;
                #[cfg(feature = "alloc")]
                match str {
                    alloc::borrow::Cow::Borrowed(str) => visitor.visit_borrowed_str(str),
                    alloc::borrow::Cow::Owned(string) => visitor.visit_string(string)
                }
                #[cfg(not(feature = "alloc"))]
                visitor.visit_borrowed_str(str)
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
        SeqDeserializer { de, remaining: len }
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
        match_tag! { self.pop_tag()?, Tag::UnitVariant => Ok(())}
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, R::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match_tag! {
            self.pop_tag()?,
            Tag::NewTypeVariant => seed.deserialize(self)
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, R::Error>
    where
        V: Visitor<'de>,
    {
        use de::Deserializer;
        match_tag! {
            self.pop_tag()?,
            Tag::TupleVariant => {
                self.peeked_tag = Some(Tag::Tuple);
                self.deserialize_tuple(len, visitor)
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
        use de::Deserializer;
        match_tag! {
            self.pop_tag()?,
            Tag::StructVariant => {
                self.peeked_tag = Some(Tag::Struct);
                self.deserialize_struct("", fields, visitor)
            }
        }
    }
}
