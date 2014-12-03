use audiotag::{TagError, TagResult};
use audiotag::ErrorKind::{InvalidInputError, StringDecodingError, UnsupportedFeatureError};

use std::io::IoResult;
use frame::Encoding;
use frame::Content::*;
use util;
use std::fmt;

#[deriving(Show, PartialEq)]
pub enum FieldType {
    TextEncoding,
    Latin1,
    Latin1Full,
    Latin1List,
    String,
    StringFull,
    StringList,
    Language,
    FrameId,
    Date,
    Int8,
    Int16,
    Int24,
    Int32,
    Int32Plus,
    BinaryData,
}

impl FieldType {
    pub fn get_encoding(&self) -> Option<Encoding> {
        //TODO: this
        use self::FieldType::*;
        match *self {
            Latin1|Latin1Full|Latin1List => Some(Encoding::Latin1),
            String|StringFull|StringList => None,
            _ => None
        }
    }

    pub fn is_list(&self) -> bool {
        use self::FieldType::*;
        *self == Latin1List || *self == StringList
    }
    //TODO: to/from char? names?
    
    pub fn as_char(&self) -> char {
        ['e', 'a', 'A', 'a', 's', 'S', 's', 'l', 'f', 't', '1', '2', '3', '4', 'c', 'd', ][*self as uint]
    }
    pub fn name(&self) -> &'static str { [
        "textencoding",
        "latin1 string",
        "latin1 string with newlines",
        "latin1 strings",
        "encoded string",
        "encoded string with newlines",
        "encoded strings",
        "language code",
        "frame ID",
        "time/date",
        "byte",
        "int16",
        "int24",
        "int32",
        "counter",
        "data",
        ][*self as uint]
    }
}

/// Describes how precise a date is. The ID3v2.3 spec describes a subset of the
/// ISO 8601 specification that may be truncated to year, month, day, hour,
/// minute, or second precision, as denoted by this enumeration.
pub enum DatePrecision {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second
}

/// A parsed date in the subset of ISO 8601 specified by the ID3v2 spec:
/// "yyyy, yyyy-MM, yyyy-MM-dd, yyyy-MM-ddTHH, yyyy-MM-ddTHH:mm and
/// yyyy-MM-ddTHH:mm:ss"
pub struct Date {
    /// seconds since 0000-00-00T00:00:00
    seconds: u64,
    /// how precisely the time is specified
    precision: DatePrecision,
}

/// A variable-length integer, used to store, for example, playback counts
pub struct BigNum {
    data: Vec<u8>
}

//TODO: this
impl BigNum {
    fn incr(&mut self)
    {
    }
}
impl ::std::str::FromStr for BigNum {
    fn from_str(s: &str) -> Option<BigNum>
    {
        None
    }
}
impl fmt::Show for BigNum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        loop {}
    }
}

/// A parsed ID3v2 field, which is the atomic component from which frames are
/// composed, and which stores one type of primitive or list of homogeneous primitives.
pub enum Field {
    TextEncoding(Encoding),
    Latin1(Vec<u8>),
    Latin1Full(Vec<u8>),
    Latin1List(Vec<Vec<u8>>),
    String(Vec<u8>),
    StringFull(Vec<u8>),
    StringList(Vec<Vec<u8>>),
    Language([u8, ..3]),
    FrameId([u8, ..4]),
    Date(Date),
    Int8(u8),
    Int16(u8, u8),
    Int24(u8, u8, u8),
    Int32(u8, u8, u8, u8),
    Int32Plus(BigNum),
    BinaryData(Vec<u8>),
}

impl Field {
    /// Writes the field to the given writer. If @unsync is true, any byte patterns
    /// of the form "%11111111 111xxxxx" are written as "%11111111 00000000 111xxxxx".
    fn serialize<W: Writer>(&self, writer: &mut W, unsync: bool) -> IoResult<()> {
        use self::Field::*;
        match *self
        {
            TextEncoding(ref enc) => (),
            Latin1(ref s) => (),
            Latin1Full(ref s) => (),
            Latin1List(ref strs) => (),
            String(ref s) => (),
            StringFull(ref s) => (),
            StringList(ref strs) => (),
            Language(ref lang) => (),
            FrameId(ref id) => (),
            Date(ref date) => (),
            Int8(b0) => (),
            Int16(b1, b0) => (),
            Int24(b2, b1, b0) => (),
            Int32(b3, b2, b1, b0) => (),
            Int32Plus(ref BigNum) => (),
            BinaryData(ref data) => (),
        };
        Ok(())
    }
    /// Attempts to read a field of the given type. If the field is malformed,
    /// writes the bytes which could not be parsed to the given writer, if any.
    fn read<R: Reader, W: Writer>(reader: &mut R, ftype: FieldType, size: Option<uint>, unparsable: Option<&mut W>) -> IoResult<Field> {
        use self::FieldType::*;
        match ftype
        {
            TextEncoding => (),
            Latin1 => (),
            Latin1Full => (),
            Latin1List => (),
            String => (),
            StringFull => (),
            StringList => (),
            Language => (),
            FrameId => (),
            Date => (),
            Int8 => (),
            Int16 => (),
            Int24 => (),
            Int32 => (),
            Int32Plus => (),
            BinaryData => (),
        };
        loop {}
    }
}
