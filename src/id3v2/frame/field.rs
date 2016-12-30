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
        ['e', 'a', 'A', 'a', 's', 'S', 's', 'l', 'f', '1', '2', '3', '4', 'c', 'd', ][*self as usize]
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
        "byte",
        "int16",
        "int24",
        "int32",
        "counter",
        "data",
        ][*self as usize]
    }
}

/// A variable-length integer used to store, for example, playback counts.
#[derive(PartialEq, Clone)]
pub struct BigNum {
    /// Two base-10 digits per limb; most significant limb at 'push' end of Vec.
    data: Vec<u8>
}

impl BigNum {
    /// Create a new bignum with the given data as its backing store.
    pub fn new(mut data: Vec<u8>) -> BigNum {
        BigNum::drop_leading_zeros(&mut data);
        BigNum {data: data}
    }
    /// Increments the value stored in the bignum by 1.
    pub fn incr(&mut self) {
        for i in &mut self.data {
            *i += 1;
            if *i == 100 {
                *i = 0;
            } else {
                return
            }
        }
        //carry at the end of the loop
        self.data.push(1);
    }
    //remove leading zero bytes
    fn drop_leading_zeros(data: &mut Vec<u8>) {
        loop {
            match data.pop() {
                None => {break},
                Some(0) => {},
                Some(n) => {data.push(n); break},
            }
        }
    }
}
impl ::std::str::FromStr for BigNum {
    type Err=();
    fn from_str(s: &str) -> Result<BigNum, ()> {
        let mut ones: Option<u8> = None;
        let mut n = BigNum::new(vec![]);
        for i in s.chars().rev() {
            match i.to_digit(10) {
                Some(d) => match ones {
                    Some(o) => {ones = None; n.data.push(o+10*d as u8)},
                    None => {ones = Some(d as u8)},
                },
                None => return Err(()),
            };
        }
        if let Some(o) = ones {
            n.data.push(o);
        }
        BigNum::drop_leading_zeros(&mut n.data);
        Ok(n)
    }
}
impl fmt::Display for BigNum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.data.len() == 0 {
            0.fmt(fmt)
        } else {
            let nn = self.data[self.data.len()-1];
            try!(nn.fmt(fmt));
            let mut iter = self.data.iter().rev();
            iter.next();//skip the first digit
            for i in iter {
                try!(write!(fmt, "{:02}", i));
            }
            Ok(())
        }
    }
}
impl fmt::Debug for BigNum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, fmt)
    }
}

#[test]
fn test_bignum_create() {
    assert_eq!(BigNum::new(vec![]), BigNum::new(vec![0]));
    assert_eq!(BigNum::new(vec![0, 0, 0, 0]), BigNum::new(vec![0]));
    assert_eq!(BigNum::new(vec![0]), BigNum::new(vec![0]));
}

#[test]
fn test_bignum_parse() {
    assert_eq!(BigNum::new(vec![0]), "".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![0]), "0".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![0]), "00".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![1]), "1".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![10]), "10".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![95]), "95".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![99]), "99".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![23, 1]), "123".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![67, 45, 23, 1]), "1234567".parse::<BigNum>().unwrap());
    assert_eq!(BigNum::new(vec![67, 45, 23]), "0234567".parse::<BigNum>().unwrap());
}

#[test]
fn test_bignum_print() {
    assert_eq!(BigNum::new(vec![0]).to_string(), "0");
    assert_eq!(BigNum::new(vec![99]).to_string(), "99");
    assert_eq!(BigNum::new(vec![04, 32]).to_string(), "3204");
    assert_eq!(BigNum::new(vec![00, 1]).to_string(), "100");
    assert_eq!(BigNum::new(vec![00, 10]).to_string(), "1000");
    assert_eq!(BigNum::new(vec![00, 00, 1]).to_string(), "10000");
    assert_eq!(BigNum::new(vec![00, 00, 1, 00]).to_string(), "10000");
}

#[test]
fn test_bignum_incr() {
    let mut a = BigNum::new(vec![0]);
    assert_eq!(a, BigNum::new(vec![0]));

    a.incr();
    assert_eq!(a, BigNum::new(vec![1]));

    a.incr();
    assert_eq!(a, BigNum::new(vec![2]));

    let mut b = BigNum::new(vec![99]);
    b.incr();
    assert_eq!(b, BigNum::new(vec![00, 1]));
}

#[test]
fn test_bignum_roundtrip() {
    let mut x = "0009954".parse::<BigNum>().unwrap();
    assert_eq!(x.to_string(), "9954");
    for i in 1..50 {
        x.incr();
        assert_eq!(x.to_string(), (9954+i).to_string());
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
    /// Can only fail due to errors originating in the writer itself, rather than 
    /// serialization.
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
                    Ok(1) => {buf.extend(&byte); 1},
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
                for (i, j) in &mut lang.iter_mut().zip(buf.iter())
				{
					*i = *j;
				}
                Ok(Field::Language(lang))
            },
            FrameIdV2 => {
                let mut id = [0u8; 3];
                for (i, j) in &mut id.iter_mut().zip(buf.iter())
				{
					*i = *j;
				}
                Ok(Field::FrameIdV2(id))
            },
            FrameIdV34 => {
                let mut id = [0u8; 4];
                for (i, j) in &mut id.iter_mut().zip(buf.iter())
				{
					*i = *j;
				}
                Ok(Field::FrameIdV34(id))
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
