use frame::Frame;
use audiotag::TagResult;

pub use self::v2::FrameV2;
pub use self::v3::FrameV3;
pub use self::v4::FrameV4;

macro_rules! id_or_padding {
    ($reader:ident, $n:expr) => {
        {
            let mut buf = [0, ..$n];
            let byte = try!($reader.read_byte());
            if byte == 0 { // padding
                return Ok(None);
            }
            buf[0] = byte;
            try!($reader.read_at_least($n - 1, buf.slice_from_mut(1)));
            //try_string!(buf)
            buf
        }
    };

}

/// A trait for reading and writing frames.
pub trait FrameStream {
    /// Returns a tuple containing the number of bytes read and a frame. If pading is encountered
    /// then `None` is returned.
    fn read(reader: &mut Reader, _: Option<Self>) -> TagResult<Option<(u32, Frame)>>;

    /// Attempts to write the frame to the writer.
    fn write(writer: &mut Writer, frame: &Frame, _: Option<Self>) -> TagResult<u32>;
}

mod v2;
mod v3;
mod v4;
