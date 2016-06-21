use std::io::{self, Read, Write};
use std::io::ErrorKind::InvalidInput;
use self::frame::{Frame, Encoding, Id};
use self::frame::field::Field;

pub use self::error::{Error, ErrorKind};

use util;
use std::fmt;

mod error;

/// Tools for working with ID3v2 frames.
pub mod frame;
/// High-level, lossy, and simple accessors for basic tag content.
pub mod simple;

/// An ID3v2 tag containing metadata frames.
#[derive(Debug)]
pub struct Tag {
    /// The version of the ID3v2 tag.
    version: Version,
    /// The ID3v2 header flags.
    flags: TagFlags,
    /// A vector of frames included in the tag.
    pub frames: Vec<Frame>,
    /// The size of padding which was included in the tag's serialized form.
    padding_len: u32,
    /// Extended header data (ID3v2.3 or ID3v2.4), if present.
    extended_header: Option<ExtendedHeader>,
}

/// A flag indicating the presence of a particular piece of ID3v2 extended header data.
#[derive(Debug)]
pub enum ExtendedFlag {
    /// Indicates that this ID3v2 tag is an update to an earlier tag in the stream, as
    /// might occur in streaming media playback to override the previous track's title
    /// and other metadata. This flag has no payload (ID3v2.4).
    Update,
    /// Indicates the presence of a payload containing a CRC32 checksum of the frame
    /// data (before unsynchronization) between the extended header and the padding
    /// (ID3v2.3 or ID3v2.4).
    Crc,
    /// Indicates a 1-byte payload specifying restrictions to be placed on the tag,
    /// such as total tag size, text encodings, string lengths, image formats, and
    /// image dimensions (ID3v2.4).
    TagRestrictions,
    /// An unknown extended header entry. To comply with the ID3v2.4 spec, unknown
    /// extended header data MUST be removed when the tag is modified. The payload
    /// may be any size.
    Unknown(u8),//TODO(sp3d): preserve flag index!
}

impl ExtendedFlag
{
    /// Find the index of an extended header flag in a tag of the given ID3v2 version,
    /// counting from 0 at the first byte's MSB.
    pub fn to_index(&self, version: Version) -> u8
    {
        match (version, self)
        {
            (Version::V3, &ExtendedFlag::Crc) => 0,
            (Version::V3, &ExtendedFlag::Unknown(n)) => n,
            (Version::V4, &ExtendedFlag::Update) => 1,
            (Version::V4, &ExtendedFlag::Crc) => 2,
            (Version::V4, &ExtendedFlag::TagRestrictions) => 3,
            (Version::V4, &ExtendedFlag::Unknown(n)) => n,
            _ => panic!("extended header flag incompatible with ID3v2 version"),
        }
    }
    /// Obtain the meaning of an ID3v2 extended header flag from the index of its
    /// bit in the flag bytes, counting from 0 at the first byte's MSB.
    pub fn from_index(n: u8, version: Version) -> ExtendedFlag
    {
        match (version, n)
        {
            (Version::V3, 0) => ExtendedFlag::Crc,
            (Version::V3, n) => ExtendedFlag::Unknown(n),
            (Version::V4, 1) => ExtendedFlag::Update,
            (Version::V4, 2) => ExtendedFlag::Crc,
            (Version::V4, 3) => ExtendedFlag::TagRestrictions,
            (Version::V4, n) => ExtendedFlag::Unknown(n),
            _ => panic!("extended header flag incompatible with ID3v2 version"),
        }
    }
}


/// An iterator adaptor that groups iterator elements. Consecutive elements
/// that map to the same key ("runs"), are succesively passed to the folding closure.
///
/// See [*.group_by()*](trait.Itertools.html#method.group_by) for more information.
pub struct GroupBy<I, FK, K, FV, V> where
    I: Iterator,
    FK: FnMut(&I::Item) -> K,
    FV: FnMut(::std::iter::TakeWhile<&mut ::std::iter::Peekable<I>, &mut FnMut(&I::Item) -> bool>) -> V,
{
    key: FK,
    fold: FV,
    iter: ::std::iter::Peekable<I>,
}

impl<I, FK, K, FV, V> GroupBy<I, FK, K, FV, V> where
    I: Iterator,
    FK: FnMut(&I::Item) -> K,
    FV: FnMut(::std::iter::TakeWhile<&mut ::std::iter::Peekable<I>, &mut FnMut(&I::Item) -> bool>) -> V,
{
    /// Create a new `GroupBy` iterator.
    pub fn new(iter: I, key: FK, fold: FV) -> Self
    {
        GroupBy{key: key, fold: fold, iter: iter.peekable(), }
    }
}

impl<K, I, FK, FV, V> Iterator for GroupBy<I, FK, K, FV, V> where
    K: PartialEq,
    I: Iterator,
    FK: FnMut(&I::Item) -> K,
    FV: FnMut(::std::iter::TakeWhile<&mut ::std::iter::Peekable<I>, &mut FnMut(&I::Item) -> bool>) -> V,
{
    type Item = V;
    fn next(&mut self) -> Option<V>
    {
        let some = self.iter.peek().is_some();
        if some
        {
            let key = (self.key)(self.iter.peek().unwrap());
            let mut ffold = &mut self.fold;
            let fkey = &mut self.key;
            let mut iter = &mut self.iter;
            let v=(ffold)(iter.take_while(&mut |x| (fkey)(x)==key ));
            Some(v)
        }
        else
        {
            None
        }
    }

/*    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let stored_count = self.current_key.is_some() as usize;
        let mut sh = size_hint::add_scalar(self.iter.size_hint(),
                                           stored_count);
        if sh.0 > 0 {
            sh.0 = 1;
        }
        sh
    }*/
}


/*
pub struct GroupBy<>
{
    iter: I,
    callback: F,
    group: P
}
fn group_by<K: PartialEq, T, I: Iterator<Item=T>>(x: I, compare:  -> bool each: ) -> GroupBy<>
{
}
impl<T, V, I: Iterator<Item=T>, F: Fn(I) -> V> Iterator for GroupBy<I, T>
{
    type Item = V;
    fn next(&mut self)
    {
        self.iter.take_while()
    }
}
*/


/// An ID3v2 extended header, which consists of a series of flags and
/// corresponding data payloads.
#[derive(Debug)]
pub struct ExtendedHeader {
    flag_data: Vec<(ExtendedFlag, Vec<u8>)>
}

impl ExtendedHeader {
    /// Return the size in bytes of the serialized extended header.
    pub fn size(&self) -> usize {
        let flag_data_len: usize=self.flag_data.iter().map(|&(_, ref vec)| vec.len()).sum();
        4/*size field*/+1/*bytes of flags*/+flag_data_len
    }
    /// Write the extended header to a writer.
    pub fn write_to(&self, writer: &mut Write, version: Version) -> io::Result<u32> {
        let size = self.size() as u32;
        //TODO: verify endianness?
        try!(writer.write(&util::u32_to_bytes(util::synchsafe(size))));
        match version
        {
            Version::V2 => panic!("attempting to write extended header for an ID3v2.2 tag"),
            Version::V3 => try!(writer.write(&[1u8])),
            Version::V4 => try!(writer.write(&[42u8])),//TODO(sp3d): try!(writer.write(n_flag_bytes)),
        };
        //TODO(sp3d): write flag bytes
        //write flag payloads
        for &(_, ref vec) in self.flag_data.iter() {
            try!(writer.write(&[vec.len() as u8]));
            try!(writer.write(&*vec));
        }
        Ok(size)
    }
    /// Parse an ID3v2 extended header for a tag with the given ID3v2 version from a reader.
    /// The version must be Version::V3 or Version::V4.
    pub fn parse<R: Read>(reader: &mut R, version: Version) -> io::Result<(ExtendedHeader, usize)> {
        let mut offset = 0;
        let size = util::unsynchsafe(read_be_u32!(reader));
        offset += 4;

        //figure out how many bytes of flags to read
        let n_flag_bytes = match version
        {
            Version::V2 => panic!("attempting to parse extended header for an ID3v2.2 tag"),
            Version::V3 => 2,
            Version::V4 => {
                offset += 1;
                read_u8!(reader)
            }
        };

        //read the flags themselves
        let mut flags = vec![];
        let mut bit_index = 0;
        for _ in 0..n_flag_bytes
        {
            let flag_byte = read_u8!(reader);
            offset += 1;

            for bit in 0..8//flag_byte.one_bits_from_msb()
            {
                let bit = (flag_byte>>(7-bit)) & 1;
                if bit == 1
                {
                    flags.push(ExtendedFlag::from_index(bit_index, version));
                }
                bit_index += 1;
            }
        }
        let mut flag_data=vec![];
        let mut size_remaining = size;

        //read the payload, in (data_size, data) format, for each flag
        for flag in flags
        {
            let data_size = read_u8!(reader) as u32;
            offset += 1;

            if size_remaining < data_size
            {
                //TODO(sp3d): return error
                //return Err("ran out of data before running out of flags");
                panic!("ran out of data before running out of flags");
            }

            let mut flag_datum = vec![0; data_size as usize]; try!(reader.read(&mut flag_datum)); //read_all!(reader, &mut ext_header);
            flag_data.push((flag, flag_datum));

            size_remaining -= data_size;
            offset += data_size as usize;
        }

        Ok((ExtendedHeader { flag_data: flag_data }, offset))
    }
}

/// Flags used in ID3v2 tag headers.
#[derive(Debug, Copy, Clone)]
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
    /// Returns the value of a byte in which only this flag is set.
    #[inline]
    pub fn value(&self) -> u8 {
        [0x80, 0x40, 0x20, 0x10, 0x40][*self as usize]
    }
}

/// The flags set in an ID3v2 header.
#[derive(Copy, Clone)]
pub struct TagFlags {
    byte: u8,
    version: Version,
}

impl fmt::Debug for TagFlags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        //TODO(sp3d): verify that the Ok case returns the right value
        use self::TagFlag::*;
        try!(fmt.write_str("{"));
        for i in [Unsynchronization, ExtendedHeader, Experimental, Footer, Compression].iter() {
            if self.get(*i) {
                try!(i.fmt(fmt));
                try!(fmt.write_str(" "));
            }
        }
        fmt.write_str("}")
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
            info!("Unknown flags found while parsing flags byte of {:?} tag: {}", version, byte);
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
            warn!("Attempt to set incompatible flag ({:?}) on version {:?} tag!", which, self.version);
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Copy, Clone)]
pub enum Version {
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

macro_rules! id_func (($name: ident, $v2_name: expr, $v34_name: expr) => (
impl Version {
    #[allow(missing_docs)]
    pub fn $name(&self) -> frame::Id {
        match *self {
            Version::V2 => Id::V2(*$v2_name),
            Version::V3 => Id::V3(*$v34_name),
            Version::V4 => Id::V4(*$v34_name),
        }
    }
}
));

impl Version {
    /// Returns the way this ID3v2 version is encoded in an ID3 tag.
    #[inline]
    pub fn to_bytes(&self) -> [u8; 2] {
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

    /// Returns whether an encoding is compatible with this version of tag.
    ///
    /// ID3v2.4 is compatible with UTF-8 and UTF-16be in addition to UTF-16 and
    /// Latin-1, which are supported by v2.2 and v2.3.
    pub fn encoding_compatible(&self, encoding: Encoding) -> bool {
        if *self >= Version::V4 {
            true
        } else {
            encoding == Encoding::UTF16 || encoding == Encoding::Latin1
        }
    }

    /// Returns the encodings compatible with the frame's version.
    #[inline]
    pub fn compatible_encodings(&self) -> &[Encoding] {
        match *self {
            Version::V2|Version::V3 => static_arr!(Encoding, [Encoding::Latin1, Encoding::UTF16]),
            Version::V4 => static_arr!(Encoding, [Encoding::Latin1, Encoding::UTF16, Encoding::UTF16BE, Encoding::UTF8]),
        }
    }
}
// Frame ID Querying {{{
    id_func!(artist_id, b"TP1", b"TPE1");
    id_func!(album_artist_id, b"TP2", b"TPE2");
    id_func!(album_id, b"TAL", b"TALB");
    id_func!(title_id, b"TT2", b"TIT2");
    id_func!(genre_id, b"TCO", b"TCON");
    id_func!(year_id, b"TYE", b"TYER");
    id_func!(track_id, b"TRK", b"TRCK");
    id_func!(lyrics_id, b"ULT", b"USLT");
    id_func!(picture_id, b"PIC", b"APIC");
    id_func!(comment_id, b"COM", b"COMM");
    id_func!(txxx_id, b"TXX", b"TXXX");
// }}}

/// Checks for presence of the signature indicating an ID3v2 tag at the reader's current offset.
/// Consumes 3 bytes from the reader.
pub fn probe_tag<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut identifier = [0u8; 3];
    try!(reader.read(&mut identifier));
    Ok(identifier == *b"ID3")
}

/// Read an ID3v2 tag from a reader.
pub fn read_tag<R: Read>(mut reader: &mut R) -> Result<Option<Tag>, io::Error> {
    use self::TagFlag::*;
    let mut tag = Tag::new();

    if !try!(probe_tag(reader)) {
        return Ok(None)
    }

    let mut version_bytes = [0u8; 2];
    try!(reader.read(&mut version_bytes));

    debug!("tag version bytes {:?}", version_bytes);

    tag.version = match version_bytes {
        [2, 0] => Version::V2,
        [3, 0] => Version::V3,
        [4, 0] => Version::V4,
        _ => return Err(io::Error::new(InvalidInput, "unsupported ID3 tag version").into()),
    };

    tag.flags = TagFlags::from_byte(read_u8!(reader), tag.version());

    if tag.flags.get(Unsynchronization) {
        panic!("TODO: ID3v2 unsynchronization is not yet implemented");
    } else if tag.flags.get(Compression) {
        panic!("ID3v2.2 compression is unsupported");
    }

    let tag_size = util::unsynchsafe(read_be_u32!(reader));

    let mut offset = 10;

    // TODO actually use the extended header data
    if tag.flags.get(ExtendedHeader) {
        let (eh, eh_size) = try!(self::ExtendedHeader::parse(&mut reader, tag.version));
        tag.extended_header = Some(eh);
        offset += eh_size;
    }

    let mut padding_len = 0;

    while offset < tag_size as usize + 10 {
        let frame = match Frame::read_from(reader, tag.version()) {
            Ok((bytes_read, maybe_frame)) => {
                offset += bytes_read as usize;
                match maybe_frame {
                    Some(frame) => frame,
                    None => {padding_len += bytes_read; continue}, //start of padding
                }
            },
            Err(err) => {
                debug!("{}", err);
                return Err(io::Error::new(InvalidInput, err.to_string()));
            },
        };

        tag.frames.push(frame);
    }

    tag.padding_len = padding_len as u32;

    Ok(Some(tag))
}

// Tag {{{
impl Tag {
    /// Create a new ID3v2.4 tag with no frames.
    #[inline]
    pub fn new() -> Tag {
        Tag::with_version(Version::V4)
    }

    /// Create a new ID3 tag with the specified version.
    #[inline]
    pub fn with_version(version: Version) -> Tag {
        Tag {
            version: version,
            flags: TagFlags::new(version),
            frames: Vec::new(),
            padding_len: 0,
            extended_header: None,
        }
    }

    /// Get the tag's ID3v2 version.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the serialized size of the tag.
    #[inline]
    pub fn size(&self) -> u32 {
        10 + self.frames.iter().map(|x| x.size()).sum::<u32>()
    }

    /// Serialize the ID3v2 tag to a writer. If successful, returns the number
    /// of bytes written.
    pub fn write_to(&self, writer: &mut Write) -> Result<u32, io::Error> {
        try!(writer.write(b"ID3"));
        try!(writer.write(&self.version().to_bytes()));
        try!(writer.write(&[self.flags().to_byte()]));
        try!(writer.write(&util::u32_to_bytes(u32::to_be(util::synchsafe(self.size())))));

        let mut bytes_written = 10;

        if let Some(ref extended) = self.extended_header {
            debug!("writing extended header");
            try!(extended.write_to(writer, self.version));
        };

        for frame in &self.frames {
            debug!("writing {:?}", frame.id);
            bytes_written += try!(frame.write_to(writer));
        }
        Ok(bytes_written)
    }

    /// Converts the tag to the specified version, dropping any data that
    /// cannot be represented in the new version.
    ///
    /// Since this is a lossy conversion, converting a tag from version A to
    /// version B and then back to its original version is unlikely to preserve
    /// all tag data.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::Version::{V3, V4};
    ///
    /// let mut tag = id3v2::Tag::with_version(V4);
    /// assert_eq!(tag.version(), V4);
    ///
    /// tag.convert_version(V3);
    /// assert_eq!(tag.version(), V3);
    /// ```
    pub fn convert_version(&mut self, version: Version) {
        if self.version == version {
            return;
        }

        self.version = version;

        let mut remove = Vec::new();
        for frame in self.frames.iter_mut() {
            if !frame.convert_version(version) {
                remove.push(frame as *mut _ as *const _);
            }
        }

        self.frames.retain(|frame: &Frame| !remove.contains(&(frame as *const _)));
    }

    /// Returns a vector of references to all frames in the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new(Id::V4(*b"TPE1")));
    /// tag.add_frame(Frame::new(Id::V4(*b"APIC")));
    ///
    /// assert_eq!(tag.get_frames().len(), 2);
    /// ```
    #[inline]
    pub fn get_frames<'a>(&'a self) -> &'a Vec<Frame> {
        &self.frames
    }

    /// Get a tag's flags.
    #[inline]
    pub fn flags(&self) -> TagFlags {
        self.flags
    }

    /// Returns a reference to the first frame with the specified identifier.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new(Id::V4(*b"TIT2")));
    ///
    /// assert!(tag.get_frame_by_id(Id::V4(*b"TIT2")).is_some());
    /// assert!(tag.get_frame_by_id(Id::V4(*b"TCON")).is_none());
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
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new(Id::V4(*b"TXXX")));
    /// tag.add_frame(Frame::new(Id::V4(*b"TXXX")));
    /// tag.add_frame(Frame::new(Id::V4(*b"TALB")));
    ///
    /// assert_eq!(tag.get_frames_by_id(Id::V4(*b"TXXX")).len(), 2);
    /// assert_eq!(tag.get_frames_by_id(Id::V4(*b"TALB")).len(), 1);
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

    /// Adds a frame to the tag. The versions of the tag and frame must match.
    ///
    /// Returns TRUE after adding the frame if the versions matched, and
    /// returns FALSE and does nothing if not.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let id = Id::V4(*b"TALB");
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_frame(Frame::new(id));
    /// assert_eq!(tag.get_frames()[0].id, id);
    /// ```
    pub fn add_frame(&mut self, frame: Frame) -> bool {
        if frame.version() != self.version() {
            return false;
        }
        self.frames.push(frame);
        true
    }

    /// Adds a text frame with the given ID and a UTF-8 string as content.
    /// Returns whether the frame successfully created.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Id;
    ///
    /// let id = Id::V4(*b"TCON");
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_text_frame(id, "Metal");
    /// assert_eq!(tag.text_frame_text(id).unwrap(), "Metal");
    /// ```
    #[inline]
    pub fn add_text_frame(&mut self, id: frame::Id, text: &str) -> bool {
        match Frame::new_text_frame(id, text, Encoding::UTF8) {
            Some(frame) => {
                self.remove_frames_by_id(id);
                self.frames.push(frame);
                true
            }
            None => false,
        }
    }

    /// Adds a text frame with the given contents, which will be transcoded from
    /// UTF-8 to the specified encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Id;
    /// use id3::id3v2::frame::Encoding::UTF16;
    ///
    /// let id = Id::V4(*b"TRCK");
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_text_frame_enc(id, "1/13", UTF16);
    /// assert_eq!(tag.text_frame_text(id).unwrap(), "1/13");
    /// ```

    /* TODO(sp3d): find a more type-safe way to encode this
    as formulated, there are lots of errors that can be made:
    incompatible version+encoding, lossy transcoding into Latin-1, non-text IDs
    some of these should be preventable in the typesystem
    or handled explicitly as behavior option arguments for encoding*/
    pub fn add_text_frame_enc(&mut self, id: frame::Id, text: &str, encoding: Encoding) {
        self.remove_frames_by_id(id);
        let frame = Frame::new_text_frame(id, text, encoding).expect("ID is not a text frame!");
        self.frames.push(frame);
    }

    /// Removes all frames with the specified identifier.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_frame(Frame::new(Id::V4(*b"TXXX")));
    /// tag.add_frame(Frame::new(Id::V4(*b"TXXX")));
    /// tag.add_frame(Frame::new(Id::V4(*b"USLT")));
    ///
    /// assert_eq!(tag.get_frames().len(), 3);
    ///
    /// tag.remove_frames_by_id(Id::V4(*b"TXXX"));
    /// assert_eq!(tag.get_frames().len(), 1);
    ///
    /// tag.remove_frames_by_id(Id::V4(*b"USLT"));
    /// assert_eq!(tag.get_frames().len(), 0);
    /// ```
    pub fn remove_frames_by_id(&mut self, id: frame::Id) {
        self.frames.retain(|frame| {
            frame.id != id
        });
    }

    /// Returns the content of the first text frame with the specified identifier,
    /// converted to UTF8, or `None` if the frame with the specified ID does not
    /// exist or does not have textual content.
    pub fn text_frame_text(&self, id: frame::Id) -> Option<String> {
        match self.get_frame_by_id(id) {
            Some(frame) => match &*frame.fields {
                [Field::TextEncoding(encoding), Field::String(ref text)] => util::string_from_encoding(encoding, &text),
                _ => None
            },
            None => None
        }
    }
}
