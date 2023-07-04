use core::fmt::Display;

pub const UNSIZED_STRING_END_MARKER: [u8; 2] = [0xD8, 0x00];

pub fn end_of_str(bytes: &[u8; 2]) -> bool {
    bytes == &UNSIZED_STRING_END_MARKER
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

impl Tag {
    pub fn encode_char(c: char, buff: &mut [u8]) -> (Self, &[u8]) {
        let bytes = c.encode_utf8(buff).as_bytes();
        let tag = match bytes.len() {
            1 => Tag::Char1,
            2 => Tag::Char2,
            3 => Tag::Char3,
            4 => Tag::Char4,
            _ => unreachable!(),
        };
        (tag, bytes)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TagParsingError {
    #[cfg(no_integer128)]
    Integer128,
    InvalidTag(u8),
    UnexpectedTag {
        expected: &'static str,
        got: Tag,
    },
}

impl TagParsingError {
    pub fn unexpected(expected: &'static str, got: Tag) -> Self {
        Self::UnexpectedTag { expected, got }
    }
}

impl Display for TagParsingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(no_integer128)]
            TagParsingError::Integer128 => {
                f.write_str("This platform doesn't support 128 bits integers.")
            }
            TagParsingError::InvalidTag(tag) => f.write_fmt(format_args!(
                "Invalid tag for data type: expected byte beetween 0 and 31 included, got {}",
                tag
            )),
            TagParsingError::UnexpectedTag { expected, got } => {
                f.write_fmt(format_args!("Expected {} but got {:?}", expected, got))
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
            #[cfg(no_integer128)]
            37 | 36 => Err(TagParsingError::Integer128),
            tag => Err(TagParsingError::InvalidTag(tag)),
        }
    }
}

impl From<Tag> for u8 {
    fn from(value: Tag) -> Self {
        value as u8
    }
}
