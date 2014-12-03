#rust-id3

rust-id3 is a low-level library for reading and writing ID3v1 and ID3v2 tags.

##Usage

```rust
use id3::FileTags;
use id3::id3v2::frame::Id;

let mut tag = Filetags::read_from_path(&Path::new("music.mp3")).unwrap();

// print the artist
println!("{}", tag.text_frame_text(Id::V4(*b"TALB")).unwrap());
```

##Supported ID3 Versions

  * ID3v1, ID3v1.1 (track number support), "enhanced" (90-byte) ID3v1
  * ID3v2.2, 2.3, 2.4

See COMPLIANCE.txt for a discussion of specific ID3 features which are unsupported.

##Unsupported Features

  * Unsynchronization
  * Grouping identity
  * Encryption

##Contributors

  * [James Hurst](https://github.com/jameshurst) 
    * Substantial work on ID3v2
  * [Olivier Renaud](https://bitbucket.org/olivren) 
    * Initial ID3v1 reading code 

