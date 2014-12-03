use id3v2::frame::stream::FrameStream;
use id3v2::frame::{Frame, Id};
use id3v2::Error;
use std::io::{self, Read, Write};
use util;

pub struct FrameV2;
impl FrameStream for FrameV2 {
    fn read(reader: &mut Read, _: Option<FrameV2>) -> Result<Option<(u32, Frame)>, Error> {
        let id = id_or_padding!(reader, 3);
        debug!("reading {:?}", id); 

        let mut frame = Frame::new(Id::V2(id));

        let mut sizebytes = [0u8; 3]; read_all!(reader, &mut sizebytes);
        let read_size = ((sizebytes[0] as u32) << 16) | ((sizebytes[1] as u32) << 8) | sizebytes[2] as u32;

        let mut data = vec![0; read_size as usize]; read_all!(reader, &mut *data);
        frame.fields = try!(frame.parse_fields(&*data));

        Ok(Some((6 + read_size, frame)))
    }

    fn write(writer: &mut Write, frame: &Frame, _: Option<FrameV2>) -> Result<u32, io::Error> {
        let content_bytes = frame.fields_to_bytes();
        let content_size = content_bytes.len() as u32;

        if let Id::V2(id_bytes)=frame.id {
            try!(writer.write(&id_bytes));
        } else {
            panic!("internal error: writing v2.2 frame but frame ID is not v2.2!");
        }

        try!(writer.write(&util::u32_to_bytes(content_size)[1..]));
        try!(writer.write(&content_bytes));

        Ok(6 + content_size)
    }
}
