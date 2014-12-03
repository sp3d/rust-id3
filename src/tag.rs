extern crate std;

use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use audiotag::{AudioTag, TagResult};

use id3v1;
use id3v2;
use id3v2::frame::Frame;
use util;

static DEFAULT_FILE_DISCARD: [&'static str; 11] = [
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
    pub path: Option<PathBuf>,
}

impl FileTags
{
    /// Create a FileTags structure from pre-parsed tags
    pub fn from_tags(v1: Option<id3v1::Tag>, v2: Option<id3v2::Tag>) -> FileTags
    {
        FileTags {v1: v1, v2: v2, path: None}
    }

    /// Reads a FileTags from a reader that can be seeked.
    pub fn from_seekable<R: Read+Seek>(reader: &mut R) -> io::Result<FileTags> {
        let v2 = id3v2::read_tag(reader).ok();
        drop(reader.seek(SeekFrom::End(-id3v1::TAG_OFFSET)));
        let mut v1=id3v1::read_tag(reader).ok().unwrap_or(None);
        if let Some(ref mut v1) = v1 {
            try!(reader.seek(SeekFrom::End(-id3v1::TAGPLUS_OFFSET)));
            try!(id3v1::read_xtag(reader, v1));
        }
        Ok(FileTags {v1: v1, v2: v2, path: None})
    }

    /// Returns whether a reader may have an ID3v2 tag at its current location.
    /// Advances the reader by 3 bytes.
    pub fn is_candidate(reader: &mut Read, _: Option<FileTags>) -> bool {
        let mut identifier = [0u8; 3];
        drop(reader.read(&mut identifier));
        identifier == *b"ID3"
    }

    /// Reads a FileTags from a reader.
    pub fn read_from<R: Read>(reader: &mut R) -> TagResult<FileTags> {
        let v2 = id3v2::read_tag(reader).ok();
        //TODO(sp3d): read the v1 tag from the right place
        let v1 = id3v1::read_tag(reader).unwrap_or(None);
        Ok(FileTags {v1: v1, v2: v2, path: None})
    }

    /// Write a FileTags to a writer. This does not presently place the v1 tag after the audio data.
    pub fn write_to(&mut self, writer: &mut Write) -> TagResult<()> {
        // remove frames which have the flags indicating they should be removed 
        match self.v2 {
            Some(ref mut id3v2) => {
                id3v2.frames.retain(|frame| {
                    !(frame.offset != 0 
                      && (frame.tag_alter_preservation() 
                          || (frame.file_alter_preservation() 
                                  || DEFAULT_FILE_DISCARD.contains(&std::str::from_utf8(frame.id.name()).ok().unwrap()))))
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
                try!(writer.write(&id3v2.version.to_bytes())); 
                try!(writer.write(&[id3v2.flags.to_byte()]));
                try!(writer.write(&util::u32_to_bytes(u32::to_be(util::synchsafe(id3v2.size)))));

                let mut bytes_written = 10;

                for frame in id3v2.frames.iter_mut() {
                    debug!("writing {:?}", frame.id);

                    frame.offset = bytes_written;

                    bytes_written += match data_cache.get(&(frame as *mut _ as *const _)) {
                        Some(data) => { 
                            try!(writer.write(&*data));
                            data.len() as u32
                        },
                        None => try!(frame.write_to(writer))
                    }
                }

                id3v2.offset = bytes_written;
                id3v2.modified_offset = id3v2.offset;

                // write padding
                try!(writer.write(&*vec![0; PADDING_BYTES as usize]));
            },
            None => (),
        }
        Ok(())
    }

    /// Reads a FileTags from a given file, saving the file's path in the instance.
    pub fn read_from_path(path: &Path) -> TagResult<FileTags> {
        let mut file = try!(File::open(path));
        let mut tag = try!(FileTags::read_from(&mut file));
        tag.path=Some(path.to_owned());
        Ok(tag)
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
