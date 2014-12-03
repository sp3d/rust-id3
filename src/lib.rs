//! A library to read and write ID3v2 tags. ID3 versions v2.2, v2.3, and v2.4 are supported.
//! 
//! # Modifying an existing tag
//!
//! ```no_run
//! use id3::FileTags;
//! use id3::id3v2::frame::Id;
//!
//! let mut path = &std::path::Path::new("music.mp3");
//! let mut tags = FileTags::from_path(path).unwrap();
//!
//! // print the artist
//! println!("{}", tags.v2.as_ref().unwrap().text_frame_text(Id::V4(*b"TALB")).unwrap());
//!
//! ```
//!
//! # Creating a new tag
//!
//! ```no_run
//! use id3::id3v2;
//! use id3::id3v2::frame::{Frame, Id};
//! use id3::id3v2::Version::V4;
//! use id3::FileTags;
//! use id3::id3v2::frame::Encoding;
//!
//! let mut tags = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V4)));
//! 
//! // set the album the hardest way
//! let mut frame = Frame::new_text_frame(Id::V4(*b"TALB"), "my album", Encoding::UTF8);
//! tags.v2.as_mut().unwrap().add_frame(frame.unwrap());
//!
//! // or the slightly easier way
//! tags.v2.as_mut().unwrap().add_text_frame(Id::V4(*b"TALB"), "my album");
//!
//! tags.store_at_path(&std::path::Path::new("music.mp3")).unwrap();
//! ```

#![crate_name = "id3"]
#![crate_type = "dylib"]
#![warn(missing_docs)]
#![feature(plugin, slice_patterns, rustc_private, vec_push_all, slice_bytes, str_utf16, iter_arith)]
#![plugin(phf_macros)]

extern crate phf;

#[macro_use]
extern crate log;
extern crate num;

/// Utilities used for the data formats involved in reading/writing ID3 tags.
pub mod util;

/// Functionality for handling ID3v1 tags.
pub mod id3v1;
/// Functionality for handling ID3v2 tags.
pub mod id3v2;

mod filetags;

/// Common functionality for handling ID3 tags in files.
pub use filetags::FileTags;

mod parsers;
