extern crate byteorder;
extern crate flate2;

use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use id3v2::frame::stream::FrameStream;
use id3v2::frame::{Frame, Id};
use id3v2::Error;
use id3v2::ErrorKind::{UnsupportedFeature, InvalidTag};
use std::io::{self, Read, Write};
use self::flate2::write::ZlibEncoder;
use util;

pub struct FrameV4;
impl FrameStream for FrameV4 {
    fn read(reader: &mut Read, _: Option<FrameV4>, unsynchronization: bool) -> Result<(u32, Option<Frame>), Error> {
        let id = id_or_padding!(reader, 4);
        debug!("reading {:?}", id); 

        let mut frame = Frame::new(Id::V4(id));

        let content_size = util::unsynchsafe(try!(reader.read_u32::<BigEndian>()));

        let frameflags = try!(reader.read_u16::<BigEndian>());
        frame.flags.tag_alter_preservation = frameflags & 0x4000 != 0;
        frame.flags.file_alter_preservation = frameflags & 0x2000 != 0;
        frame.flags.read_only = frameflags & 0x1000 != 0;
        frame.flags.grouping_identity = frameflags & 0x40 != 0;
        frame.flags.compression = frameflags & 0x08 != 0;
        frame.flags.encryption = frameflags & 0x04 != 0;
        frame.flags.unsynchronization = frameflags & 0x02 != 0;
        frame.flags.data_length_indicator = frameflags & 0x01 != 0;

        /*
        Frame flag order for ID3v2.4 is:
            h - Grouping identity
            k - Compression
            m - Encryption
            p - Data length indicator
        */

        if frame.flags.grouping_identity {
            frame.group_symbol = try!(reader.read_u8());
        }
        if frame.flags.compression {
            if !frame.flags.data_length_indicator {
                debug!("[{:?}] compression without data length indicator", frame.id);
                return Err(Error::new(InvalidTag, "compression specified but data length indicator bit not set"));
            }
        }
        if frame.flags.encryption {
            //TODO: add decryption hook
            debug!("[{:?}] encryption is not supported", frame.id);
            return Err(Error::new(UnsupportedFeature, "encryption is not supported"));
        }
        let mut read_size = content_size;
        if frame.flags.data_length_indicator {
            let _decompressed_size = util::unsynchsafe(try!(reader.read_u32::<BigEndian>()));
            read_size -= 4;
        }

        let mut data = vec![0; read_size as usize]; read_all!(reader, &mut *data);
        if frame.flags.unsynchronization {
            util::resynchronize(&mut data);
        }

        frame.fields = try!(frame.parse_fields(&*data));

        Ok((10 + content_size, Some(frame)))
    }

    fn write(writer: &mut Write, frame: &Frame, _: Option<FrameV4>, unsynchronization: bool) -> Result<u32, io::Error> {
        let mut content_bytes = frame.fields_to_bytes();
        let mut content_size = content_bytes.len() as u32;
        let decompressed_size = content_size;

        if frame.flags.compression {
            debug!("[{:?}] compressing frame content", frame.id);
            let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::Default);
            try!(encoder.write_all(&content_bytes[..]));
            content_bytes = try!(encoder.finish());
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
        if unsynchronization {
            util::unsynchronize(&mut content_bytes);
        }
        try!(writer.write(&*content_bytes));

        Ok(10 + content_size)
    }
}
