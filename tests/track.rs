extern crate id3;

use id3::id3v2;
use id3::id3v2::Version::*;
use id3::id3v2::frame::{Frame, Id, Encoding};

static ID: Id = Id::V4(*b"TRCK");
static TRACK: u32 = 5;
static TOTAL: u32 = 10;
static INVALID: &'static str = "invalid";

// UTF8 {{{
#[test]
fn utf8() {
    let mut v2 = id3v2::Tag::with_version(V4);

    v2.set_track_enc(TRACK, Encoding::UTF8);
    v2.set_total_tracks_enc(TOTAL, Encoding::UTF8);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}/{}", TRACK, TOTAL)));

    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", TRACK, TOTAL).into_bytes().into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf8_only_track() {
    let mut v2 = id3v2::Tag::with_version(V4);

    v2.set_track_enc(TRACK, Encoding::UTF8);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}", TRACK)));

    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}", TRACK).into_bytes().into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf8_invalid() {
    let mut v2 = id3v2::Tag::with_version(V4);
    
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", INVALID, TOTAL).into_bytes().into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);
    v2.remove_frames_by_id(ID);

    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", TRACK, INVALID).into_bytes().into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);
}
//}}}

// UTF16 {{{
#[test]
fn utf16() {
    let mut v2 = id3v2::Tag::with_version(V4);

    v2.set_track_enc(TRACK, Encoding::UTF16);
    v2.set_total_tracks_enc(TOTAL, Encoding::UTF16);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}/{}", TRACK, TOTAL)));
    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(&*format!("{}/{}", TRACK, TOTAL)).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16_only_track() {
    let mut v2 = id3v2::Tag::with_version(V4);

    v2.set_track_enc(TRACK, Encoding::UTF16);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}", TRACK)));

    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(&*format!("{}", TRACK)).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16_invalid() {
    let mut v2 = id3v2::Tag::with_version(V4);
    
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(&*format!("{}/{}", INVALID, TOTAL)).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);
    v2.remove_frames_by_id(ID);

    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(&*format!("{}/{}", TRACK, INVALID)).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);
}
//}}}

// UTF16BE {{{
#[test]
fn utf16be() {
    let mut v2 = id3v2::Tag::with_version(V4);

    v2.set_track_enc(TRACK, Encoding::UTF16BE);
    v2.set_total_tracks_enc(TOTAL, Encoding::UTF16BE);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}/{}", TRACK, TOTAL)));

    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(&*format!("{}/{}", TRACK, TOTAL)).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16be_only_track() {
    let mut v2 = id3v2::Tag::with_version(V4);
    v2.set_track_enc(TRACK, Encoding::UTF16BE);
    assert_eq!(v2.text_frame_text(ID), Some(format!("{}", TRACK)));

    let frame = v2.get_frame_by_id(ID).unwrap();

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(&*format!("{}", TRACK)).into_iter());
    assert_eq!(frame.fields_to_bytes(), data);
}

#[test]
fn utf16be_invalid() {
    let mut v2 = id3v2::Tag::with_version(V4);
    
    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(&*format!("{}/{}", INVALID, TOTAL)).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);

    v2.remove_frames_by_id(ID);

    let mut frame = Frame::new(ID);
    let mut data = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(&*format!("{}/{}", TRACK, INVALID)).into_iter());
    frame.fields = frame.parse_fields(&*data).unwrap();
    v2.add_frame(frame);
}
//}}}
