extern crate std;

use std::io::{self, Read, Write, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

use id3v1;
use id3v2;

static DEFAULT_FILE_DISCARD: [&'static [u8]; 11] = [
    b"AENC", b"ETCO", b"EQUA", b"MLLT", b"POSS",
    b"SYLT", b"SYTC", b"RVAD", b"TENC", b"TLEN", b"TSIZ"
];
static PADDING_BYTES: u32 = 2048;

//enum Chunk()

/// Represents a set of ID3v1 and/or ID3v2 tags associated with particular file on disk.
pub struct TaggedFile<'a, R: 'a> {
    /// The ID3v1 tag (combined with ID3v1.1 and Extended ID3v1 data) stored in the file, if any.
    pub v1: Option<id3v1::Tag>,
    /// The ID3v2 tag stored at the file's start, if any. Does not describe tags which start midway through the file, as in streams.
    pub v2: Option<id3v2::Tag>,
    /// The range in the file in which audio or other ID3-wrapped data is stored
    pub data_bounds: std::ops::Range<u64>,
    ///
    pub data_reader: &'a mut R,
}

impl<'a, R> TaggedFile<'a, R>
where R: Read+Seek
{
    /// Reads a TaggedFile from a seekable reader.
    pub fn from_seekable(reader: &'a mut R) -> Result<Self, io::Error> {
        let v2 = try!(id3v2::read_tag(reader));
        let audio_start = match v2
        {
            Some(ref _tag) => try!(reader.seek(SeekFrom::Current(0))),
            None => 0,
        };

        let v1_offset = try!(reader.seek(SeekFrom::End(-id3v1::TAG_OFFSET)));
        let audio_end = if try!(id3v1::probe_tag(reader)) {
            let xtag_offset = try!(reader.seek(SeekFrom::End(-id3v1::TAGPLUS_OFFSET)));
            if try!(id3v1::probe_xtag(reader))
            {
                xtag_offset
            } else {
                v1_offset
            }
        } else {
            try!(reader.seek(SeekFrom::End(0)))
        };

        let v1 = try!(id3v1::read_seek(reader, true));
        Ok(TaggedFile {v1: v1, v2: v2, data_bounds: audio_start..audio_end, data_reader: reader})
    }

    /// Returns whether a reader may have an ID3v2 tag at its current location.
    /// Advances the reader by 3 bytes.
    pub fn is_candidate(reader: &mut Read) -> bool {
        let mut identifier = [0u8; 3];
        drop(reader.read(&mut identifier));
        identifier == *b"ID3"
    }

    /// Write a TaggedFile to a writer. This does not presently place the v1 tag after the audio data.
    /// 
    pub fn write_to(&mut self, writer: &mut Write, unsynchronization: bool) -> Result<usize, io::Error> {
        let v: Result<usize, io::Error> =
        if let Some(ref mut id3v2) = self.v2 {
            // remove frames which have the flags indicating they should be removed
            id3v2.frames.retain(|frame| {
                !(frame.tag_alter_preservation()
                      || frame.file_alter_preservation()
                      || DEFAULT_FILE_DISCARD.contains(&frame.id.name()))
            });

            // write id3v2 tag
            let mut bytes_written: usize = try!(id3v2.write_to(writer, unsynchronization)) as usize;

            // write padding
            bytes_written += try!(writer.write(&*vec![0; PADDING_BYTES as usize]));
            Ok(bytes_written)
        }
        else
        {
            Ok(0usize)
        };

        //TODO(sp3d): implement:
        //grow file (if necessary) to padded_v2_size+old.data_bounds.size+v1_size
        //move reader[old.data_bounds][..] to [padded_v2_size..]
        //shrink file (if necessary) to padded_v2_size+old.data_bounds.size+v1_size
        //write v2 into file
        //write v1 into file
        unimplemented!()
    }

    /*/// Reads any present ID3v1 and ID3v2 tags from the file at a path.
    /// 
    /// Note that only ID3v2 tags at the start of the file and ID3v1 tags at its
    /// end will be found.
    pub fn from_path(path: &Path) -> Result<TaggedFile<'a, ::std::fs::File>, io::Error> {
        let mut file = try!(File::open(path));
        let tag = try!(TaggedFile::from_seekable(&mut file));
        Ok(tag)
    }*/

    /// Stores data wrapped by ID3v1 and ID3v2 tags in a file at the given path.
    pub fn store_at_path(&self, path: &Path) -> Result<usize, io::Error>
    {
        let mut file = try!(File::open(path));
        let reader = &mut file;

        let ft = try!(TaggedFile::from_seekable(reader));
        unimplemented!()
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
