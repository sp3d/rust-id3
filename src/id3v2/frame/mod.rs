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
use id3v2::Error;

use std::io::{self, Read, Write};

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
#[derive(PartialEq, Copy, Clone)]
#[allow(missing_docs)]
pub enum Id {
    V2([u8; 3]),
    V3([u8; 4]),
    V4([u8; 4]),
}

impl Id {
    /// Returns the ID3v2 version to which an ID belongs
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
            Id::V2(ref id) => &*id,
            Id::V3(ref id) => &*id,
            Id::V4(ref id) => &*id,
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

impl fmt::Debug for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Id::V2(id) => id.fmt(fmt),
            Id::V3(id) => id.fmt(fmt),
            Id::V4(id) => id.fmt(fmt),
        }
    }
}

/// An ID3v2 frame, containing an ID specifying its purpose/format and a set of fields which constitute its content.
#[derive(Debug)]
pub struct Frame {
    /// The frame identifier, namespaced to the ID3v2.x version to which the frame belongs.
    pub id: Id,
    /// Flags governing serialization and the frame's relationship to the ID3v2 tag as a whole.
    flags: FrameFlags,
    /// The parsed content of the frame.
    pub fields: Vec<Field>,
    /// Group symbol indicating to which group this frame belongs, which should have an
    /// associated GRID frame in the tag specifying an owner URL. Values less than 0x80 are
    /// "reserved" per the ID3v2.3, and values outside the range 0x80-0xf0 are forbidden by
    /// ID3v2.4. Frames with the same group symbol should be processed (e.g., removed) as a unit.
    group_symbol: u8,
    /// Byte with similar semantics to the "group symbol", but for frame-level encryption and
    /// with owners specified in an ENCR frame.
    encryption_method: u8,
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
            group_symbol: 0,
            encryption_method: 0,
        }
    }

    /// Returns the size in bytes of this frame when serialized.
    pub fn size(&self) -> u32 {
        self.write_to(std::io::sink().by_ref()).unwrap()
    }

    /// Creates a new ID3v2 text frame with the specified version and identifier,
    /// using the provided string as the text frame's content. The string will
    /// be transcoded to the specified encoding for storage in the frame.
    ///
    /// Returns `None` if the given id does not specify a text frame, or if the
    /// specified encoding is not compatible with the version of the ID. Note
    /// that TXX/TXXX are not "regular" text frames and cannot be created with
    /// this function.
    pub fn new_text_frame(id: Id, s: &str, encoding: Encoding) -> Option<Frame> {
        if !id.is_text() || !id.version().encoding_compatible(encoding) {
            return None
        }
        let mut frame = Frame::new(id);
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
    /// Sets the encoding used by text data in this frame, and transcodes the
    /// contents of `String`, `StringFull`, and `StringList` fields from the old
    /// encoding to the new one. Returns `true` if successful.
    ///
    /// Returns `false` and does not modify the frame if the specified encoding
    /// is not compatible with the frame's version.
    ///
    /// Returns `true` and does nothing if the frame does not begin with a
    /// `TextEncoding` field.
    pub fn set_encoding(&mut self, encoding: Encoding) -> bool {
        if !self.version().encoding_compatible(encoding) {
            return false;
        }

        let old_encoding;
        if let Some(&mut Field::TextEncoding(ref mut enc)) = self.fields.get_mut(0) {
            old_encoding = *enc;
            *enc = encoding;
        } else {
            return false;
        }

        if old_encoding == encoding {
            return true;
        }

        //TODO(sp3d): transcode strings!
        for f in self.fields.iter_mut() {
            match f {
                &mut Field::String(ref mut _s) => {
                    
                },
                &mut Field::StringFull(ref mut _s) => {
                    
                },
                &mut Field::StringList(ref mut _s) => {
                    
                },
                _ => (),
            }
        }
        true
    }

    #[inline]
    /// Returns whether the frame was stored using zlib compression.
    pub fn compression(&self) -> bool {
        self.flags.compression
    }

    #[inline]
    /// Sets whether zlib compression will be used when storing the frame.
    pub fn set_compression(&mut self, compression: bool) {
        self.flags.compression = compression;
        if compression && self.version() >= Version::V4 {
            self.flags.data_length_indicator = true;
        }
    }

    #[inline]
    /// Returns the frame's "tag alter preservation" flag.
    ///
    /// This flag indicates whether parsers which do *not* recognize this frame
    /// should discard the frame upon modifying any aspect of its containing
    /// tag. This includes modifications to padding and frame order.
    pub fn tag_alter_preservation(&self) -> bool {
        self.flags.tag_alter_preservation
    }

    #[inline]
    /// Sets the frame's "tag alter preservation" flag.
    ///
    /// This flag indicates whether parsers which do *not* recognize this frame
    /// should discard the frame upon modifying any aspect of its containing
    /// tag. This includes modifications to padding and frame order.
    pub fn set_tag_alter_preservation(&mut self, tag_alter_preservation: bool) {
        self.flags.tag_alter_preservation = tag_alter_preservation;
    }

    #[inline]
    /// Returns the frame's "file alter preservation" flag.
    ///
    /// This flag indicates whether parsers which do *not* recognize this frame
    /// should discard the frame upon modifying part (but not replacing all) of
    /// the non-tag data in the file.
    pub fn file_alter_preservation(&self) -> bool {
        self.flags.file_alter_preservation
    }

    #[inline]
    /// Sets the frame's "file alter preservation" flag.
    ///
    /// This flag indicates whether parsers which do *not* recognize this frame
    /// should discard the frame upon modifying part (but not replacing all) of
    /// the non-tag data in the file.
    pub fn set_file_alter_preservation(&mut self, file_alter_preservation: bool) {
        self.flags.file_alter_preservation = file_alter_preservation;
    }

    #[inline]
    /// Returns the frame's "read only" flag.
    ///
    /// This flag indicates whether the frame is intended to be "read-only",
    /// for example if the validity of other frames' data is depends on the
    /// contents of the frame.
    pub fn read_only(&self) -> bool {
        self.flags.read_only
    }

    #[inline]
    /// Sets the frame's "read only" flag.
    ///
    /// This flag indicates whether the frame is intended to be "read-only",
    /// for example if the validity of other frames' data depends on the
    /// contents of the frame.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.flags.read_only = read_only;
    }

    /// Returns the version of the tag which this frame belongs to.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2::Version;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let frame = Frame::new(Id::V4(*b"TALB"));
    /// assert_eq!(frame.version(), Version::V4)
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.id.version()
    }

    /// Converts the frame to a different version of ID3v2. This converts the
    /// frame identifier from its previous version to the corresponding frame
    /// identifier in the new version, filling in empty or zero data for new
    /// fields in the new frame layout, and removing fields which are not
    /// present in the new layout. Text fields will be changed to UTF-16 if they
    /// were previously encoded as UTF-8 or UTF-16be and the new version does
    /// not support their old encoding.
    ///
    /// Returns `true` if the conversion was successful. Returns `false` if the
    /// frame identifier could not be converted.
    ///
    /// Warning: not fully implemented yet! Calling this *will* result in
    /// mangled tags!
    //#[deprecated = "not fully implemented yet!"]
    pub fn convert_version(&mut self, to: Version) -> bool {
        use id3v2::Version::*;
        let from = self.id;

        // convert frame ID
        // no-op if versions are equal or "compatible" like V3/V4 are
        match (from, to) {
            (x, y) if x.version() == y => { return true },
            (Id::V3(_), V4) | (Id::V4(_), V3) => { return true },
            (Id::V3(id), V2) | (Id::V4(id), V2) => {
                // attempt to convert the id
                self.id = match frameinfo::convert_id_3_to_2(id) {
                    Some(new_id) => Id::V2(new_id),
                    None => {
                        debug!("no ID3v2.3/4 to ID3v2.2 mapping for {:?}", self.id);
                        return false;
                    }
                }
            },
            (Id::V2(id), V3)|(Id::V2(id), V4) => {
                // attempt to convert the id
                self.id = match frameinfo::convert_id_2_to_3(id) {
                    Some(new_id) => match to {V3 => Id::V3(new_id), V4 => Id::V4(new_id), _ => unreachable!() },
                    None => {
                        debug!("no ID3v2.2 to ID3v2.3/4 mapping for {:?}", self.id);
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

        //TODO(sp3d): convert frame format itself, adding/dropping fields!

        // convert text fields to an encoding compatible with the new version
        match (self.id.version(), to) {
            // ID3v2.3 and ID3v2.2 do not support UTF-16BE or UTF-8 encodings
            (V4, V3) | (V4, V2) => {
                match self.encoding() {
                    Some(Encoding::UTF16BE) | Some(Encoding::UTF8) => {
                        self.set_encoding(Encoding::UTF16);
                    },
                    _ => (),
                }
            }
            // encodings are forward-compatible and between ID3v2.2 and ID3v2.3
            _ => (),
        }
        
        true
    }

    /// Attempts to read a frame from the reader.
    ///
    /// Returns a tuple containing the number of bytes read and a frame. If padding
    /// is encountered then `None` is returned.

    #[inline]
    pub fn read_from(reader: &mut Read, version: Version) -> Result<Option<(u32, Frame)>, Error> {
        match version {
            Version::V2 => FrameStream::read(reader, None::<FrameV2>),
            Version::V3 => FrameStream::read(reader, None::<FrameV3>),
            Version::V4 => FrameStream::read(reader, None::<FrameV4>),
        }
    }

    /// Attempts to write the frame to the writer.
    #[inline]
    pub fn write_to(&self, writer: &mut Write) -> Result<u32, io::Error> {
        match self.version() {
            Version::V2 => FrameStream::write(writer, self, None::<FrameV2>),
            Version::V3 => FrameStream::write(writer, self, None::<FrameV3>),
            Version::V4 => FrameStream::write(writer, self, None::<FrameV4>),
        }
    }

    /// Creates a vector representation of the fields of a frame suitable for writing to an ID3 tag.
    #[inline]
    pub fn fields_to_bytes(&self) -> Vec<u8> {
        let request = EncoderRequest { version: self.version(), fields: &*self.fields };
        parsers::encode(request)
    }

    // Parsing {{{
    /// Parses the provided data into the field storage for the frame. If the compression
    /// flag is set to true then decompression will be performed.
    ///
    /// Returns `Err` if the data is invalid for the frame type.
    pub fn parse_fields(&self, data: &[u8]) -> Result<Vec<Field>, Error> {
        let decompressed_opt = if self.flags.compression {
            Some(flate::inflate_bytes_zlib(data).unwrap())
        } else {
            None
        };

        let result = try!(parsers::decode(DecoderRequest {
            id: self.id,
            data: match decompressed_opt {
                Some(ref decompressed) => &*decompressed,
                None => data
            }
        }));

        Ok(result.fields)
    }

    /// Serializes and reparses the frame's fields; should be a nop.
    #[inline]
    pub fn reparse(&mut self) {
        let data = self.fields_to_bytes();
        self.fields = self.parse_fields(&*data).unwrap();
    }
    // }}}

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
    use util;

    #[test]
    fn test_frame_flags_to_bytes_v3() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.to_bytes(0x3), [0x0, 0x0]);
        flags.tag_alter_preservation = true;
        flags.file_alter_preservation = true;
        flags.read_only = true;
        flags.compression = true;
        flags.encryption = true;
        flags.grouping_identity = true;
        assert_eq!(flags.to_bytes(0x3), [0xE0, 0xE0]);
    }

    #[test]
    fn test_frame_flags_to_bytes_v4() {
        let mut flags = FrameFlags::new();
        assert_eq!(flags.to_bytes(0x4), [0x0, 0x0]);
        flags.tag_alter_preservation = true;
        flags.file_alter_preservation = true;
        flags.read_only = true;
        flags.grouping_identity = true;
        flags.compression = true;
        flags.encryption = true;
        flags.unsynchronization = true;
        flags.data_length_indicator = true;
        assert_eq!(flags.to_bytes(0x4), [0x70, 0x4F]);
    }

    #[test]
    fn test_to_bytes_v2() {
        let id = *b"TAL";
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V2(id));

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(&*data).unwrap();
        println!("{:?}", frame.fields);

        let mut bytes = Vec::new();
        bytes.push_all(&id);
        bytes.push_all(&util::u32_to_bytes(data.len() as u32)[1..]);
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }

    #[test]
    fn test_to_bytes_v3() {
        let id = *b"TALB";
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V3(id));

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(&*data).unwrap();
        println!("{:?}", frame.fields);

        let mut bytes = Vec::new();
        bytes.push_all(&id);
        bytes.push_all(&util::u32_to_bytes(data.len() as u32));
        bytes.push_all(&[0x00, 0x00]);
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }

    #[test]
    fn test_to_bytes_v4() {
        let id = *b"TALB";
        let text = "album";
        let encoding = Encoding::UTF16;

        let mut frame = Frame::new(Id::V4(id));

        frame.flags.tag_alter_preservation = true;
        frame.flags.file_alter_preservation = true;

        let mut data = Vec::new();
        data.push(encoding as u8);
        data.extend(util::string_to_utf16(text).into_iter());

        frame.fields = frame.parse_fields(&*data).unwrap();

        let mut bytes = Vec::new();
        bytes.push_all(&id);
        bytes.push_all(&util::u32_to_bytes(util::synchsafe(data.len() as u32)));
        bytes.push_all(&[0x60, 0x00]);
        bytes.extend(data.into_iter());

        let mut writer = Vec::new();
        frame.write_to(&mut writer).unwrap();
        assert_eq!(writer, bytes);
    }
}
