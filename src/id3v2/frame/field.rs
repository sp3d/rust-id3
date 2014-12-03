#![allow(dead_code, unused_variables)]
use std::io::{self, Read, Write};
use id3v2::frame::Encoding;
use std::fmt;
use util;

/// The various types of primitive data which may be encoded in a field.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FieldType {
    TextEncoding,
    Latin1,
    Latin1Full,
    Latin1List,
    String,
    StringFull,
    StringList,
    Language,
    FrameIdV2,
    FrameIdV34,
    Date,
    Int8,
    Int16,
    Int24,
    Int32,
    Int32Plus,
    BinaryData,
}

impl FieldType {
    /// Get the encoding, if any, associated with a field. TODO: this may be a nonsense method.
    pub fn get_encoding(&self) -> Option<Encoding> {
        //TODO: this
        use self::FieldType::*;
        match *self {
            Latin1|Latin1Full|Latin1List => Some(Encoding::Latin1),
            String|StringFull|StringList => None,
            _ => None
        }
    }

    /// Indicates whether fields of this type contain a list of multiple pieces of data.
    pub fn is_list(&self) -> bool {
        use self::FieldType::*;
        *self == Latin1List || *self == StringList
    }
    //TODO: to/from char? names?

    /// Get a single character shorthand for this type of field. Fields which
    /// are lists are represented as the same character as the corresponding
    /// non-list field type. Capital letters indicate "full" strings which may
    /// contain newlines.
    pub fn as_char(&self) -> char {
        ['e', 'a', 'A', 'a', 's', 'S', 's', 'l', 'f', 't', '1', '2', '3', '4', 'c', 'd', ][*self as usize]
    }

    /// Get a short name which describes what this kind of field is.
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
        ][*self as usize]
    }
}

/// Describes how precise a date is. The ID3v2.3 spec describes a subset of the
/// ISO 8601 specification that may be truncated to year, month, day, hour,
/// minute, or second precision, as denoted by this enumeration.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DatePrecision {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second
}

/// A parsed timestamp in the subset of ISO 8601 specified by the ID3v2 spec:
/// "yyyy, yyyy-MM, yyyy-MM-dd, yyyy-MM-ddTHH, yyyy-MM-ddTHH:mm and
/// yyyy-MM-ddTHH:mm:ss"
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Timestamp {
    /// seconds since 0000-00-00T00:00:00
    seconds: u64,
    /// how precisely the time is specified
    precision: DatePrecision,
}

impl Timestamp {
    /// Parse a string of the format YYYYMMDD into a timestamp with `Day` precision.
    ///
    /// Returns `None` if MM or DD is out of bounds, or if parsing fails.
    fn parse_8char(s: &[u8]) -> Option<Timestamp> {
        //TODO(sp3d): implement
        loop {}
    }

    /*    /// Format the year, month, and day components of a timestamp into a string
    /// parsable by `parse_8char`.
    ///
    /// Returns `None` if the date cannot be represented in 8 chars (if the year is >9999).
        fn print_8char(&str) -> Option<[u8; 8]> {
        //TODO(sp3d): warn about precision loss?
        //TODO(sp3d): implement
        loop {}
    }*/
}

/// A variable-length integer, used to store, for example, playback counts.
#[derive(PartialEq, Clone)]
pub struct BigNum {
    data: Vec<u8>
}

impl BigNum {
    /// Create a new bignum with the given data as its backing store.
    pub fn new(data: Vec<u8>) -> BigNum {
        BigNum {data: data}
    }
    /// Increments the value stored in the bignum by 1.
    pub fn incr(&mut self)
    {
        //TODO(sp3d): implement
    }
}
impl ::std::str::FromStr for BigNum {
    type Err=();
    fn from_str(s: &str) -> Result<BigNum, ()>
    {
        Err(panic!("not implemented"))
    }
}
impl fmt::Debug for BigNum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
    {
        panic!("not implemented")
    }
}

/// A parsed ID3v2 field, which is the atomic component from which frames are
/// composed, and which stores one primitive or a list of homogeneous string primitives.
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum Field {
    TextEncoding(Encoding),
    Latin1(Vec<u8>),
    Latin1Full(Vec<u8>),
    Latin1List(Vec<Vec<u8>>),
    String(Vec<u8>),
    StringFull(Vec<u8>),
    StringList(Vec<Vec<u8>>),
    Language([u8; 3]),
    FrameIdV2([u8; 3]),
    FrameIdV34([u8; 4]),
    Date(Timestamp),
    Int8(u8),
    Int16(u8, u8),
    Int24(u8, u8, u8),
    Int32(u8, u8, u8, u8),
    Int32Plus(BigNum),
    BinaryData(Vec<u8>),
}

impl Field {
    /// Write the field to the given writer. If @unsync is true, any byte patterns
    /// of the form "%11111111 111xxxxx" are written as "%11111111 00000000 111xxxxx".
    pub fn serialize<W: Write>(&self, writer: &mut W, encoding: Option<Encoding>, is_last: bool, unsync: bool) -> io::Result<()> {
        use self::Field::*;
        //TODO(sp3d): support unsync!
        match *self
        {
            TextEncoding(ref enc) => try!(writer.write(&[*enc as u8])),
            Latin1(ref s)|Latin1Full(ref s) => {
                try!(writer.write(&*s));
                if !is_last {
                    try!(writer.write(util::delim(Encoding::Latin1)))
                }else{0}
            },
            Latin1List(ref strs) => try!(writer.write(&*strs[0])),//TODO(sp3d): this is wrong.
            String(ref s)|StringFull(ref s) => {
                try!(writer.write(&*s));
                if !is_last {
                    try!(writer.write(util::delim(encoding.expect("String fields' encoding must be specified for serialization"))))
                }else{0}
            },
            StringList(ref strs) => try!(writer.write(&*strs[0])),//TODO(sp3d): this is wrong.
            Language(ref lang) => try!(writer.write(&*lang)),
            FrameIdV2(ref id) => try!(writer.write(&*id)),
            FrameIdV34(ref id) => try!(writer.write(&*id)),
            Date(ref ts) => {
                panic!("timestamp -> 8char not implemented yet")
                //try!(writer.write(&*ts)),
            },
            Int8(b0) => try!(writer.write(&[b0])),
            Int16(b1, b0) => try!(writer.write(&[b1,b0])),
            Int24(b2, b1, b0) => try!(writer.write(&[b2,b1,b0])),
            Int32(b3, b2, b1, b0) => try!(writer.write(&[b3,b2,b1,b0])),
            Int32Plus(ref bignum) => try!(writer.write(&*bignum.data)),
            BinaryData(ref data) => try!(writer.write(&*data)),
        };
        Ok(())
    }

    /// Read a sequence of bytes until `delim_len` consecutive zero bytes are read
    /// or max_len bytes are read, whichever comes first. Reads but discards the
    /// sequence of zero bytes.
    ///
    /// If an I/O error is encountered, the buffer is returned as it stands.
    ///
    /// The value returned is a pair of (data, whether the delimiter was found).
    #[inline]
    fn read_until_delim<R: Read>(reader: &mut R, delim_len: Option<u8>, max_len: usize) -> (Vec<u8>, bool) {
        //TODO: is this slow? benchmark
        let unit_len = delim_len.unwrap_or(1);
        
        let mut buf = vec![];
        let mut consecutive: u8;

        //while Some(consecutive) != delim_len && buf.len() < max_len
        loop {
            consecutive = 0;
            for _ in 0..unit_len {
                let mut byte = [0u8];
                let _n_read = match {reader.read(&mut byte)} {
                    Ok(0) => break,
                    Ok(1) => {buf.push_all(&byte); 1},
                    Ok(n) => {panic!("read neither 0 nor 1 bytes into a 1-byte buffer!");},
                    Err(_) => return (buf, false),
                };
                if byte == [0u8] {
                    consecutive += 1;
                }
            }
            if Some(consecutive) == delim_len {
                for _ in 0..delim_len.unwrap() {
                    buf.pop();
                }
                break
            }
            if buf.len() >= max_len {
                //never found a delimiter
                break
            }
        }
        (buf, Some(consecutive) == delim_len)
    }

    /// Attempt to read a field of the given type. If the field is malformed,
    /// writes the bytes which could not be parsed to the given writer, if any.
    pub fn parse<R: Read, W: Write>(reader: &mut R, ftype: FieldType, encoding: Option<Encoding>, len: usize, is_last: bool, unparsable: Option<&mut W>) -> io::Result<Field> {
        use std::slice::bytes;
        use self::FieldType::*;

        let len_min: usize = match ftype {
            TextEncoding => 1,
            Latin1 => 0,
            Latin1Full => 0,
            Latin1List => 0,
            String => 0,
            StringFull => 0,
            StringList => 0,
            Language => 3,
            FrameIdV2 => 3,
            FrameIdV34 => 4,
            Date => 8,
            Int8 => 1,
            Int16 => 2,
            Int24 => 3,
            Int32 => 4,
            Int32Plus => 0,
            BinaryData => 0,
        };

        let delim_len = match ftype {
            Latin1|Latin1Full/*|Latin1List*/ => Some(1u8),
            String|StringFull/*|StringList*/ => Some(util::delim_len(encoding.expect("String fields' encoding must be specified for parsing")) as u8),
            Int32Plus => None,
            BinaryData => None,
            _ => None,
        };

        let mut fixed_buf=[0u8; 8];
        let grow_buf;
        let (buf, len_read, saw_delim) = if len_min > 0 {
            let buf = &mut fixed_buf[..len_min];
            let len_read=read_at_least!(reader, buf, len_min);
            println!("read {:?}/{:?}B: {:?}", len_read, len_min, buf);
            (&*buf, len_read, false)
        } else {
            let (grow_buf_new, saw_delim) = Field::read_until_delim(reader, delim_len, len);
            grow_buf = grow_buf_new;
            let len_read = grow_buf.len();
            (&*grow_buf, len_read, saw_delim)
        };

        //TODO(sp3d): revisit idea of unparsable writer... seems a little daft
        /*if len_read != len {
            if let Some(writer) = unparsable {
                writer.write(buf.slice_to(len_read));
            }
            return Err(io::Error {kind: io::ErrorKind::NoProgress, desc: "", detail: None})
        }*/

        //on non-final fields, delimiters are mandatory for delimited field types
        if !is_last {
            if delim_len.is_some() && !saw_delim {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("No delimiter encountered for stringlike ({:?}) field", ftype)))
            }
        }

        match ftype {
            TextEncoding => {
                match Encoding::from_u8(buf[0]) {
                    Some(encoding) => Ok(Field::TextEncoding(encoding)),
                    None => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid encoding specifier")),
                }
            },
            Latin1 => {
                //TODO(sp3d): reject newlines?
                Ok(Field::Latin1(buf.to_vec()))
            },
            Latin1Full => {
                Ok(Field::Latin1Full(buf.to_vec()))
            },
            Latin1List => {
                //TODO(sp3d): check encoding? reject newlines? is this right?
                Ok(Field::Latin1List(vec![buf.to_vec()]))
            },
            String => {
                //TODO(sp3d): reject newlines? check encoding?
                Ok(Field::String(buf.to_vec()))
            },
            StringFull => {
                //TODO(sp3d): check encoding?
                Ok(Field::StringFull(buf.to_vec()))
            },
            StringList => {
                //TODO(sp3d): check encoding? reject newlines? is this right?
                //buf.split(delim)
                Ok(Field::StringList(vec![buf.to_vec()]))
                /*let mut strings = vec![];
                let mut remaining = len - len_read;
                while remaining > 0 {
                    let read_vec = read_until_delim(reader, delim_len, remaining);
                    remaining -= read_vec.len();
                }*/
            },//panic!("how the heck do you encode a stringlist even tho"),
            Language => {
                let mut lang = [0u8; 3];
                bytes::copy_memory(buf, &mut lang);
                Ok(Field::Language(lang))
            },
            FrameIdV2 => {
                let mut id = [0u8; 3];
                bytes::copy_memory(buf, &mut id);
                Ok(Field::FrameIdV2(id))
            },
            FrameIdV34 => {
                let mut id = [0u8; 4];
                bytes::copy_memory(buf, &mut id);
                Ok(Field::FrameIdV34(id))
            },
            Date => {
                let mut date = [0u8; 8];
                bytes::copy_memory(buf, &mut date);
                Ok(Field::Date(Timestamp::parse_8char(&date).expect("Timestamp failed to parse!")))
            },
            Int8 => {
                Ok(Field::Int8(buf[0]))
            },
            Int16 => {
                Ok(Field::Int16(buf[0], buf[1]))
            },
            Int24 => {
                Ok(Field::Int24(buf[0], buf[1], buf[2]))
            },
            Int32 => {
                Ok(Field::Int32(buf[0], buf[1], buf[2], buf[3]))
            },
            Int32Plus => {
                Ok(Field::Int32Plus(BigNum::new(buf.to_vec())))
            },
            BinaryData =>  {
                Ok(Field::BinaryData(buf.to_vec()))
            },
        }
    }
    //let unused: Vec<u8> = buf.slice_from(len_read).to_vec();
}
