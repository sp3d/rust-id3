extern crate std;

use std::io::{self, Read, Write, Seek};
use std::fs::File;
use std::path::Path;

use id3v1;
use id3v2;

static DEFAULT_FILE_DISCARD: [&'static [u8]; 11] = [
    b"AENC", b"ETCO", b"EQUA", b"MLLT", b"POSS",
    b"SYLT", b"SYTC", b"RVAD", b"TENC", b"TLEN", b"TSIZ"
];
static PADDING_BYTES: u32 = 2048;

/// Represents a set of ID3v1 and/or ID3v2 tags which might be associated with a
/// particular file on disk.
pub struct FileTags {
    /// The ID3v1 tag (combined with ID3v1.1 and Extended ID3v1 data) stored in the file, if any.
    pub v1: Option<id3v1::Tag>,
    /// The ID3v2 tag stored at the file's start, if any. Does not describe tags which start midway through the file, as in streams.
    pub v2: Option<id3v2::Tag>,
}

impl FileTags
{
    /// Create a FileTags structure from pre-parsed tags
    pub fn from_tags(v1: Option<id3v1::Tag>, v2: Option<id3v2::Tag>) -> FileTags
    {
        FileTags {v1: v1, v2: v2}
    }

    /// Reads a FileTags from a reader that can be seeked.
    pub fn from_seekable<R: Read+Seek>(reader: &mut R) -> Result<FileTags, io::Error> {
        let v2 = try!(id3v2::read_tag(reader));
        let v1 = try!(id3v1::read_seek(reader, true));
        Ok(FileTags {v1: v1, v2: v2})
    }

    /// Returns whether a reader may have an ID3v2 tag at its current location.
    /// Advances the reader by 3 bytes.
    pub fn is_candidate(reader: &mut Read, _: Option<FileTags>) -> bool {
        let mut identifier = [0u8; 3];
        drop(reader.read(&mut identifier));
        identifier == *b"ID3"
    }

    /// Write a FileTags to a writer. This does not presently place the v1 tag after the audio data.
    /// 
    pub fn write_to(&mut self, writer: &mut Write) -> Result<usize, io::Error> {
        if let Some(ref mut id3v2) = self.v2 {
            // remove frames which have the flags indicating they should be removed
            id3v2.frames.retain(|frame| {
                !(frame.tag_alter_preservation()
                      || frame.file_alter_preservation()
                      || DEFAULT_FILE_DISCARD.contains(&frame.id.name()))
            });

            // write id3v2 tag
            let mut bytes_written = try!(id3v2.write_to(writer)) as usize;

            // write padding
            bytes_written += try!(writer.write(&*vec![0; PADDING_BYTES as usize]));
            Ok(bytes_written)
        }
        else
        {
            Ok(0)
        }
    }

    /// Reads any present ID3v1 and ID3v2 tags from the file at a path.
    /// 
    /// Note that only ID3v2 tags at the start of the file and ID3v1 tags at its
    /// end will be found.
    pub fn from_path(path: &Path) -> Result<FileTags, io::Error> {
        let mut file = try!(File::open(path));
        let tag = try!(FileTags::from_seekable(&mut file));
        Ok(tag)
    }

    /// Stores ID3v1 and ID3v2 tags in the file at a path, removing any old
    /// ID3v2 tag found at the start of the file and any old ID3v1 tag found at
    /// the end of the file.
    pub fn store_at_path(&self, path: &Path) -> Result<usize, io::Error>
    {
        // byte range initially occupied by audio in the file
        //let audio_extent: Range<usize> = 
        //TODO(sp3d): implement
        Ok(0)
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
