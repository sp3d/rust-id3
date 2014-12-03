extern crate std;
extern crate flate;

pub use self::encoding::Encoding;
pub use self::picture::PictureType;
pub use self::flags::FrameFlags;
pub use self::field::Field;

pub use self::frameinfo::{frame_description, frame_format, convert_id_2_to_3,
convert_id_3_to_2};

use self::stream::{FrameStream, FrameV2, FrameV3, FrameV4};
use id3v2::Version;

use audiotag::TagResult;

use util;
use parsers;
use parsers::{DecoderRequest, EncoderRequest};

use std::fmt;

mod picture;
mod encoding;
mod flags;
mod stream;
mod frameinfo;
/// Atomic units which are composed to make up ID3v2 frames.
pub mod field;

/// The version of an ID3v2 tag to which a frame belongs, and the frame ID as
/// specified by that version of ID3v2.
#[deriving(PartialEq, Copy)]
#[allow(missing_docs)]
pub enum Id {
    V2([u8, ..3]),
    V3([u8, ..4]),
    V4([u8, ..4]),
}

impl Id {
    /// Returns the ID3v2 Version to which an ID belongs
    #[inline]
    pub fn version(&self) -> Version {
        match *self {
            Id::V2(_) => Version::V2,
            Id::V3(_) => Version::V3,
            Id::V4(_) => Version::V4,
        }
    }
    /// Returns the frame ID string stored in an ID. This should be considered a
    /// "last resort" for when the desired behavior is not implemented by this
    /// library; for most common functionality higher-level functions are available
    /// and preferred.
    #[inline]
    pub fn name(&self) -> &[u8] {
        match *self {
            Id::V2(ref id) => id.as_slice(),
            Id::V3(ref id) => id.as_slice(),
            Id::V4(ref id) => id.as_slice(),
        }
    }
    /// Returns whether this ID corresponds to a standard-layout text frame.
    /// Note that this category excludes the TXX/TXXX frames, which have
    /// different layout and semantics.
    #[inline]
    pub fn is_text(&self) -> bool {
        self.name()[0] == b'T' && self.name() != b"TXX" && self.name() != b"TXXX"
    }
    /// Returns whether this ID corresponds to a standard-layout URL frame.
    /// Note that this category excludes the WXX/WXXX frames, which have
    /// different layout and semantics.
    #[inline]
    pub fn is_url(&self) -> bool {
        self.name()[0] == b'W' && self.name() != b"WXX" && self.name() != b"WXXX"
    }
}

impl fmt::Show for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Id::V2(id) => id.fmt(fmt),
            Id::V3(id) => id.fmt(fmt),
            Id::V4(id) => id.fmt(fmt),
        }
    }
}

/// An ID3v2 frame, containing an ID specifying its purpose/format and a set of fields which constitute its content.
#[deriving(Show)]
pub struct Frame {
    /// The frame identifier, namespaced to the ID3v2.x version to which the frame belongs.
    pub id: Id,
    /// Flags governing serialization and the frame's relationship to the ID3v2 tag as a whole.
    flags: FrameFlags,
    /// The parsed content of the frame.
    pub fields: Vec<Field>,
    /// The offset of this frame in the file from which it was loaded.
    //TODO(sp3d): shouldn't this be offset in the tag, not file?
    pub offset: u32,
}

impl PartialEq for Frame {
    #[inline]
    fn eq(&self, other: &Frame) -> bool {
        self == other
    }

    #[inline]
    fn ne(&self, other: &Frame) -> bool {
        self != other
    }
}

impl Frame {
    /// Creates a new ID3v2 frame with the specified version and identifier.
    #[inline]
    pub fn new(id: Id) -> Frame {
        Frame {
            id: id,
            flags: FrameFlags::new(),
            fields: vec![],
            offset: 0
        }
    }

    /// Creates a new ID3v2 text frame with the specified version and identifier,
    /// using the provided string as the text frame's content and the default
    /// encoding for the version.
    ///
    /// Returns `None` if the given id does not specify a text frame. Note that
    /// TXX/TXXX are not "regular" text frames and cannot be created with this
    /// function.
    pub fn new_text_frame(id: Id, s: &str) -> Option<Frame> {
        if !id.is_text() {
            return None
        }
        let mut frame = Frame::new(id);
        let encoding = id.version().default_encoding();
        let encoded: Vec<u8> = util::encode_string(s, encoding);
        //TODO(sp3d): disallow newline characters?
        frame.fields = match id.version() {
            Version::V2 => vec![Field::TextEncoding(encoding), Field::String(encoded)],
            Version::V3 => vec![Field::TextEncoding(encoding), Field::String(encoded)],
            //TODO(sp3d): StringList for V4?
            Version::V4 => vec![Field::TextEncoding(encoding), Field::String(encoded)],
        };
        Some(frame)
    }

    /// Creates a new ID3v2 URL frame with the specified version and identifier,
    /// using the provided URL (encoded in Latin-1) as the frame's URL.
    ///
    /// Returns `None` if the given id does not specify a URL frame. Note that
    /// WXX/WXXX are not "regular" URL frames and cannot be created with this
    /// function.
    pub fn new_url_frame(id: Id, url: &[u8]) -> Option<Frame> {
        if !id.is_url() {
            return None
        }
        let mut frame = Frame::new(id);
        //TODO(sp3d): disallow newline characters? validate Latin-1?
        frame.fields = vec![Field::Latin1(url.to_vec())];
        Some(frame)
    }

    /// Returns an encoding compatible with the current version based on the requested encoding.
    #[inline]
    fn compatible_encoding(&self, requested_encoding: Encoding) -> Encoding {
        if self.version() < Version::V4 {
            match requested_encoding {
                Encoding::Latin1 => Encoding::Latin1,
                _ => Encoding::UTF16, // if UTF16BE or UTF8 is requested, just return UTF16
            }
        } else {
            requested_encoding
        }
    }

    // Getters/Setters
    #[inline]
    /// Returns the encoding used by text data in this frame, if any.
    pub fn encoding(&self) -> Option<Encoding> {
        if let Some(&Field::TextEncoding(encoding)) = self.fields.get(0) {
            Some(encoding)
        } else {
            None
        }
    }

    #[inline]
    /// Sets the encoding used by text data in this frame. If the encoding is
    /// not compatible with the frame version, another encoding will be chosen.
    pub fn set_encoding(&mut self, encoding: Encoding) {
        //TODO: this
        //self.encoding = self.compatible_encoding(encoding);
    }

    #[inline]
    /// Returns whether the compression flag is set.
    pub fn compression(&self) -> bool {
        self.flags.compression
    }

    #[inline]
    /// Sets the compression flag.
    pub fn set_compression(&mut self, compression: bool) {
        self.flags.compression = compression;
        if compression && self.version() >= Version::V4 {
            self.flags.data_length_indicator = true;
        }
    }

    #[inline]
    /// Returns whether the tag_alter_preservation flag is set.
    pub fn tag_alter_preservation(&self) -> bool {
        self.flags.tag_alter_preservation
    }

    #[inline]
    /// Sets the tag_alter_preservation flag.
    pub fn set_tag_alter_preservation(&mut self, tag_alter_preservation: bool) {
        self.flags.tag_alter_preservation = tag_alter_preservation;
    }

    #[inline]
    /// Returns whether the file_alter_preservation flag is set.
    pub fn file_alter_preservation(&self) -> bool {
        self.flags.file_alter_preservation
    }

    #[inline]
    /// Sets the file_alter_preservation flag.
    pub fn set_file_alter_preservation(&mut self, file_alter_preservation: bool) {
        self.flags.file_alter_preservation = file_alter_preservation;
    }

    /// Returns the version of the tag which this frame belongs to.
    ///
    /// # Example
    /// ```
    /// use id3::Frame;
    ///
    /// let frame = Frame::new("USLT".into_string(), 4);
    /// assert_eq!(frame.version(), 4)
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.id.version()
    }

    /// Sets the version of the tag. This converts the frame identifier from the previous version
    /// to the corresponding frame identifier in the new version.
    ///
    /// Returns `true` if the conversion was successful. Returns `false` if the frame identifier
    /// could not be converted.
    pub fn set_version(&mut self, to: Version) -> bool {
        use id3v2::Version::*;
        // no-op if versions are equal or "compatible" like V3/V4 are
        let from = self.id;
        match (from, to) {
            (x, y) if x.version() == y => { return true },
            (Id::V3(_), V4) | (Id::V4(_), V3) => { return true },
            (Id::V3(id), V2) | (Id::V4(id), V2) => {
                // attempt to convert the id
                self.id = match frameinfo::convert_id_3_to_2(id) {
                    Some(new_id) => Id::V2(new_id),
                    None => {
                        debug!("no ID3v2.3 to ID3v2.3 mapping for {}", self.id);
                        return false;
                    }
                }
            },
            (Id::V2(id), V3)|(Id::V2(id), V4) => {
                // attempt to convert the id
                self.id = match frameinfo::convert_id_2_to_3(id) {
                    Some(new_id) => (match to {V3 => Id::V3, V4 => Id::V4, _ => unreachable!() })(new_id),
                    None => {
                        debug!("no ID3v2.2 to ID3v2.3 mapping for {}", self.id);
                        return false;
                    }
                };
                // if the new version is v2.4 and the frame is compressed, we must enable the
                // data_length_indicator flag
                if to == V4 && self.flags.compression {
                    self.flags.data_length_indicator = true;
                }
            },
            _ => unreachable!(),
        }

        let encoding = self.compatible_encoding(self.encoding().unwrap_or(Encoding::UTF8));
        self.set_encoding(encoding);
        true
    }

    /// Attempts to read a frame from the reader.
    ///
    /// Returns a tuple containing the number of bytes read and a frame. If padding
    /// is encountered then `None` is returned.

    #[inline]
    pub fn read_from(reader: &mut Reader, version: Version) -> TagResult<Option<(u32, Frame)>> {
        match version {
            Version::V2 => FrameStream::read(reader, None::<FrameV2>),
            Version::V3 => FrameStream::read(reader, None::<FrameV3>),
            Version::V4 => FrameStream::read(reader, None::<FrameV4>),
        }
    }

    /// Attempts to write the frame to the writer.
    #[inline]
    pub fn write_to(&self, writer: &mut Writer) -> TagResult<u32> {
        match self.version() {
            Version::V2 => FrameStream::write(writer, self, None::<FrameV2>),
            Version::V3 => FrameStream::write(writer, self, None::<FrameV3>),
            Version::V4 => FrameStream::write(writer, self, None::<FrameV4>),
        }
    }

    /// Creates a vector representation of the fields of a frame suitable for writing to an ID3 tag.
    #[inline]
    pub fn fields_to_bytes(&self) -> Vec<u8> {
        let request = EncoderRequest { version: self.version(), encoding: self.encoding().unwrap_or(self.id.version().default_encoding()), fields: self.fields.as_slice() };
        parsers::encode(request)
    }

    // Parsing {{{
    /// Parses the provided data into the field storage for the frame. If the compression
    /// flag is set to true then decompression will be performed.
    ///
    /// Returns `Err` if the data is invalid for the frame type.
    pub fn parse_fields(&self, data: &[u8]) -> TagResult<Vec<Field>> {
        let decompressed_opt = if self.flags.compression {
            Some(flate::inflate_bytes_zlib(data).unwrap())
        } else {
            None
        };

        let result = try!(parsers::decode(DecoderRequest {
            id: self.id,
            encoding: self.encoding(),
            data: match decompressed_opt {
                Some(ref decompressed) => decompressed.as_slice(),
                None => data
            }
        }));

        Ok(result.fields)
    }

    /// Serializes and reparses the frame's fields; should be a nop.
    #[inline]
    pub fn reparse(&mut self) {
        let data = self.fields_to_bytes();
        self.fields = self.parse_fields(data.as_slice()).unwrap();
    }
    // }}}

    /// Returns a string representing the parsed content.
    #[deprecated = "This API is does not correspond to ID3v2 semantics and will be removed."]
    pub fn text(&self) -> Option<String> {
        None
    }

    /// Returns a string describing the frame type.
    #[inline]
    pub fn description(&self) -> &'static str {
        frameinfo::frame_description(self.id)
    }
}

// Tests {{{
#[cfg(test)]
mod tests {
    use id3v2::frame::{Id, Frame, FrameFlags, Encoding};
    use id3v2::Version;
    use util;

    #[test]
    fn test_frame_flags_to_bytes_v3() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.to_bytes(0x3), vec!(0x0, 0x0));
        flags.tag_alter_preservation = true;
        flags.file_alter_preservation = true;
        flags.read_only = true;
        flags.compression = true;
        flags.encryption = true;
        flags.grouping_identity = true;
        assert_eq!(flags.to_bytes(0x3), vec!(0xE0, 0xE0));
    }

    #[test]
    fn test_frame_flags_to_bytes_v4() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.to_bytes(0x4), vec!(0x0, 0x0));
        flags.tag_alter_preservation = true;
        flags.file_alter_preservation = true;
        flags.read_only = true;
        flags.grouping_identity = true;
        flags.compression = true;
        flags.encryption = true;
        flags.unsynchronization = true;
        flags.data_length_indicator = true;
        assert_eq!(flags.to_bytes(0x4), vec!(0x70, 0x4F));
    }

    #[test]
    fn test_to_bytes_v2() {
        let id = b!("TAL");
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V2(id));

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(data.as_slice()).unwrap();
        println!("{}", frame.fields);

        let mut bytes = Vec::new();
        bytes.push_all(id.as_slice());
        bytes.push_all(util::u32_to_bytes(data.len() as u32).slice_from(1));
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }

    #[test]
    fn test_to_bytes_v3() {
        let id = b!("TALB");
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V3(id));

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(data.as_slice()).unwrap();
        println!("{}", frame.fields);

        let mut bytes = Vec::new();
        bytes.push_all(id.as_slice());
        bytes.extend(util::u32_to_bytes(data.len() as u32).into_iter());
        bytes.push_all(&[0x00, 0x00]);
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }

    #[test]
    fn test_to_bytes_v4() {
        let id = b!("TALB");
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V4(id));

        frame.flags.tag_alter_preservation = true;
        frame.flags.file_alter_preservation = true;

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(data.as_slice()).unwrap();

        let mut bytes = Vec::new();
        bytes.push_all(id.as_slice());
        bytes.extend(util::u32_to_bytes(util::synchsafe(data.len() as u32)).into_iter());
        bytes.push_all(&[0x60, 0x00]);
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }
}
