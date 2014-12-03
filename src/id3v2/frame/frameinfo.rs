use frame::field::FieldType;
use frame::field::FieldType::*;

static FRAME_FORMATS_V34: phf::Map<[u8, ..3], &'static [FieldType]> = phf_map! {
b!("UFID") => [Latin1,BinaryData,],
b!("TXXX") => [TextEncoding,String,String,],
b!("WXXX") => [TextEncoding,String,Latin1,],
b!("MCDI") => [BinaryData,],
b!("ETCO") => [Int8,BinaryData,],
b!("MLLT") => [Int16,Int24,Int24,Int8,Int8,BinaryData,],
b!("SYTC") => [Int8,BinaryData,],
b!("USLT") => [TextEncoding,Language,String,StringFULL,],
b!("SYLT") => [TextEncoding,Language,Int8,Int8,String,BinaryData,],
b!("COMM") => [TextEncoding,Language,String,StringFULL,],
b!("RVA2") => [Latin1,BinaryData,],
b!("EQU2") => [Int8,Latin1,BinaryData,],
b!("RVRB") => [Int16,Int16,Int8,Int8,Int8,Int8,Int8,Int8,Int8,Int8,],
b!("APIC") => [TextEncoding,Latin1,Int8,String,BinaryData,],
b!("GEOB") => [TextEncoding,Latin1,String,String,BinaryData,],
b!("PCNT") => [Int32Plus,],
b!("POPM") => [Latin1,Int8,Int32Plus,],
b!("RBUF") => [Int24,Int8,Int32,],
b!("AENC") => [Latin1,Int16,Int16,BinaryData,],
b!("LINK") => [FrameId,Latin1,Latin1List,],
b!("POSS") => [Int8,BinaryData,],
b!("USER") => [TextEncoding,Language,String,],
b!("OWNE") => [TextEncoding,Latin1,Date,String,],
b!("COMR") => [TextEncoding,Latin1,Date,Latin1,Int8,String,String,Latin1,BinaryData,],
b!("ENCR") => [Latin1,Int8,BinaryData,],
b!("GRID") => [Latin1,Int8,BinaryData,],
b!("PRIV") => [Latin1,BinaryData,],
b!("SIGN") => [Int8,BinaryData,],
b!("SEEK") => [Int32,],
b!("ASPI") => [Int32,Int32,Int16,Int8,BinaryData,],
b!("ZOBS") => [FrameId,BinaryData,],

b!("text") => [TextEncoding,StringList,],
b!("url") => [Latin1,],
b!("unknown") => [BinaryData,],
};

FRAME_DISCARD