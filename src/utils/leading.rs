/// Counts the leading `BYTE` in a stream of byte chunk.
///
/// Counts stream-wise, not chunk-wise.
/// The count is decremented if the following byte is not ASCII.
///
/// # Usage
///
/// A simple example:
///
/// ```
/// # use grappe::utils::Leading;
/// let mut leading = Leading::<b' ', 5>::default();
///
/// // The file we are reading, in chunks
/// let file = &["  ", "  hello", " 🦀!"];
///
/// // Feeding gives another chunk stream, without leading bytes
/// assert!(leading.feed(file[0].as_bytes()) == (b"", b""));
/// assert!(leading.feed(file[1].as_bytes()) == (b"", b"hello"));
/// assert!(leading.feed(file[2].as_bytes()) == (b"", " 🦀!".as_bytes()));
///
/// // Get the count after feeding all chunks
/// assert!(leading.done() == 4);
/// ```
///
/// An example of the tricky case:
///
/// ```
/// # use grappe::utils::Leading;
/// let mut leading = Leading::<b' ', 5>::default();
///
/// // The file we are reading, in chunks
/// let file = &["  ", "  ", "🦀"]; // Crab emoji does not start with ASCII
///
/// // Feeding gives another chunk stream, without leading bytes
/// assert!(leading.feed(file[0].as_bytes()) == (b"", b""));
/// assert!(leading.feed(file[1].as_bytes()) == (b"", b""));
/// assert!(leading.feed(file[2].as_bytes()) == (b" ", "🦀".as_bytes()));
///
/// // Get the count after feeding all chunks
/// assert!(leading.done() == 3); // The last space is followed by non-ASCII
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct Leading<const BYTE: u8, const MAX: usize> {
    state: State,
}

#[derive(Copy, Clone, Debug)]
enum State {
    Count(usize),
    Pass(usize),
}

impl Default for State {
    fn default() -> Self {
        Self::Count(0)
    }
}

impl<const BYTE: u8, const MAX: usize> Leading<BYTE, MAX> {
    /// Feeds a chunk.
    ///
    /// Returns:
    /// - a potential extra `BYTE` (if previous chunk is full of `BYTE`
    /// and the current chunk starts with non-ASCII), and
    /// - the chunk stripped of (stream-wise) leading `BYTE`s.
    pub fn feed<'a>(&mut self, bytes: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        match self.state {
            State::Count(count) => {
                debug_assert!(count <= MAX);
                let more = bytes[..(MAX - count).min(bytes.len())]
                    .iter()
                    .take_while(|&&b| b == BYTE)
                    .count();

                debug_assert!(count + more <= MAX);

                if let Some(&byte) = bytes.get(more) {
                    debug_assert!(byte != BYTE || count + more == MAX);

                    let (count, carry, bytes): (_, &[_], _) =
                        // Everything is fine if followed by ASCII
                        if byte.is_ascii() {
                            (count + more, &[], &bytes[more..])
                        }
                        // Not followed by ASCII, return from previous byte
                        else if let Some(more) = more.checked_sub(1) {
                            (count + more, &[], &bytes[more..])
                        }
                        // Non ASCII at start of chunk
                        // Emit virtual previous byte
                        else if let Some(count) = count.checked_sub(1) {
                            (count, &[BYTE], bytes)
                        }
                        // No BYTEs nor ASCII
                        // First byte of first chunk is not ASCII
                        else {
                            (0, &[], bytes)
                        };

                    self.state = State::Pass(count);
                    (carry, bytes)
                } else {
                    self.state = State::Count(count + more);
                    (&[], &[])
                }
            }
            State::Pass(_) => (&[], bytes),
        }
    }

    /// Returns the count of stream-wise leading `BYTE`
    /// which are not followed by non-ASCII.
    ///
    /// Call after feeding all chunks.
    /// State is reset to allow reuse for another chunk stream.
    pub fn done(&mut self) -> usize {
        let state = self.state;
        self.state = State::default();

        match state {
            State::Count(count) => count,
            State::Pass(count) => count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTE: u8 = b' ';
    const DATA: &[(&str, usize)] = &[
        ("", 0),
        ("hello 🦀", 0),
        (" hello 🦀", 1),
        ("  hello 🦀", 2),
        ("   hello 🦀", 3),
        ("    hello 🦀", 4),
        ("🦀", 0),
        (" 🦀", 0),
        ("  🦀", 1),
        ("   🦀", 2),
        ("    🦀", 3),
    ];

    #[test]
    fn test() {
        test_at_max::<0>();
        test_at_max::<1>();
        test_at_max::<2>();
        test_at_max::<3>();
        test_at_max::<4>();
        test_at_max::<5>();
    }

    fn test_at_max<const MAX: usize>() {
        for (str, expected) in DATA {
            for size in 1..=str.len() {
                let mut chunks = str.as_bytes().chunks(size);
                let mut leading = Leading::<BYTE, MAX>::default();
                let mut vec = Vec::new();

                for chunk in chunks {
                    let (carry, bytes) = leading.feed(chunk);
                    vec.extend(carry);
                    vec.extend(bytes);
                }

                let actual = leading.done();
                (0..actual).for_each(|_| vec.insert(0, BYTE));

                dbg!(((str, size, MAX), (actual, expected)));
                assert!(expected.min(&MAX) == &actual);
                assert!(vec == str.as_bytes());
            }
        }
    }
}
