use crate::error::{NoWriterError, SerError};
use crate::tag::{Tag, UNSIZED_STRING_END_MARKER};
use crate::utils::write::{BuffWriter, DummyWriter, EndOfBuff, Write};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::fmt;
use serde::ser::{Error, SerializeMap};
use serde::{ser, serde_if_integer128, Serialize};
#[cfg(feature = "std")]
use std::io;

pub type Result<T, We = NoWriterError> = core::result::Result<T, SerError<We>>;

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

    fn write_tag_then_bytes(&mut self, tag: Tag, bytes: &[u8]) -> Result<usize, W::Error> {
        let mut wb = self.write_byte(tag.into())?;
        wb += self.write_bytes(bytes)?;
        Ok(wb)
    }

    fn write_tag_then_len(&mut self, tag: Tag, len: usize) -> Result<usize, W::Error> {
        self.write_tag_then_serialize(tag, &len)
    }

    fn write_tag_then_seq(&mut self, tag: Tag, bytes: &[u8]) -> Result<usize, W::Error> {
        let mut wb = self.write_tag_then_len(tag, bytes.len())?;
        wb += self.write_bytes(bytes)?;
        Ok(wb)
    }

    fn write_tag_then_variant(&mut self, tag: Tag, variant_index: u32) -> Result<usize, W::Error> {
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

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub fn to_bytes<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = Vec::new();
    Serializer::to_writer(value, &mut output)?;
    Ok(output)
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

pub fn to_buff<'a, T: ?Sized>(value: &T, buff: &'a mut [u8]) -> Result<BuffWriter<'a>, EndOfBuff>
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

macro_rules! implement_number {
    ($fn_name:ident, $t:ident, $tag:expr) => {
        fn $fn_name(self, value: $t) -> Result<Self::Ok, W::Error> {
            self.write_tag_then_bytes($tag, &value.to_be_bytes())
        }
    };
    // for compactness the "compact-nums" feature allow number to be serialized in the smallest format they can fit in
    // for exemple a u64 with a value that can fit in a u16 will be serialized as a u16
    ($fn_name:ident, $t:ident, $tag:expr, $sub:ty, $forward_fn:ident) => {
        #[cfg(feature = "compact-nums")]
        fn $fn_name(self, value: $t) -> Result<Self::Ok, W::Error> {
            if let Ok(value) = <$sub>::try_from(value) {
                self.$forward_fn(value)
            } else {
                self.write_tag_then_bytes(Tag::U64, &value.to_be_bytes())
            }
        }

        #[cfg(not(feature = "compact-nums"))]
        implement_number! { $fn_name, $t, $tag }
    };
}

impl<'a, W: Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = usize;

    type Error = SerError<W::Error>;

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

    implement_number!(serialize_i8, i8, Tag::I8);
    implement_number!(serialize_i16, i16, Tag::I16, i8, serialize_i8);
    implement_number!(serialize_i32, i32, Tag::I32, i16, serialize_i16);
    implement_number!(serialize_i64, i64, Tag::I64, i32, serialize_i32);
    implement_number!(serialize_u8, u8, Tag::U8);
    implement_number!(serialize_u16, u16, Tag::U16, u8, serialize_u8);
    implement_number!(serialize_u32, u32, Tag::U32, u16, serialize_u16);
    implement_number!(serialize_u64, u64, Tag::U64, u32, serialize_u32);
    implement_number!(serialize_f32, f32, Tag::F32);
    implement_number!(serialize_f64, f64, Tag::F64);

    serde_if_integer128! {
        implement_number!(serialize_i128, i128, Tag::I128);
        implement_number!(serialize_u128, u128, Tag::U128);
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
        self.write_tag_then_seq(Tag::ByteArray, v)
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

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, W::Error> {
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
        let mut wb = self.write_tag_then_variant(Tag::NewTypeVariant, variant_index)?;
        wb += value.serialize(self)?;
        Ok(wb)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, W::Error> {
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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, W::Error> {
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
        // unknown str length marker
        let mut wb = self.write_tag(Tag::NullTerminatedString)?;
        let mut collector = StrCollector::new(&mut self.writer);
        if fmt::write(&mut collector, format_args!("{}", value)).is_err() {
            let err = match collector.error {
                Some(err) => SerError::WriterError(err),
                // what ? unreachable!() would be the right choice but I want panic free and I don't know if compiler can optimise that away
                // so custom it is
                None => SerError::custom("Something went really wrong."),
            };
            return Err(err);
        }
        wb += collector.written_bytes;
        // "null" terminated str
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
    pub fn new(serializer: &'a mut Serializer<W>, written_bytes: usize, known_size: bool) -> Self {
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
            self.written_bytes_count += self.serializer.write_tag(Tag::UnsizedSeqEnd)?;
        }
        Ok(self.written_bytes_count)
    }
}

impl<'a, W: Write> ser::SerializeSeq for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = SerError<W::Error>;

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

    type Error = SerError<W::Error>;

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

    type Error = SerError<W::Error>;

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

    type Error = SerError<W::Error>;

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

    type Error = SerError<W::Error>;

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

    type Error = SerError<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result<Self::Ok, W::Error> {
        self.finish()
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for SeqSerializer<'a, W> {
    type Ok = usize;

    type Error = SerError<W::Error>;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), W::Error>
    where
        T: Serialize,
    {
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
