use super::*;

/// Either `&[u8]` or [`Eol`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Split<'a> {
    Bytes(&'a [u8]),
    Eol(Eol),
}

impl<'a> Split<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        match self {
            Self::Bytes(bytes) => bytes,
            Self::Eol(eol) => eol.as_bytes(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum State {
    LF,
    VT,
    FF,
    CR,
    NEL,
    LS,
    PS,
    CRLF,
    NEL0,
    S0,
    S1,
}

/// Splits byte chunks at end-of-line sequences.
///
/// Care is taken to not split further than EOLs or chunk end,
/// to maintain eventual UTF-8 validity.
///
/// # Usage
///
/// ```
/// # use grappe::eol::Eol;
/// # use grappe::eol::Split;
/// # use grappe::eol::Splitter;
/// let mut splitter = Splitter::default();
///
/// // The file we are reading
/// let file = b"Hello\r\nrust\r";
///
/// // Let's pretend we are using a BufReader...
/// let mut chunks = file.chunks(6);
///
/// // Feed first byte chunk (`b"Hello\r"`)
/// let mut splits = splitter.split(chunks.next().unwrap());
/// assert!(splits.next() == Some(Split::Bytes(b"Hello")));
/// assert!(splits.next() == None);
///
/// // Feed second byte chunk (`b"\nrust\r"`)
/// let mut splits = splitter.split(chunks.next().unwrap());
/// assert!(splits.next() == Some(Split::Eol(Eol::CRLF)));
/// assert!(splits.next() == Some(Split::Bytes(b"rust")));
/// assert!(splits.next() == None);
///
/// // After feeding all byte chunks
/// assert!(splitter.done() == Some(Split::Eol(Eol::CR)));
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct Splitter {
    state: Option<State>,
}

impl Splitter {
    /// Feeds chunk to split, returning the iterator.
    ///
    /// The returned iterator must be exhausted
    /// before calling this function again.
    pub fn split<'a>(&'a mut self, bytes: &'a [u8]) -> Splits<'a> {
        Splits {
            state: &mut self.state,
            bytes,
        }
    }

    /// Retrieves the very last split of the byte stream.
    ///
    /// Call after feeding all chunks and exhausting those iterators.
    /// State is reset to allow reuse for another byte chunk
    /// stream.
    pub fn done(&mut self) -> Option<Split<'static>> {
        let state = self.state?;
        self.state = None;

        Some(match state {
            State::CR => Split::Eol(Eol::CR),
            State::NEL0 => Split::Bytes(&[NEL0]),
            State::S0 => Split::Bytes(&[S0]),
            State::S1 => Split::Bytes(&[S0, S1]),
            _ => unreachable!(),
        })
    }
}

/// Iterator of [`Split`]s, returned from [`Splitter::split`].
#[derive(Debug)]
pub struct Splits<'a> {
    state: &'a mut Option<State>,
    bytes: &'a [u8],
}

impl<'a> Splits<'a> {
    fn take(&mut self, take: usize, skip: usize) -> &'a [u8] {
        let taken = &self.bytes[..take];
        self.skip(take + skip);
        taken
    }

    fn skip(&mut self, skip: usize) -> &mut Self {
        self.bytes = &self.bytes[skip..];
        self
    }

    fn set<T: Into<Option<State>>>(&mut self, state: T) -> &mut Self {
        *self.state = state.into();
        self
    }

    fn bytes<'b>(&mut self, bytes: &'b [u8]) -> Option<Split<'b>> {
        self.set(None);
        Some(Split::Bytes(bytes))
    }

    fn eol(&mut self, eol: Eol) -> Option<Split<'a>> {
        self.set(None);
        Some(Split::Eol(eol))
    }

    fn none(&mut self) -> Option<Split<'a>> {
        macro_rules! ret {
            ($self:ident, $state:ident, $take:ident, $skip:literal) => {{
                let bytes = $self.set(State::$state).take($take, $skip);

                // Those partial eol states must happen only
                // at the end of the input bytes,
                // to avoid unnecessary splits
                debug_assert!(match self.state.unwrap() {
                    State::NEL0 | State::S0 | State::S1 => self.bytes.is_empty(),
                    _ => true,
                });

                // TODO we can avoid this check by testing first byte separately,
                // because here we actually check if $take == 0 (~ i == 0)
                return if bytes.is_empty() {
                    self.next()
                } else {
                    Some(Split::Bytes(bytes))
                };
            }};
        }

        let bytes = self.bytes;

        if bytes.is_empty() {
            return None;
        }

        for (i, &byte) in bytes.iter().enumerate() {
            match byte {
                LF => ret!(self, LF, i, 1),
                CR => ret!(self, CR, i, 1),
                VT => ret!(self, VT, i, 1),
                FF => ret!(self, FF, i, 1),
                NEL0 => match bytes.get(i + 1) {
                    None => ret!(self, NEL0, i, 1),
                    Some(&NEL1) => ret!(self, NEL, i, 2),
                    _ => {}
                },
                S0 => match bytes.get(i + 1) {
                    None => ret!(self, S0, i, 1),
                    Some(&S1) => match bytes.get(i + 2) {
                        None => ret!(self, S1, i, 2),
                        Some(&LS2) => ret!(self, LS, i, 3),
                        Some(&PS2) => ret!(self, PS, i, 3),
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        self.bytes = &[];
        Some(Split::Bytes(bytes))
    }

    fn cr(&mut self) -> Option<Split<'a>> {
        match self.bytes.get(0) {
            Some(&LF) => self.skip(1).eol(Eol::CRLF),
            Some(_) => self.eol(Eol::CR),
            _ => None,
        }
    }

    fn nel0(&mut self) -> Option<Split<'a>> {
        match self.bytes.get(0) {
            Some(&NEL1) => self.skip(1).eol(Eol::NEL),
            Some(_) => self.bytes(&[NEL0]),
            _ => None,
        }
    }

    fn s0(&mut self) -> Option<Split<'a>> {
        match self.bytes.get(0) {
            Some(&S1) => self.skip(1).set(State::S1).s1(),
            Some(_) => self.bytes(&[S0]),
            _ => None,
        }
    }

    fn s1(&mut self) -> Option<Split<'a>> {
        match self.bytes.get(0) {
            Some(&LS2) => self.skip(1).eol(Eol::LS),
            Some(&PS2) => self.skip(1).eol(Eol::PS),
            Some(_) => self.bytes(&[S0, S1]),
            _ => None,
        }
    }
}

impl<'a> Iterator for Splits<'a> {
    type Item = Split<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            None => self.none(),
            Some(State::LF) => self.eol(Eol::LF),
            Some(State::CRLF) => self.eol(Eol::CRLF),
            Some(State::VT) => self.eol(Eol::VT),
            Some(State::FF) => self.eol(Eol::FF),
            Some(State::NEL) => self.eol(Eol::NEL),
            Some(State::LS) => self.eol(Eol::LS),
            Some(State::PS) => self.eol(Eol::PS),
            Some(State::CR) => self.cr(),
            Some(State::NEL0) => self.nel0(),
            Some(State::S0) => self.s0(),
            Some(State::S1) => self.s1(),
        }
    }
}
