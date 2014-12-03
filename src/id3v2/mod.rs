use std::cmp::min;
use std::io::{SeekEnd, IoResult};
use audiotag::{TagError, TagResult};
use audiotag::ErrorKind::{InvalidInputError, UnsupportedFeatureError};
use self::frame::{Frame, Encoding, PictureType, Id};
use self::frame::field::Field;
use util;
use std::fmt;

/// Tools for working with ID3v2 frames.
pub mod frame;
/// High-level, lossy, and simple accessors for basic tag content.
pub mod simple;

/// An ID3v2 tag containing metadata frames. 
#[deriving(Show)]
pub struct Tag {
    /// The version of the tag. The first byte represents the major version number, while the
    /// second byte represents the revision number.
    pub version: Version,
    /// The ID3 header flags.
    pub flags: TagFlags,
    /// A vector of frames included in the tag.
    pub frames: Vec<Frame>,
    /// The size of the tag when read from a file.
    pub size: u32,
    /// The offset of the end of the last frame that was read.
    pub offset: u32,
    /// The offset of the first modified frame.
    pub modified_offset: u32,
}

/// Flags used in ID3v2 tag headers.
#[deriving(Show, Copy)]
pub enum TagFlag {
    /// Indicates whether or not unsynchronization is used. Valid in all ID3v2 tag versions.
    Unsynchronization,
    /// Indicates whether or not the header is followed by an extended header. Valid in ID3v2.3/4 tags.
    ExtendedHeader,
    /// Indicates whether the tag is in an experimental stage. Valid in ID3v2.3/4 tags.
    Experimental,
    /// Indicates whether a footer is present. Valid in ID3v2.4 tags.
    Footer,
    /// Indicates whether or not compression is used. This flag is only valid in ID3v2.2 tags.
    Compression,
}

impl TagFlag {
    #[inline]
    pub fn value(&self) -> u8 {
        [0x80, 0x40, 0x20, 0x10, 0x40][*self as uint]
    }
}

/// The flags set in an ID3v2 header.
#[deriving(Copy)]
pub struct TagFlags {
    byte: u8,
    version: Version,
}

impl fmt::Show for TagFlags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //TODO(sp3d): verify that the Ok case returns the right value
        use self::TagFlag::*;
        try!(fmt.write(b"{"));
        for i in [Unsynchronization, ExtendedHeader, Experimental, Footer, Compression].iter() {
            if self.get(*i) {
                try!(i.fmt(fmt))
                try!(fmt.write(b" "))
            }
        }
        fmt.write(b"}")
    }
}

// TagFlags {{{
impl TagFlags {
    /// Create a new `TagFlags` with all flags set to false.
    #[inline]
    pub fn new(version: Version) -> TagFlags {
        TagFlags {
            byte: 0u8,
            version: version,
        }
    }

    /// Create a new `TagFlags` using the provided byte.
    pub fn from_byte(byte: u8, version: Version) -> TagFlags {
        if match version {
            Version::V3|Version::V4 => byte & !0xF0 != 0,
            Version::V2 => byte & !0xC0 != 0,
        } {
            info!("Unknown flags found while parsing flags byte of {} tag: {}", version, byte);
        }
        TagFlags {
            byte: byte,
            version: version,
        }
    }

    fn supported(&self, which: TagFlag) -> bool {
        use self::TagFlag::*;
        match which {
            Unsynchronization => true,
            ExtendedHeader => self.version >= Version::V3,
            Experimental => self.version >= Version::V3,
            Footer => self.version >= Version::V4,
            Compression => self.version == Version::V2,
        }
    }

    /// Get the state of a flag.
    pub fn get(&self, which: TagFlag) -> bool {
        self.supported(which) && {
            self.byte & which.value() != 0
        }
    }

    /// Set a flag in the flags to the given value.
    pub fn set(&mut self, which: TagFlag, val: bool) {
        if self.supported(which)
        {
            if val {
                self.byte |= which.value();
            } else {
                self.byte &= !which.value();
            }
        } else {
            warn!("Attempt to set incompatible flag ({}) on version {} tag!", which, self.version);
        }
    }

    /// Create a byte representation of the flags suitable for writing to an ID3 tag.
    pub fn to_byte(&self) -> u8 {
        self.byte
    }
}
// }}}

/// The version of an ID3v2 tag. Supported versions include 2.2, 2.3, and 2.4. When writing new
/// tags, prefer the highest possible version unless specific legacy software demands otherwise.
#[allow(non_camel_case_types, missing_docs)]
#[deriving(Show, PartialEq, Eq, PartialOrd, Copy)]
pub enum Version {
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

macro_rules! id_func (($name: ident, $v2_name: expr, $v34_name: expr) => (
    #[inline]
impl Version {
    #[allow(missing_docs)]
    pub fn $name(&self) -> frame::Id {
        match *self {
            Version::V2 => Id::V2(b!($v2_name)),
            Version::V3 => Id::V3(b!($v34_name)),
            Version::V4 => Id::V4(b!($v34_name)),
        }
    }
}
))

impl Version {
    /// Returns the way this ID3v2 version is encoded in an ID3 tag.
    #[inline]
    pub fn to_bytes(&self) -> [u8, ..2] {
        [*self as u8, 0]
    }

    /// Returns the "best" text encoding compatible with this version of tag.
    ///
    /// For ID3 versions at least v2.4 this is UTF8. For versions less than v2.4,
    /// this is UTF16.
    #[inline]
    pub fn default_encoding(&self) -> Encoding {
        if *self >= Version::V4 {
            Encoding::UTF8
        } else {
            Encoding::UTF16
        }
    }
}
// Frame ID Querying {{{
    id_func!(artist_id, "TP1", "TPE1")
    id_func!(album_artist_id, "TP2", "TPE2")
    id_func!(album_id, "TAL", "TALB")
    id_func!(title_id, "TT2", "TIT2")
    id_func!(genre_id, "TCO", "TCON")
    id_func!(year_id, "TYE", "TYER")
    id_func!(track_id, "TRK", "TRCK")
    id_func!(lyrics_id, "ULT", "USLT")
    id_func!(picture_id, "PIC", "APIC")
    id_func!(comment_id, "COM", "COMM")
    id_func!(txxx_id, "TXX", "TXXX")
// }}}

/// Checks for presence of the signature indicating an ID3v2 tag at the reader's current offset.
/// Consumes 3 bytes from the reader.
pub fn probe_tag<R: Reader>(reader: &mut R) -> IoResult<bool> {
    let identifier = try!(reader.read_exact(3));
    Ok(identifier.as_slice() == b"ID3")
}

/// Read an ID3v2 tag from a reader.
pub fn read_tag<R: Reader>(reader: &mut R) -> TagResult<Tag> {
    use self::TagFlag::*;
    let mut tag = Tag::new();

    if !try!(probe_tag(reader)) {
        debug!("no ID3 tag found");
        return Err(TagError::new(InvalidInputError, "buffer does not contain an ID3 tag"))
    }

    let mut version_bytes = [0u8, ..2];
    try!(reader.read(&mut version_bytes));

    debug!("tag version {}", version_bytes);

    tag.version = match version_bytes.as_slice() {
        [2, 0] => Version::V2,
        [3, 0] => Version::V3,
        [4, 0] => Version::V4,
        _ => return Err(TagError::new(InvalidInputError, "unsupported ID3 tag version")),
    };

    tag.flags = TagFlags::from_byte(try!(reader.read_byte()), tag.version());

    if tag.flags.get(Unsynchronization) {
        warn!("unsynchronization is unsupported");
        return Err(TagError::new(UnsupportedFeatureError, "unsynchronization is not supported"))
    } else if tag.flags.get(Compression) {
        warn!("ID3v2.2 compression is unsupported");
        return Err(TagError::new(UnsupportedFeatureError, "ID3v2.2 compression is not supported"));
    }

    tag.size = util::unsynchsafe(try!(reader.read_be_u32()));

    let mut offset = 10;

    // TODO actually use the extended header data
    if tag.flags.get(ExtendedHeader) {
        let ext_size = util::unsynchsafe(try!(reader.read_be_u32()));
        offset += 4;
        let _ = try!(reader.read_exact(ext_size as uint));
        offset += ext_size;
    }

    while offset < tag.size + 10 {
        let (bytes_read, mut frame) = match Frame::read_from(reader, tag.version()) {
            Ok(opt) => match opt {
                Some(frame) => frame,
                None => break //padding
            },
            Err(err) => {
                debug!("{}", err);
                return Err(err);
            }
        };

        frame.offset = offset;
        tag.frames.push(frame);

        offset += bytes_read;
    }

    tag.offset = offset;
    tag.modified_offset = tag.offset;

    Ok(tag)
}

// Tag {{{
impl Tag {
    /// Create a new ID3v2.4 tag with no frames. 
    #[inline]
    pub fn new() -> Tag {
        Tag { 
            version: Version::V4,
            flags: TagFlags::new(Version::V4),
            frames: Vec::new(),
            size: 0,
            offset: 0,
            modified_offset: 0,
        }
    }

    /// Create a new ID3 tag with the specified version.
    #[inline]
    pub fn with_version(version: Version) -> Tag {
        let mut tag = Tag::new();
        tag.version = version;
        tag
    }

    /// Get the version of this tag.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Sets the version of this tag.
    ///
    /// Any frames that could not be converted to the new version will be dropped.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::Version::{V3, V4};
    ///
    /// let mut tag = id3v2::Tag::with_version(V4);
    /// assert_eq!(tag.version(), V4);
    ///
    /// tag.set_version(V3);
    /// assert_eq!(tag.version(), V3);
    /// ```
    pub fn set_version(&mut self, version: Version) {
        if self.version == version {
            return;
        }

        self.version = version;

        let mut remove = Vec::new();
        for frame in self.frames.iter_mut() {
            if !frame.set_version(version) {
                remove.push(frame as *mut _ as *const _);
            }
        }

        self.modified_offset = 0;

        self.frames.retain(|frame: &Frame| !remove.contains(&(frame as *const _)));
    }

    /// Returns a vector of references to all frames in the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new("TPE1"));
    /// tag.add_frame(Frame::new("APIC"));
    ///
    /// assert_eq!(tag.get_frames().len(), 2);
    /// ```
    #[inline]
    pub fn get_frames<'a>(&'a self) -> &'a Vec<Frame> {
        &self.frames
    }

    /// Returns a reference to the first frame with the specified identifier.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new("TIT2"));
    ///
    /// assert!(tag.get_frame_by_id("TIT2").is_some());
    /// assert!(tag.get_frame_by_id("TCON").is_none());
    /// ```
    pub fn get_frame_by_id<'a>(&'a self, id: frame::Id) -> Option<&'a Frame> {
        for frame in self.frames.iter() {
            if frame.id == id {
                return Some(frame);
            }
        }

        None
    }

    /// Returns a vector of references to frames with the specified identifier.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new("TXXX"));
    /// tag.add_frame(Frame::new("TXXX"));
    /// tag.add_frame(Frame::new("TALB"));
    ///
    /// assert_eq!(tag.get_frames_by_id("TXXX").len(), 2);
    /// assert_eq!(tag.get_frames_by_id("TALB").len(), 1);
    /// ```
    pub fn get_frames_by_id<'a>(&'a self, id: frame::Id) -> Vec<&'a Frame> {
        let mut matches = Vec::new();
        for frame in self.frames.iter() {
            if frame.id == id {
                matches.push(frame);
            }
        }

        matches
    }

    /// Adds a frame to the tag. The frame identifier will attempt to be converted into the
    /// corresponding identifier for the tag version.
    ///
    /// Returns whether the frame was added to the tag. The only reason the frame would not be
    /// added to the tag is if the frame identifier could not be converted from the frame version
    /// to the tag version.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Id;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_frame(Frame::new(Id::V4(b"TALB")));
    /// assert_eq!(tag.get_frames()[0].id, Id::V4(b"TALB"));
    /// ```
    pub fn add_frame(&mut self, mut frame: Frame) -> bool {
        frame.offset = 0;
        if !frame.set_version(self.version()) {
            return false;
        }
        self.frames.push(frame);
        true
    }

    /// Adds a text frame using the default text encoding. Returns whether the
    /// ID was a valid text frame ID and the frame successfully created.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Id;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_text_frame(Id::V4(b!("TCON")), "Metal");
    /// assert_eq!(tag.get_frame_by_id("TCON").unwrap().content.text().as_slice(), "Metal");
    /// ```
    #[inline]
    pub fn add_text_frame(&mut self, id: frame::Id, text: &str) -> bool {
        match Frame::new_text_frame(id, text) {
            Some(frame) => {
                self.remove_frames_by_id(id);
                self.frames.push(frame);
                true
            }
            None => false,
        }
    }

    /// Adds a text frame using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_text_frame_enc("TRCK", "1/13", UTF16);
    /// assert_eq!(tag.get_frame_by_id("TRCK").unwrap().content.text().as_slice(), "1/13");
    /// ```
    /*
    //TODO(sp3d): find a more type-safe way to encode this
    as formulated, there are lots of errors that can be made:
    incompatible version+encoding, lossy transcoding into Latin-1, non-text IDs
    some of these should be preventable in the typesystem
    or handled explicitly as behavior option arguments for encoding*/
    pub fn add_text_frame_enc(&mut self, id: frame::Id, text: &str, encoding: Encoding) {
        self.remove_frames_by_id(id);
        let mut frame = Frame::new_text_frame(id, text/*, encoding*/).unwrap();
        frame.set_encoding(encoding);
        self.frames.push(frame);
    }

    /// Removes all frames with the specified identifier.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new("TXXX"));
    /// tag.add_frame(Frame::new("TXXX"));
    /// tag.add_frame(Frame::new("USLT"));
    ///
    /// assert_eq!(tag.get_frames().len(), 3);
    ///
    /// tag.remove_frames_by_id("TXXX");
    /// assert_eq!(tag.get_frames().len(), 1);
    ///
    /// tag.remove_frames_by_id("USLT");
    /// assert_eq!(tag.get_frames().len(), 0);
    /// ```
    pub fn remove_frames_by_id(&mut self, id: frame::Id) {
        let mut modified_offset = self.modified_offset;
        {
            let set_modified_offset = |offset: u32| {
                if offset != 0 {
                    modified_offset = min(modified_offset, offset);
                }
                false
            };
            self.frames.retain(|frame| {
                frame.id != id || set_modified_offset(frame.offset)
            });
        }
        self.modified_offset = modified_offset;
    }

    /// Returns the content of a text frame with the specified identifier,
    /// converted to UTF8, or `None` if the frame with the specified ID can't be found, if the content is not
    /// textual.
    pub fn text_frame_text(&self, id: frame::Id) -> Option<String> {
        match self.get_frame_by_id(id) {
            Some(frame) => match frame.fields.as_slice() {
                [Field::TextEncoding(encoding), Field::String(ref text)] => util::string_from_encoding(encoding, text.as_slice()),
                _ => None
            },
            None => None
        }
    }

    // Getters/Setters {{{
    /// Returns a vector of the user defined text frames' (TXXX) key/value pairs.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    /// use id3::frame;
    /// use id3::Content::ExtendedTextContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// let mut frame = Frame::new("TXXX");
    /// frame.fields = ExtendedTextContent(frame::ExtendedText { 
    ///     key: "key1".into_string(),
    ///     value: "value1".into_string()
    /// });
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("TXXX");
    /// frame.fields = ExtendedTextContent(frame::ExtendedText { 
    ///     key: "key2".into_string(),
    ///     value: "value2".into_string()
    /// }); 
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.txxx().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    pub fn txxx(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for frame in self.get_frames_by_id(self.version().txxx_id()).iter() {
            match frame.fields.as_slice() {
                //TODO(sp3d): rebuild this on top of fields
                //ExtendedTextContent(ref ext) => out.push((ext.key.clone(), ext.value.clone())),
                _ => { }
            }
        }

        out
    }

    /// Adds a user defined text frame (TXXX).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx("key1", "value1");
    /// tag.add_txxx("key2", "value2");
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.txxx().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    #[inline]
    //pub fn add_txxx<E: Encoding>(&mut self, key: EncodedString<E>, value: EncodedString<E>) {
    pub fn add_txxx(&mut self, key: &str, value: &str) {
        let encoding = self.version().default_encoding();
        self.add_txxx_enc(key, value, encoding);
    }

    /// Adds a user defined text frame (TXXX) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx_enc("key1", "value1", UTF16);
    /// tag.add_txxx_enc("key2", "value2", UTF16);
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.txxx().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    //TODO(sp3d): there has to be a better way of dealing with encoded strings!
    pub fn add_txxx_enc(&mut self, key: &str, value: &str, encoding: Encoding) {
        let key = key.into_string();

        self.remove_txxx(Some(key.as_slice()), None);

        let mut frame = Frame::new(self.version().txxx_id());
        frame.set_encoding(encoding);
        //TODO(sp3d): rebuild this on top of fields
        /*frame.fields = ExtendedTextContent(frame::ExtendedText { 
            key: key, 
            value: value.into_string() 
        });*/
        
        self.frames.push(frame);
    }

    /// Removes the user defined text frame (TXXX) with the specified key and value.
    /// A key or value may be `None` to specify a wildcard value.
    /// 
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx("key1", "value1");
    /// tag.add_txxx("key2", "value2");
    /// tag.add_txxx("key3", "value2");
    /// tag.add_txxx("key4", "value3");
    /// tag.add_txxx("key5", "value4");
    /// assert_eq!(tag.txxx().len(), 5);
    ///
    /// tag.remove_txxx(Some("key1"), None);
    /// assert_eq!(tag.txxx().len(), 4);
    ///
    /// tag.remove_txxx(None, Some("value2"));
    /// assert_eq!(tag.txxx().len(), 2);
    ///
    /// tag.remove_txxx(Some("key4"), Some("value3"));
    /// assert_eq!(tag.txxx().len(), 1);
    ///
    /// tag.remove_txxx(None, None);
    /// assert_eq!(tag.txxx().len(), 0);
    /// ```
    pub fn remove_txxx(&mut self, key: Option<&str>, val: Option<&str>) {
        let mut modified_offset = self.modified_offset;

        let id = self.version().txxx_id();
        self.frames.retain(|frame| {
            let mut key_match = false;
            let mut val_match = false;

            if frame.id == id {
                match frame.fields.as_slice() {
                    [Field::TextEncoding(_), Field::String(ref f_key), Field::String(ref f_val)] => {
                        //TODO(sp3d): checking byte equality is wrong; encodings need to be considered
                        key_match = key.unwrap_or("").as_bytes() == f_key.as_slice();
                        val_match = val.unwrap_or("").as_bytes() == f_val.as_slice();
                    },
                    _ => {
                        // remove frames that we can't parse
                        key_match = true;
                        val_match = true;
                    }
                }
            }

            if key_match && val_match && frame.offset != 0 {
                modified_offset = min(modified_offset, frame.offset);
            }

            !(key_match && val_match) // true if we want to keep the item
        });

        self.modified_offset = modified_offset;
    }

    /// Returns a vector of references to the pictures in the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    /// use id3::frame::Picture;
    /// use id3::Content::PictureContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// 
    /// let mut frame = Frame::new("APIC");
    /// frame.fields = PictureContent(Picture::new());
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("APIC");
    /// frame.fields = PictureContent(Picture::new());
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.pictures().len(), 2);
    /// ```
    pub fn pictures(&self) -> Vec<&simple::Picture> {
        //TODO(sp3d): rebuild this on top of fields
        let mut pictures = Vec::new();
        for frame in self.get_frames_by_id(self.version().picture_id()).iter() {
            match frame.fields.as_slice() {
                _ => { }
            }
        }
        pictures
    }

    /// Adds a picture frame (APIC).
    /// Any other pictures with the same type will be removed from the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::PictureType::Other;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture("image/jpeg", Other, vec!());
    /// tag.add_picture("image/png", Other, vec!());
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(tag.pictures()[0].mime_type.as_slice(), "image/png");
    /// ```
    #[inline]
    pub fn add_picture(&mut self, mime_type: &str, picture_type: PictureType, data: Vec<u8>) {
        self.add_picture_enc(mime_type, picture_type, "", data, Encoding::Latin1);
    }

    /// Adds a picture frame (APIC) using the specified text encoding.
    /// Any other pictures with the same type will be removed from the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::PictureType::Other;
    /// use id3::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture_enc("image/jpeg", Other, "", vec!(), UTF16);
    /// tag.add_picture_enc("image/png", Other, "", vec!(), UTF16);
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(tag.pictures()[0].mime_type.as_slice(), "image/png");
    /// ```
    pub fn add_picture_enc(&mut self, mime_type: &str, picture_type: PictureType, description: &str, data: Vec<u8>, encoding: Encoding) {
        //TODO(sp3d): rebuild this on top of fields
        /*
        self.remove_picture_type(picture_type);

        let mut frame = Frame::new(self.version().picture_id());

        frame.set_encoding(encoding);
        frame.fields = PictureContent(Picture { 
            mime_type: mime_type.into_string(), 
            picture_type: picture_type, 
            description: description.into_string(), 
            data: data 
        });

        self.frames.push(frame);
        */
    }

    /// Removes all pictures of the specified type.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::PictureType::{CoverFront, Other};
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture("image/jpeg", CoverFront, vec!());
    /// tag.add_picture("image/png", Other, vec!());
    /// assert_eq!(tag.pictures().len(), 2);
    ///
    /// tag.remove_picture_type(CoverFront);
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(tag.pictures()[0].picture_type, Other);
    /// ```
    pub fn remove_picture_type(&mut self, picture_type: PictureType) {
        let mut modified_offset = self.modified_offset;

        let id = self.version().picture_id();
        self.frames.retain(|frame| {
            if frame.id == id {
                match frame.fields.as_slice() {
                    //TODO(sp3d): rebuild this on top of fields
                    //PictureContent(ref picture) => picture,
                    _ => return false
                };

                if /*pic.picture_type == picture_type && */frame.offset != 0 {
                    modified_offset = min(modified_offset, frame.offset);
                }

                return false/*pic.picture_type != picture_type*/
            }

            true
        });

        self.modified_offset = modified_offset;
    }

    /// Returns a vector of the user comment frames' (COMM) key/value pairs.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    /// use id3::frame;
    /// use id3::Content::CommentContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// let mut frame = Frame::new("COMM");
    /// frame.fields = CommentContent(frame::Comment {
    ///     lang: "eng".into_string(),
    ///     description: "key1".into_string(),
    ///     text: "value1".into_string()
    /// });
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("COMM");
    /// frame.fields = CommentContent(frame::Comment { 
    ///     lang: "eng".into_string(),
    ///     description: "key2".into_string(),
    ///     text: "value2".into_string()
    /// });
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.comments().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    pub fn comments(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for frame in self.get_frames_by_id(self.version().comment_id()).iter() {
            match frame.fields.as_slice() {
                //TODO(sp3d): rebuild this on top of fields
                /*CommentContent(ref comment) => out.push((comment.description.clone(), 
                                                         comment.text.clone())),*/
                _ => { }
            }
        }

        out
    }
 
    /// Adds a user comment frame (COMM).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment("key1", "value1");
    /// tag.add_comment("key2", "value2");
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.comments().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    #[inline]
    pub fn add_comment(&mut self, description: &str, text: &str) {
        let encoding = self.version().default_encoding();
        self.add_comment_enc("eng", description, text, encoding);
    }

    /// Adds a user comment frame (COMM) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment_enc("eng", "key1", "value1", UTF16);
    /// tag.add_comment_enc("eng", "key2", "value2", UTF16);
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".into_string(), "value1".into_string())));
    /// assert!(tag.comments().contains(&("key2".into_string(), "value2".into_string())));
    /// ```
    pub fn add_comment_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding) {
        let description = description.into_string();

        self.remove_comment(Some(description.as_slice()), None);

        let mut frame = Frame::new(self.version().comment_id());

        //TODO(sp3d): rebuild this on top of fields
        /*frame.set_encoding(encoding);
        frame.fields = CommentContent(frame::Comment { 
            lang: lang.into_string(), 
            description: description, 
            text: text.into_string() 
        });*/
       
        self.frames.push(frame);
    }

    /// Removes the user comment frame (COMM) with the specified key and value.
    /// A key or value may be `None` to specify a wildcard value.
    /// 
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment("key1", "value1");
    /// tag.add_comment("key2", "value2");
    /// tag.add_comment("key3", "value2");
    /// tag.add_comment("key4", "value3");
    /// tag.add_comment("key5", "value4");
    /// assert_eq!(tag.comments().len(), 5);
    ///
    /// tag.remove_comment(Some("key1"), None);
    /// assert_eq!(tag.comments().len(), 4);
    ///
    /// tag.remove_comment(None, Some("value2"));
    /// assert_eq!(tag.comments().len(), 2);
    ///
    /// tag.remove_comment(Some("key4"), Some("value3"));
    /// assert_eq!(tag.comments().len(), 1);
    ///
    /// tag.remove_comment(None, None);
    /// assert_eq!(tag.comments().len(), 0);
    /// ```
    pub fn remove_comment(&mut self, description: Option<&str>, text: Option<&str>) {
        let mut modified_offset = self.modified_offset;

        let id = self.version().comment_id();
        self.frames.retain(|frame| {
            let mut description_match = false;
            let mut text_match = false;

            if frame.id == id {
                match frame.fields.as_slice() {
                    //TODO(sp3d): rebuild this on top of fields
                    /*
                    CommentContent(ref comment) =>  {
                        match description {
                            Some(s) => description_match = s == comment.description.as_slice(),
                            None => description_match = true
                        }

                        match text {
                            Some(s) => text_match = s == comment.text.as_slice(),
                            None => text_match = true 
                        }
                    },*/
                    _ => { // remove frames that we can't parse
                        description_match = true;
                        text_match = true;
                    }
                }
            }

            if description_match && text_match && frame.offset != 0 {
                modified_offset = frame.offset;
            }

            !(description_match && text_match) // true if we want to keep the item
        });

        self.modified_offset = modified_offset;
    }

    /// Sets the artist (TPE1) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_artist_enc("artist", UTF16);
    /// assert_eq!(tag.artist().unwrap().as_slice(), "artist");
    /// ```
    #[inline]
    pub fn set_artist_enc(&mut self, artist: &str, encoding: Encoding) {
        let id = self.version().artist_id();
        self.add_text_frame_enc(id, artist, encoding);
    }

    /// Sets the album artist (TPE2) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_album_artist_enc("album artist", UTF16);
    /// assert_eq!(tag.album_artist().unwrap().as_slice(), "album artist");
    /// ```
    #[inline]
    pub fn set_album_artist_enc(&mut self, album_artist: &str, encoding: Encoding) {
        self.remove_frames_by_id(Id::V3(b!("TSOP")));
        self.remove_frames_by_id(Id::V4(b!("TSOP")));
        let id = self.version().album_artist_id();
        self.add_text_frame_enc(id, album_artist, encoding);
    }

    /// Sets the album (TALB) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_album_enc("album", UTF16);
    /// assert_eq!(tag.album().unwrap().as_slice(), "album");
    /// ```
    #[inline]
    pub fn set_album_enc(&mut self, album: &str, encoding: Encoding) {
        let id = self.version().album_id();
        self.add_text_frame_enc(id, album, encoding);
    }

    /// Sets the song title (TIT2) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_title_enc("title", UTF16);
    /// assert_eq!(tag.title().unwrap().as_slice(), "title");
    /// ```
    #[inline]
    pub fn set_title_enc(&mut self, title: &str, encoding: Encoding) {
        self.remove_frames_by_id(Id::V3(b!("TSOT")));
        self.remove_frames_by_id(Id::V4(b!("TSOT")));
        let id = self.version().title_id();
        self.add_text_frame_enc(id, title, encoding);
    }

    /// Sets the genre (TCON) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_genre_enc("genre", UTF16);
    /// assert_eq!(tag.genre().unwrap().as_slice(), "genre");
    /// ```
    #[inline]
    pub fn set_genre_enc(&mut self, genre: &str, encoding: Encoding) {
        let id = self.version().genre_id();
        self.add_text_frame_enc(id, genre, encoding);
    }

    /// Returns the year (TYER).
    /// Returns `None` if the year frame could not be found or if it could not be parsed.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    /// use id3::Content::TextContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// assert!(tag.year().is_none());
    ///
    /// let mut frame_valid = Frame::new("TYER");
    /// frame_valid.content = TextContent("2014".into_string());
    /// tag.add_frame(frame_valid);
    /// assert_eq!(tag.year().unwrap(), 2014);
    ///
    /// tag.remove_frames_by_id("TYER");
    ///
    /// let mut frame_invalid = Frame::new("TYER");
    /// frame_invalid.content = TextContent("nope".into_string());
    /// tag.add_frame(frame_invalid);
    /// assert!(tag.year().is_none());
    /// ```
    pub fn year(&self) -> Option<uint> {
        let id = self.version().year_id();
        //TODO(sp3d): rebuild this on top of fields
        match self.get_frame_by_id(id) {
            Some(frame) => {
                match frame.fields.as_slice() {
                    //TextContent(ref text) => from_str(text.as_slice()),
                    _ => None
                }
            },
            None => None
        }
    }

    /// Sets the year (TYER).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.set_year(2014);
    /// assert_eq!(tag.year().unwrap(), 2014);
    /// ```
    #[inline]
    pub fn set_year(&mut self, year: uint) {
        let id = self.version().year_id();
        self.add_text_frame_enc(id, format!("{}", year).as_slice(), Encoding::Latin1);
    }

    /// Sets the year (TYER) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.set_year_enc(2014, UTF16);
    /// assert_eq!(tag.year().unwrap(), 2014);
    /// ```
    #[inline]
    pub fn set_year_enc(&mut self, year: uint, encoding: Encoding) {
        let id = self.version().year_id();
        self.add_text_frame_enc(id, format!("{}", year).as_slice(), encoding);
    }

    /// Returns the (track, total_tracks) tuple.
    pub fn track_pair(&self) -> Option<(u32, Option<u32>)> {
        match self.get_frame_by_id(self.version().track_id()) {
            Some(frame) => {
                //TODO(sp3d): rebuild this on top of fields
                match frame.fields.as_slice() {
                    /*TextContent(ref text) => {
                        let split: Vec<&str> = text.as_slice().splitn(2, '/').collect();

                        let total_tracks = if split.len() == 2 {
                            match from_str(split[1]) {
                                Some(total_tracks) => Some(total_tracks),
                                None => return None
                            }
                        } else {
                            None
                        };

                        match from_str(split[0]) {
                            Some(track) => Some((track, total_tracks)),
                            None => None
                        }
                    },*/
                    _ => None
                }
            },
            None => None
        }
    }

    /// Sets the track number (TRCK) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_track_enc(5, UTF16);
    /// assert_eq!(tag.track().unwrap(), 5);
    /// ```
    pub fn set_track_enc(&mut self, track: u32, encoding: Encoding) {
        let text = match self.track_pair().and_then(|(_, total_tracks)| total_tracks) {
            Some(n) => format!("{}/{}", track, n),
            None => format!("{}", track)
        };

        let id = self.version().track_id();
        self.add_text_frame_enc(id, text.as_slice(), encoding);
    }


    /// Sets the total number of tracks (TRCK) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_total_tracks_enc(12, UTF16);
    /// assert_eq!(tag.total_tracks().unwrap(), 12);
    /// ```
    pub fn set_total_tracks_enc(&mut self, total_tracks: u32, encoding: Encoding) {
        let text = match self.track_pair() {
            Some((track, _)) => format!("{}/{}", track, total_tracks),
            None => format!("1/{}", total_tracks)
        };

        let id = self.version().track_id();
        self.add_text_frame_enc(id, text.as_slice(), encoding);
    }


    /// Sets the lyrics text (USLT) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::{AudioTag, id3v2};
    /// use id3::Encoding::UTF16;
    /// use id3::tag::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_lyrics_enc("eng", "description", "lyrics", UTF16);
    /// assert_eq!(tag.lyrics().unwrap().as_slice(), "lyrics");
    /// ```
    pub fn set_lyrics_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding) {
        let id = self.version().lyrics_id();
        self.remove_frames_by_id(id);

        let mut frame = Frame::new(id);

        frame.set_encoding(encoding);
        //TODO(sp3d): rebuild this on top of fields
        /*frame.fields = LyricsContent(frame::Lyrics { 
            lang: lang.into_string(), 
            description: description.into_string(), 
            text: text.into_string() 
        });*/
        
        self.frames.push(frame);
    }
    //}}}
}
