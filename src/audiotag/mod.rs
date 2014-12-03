//! A deprecated abstraction over various kinds of audio tags
#![warn(missing_docs)]

use std::fmt;
use std::io::{self, Read, Write, Seek};
use std::path::Path;
use std::error::{Error};

/// A result encapsulating a TagError or a value type.
pub type TagResult<T> = Result<T, TagError>;

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error kind indicating that an IO error has occurred. Contains the original Error.
    InternalIoError(io::Error),
    /// An error kind indicating that a string decoding error has occurred. Contains the invalid
    /// bytes.
    StringDecodingError(Vec<u8>),
    /// An error kind indicating that some input was invalid.
    InvalidInputError,
    /// An error kind indicating that a feature is not supported.
    UnsupportedFeatureError
}

/// A structure able to represent any error that may occur while performing metadata operations.
pub struct TagError {
    /// The kind of error.
    pub kind: ErrorKind,
    /// A human readable string describing the error.
    pub description: &'static str,
}

impl TagError {
    /// Creates a new `TagError` using the error kind and description.
    pub fn new(kind: ErrorKind, description: &'static str) -> TagError {
        TagError { kind: kind, description: description }
    }

    /// Returns true of the error kind is `InternalIoError`.
    pub fn is_io_error(&self) -> bool {
        match self.kind {
            ErrorKind::InternalIoError(_) => true,
            _ => false
        }
    }

    /// Returns the `IoError` contained in `InternalIoError`. Panics if called on a non
    /// `InternalIoError` value.
    pub fn io_error(&self) -> &io::Error {
        match self.kind {
            ErrorKind::InternalIoError(ref err) => err,
            _ => panic!("called ErrorKind::io_error() on a non `InternalIoError` value") 
        }
    }
}

impl Error for TagError {
    fn description(&self) -> &str {
        if self.cause().is_some() {
            self.cause().unwrap().description()
        } else if self.is_io_error() {
            self.io_error().description()
        } else {
            self.description
        }
    }
}

impl From<io::Error> for TagError {
    fn from(err: io::Error) -> TagError {
        TagError { kind: ErrorKind::InternalIoError(err), description: "" }
    }
}

impl fmt::Debug for TagError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(out, "{:?}: {}", self.kind, self.description())
        } else {
            write!(out, "{}", self.description())
        }
    }
}

impl fmt::Display for TagError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        if self.description != "" {
            write!(out, "{:?}: {}", self.kind, self.description())
        } else {
            write!(out, "{}", self.description())
        }
    }
}

/// A trait defining generic methods to read/write audio metadata.
pub trait AudioTag {
    /// Returns the contents of the file at the specified path without the audio metadata.
    fn skip_metadata<R: Read + Seek>(reader: &mut R, _: Option<Self>) -> Vec<u8>;

    /// Returns true if the reader might contain valid metadata.
    fn is_candidate(reader: &mut Read, _: Option<Self>) -> bool;

    /// Creates a tag by reading data from the reader.
    fn read_from(reader: &mut Read) -> TagResult<Self>;

    /// Writes the tag to the writer.
    fn write_to(&mut self, writer: &mut Write) -> TagResult<()>;

    /// Creates a tag by reading data at the specified path.
    fn read_from_path(path: &Path) -> TagResult<Self>;

    /// Writes the tag to the specified path while preserving the original file data.
    fn write_to_path(&mut self, path: &Path) -> TagResult<()>;

    /// Saves the tag back to the path from which it was read. Panics if the tag was not read from
    /// a path.
    fn save(&mut self) -> TagResult<()>;
    
    /// Returns the artist or `None` if the artist is not specified.
    fn artist(&self) -> Option<String>;
    /// Sets the artist.
    fn set_artist<T: Into<String>>(&mut self, artist: T);
    /// Removes the artist.
    fn remove_artist(&mut self);

    /// Returns the album artist or `None` if the album artist is not specified.
    fn album_artist(&self) -> Option<String>;
    /// Sets the album artist.
    fn set_album_artist<T: Into<String>>(&mut self, album_artist: T);
    /// Removes the album artist.
    fn remove_album_artist(&mut self);

    /// Returns the album or `None` if the album is not specified.
    fn album(&self) -> Option<String>;
    /// Sets the album.
    fn set_album<T: Into<String>>(&mut self, album: T);
    /// Removes the album.
    fn remove_album(&mut self);

    /// Returns the genre or `None` if the genre is not specified.
    fn genre(&self) -> Option<String>;
    /// Sets the genre.
    fn set_genre<T: Into<String>>(&mut self, genre: T);
    /// Removes the genre.
    fn remove_genre(&mut self);

    /// Returns the title or `None` if the title is not specified.
    fn title(&self) -> Option<String>;
    /// Sets the title.
    fn set_title<T: Into<String>>(&mut self, title: T);
    /// Removes the title.
    fn remove_title(&mut self);

    /// Returns the track or `None` if the track is not specified.
    fn track(&self) -> Option<u32>;
    /// Sets the track.
    fn set_track(&mut self, track: u32);
    /// Removes the track and the total number of tracks.
    fn remove_track(&mut self);

    /// Returns the total number of tracks or `None` if the total number of tracks is not
    /// specified.
    fn total_tracks(&self) -> Option<u32>;
    /// Sets the total number of tracks.
    fn set_total_tracks(&mut self, total_tracks: u32);
    /// Removes the total number of tracks.
    fn remove_total_tracks(&mut self);

    /// Returns the lyrics of `None` if the lyrics are not specified.
    fn lyrics(&self) -> Option<String>;
    /// Sets the lyrics.
    fn set_lyrics<T: Into<String>>(&mut self, lyrics: T);
    /// Removes the lyrics.
    fn remove_lyrics(&mut self);

    /// Sets the picture.
    fn set_picture<T: Into<String>>(&mut self, mime_type: T, data: Vec<u8>);
    /// Removes the picture;
    fn remove_picture(&mut self);

    /// Returns all the metadata in key, value tuples.
    fn all_metadata(&self) -> Vec<(String, String)>;
}

