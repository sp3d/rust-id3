use id3v2::frame::PictureType;

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
/// The parsed contents of an extended text frame.
pub struct ExtendedText {
    pub key: String,
    pub value: String
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
/// The parsed contents of an unsynchronized lyrics frame.
pub struct Lyrics {
    pub lang: String,
    pub description: String,
    pub text: String
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
/// The parsed contents of a comment frame.
pub struct Comment {
    pub lang: String,
    pub description: String,
    pub text: String
}

#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
/// The parsed contents of an extended link frame.
pub struct ExtendedLink {
    pub description: String,
    pub link: String
}

/// A structure representing an ID3 picture frame's contents.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    /// The picture's MIME type.
    pub mime_type: String,
    /// The type of picture.
    pub picture_type: PictureType,
    /// A description of the picture's contents.
    pub description: String,
    /// The image data.
    pub data: Vec<u8>
}

impl Picture {
    /// Creates a new `Picture` with empty values.
    pub fn new() -> Picture {
        Picture { mime_type: String::new(), picture_type: PictureType::Other, description: String::new(), data: Vec::new() } 
    }
}
