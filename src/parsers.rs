use audiotag::{TagError, TagResult};
use audiotag::ErrorKind::{InvalidInputError, StringDecodingError, UnsupportedFeatureError};

use id3v2::frame::field::Field;
use id3v2::frame::{Frame, Id, Encoding, PictureType};
use super::id3v2::Version;
use util;

/// The result of a successfully parsed frame.
pub struct DecoderResult {
    /// The text encoding used in the frame.
    pub encoding: Encoding,
    /// The parsed content of the frame.
    pub fields: Vec<Field>
}

impl DecoderResult {
    /// Creates a new `DecoderResult` with the provided encoding and contents.
    #[inline]
    pub fn new(encoding: Encoding, fields: Vec<Field>) -> DecoderResult {
        DecoderResult { encoding: encoding, fields: fields }
    }
}

pub struct DecoderRequest<'a> {
    pub id: Id,
    pub encoding: Option<Encoding>,
    pub data: &'a [u8],
}

pub struct EncoderRequest<'a> {
    pub version: Version,
    pub encoding: Encoding,
    pub fields: &'a [Field],
}

/*impl<'a> DecoderRequest<'a> {
    fn encoding(&self) -> Option<Encoding> {
        if let Some(Field::TextEncoding(encoding)) = self.fields.get(0) {
            Some(encoding)
        } else {
            None
        }
    }
}*/

/// Creates a vector representation of the request.
pub fn encode(request: EncoderRequest) -> Vec<u8> {
    let mut encoded = vec![];
    let last = match request.fields.last() {
        Some(x) => x as *const _,
        None => 0u as *const _,
    };
    for i in request.fields.iter() {
        i.serialize(&mut encoded, Some(request.encoding), i as *const _ == last, false/*unsync*/);
    }
    encoded
}

/// Attempts to decode the request.
pub fn decode(mut request: DecoderRequest) -> TagResult<Frame> {
    let mut encoding = None;//request.encoding;
    let mut fields = vec![];
    let field_types = match util::frame_format(request.id) {
        Some(ft) => ft,
        None => {return Err(TagError::new(InvalidInputError, "No format could be chosen for the frame ID"))},
    };
    let last = match field_types.last() {
        Some(x) => x as *const _,
        None => 0u as *const _,
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
            Err(what) => {println!("{}", what); return Err(::std::error::FromError::from_error(what))},
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
            Encoding::Latin1 | Encoding::UTF8 => Vec::from_elem(1, 0),
            Encoding::UTF16 | Encoding::UTF16BE => Vec::from_elem(2, 0)
        }
    }

    #[test]
    fn test_apic_v2() {
        assert!(parsers::decode(DecoderRequest { id: V2(b!("PIC")), encoding: Some(Encoding::UTF16), data: &[] } ).is_err());

        let mut format_map = vec![
            ("image/jpeg", b"JPG"),
            ("image/png", b"PNG"),
        ];

        for (mime_type, format) in format_map.into_iter() {
            for description in vec!("", "description").into_iter() {
                let picture_type = PictureType::CoverFront;
                let picture_data = vec!(0xF9, 0x90, 0x3A, 0x02, 0xBD);

                for encoding in vec!(Encoding::Latin1, Encoding::UTF16).into_iter() {
                    println!("`{}`, `{}`, `{}`", mime_type, description, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(format);
                    data.push(picture_type as u8);
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.push_all(picture_data.as_slice());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Int24(format[0], format[1], format[2]),
                        Field::Int8(picture_type as u8),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::BinaryData(picture_data.clone()),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V2(b!("PIC")),
                        encoding: Some(Encoding::UTF16),
                        data: data.as_slice(),
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        encoding: encoding,
                        fields: fields.as_slice(),
                        version: Version::V2,
                    }), data);
                }
            }
        }
    }

    #[test]
    fn test_apic_v3() {
        assert!(parsers::decode(DecoderRequest { id: V3(b!("APIC")), encoding: Some(Encoding::Latin1), data: &[] } ).is_err());

        for mime_type in vec!("", "image/jpeg").into_iter() {
            for description in vec!("", "description").into_iter() {
                let picture_type = PictureType::CoverFront;
                let picture_data = vec!(0xF9, 0x90, 0x3A, 0x02, 0xBD);

                for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                    println!("`{}`, `{}`, `{}`", mime_type, description, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(mime_type.as_bytes());
                    data.push(0x0);
                    data.push(picture_type as u8);
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.push_all(picture_data.as_slice());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Latin1(mime_type.into_string().into_bytes()),
                        Field::Int8(picture_type as u8),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::BinaryData(picture_data.clone()),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V3(b!("APIC")),
                        encoding: Some(encoding),
                        data: data.as_slice(),
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        encoding: encoding,
                        fields: fields.as_slice(),
                        version: Version::V3 }), data);
                }
            }
        }
    }

    #[test]
    fn test_comm() {
        assert!(parsers::decode(DecoderRequest { id: V4(b!("COMM")), encoding: Some(Encoding::UTF8), data: &[] } ).is_err());

        println!("valid");
        for description in vec!("", "description").into_iter() {
            for comment in vec!("", "comment").into_iter() {
                for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                    println!("`{}`, `{}`, `{}`", description, comment, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(b"ENG");
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.extend(bytes_for_encoding(comment, encoding).into_iter());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Language(b!("ENG")),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::StringFull(bytes_for_encoding(comment, encoding))
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(b!("COMM")),
                        encoding: Some(encoding),
                        data: data.as_slice(),
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        encoding: encoding,
                        fields: fields.as_slice(), version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "description";
        let comment = "comment";
        for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
            println!("`{}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.push_all(b"ENG");
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.extend(bytes_for_encoding(comment, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(b!("COMM")),
                encoding: Some(encoding),
                data: data.as_slice()
            }).is_err());
        }

    }

    #[test]
    fn test_text() {
        assert!(parsers::decode(DecoderRequest { id: V4(b!("TALB")), encoding: Some(Encoding::UTF8), data: &[] } ).is_err());

        for text in vec!("", "text").into_iter() {
            for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                println!("`{}`, `{}`", text, encoding);
                let mut data = Vec::new();
                data.push(encoding as u8);
                data.extend(bytes_for_encoding(text, encoding).into_iter());

                let fields = vec![
                    Field::TextEncoding(encoding),
                    Field::StringList(vec![bytes_for_encoding(text, encoding)]),
                ];

                assert_eq!(parsers::decode(DecoderRequest {
                    encoding: Some(encoding),
                    id: V4(b!("TALB")),
                    data: data.as_slice()
                }).unwrap().fields.as_slice(), fields);
                assert_eq!(parsers::encode(EncoderRequest {
                    encoding: encoding,
                    fields: fields.as_slice(),
                    version: Version::V3
                } ), data);
            }
        }
    }

    #[test]
    fn test_txxx() {
        assert!(parsers::decode(DecoderRequest { id: V4(b!("TXXX")), encoding: Some(Encoding::UTF8), data: &[] } ).is_err());

        println!("valid");
        for key in vec!("", "key").into_iter() {
            for value in vec!("", "value").into_iter() {
                for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                    println!("{}", encoding);
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
                        id: V4(b!("TXXX")),
                        encoding: Some(encoding),
                        data: data.as_slice(),
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        encoding: encoding,
                        fields: fields.as_slice(),
                        version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let key = "key";
        let value = "value";
        for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
            println!("`{}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.extend(bytes_for_encoding(key, encoding).into_iter());
            data.extend(bytes_for_encoding(value, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(b!("TXXX")),
                encoding: Some(encoding),
                data: data.as_slice(),
            }).is_err());
        }
    }

    #[test]
    fn test_weblink() {
        for link in vec!("", "http://www.rust-lang.org/").into_iter() {
            println!("`{}`", link);
            let data = link.as_bytes().to_vec();

            let fields = vec![
                Field::Latin1(link.as_bytes().to_vec()),
            ];

            assert_eq!(parsers::decode(DecoderRequest {
                id: V4(b!("WOAF")),
                encoding: Some(Encoding::Latin1),
                data: data.as_slice(),
            }).unwrap().fields.as_slice(), fields);
            assert_eq!(parsers::encode(EncoderRequest {
                encoding: Encoding::Latin1,
                fields: fields.as_slice(),
                version: Version::V3
            }), data);
        }
    }

    #[test]
    fn test_wxxx() {
        assert!(parsers::decode(DecoderRequest { id: V4(b!("WXXX")), encoding: Some(Encoding::UTF8), data: &[] } ).is_err());

        println!("valid");
        for description in vec!("", "rust").into_iter() {
            for link in vec!("", "http://www.rust-lang.org/").into_iter() {
                for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                    println!("`{}`, `{}`, `{}`", description, link, encoding);
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
                        id: V4(b!("WXXX")),
                        encoding: Some(encoding),
                        data: data.as_slice()
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        encoding: encoding,
                        fields: fields.as_slice(),
                        version: Version::V3
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "rust";
        let link = "http://www.rust-lang.org/";
        for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
            println!("`{}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.push_all(link.as_bytes());
            assert!(parsers::decode(DecoderRequest {
                id: V4(b!("WXXX")),
                encoding: Some(encoding),
                data: data.as_slice(),
            }).is_err());
        }
    }

    #[test]
    fn test_uslt() {
        assert!(parsers::decode(DecoderRequest { id: V4(b!("USLT")), encoding: Some(Encoding::UTF8), data: &[] } ).is_err());

        println!("valid");
        for description in vec!("", "description").into_iter() {
            for text in vec!("", "lyrics").into_iter() {
                for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
                    println!("`{}`, `{}, `{}`", description, text, encoding);
                    let mut data = Vec::new();
                    data.push(encoding as u8);
                    data.push_all(b"ENG");
                    data.extend(bytes_for_encoding(description, encoding).into_iter());
                    data.extend(delim_for_encoding(encoding).into_iter());
                    data.extend(bytes_for_encoding(text, encoding).into_iter());

                    let fields = vec![
                        Field::TextEncoding(encoding),
                        Field::Language(b!("ENG")),
                        Field::String(bytes_for_encoding(description, encoding)),
                        Field::StringFull(bytes_for_encoding(text, encoding)),
                    ];

                    assert_eq!(parsers::decode(DecoderRequest {
                        id: V4(b!("USLT")),
                        encoding: Some(encoding),
                        data: data.as_slice(),
                    }).unwrap().fields, fields);
                    assert_eq!(parsers::encode(EncoderRequest {
                        version: Version::V3,
                        encoding: encoding,
                        fields: fields.as_slice(),
                    }), data);
                }
            }
        }

        println!("invalid");
        let description = "description";
        let lyrics = "lyrics";
        for encoding in vec!(Encoding::UTF8, Encoding::UTF16, Encoding::UTF16BE).into_iter() {
            println!("`{}`", encoding);
            let mut data = Vec::new();
            data.push(encoding as u8);
            data.push_all(b"eng");
            data.extend(bytes_for_encoding(description, encoding).into_iter());
            data.extend(bytes_for_encoding(lyrics, encoding).into_iter());
            assert!(parsers::decode(DecoderRequest {
                id: V4(b!("USLT")),
                encoding: Some(encoding),
                data: data.as_slice(),
            }).is_err());
        }
    }
}
// }}}
