extern crate std;

use phf;
use id3v2::frame::{Encoding, Id};
use std::mem::transmute;
use std::string;

/// Returns the converted to the given encoding. Characters which could not be
/// represented in the target encoding are replaced with U+FFFD or '?'.
pub fn encode_string(s: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        //TODO(sp3d): properly encode Latin1
        Encoding::Latin1 => s.to_ascii().to_vec().into_bytes(),
        Encoding::UTF8 => s.as_bytes().to_vec(),
        Encoding::UTF16 => string_to_utf16(s),
        Encoding::UTF16BE => string_to_utf16be(s) 
    }
}

/// Returns the synchsafe varaiant of a `u32` value.
#[inline]
pub fn synchsafe(n: u32) -> u32 {
    let mut x: u32 = n & 0x7F | (n & 0xFFFFFF80) << 1;
    x = x & 0x7FFF | (x & 0xFFFF8000) << 1;
    x = x & 0x7FFFFF | (x & 0xFF800000) << 1;
    x
}

/// Returns the unsynchsafe varaiant of a `u32` value.
#[inline]
pub fn unsynchsafe(n: u32) -> u32 {
    (n & 0xFF | (n & 0xFF00) >> 1 | (n & 0xFF0000) >> 2 | (n & 0xFF000000) >> 3)
}

/// Returns a vector representation of a `u32` value.
#[inline]
pub fn u32_to_bytes(n: u32) -> Vec<u8> {
    vec!(((n & 0xFF000000) >> 24) as u8, 
         ((n & 0xFF0000) >> 16) as u8, 
         ((n & 0xFF00) >> 8) as u8, 
         (n & 0xFF) as u8
        )
}

/// Returns a string created from the vector using the specified encoding.
/// Returns `None` if the vector is not a valid string of the specified
/// encoding type.
#[inline]
pub fn string_from_encoding(encoding: Encoding, data: &[u8]) -> Option<string::String> {
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => string_from_utf8(data),
        Encoding::UTF16 => string_from_utf16(data),
        Encoding::UTF16BE => string_from_utf16be(data) 
    }
}

/// Returns a string created from the vector using UTF-8 encoding, removing any
/// trailing nul bytes.
/// Returns `None` if the vector is not a valid UTF-8 string.
pub fn string_from_utf8(data: &[u8]) -> Option<string::String> {
    let data: Vec<u8> = data.iter().take_while(|&c| *c != 0).map(|c| *c).collect();
    string::String::from_utf8(data).ok()
}

/// Returns a string created from the vector using UTF-16 (with byte order mark) encoding.
/// Returns `None` if the vector is not a valid UTF-16 string.
pub fn string_from_utf16(data: &[u8]) -> Option<string::String> {
    if data.len() < 2 || data.len() % 2 != 0 { 
        return None;
    }

    if data[0] == 0xFF && data[1] == 0xFE { // little endian
        string_from_utf16le(data.slice_from(2))
    } else { // big endian
        string_from_utf16be(data.slice_from(2))
    }
}

/// Returns a string created from the vector using UTF-16LE encoding.
/// Returns `None` if the vector is not a valid UTF-16LE string.
pub fn string_from_utf16le(data: &[u8]) -> Option<string::String> {
    if data.len() % 2 != 0 { 
        return None;
    }

    if cfg!(target_endian = "little") {
        let buf = unsafe { transmute::<_, &[u16]>(data) };
        string::String::from_utf16(buf.slice_to(data.len() / 2))
    } else {
        let mut buf: Vec<u16> = Vec::with_capacity(data.len() / 2);
        let mut it = std::iter::range_step(0, data.len(), 2);

        for i in it {
            buf.push(data[i] as u16 | data[i + 1] as u16 << 8);
        }

        string::String::from_utf16(buf.as_slice())
    }
}

/// Returns a string created from the vector using UTF-16BE encoding.
/// Returns `None` if the vector is not a valid UTF-16BE string.
pub fn string_from_utf16be(data: &[u8]) -> Option<string::String> {
    if data.len() % 2 != 0 { 
        return None;
    }
    if cfg!(target_endian = "big") {
        let buf = unsafe { transmute::<_, &[u16]>(data) };
        string::String::from_utf16(buf.slice_to(data.len() / 2))
    } else {
        let mut buf: Vec<u16> = Vec::with_capacity(data.len() / 2);
        let mut it = std::iter::range_step(0, data.len(), 2);

        for i in it {
            buf.push(data[i] as u16 << 8 | data[i + 1] as u16);
        }

        string::String::from_utf16(buf.as_slice())
    }
}

/// Returns a UTF-16 (with native byte order) vector representation of the string.
pub fn string_to_utf16(text: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(2 + text.len() * 2);

    if cfg!(target_endian = "little") {
        out.push_all(&[0xFF, 0xFE]); // add little endian BOM
        out.extend(string_to_utf16le(text).into_iter());
    } else {
        out.push_all(&[0xFE, 0xFF]); // add big endian BOM
        out.extend(string_to_utf16be(text).into_iter());
    }
    out
}

/// Returns a UTF-16BE vector representation of the string.
pub fn string_to_utf16be(text: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(text.len() * 2);
    for c in text.as_slice().utf16_units() {
        out.push(((c & 0xFF00) >> 8) as u8);
        out.push((c & 0x00FF) as u8);
    }

    out
}

/// Returns a UTF-16LE vector representation of the string.
pub fn string_to_utf16le(text: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(text.len() * 2);
    for c in text.utf16_units() {
        out.push((c & 0x00FF) as u8);
        out.push(((c & 0xFF00) >> 8) as u8);
    }

    out
}

/// Get string-terminating delimiter for the specified text encoding.
#[inline(always)]
pub fn delim(encoding: Encoding) -> &'static [u8] {
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => {const _F: &'static [u8] = &[0u8]; _F},
        Encoding::UTF16 | Encoding::UTF16BE => {const _F: &'static [u8] = &[0u8, 0u8]; _F},
    }
}

/// Get the length of the delimiter for the specified text encoding.
#[inline]
pub fn delim_len(encoding: Encoding) -> uint {
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => 1,
        Encoding::UTF16 | Encoding::UTF16BE => 2
    }
}

// Tests {{{
#[cfg(test)]
mod tests {
    use util;
    use id3v2::frame::Encoding;

    #[test]
    fn test_synchsafe() {
        assert_eq!(681570, util::synchsafe(176994));
        assert_eq!(176994, util::unsynchsafe(681570));
    }

    #[test]
    fn test_strings() {
        let text: &str = "śốмễ śŧŗỉňĝ";

        let mut utf8 = text.as_bytes().to_vec();
        utf8.push(0);
        assert_eq!(util::string_from_utf8(utf8.as_slice()).unwrap().as_slice(), text);

        // should use little endian BOM
        assert_eq!(util::string_to_utf16(text).as_slice(), b"\xFF\xFE\x5B\x01\xD1\x1E\x3C\x04\xC5\x1E\x20\x00\x5B\x01\x67\x01\x57\x01\xC9\x1E\x48\x01\x1D\x01");

        assert_eq!(util::string_to_utf16be(text).as_slice(), b"\x01\x5B\x1E\xD1\x04\x3C\x1E\xC5\x00\x20\x01\x5B\x01\x67\x01\x57\x1E\xC9\x01\x48\x01\x1D");
        assert_eq!(util::string_to_utf16le(text).as_slice(), b"\x5B\x01\xD1\x1E\x3C\x04\xC5\x1E\x20\x00\x5B\x01\x67\x01\x57\x01\xC9\x1E\x48\x01\x1D\x01");

        assert_eq!(util::string_from_encoding(Encoding::UTF16BE, b"\x01\x5B\x1E\xD1\x04\x3C\x1E\xC5\x00\x20\x01\x5B\x01\x67\x01\x57\x1E\xC9\x01\x48\x01\x1D").unwrap().as_slice(), text);
        assert_eq!(util::string_from_utf16be(b"\x01\x5B\x1E\xD1\x04\x3C\x1E\xC5\x00\x20\x01\x5B\x01\x67\x01\x57\x1E\xC9\x01\x48\x01\x1D").unwrap().as_slice(), text);

        assert_eq!(util::string_from_utf16le(b"\x5B\x01\xD1\x1E\x3C\x04\xC5\x1E\x20\x00\x5B\x01\x67\x01\x57\x01\xC9\x1E\x48\x01\x1D\x01").unwrap().as_slice(), text);

        // big endian BOM
        assert_eq!(util::string_from_encoding(Encoding::UTF16, b"\xFE\xFF\x01\x5B\x1E\xD1\x04\x3C\x1E\xC5\x00\x20\x01\x5B\x01\x67\x01\x57\x1E\xC9\x01\x48\x01\x1D").unwrap().as_slice(), text);
        assert_eq!(util::string_from_utf16(b"\xFE\xFF\x01\x5B\x1E\xD1\x04\x3C\x1E\xC5\x00\x20\x01\x5B\x01\x67\x01\x57\x1E\xC9\x01\x48\x01\x1D").unwrap().as_slice(), text);

        // little endian BOM 
        assert_eq!(util::string_from_encoding(Encoding::UTF16, b"\xFF\xFE\x5B\x01\xD1\x1E\x3C\x04\xC5\x1E\x20\x00\x5B\x01\x67\x01\x57\x01\xC9\x1E\x48\x01\x1D\x01").unwrap().as_slice(), text);
        assert_eq!(util::string_from_utf16(b"\xFF\xFE\x5B\x01\xD1\x1E\x3C\x04\xC5\x1E\x20\x00\x5B\x01\x67\x01\x57\x01\xC9\x1E\x48\x01\x1D\x01").unwrap().as_slice(), text);
    }

    #[test]
    fn test_u32_to_bytes() {
        assert_eq!(util::u32_to_bytes(0x4B92DF71), vec!(0x4B as u8, 0x92 as u8, 0xDF as u8, 0x71 as u8));
    }
}
