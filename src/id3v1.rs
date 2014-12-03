use std::io::{self, Read, Write, Seek, SeekFrom};
use std::iter::IntoIterator;
use num::Bounded;
use std::fmt;

/// The fields in an ID3v1 tag, including the "1.1" track number field.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum Fields {
    Title,
    Artist,
    Album,
    Year,
    Comment,
    Track,
    Genre,
}

impl Fields {
    fn length(&self) -> usize {
        LENGTHS[*self as usize] as usize
    }
}

static LENGTHS: &'static [i8]=&[30, 30, 30, 4, 30, -1, 1];

static TAG: &'static [u8] = b"TAG";
/// How far from the end of a file to probe for an ID3 tag signature.
pub static TAG_OFFSET: i64 = 128;

static TAGPLUS: &'static [u8] = b"TAG+";
/// How far from the end of a file to probe for an extended ID3 tag signature.
pub static TAGPLUS_OFFSET: i64 = 355;

static XLENGTHS: &'static [i8]=&[60, 60, 60, 30, 6, 6];

/// The fields in an extended ID3v1 tag.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum XFields {
    XTitle,
    XArtist,
    XAlbum,
    Speed,
    XGenre,
    Start,
    End,
}

impl XFields {
    fn length(&self) -> usize {
        XLENGTHS[*self as usize] as usize
    }
}

/// ID3v1's notion of a four-digit year.
#[derive(Debug, Copy, Clone)]
pub struct Year
{
    value: u16,
}

impl Year {
    fn value(&self) -> u16 {
        self.value
    }
    fn new(year: u16) -> Option<Year> {
        #![allow(deprecated)]
        let max: Year = Bounded::max_value();
        if year > max.value() {
            None
        } else {
            Some(Year {value: year})
        }
    }
}

impl Bounded for Year {
    #![allow(deprecated)]
    fn min_value() -> Year {
        Year {value: 0}
    }
    fn max_value() -> Year {
        Year {value: 9999}
    }
}

/// ID3v1 extended time tags--encoded in the format "mmm:ss", a valid value can be a maximum of 999m99s = 999*60+99 = 60039 seconds.
#[derive(Copy, Clone, Debug)]
pub struct Time
{
    value: u16,
}

impl Time {
    fn seconds(&self) -> u16 {
        self.value
    }
    fn new(seconds: u16) -> Option<Time> {
        #![allow(deprecated)]
        let max: Time = Bounded::max_value();
        if seconds > max.seconds() {
            None
        } else {
            Some(Time {value: seconds})
        }
    }
}

impl Bounded for Time {
    #![allow(deprecated)]
    fn min_value() -> Time {
        Time {value: 0}
    }
    fn max_value() -> Time {
        Time {value: 60039}
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:03}:{:02}", self.seconds()/60, self.seconds()%60)
    }
}

/// Parsed ID3v1 tag metadata.
#[derive(Debug)]
pub struct Tag {
    /// The full title (ID3v1 + extension if present).
    pub title: Vec<u8>,
    /// The full artist (ID3v1 + extension if present).
    pub artist: Vec<u8>,
    /// The full album (ID3v1 + extension if present).
    pub album: Vec<u8>,
    /// A 4-digit string, if we are lucky
    pub year: Year,
    /// A free-form comment.
    pub comment: Vec<u8>,
    /// Number of the track, 0 if not set. ID3v1.1 data.
    pub track: u8,
    /// The genre mapping is standardized up to 79, some extensions exist.
    /// http://eyed3.nicfit.net/plugins/genres_plugin.html
    pub genre: u8,
    /// 1 (slow), 2, 3, 4 (fast) or 0 (not set). ID3v1 extended data.
    pub speed: u8,
    /// Free-form genre string. ID3v1 extended data.
    pub genre_str: Vec<u8>,
    /// The real start of the track, mmm:ss. ID3v1 extended data.
    pub start_time: Time,
    /// The real end of the track, mmm:ss. ID3v1 extended data.
    pub end_time: Time,
}

fn write_zero_padded<W: Write>(writer: &mut W, data: &[u8], offset: usize, len: usize) -> Result<(), io::Error> {
    let start = ::std::cmp::min(offset, data.len());
    let actual_len = ::std::cmp::min(offset+len, data.len());
    try!(writer.write(&data[start..actual_len]));
    for _ in 0..(len-(actual_len-start)) {
        try!(writer.write(&[0]));
    }
    Ok(())
}

impl Tag {
    /// Create a new ID3v1 tag with no information.
    pub fn new() -> Tag {
        Tag {
            title: vec![], artist: vec![], album: vec![], year: Year::new(0).unwrap(), comment: vec![], track: 0,
            genre: 0, speed: 0, genre_str: vec![], start_time: Time::new(0).unwrap(), end_time: Time::new(0).unwrap()
        }
    }
    /// Returns whether the tag contains information which would be lost if the extended tag were not written.
    pub fn has_extended_data(&self) -> bool {
        use self::Fields::*;
        self.title.len() > Title.length() ||
        self.artist.len() > Artist.length() ||
        self.album.len() > Album.length() ||
        self.speed > 0 ||
        self.genre_str.len() > 0 ||
        self.start_time.seconds() > 0 ||
        self.end_time.seconds() > 0
    }
    /// Write the simple ID3 tag (128 bytes) into the given writer.
    /// If write_track_number is true, the comment field will be truncated to 28 bytes and the removed two bytes will be used for a NUL and the track number.
    pub fn write<W: Write>(&self, writer: &mut W, write_track_number: bool) -> Result<(), io::Error> {
        use self::Fields::*;
        try!(writer.write(TAG));
        try!(write_zero_padded(writer, &*self.title, 0, Title.length()));
        try!(write_zero_padded(writer, &*self.artist, 0, Artist.length()));
        try!(write_zero_padded(writer, &*self.album, 0, Album.length()));
        try!(write!(writer,"{:04}", self.year.value()));
        if write_track_number {
            try!(writer.write(&self.comment[..Comment.length()-2]));
            try!(writer.write(&[0]));
            try!(writer.write(&[self.track]));
        } else {
            try!(writer.write(&self.comment[..Comment.length()]));
        }
        try!(writer.write(&[self.genre]));
        Ok(())
    }
    /// Write the extended portion of an ID3v1 tag (227 bytes) into the given writer.
    pub fn write_extended<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        use self::Fields::*;
        use self::XFields::*;
        try!(write_zero_padded(writer, &*self.title, Title.length(), XTitle.length()));
        try!(write_zero_padded(writer, &*self.artist, Artist.length(), XArtist.length()));
        try!(write_zero_padded(writer, &*self.album, Album.length(), XAlbum.length()));
        try!(writer.write(&[self.speed]));
        try!(write_zero_padded(writer, &*self.genre_str, 0, XGenre.length()));
        try!(write!(writer,"{}", self.start_time));
        try!(write!(writer,"{}", self.end_time));
        Ok(())
    }
}

/// ID3v1 tag reading helpers.
trait ID3v1Helpers {
    /// Read `n` bytes starting at an offset from the end.
    fn read_from_end(&mut self, n:usize, offset:i64) -> Result<Vec<u8>, io::Error>;

    /// Read a null-terminated ISO-8859-1 string of size at most `n`, at an offset from the end.
    fn read_str(&mut self, n: usize, offset: i64) -> Result<String, io::Error>;
}

impl<R: Read + Seek> ID3v1Helpers for R {
    #[inline]
    fn read_from_end(&mut self, n: usize, offset:i64) -> Result<Vec<u8>, io::Error> {
        try!(self.seek(SeekFrom::End(-offset)));
        let mut v=Vec::with_capacity(n);
        self.read(&mut v).and(Ok(v))
    }

    #[inline]
    fn read_str(&mut self, n: usize, offset: i64) -> Result<String, io::Error> {
        self.read_from_end(n, offset).map(|vec| extract_nz_88591(vec))
    }
}

/// Checks for presence of the signature indicating an ID3v1 tag at the reader's current offset.
/// Consumes 3 bytes from the reader.
#[inline]
pub fn probe_tag<R: Read>(reader: &mut R) -> Result<bool, io::Error> {
    let mut x=&mut [0; 3/*TAG.len()*/];
    reader.read(x).and(Ok(TAG == x))
}

/// Checks for presence of the signature indicating an ID3v1 extended metadata tag at the reader's current offset.
/// Consumes 4 bytes from the reader.
#[inline]
pub fn probe_xtag<R: Read>(reader: &mut R) -> Result<bool, io::Error> {
    let mut x=&mut [0; 4/*TAGPLUS.len()*/];
    reader.read(x).and(Ok(TAGPLUS == x))
}

fn parse_year(s: &[u8]) -> Year {
    let zero = Year::new(0).unwrap();
    match ::std::str::from_utf8(s) {
        Ok(st) => {
            let mn: Option<u16> = str::parse(st).ok();
            let n = mn.unwrap_or(0);
            Year::new(n).unwrap_or(zero)
        },
        Err(_) => zero
    }
}

fn parse_time(s: &[u8]) -> Time {
    enum State {
        Seconds,
        Minutes,
        LeadingWhitespace,
    }

    let zero = Time::new(0).unwrap();

    let mut mult: u64=1;
    let mut seconds: u64=0;
    let mut state=State::Seconds;

    fn is_digit(s: u8) -> bool {
        s >= b'0' && s <= b'9'
    }
    fn value(s: u8) -> u8 {
        s-b'0'
    }
    for &i in s.iter().rev() {
        match state {
            State::Seconds =>
                if is_digit(i) {
                    seconds+=mult*value(i) as u64;
                    mult*=10;
                } else if i == b':' {
                    state=State::Minutes;
                    mult=60;
                } else {
                    return zero;
                },
            State::Minutes =>
                if is_digit(i) {
                    seconds+=mult*value(i) as u64;
                    mult*=60;
                } else if (i as char).is_whitespace() {
                    state=State::LeadingWhitespace;
                },
            State::LeadingWhitespace =>
                if (i as char).is_whitespace() {
                    continue
                } else {
                    return zero;
                },
        }
    }
    if seconds > 65535 {
        zero
    } else {
        Time::new(seconds as u16).unwrap_or(zero)
    }
}

/// Read an ID3v1 tag from a reader.
pub fn read_tag<R: Read>(reader: &mut R) -> Result<Tag, io::Error> {
    use self::Fields::*;

    let mut tag = Tag::new();
    // Try to read ID3v1 metadata.
    let has_tag = try!(probe_tag(reader));
    if has_tag {
        maybe_read!(reader, tag.title, Title.length());
        maybe_read!(reader, tag.artist, Artist.length());
        maybe_read!(reader, tag.album, Album.length());
        let mut year_str=vec![]; maybe_read!(reader, year_str, Year.length());
        tag.year=parse_year(&*year_str);
        maybe_read!(reader, tag.comment, Comment.length()-2);
        let track_guard_byte=read_u8!(reader);
        if track_guard_byte == 0 {
            tag.track=read_u8!(reader);
        } else {
            tag.comment.push(track_guard_byte);
            maybe_read!(reader, tag.comment, 1);
        }
        tag.genre=read_u8!(reader);
    }

    Ok(tag)
}

/// Read the extended portion of an extended ID3v1 tag from a reader, combining its data with a previously-read ID3v1 tag.
pub fn read_xtag<R: Read>(reader: &mut R, tag: &mut Tag) -> Result<(), io::Error> {
    use self::Fields::*;
    use self::XFields::*;

    // Try to read ID3v1 extended metadata.
    let has_xtag = try!(probe_xtag(reader));
    if has_xtag {
        maybe_read!(reader, tag.title, XTitle.length());
        maybe_read!(reader, tag.artist, XArtist.length());
        maybe_read!(reader, tag.album, XAlbum.length());
        tag.speed = read_u8!(reader);
        maybe_read!(reader, tag.genre_str, Genre.length());
        let mut start_str=vec![]; maybe_read!(reader, start_str, Start.length());
        tag.start_time=parse_time(&*start_str);
        let mut end_str=vec![]; maybe_read!(reader, end_str, End.length());
        tag.end_time=parse_time(&*end_str);
    }
    Ok(())
}

/// Read a string from a null-terminated ISO-8859-1 byte vector.
///
/// Read the whole vector if there is no null byte.
///
/// This function cannot fail, because UTF-8 is compatible with ISO-8859-1
/// at the code point level.
#[inline]
fn extract_nz_88591(s: Vec<u8>) -> String {
    // This works because the ISO 8859-1 code points match the unicode code
    // points. So,`c as char` will map correctly from ISO to unicode.
    s.into_iter().take_while(|&c| c!=0).map(|c| c as char).collect()
}

/// Remove trailing zeros from an &[u8].
pub fn truncate_zeros(mut s: &[u8]) -> &[u8] {
    while s.len() > 0 && s[s.len()-1] == 0 {
        s=&s[..s.len()-1]
    }
    s
}

/*pub fn read_seek<R: Read + Seek>(reader: &mut R) {
}*/

#[test]
fn smoke_test() {
    use std::path::Path;
    let mut f=::std::fs::File::open(&Path::new("test.mp3")).ok().expect("could not open `test.mp3`");
    f.seek(SeekFrom::End(-TAG_OFFSET));
    let mut tag=read_tag(&mut f).unwrap();
    println!("{:?}", tag);
    f.seek(SeekFrom::End(-TAGPLUS_OFFSET));
    read_xtag(&mut f, &mut tag);
    println!("{:?}", tag);
}
