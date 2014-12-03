/// Text encodings used in ID3v2 frames.
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Encoding {
    /// ISO-8859-1 text encoding, also referred to as Latin-1 encoding.
    Latin1 = 0,
    /// UTF-16 text encoding with a byte order mark.
    UTF16 = 1,
    /// UTF-16BE text encoding without a byte order mark. This encoding is only used in id3v2.4.
    UTF16BE = 2,
    /// UTF-8 text encoding. This encoding is only used in id3v2.4.
    UTF8 = 3,
}

impl Encoding
{
    /// Returns the encoding specified by the given byte value in an encoding field of 
    /// an ID3v2 frame, if any.
    pub fn from_u8(n: u8) -> Option<Encoding>
    {
        match n
        {
            0 => Some(Encoding::Latin1),
            1 => Some(Encoding::UTF16),
            2 => Some(Encoding::UTF16BE),
            3 => Some(Encoding::UTF8),
            _ => None,
        }
    }
}
