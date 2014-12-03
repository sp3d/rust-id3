use audiotag::{TagError, TagResult};
use audiotag::ErrorKind::{InvalidInputError, /*StringDecodingError, UnsupportedFeatureError*/};

use id3v2::frame::field::Field;
use id3v2::frame::{self, Frame, Id, Encoding};
use id3v2::Version;

pub struct DecoderRequest<'a> {
    pub id: Id,
    pub data: &'a [u8],
}

pub struct EncoderRequest<'a> {
    pub version: Version,
    pub fields: &'a [Field],
}

impl<'a> EncoderRequest<'a> {
    fn encoding(&self) -> Option<Encoding> {
        if let Some(&Field::TextEncoding(encoding)) = self.fields.get(0) {
            Some(encoding)
        } else {
            None
        }
    }
}

/// Creates a vector representation of the request.
pub fn encode(request: EncoderRequest) -> Vec<u8> {
    let mut encoded = vec![];
    let last = match request.fields.last() {
        Some(x) => x as *const _,
        None => 0 as *const _,
    };
    for i in request.fields.iter() {
        //Field::serialize only fails if the writer fails to write, and a vec won't, so we can drop()
        drop(i.serialize(&mut encoded, request.encoding(), i as *const _ == last, false/*unsync*/));
    }
    encoded
}

/// Attempts to decode the request.
pub fn decode(mut request: DecoderRequest) -> TagResult<Frame> {
    let mut encoding = None;//request.encoding;
    let mut fields = vec![];
    let field_types = match frame::frame_format(request.id) {
        Some(ft) => ft,
        None => {return Err(TagError::new(InvalidInputError, "No format could be chosen for the frame ID"))},
    };
    let last = match field_types.last() {
        Some(x) => x as *const _,
        None => 0 as *const _,
    };
    for ftype in field_types.iter() {
        let out: Option<&mut Vec<u8>> = None;
        match Field::parse(&mut request.data, *ftype, encoding, request.data.len(), ftype as *const _ == last, out) {
            Ok(field) => {
                //if no encoding was specified in the request, try to pick up one from a preceding field
                if encoding.is_none() {
                    if let &Field::TextEncoding(decoded_enc) = &field  {
                        encoding = Some(decoded_enc);
                    }
                }
                fields.push(field)
            },
            Err(what) => {println!("{}", what); return Err(::std::convert::From::from(what))},
        }
    }
    let mut frame = Frame::new(request.id);
    frame.fields = fields;
    Ok(frame)
}

// Tests {{{
#[cfg(test)]
mod tests {
    use parsers;
    use parsers::{DecoderRequest, EncoderRequest};
    use util;
    use id3v2::Version;
    use id3v2::frame::Id::*;
    use id3v2::frame::field::Field;
    use id3v2::frame::{PictureType, Encoding};

    fn bytes_for_encoding(text: &str, encoding: Encoding) -> Vec<u8> {
        match encoding {
            Encoding::Latin1 | Encoding::UTF8 => text.as_bytes().to_vec(),
            Encoding::UTF16 => util::string_to_utf16(text),
            Encoding::UTF16BE => util::string_to_utf16be(text)
        }
    }

    fn delim_for_encoding(encoding: Encoding) -> Vec<u8> {
        match encoding {
            Encoding::Latin1 | Encoding::UTF8 => vec![0u8; 1],
            Encoding::UTF16 | Encoding::UTF16BE => vec![0u8; 2],
        }
    }

    #[test]
    fn test_apic_v2() {
        assert!(parsers::decode(DecoderRequest { id: V2(*b"PIC"), data: &[] } ).is_err());

        let format_map = &[
            ("image/jpeg", b"JPG"),
            ("image/png", b"PNG"),
        ];

        for &(mime_type, format) in format_map {
            for description in &["", "description"] {
                let picture_type = PictureType::CoverFront;
                let picture_data = vec!(0xF9, 0x90, 0x3A, 0x02, 0xBD);

                for &encoding in &[Encoding::Latin1, Encoding::UTF16] {
                    println!("`{}`, `{}`, `{:?}`", mime_type, description, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(format);
                    data.push(picture_type as u8);
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.push_all(&*picture_data);

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Int24(format[0], format[1], format[2]),
                        Field::Int8(picture_type as u8),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::BinaryData(picture_data.clone()),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V2(*b"PIC"),
                        data: &*data,
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        fields: &*fields,
                        version: Version::V2,
                    }), data);
                }
            }
        }
    }

    #[test]
    fn test_apic_v3() {
        assert!(parsers::decode(DecoderRequest { id: V3(*b"APIC"), data: &[] } ).is_err());

        for mime_type in &["", "image/jpeg"] {
            for description in &["", "description"] {
                let picture_type = PictureType::CoverFront;
                let picture_data = vec!(0xF9, 0x90, 0x3A, 0x02, 0xBD);

                for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                    println!("`{}`, `{}`, `{:?}`", mime_type, description, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(mime_type.as_bytes());
                    data.push(0x0);
                    data.push(picture_type as u8);
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.push_all(&*picture_data);

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Latin1(mime_type.as_bytes().to_vec()),
                        Field::Int8(picture_type as u8),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::BinaryData(picture_data.clone()),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V3(*b"APIC"),
                        data: &*data,
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        fields: &*fields,
                        version: Version::V3 }), data);
                }
            }
        }
    }

    #[test]
    fn test_comm() {
        assert!(parsers::decode(DecoderRequest { id: V4(*b"COMM"), data: &[] } ).is_err());

        println!("valid");
        for description in &["", "description"] {
            for comment in &["", "comment"] {
                for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                    println!("`{}`, `{}`, `{:?}`", description, comment, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(b"ENG");
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.extend(bytes_for_encoding(comment, encoding).into_iter());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Language(*b"ENG"),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::StringFull(bytes_for_encoding(comment, encoding))
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(*b"COMM"),
                        data: &*data,
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        fields: &*fields, version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "description";
        let comment = "comment";
        for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
            println!("`{:?}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.push_all(b"ENG");
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.extend(bytes_for_encoding(comment, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(*b"COMM"),
                data: &*data
            }).is_err());
        }

    }

    #[test]
    fn test_text() {
        assert!(parsers::decode(DecoderRequest { id: V4(*b"TALB"), data: &[] } ).is_err());

        for text in &["", "text"] {
            for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                println!("`{}`, `{:?}`", text, encoding);
                let mut data = Vec::new();
                data.push(encoding as u8);
                data.extend(bytes_for_encoding(text, encoding).into_iter());

                let fields = vec![
                    Field::TextEncoding(encoding),
                    Field::StringList(vec![bytes_for_encoding(text, encoding)]),
                ];

                assert_eq!(parsers::decode(DecoderRequest {
                    id: V4(*b"TALB"),
                    data: &*data
                }).unwrap().fields, fields);
                assert_eq!(parsers::encode(EncoderRequest {
                    fields: &*fields,
                    version: Version::V3
                } ), data);
            }
        }
    }

    #[test]
    fn test_txxx() {
        assert!(parsers::decode(DecoderRequest { id: V4(*b"TXXX"), data: &[] } ).is_err());

        println!("valid");
        for key in &["", "key"] {
            for value in &["", "value"] {
                for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                    println!("{:?}", encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.extend(bytes_for_encoding(key, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.extend(bytes_for_encoding(value, encoding).into_iter());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::String(bytes_for_encoding(key, encoding)),
                        Field::String(bytes_for_encoding(value, encoding)),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(*b"TXXX"),
                        data: &*data,
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        fields: &*fields,
                        version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let key = "key";
        let value = "value";
        for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
            println!("`{:?}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.extend(bytes_for_encoding(key, encoding).into_iter());
            data.extend(bytes_for_encoding(value, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(*b"TXXX"),
                data: &*data,
            }).is_err());
        }
    }

    #[test]
    fn test_weblink() {
        for link in &["", "http://www.rust-lang.org/"] {
            println!("`{}`", link);
            let data = link.as_bytes().to_vec();

            let fields = vec![
                Field::Latin1(link.as_bytes().to_vec()),
            ];

            assert_eq!(parsers::decode(DecoderRequest {
                id: V4(*b"WOAF"),
                data: &*data,
            }).unwrap().fields, fields);
            assert_eq!(parsers::encode(EncoderRequest {
                fields: &*fields,
                version: Version::V3
            }), data);
        }
    }

    #[test]
    fn test_wxxx() {
        assert!(parsers::decode(DecoderRequest { id: V4(*b"WXXX"), data: &[] } ).is_err());

        println!("valid");
        for description in &["", "rust"] {
            for link in &["", "http://www.rust-lang.org/"] {
                for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                    println!("`{}`, `{}`, `{:?}`", description, link, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.push_all(link.as_bytes());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::Latin1(link.as_bytes().to_vec()),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(*b"WXXX"),
                        data: &*data
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        fields: &*fields,
                        version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "rust";
        let link = "http://www.rust-lang.org/";
        for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
            println!("`{:?}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.push_all(link.as_bytes());
            assert!(parsers::decode(DecoderRequest {
                id: V4(*b"WXXX"),
                data: &*data,
            }).is_err());
        }
    }

    #[test]
    fn test_uslt() {
        assert!(parsers::decode(DecoderRequest { id: V4(*b"USLT"), data: &[] } ).is_err());

        println!("valid");
        for description in &["", "description"] {
            for text in &["", "lyrics"] {
                for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
                    println!("`{}`, `{}, `{:?}`", description, text, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(b"ENG");
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.extend(bytes_for_encoding(text, encoding).into_iter());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Language(*b"ENG"),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::StringFull(bytes_for_encoding(text, encoding)),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(*b"USLT"),
                        data: &*data,
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        version: Version::V3,
                        fields: &*fields,
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "description";
        let lyrics = "lyrics";
        for &encoding in &[Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE] {
            println!("`{:?}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.push_all(b"eng");
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.extend(bytes_for_encoding(lyrics, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(*b"USLT"),
                data: &*data,
            }).is_err());
        }
    }
}
// }}}
