#![feature(globs)]

extern crate id3;

use id3::{AudioTag, id3v2, Frame, Encoding};
use id3::tag::FileTags;
use id3::id3v2::SupportedVersion::*;

static ID: &'static str = "TRCK";
static TRACK: u32 = 5;
static TOTAL: u32 = 10;
static INVALID: &'static str = "invalid";

// UTF8 {{{
#[test]
fn utf8() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF8);
    tag.v2.as_mut().unwrap().set_total_tracks_enc(TOTAL, Encoding::UTF8);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert_eq!(tag.total_tracks().unwrap(), TOTAL);
    assert_eq!(frame.content.text().as_slice(), format!("{}/{}", TRACK, TOTAL).as_slice());

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", TRACK, TOTAL).into_bytes().into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf8_only_track() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF8);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert!(tag.total_tracks().is_none());
    assert_eq!(frame.text().unwrap().as_slice(), format!("{}", TRACK).as_slice());
    assert_eq!(frame.content.text().as_slice(), format!("{}", TRACK).as_slice());

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}", TRACK).into_bytes().into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf8_invalid() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));
    
    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", INVALID, TOTAL).into_bytes().into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());

    tag.v2.as_mut().unwrap().remove_frames_by_id(ID);

    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF8 as u8);
    data.extend(format!("{}/{}", TRACK, INVALID).into_bytes().into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());
}
//}}}

// UTF16 {{{
#[test]
fn utf16() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF16);
    tag.v2.as_mut().unwrap().set_total_tracks_enc(TOTAL, Encoding::UTF16);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert_eq!(tag.total_tracks().unwrap(), TOTAL);
    assert_eq!(frame.content.text().as_slice(), format!("{}/{}", TRACK, TOTAL).as_slice());

    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(format!("{}/{}", TRACK, TOTAL).as_slice()).into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf16_only_track() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF16);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert!(tag.total_tracks().is_none());
    assert_eq!(frame.content.text().as_slice(), format!("{}", TRACK).as_slice());

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(format!("{}", TRACK).as_slice()).into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf16_invalid() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));
    
    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(format!("{}/{}", INVALID, TOTAL).as_slice()).into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());

    tag.v2.as_mut().unwrap().remove_frames_by_id(ID);

    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF16 as u8);
    data.extend(id3::util::string_to_utf16(format!("{}/{}", TRACK, INVALID).as_slice()).into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());
}
//}}}

// UTF16BE {{{
#[test]
fn utf16be() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF16BE);
    tag.v2.as_mut().unwrap().set_total_tracks_enc(TOTAL, Encoding::UTF16BE);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert_eq!(tag.total_tracks().unwrap(), TOTAL);
    assert_eq!(frame.content.text().as_slice(), format!("{}/{}", TRACK, TOTAL).as_slice());

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(format!("{}/{}", TRACK, TOTAL).as_slice()).into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf16be_only_track() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));

    tag.v2.as_mut().unwrap().set_track_enc(TRACK, Encoding::UTF16BE);
    let frame = tag.v2.as_ref().unwrap().get_frame_by_id(ID).unwrap();

    assert_eq!(tag.track().unwrap(), TRACK);
    assert!(tag.total_tracks().is_none());
    assert_eq!(frame.content.text().as_slice(), format!("{}", TRACK).as_slice());

    let mut data: Vec<u8> = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(format!("{}", TRACK).as_slice()).into_iter());
    assert_eq!(frame.content_to_bytes(), data);
}

#[test]
fn utf16be_invalid() {
    let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::with_version(V2_4)));
    
    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(format!("{}/{}", INVALID, TOTAL).as_slice()).into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());

    tag.v2.as_mut().unwrap().remove_frames_by_id(ID);

    let mut frame = Frame::with_version(ID.into_string(), 4);
    let mut data = Vec::new();
    data.push(Encoding::UTF16BE as u8);
    data.extend(id3::util::string_to_utf16be(format!("{}/{}", TRACK, INVALID).as_slice()).into_iter());
    frame.parse_data(data.as_slice()).unwrap();
    tag.v2.as_mut().unwrap().add_frame(frame);
    assert!(tag.track().is_none());
    assert!(tag.total_tracks().is_none());
}
//}}}
