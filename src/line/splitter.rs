#[derive(Copy, Clone, Default, Debug)]
struct State {
    len:   usize,
    chars: usize,
}

/// Splits an `&str` stream in chunks of `MAX` bytes.
///
/// `MAX` must be greater or equal to `Self::MIN` (`4`).
#[derive(Copy, Clone, Debug)]
pub struct Splitter<const MAX: usize> {
    state: State,
}

impl<const MAX: usize> Splitter<MAX> {
    /// The minimun size of chunks, in bytes.
    ///
    /// Equals the size of a `char` (`4`).
    pub const MIN: usize = std::mem::size_of::<char>();

    /// Returns a new `Splitter`.
    ///
    /// # Panics
    ///
    /// Panics if `MAX < Self::MIN`.
    pub fn new() -> Self {
        Self::assert();

        Self {
            state: Default::default(),
        }
    }

    /// Returns a new `Splitter`, with state initialized to `len` and `chars`.
    ///
    /// # Panics
    ///
    /// Panics if `MAX < Self::MIN`.
    pub fn with(len: usize, chars: usize) -> Self {
        Self::assert();

        Self {
            state: State {
                len:   len.min(MAX),
                chars: chars.min(MAX),
            },
        }
    }

    /// Feeds an `&str`, returning an iterator over [`Split<&str>`]s.
    pub fn feed_str<'a, 'b>(&'a mut self, str: &'b str) -> Splits<'a, 'b, MAX> {
        Splits {
            state: &mut self.state,
            str,
        }
    }

    /// Feeds a `char`, returning a [`Split<char>`].
    pub fn feed_char(&mut self, char: char) -> Split<char> {
        let len = char.len_utf8();
        let chars = 1;

        debug_assert!(self.state.len <= MAX);
        debug_assert!(self.state.chars <= MAX);

        if self.state.len + len > MAX {
            self.state.len = 0;
            self.state.chars = 0;
        }

        let split = Split {
            len,
            chars,
            offset_len: self.state.len,
            offset_chars: self.state.chars,
            acc_len: self.state.len + len,
            acc_chars: self.state.chars + chars,
            split: char,
        };

        debug_assert!(split.acc_len <= MAX);
        debug_assert!(split.acc_chars <= MAX);

        if split.acc_len == MAX {
            self.state.len = 0;
            self.state.chars = 0;
        } else {
            self.state.len = split.acc_len;
            self.state.chars = split.acc_chars;
        }

        split
    }

    /// Resets the state.
    pub fn done(&mut self) {
        self.state.len = 0;
        self.state.chars = 0;
    }

    /// Resets the state with `len` and `chars`.
    pub fn reset(&mut self, len: usize, chars: usize) {
        self.state.len = len.min(MAX);
        self.state.chars = chars.min(MAX);
    }

    fn assert() {
        assert!(
            MAX >= Self::MIN,
            "Splitting `&str` in chunks of {} byte{} is not allowed (minimun is {})",
            MAX,
            if MAX > 1 { "s" } else { "" },
            Self::MIN
        );
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Split<T> {
    pub len:          usize,
    pub chars:        usize,
    pub offset_len:   usize,
    pub offset_chars: usize,
    pub acc_len:      usize,
    pub acc_chars:    usize,
    pub split:        T,
}

impl<T> Split<T> {
    pub fn is_start(&self) -> bool {
        self.offset_len == 0
    }
}

#[derive(Debug)]
pub struct Splits<'a, 'b, const MAX: usize> {
    state: &'a mut State,
    str:   &'b str,
}

impl<'a, 'b, const MAX: usize> Splits<'a, 'b, MAX> {}

impl<'a, 'b, const MAX: usize> Iterator for Splits<'a, 'b, MAX> {
    type Item = Split<&'b str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.str.is_empty() {
            return None;
        }

        debug_assert!(self.state.len <= MAX);
        debug_assert!(self.state.chars <= MAX);

        let mut len = self.str.len().min(MAX - self.state.len);
        while !self.str.is_char_boundary(len) {
            len -= 1;
        }

        if len == 0 {
            self.state.len = 0;
            self.state.chars = 0;

            return self.next();
        }

        let str = &self.str[..len];
        let chars = str.chars().count();

        let split = Split {
            len,
            chars,
            offset_len: self.state.len,
            offset_chars: self.state.chars,
            acc_len: self.state.len + len,
            acc_chars: self.state.chars + chars,
            split: str,
        };

        debug_assert!(split.acc_len <= MAX);
        debug_assert!(split.acc_chars <= MAX);

        if split.acc_len == MAX {
            self.state.len = 0;
            self.state.chars = 0;
        } else {
            self.state.len = split.acc_len;
            self.state.chars = split.acc_chars;
        }

        self.str = &self.str[len..];

        Some(split)
    }
}
