extern crate std;

use phf;
use std::rand::{mod, Rng};
use frame::Encoding;
use std::mem::transmute;

/// Returns a random sequence of 16 bytes, intended to be used as a UUID.
#[inline]
pub fn uuid() -> Vec<u8> {
    rand::task_rng().gen_iter::<u8>().take(16).collect()
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
pub fn string_from_encoding(encoding: Encoding, data: &[u8]) -> Option<String> {
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => string_from_utf8(data),
        Encoding::UTF16 => string_from_utf16(data),
        Encoding::UTF16BE => string_from_utf16be(data) 
    }
}

/// Returns a string created from the vector using UTF-8 encoding, removing any trailing null
/// bytes.
/// Returns `None` if the vector is not a valid UTF-8 string.
pub fn string_from_utf8(data: &[u8]) -> Option<String> {
    let data: Vec<u8> = data.iter().take_while(|&c| *c != 0).map(|c| *c).collect();
    String::from_utf8(data).ok()
}

/// Returns a string created from the vector using UTF-16 (with byte order mark) encoding.
/// Returns `None` if the vector is not a valid UTF-16 string.
pub fn string_from_utf16(data: &[u8]) -> Option<String> {
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
pub fn string_from_utf16le(data: &[u8]) -> Option<String> {
    if data.len() % 2 != 0 { 
        return None;
    }

    if cfg!(target_endian = "little") {
        let buf = unsafe { transmute::<_, &[u16]>(data) };
        String::from_utf16(buf.slice_to(data.len() / 2))
    } else {
        let mut buf: Vec<u16> = Vec::with_capacity(data.len() / 2);
        let mut it = std::iter::range_step(0, data.len(), 2);

        for i in it {
            buf.push(data[i] as u16 | data[i + 1] as u16 << 8);
        }

        String::from_utf16(buf.as_slice())
    }
}

/// Returns a string created from the vector using UTF-16BE encoding.
/// Returns `None` if the vector is not a valid UTF-16BE string.
pub fn string_from_utf16be(data: &[u8]) -> Option<String> {
    if data.len() % 2 != 0 { 
        return None;
    }
    if cfg!(target_endian = "big") {
        let buf = unsafe { transmute::<_, &[u16]>(data) };
        String::from_utf16(buf.slice_to(data.len() / 2))
    } else {
        let mut buf: Vec<u16> = Vec::with_capacity(data.len() / 2);
        let mut it = std::iter::range_step(0, data.len(), 2);

        for i in it {
            buf.push(data[i] as u16 << 8 | data[i + 1] as u16);
        }

        String::from_utf16(buf.as_slice())
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

/// Returns the index of the first delimiter for the specified encoding.
pub fn find_delim(encoding: Encoding, data: &[u8], index: uint) -> Option<uint> {
    let mut i = index;
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => {
            if i >= data.len() {
                return None;
            }

            for c in data.slice_from(i).iter() {
                if *c == 0 {
                    break;
                }
                i += 1;
            }

            if i == data.len() { // delimiter was not found
                return None;
            }

            Some(i)
        },
        Encoding::UTF16 | Encoding::UTF16BE => {
            while i + 1 < data.len() 
                && (data[i] != 0 || data[i + 1] != 0) {
                    i += 2;
                }

            if i + 1 >= data.len() { // delimiter was not found
                return None;
            }

            Some(i)
        }
    } 
}

/// Returns the delimiter length for the specified encoding.
#[inline]
pub fn delim_len(encoding: Encoding) -> uint {
    match encoding {
        Encoding::Latin1 | Encoding::UTF8 => 1,
        Encoding::UTF16 | Encoding::UTF16BE => 2
    }
}

static ID_2_TO_3: phf::Map<[u8, ..3], [u8, ..4]> = phf_map! {
    b!("BUF") => b!("RBUF"),

    b!("CNT") => b!("PCNT"),
    b!("COM") => b!("COMM"),
    b!("CRA") => b!("AENC"),

    b!("ETC") => b!("ETCO"),

    b!("GEO") => b!("GEOB"),

    b!("IPL") => b!("IPLS"),

    b!("LNK") => b!("LINK"),

    b!("MCI") => b!("MCDI"),
    b!("MLL") => b!("MLLT"),

    b!("PIC") => b!("APIC"),
    b!("POP") => b!("POPM"),

    b!("REV") => b!("RVRB"),

    b!("SLT") => b!("SYLT"),
    b!("STC") => b!("SYTC"),

    b!("TAL") => b!("TALB"),
    b!("TBP") => b!("TBPM"),
    b!("TCM") => b!("TCOM"),
    b!("TCO") => b!("TCON"),
    b!("TCR") => b!("TCOP"),
    b!("TDY") => b!("TDLY"),
    b!("TEN") => b!("TENC"),
    b!("TFT") => b!("TFLT"),
    b!("TKE") => b!("TKEY"),
    b!("TLA") => b!("TLAN"),
    b!("TLE") => b!("TLEN"),
    b!("TMT") => b!("TMED"),
    b!("TOA") => b!("TOPE"),
    b!("TOF") => b!("TOFN"),
    b!("TOL") => b!("TOLY"),
    b!("TOT") => b!("TOAL"),
    b!("TP1") => b!("TPE1"),
    b!("TP2") => b!("TPE2"),
    b!("TP3") => b!("TPE3"),
    b!("TP4") => b!("TPE4"),
    b!("TPA") => b!("TPOS"),
    b!("TPB") => b!("TPUB"),
    b!("TRC") => b!("TSRC"),
    b!("TRK") => b!("TRCK"),
    b!("TSS") => b!("TSSE"),
    b!("TT1") => b!("TIT1"),
    b!("TT2") => b!("TIT2"),
    b!("TT3") => b!("TIT3"),
    b!("TXT") => b!("TEXT"),
    b!("TXX") => b!("TXXX"),
    b!("TYE") => b!("TYER"),

    b!("UFI") => b!("UFID"),
    b!("ULT") => b!("USLT"),

    b!("WAF") => b!("WOAF"),
    b!("WAR") => b!("WOAR"),
    b!("WAS") => b!("WOAS"),
    b!("WCM") => b!("WCOM"),
    b!("WCP") => b!("WCOP"),
    b!("WPB") => b!("WPUB"),
    b!("WXX") => b!("WXXX"),
};

/// Returns the corresponding ID3v2.3/ID3v2.4 ID given the ID3v2.2 ID. 
#[inline]
pub fn convert_id_2_to_3(id: [u8, ..3]) -> Option<[u8, ..4]> {
    ID_2_TO_3.get(&id).map(|t| *t)
}

static ID_3_TO_2: phf::Map<[u8, ..4], [u8, ..3]> = phf_map! {
    b!("RBUF") => b!("BUF"),

    b!("PCNT") => b!("CNT"),
    b!("COMM") => b!("COM"),
    b!("AENC") => b!("CRA"),

    b!("ETCO") => b!("ETC"),

    b!("GEOB") => b!("GEO"),

    b!("IPLS") => b!("IPL"),

    b!("LINK") => b!("LNK"),

    b!("MCDI") => b!("MCI"),
    b!("MLLT") => b!("MLL"),

    b!("APIC") => b!("PIC"),
    b!("POPM") => b!("POP"),

    b!("RVRB") => b!("REV"),

    b!("SYLT") => b!("SLT"),
    b!("SYTC") => b!("STC"),

    b!("TALB") => b!("TAL"),
    b!("TBPM") => b!("TBP"),
    b!("TCOM") => b!("TCM"),
    b!("TCON") => b!("TCO"),
    b!("TCOP") => b!("TCR"),
    b!("TDLY") => b!("TDY"),
    b!("TENC") => b!("TEN"),
    b!("TFLT") => b!("TFT"),
    b!("TKEY") => b!("TKE"),
    b!("TLAN") => b!("TLA"),
    b!("TLEN") => b!("TLE"),
    b!("TMED") => b!("TMT"),
    b!("TOPE") => b!("TOA"),
    b!("TOFN") => b!("TOF"),
    b!("TOLY") => b!("TOL"),
    b!("TOAL") => b!("TOT"),
    b!("TPE1") => b!("TP1"),
    b!("TPE2") => b!("TP2"),
    b!("TPE3") => b!("TP3"),
    b!("TPE4") => b!("TP4"),
    b!("TPOS") => b!("TPA"),
    b!("TPUB") => b!("TPB"),
    b!("TSRC") => b!("TRC"),
    b!("TRCK") => b!("TRK"),
    b!("TSSE") => b!("TSS"),
    b!("TIT1") => b!("TT1"),
    b!("TIT2") => b!("TT2"),
    b!("TIT3") => b!("TT3"),
    b!("TEXT") => b!("TXT"),
    b!("TXXX") => b!("TXX"),
    b!("TYER") => b!("TYE"),

    b!("UFID") => b!("UFI"),
    b!("USLT") => b!("ULT"),

    b!("WOAF") => b!("WAF"),
    b!("WOAR") => b!("WAR"),
    b!("WOAS") => b!("WAS"),
    b!("WCOM") => b!("WCM"),
    b!("WCOP") => b!("WCP"),
    b!("WPUB") => b!("WPB"),
    b!("WXXX") => b!("WXX"),
};

/// Returns the corresponding ID3v2.2 ID given the ID3v2.3/ID3v2.3 ID. 
#[inline]
pub fn convert_id_3_to_2(id: [u8, ..4]) -> Option<[u8, ..3]> {
    ID_3_TO_2.get(&id).map(|t| *t)
}

static FRAME_DESCRIPTIONS: phf::Map<[u8, ..4], &'static str> = phf_map! {
    b!("AENC") => "Audio encryption",
    b!("APIC") => "Attached picture",
    b!("ASPI") => "Audio seek point index",

    b!("COMM") => "Comments",
    b!("COMR") => "Commercial frame",

    b!("ENCR") => "Encryption method registration",
    b!("EQU2") => "Equalisation (2)",
    b!("EQUA") => "Equalization",
    b!("ETCO") => "Event timing codes",

    b!("IPLS") => "Involved people list",

    b!("GEOB") => "General encapsulated object",
    b!("GRID") => "Group identification registration",

    b!("LINK") => "Linked information",

    b!("MCDI") => "Music CD identifier",
    b!("MLLT") => "MPEG location lookup table",

    b!("OWNE") => "Ownership frame",

    b!("PRIV") => "Private frame",
    b!("PCNT") => "Play counter",
    b!("POPM") => "Popularimeter",
    b!("POSS") => "Position synchronisation frame",

    b!("RBUF") => "Recommended buffer size",
    b!("RVA2") => "Relative volume adjustment (2)",
    b!("RVAD") => "Relative volume adjustment",
    b!("RVRB") => "Reverb",

    b!("SEEK") => "Seek frame",
    b!("SIGN") => "Signature frame",
    b!("SYLT") => "Synchronised lyric/text",
    b!("SYTC") => "Synchronised tempo codes",

    b!("TALB") => "Album/Movie/Show title",
    b!("TBPM") => "BPM (beats per minute)",
    b!("TCOM") => "Composer",
    b!("TCON") => "Content type",
    b!("TCOP") => "Copyright message",
    b!("TDAT") => "Date",
    b!("TDEN") => "Encoding time",
    b!("TDLY") => "Playlist delay",
    b!("TDOR") => "Original release time",
    b!("TDRC") => "Recording time",
    b!("TDRL") => "Release time",
    b!("TDTG") => "Tagging time",
    b!("TENC") => "Encoded by",
    b!("TEXT") => "Lyricist/Text writer",
    b!("TFLT") => "File type",
    b!("TIME") => "Time",
    b!("TIPL") => "Involved people list",
    b!("TIT1") => "Content group description",
    b!("TIT2") => "Title/songname/content description",
    b!("TIT3") => "Subtitle/Description refinement",
    b!("TKEY") => "Initial key",
    b!("TLAN") => "Language(s)",
    b!("TLEN") => "Length",
    b!("TMCL") => "Musician credits list",
    b!("TMED") => "Media type",
    b!("TMOO") => "Mood",
    b!("TOAL") => "Original album/movie/show title",
    b!("TOFN") => "Original filename",
    b!("TOLY") => "Original lyricist(s)/text writer(s)",
    b!("TOPE") => "Original artist(s)/performer(s)",
    b!("TORY") => "Original release year",
    b!("TOWN") => "File owner/licensee",
    b!("TPE1") => "Lead performer(s)/Soloist(s)",
    b!("TPE2") => "Band/orchestra/accompaniment",
    b!("TPE3") => "Conductor/performer refinement",
    b!("TPE4") => "Interpreted, remixed, or otherwise modified by",
    b!("TPOS") => "Part of a set",
    b!("TPRO") => "Produced notice",
    b!("TPUB") => "Publisher",
    b!("TRCK") => "Track number/Position in set",
    b!("TRDA") => "Recording dates",
    b!("TRSN") => "Internet radio station name",
    b!("TRSO") => "Internet radio station owner",
    b!("TSIZ") => "Size",
    b!("TSO2") => "Album artist sort order",
    b!("TSOA") => "Album sort order",
    b!("TSOC") => "Composer sort order",
    b!("TSOP") => "Performer sort order",
    b!("TSOT") => "Title sort order",
    b!("TSRC") => "ISRC (international standard recording code)",
    b!("TSSE") => "Software/Hardware and settings used for encoding",
    b!("TYER") => "Year",
    b!("TSST") => "Set subtitle",
    b!("TXXX") => "User defined text information frame",

    b!("UFID") => "Unique file identifier",
    b!("USER") => "Terms of use",
    b!("USLT") => "Unsynchronised lyric/text transcription",

    b!("WCOM") => "Commercial information",
    b!("WCOP") => "Copyright/Legal information",
    b!("WOAF") => "Official audio file webpage",
    b!("WOAR") => "Official artist/performer webpage",
    b!("WOAS") => "Official audio source webpage",
    b!("WORS") => "Official Internet radio station homepage",
    b!("WPAY") => "Payment",
    b!("WPUB") => "Publishers official webpage",
    b!("WXXX") => "User defined URL link frame",
};

/// Returns a string describing the frame type.
#[inline]
pub fn frame_description(id: [u8, ..4]) -> &'static str {
    match FRAME_DESCRIPTIONS.get(&id).map(|t| *t){
        Some(desc) => desc,
        None => ""
    }
}

// Tests {{{
#[cfg(test)]
mod tests {
    use util;
    use frame::Encoding;

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
    fn test_find_delim() {
        assert_eq!(util::find_delim(Encoding::UTF8, &[0x0, 0xFF, 0xFF, 0xFF, 0x0], 3).unwrap(), 4);
        assert!(util::find_delim(Encoding::UTF8, &[0x0, 0xFF, 0xFF, 0xFF, 0xFF], 3).is_none());

        assert_eq!(util::find_delim(Encoding::UTF16, &[0x0, 0xFF, 0x0, 0xFF, 0x0, 0x0, 0xFF, 0xFF], 2).unwrap(), 4);
        assert!(util::find_delim(Encoding::UTF16, &[0x0, 0xFF, 0x0, 0xFF, 0x0, 0xFF, 0xFF, 0xFF], 2).is_none());

        assert_eq!(util::find_delim(Encoding::UTF16BE, &[0x0, 0xFF, 0x0, 0xFF, 0x0, 0x0, 0xFF, 0xFF], 2).unwrap(), 4);
        assert!(util::find_delim(Encoding::UTF16BE, &[0x0, 0xFF, 0x0, 0xFF, 0x0, 0xFF, 0xFF, 0xFF], 2).is_none());
    }

    #[test]
    fn test_u32_to_bytes() {
        assert_eq!(util::u32_to_bytes(0x4B92DF71), vec!(0x4B as u8, 0x92 as u8, 0xDF as u8, 0x71 as u8));
    }
}
