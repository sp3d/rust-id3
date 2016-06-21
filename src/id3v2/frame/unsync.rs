pub struct DeUnsyncIter<I> {
    underlying: I,
    last_was_ff: bool,
}

pub trait DeUnsyncIterator<IterItem>: Iterator<Item=IterItem>+Sized
where IterItem: Into<u8> {
    fn deunsynchronized(self) -> DeUnsyncIter<Self> {
        DeUnsyncIter {
            underlying: self,
            last_was_ff: false,
        }
    }
}

impl<I: Iterator> DeUnsyncIterator<<I as Iterator>::Item> for I
where <I as Iterator>::Item: Into<u8> {}

impl <I: Iterator> Iterator for DeUnsyncIter<I>
where I::Item: Into<u8>
{
    type Item=u8;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.underlying.next() {
            Some(x) => {
                let x = x.into();
                if self.last_was_ff && x == 0 {
                    /* discard unsync zero and pass the next byte through verbatim */
                    self.underlying.next().map(Into::into)
                } else {
                    self.last_was_ff = x==0xff;
                    Some(x)
                }
            },
            None => None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, maybe_upper) = self.underlying.size_hint();
        (lower/2, maybe_upper)
    }
}

pub struct UnsyncIter<I> {
    underlying: I,
    next_byte: Option<u8>,
    last_was_ff: bool,
}

pub trait UnsyncIterator<IterItem>: Iterator<Item=IterItem>+Sized
where IterItem: Into<u8> {
    fn unsynchronized(self) -> UnsyncIter<Self> {
        UnsyncIter {
            underlying: self,
            next_byte: None,
            last_was_ff: false,
        }
    }
}

impl<I: Iterator> UnsyncIterator<<I as Iterator>::Item> for I
where <I as Iterator>::Item: Into<u8> {}

impl <I: Iterator> Iterator for UnsyncIter<I>
where I::Item: Into<u8>
{
    type Item=u8;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        /* if we had a queued byte, dispense it */
        if self.next_byte.is_some() {
            let b2 = self.next_byte.unwrap().into();
            self.next_byte = None;
            return Some(b2)
        }
        match self.underlying.next() {
            Some(x) => {
                let x = x.into();
                if self.last_was_ff && (x==0 || x & 0b11100000 == 0b11100000) {
                    self.last_was_ff = x==0xff;
                    self.next_byte = Some(x);
                    Some(0)
                } else {
                    self.last_was_ff = x==0xff;
                    Some(x)
                }
            },
            /* a trailing 0xff must assume that the next byte could be 0 or 0b111xxxxx */
            None => if self.last_was_ff {
                self.last_was_ff = false;
                Some(0)
            } else {
                None
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, maybe_upper) = self.underlying.size_hint();
        (lower, maybe_upper.and_then(|x| x.checked_mul(2)))
    }
}

static TEST_PAIRS: &'static [(&'static [u8], &'static [u8])] = &[
    (b"\xff\xff\xe0ok", b"\xff\x00\xff\x00\xe0ok"),
    (b"dfdata\xff", b"dfdata\xff\x00"),
    (b"never", b"never"),
];

#[test]
fn test_unsync() {
    let unsync = |x: &[u8]| x.into_iter().map(|x|*x).unsynchronized().collect::<Vec<u8>>();
    for &(raw, unsynced) in TEST_PAIRS
    {
        assert_eq!(&*unsync(raw), unsynced);
    }
}

#[test]
fn test_deunsync() {
    let deunsync = |x: &[u8]| x.into_iter().map(|x|*x).deunsynchronized().collect::<Vec<u8>>();
    for &(raw, unsynced) in TEST_PAIRS
    {
        assert_eq!(raw, &*deunsync(unsynced));
    }
}

#[test]
fn test_inverse() {
    let unsync = |x: &[u8]| x.into_iter().map(|x|*x).unsynchronized().collect::<Vec<u8>>();
    let deunsync = |x: &[u8]| x.into_iter().map(|x|*x).deunsynchronized().collect::<Vec<u8>>();
    for &(raw, unsynced) in TEST_PAIRS
    {
        assert_eq!(&*deunsync(&*unsync(raw)), raw);
        assert_eq!(&*unsync(&*deunsync(unsynced)), unsynced);
    }
}
