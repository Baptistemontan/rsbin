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

    /// The result Cow<[u8]> should end with the last 2 bytes passed to the `end_of_str` callback.
    #[cfg(feature = "alloc")]
    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<Cow<'de, [u8]>, Self::Error>;

    /// The result Cow<[u8]> should end with the last 2 bytes passed to the `end_of_str` callback.
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
        self.pop_slice(len + 2)
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

#[cfg(feature = "std")]
impl<'de, T: ?Sized> Read<'de> for T
where
    T: std::io::Read,
{
    type Error = std::io::Error;

    fn read_to_buff(&mut self, buff: &mut [u8]) -> Result<(), Self::Error> {
        self.read_exact(buff)
    }

    fn read_bytes(&mut self, len: usize) -> Result<Cow<'de, [u8]>, Self::Error> {
        let mut buff = vec![0; len];
        self.read_exact(&mut buff)?;
        Ok(Cow::Owned(buff))
    }

    fn read_bytes_until(
        &mut self,
        end_of_str: fn(&[u8; 2]) -> bool,
    ) -> Result<Cow<'de, [u8]>, Self::Error> {
        let mut intermidiate = [0; 2];
        self.read_exact(&mut intermidiate)?;
        let mut buff: Vec<u8> = intermidiate.into();
        while !end_of_str(&intermidiate) {
            intermidiate.swap(0, 1);
            self.read_exact(&mut intermidiate[1..])?;
            buff.push(intermidiate[1]);
        }
        Ok(buff.into())
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use crate::tag::{end_of_str, UNSIZED_STRING_END_MARKER};

    use super::*;

    #[test]
    fn test_read_until_io_read() {
        const STRING: &[u8] = b"test_string";
        let mut bytes = STRING.to_vec();
        bytes.extend_from_slice(&UNSIZED_STRING_END_MARKER);
        let mut bytes_ref: &[u8] = &bytes;
        let v = bytes_ref.read_bytes_until(end_of_str).unwrap();
        assert_eq!(v, bytes);
    }

    #[test]
    #[should_panic]
    fn test_read_until_io_read_eof() {
        const STRING: &[u8] = b"test_string";
        let mut bytes = STRING.to_vec();
        bytes.extend_from_slice(&UNSIZED_STRING_END_MARKER);
        let mut bytes_ref: &[u8] = &bytes;
        let v = bytes_ref.read_bytes_until(|_| false).unwrap();
        assert_eq!(v, bytes);
    }

    #[test]
    fn test_read_until_buff_reader() {
        const STRING: &[u8] = b"test_string";
        let mut bytes = STRING.to_vec();
        bytes.extend_from_slice(&UNSIZED_STRING_END_MARKER);
        let mut buff_reader = BuffReader::new(&bytes);
        let v = buff_reader.read_bytes_until(end_of_str).unwrap();
        assert_eq!(v, bytes);
    }

    #[test]
    #[should_panic]
    fn test_read_until_buff_reader_eof() {
        const STRING: &[u8] = b"test_string";
        let mut bytes = STRING.to_vec();
        bytes.extend_from_slice(&UNSIZED_STRING_END_MARKER);
        let mut buff_reader = BuffReader::new(&bytes);
        let v = buff_reader.read_bytes_until(|_| false).unwrap();
        assert_eq!(v, bytes);
    }
}
