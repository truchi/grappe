#[macro_use]
mod macros;

mod leading;
mod stack_vec;
mod validator;

pub use leading::*;
pub use stack_vec::*;
pub use validator::*;

use std::cmp::Ordering;
use std::ops::Bound;
use std::ops::Range;
use std::ops::RangeBounds;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Either<L, R> {
    /// A value of type `L`.
    Left(L),
    /// A value of type `R`.
    Right(R),
}

/// Returns the display width of a string slice.
pub fn width(str: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(str)
}

/// Returns the display width of a char.
pub fn cwidth(char: char) -> u8 {
    unicode_width::UnicodeWidthChar::width(char).unwrap_or(0) as u8
}

/// Returns an iterator over the extended grapheme clusters of a string
/// slice.
pub fn clusters(str: &str) -> impl DoubleEndedIterator<Item = &str> {
    unicode_segmentation::UnicodeSegmentation::graphemes(str, true)
}

/// Returns an iterator over the lines of a string slice.
pub fn lines(str: &str) -> impl DoubleEndedIterator<Item = &str> {
    str.split('\n').map(|line| {
        let l = line.len();
        if l > 0 && line.as_bytes()[l - 1] == b'\r' {
            &line[0..l - 1]
        } else {
            line
        }
    })
}

/// Returns an ASCII byte if `str` is an ASCII byte,
/// `None` otherwise.
pub fn as_ascii(str: &str) -> Option<u8> {
    if str.len() == 1 {
        // SAFETY: 0 < 1
        let byte = *unsafe { get!(str.as_bytes(), 0) };

        if byte.is_ascii() {
            return Some(byte);
        }
    }

    None
}

/// Returns `true` if `str` is an ASCII byte,
/// `false` otherwise.
pub fn is_ascii(str: &str) -> bool {
    as_ascii(str).is_some()
}

/// Returns a byte if `str` is a single byte,
/// `None` otherwise.
pub fn as_byte(str: &str) -> Option<u8> {
    if str.len() == 1 {
        // SAFETY: 0 < 1
        Some(*unsafe { get!(str.as_bytes(), 0) })
    } else {
        None
    }
}

/// Returns `true` if `str` is an ASCII byte,
/// `false` otherwise.
pub fn is_byte(str: &str) -> bool {
    as_byte(str).is_some()
}

/// Bounds `range` to `0..len`,
/// or returns `None` if `start > end`.
pub fn to_range<T: RangeBounds<usize>>(range: T, len: usize) -> Option<Range<usize>> {
    let end = to_end(range.end_bound(), len);
    if end > len {
        return None;
    }

    let start = to_start(range.end_bound());
    if end > start {
        return None;
    }

    Some(start..end)
}

/// Bounds `range` to `0..len`,
/// without checking if `start <= end`.
pub fn to_range_unchecked<T: RangeBounds<usize>>(range: T, len: usize) -> Range<usize> {
    let end = to_end(range.end_bound(), len);
    let start = to_start(range.end_bound());

    debug_assert!(start <= end);
    debug_assert!(end <= len);
    start..end
}

fn to_start(start: Bound<&usize>) -> usize {
    match start {
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => i + 1,
        Bound::Unbounded => 0,
    }
}

fn to_end(end: Bound<&usize>, len: usize) -> usize {
    match end {
        Bound::Included(&i) => i + 1,
        Bound::Excluded(&i) => i,
        Bound::Unbounded => len,
    }
}

pub fn range_partial_cmp<T: Ord>(start1: T, end1: T, start2: T, end2: T) -> Option<Ordering> {
    use Ordering::*;

    match start1.cmp(&start2) {
        // v         1
        //    v----v 2
        Less => match end1.cmp(&start2) {
            // v-v       1
            //    v----v 2
            Less => Some(Less),
            // v--v      1
            //    v----v 2
            Equal => Some(Less),
            // v----v    1
            //    v----v 2
            Greater => None,
        },
        // v           1
        // v----v      2
        Equal => match end1.cmp(&start2) {
            // end1 < start1: not allowed
            Less => unreachable!(),
            // v----v      1
            // v----v      2
            Equal => Some(Equal),
            // v-------v   1
            // v----v      2
            Greater => None,
        },
        //    v---v    1
        // v           2
        Greater => match end2.cmp(&start1) {
            //    v---v    1
            // v-v         2
            Less => Some(Greater),
            //    v---v    1
            // v--v        2
            Equal => Some(Greater),
            //    v---v    1
            // v----v      2
            Greater => None,
        },
    }
}

/// Adds (saturating) to `count` the number of leading bytes in `bytes`
/// satisfying `f`, then substracts (saturating) 1 if the next byte
/// is not ASCII.
///
/// This enables a simple and fast way to find ASCII clusters (while potentially
/// giving false negatives): when two ASCII bytes follows, the first one is
/// ASCII. Unless you ask for `'\r'`,`"\r\n"` being two ASCII bytes but one
/// cluster.
pub fn leading<T: AsRef<[u8]>>(f: fn(&u8) -> bool, bytes: T, count: &mut u8) {
    let bytes = bytes.as_ref();
    let initial = *count;
    let max = ((u8::MAX - initial) as usize).min(bytes.len());

    fn sub(b: &u8, count: &mut u8) {
        if !b.is_ascii() && *count != 0 {
            *count -= 1;
        }
    }

    for b in &bytes[..max] {
        if f(b) {
            debug_assert!(*b != b'\r', "Don't do that!");
            *count += 1;
        } else {
            return sub(b, count);
        }
    }

    bytes
        .get((*count - initial) as usize)
        .map(|b| sub(b, count));
}

/// Counts the number of ASCII space clusters at the beginning of `bytes`.
///
/// May give a false negative.
pub fn leading_spaces<T: AsRef<[u8]>>(bytes: T, count: &mut u8) {
    leading(is_space, bytes, count);
}

/// Counts the number of ASCII clusters of width 1 at the beginning of `bytes`.
///
/// May give a false negative.
pub fn leading_ascii1s<T: AsRef<[u8]>>(bytes: T, count: &mut u8) {
    leading(is_ascii1, bytes, count);
}

/// Counts the number of ASCII clusters of width 1 from the end of `bytes`.
pub fn trailing_ascii1s<T: AsRef<[u8]>>(bytes: T, right: &mut u8) {
    let bytes = bytes.as_ref();
    let start = bytes.len() - ((u8::MAX - *right) as usize).min(bytes.len());

    for b in bytes[start..].iter().rev() {
        if is_ascii1(b) {
            *right += 1;
        } else {
            return;
        }
    }
}

/// Returns `true` if `byte` is ASCII of width `1` (`b' '..=b'~'`).
pub fn is_ascii1(byte: &u8) -> bool {
    (b' '..=b'~').contains(byte)
}

/// Returns `true` if `byte` is an ASCII space (`' '`).
pub fn is_space(byte: &u8) -> bool {
    byte == &b' '
}
