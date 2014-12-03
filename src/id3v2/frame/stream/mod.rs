use id3v2::frame::Frame;
use audiotag::TagResult;
use std::io::{Read, Write};

pub use self::v2::FrameV2;
pub use self::v3::FrameV3;
pub use self::v4::FrameV4;

macro_rules! id_or_padding {
    ($reader:ident, $n:expr) => {
        {
            let mut buf = [0; $n];
            try!($reader.read(&mut buf[0..1]));
            if buf[0] == 0 { // padding
                return Ok(None);
            }
            try!($reader.read(&mut buf[1..]));
            buf
        }
    };

}

/// A trait for reading and writing frames.
pub trait FrameStream {
    /// Returns a tuple containing the number of bytes read and a frame. If pading is encountered
    /// then `None` is returned.
    fn read(reader: &mut Read, _: Option<Self>) -> TagResult<Option<(u32, Frame)>>;

    /// Attempts to write the frame to the writer.
    fn write(writer: &mut Write, frame: &Frame, _: Option<Self>) -> TagResult<u32>;
}

mod v2;
mod v3;
mod v4;
