use std::cmp::min;
use frame::{mod, Frame, Encoding, Picture, PictureType};
use frame::Content::{PictureContent, CommentContent, TextContent, ExtendedTextContent, LyricsContent};

/// An ID3 tag containing metadata frames. 
#[deriving(Show)]
pub struct Tag {
    /// The version of the tag. The first byte represents the major version number, while the
    /// second byte represents the revision number.
    pub version: SupportedVersion,
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

/// Flags used in the ID3v2 header.
#[deriving(Show)]
pub struct TagFlags {
    /// Indicates whether or not unsynchronization is used.
    pub unsynchronization: bool,
    /// Indicates whether or not the header is followed by an extended header.
    pub extended_header: bool,
    /// Indicates whether the tag is in an experimental stage.
    pub experimental: bool,
    /// Indicates whether a footer is present.
    pub footer: bool,
    /// Indicates whether or not compression is used. This flag is only used in ID3v2.2.
    pub compression: bool // v2.2 only
}

// TagFlags {{{
impl TagFlags {
    /// Creates a new `TagFlags` with all flags set to false.
    #[inline]
    pub fn new() -> TagFlags {
        TagFlags { 
            unsynchronization: false, extended_header: false, experimental: false, 
            footer: false, compression: false 
        }
    }

    /// Creates a new `TagFlags` using the provided byte.
    pub fn from_byte(byte: u8, version: u8) -> TagFlags {
        let mut flags = TagFlags::new();

        flags.unsynchronization = byte & 0x80 != 0;

        if version == 2 {
            flags.compression = byte & 0x40 != 0;
        } else {
            flags.extended_header = byte & 0x40 != 0;
            flags.experimental = byte & 0x20 != 0;

            if version == 4 {
                flags.footer = byte & 0x10 != 0;
            }
        }

        flags
    }

    /// Creates a byte representation of the flags suitable for writing to an ID3 tag.
    pub fn to_byte(&self, version: u8) -> u8 {
        let mut byte = 0;
       
        if self.unsynchronization {
            byte |= 0x80;
        }

        if version == 2 {
            if self.compression {
                byte |= 0x40;
            }
        } else {
            if self.extended_header {
                byte |= 0x40;
            }

            if self.experimental {
                byte |= 0x20
            }

            if version == 4 {
                if self.footer {
                    byte |= 0x10;
                }
            }
        }

        byte
    }
}
// }}}

/// The version of an ID3v2 tag. Supported versions include 2.2, 2.3, and 2.4. When writing new
/// tags, prefer the highest possible version unless specific legacy software demands otherwise.
#[allow(non_camel_case_types, missing_docs)]
#[deriving(Show, PartialEq, Eq)]
pub enum SupportedVersion {
    V2_2 = 2,
    V2_3 = 3,
    V2_4 = 4,
}

#[allow(missing_docs)]
impl SupportedVersion {
    pub fn to_bytes(&self) -> [u8, ..2] {
        [*self as u8, 0]
    }

    // Frame ID Querying {{{
    #[inline]
    pub fn artist_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TP1" } else { "TPE1" }
    }

    #[inline]
    pub fn album_artist_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TP2" } else { "TPE2" }
    }

    #[inline]
    pub fn album_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TAL" } else { "TALB" }
    }

    #[inline]
    pub fn title_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TT2" } else { "TIT2" }
    }

    #[inline]
    pub fn genre_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TCO" } else { "TCON" }
    }

    #[inline]
    pub fn year_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TYE" } else { "TYER" }
    }

    #[inline]
    pub fn track_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TRK" } else { "TRCK" }
    }

    #[inline]
    pub fn lyrics_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "ULT" } else { "USLT" }
    }

    #[inline]
    pub fn picture_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "PIC" } else { "APIC" }
    }

    #[inline]
    pub fn comment_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "COM" } else { "COMM" }
    }

    #[inline]
    pub fn txxx_id(&self) -> &'static str {
        if *self == SupportedVersion::V2_2 { "TXX" } else { "TXXX" }
    }
    // }}}
}

// Tag {{{
impl Tag {
    /// Creates a new ID3v2.3 tag with no frames. 
    #[inline]
    pub fn new() -> Tag {
        Tag { 
            version: SupportedVersion::V2_4,
            flags: TagFlags::new(), 
            frames: Vec::new(),
            size: 0,
            offset: 0,
            modified_offset: 0,
        }
    }

    /// Creates a new ID3 tag with the specified version.
    #[inline]
    pub fn with_version(version: SupportedVersion) -> Tag {
        let mut tag = Tag::new();
        tag.version = version;
        tag
    }

    /// Get the version of this tag.
    #[inline]
    pub fn version(&self) -> SupportedVersion {
        self.version
    }

    /// Sets the version of this tag.
    ///
    /// Any frames that could not be converted to the new version will be dropped.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::SupportedVersion::{V2_3, V2_4};
    ///
    /// let mut tag = id3v2::Tag::with_version(V2_4);
    /// assert_eq!(tag.version(), V2_4);
    ///
    /// tag.set_version(V2_3);
    /// assert_eq!(tag.version(), V2_3);
    /// ```
    pub fn set_version(&mut self, version: SupportedVersion) {
        if self.version == version {
            return;
        }

        self.version = version;
        
        let mut remove_uuid = Vec::new();
        for frame in self.frames.iter_mut() {
            if !frame.set_version(version.to_bytes()[0]) {
                remove_uuid.push(frame.uuid.clone());
            }
        }

        self.modified_offset = 0;
            
        self.frames.retain(|frame: &Frame| !remove_uuid.contains(&frame.uuid));
    }

    /// Returns the default unicode encoding that should be used for this tag.
    ///
    /// For ID3 versions at least v2.4 this returns UTF8. For versions less than v2.4 this
    /// returns UTF16.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::SupportedVersion::{V2_3, V2_4};
    /// use id3::Encoding::{UTF16, UTF8};
    ///
    /// let mut tag_v3 = id3v2::Tag::with_version(V2_3);
    /// assert_eq!(tag_v3.default_encoding(), UTF16);
    ///
    /// let mut tag_v4 = id3v2::Tag::with_version(V2_4);
    /// assert_eq!(tag_v4.default_encoding(), UTF8);
    /// ```
    #[inline]
    pub fn default_encoding(&self) -> Encoding {
        if self.version == SupportedVersion::V2_4 {
            Encoding::UTF8
        } else {
            Encoding::UTF16
        }
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
    pub fn get_frame_by_id<'a>(&'a self, id: &str) -> Option<&'a Frame> {
        for frame in self.frames.iter() {
            if frame.id.as_slice() == id {
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
    pub fn get_frames_by_id<'a>(&'a self, id: &str) -> Vec<&'a Frame> {
        let mut matches = Vec::new();
        for frame in self.frames.iter() {
            if frame.id.as_slice() == id {
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
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_frame(Frame::new("TALB"));
    /// assert_eq!(tag.get_frames()[0].id.as_slice(), "TALB");
    /// ```
    pub fn add_frame(&mut self, mut frame: Frame) -> bool {
        frame.generate_uuid();
        frame.offset = 0;
        if !frame.set_version(self.version().to_bytes()[0]) {
            return false;
        }
        self.frames.push(frame);
        true
    }

    /// Adds a text frame using the default text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_text_frame("TCON", "Metal");
    /// assert_eq!(tag.get_frame_by_id("TCON").unwrap().content.text().as_slice(), "Metal");
    /// ```
    #[inline]
    pub fn add_text_frame<K: StrAllocating, V: StrAllocating>(&mut self, id: K, text: V) {
        let encoding = self.default_encoding();
        self.add_text_frame_enc(id, text, encoding);
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
    pub fn add_text_frame_enc<K: StrAllocating, V: StrAllocating>(&mut self, id: K, text: V, encoding: Encoding) {
        let id = id.into_string();

        self.remove_frames_by_id(id.as_slice());
       
        let mut frame = Frame::with_version(id, self.version().to_bytes()[0]);
        frame.set_encoding(encoding);
        frame.content = TextContent(text.into_string());

        self.frames.push(frame);
    }

    /// Removes the frame with the specified uuid.
    /// 
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::frame::Frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new("TPE2"));
    /// assert_eq!(tag.get_frames().len(), 1);
    ///
    /// let uuid = tag.get_frames()[0].uuid.clone();
    /// tag.remove_frame_by_uuid(uuid.as_slice());
    /// assert_eq!(tag.get_frames().len(), 0);
    /// ```
    pub fn remove_frame_by_uuid(&mut self, uuid: &[u8]) {
        let mut modified_offset = self.modified_offset;
        {
            let set_modified_offset = |offset: u32| {
                if offset != 0 {
                    modified_offset = min(modified_offset, offset);
                }
                false
            };
            self.frames.retain(|frame| {
                frame.uuid.as_slice() != uuid || set_modified_offset(frame.offset)
            });
        }
        self.modified_offset = modified_offset;
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
    pub fn remove_frames_by_id(&mut self, id: &str) {
        let mut modified_offset = self.modified_offset;
        {
            let set_modified_offset = |offset: u32| {
                if offset != 0 {
                    modified_offset = min(modified_offset, offset);
                }
                false
            };
            self.frames.retain(|frame| {
                frame.id.as_slice() != id || set_modified_offset(frame.offset)
            });
        }
        self.modified_offset = modified_offset;
    }

    /// Returns the `TextContent` string for the frame with the specified identifier.
    /// Returns `None` if the frame with the specified ID can't be found or if the content is not
    /// `TextContent`.
    pub fn text_for_frame_id(&self, id: &str) -> Option<String> {
        match self.get_frame_by_id(id) {
            Some(frame) => match frame.content {
                TextContent(ref text) => Some(text.clone()),
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
    /// frame.content = ExtendedTextContent(frame::ExtendedText { 
    ///     key: "key1".into_string(),
    ///     value: "value1".into_string()
    /// });
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("TXXX");
    /// frame.content = ExtendedTextContent(frame::ExtendedText { 
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
            match frame.content {
                ExtendedTextContent(ref ext) => out.push((ext.key.clone(), ext.value.clone())),
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
    pub fn add_txxx<K: StrAllocating, V: StrAllocating>(&mut self, key: K, value: V) {
        let encoding = self.default_encoding();
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
    pub fn add_txxx_enc<K: StrAllocating, V: StrAllocating>(&mut self, key: K, value: V, encoding: Encoding) {
        let key = key.into_string();

        self.remove_txxx(Some(key.as_slice()), None);

        let mut frame = Frame::with_version(self.version().txxx_id(), self.version().to_bytes()[0]);
        frame.set_encoding(encoding);
        frame.content = ExtendedTextContent(frame::ExtendedText { 
            key: key, 
            value: value.into_string() 
        });
        
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
    pub fn remove_txxx(&mut self, key: Option<&str>, value: Option<&str>) {
        let mut modified_offset = self.modified_offset;

        let id = self.version().txxx_id();
        self.frames.retain(|frame| {
            let mut key_match = false;
            let mut value_match = false;

            if frame.id.as_slice() == id {
                match frame.content {
                    ExtendedTextContent(ref ext) => {
                        match key {
                            Some(s) => key_match = s == ext.key.as_slice(),
                            None => key_match = true
                        }

                        match value {
                            Some(s) => value_match = s == ext.value.as_slice(),
                            None => value_match = true 
                        }
                    },
                    _ => { // remove frames that we can't parse
                        key_match = true;
                        value_match = true;
                    }
                }
            }

            if key_match && value_match && frame.offset != 0 {
                modified_offset = min(modified_offset, frame.offset);
            }

            !(key_match && value_match) // true if we want to keep the item
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
    /// frame.content = PictureContent(Picture::new());
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("APIC");
    /// frame.content = PictureContent(Picture::new());
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.pictures().len(), 2);
    /// ```
    pub fn pictures(&self) -> Vec<&Picture> {
        let mut pictures = Vec::new();
        for frame in self.get_frames_by_id(self.version().picture_id()).iter() {
            match frame.content {
                PictureContent(ref picture) => pictures.push(picture),
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
    pub fn add_picture<T: StrAllocating>(&mut self, mime_type: T, picture_type: PictureType, data: Vec<u8>) {
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
    pub fn add_picture_enc<S: StrAllocating, T: StrAllocating>(&mut self, mime_type: S, picture_type: PictureType, description: T, data: Vec<u8>, encoding: Encoding) {
        self.remove_picture_type(picture_type);

        let mut frame = Frame::with_version(self.version().picture_id(), self.version().to_bytes()[0]);

        frame.set_encoding(encoding);
        frame.content = PictureContent(Picture { 
            mime_type: mime_type.into_string(), 
            picture_type: picture_type, 
            description: description.into_string(), 
            data: data 
        });

        self.frames.push(frame);
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
            if frame.id.as_slice() == id {
                let pic = match frame.content {
                    PictureContent(ref picture) => picture,
                    _ => return false
                };

                if pic.picture_type == picture_type && frame.offset != 0 {
                    modified_offset = min(modified_offset, frame.offset);
                }

                return pic.picture_type != picture_type
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
    /// frame.content = CommentContent(frame::Comment {
    ///     lang: "eng".into_string(),
    ///     description: "key1".into_string(),
    ///     text: "value1".into_string()
    /// });
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new("COMM");
    /// frame.content = CommentContent(frame::Comment { 
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
            match frame.content {
                CommentContent(ref comment) => out.push((comment.description.clone(), 
                                                         comment.text.clone())),
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
    pub fn add_comment<K: StrAllocating, V: StrAllocating>(&mut self, description: K, text: V) {
        let encoding = self.default_encoding();
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
    pub fn add_comment_enc<L: StrAllocating, K: StrAllocating, V: StrAllocating>(&mut self, lang: L, description: K, text: V, encoding: Encoding) {
        let description = description.into_string();

        self.remove_comment(Some(description.as_slice()), None);

        let mut frame = Frame::with_version(self.version().comment_id(), self.version().to_bytes()[0]);

        frame.set_encoding(encoding);
        frame.content = CommentContent(frame::Comment { 
            lang: lang.into_string(), 
            description: description, 
            text: text.into_string() 
        });
       
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

            if frame.id.as_slice() == id {
                match frame.content {
                    CommentContent(ref comment) =>  {
                        match description {
                            Some(s) => description_match = s == comment.description.as_slice(),
                            None => description_match = true
                        }

                        match text {
                            Some(s) => text_match = s == comment.text.as_slice(),
                            None => text_match = true 
                        }
                    },
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
    pub fn set_artist_enc<T: StrAllocating>(&mut self, artist: T, encoding: Encoding) {
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
    pub fn set_album_artist_enc<T: StrAllocating>(&mut self, album_artist: T, encoding: Encoding) {
        self.remove_frames_by_id("TSOP");
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
    pub fn set_album_enc<T: StrAllocating>(&mut self, album: T, encoding: Encoding) {
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
    pub fn set_title_enc<T: StrAllocating>(&mut self, title: T, encoding: Encoding) {
        self.remove_frames_by_id("TSOT");
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
    pub fn set_genre_enc<T: StrAllocating>(&mut self, genre: T, encoding: Encoding) {
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
        match self.get_frame_by_id(id) {
            Some(frame) => {
                match frame.content {
                    TextContent(ref text) => from_str(text.as_slice()),
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
        self.add_text_frame_enc(id, format!("{}", year), Encoding::Latin1);
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
        self.add_text_frame_enc(id, format!("{}", year), encoding);
    }

    /// Returns the (track, total_tracks) tuple.
    pub fn track_pair(&self) -> Option<(u32, Option<u32>)> {
        match self.get_frame_by_id(self.version().track_id()) {
            Some(frame) => {
                match frame.content {
                    TextContent(ref text) => {
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
                    },
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
        self.add_text_frame_enc(id, text, encoding);
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
        self.add_text_frame_enc(id, text, encoding);
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
    pub fn set_lyrics_enc<L: StrAllocating, K: StrAllocating, V: StrAllocating>(&mut self, lang: L, description: K, text: V, encoding: Encoding) {
        let id = self.version().lyrics_id();
        self.remove_frames_by_id(id);

        let mut frame = Frame::with_version(id, self.version().to_bytes()[0]);

        frame.set_encoding(encoding);
        frame.content = LyricsContent(frame::Lyrics { 
            lang: lang.into_string(), 
            description: description.into_string(), 
            text: text.into_string() 
        });
        
        self.frames.push(frame);
    }
    //}}}
}
