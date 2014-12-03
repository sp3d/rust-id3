#![feature(vec_push_all)]

extern crate id3;

use id3::id3v2;
use id3::id3v2::Version::*;
use id3::id3v2::frame::{Frame, Id, Encoding};
use id3::id3v2::simple::Simple;

static ID: Id = Id::V4(*b"TYER");
static YEAR: usize = 2014;
static YEARSTR: &'static str = "2014";
static INVALID: &'static str = "invalid";

// UTF8 {{{
#[test]
fn utf8() {
    let mut tag = id3v2::Tag::with_version(V4);

    tag.set_year_enc(YEAR, Encoding::UTF8);
    let frame = tag.get_frame_by_id(ID).unwrap();
    
    assert_eq!(tag.year().unwrap(), YEAR);
    assert_eq!(tag.text_frame_text(ID), Some(YEARSTR.to_owned()));

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.push_all(YEARSTR.as_bytes());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf8_invalid() {
    let mut tag = id3v2::Tag::with_version(V4);
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.push_all(INVALID.as_bytes());
    frame.fields = frame.parse_fields(&*data).unwrap();
    tag.add_frame(frame);
    assert!(tag.year().is_none());
}
//}}}

// UTF16 {{{
#[test]
fn utf16() {
    let mut tag = id3v2::Tag::with_version(V4);

    tag.set_year_enc(YEAR, Encoding::UTF16);
    let frame = tag.get_frame_by_id(ID).unwrap();

    assert_eq!(tag.year().unwrap(), YEAR);
    assert_eq!(tag.text_frame_text(ID), Some(YEARSTR.to_owned()));

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(YEARSTR).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16_invalid() {
    let mut tag = id3v2::Tag::with_version(V4);
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(INVALID).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    tag.add_frame(frame);
    assert!(tag.year().is_none());
}
//}}}

// UTF16BE {{{
#[test]
fn utf16be() {
    let mut tag = id3v2::Tag::with_version(V4);

    tag.set_year_enc(YEAR, Encoding::UTF16BE);
    let frame = tag.get_frame_by_id(ID).unwrap();

    assert_eq!(tag.year().unwrap(), YEAR);
    assert_eq!(tag.text_frame_text(ID), Some(YEARSTR.to_owned()));

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(YEARSTR).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16be_invalid() {
    let mut tag = id3v2::Tag::with_version(V4);
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(INVALID).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    tag.add_frame(frame);
    assert!(tag.year().is_none());
}
//}}}
