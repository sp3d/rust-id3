/// Text encodings used in ID3v2 frames.
#[deriving(Show, FromPrimitive, PartialEq, Copy)]
pub enum Encoding {
    /// ISO-8859-1 text encoding, also referred to as Latin-1 encoding.
    Latin1,
    /// UTF-16 text encoding with a byte order mark.
    UTF16,
    /// UTF-16BE text encoding without a byte order mark. This encoding is only used in id3v2.4.
    UTF16BE,
    /// UTF-8 text encoding. This encoding is only used in id3v2.4.
    UTF8 
}
