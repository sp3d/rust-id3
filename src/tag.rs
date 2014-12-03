extern crate std;

use std::io::{File, SeekSet, SeekCur};
use std::collections::HashMap;

use audiotag::{AudioTag, TagError, TagResult};
use audiotag::ErrorKind::{InvalidInputError, UnsupportedFeatureError};

use id3v1;
use id3v2;
use id3v2::frame::{Id, Frame, Encoding, PictureType};
use util;

static DEFAULT_FILE_DISCARD: [&'static str, ..11] = [
    "AENC", "ETCO", "EQUA", "MLLT", "POSS", 
    "SYLT", "SYTC", "RVAD", "TENC", "TLEN", "TSIZ"
];
static PADDING_BYTES: u32 = 2048;

/// Represents a file on disk which may have an ID3v1 and/or ID3v2 tag in the standard locations.
pub struct FileTags {
    /// The ID3v1 tag (combined with ID3v1.1 and Extended ID3v1 data) stored in the file, if any.
    pub v1: Option<id3v1::Tag>,
    /// The ID3v2 tag stored at the file's start, if any. Does not describe tags which start midway through the file, as in streams.
    pub v2: Option<id3v2::Tag>,
    /// The path, if any, that this file was loaded from.
    pub path: Option<Path>,
    /// Indicates if the path that we are writing to is not the same as the path we read from.
    path_changed: bool,
    /// Indicates if when writing, an ID3v1 tag should be removed.
    remove_v1: bool
}

impl FileTags
{
    /// Create a FileTags structure from pre-parsed tags
    pub fn from_tags(v1: Option<id3v1::Tag>, v2: Option<id3v2::Tag>) -> FileTags
    {
        FileTags {v1: v1, v2: v2, path: None, path_changed: false, remove_v1: false}
    }
}

impl AudioTag for FileTags {
    // Reading/Writing {{{
    fn skip_metadata<R: Reader + Seek>(reader: &mut R, _: Option<FileTags>) -> Vec<u8> {
        macro_rules! try_io {
            ($reader:ident, $action:expr) => {
                match $action { 
                    Ok(bytes) => bytes, 
                    Err(_) => {
                        match $reader.seek(0, SeekSet) {
                            Ok(_) => {
                                match $reader.read_to_end() {
                                    Ok(bytes) => return bytes,
                                    Err(_) => return Vec::new()
                                }
                            },
                            Err(_) => return Vec::new()
                        }
                    }
                }
            }
        }

        let ident = try_io!(reader, reader.read_exact(3));
        if ident.as_slice() == b"ID3" {
            try_io!(reader, reader.seek(3, SeekCur));
            let offset = 10 + util::unsynchsafe(try_io!(reader, reader.read_be_u32()));   
            try_io!(reader, reader.seek(offset as i64, SeekSet));
        } else {
            try_io!(reader, reader.seek(0, SeekSet));
        }

        try_io!(reader, reader.read_to_end())
    }

    fn is_candidate(reader: &mut Reader, _: Option<FileTags>) -> bool {
        macro_rules! try_or_false {
            ($action:expr) => {
                match $action { 
                    Ok(result) => result, 
                    Err(_) => return false 
                }
            }
        }

        (try_or_false!(reader.read_exact(3))).as_slice() == b"ID3"
    }

    fn read_from(reader: &mut Reader) -> TagResult<FileTags> {
        use id3v2::TagFlag::*;
        let mut tag = id3v2::Tag::new();

        let identifier = try!(reader.read_exact(3));
        if identifier.as_slice() != b"ID3" {
            debug!("no ID3 tag found");
            return Err(TagError::new(InvalidInputError, "buffer does not contain an ID3 tag"))
        }

        let mut version_bytes = [0u8, ..2];
        try!(reader.read(&mut version_bytes));

        debug!("tag version {}", version_bytes);

        tag.version = match version_bytes.as_slice() {
            [2, 0] => id3v2::Version::V2,
            [3, 0] => id3v2::Version::V3,
            [4, 0] => id3v2::Version::V4,
            _ => return Err(TagError::new(InvalidInputError, "unsupported ID3 tag version")),
        };

        tag.flags = id3v2::TagFlags::from_byte(try!(reader.read_byte()), tag.version());

        if tag.flags.get(Unsynchronization) {
            debug!("unsynchronization is unsupported");
            return Err(TagError::new(UnsupportedFeatureError, "unsynchronization is not supported"))
        } else if tag.flags.get(Compression) {
            debug!("ID3v2.2 compression is unsupported");
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

        Ok(FileTags {v1: None, v2: Some(tag), path: None, path_changed: false, remove_v1: false, })
    }

    fn write_to(&mut self, writer: &mut Writer) -> TagResult<()> {
        // remove frames which have the flags indicating they should be removed 
        match self.v2 {
            Some(ref mut id3v2) => {
                id3v2.frames.retain(|frame| {
                    !(frame.offset != 0 
                      && (frame.tag_alter_preservation() 
                          || (frame.file_alter_preservation() 
                                  || DEFAULT_FILE_DISCARD.contains(&&*frame.id.to_string()))))
                });

                let mut data_cache: HashMap<*const Frame, Vec<u8>> = HashMap::new();
                let mut size = 0;

                for frame in id3v2.frames.iter() {
                    let mut frame_writer = Vec::new();
                    size += try!(frame.write_to(&mut frame_writer));
                    data_cache.insert(frame as *const _, frame_writer);
                }

                id3v2.size = size + PADDING_BYTES;

                try!(writer.write(b"ID3"));
                try!(writer.write(id3v2.version.to_bytes().as_slice())); 
                try!(writer.write_u8(id3v2.flags.to_byte()));
                try!(writer.write_be_u32(util::synchsafe(id3v2.size)));

                let mut bytes_written = 10;

                for frame in id3v2.frames.iter_mut() {
                    debug!("writing {}", frame.id);

                    frame.offset = bytes_written;

                    bytes_written += match data_cache.get(&(frame as *mut _ as *const _)) {
                        Some(data) => { 
                            try!(writer.write(data.as_slice()));
                            data.len() as u32
                        },
                        None => try!(frame.write_to(writer))
                    }
                }

                id3v2.offset = bytes_written;
                id3v2.modified_offset = id3v2.offset;

                // write padding
                for _ in range(0, PADDING_BYTES) {
                    try!(writer.write_u8(0));
                }
            },
            None => (),
        }
        Ok(())
    }

    fn read_from_path(path: &Path) -> TagResult<FileTags> {
        let mut file = try!(File::open(path));
        let mut tag = try!(AudioTag::read_from(&mut file));
        tag.path=Some(path.clone());
        Ok(tag)
    }

    fn write_to_path(&mut self, path: &Path) -> TagResult<()> {
/*        let data_opt = {
            match File::open(path) {
                Ok(mut file) => {
                    // remove the ID3v1 tag if the remove_v1 flag is set
                    let remove_bytes = if self.remove_v1 {
                        if try!(id3v1::probe_xtag(&mut file)) {
                            Some(id3v1::TAGPLUS_OFFSET as uint)
                        } else if try!(id3v1::probe_tag(&mut file)) {
                            Some(id3v1::TAG_OFFSET as uint)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut data = AudioTag::skip_metadata(&mut file, None::<id3v2::Tag>);
                    match remove_bytes {
                        Some(n) => if n <= data.len() {
                            data = data.slice_to(data.len() - n).to_vec();
                        },
                        None => {}
                    }
                    Some(data)
                }
                Err(_) => None
            }
        };

        let mut file = try!(File::open_mode(path, Truncate, Write));
        self.write_to(&mut file).unwrap();
        
        match data_opt {
            Some(data) => file.write(data.as_slice()).unwrap(),
            None => {}
        }
*/
        Ok(())
    }

    fn save(&mut self) -> TagResult<()> {
    /*
        if self.path.is_none() {
            panic!("attempted to save file which was not read from a path");
        }

        // remove any old frames that have the tag_alter_presevation flag
        let mut modified_offset = self.modified_offset;
        {
            let set_modified_offset = |offset: u32| {
                if offset != 0 {
                    modified_offset = min(modified_offset, offset);
                }
                false
            };       
            self.frames.retain(|frame| {
                frame.offset == 0 || !frame.tag_alter_preservation() 
                    || set_modified_offset(frame.offset)
            });
        }
        self.modified_offset = modified_offset;

        let mut data_cache: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let mut size = 0;

        for frame in self.frames.iter() {
            let mut frame_writer = Vec::new();
            size += try!(frame.write_to(&mut frame_writer));
            data_cache.insert(frame.uuid.clone(), frame_writer);
        }

        debug!("modified offset: {}", self.modified_offset); 
       
        if size <= self.size && self.modified_offset >= 10 {
            debug!("writing using padding");

            let mut writer = try!(File::open_mode(self.path.as_ref().unwrap(), Open, Write));

            let mut offset = self.modified_offset;
            try!(writer.seek(offset as i64, SeekSet));

            for frame in self.frames.iter_mut() {
                if frame.offset == 0 || frame.offset > self.modified_offset {
                    debug!("writing {}", frame.id);
                    frame.offset = offset;
                    offset += match data_cache.get(&frame.uuid) {
                        Some(data) => { 
                            try!(writer.write(data.as_slice()));
                            data.len() as u32
                        },
                        None => try!(frame.write_to(&mut writer))
                    }
                }
            }

            if self.offset > offset {
                for _ in range(offset, self.offset) {
                    try!(writer.write_u8(0));
                }
            }

            self.offset = offset;
            self.modified_offset = self.offset;
            Ok(())
        } else {
            debug!("rewriting file");
            let path = self.path.clone().unwrap();
            self.write_to_path(&path)
        }
*/
        Ok(())
    }
    //}}}
    
    #[inline]
    fn artist(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| x.text_frame_text(x.version().artist_id()))
    }

    #[inline]
    fn set_artist<T: StrAllocating>(&mut self, artist: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_artist_enc(artist.as_slice(), encoding);
        }
    }

    #[inline]
    fn remove_artist(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().artist_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn album_artist(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| {
            x.text_frame_text(x.version().album_artist_id())
        })
    }

    #[inline]
    fn set_album_artist<T: StrAllocating>(&mut self, album_artist: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_album_artist_enc(album_artist.as_slice(), encoding);
        }
    }

    #[inline]
    fn remove_album_artist(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().album_artist_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn album(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| {
            x.text_frame_text(x.version().album_id())
        })
    }

    fn set_album<T: StrAllocating>(&mut self, album: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_album_enc(album.as_slice(), encoding);
        }
    }

    #[inline]
    fn remove_album(&mut self) {
        if let Some(ref mut x)=self.v2 {
            x.remove_frames_by_id(Id::V3(b!("TSOP")));
            x.remove_frames_by_id(Id::V4(b!("TSOP")));
            let id = x.version().album_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn title(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| {
            x.text_frame_text(x.version().title_id())
        })
    }

    #[inline]
    fn set_title<T: StrAllocating>(&mut self, title: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_title_enc(title.as_slice(), encoding);
        }
    }

    #[inline]
    fn remove_title(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().title_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn genre(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| {
            x.text_frame_text(x.version().genre_id())
        })
    }

    #[inline]
    fn set_genre<T: StrAllocating>(&mut self, genre: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_genre_enc(genre.as_slice(), encoding);
        }
    }

    #[inline]
    fn remove_genre(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().genre_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn track(&self) -> Option<u32> {
        self.v2.as_ref().and_then(|x| {
            x.track_pair().and_then(|(track, _)| Some(track))
        })
    }

    #[inline]
    fn set_track(&mut self, track: u32) {
        if let Some(ref mut x)=self.v2 {
            x.set_track_enc(track, Encoding::Latin1);
        }
    }

    #[inline]
    fn remove_track(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().track_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn total_tracks(&self) -> Option<u32> {
        self.v2.as_ref().and_then(|x| {
            x.track_pair().and_then(|(_, total_tracks)| total_tracks)
        })
    }

    #[inline]
    fn set_total_tracks(&mut self, total_tracks: u32) {
        if let Some(ref mut x)=self.v2 {
            x.set_total_tracks_enc(total_tracks, Encoding::Latin1);
        }
    }

    fn remove_total_tracks(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().track_id();
            match x.track_pair() {
                Some((track, _)) => {x.add_text_frame(id, format!("{}", track).as_slice());},
                None => {},
            }
        }
    }

    fn lyrics(&self) -> Option<String> {
        self.v2.as_ref().and_then(|x| {
            match x.get_frame_by_id(x.version().lyrics_id()) {
                Some(frame) => match frame.fields {
                    //TODO(sp3d): rebuild this on top of fields
                    //LyricsContent(ref lyrics) => Some(lyrics.text.clone()),
                    _ => None
                },
                None => None
            }
        })
    }

    #[inline]
    fn set_lyrics<T: StrAllocating>(&mut self, text: T) {
        if let Some(ref mut x)=self.v2 {
            let encoding = x.version().default_encoding();
            x.set_lyrics_enc("eng", text.as_slice(), "", encoding);
        }
    }

    #[inline]
    fn remove_lyrics(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().lyrics_id();
            x.remove_frames_by_id(id);
        }
    }

    #[inline]
    fn set_picture<T: StrAllocating>(&mut self, mime_type: T, data: Vec<u8>) {
        self.remove_picture();
        if let Some(ref mut x)=self.v2 {
            x.add_picture(mime_type.as_slice(), PictureType::Other, data);
        }
    }

    #[inline]
    fn remove_picture(&mut self) {
        if let Some(ref mut x)=self.v2 {
            let id = x.version().picture_id();
            x.remove_frames_by_id(id);
        }
    }

    fn all_metadata(&self) -> Vec<(String, String)> {
        let mut metadata = Vec::new();
        if let Some(ref x)=self.v2 {
            for frame in x.frames.iter() {
                match frame.text() {
                    Some(text) => metadata.push((frame.id.to_string(), text)),
                    None => {}
                }
            }
        }
        metadata
    }
}
// }}}

// Tests {{{
#[cfg(test)]
mod tests {
    use id3v2::TagFlags;
    use id3v2::TagFlag::*;
    use id3v2::Version::*;

    #[test]
    fn test_flags_to_bytes() {
        let mut flags = TagFlags::new(V4);
        assert_eq!(flags.to_byte(), 0x0);
        flags.set(Unsynchronization, true);
        flags.set(ExtendedHeader, true);
        flags.set(Experimental, true);
        flags.set(Footer, true);
        assert_eq!(flags.to_byte(), 0xF0);
    }
}
// }}}
