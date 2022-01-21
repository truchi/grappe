use super::*;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::Deref;
use unicode_segmentation::GraphemeCursor;

#[cfg(test)]
mod tests;

/// A unicode grapheme cluster with display width.
#[derive(Copy, Clone, Eq)]
pub struct Cluster<'a> {
    /// `str`.
    str:   &'a str,
    /// Display width.
    width: u8,
}

impl Cluster<'static> {
    /// A space.
    pub const SPACE: Self = Self {
        str:   " ",
        width: 1,
    };
}

impl<'a> Cluster<'a> {
    /// Creates a new `Cluster` from raw parts `str` and `width`.
    pub(super) fn from_raw(str: &'a str, width: u8) -> Self {
        debug_assert!(utils::clusters(str).count() == 1);
        debug_assert!(utils::width(str) == width as usize);

        Self { str, width }
    }

    /// Returns an iterator of `Cluster`s from an `str`.
    pub fn clusters(str: &'a str) -> impl DoubleEndedIterator<Item = Cluster> {
        utils::clusters(str).map(|cluster| Cluster {
            str:   cluster,
            width: utils::width(cluster) as u8,
        })
    }

    /// Returns the underlying `str`.
    pub fn as_str(&self) -> &str {
        self.str
    }

    /// Returns the display width.
    pub fn width(&self) -> u8 {
        self.width
    }
}

impl<'a> Cluster<'a> {
    /// Returns the [`Cluster`] at byte `index` in `str`,
    /// along with its actual byte index.
    pub fn at_index(str: &'a str, mut index: usize) -> (usize, Option<Cluster<'a>>) {
        while !str.is_char_boundary(index) {
            index -= 1;
        }

        let mut start = GraphemeCursor::new(index, str.len(), true);
        let mut end = GraphemeCursor::new(index, str.len(), true);

        let end = end.next_boundary(str, 0).unwrap().unwrap_or(str.len());
        let start = if start.is_boundary(str, 0).unwrap() {
            index
        } else {
            start.prev_boundary(str, 0).unwrap().unwrap_or(0)
        };

        if start == str.len() {
            (start, None)
        } else {
            let cluster = unsafe { get!(str, start..end) };
            let cluster = Cluster::from_raw(cluster, utils::width(cluster) as u8);

            (start, Some(cluster))
        }
    }
}

impl<'a> PartialEq for Cluster<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl<'a> Deref for Cluster<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.str
    }
}

impl<'a> ToString for Cluster<'a> {
    fn to_string(&self) -> String {
        self.str.to_owned()
    }
}

impl<'a> Debug for Cluster<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}) {:?}", self.width, self.str)
    }
}
