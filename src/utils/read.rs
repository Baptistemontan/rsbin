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
