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
static FRAME_INFO_V2: phf::Map<[u8; 3], FrameInfo<'static>> = phf_map! {
    [66, 85, 70] => frame_info!([Int24, Int8, Int32,], "Recommended buffer size"),
    [67, 78, 84] => frame_info!([Int32Plus,], "Play counter"),
    [67, 79, 77] => frame_info!([TextEncoding,Language,String,String,], "Comments"),
    [67, 82, 65] => frame_info!([String,Int16,Int16,BinaryData,], "Audio encryption"),
    [67, 82, 77] => frame_info!([String,String,BinaryData,], "Encrypted meta frame"),

    [69, 84, 67] => frame_info!([Int8,BinaryData,], "Event timing codes"),
    [69, 81, 85] => frame_info!([Int8,BinaryData,], "Equalization"),

    [71, 69, 79] => frame_info!([TextEncoding,Latin1,String,String,BinaryData,], "General encapsulated object"),

    [73, 80, 76] => frame_info!([TextEncoding,StringList,], "Involved people list"),

    [76, 78, 75] => frame_info!([FrameIdV2,String,StringList,], "Linked information"),//TODO: verify

    [77, 67, 73] => frame_info!([BinaryData,], "Music CD Identifier"),
    [77, 76, 76] => frame_info!([Int16,Int24,Int24,Int8,Int8,BinaryData,], "MPEG location lookup table"),

    [80, 73, 67] => frame_info!([TextEncoding,Int24,Int8,String,BinaryData,], "Attached picture"),
    [80, 79, 80] => frame_info!([Latin1,Int8,Int32Plus,], "Popularimeter"),

    [82, 69, 86] => frame_info!([Int16,Int16,Int8,Int8,Int8,Int8,Int8,Int8,Int8,Int8,], "Reverb"),
    [82, 86, 65] => frame_info!([Int8,Int8,BinaryData,], "Relative volume adjustment"),

    [83, 76, 84] => frame_info!([TextEncoding,Language,Int8,Int8,String,BinaryData,], "Synchronized lyric/text"),
    [83, 84, 67] => frame_info!([Int8,BinaryData,], "Synced tempo codes"),

    [84, 65, 76] => frame_info!([TextEncoding,String,], "Album/Movie/Debug title"),
    [84, 66, 80] => frame_info!([TextEncoding,String,], "BPM (Beats Per Minute)"),
    [84, 67, 77] => frame_info!([TextEncoding,String,], "Composer"),
    [84, 67, 79] => frame_info!([TextEncoding,String,], "Content type"),
    [84, 67, 82] => frame_info!([TextEncoding,String,], "Copyright message"),
    [84, 68, 65] => frame_info!([TextEncoding,String,], "Date"),
    [84, 68, 89] => frame_info!([TextEncoding,String,], "Playlist delay"),
    [84, 69, 78] => frame_info!([TextEncoding,String,], "Encoded by"),
    [84, 70, 84] => frame_info!([TextEncoding,String,], "File type"),
    [84, 73, 77] => frame_info!([TextEncoding,String,], "Time"),
    [84, 75, 69] => frame_info!([TextEncoding,String,], "Initial key"),
    [84, 76, 65] => frame_info!([TextEncoding,String,], "Language(s)"),
    [84, 76, 69] => frame_info!([TextEncoding,String,], "Length"),
    [84, 77, 84] => frame_info!([TextEncoding,String,], "Media type"),
    [84, 79, 65] => frame_info!([TextEncoding,String,], "Original artist(s)/performer(s)"),
    [84, 79, 70] => frame_info!([TextEncoding,String,], "Original filename"),
    [84, 79, 76] => frame_info!([TextEncoding,String,], "Original Lyricist(s)/text writer(s)"),
    [84, 79, 82] => frame_info!([TextEncoding,String,], "Original release year"),
    [84, 79, 84] => frame_info!([TextEncoding,String,], "Original album/Movie/Debug title"),
    [84, 80, 49] => frame_info!([TextEncoding,String,], "Lead artist(s)/Lead performer(s)/Soloist(s)/Performing group"),
    [84, 80, 50] => frame_info!([TextEncoding,String,], "Band/Orchestra/Accompaniment"),
    [84, 80, 51] => frame_info!([TextEncoding,String,], "Conductor/Performer refinement"),
    [84, 80, 52] => frame_info!([TextEncoding,String,], "Interpreted, remixed, or otherwise modified by"),
    [84, 80, 65] => frame_info!([TextEncoding,String,], "Part of a set"),
    [84, 80, 66] => frame_info!([TextEncoding,String,], "Publisher"),
    [84, 82, 67] => frame_info!([TextEncoding,String,], "ISRC (International Standard Recording Code)"),
    [84, 82, 68] => frame_info!([TextEncoding,String,], "Recording dates"),
    [84, 82, 75] => frame_info!([TextEncoding,String,], "Track number/Position in set"),
    [84, 83, 73] => frame_info!([TextEncoding,String,], "Size"),
    [84, 83, 83] => frame_info!([TextEncoding,String,], "Software/hardware and settings used for encoding"),
    [84, 84, 49] => frame_info!([TextEncoding,String,], "Content group description"),
    [84, 84, 50] => frame_info!([TextEncoding,String,], "Title/Songname/Content description"),
    [84, 84, 51] => frame_info!([TextEncoding,String,], "Subtitle/Description refinement"),
    [84, 88, 84] => frame_info!([TextEncoding,String,], "Lyricist/text writer"),
    [84, 88, 88] => frame_info!([TextEncoding,String,], "User defined text information frame"),
    [84, 89, 69] => frame_info!([TextEncoding,String,], "Year"),

    [85, 70, 73] => frame_info!([Latin1,BinaryData,], "Unique file identifier"), //TODO: verify
    [85, 76, 84] => frame_info!([TextEncoding,Language,String,StringFull,], "Unsychronized lyric/text transcription"),

    [87, 65, 70] => frame_info!([Latin1,], "Official audio file webpage"),
    [87, 65, 82] => frame_info!([Latin1,], "Official artist/performer webpage"),
    [87, 65, 83] => frame_info!([Latin1,], "Official audio source webpage"),
    [87, 67, 77] => frame_info!([Latin1,], "Commercial information"),
    [87, 67, 80] => frame_info!([Latin1,], "Copyright/Legal information"),
    [87, 80, 66] => frame_info!([Latin1,], "Publishers official webpage"),
    [87, 88, 88] => frame_info!([Latin1,], "User defined URL link frame"),
};

static FRAME_INFO_V3: phf::Map<[u8; 4], FrameInfo<'static>> = phf_map! {
    [69, 81, 85, 65] => frame_info!([Int8,BinaryData,], "Equalization"),
    [73, 80, 76, 83] => frame_info!([TextEncoding,StringList,], "Involved people list"),
    [82, 86, 65, 68] => frame_info!([Int32,Int8,BinaryData,], "Relative volume adjustment"),
};

static FRAME_INFO_V4: phf::Map<[u8; 4], FrameInfo<'static>> = phf_map! {
    [65, 83, 80, 73] => frame_info!([Int32,Int32,Int16,Int8,BinaryData,], "Audio seek point index"),
    [69, 81, 85, 50] => frame_info!([Int8,Latin1,BinaryData,], "Equalisation (2)"),
    [82, 86, 65, 50] => frame_info!([Latin1,BinaryData,], "Relative volume adjustment (2)"),
    [83, 69, 69, 75] => frame_info!([Int32,], "Seek frame"),
    [83, 73, 71, 78] => frame_info!([Int8,BinaryData,], "Signature frame"),
    [84, 68, 69, 78] => frame_info!([TextEncoding,StringList,], "Encoding time"),
    [84, 68, 79, 82] => frame_info!([TextEncoding,StringList,], "Original release time"),
    [84, 68, 82, 67] => frame_info!([TextEncoding,StringList,], "Recording time"),
    [84, 68, 82, 76] => frame_info!([TextEncoding,StringList,], "Release time"),
    [84, 68, 84, 71] => frame_info!([TextEncoding,StringList,], "Tagging time"),
    [84, 73, 80, 76] => frame_info!([TextEncoding,StringList,], "Involved people list"),
    [84, 77, 67, 76] => frame_info!([TextEncoding,StringList,], "Musician credits list"),
    [84, 77, 79, 79] => frame_info!([TextEncoding,StringList,], "Mood"),
    [84, 80, 82, 79] => frame_info!([TextEncoding,StringList,], "Produced notice"),
    [84, 83, 79, 65] => frame_info!([TextEncoding,StringList,], "Album sort order"),
    [84, 83, 79, 80] => frame_info!([TextEncoding,StringList,], "Performer sort order"),
    [84, 83, 79, 84] => frame_info!([TextEncoding,StringList,], "Title sort order"),
    [84, 83, 83, 84] => frame_info!([TextEncoding,StringList,], "Set subtitle"),
};

static FRAME_INFO_V34: phf::Map<[u8; 4], FrameInfo<'static>> = phf_map! {
    [65, 69, 78, 67] => frame_info!([Latin1,Int16,Int16,BinaryData,], "Audio encryption"),
    [65, 80, 73, 67] => frame_info!([TextEncoding,Latin1,Int8,String,BinaryData,], "Attached picture"),

    [67, 79, 77, 77] => frame_info!([TextEncoding,Language,String,StringFull,], "Comments"),
    [67, 79, 77, 82] => frame_info!([TextEncoding,Latin1,Latin1,Latin1,Int8,String,String,Latin1,BinaryData,], "Commercial frame"),

    [69, 78, 67, 82] => frame_info!([Latin1,Int8,BinaryData,], "Encryption method registration"),
    [69, 84, 67, 79] => frame_info!([Int8,BinaryData,], "Event timing codes"),


    [71, 69, 79, 66] => frame_info!([TextEncoding,Latin1,String,String,BinaryData,], "General encapsulated object"),
    [71, 82, 73, 68] => frame_info!([Latin1,Int8,BinaryData,], "Group identification registration"),

    [76, 73, 78, 75] => frame_info!([FrameIdV34,Latin1,Latin1List,], "Linked information"),

    [77, 67, 68, 73] => frame_info!([BinaryData,], "Music CD identifier"),
    [77, 76, 76, 84] => frame_info!([Int16,Int24,Int24,Int8,Int8,BinaryData,], "MPEG location lookup table"),

    [79, 87, 78, 69] => frame_info!([TextEncoding,Latin1,Latin1,String,], "Ownership frame"),

    [80, 82, 73, 86] => frame_info!([Latin1,BinaryData,], "Private frame"),
    [80, 67, 78, 84] => frame_info!([Int32Plus,], "Play counter"),
    [80, 79, 80, 77] => frame_info!([Latin1,Int8,Int32Plus,], "Popularimeter"),
    [80, 79, 83, 83] => frame_info!([Int8,BinaryData,], "Position synchronisation frame"),

    [82, 66, 85, 70] => frame_info!([Int24,Int8,Int32,], "Recommended buffer size"),
    [82, 86, 82, 66] => frame_info!([Int16,Int16,Int8,Int8,Int8,Int8,Int8,Int8,Int8,Int8,], "Reverb"),

    [83, 89, 76, 84] => frame_info!([TextEncoding,Language,Int8,Int8,String,BinaryData,], "Synchronised lyric/text"),
    [83, 89, 84, 67] => frame_info!([Int8,BinaryData,], "Synchronised tempo codes"),

    [84, 65, 76, 66] => frame_info!([TextEncoding,StringList,], "Album/Movie/Debug title"),
    [84, 66, 80, 77] => frame_info!([TextEncoding,StringList,], "BPM (beats per minute)"),
    [84, 67, 79, 77] => frame_info!([TextEncoding,StringList,], "Composer"),
    [84, 67, 79, 78] => frame_info!([TextEncoding,StringList,], "Content type"),
    [84, 67, 79, 80] => frame_info!([TextEncoding,StringList,], "Copyright message"),
    [84, 68, 65, 84] => frame_info!([TextEncoding,StringList,], "Date"),
    [84, 68, 76, 89] => frame_info!([TextEncoding,StringList,], "Playlist delay"),
    [84, 69, 78, 67] => frame_info!([TextEncoding,StringList,], "Encoded by"),
    [84, 69, 88, 84] => frame_info!([TextEncoding,StringList,], "Lyricist/Text writer"),
    [84, 70, 76, 84] => frame_info!([TextEncoding,StringList,], "File type"),
    [84, 73, 77, 69] => frame_info!([TextEncoding,StringList,], "Time"),
    [84, 73, 84, 49] => frame_info!([TextEncoding,StringList,], "Content group description"),
    [84, 73, 84, 50] => frame_info!([TextEncoding,StringList,], "Title/songname/content description"),
    [84, 73, 84, 51] => frame_info!([TextEncoding,StringList,], "Subtitle/Description refinement"),
    [84, 75, 69, 89] => frame_info!([TextEncoding,StringList,], "Initial key"),
    [84, 76, 65, 78] => frame_info!([TextEncoding,StringList,], "Language(s)"),
    [84, 76, 69, 78] => frame_info!([TextEncoding,StringList,], "Length"),
    [84, 77, 69, 68] => frame_info!([TextEncoding,StringList,], "Media type"),
    [84, 79, 65, 76] => frame_info!([TextEncoding,StringList,], "Original album/movie/show title"),
    [84, 79, 70, 78] => frame_info!([TextEncoding,StringList,], "Original filename"),
    [84, 79, 76, 89] => frame_info!([TextEncoding,StringList,], "Original lyricist(s)/text writer(s)"),
    [84, 79, 80, 69] => frame_info!([TextEncoding,StringList,], "Original artist(s)/performer(s)"),
    [84, 79, 82, 89] => frame_info!([TextEncoding,StringList,], "Original release year"),
    [84, 79, 87, 78] => frame_info!([TextEncoding,StringList,], "File owner/licensee"),
    [84, 80, 69, 49] => frame_info!([TextEncoding,StringList,], "Lead performer(s)/Soloist(s)"),
    [84, 80, 69, 50] => frame_info!([TextEncoding,StringList,], "Band/orchestra/accompaniment"),
    [84, 80, 69, 51] => frame_info!([TextEncoding,StringList,], "Conductor/performer refinement"),
    [84, 80, 69, 52] => frame_info!([TextEncoding,StringList,], "Interpreted, remixed, or otherwise modified by"),
    [84, 80, 79, 83] => frame_info!([TextEncoding,StringList,], "Part of a set"),
    [84, 80, 85, 66] => frame_info!([TextEncoding,StringList,], "Publisher"),
    [84, 82, 67, 75] => frame_info!([TextEncoding,StringList,], "Track number/Position in set"),
    [84, 82, 68, 65] => frame_info!([TextEncoding,StringList,], "Recording dates"),
    [84, 82, 83, 78] => frame_info!([TextEncoding,StringList,], "Internet radio station name"),
    [84, 82, 83, 79] => frame_info!([TextEncoding,StringList,], "Internet radio station owner"),
    [84, 83, 73, 90] => frame_info!([TextEncoding,StringList,], "Size"),
    [84, 83, 79, 50] => frame_info!([TextEncoding,StringList,], "Album artist sort order"),
    [84, 83, 79, 67] => frame_info!([TextEncoding,StringList,], "Composer sort order"),
    [84, 83, 82, 67] => frame_info!([TextEncoding,StringList,], "ISRC (international standard recording code)"),
    [84, 83, 83, 69] => frame_info!([TextEncoding,StringList,], "Software/Hardware and settings used for encoding"),
    [84, 89, 69, 82] => frame_info!([TextEncoding,StringList,], "Year"),
    [84, 88, 88, 88] => frame_info!([TextEncoding,String,String,], "User defined text information frame"),

    [85, 70, 73, 68] => frame_info!([Latin1,BinaryData,], "Unique file identifier"),
    [85, 83, 69, 82] => frame_info!([TextEncoding,Language,String,], "Terms of use"),
    [85, 83, 76, 84] => frame_info!([TextEncoding,Language,String,StringFull,], "Unsynchronised lyric/text transcription"),

    [87, 67, 79, 77] => frame_info!([Latin1,], "Commercial information"),
    [87, 67, 79, 80] => frame_info!([Latin1,], "Copyright/Legal information"),
    [87, 79, 65, 70] => frame_info!([Latin1,], "Official audio file webpage"),
    [87, 79, 65, 82] => frame_info!([Latin1,], "Official artist/performer webpage"),
    [87, 79, 65, 83] => frame_info!([Latin1,], "Official audio source webpage"),
    [87, 79, 82, 83] => frame_info!([Latin1,], "Official Internet radio station homepage"),
    [87, 80, 65, 89] => frame_info!([Latin1,], "Payment"),
    [87, 80, 85, 66] => frame_info!([Latin1,], "Publishers official webpage"),
    [87, 88, 88, 88] => frame_info!([TextEncoding,String,Latin1,], "User defined URL link frame"),

//TODO: see how this relates to specs
    [90, 79, 66, 83] => frame_info!([BinaryData,], "Obsolete frame"),
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

static ID_2_TO_3: phf::Map<[u8; 3], [u8; 4]> = phf_map! {
    [66, 85, 70] => [82, 66, 85, 70],

    [67, 78, 84] => [80, 67, 78, 84],
    [67, 79, 77] => [67, 79, 77, 77],
    [67, 82, 65] => [65, 69, 78, 67],

    [69, 84, 67] => [69, 84, 67, 79],

    [71, 69, 79] => [71, 69, 79, 66],

    [73, 80, 76] => [73, 80, 76, 83],

    [76, 78, 75] => [76, 73, 78, 75],

    [77, 67, 73] => [77, 67, 68, 73],
    [77, 76, 76] => [77, 76, 76, 84],

    [80, 73, 67] => [65, 80, 73, 67],
    [80, 79, 80] => [80, 79, 80, 77],

    [82, 69, 86] => [82, 86, 82, 66],

    [83, 76, 84] => [83, 89, 76, 84],
    [83, 84, 67] => [83, 89, 84, 67],

    [84, 65, 76] => [84, 65, 76, 66],
    [84, 66, 80] => [84, 66, 80, 77],
    [84, 67, 77] => [84, 67, 79, 77],
    [84, 67, 79] => [84, 67, 79, 78],
    [84, 67, 82] => [84, 67, 79, 80],
    [84, 68, 89] => [84, 68, 76, 89],
    [84, 69, 78] => [84, 69, 78, 67],
    [84, 70, 84] => [84, 70, 76, 84],
    [84, 75, 69] => [84, 75, 69, 89],
    [84, 76, 65] => [84, 76, 65, 78],
    [84, 76, 69] => [84, 76, 69, 78],
    [84, 77, 84] => [84, 77, 69, 68],
    [84, 79, 65] => [84, 79, 80, 69],
    [84, 79, 70] => [84, 79, 70, 78],
    [84, 79, 76] => [84, 79, 76, 89],
    [84, 79, 84] => [84, 79, 65, 76],
    [84, 80, 49] => [84, 80, 69, 49],
    [84, 80, 50] => [84, 80, 69, 50],
    [84, 80, 51] => [84, 80, 69, 51],
    [84, 80, 52] => [84, 80, 69, 52],
    [84, 80, 65] => [84, 80, 79, 83],
    [84, 80, 66] => [84, 80, 85, 66],
    [84, 82, 67] => [84, 83, 82, 67],
    [84, 82, 75] => [84, 82, 67, 75],
    [84, 83, 83] => [84, 83, 83, 69],
    [84, 84, 49] => [84, 73, 84, 49],
    [84, 84, 50] => [84, 73, 84, 50],
    [84, 84, 51] => [84, 73, 84, 51],
    [84, 88, 84] => [84, 69, 88, 84],
    [84, 88, 88] => [84, 88, 88, 88],
    [84, 89, 69] => [84, 89, 69, 82],

    [85, 70, 73] => [85, 70, 73, 68],
    [85, 76, 84] => [85, 83, 76, 84],

    [87, 65, 70] => [87, 79, 65, 70],
    [87, 65, 82] => [87, 79, 65, 82],
    [87, 65, 83] => [87, 79, 65, 83],
    [87, 67, 77] => [87, 67, 79, 77],
    [87, 67, 80] => [87, 67, 79, 80],
    [87, 80, 66] => [87, 80, 85, 66],
    [87, 88, 88] => [87, 88, 88, 88],
};

/// Returns the corresponding ID3v2.3/ID3v2.4 ID given the ID3v2.2 ID. 
#[inline]
pub fn convert_id_2_to_3(id: [u8; 3]) -> Option<[u8; 4]> {
    ID_2_TO_3.get(&id).map(|t| *t)
}

static ID_3_TO_2: phf::Map<[u8; 4], [u8; 3]> = phf_map! {
    [82, 66, 85, 70] => [66, 85, 70],

    [80, 67, 78, 84] => [67, 78, 84],
    [67, 79, 77, 77] => [67, 79, 77],
    [65, 69, 78, 67] => [67, 82, 65],

    [69, 84, 67, 79] => [69, 84, 67],

    [71, 69, 79, 66] => [71, 69, 79],

    [73, 80, 76, 83] => [73, 80, 76],

    [76, 73, 78, 75] => [76, 78, 75],

    [77, 67, 68, 73] => [77, 67, 73],
    [77, 76, 76, 84] => [77, 76, 76],

    [65, 80, 73, 67] => [80, 73, 67],
    [80, 79, 80, 77] => [80, 79, 80],

    [82, 86, 82, 66] => [82, 69, 86],

    [83, 89, 76, 84] => [83, 76, 84],
    [83, 89, 84, 67] => [83, 84, 67],

    [84, 65, 76, 66] => [84, 65, 76],
    [84, 66, 80, 77] => [84, 66, 80],
    [84, 67, 79, 77] => [84, 67, 77],
    [84, 67, 79, 78] => [84, 67, 79],
    [84, 67, 79, 80] => [84, 67, 82],
    [84, 68, 76, 89] => [84, 68, 89],
    [84, 69, 78, 67] => [84, 69, 78],
    [84, 70, 76, 84] => [84, 70, 84],
    [84, 75, 69, 89] => [84, 75, 69],
    [84, 76, 65, 78] => [84, 76, 65],
    [84, 76, 69, 78] => [84, 76, 69],
    [84, 77, 69, 68] => [84, 77, 84],
    [84, 79, 80, 69] => [84, 79, 65],
    [84, 79, 70, 78] => [84, 79, 70],
    [84, 79, 76, 89] => [84, 79, 76],
    [84, 79, 65, 76] => [84, 79, 84],
    [84, 80, 69, 49] => [84, 80, 49],
    [84, 80, 69, 50] => [84, 80, 50],
    [84, 80, 69, 51] => [84, 80, 51],
    [84, 80, 69, 52] => [84, 80, 52],
    [84, 80, 79, 83] => [84, 80, 65],
    [84, 80, 85, 66] => [84, 80, 66],
    [84, 83, 82, 67] => [84, 82, 67],
    [84, 82, 67, 75] => [84, 82, 75],
    [84, 83, 83, 69] => [84, 83, 83],
    [84, 73, 84, 49] => [84, 84, 49],
    [84, 73, 84, 50] => [84, 84, 50],
    [84, 73, 84, 51] => [84, 84, 51],
    [84, 69, 88, 84] => [84, 88, 84],
    [84, 88, 88, 88] => [84, 88, 88],
    [84, 89, 69, 82] => [84, 89, 69],

    [85, 70, 73, 68] => [85, 70, 73],
    [85, 83, 76, 84] => [85, 76, 84],

    [87, 79, 65, 70] => [87, 65, 70],
    [87, 79, 65, 82] => [87, 65, 82],
    [87, 79, 65, 83] => [87, 65, 83],
    [87, 67, 79, 77] => [87, 67, 77],
    [87, 67, 79, 80] => [87, 67, 80],
    [87, 80, 85, 66] => [87, 80, 66],
    [87, 88, 88, 88] => [87, 88, 88],
};

/// Returns the corresponding ID3v2.2 ID given the ID3v2.3/ID3v2.3 ID. 
#[inline]
pub fn convert_id_3_to_2(id: [u8; 4]) -> Option<[u8; 3]> {
    ID_3_TO_2.get(&id).map(|t| *t)
}
