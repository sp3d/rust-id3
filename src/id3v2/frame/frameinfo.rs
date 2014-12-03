#![plugin(phf_macros)]
extern crate std;

use phf;
use id3v2::frame::Id;
use id3v2::frame::field::FieldType;
use id3v2::frame::field::FieldType::*;

struct FrameInfo<'a> {
    fields: &'a [FieldType],
    desc: &'a str,
/* it might be a good idea to have info on whether or not this sort of frame
is intended to be dropped on tag modification, or if it's obsolete, or so on */
//    flags: FrameProcessingFlags
}

macro_rules! frame_info(($fields: expr, $desc: expr) => (FrameInfo {fields: {const _F: &'static [FieldType] = &$fields; _F}, desc: $desc}));

//TODO: see if String is the right type for "<textstring> $00 ($00)"
static FRAME_INFO_V2: phf::Map<&'static [u8; 3], FrameInfo<'static>> = phf_map! {
    b"BUF" => frame_info!([Int24, Int8, Int32,], "Recommended buffer size"),
    b"CNT" => frame_info!([Int32Plus,], "Play counter"),
    b"COM" => frame_info!([TextEncoding,Language,String,String,], "Comments"),
    b"CRA" => frame_info!([String,Int16,Int16,BinaryData,], "Audio encryption"),
    b"CRM" => frame_info!([String,String,BinaryData,], "Encrypted meta frame"),

    b"ETC" => frame_info!([Int8,BinaryData,], "Event timing codes"),
    b"EQU" => frame_info!([Int8,BinaryData,], "Equalization"),

    b"GEO" => frame_info!([TextEncoding,Latin1,String,String,BinaryData,], "General encapsulated object"),

    b"IPL" => frame_info!([TextEncoding,StringList,], "Involved people list"),

    b"LNK" => frame_info!([FrameIdV2,String,StringList,], "Linked information"),//TODO: verify

    b"MCI" => frame_info!([BinaryData,], "Music CD Identifier"),
    b"MLL" => frame_info!([Int16,Int24,Int24,Int8,Int8,BinaryData,], "MPEG location lookup table"),

    b"PIC" => frame_info!([TextEncoding,Int24,Int8,String,BinaryData,], "Attached picture"),
    b"POP" => frame_info!([Latin1,Int8,Int32Plus,], "Popularimeter"),

    b"REV" => frame_info!([Int16,Int16,Int8,Int8,Int8,Int8,Int8,Int8,Int8,Int8,], "Reverb"),
    b"RVA" => frame_info!([Int8,Int8,BinaryData,], "Relative volume adjustment"),

    b"SLT" => frame_info!([TextEncoding,Language,Int8,Int8,String,BinaryData,], "Synchronized lyric/text"),
    b"STC" => frame_info!([Int8,BinaryData,], "Synced tempo codes"),

    b"TAL" => frame_info!([TextEncoding,String,], "Album/Movie/Debug title"),
    b"TBP" => frame_info!([TextEncoding,String,], "BPM (Beats Per Minute)"),
    b"TCM" => frame_info!([TextEncoding,String,], "Composer"),
    b"TCO" => frame_info!([TextEncoding,String,], "Content type"),
    b"TCR" => frame_info!([TextEncoding,String,], "Copyright message"),
    b"TDA" => frame_info!([TextEncoding,String,], "Date"),
    b"TDY" => frame_info!([TextEncoding,String,], "Playlist delay"),
    b"TEN" => frame_info!([TextEncoding,String,], "Encoded by"),
    b"TFT" => frame_info!([TextEncoding,String,], "File type"),
    b"TIM" => frame_info!([TextEncoding,String,], "Time"),
    b"TKE" => frame_info!([TextEncoding,String,], "Initial key"),
    b"TLA" => frame_info!([TextEncoding,String,], "Language(s)"),
    b"TLE" => frame_info!([TextEncoding,String,], "Length"),
    b"TMT" => frame_info!([TextEncoding,String,], "Media type"),
    b"TOA" => frame_info!([TextEncoding,String,], "Original artist(s)/performer(s)"),
    b"TOF" => frame_info!([TextEncoding,String,], "Original filename"),
    b"TOL" => frame_info!([TextEncoding,String,], "Original Lyricist(s)/text writer(s)"),
    b"TOR" => frame_info!([TextEncoding,String,], "Original release year"),
    b"TOT" => frame_info!([TextEncoding,String,], "Original album/Movie/Debug title"),
    b"TP1" => frame_info!([TextEncoding,String,], "Lead artist(s)/Lead performer(s)/Soloist(s)/Performing group"),
    b"TP2" => frame_info!([TextEncoding,String,], "Band/Orchestra/Accompaniment"),
    b"TP3" => frame_info!([TextEncoding,String,], "Conductor/Performer refinement"),
    b"TP4" => frame_info!([TextEncoding,String,], "Interpreted, remixed, or otherwise modified by"),
    b"TPA" => frame_info!([TextEncoding,String,], "Part of a set"),
    b"TPB" => frame_info!([TextEncoding,String,], "Publisher"),
    b"TRC" => frame_info!([TextEncoding,String,], "ISRC (International Standard Recording Code)"),
    b"TRD" => frame_info!([TextEncoding,String,], "Recording dates"),
    b"TRK" => frame_info!([TextEncoding,String,], "Track number/Position in set"),
    b"TSI" => frame_info!([TextEncoding,String,], "Size"),
    b"TSS" => frame_info!([TextEncoding,String,], "Software/hardware and settings used for encoding"),
    b"TT1" => frame_info!([TextEncoding,String,], "Content group description"),
    b"TT2" => frame_info!([TextEncoding,String,], "Title/Songname/Content description"),
    b"TT3" => frame_info!([TextEncoding,String,], "Subtitle/Description refinement"),
    b"TXT" => frame_info!([TextEncoding,String,], "Lyricist/text writer"),
    b"TXX" => frame_info!([TextEncoding,String,], "User defined text information frame"),
    b"TYE" => frame_info!([TextEncoding,String,], "Year"),

    b"UFI" => frame_info!([Latin1,BinaryData,], "Unique file identifier"), //TODO: verify
    b"ULT" => frame_info!([TextEncoding,Language,String,StringFull,], "Unsychronized lyric/text transcription"),

    b"WAF" => frame_info!([Latin1,], "Official audio file webpage"),
    b"WAR" => frame_info!([Latin1,], "Official artist/performer webpage"),
    b"WAS" => frame_info!([Latin1,], "Official audio source webpage"),
    b"WCM" => frame_info!([Latin1,], "Commercial information"),
    b"WCP" => frame_info!([Latin1,], "Copyright/Legal information"),
    b"WPB" => frame_info!([Latin1,], "Publishers official webpage"),
    b"WXX" => frame_info!([Latin1,], "User defined URL link frame"),
};

static FRAME_INFO_V3: phf::Map<&'static [u8; 4], FrameInfo<'static>> = phf_map! {
    b"EQUA" => frame_info!([Int8,BinaryData,], "Equalization"),
    b"IPLS" => frame_info!([TextEncoding,StringList,], "Involved people list"),
    b"RVAD" => frame_info!([Int32,Int8,BinaryData,], "Relative volume adjustment"),
};

static FRAME_INFO_V4: phf::Map<&'static [u8; 4], FrameInfo<'static>> = phf_map! {
    b"ASPI" => frame_info!([Int32,Int32,Int16,Int8,BinaryData,], "Audio seek point index"),
    b"EQU2" => frame_info!([Int8,Latin1,BinaryData,], "Equalisation (2)"),
    b"RVA2" => frame_info!([Latin1,BinaryData,], "Relative volume adjustment (2)"),
    b"SEEK" => frame_info!([Int32,], "Seek frame"),
    b"SIGN" => frame_info!([Int8,BinaryData,], "Signature frame"),
    b"TDEN" => frame_info!([TextEncoding,StringList,], "Encoding time"),
    b"TDOR" => frame_info!([TextEncoding,StringList,], "Original release time"),
    b"TDRC" => frame_info!([TextEncoding,StringList,], "Recording time"),
    b"TDRL" => frame_info!([TextEncoding,StringList,], "Release time"),
    b"TDTG" => frame_info!([TextEncoding,StringList,], "Tagging time"),
    b"TIPL" => frame_info!([TextEncoding,StringList,], "Involved people list"),
    b"TMCL" => frame_info!([TextEncoding,StringList,], "Musician credits list"),
    b"TMOO" => frame_info!([TextEncoding,StringList,], "Mood"),
    b"TPRO" => frame_info!([TextEncoding,StringList,], "Produced notice"),
    b"TSOA" => frame_info!([TextEncoding,StringList,], "Album sort order"),
    b"TSOP" => frame_info!([TextEncoding,StringList,], "Performer sort order"),
    b"TSOT" => frame_info!([TextEncoding,StringList,], "Title sort order"),
    b"TSST" => frame_info!([TextEncoding,StringList,], "Set subtitle"),
};

static FRAME_INFO_V34: phf::Map<&'static [u8; 4], FrameInfo<'static>> = phf_map! {
    b"AENC" => frame_info!([Latin1,Int16,Int16,BinaryData,], "Audio encryption"),
    b"APIC" => frame_info!([TextEncoding,Latin1,Int8,String,BinaryData,], "Attached picture"),

    b"COMM" => frame_info!([TextEncoding,Language,String,StringFull,], "Comments"),
    b"COMR" => frame_info!([TextEncoding,Latin1,Date,Latin1,Int8,String,String,Latin1,BinaryData,], "Commercial frame"),

    b"ENCR" => frame_info!([Latin1,Int8,BinaryData,], "Encryption method registration"),
    b"ETCO" => frame_info!([Int8,BinaryData,], "Event timing codes"),


    b"GEOB" => frame_info!([TextEncoding,Latin1,String,String,BinaryData,], "General encapsulated object"),
    b"GRID" => frame_info!([Latin1,Int8,BinaryData,], "Group identification registration"),

    b"LINK" => frame_info!([FrameIdV34,Latin1,Latin1List,], "Linked information"),

    b"MCDI" => frame_info!([BinaryData,], "Music CD identifier"),
    b"MLLT" => frame_info!([Int16,Int24,Int24,Int8,Int8,BinaryData,], "MPEG location lookup table"),

    b"OWNE" => frame_info!([TextEncoding,Latin1,Date,String,], "Ownership frame"),

    b"PRIV" => frame_info!([Latin1,BinaryData,], "Private frame"),
    b"PCNT" => frame_info!([Int32Plus,], "Play counter"),
    b"POPM" => frame_info!([Latin1,Int8,Int32Plus,], "Popularimeter"),
    b"POSS" => frame_info!([Int8,BinaryData,], "Position synchronisation frame"),

    b"RBUF" => frame_info!([Int24,Int8,Int32,], "Recommended buffer size"),
    b"RVRB" => frame_info!([Int16,Int16,Int8,Int8,Int8,Int8,Int8,Int8,Int8,Int8,], "Reverb"),

    b"SYLT" => frame_info!([TextEncoding,Language,Int8,Int8,String,BinaryData,], "Synchronised lyric/text"),
    b"SYTC" => frame_info!([Int8,BinaryData,], "Synchronised tempo codes"),

    b"TALB" => frame_info!([TextEncoding,StringList,], "Album/Movie/Debug title"),
    b"TBPM" => frame_info!([TextEncoding,StringList,], "BPM (beats per minute)"),
    b"TCOM" => frame_info!([TextEncoding,StringList,], "Composer"),
    b"TCON" => frame_info!([TextEncoding,StringList,], "Content type"),
    b"TCOP" => frame_info!([TextEncoding,StringList,], "Copyright message"),
    b"TDAT" => frame_info!([TextEncoding,StringList,], "Date"),
    b"TDLY" => frame_info!([TextEncoding,StringList,], "Playlist delay"),
    b"TENC" => frame_info!([TextEncoding,StringList,], "Encoded by"),
    b"TEXT" => frame_info!([TextEncoding,StringList,], "Lyricist/Text writer"),
    b"TFLT" => frame_info!([TextEncoding,StringList,], "File type"),
    b"TIME" => frame_info!([TextEncoding,StringList,], "Time"),
    b"TIT1" => frame_info!([TextEncoding,StringList,], "Content group description"),
    b"TIT2" => frame_info!([TextEncoding,StringList,], "Title/songname/content description"),
    b"TIT3" => frame_info!([TextEncoding,StringList,], "Subtitle/Description refinement"),
    b"TKEY" => frame_info!([TextEncoding,StringList,], "Initial key"),
    b"TLAN" => frame_info!([TextEncoding,StringList,], "Language(s)"),
    b"TLEN" => frame_info!([TextEncoding,StringList,], "Length"),
    b"TMED" => frame_info!([TextEncoding,StringList,], "Media type"),
    b"TOAL" => frame_info!([TextEncoding,StringList,], "Original album/movie/show title"),
    b"TOFN" => frame_info!([TextEncoding,StringList,], "Original filename"),
    b"TOLY" => frame_info!([TextEncoding,StringList,], "Original lyricist(s)/text writer(s)"),
    b"TOPE" => frame_info!([TextEncoding,StringList,], "Original artist(s)/performer(s)"),
    b"TORY" => frame_info!([TextEncoding,StringList,], "Original release year"),
    b"TOWN" => frame_info!([TextEncoding,StringList,], "File owner/licensee"),
    b"TPE1" => frame_info!([TextEncoding,StringList,], "Lead performer(s)/Soloist(s)"),
    b"TPE2" => frame_info!([TextEncoding,StringList,], "Band/orchestra/accompaniment"),
    b"TPE3" => frame_info!([TextEncoding,StringList,], "Conductor/performer refinement"),
    b"TPE4" => frame_info!([TextEncoding,StringList,], "Interpreted, remixed, or otherwise modified by"),
    b"TPOS" => frame_info!([TextEncoding,StringList,], "Part of a set"),
    b"TPUB" => frame_info!([TextEncoding,StringList,], "Publisher"),
    b"TRCK" => frame_info!([TextEncoding,StringList,], "Track number/Position in set"),
    b"TRDA" => frame_info!([TextEncoding,StringList,], "Recording dates"),
    b"TRSN" => frame_info!([TextEncoding,StringList,], "Internet radio station name"),
    b"TRSO" => frame_info!([TextEncoding,StringList,], "Internet radio station owner"),
    b"TSIZ" => frame_info!([TextEncoding,StringList,], "Size"),
    b"TSO2" => frame_info!([TextEncoding,StringList,], "Album artist sort order"),
    b"TSOC" => frame_info!([TextEncoding,StringList,], "Composer sort order"),
    b"TSRC" => frame_info!([TextEncoding,StringList,], "ISRC (international standard recording code)"),
    b"TSSE" => frame_info!([TextEncoding,StringList,], "Software/Hardware and settings used for encoding"),
    b"TYER" => frame_info!([TextEncoding,StringList,], "Year"),
    b"TXXX" => frame_info!([TextEncoding,String,String,], "User defined text information frame"),

    b"UFID" => frame_info!([Latin1,BinaryData,], "Unique file identifier"),
    b"USER" => frame_info!([TextEncoding,Language,String,], "Terms of use"),
    b"USLT" => frame_info!([TextEncoding,Language,String,StringFull,], "Unsynchronised lyric/text transcription"),

    b"WCOM" => frame_info!([Latin1,], "Commercial information"),
    b"WCOP" => frame_info!([Latin1,], "Copyright/Legal information"),
    b"WOAF" => frame_info!([Latin1,], "Official audio file webpage"),
    b"WOAR" => frame_info!([Latin1,], "Official artist/performer webpage"),
    b"WOAS" => frame_info!([Latin1,], "Official audio source webpage"),
    b"WORS" => frame_info!([Latin1,], "Official Internet radio station homepage"),
    b"WPAY" => frame_info!([Latin1,], "Payment"),
    b"WPUB" => frame_info!([Latin1,], "Publishers official webpage"),
    b"WXXX" => frame_info!([TextEncoding,String,Latin1,], "User defined URL link frame"),

//TODO: see how this relates to specs
    b"ZOBS" => frame_info!([BinaryData,], "Obsolete frame"),
};

#[inline]
fn get_frame_info(id: Id) -> Option<&'static FrameInfo<'static>> {
    match id {
        Id::V2(x) => FRAME_INFO_V2.get(&x),
        Id::V3(x) => FRAME_INFO_V34.get(&x).or_else(|| FRAME_INFO_V3.get(&x)),
        Id::V4(x) => FRAME_INFO_V34.get(&x).or_else(|| FRAME_INFO_V4.get(&x)),
    }
}

/// Returns a string describing the frame type.
#[inline]
pub fn frame_description(id: Id) -> &'static str {
    match get_frame_info(id) {
        Some(info) => info.desc,
        None => match id.name()[0] {
            b'T' => "Unknown text frame",
            b'W' => "Unknown URL frame",
            _ => "Unknown frame",
        },
    }
}

/// Returns the layout of fields within the frame, according to the specification.
#[inline]
pub fn frame_format(id: Id) -> Option<&'static [FieldType]> {
    match get_frame_info(id) {
        Some(info) => Some(info.fields),
        None => match id.name()[0] {
            b'T' => Some({static _F: &'static [FieldType] = &[TextEncoding,StringList,]; _F}),
            b'W' => Some({static _F: &'static [FieldType] = &[Latin1,]; _F}),
            _ => None,
        }
    }
}

static ID_2_TO_3: phf::Map<&'static [u8; 3], [u8; 4]> = phf_map! {
    b"BUF" => *b"RBUF",

    b"CNT" => *b"PCNT",
    b"COM" => *b"COMM",
    b"CRA" => *b"AENC",

    b"ETC" => *b"ETCO",

    b"GEO" => *b"GEOB",

    b"IPL" => *b"IPLS",

    b"LNK" => *b"LINK",

    b"MCI" => *b"MCDI",
    b"MLL" => *b"MLLT",

    b"PIC" => *b"APIC",
    b"POP" => *b"POPM",

    b"REV" => *b"RVRB",

    b"SLT" => *b"SYLT",
    b"STC" => *b"SYTC",

    b"TAL" => *b"TALB",
    b"TBP" => *b"TBPM",
    b"TCM" => *b"TCOM",
    b"TCO" => *b"TCON",
    b"TCR" => *b"TCOP",
    b"TDY" => *b"TDLY",
    b"TEN" => *b"TENC",
    b"TFT" => *b"TFLT",
    b"TKE" => *b"TKEY",
    b"TLA" => *b"TLAN",
    b"TLE" => *b"TLEN",
    b"TMT" => *b"TMED",
    b"TOA" => *b"TOPE",
    b"TOF" => *b"TOFN",
    b"TOL" => *b"TOLY",
    b"TOT" => *b"TOAL",
    b"TP1" => *b"TPE1",
    b"TP2" => *b"TPE2",
    b"TP3" => *b"TPE3",
    b"TP4" => *b"TPE4",
    b"TPA" => *b"TPOS",
    b"TPB" => *b"TPUB",
    b"TRC" => *b"TSRC",
    b"TRK" => *b"TRCK",
    b"TSS" => *b"TSSE",
    b"TT1" => *b"TIT1",
    b"TT2" => *b"TIT2",
    b"TT3" => *b"TIT3",
    b"TXT" => *b"TEXT",
    b"TXX" => *b"TXXX",
    b"TYE" => *b"TYER",

    b"UFI" => *b"UFID",
    b"ULT" => *b"USLT",

    b"WAF" => *b"WOAF",
    b"WAR" => *b"WOAR",
    b"WAS" => *b"WOAS",
    b"WCM" => *b"WCOM",
    b"WCP" => *b"WCOP",
    b"WPB" => *b"WPUB",
    b"WXX" => *b"WXXX",
};

/// Returns the corresponding ID3v2.3/ID3v2.4 ID given the ID3v2.2 ID. 
#[inline]
pub fn convert_id_2_to_3(id: [u8; 3]) -> Option<[u8; 4]> {
    ID_2_TO_3.get(&id).map(|t| *t)
}

static ID_3_TO_2: phf::Map<&'static [u8; 4], [u8; 3]> = phf_map! {
    b"RBUF" => *b"BUF",

    b"PCNT" => *b"CNT",
    b"COMM" => *b"COM",
    b"AENC" => *b"CRA",

    b"ETCO" => *b"ETC",

    b"GEOB" => *b"GEO",

    b"IPLS" => *b"IPL",

    b"LINK" => *b"LNK",

    b"MCDI" => *b"MCI",
    b"MLLT" => *b"MLL",

    b"APIC" => *b"PIC",
    b"POPM" => *b"POP",

    b"RVRB" => *b"REV",

    b"SYLT" => *b"SLT",
    b"SYTC" => *b"STC",

    b"TALB" => *b"TAL",
    b"TBPM" => *b"TBP",
    b"TCOM" => *b"TCM",
    b"TCON" => *b"TCO",
    b"TCOP" => *b"TCR",
    b"TDLY" => *b"TDY",
    b"TENC" => *b"TEN",
    b"TFLT" => *b"TFT",
    b"TKEY" => *b"TKE",
    b"TLAN" => *b"TLA",
    b"TLEN" => *b"TLE",
    b"TMED" => *b"TMT",
    b"TOPE" => *b"TOA",
    b"TOFN" => *b"TOF",
    b"TOLY" => *b"TOL",
    b"TOAL" => *b"TOT",
    b"TPE1" => *b"TP1",
    b"TPE2" => *b"TP2",
    b"TPE3" => *b"TP3",
    b"TPE4" => *b"TP4",
    b"TPOS" => *b"TPA",
    b"TPUB" => *b"TPB",
    b"TSRC" => *b"TRC",
    b"TRCK" => *b"TRK",
    b"TSSE" => *b"TSS",
    b"TIT1" => *b"TT1",
    b"TIT2" => *b"TT2",
    b"TIT3" => *b"TT3",
    b"TEXT" => *b"TXT",
    b"TXXX" => *b"TXX",
    b"TYER" => *b"TYE",

    b"UFID" => *b"UFI",
    b"USLT" => *b"ULT",

    b"WOAF" => *b"WAF",
    b"WOAR" => *b"WAR",
    b"WOAS" => *b"WAS",
    b"WCOM" => *b"WCM",
    b"WCOP" => *b"WCP",
    b"WPUB" => *b"WPB",
    b"WXXX" => *b"WXX",
};

/// Returns the corresponding ID3v2.2 ID given the ID3v2.3/ID3v2.3 ID. 
#[inline]
pub fn convert_id_3_to_2(id: [u8; 4]) -> Option<[u8; 3]> {
    ID_3_TO_2.get(&id).map(|t| *t)
}
