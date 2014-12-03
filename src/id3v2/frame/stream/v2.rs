use id3v2::frame::stream::FrameStream;
use id3v2::frame::{Frame, Id};
use audiotag::TagResult;
use util;

pub struct FrameV2;
impl FrameStream for FrameV2 {
    fn read(reader: &mut Reader, _: Option<FrameV2>) -> TagResult<Option<(u32, Frame)>> {
        let id = id_or_padding!(reader, 3);
        debug!("reading {}", id); 

        let mut frame = Frame::new(Id::V2(id));

        let sizebytes = try!(reader.read_exact(3));
        let read_size = (sizebytes[0] as u32 << 16) | (sizebytes[1] as u32 << 8) | sizebytes[2] as u32;

        let data = try!(reader.read_exact(read_size as uint));
        frame.fields = try!(frame.parse_fields(data.as_slice()));

        Ok(Some((6 + read_size, frame)))
    }

    fn write(writer: &mut Writer, frame: &Frame, _: Option<FrameV2>) -> TagResult<u32> {
        let content_bytes = frame.fields_to_bytes();
        let content_size = content_bytes.len() as u32;

        if let Id::V2(id_bytes)=frame.id {
            try!(writer.write(id_bytes.as_slice()));
        } else {
            panic!("internal error: writing v2.2 frame but frame ID is not v2.2!");
        }

        try!(writer.write(util::u32_to_bytes(content_size).slice(1, 4)));
        try!(writer.write(content_bytes.as_slice()));

        Ok(6 + content_size)
    }
}
