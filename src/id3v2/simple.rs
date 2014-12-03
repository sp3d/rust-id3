#![allow(missing_docs, unused, unused_variables)]

use id3v2::Tag;
use id3v2::frame::{PictureType, Id, Field, Frame, Encoding};

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

#[derive(Debug, Clone, PartialEq)]
/// A structure representing an ID3 picture frame's contents.
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


/// Simple and wrong accessors for simple interpretations of common frames
pub trait Simple
{
    fn txxx(&self) -> Vec<(String, String)>;
    fn add_txxx(&mut self, key: &str, value: &str);
    fn add_txxx_enc(&mut self, key: &str, value: &str, encoding: Encoding);
    fn remove_txxx(&mut self, key: Option<&str>, val: Option<&str>);
    fn pictures(&self) -> Vec<&Picture>;
    fn add_picture(&mut self, mime_type: &str, picture_type: PictureType, data: Vec<u8>);
    fn add_picture_enc(&mut self, mime_type: &str, picture_type: PictureType, description: &str, data: Vec<u8>, encoding: Encoding);
    fn remove_picture_type(&mut self, picture_type: PictureType);
    fn comments(&self) -> Vec<(String, String)>;
    fn add_comment(&mut self, description: &str, text: &str);
    fn add_comment_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding);
    fn remove_comment(&mut self, description: Option<&str>, text: Option<&str>);
    fn set_artist_enc(&mut self, artist: &str, encoding: Encoding);
    fn set_album_artist_enc(&mut self, album_artist: &str, encoding: Encoding);
    fn set_album_enc(&mut self, album: &str, encoding: Encoding);
    fn set_title_enc(&mut self, title: &str, encoding: Encoding);
    fn set_genre_enc(&mut self, genre: &str, encoding: Encoding);
    fn year(&self) -> Option<usize>;
    fn set_year(&mut self, year: usize);
    fn set_year_enc(&mut self, year: usize, encoding: Encoding);
    fn track_pair(&self) -> Option<(u32, Option<u32>)>;
    fn set_track_enc(&mut self, track: u32, encoding: Encoding);
    fn set_total_tracks_enc(&mut self, total_tracks: u32, encoding: Encoding);
    fn set_lyrics_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding);
}

impl Simple for Tag {
    /// Returns a vector of the user defined text frames' (TXXX) key/value pairs.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    /// use id3::id3v2::frame;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx("key1", "value1");
    /// tag.add_txxx("key2", "value2");
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.txxx().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    fn txxx(&self) -> Vec<(String, String)> {
        //use std::collections::string::String;
        let mut out = Vec::new();
        for frame in self.get_frames_by_id(self.version().txxx_id()).iter() {
            match &*frame.fields {
                [Field::TextEncoding(_encoding), Field::String(ref k), Field::String(ref v)] => {
                    //TODO(sp3d): convert encoding?
                    out.push((String::from_utf8(k.clone()).unwrap(), String::from_utf8(v.clone()).unwrap()));
                },
                _ => {},
            }
        }

        out
    }

    /// Adds a user defined text frame (TXXX).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx("key1", "value1");
    /// tag.add_txxx("key2", "value2");
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.txxx().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    #[inline]
    //fn add_txxx<E: Encoding>(&mut self, key: EncodedString<E>, value: EncodedString<E>) {
    fn add_txxx(&mut self, key: &str, value: &str) {
        let encoding = self.version().default_encoding();
        self.add_txxx_enc(key, value, encoding);
    }

    /// Adds a user defined text frame (TXXX) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx_enc("key1", "value1", UTF16);
    /// tag.add_txxx_enc("key2", "value2", UTF16);
    ///
    /// assert_eq!(tag.txxx().len(), 2);
    /// assert!(tag.txxx().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.txxx().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    //TODO(sp3d): there has to be a better way of dealing with encoded strings!
    fn add_txxx_enc(&mut self, key: &str, value: &str, encoding: Encoding) {
        let key = key.to_owned();

        self.remove_txxx(Some(&key), None);

        let mut frame = Frame::new(self.version().txxx_id());
        frame.set_encoding(encoding);
        //TODO(sp3d): rebuild this on top of fields
        /*frame.fields = ExtendedTextContent(frame::ExtendedText {
            key: key,
            value: value.to_owned()
        });*/

        self.frames.push(frame);
    }

    /// Removes the user defined text frame (TXXX) with the specified key and value.
    /// A key or value may be `None` to specify a wildcard value.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_txxx("key1", "value1");
    /// tag.add_txxx("key2", "value2");
    /// tag.add_txxx("key3", "value2");
    /// tag.add_txxx("key4", "value3");
    /// tag.add_txxx("key5", "value4");
    /// assert_eq!(tag.txxx().len(), 5);
    ///
    /// tag.remove_txxx(Some("key1"), None);
    /// assert_eq!(tag.txxx().len(), 4);
    ///
    /// tag.remove_txxx(None, Some("value2"));
    /// assert_eq!(tag.txxx().len(), 2);
    ///
    /// tag.remove_txxx(Some("key4"), Some("value3"));
    /// assert_eq!(tag.txxx().len(), 1);
    ///
    /// tag.remove_txxx(None, None);
    /// assert_eq!(tag.txxx().len(), 0);
    /// ```
    fn remove_txxx(&mut self, key: Option<&str>, val: Option<&str>) {
        let id = self.version().txxx_id();
        self.frames.retain(|frame| {
            let mut key_match = false;
            let mut val_match = false;

            if frame.id == id {
                match &*frame.fields {
                    [Field::TextEncoding(_), Field::String(ref f_key), Field::String(ref f_val)] => {
                        //TODO(sp3d): checking byte equality is wrong; encodings need to be considered
                        key_match = key.unwrap_or("").as_bytes() == &**f_key;
                        val_match = val.unwrap_or("").as_bytes() == &**f_val;
                    },
                    _ => {
                        // remove frames that we can't parse
                        key_match = true;
                        val_match = true;
                    }
                }
            }

            !(key_match && val_match) // true if we want to keep the item
        });
    }

    /// Returns a vector of references to the pictures in the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    /// use id3::id3v2::frame::Picture;
    /// use id3::Content::PictureContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// let mut frame = Frame::new(Id::V4(*b"APIC"));
    /// let picture = Picture {
    ///     mime_type: String::new(),
    ///     picture_type: PictureType::Other,
    ///     description: String::new(),
    ///     data: Vec::new()
    /// };
    ////
    /// let picture = Picture {
    ///     mime_type: String::new(),
    ///     picture_type: PictureType::Other,
    ///     description: String::new(),
    ///     data: Vec::new()
    /// };
    /// 
    /// let mut frame = Frame::new(Id::V4(*b"APIC"));
    /// frame.fields = PictureContent(picture);
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.pictures().len(), 2);
    /// ```
    fn pictures(&self) -> Vec<&Picture> {
        //TODO(sp3d): rebuild this on top of fields
        let mut pictures = Vec::new();
        for frame in self.get_frames_by_id(self.version().picture_id()).iter() {
            match &frame.fields {
                _ => { }
            }
        }
        pictures
    }

    /// Adds a picture frame (APIC).
    /// Any other pictures with the same type will be removed from the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::PictureType::Other;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture("image/jpeg", Other, vec!());
    /// tag.add_picture("image/png", Other, vec!());
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(&tag.pictures()[0].mime_type, "image/png");
    /// ```
    #[inline]
    fn add_picture(&mut self, mime_type: &str, picture_type: PictureType, data: Vec<u8>) {
        self.add_picture_enc(mime_type, picture_type, "", data, Encoding::Latin1);
    }

    /// Adds a picture frame (APIC) using the specified text encoding.
    /// Any other pictures with the same type will be removed from the tag.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::PictureType::Other;
    /// use id3::id3v2::frame::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture_enc("image/jpeg", Other, "", vec!(), UTF16);
    /// tag.add_picture_enc("image/png", Other, "", vec!(), UTF16);
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(&tag.pictures()[0].mime_type, "image/png");
    /// ```
    fn add_picture_enc(&mut self, mime_type: &str, picture_type: PictureType, description: &str, data: Vec<u8>, encoding: Encoding) {
        //TODO(sp3d): rebuild this on top of fields
        /*
        self.remove_picture_type(picture_type);

        let mut frame = Frame::new(self.version().picture_id());

        frame.set_encoding(encoding);
        frame.fields = PictureContent(Picture {
            mime_type: mime_type.to_owned(),
            picture_type: picture_type,
            description: description.to_owned(),
            data: data
        });

        self.frames.push(frame);
        */
    }

    /// Removes all pictures of the specified type.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::PictureType::{CoverFront, Other};
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.add_picture("image/jpeg", CoverFront, vec!());
    /// tag.add_picture("image/png", Other, vec!());
    /// assert_eq!(tag.pictures().len(), 2);
    ///
    /// tag.remove_picture_type(CoverFront);
    /// assert_eq!(tag.pictures().len(), 1);
    /// assert_eq!(tag.pictures()[0].picture_type, Other);
    /// ```
    fn remove_picture_type(&mut self, picture_type: PictureType) {
        let id = self.version().picture_id();
        self.frames.retain(|frame| {
            if frame.id == id {
                match &frame.fields {
                    //TODO(sp3d): rebuild this on top of fields
                    //PictureContent(ref picture) => picture,
                    _ => return false
                };

                return false/*pic.picture_type != picture_type*/
            }

            true
        });
    }

    /// Returns a vector of the user comment frames' (COMM) key/value pairs.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::{Frame, Id};
    /// use id3::id3v2::frame;
    /// use id3::Content::CommentContent;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// let mut frame = Frame::new(Id::V4(*b"COMM"));
    /// frame.fields = CommentContent(frame::Comment {
    ///     lang: "eng".to_owned(),
    ///     description: "key1".to_owned(),
    ///     text: "value1".to_owned()
    /// });
    /// tag.add_frame(frame);
    ///
    /// let mut frame = Frame::new(Id::V4(*b"COMM"));
    /// frame.fields = CommentContent(frame::Comment {
    ///     lang: "eng".to_owned(),
    ///     description: "key2".to_owned(),
    ///     text: "value2".to_owned()
    /// });
    /// tag.add_frame(frame);
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.comments().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    fn comments(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for frame in self.get_frames_by_id(self.version().comment_id()).iter() {
            match &frame.fields {
                //TODO(sp3d): rebuild this on top of fields
                /*CommentContent(ref comment) => out.push((comment.description.clone(),
                                                         comment.text.clone())),*/
                _ => { }
            }
        }

        out
    }

    /// Adds a user comment frame (COMM).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment("key1", "value1");
    /// tag.add_comment("key2", "value2");
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.comments().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    #[inline]
    fn add_comment(&mut self, description: &str, text: &str) {
        let encoding = self.version().default_encoding();
        self.add_comment_enc("eng", description, text, encoding);
    }

    /// Adds a user comment frame (COMM) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment_enc("eng", "key1", "value1", UTF16);
    /// tag.add_comment_enc("eng", "key2", "value2", UTF16);
    ///
    /// assert_eq!(tag.comments().len(), 2);
    /// assert!(tag.comments().contains(&("key1".to_owned(), "value1".to_owned())));
    /// assert!(tag.comments().contains(&("key2".to_owned(), "value2".to_owned())));
    /// ```
    fn add_comment_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding) {
        let description = description.to_owned();

        self.remove_comment(Some(&description), None);

        let mut frame = Frame::new(self.version().comment_id());

        //TODO(sp3d): rebuild this on top of fields
        /*frame.set_encoding(encoding);
        frame.fields = CommentContent(frame::Comment {
            lang: lang.to_owned(),
            description: description,
            text: text.to_owned()
        });*/

        self.frames.push(frame);
    }

    /// Removes the user comment frame (COMM) with the specified key and value.
    /// A key or value may be `None` to specify a wildcard value.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    ///
    /// tag.add_comment("key1", "value1");
    /// tag.add_comment("key2", "value2");
    /// tag.add_comment("key3", "value2");
    /// tag.add_comment("key4", "value3");
    /// tag.add_comment("key5", "value4");
    /// assert_eq!(tag.comments().len(), 5);
    ///
    /// tag.remove_comment(Some("key1"), None);
    /// assert_eq!(tag.comments().len(), 4);
    ///
    /// tag.remove_comment(None, Some("value2"));
    /// assert_eq!(tag.comments().len(), 2);
    ///
    /// tag.remove_comment(Some("key4"), Some("value3"));
    /// assert_eq!(tag.comments().len(), 1);
    ///
    /// tag.remove_comment(None, None);
    /// assert_eq!(tag.comments().len(), 0);
    /// ```
    fn remove_comment(&mut self, description: Option<&str>, text: Option<&str>) {
        let id = self.version().comment_id();
        self.frames.retain(|frame| {
            let mut description_match = false;
            let mut text_match = false;

            if frame.id == id {
                match &frame.fields {
                    //TODO(sp3d): rebuild this on top of fields
                    /*
                    CommentContent(ref comment) =>  {
                        match description {
                            Some(s) => description_match = s == &comment.description(),
                            None => description_match = true
                        }

                        match text {
                            Some(s) => text_match = s == &comment.text,
                            None => text_match = true,
                        }
                    },*/
                    _ => { // remove frames that we can't parse
                        description_match = true;
                        text_match = true;
                    }
                }
            }

            !(description_match && text_match) // true if we want to keep the item
        });
    }

    /// Sets the artist (TPE1) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_artist_enc("artist", UTF16);
    /// assert_eq!(&tag.artist().unwrap(), "artist");
    /// ```
    #[inline]
    fn set_artist_enc(&mut self, artist: &str, encoding: Encoding) {
        let id = self.version().artist_id();
        self.add_text_frame_enc(id, artist, encoding);
    }

    /// Sets the album artist (TPE2) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_album_artist_enc("album artist", UTF16);
    /// assert_eq!(&tag.album_artist().unwrap(), "album artist");
    /// ```
    #[inline]
    fn set_album_artist_enc(&mut self, album_artist: &str, encoding: Encoding) {
        self.remove_frames_by_id(Id::V3(*b"TSOP"));
        self.remove_frames_by_id(Id::V4(*b"TSOP"));
        let id = self.version().album_artist_id();
        self.add_text_frame_enc(id, album_artist, encoding);
    }

    /// Sets the album (TALB) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_album_enc("album", UTF16);
    /// assert_eq!(&tag.album().unwrap(), "album");
    /// ```
    #[inline]
    fn set_album_enc(&mut self, album: &str, encoding: Encoding) {
        let id = self.version().album_id();
        self.add_text_frame_enc(id, album, encoding);
    }

    /// Sets the song title (TIT2) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_title_enc("title", UTF16);
    /// assert_eq!(&tag.title().unwrap(), "title");
    /// ```
    #[inline]
    fn set_title_enc(&mut self, title: &str, encoding: Encoding) {
        self.remove_frames_by_id(Id::V3(*b"TSOT"));
        self.remove_frames_by_id(Id::V4(*b"TSOT"));
        let id = self.version().title_id();
        self.add_text_frame_enc(id, title, encoding);
    }

    /// Sets the genre (TCON) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_genre_enc("genre", UTF16);
    /// assert_eq!(&tag.genre().unwrap(), "genre");
    /// ```
    #[inline]
    fn set_genre_enc(&mut self, genre: &str, encoding: Encoding) {
        let id = self.version().genre_id();
        self.add_text_frame_enc(id, genre, encoding);
    }

    /// Returns the year (TYER).
    /// Returns `None` if the year frame could not be found or if it could not be parsed.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding;
    /// use id3::id3v2::frame::{Frame, Id};
    ///
    /// let id = Id::V4(*b"TYER");
    ///
    /// let mut tag = id3v2::Tag::new();
    /// assert!(tag.year().is_none());
    ///
    /// tag.add_text_frame(id, "2014", Encoding::Latin1);
    /// assert_eq!(tag.year().unwrap(), 2014);
    ///
    /// tag.remove_frames_by_id(id);
    ///
    /// tag.add_text_frame(id, "nope", Encoding::Latin1);
    /// assert!(tag.year().is_none());
    /// ```
    fn year(&self) -> Option<usize> {
        let id = self.version().year_id();
        match self.text_frame_text(id) {
            Some(ref text) => text.parse().ok(),
            _ => None,
        }
    }

    /// Sets the year (TYER).
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.set_year(2014);
    /// assert_eq!(tag.year().unwrap(), 2014);
    /// ```
    #[inline]
    fn set_year(&mut self, year: usize) {
        let id = self.version().year_id();
        self.add_text_frame_enc(id, &format!("{}", year), Encoding::Latin1);
    }

    /// Sets the year (TYER) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    ///
    /// let mut tag = id3v2::Tag::new();
    /// tag.set_year_enc(2014, UTF16);
    /// assert_eq!(tag.year().unwrap(), 2014);
    /// ```
    #[inline]
    fn set_year_enc(&mut self, year: usize, encoding: Encoding) {
        let id = self.version().year_id();
        self.add_text_frame_enc(id, &format!("{}", year), encoding);
    }

    /// Returns the (track, total_tracks) tuple.
    fn track_pair(&self) -> Option<(u32, Option<u32>)> {
        match self.text_frame_text(self.version().track_id()) {
            Some(ref text) => {
                let split: Vec<&str> = text.splitn(2, '/').collect();

                let total_tracks = if split.len() == 2 {
                    match split[1].parse() {
                        Ok(total_tracks) => Some(total_tracks),
                        _ => return None
                    }
                } else {
                    None
                };

                match split[0].parse() {
                    Ok(track) => Some((track, total_tracks)),
                    _ => None
                }
            },
            None => None
        }
    }

    /// Sets the track number (TRCK) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_track_enc(5, UTF16);
    /// assert_eq!(tag.track().unwrap(), 5);
    /// ```
    fn set_track_enc(&mut self, track: u32, encoding: Encoding) {
        let text = match self.track_pair().and_then(|(_, total_tracks)| total_tracks) {
            Some(n) => format!("{}/{}", track, n),
            None => format!("{}", track)
        };

        let id = self.version().track_id();
        self.add_text_frame_enc(id, &text, encoding);
    }


    /// Sets the total number of tracks (TRCK) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_total_tracks_enc(12, UTF16);
    /// assert_eq!(tag.total_tracks().unwrap(), 12);
    /// ```
    fn set_total_tracks_enc(&mut self, total_tracks: u32, encoding: Encoding) {
        let text = match self.track_pair() {
            Some((track, _)) => format!("{}/{}", track, total_tracks),
            None => format!("1/{}", total_tracks)
        };

        let id = self.version().track_id();
        self.add_text_frame_enc(id, &text, encoding);
    }


    /// Sets the lyrics text (USLT) using the specified text encoding.
    ///
    /// # Example
    /// ```
    /// use id3::id3v2;
    /// use id3::id3v2::frame::Encoding::UTF16;
    /// use id3::FileTags;
    ///
    /// let mut tag = FileTags::from_tags(None, Some(id3v2::Tag::new()));
    /// tag.v2.as_mut().unwrap().set_lyrics_enc("eng", "description", "lyrics", UTF16);
    /// assert_eq!(&tag.lyrics().unwrap(), "lyrics");
    /// ```
    fn set_lyrics_enc(&mut self, lang: &str, description: &str, text: &str, encoding: Encoding) {
        let id = self.version().lyrics_id();
        self.remove_frames_by_id(id);

        let mut frame = Frame::new(id);

        frame.set_encoding(encoding);
        //TODO(sp3d): rebuild this on top of fields
        /*frame.fields = LyricsContent(frame::Lyrics {
            lang: lang.to_owned(),
            description: description.to_owned(),
            text: text.to_owned()
        });*/

        self.frames.push(frame);
    }
}
