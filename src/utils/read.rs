#[cfg(feature = "alloc")]
use alloc::borrow::Cow;

use crate::error::{EndOfBuff, RWError};

pub trait Read<'de> {
    type Error: RWError;

    fn read_to_buff(&mut self, buff: &mut [u8]) -> Result<(), Self::Error>;

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let mut byte = 0;
        self.read_to_buff(core::slice::from_mut(&mut byte))?;
        Ok(byte)
    }

    #[cfg(feature = "alloc")]
    fn read_bytes(&mut self, len: usize) -> Result<Cow<'de, [u8]>, Self::Error>;

    #[cfg(not(feature = "alloc"))]
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Self::Error>;

    #[cfg(feature = "alloc")]
    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<Cow<'de, [u8]>, Self::Error>;

    #[cfg(not(feature = "alloc"))]
    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<&'de [u8], Self::Error>;
}

pub struct BuffReader<'de> {
    buff: &'de [u8],
}

impl<'de> BuffReader<'de> {
    pub fn new(buff: &'de [u8]) -> Self {
        BuffReader { buff }
    }

    fn pop_slice(&mut self, len: usize) -> Result<&'de [u8], EndOfBuff> {
        if self.buff.len() < len {
            Err(EndOfBuff)
        } else {
            let (popped, rest) = self.buff.split_at(len);
            self.buff = rest;
            Ok(popped)
        }
    }

    fn read_until(&mut self, end_of_str: fn(&[u8; 2]) -> bool) -> Result<&'de [u8], EndOfBuff> {
        let len = self
            .buff
            .windows(2)
            .position(|bytes| end_of_str(bytes.try_into().unwrap()))
            .ok_or(EndOfBuff)?;
        self.pop_slice(len)
    }
}

impl<'de> Read<'de> for BuffReader<'de> {
    type Error = EndOfBuff;

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let (first, rest) = self.buff.split_first().ok_or(EndOfBuff)?;
        self.buff = rest;
        Ok(*first)
    }

    fn read_to_buff(&mut self, buff: &mut [u8]) -> Result<(), Self::Error> {
        let to_copy = self.pop_slice(buff.len())?;
        buff.copy_from_slice(to_copy);
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn read_bytes(&mut self, len: usize) -> Result<Cow<'de, [u8]>, Self::Error> {
        self.pop_slice(len).map(Cow::Borrowed)
    }

    #[cfg(not(feature = "alloc"))]
    fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Self::Error> {
        self.pop_slice(len)
    }

    #[cfg(feature = "alloc")]
    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<Cow<'de, [u8]>, Self::Error> {
        self.read_until(end_of_str).map(Cow::Borrowed)
    }

    #[cfg(not(feature = "alloc"))]
    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<&'de [u8], Self::Error> {
        self.read_until(end_of_str)
    }
}
