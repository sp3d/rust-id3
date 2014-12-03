//! A library to read and write ID3v2 tags. ID3 versions v2.2, v2.3, and v2.4 are supported.
//! 
//! # Modifying an existing tag
//!
//! ```no_run
//! use id3::AudioTag;
//!
//! let mut tag = AudioTag::read_from_path(&Path::new("music.mp3")).unwrap();
//!
//! // print the artist the hard way
//! println!("{}", tag.v2.as_ref().unwrap().get_frame_by_id("TALB").unwrap().content.text());
//! 
//! // or print it the easy way
//! println!("{}", tag.artist().unwrap());
//!
//! tag.save().unwrap();
//! ```
//!
//! # Creating a new tag
//!
//! ```no_run
//! // you need to use AudioTag in order to use the trait features
//! use id3::{id3v2, AudioTag};
//! use id3::id3v2::Frame;
//! use id3::id3v2::Version::V4;
//! use id3::tag::FileTags;
//! use id3::Content::TextContent;
//! use id3::Encoding::UTF8;
//!
//! let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V4)));
//! 
//! // set the album the hard way
//! let mut frame = Frame::with_version("TALB".into_string(), 4);
//! frame.set_encoding(UTF8);
//! frame.content = TextContent("album".into_string());
//! tag.v2.as_mut().unwrap().add_frame(frame);
//!
//! // or set it the easy way
//! tag.set_album("album".into_string());
//!
//! tag.write_to_path(&Path::new("music.mp3")).unwrap();
//! ```

#![crate_name = "id3"]
#![crate_type = "dylib"]
#![warn(missing_docs)]
#![feature(plugin, slice_patterns, core, convert, collections, rustc_private)]
#![plugin(phf_macros)]

extern crate phf;

#[macro_use]
extern crate log;
extern crate num;

pub mod audiotag;
pub use self::audiotag::{AudioTag, TagResult, TagError, ErrorKind};

/// Utilities used for the data formats involved in reading/writing ID3 tags.
pub mod util;

/// Functionality for handling ID3v1 tags.
pub mod id3v1;
/// Functionality for handling ID3v2 tags.
pub mod id3v2;
/// Common functionality for handling ID3 tags in general.
pub mod tag;

mod parsers;
