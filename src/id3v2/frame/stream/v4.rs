extern crate flate;

use id3v2::frame::stream::FrameStream;
use id3v2::frame::{Frame, Id};
use audiotag::{TagResult, TagError};
use audiotag::ErrorKind::UnsupportedFeatureError;
use std::io::{Read, Write};
use util;

pub struct FrameV4;
impl FrameStream for FrameV4 {
    fn read(reader: &mut Read, _: Option<FrameV4>) -> TagResult<Option<(u32, Frame)>> {
        let id = id_or_padding!(reader, 4);
        debug!("reading {:?}", id); 

        let mut frame = Frame::new(Id::V4(id));

        let content_size = util::unsynchsafe(read_be_u32!(reader));

        let frameflags = read_be_u16!(reader);
        frame.flags.tag_alter_preservation = frameflags & 0x4000 != 0;
        frame.flags.file_alter_preservation = frameflags & 0x2000 != 0;
        frame.flags.read_only = frameflags & 0x1000 != 0;
        frame.flags.grouping_identity = frameflags & 0x40 != 0;
        frame.flags.compression = frameflags & 0x08 != 0;
        frame.flags.encryption = frameflags & 0x04 != 0;
        frame.flags.unsynchronization = frameflags & 0x02 != 0;
        frame.flags.data_length_indicator = frameflags & 0x01 != 0;

        if frame.flags.encryption {
            debug!("[{:?}] encryption is not supported", frame.id);
            return Err(TagError::new(UnsupportedFeatureError, "encryption is not supported"));
        } else if frame.flags.grouping_identity {
            debug!("[{:?}] grouping identity is not supported", frame.id);
            return Err(TagError::new(UnsupportedFeatureError, "grouping identity is not supported"));
        } else if frame.flags.unsynchronization {
            debug!("[{:?}] unsynchronization is not supported", frame.id);
            return Err(TagError::new(UnsupportedFeatureError, "unsynchronization is not supported"));
        }

        let mut read_size = content_size;
        if frame.flags.data_length_indicator {
            let _decompressed_size = util::unsynchsafe(read_be_u32!(reader));
            read_size -= 4;
        }

        let mut data = vec![0; read_size as usize]; try!(reader.read(&mut *data));
        frame.fields = try!(frame.parse_fields(&*data));

        Ok(Some((10 + content_size, frame)))
    }

    fn write(writer: &mut Write, frame: &Frame, _: Option<FrameV4>) -> TagResult<u32> {
        let mut content_bytes = frame.fields_to_bytes();
        let mut content_size = content_bytes.len() as u32;
        let decompressed_size = content_size;

        if frame.flags.compression {
            debug!("[{:?}] compressing frame content", frame.id);
            content_bytes = flate::deflate_bytes_zlib(&*content_bytes).to_vec();
            content_size = content_bytes.len() as u32;
        }

        if frame.flags.data_length_indicator {
            content_size += 4;
        }

        if let Id::V4(id_bytes)=frame.id {
            try!(writer.write(&id_bytes));
        } else {
            panic!("internal error: writing v2.4 frame but frame ID is not v2.4!");
        }
        try!(writer.write(&util::u32_to_bytes(util::synchsafe(content_size))));
        try!(writer.write(&frame.flags.to_bytes(0x4)));
        if frame.flags.data_length_indicator {
            debug!("[{:?}] adding data length indicator", frame.id);
            try!(writer.write(&util::u32_to_bytes(util::synchsafe(decompressed_size))));
        }
        try!(writer.write(&*content_bytes));

        Ok(10 + content_size)
    }
}

