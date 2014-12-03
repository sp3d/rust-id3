use std::fmt;
use std::error;
use std::io;

/// Kinds of errors that may occur while performing metadata operations.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error kind indicating that an IO error has occurred. Contains the original Error.
    InternalIoError(io::Error),
    /// An error kind indicating that a string decoding error has occurred. Contains the invalid
    /// bytes.
    StringDecodingError(Vec<u8>),
    /// An error kind indicating that the tag was malformed.
    InvalidTag,
    /// An error kind indicating that a feature is not supported.
    UnsupportedFeature,
}

/// A structure able to represent any error that may occur while performing metadata operations.
pub struct Error {
    /// The kind of error.
    pub kind: ErrorKind,
    /// A human readable string describing the error.
    pub description: &'static str,
}

impl Error {
    /// Creates a new `Error` using the error kind and description.
    pub fn new(kind: ErrorKind, description: &'static str) -> Error {
        Error { kind: kind, description: description }
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

impl error::Error for Error {
    fn description(&self) -> &str {
        use std::error::Error;
        if self.cause().is_some() {
            self.cause().unwrap().description()
        } else if self.is_io_error() {
            self.io_error().description()
        } else {
            self.description
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error { kind: ErrorKind::InternalIoError(err), description: "" }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        if self.description != "" {
            write!(out, "{:?}: {}", self.kind, self.description())
        } else {
            write!(out, "{}", self.description())
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        if self.description != "" {
            write!(out, "{:?}: {}", self.kind, self.description())
        } else {
            write!(out, "{}", self.description())
        }
    }
}
