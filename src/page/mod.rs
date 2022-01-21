// mod builder;

// pub use builder::*;

use super::LineMeta;
use crate::Offset;
use crate::SPACES;
use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;
use std::slice::SliceIndex;

// pub(super) const BYTES: usize = 1024; // TODO - 2*size<usize>
pub(super) const BYTES: usize = 10;

pub type PageRef<'a> = Page<&'a [u8]>;
pub type PageMut<'a> = Page<&'a mut [u8]>;
pub type RcPage = Page<Rc<[u8; BYTES]>>;

#[derive(Copy, Clone, Debug)]
pub struct Page<T = [u8; BYTES]> {
    pub(super) offset: Offset,
    pub(super) first:  u16,
    pub(super) end:    u16,
    pub(super) len:    u16,
    pub(super) chars:  u16,
    pub(super) lines:  u16,
    pub(super) bytes:  T, // [u8; 1008]
}

impl Default for Page {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            first:  0,
            end:    0,
            len:    0,
            chars:  0,
            lines:  0,
            bytes:  [0; BYTES],
        }
    }
}

impl Page {
    pub fn chunks(&self) -> impl Iterator<Item = &str> {
        let mut bytes = &self[..];

        std::iter::from_fn(move || {
            if bytes.len() > BYTES - self.end as usize {
                let (meta, after) = LineMeta::deserialize(bytes);
                let (line, after) = after.split_at(meta.len as usize);
                bytes = after;

                let spaces = (meta.spaces != 0).then(|| &SPACES[0..meta.spaces as usize]);
                let line = (line.len() > 0).then(|| unsafe { utf8!(line) });
                let eol = meta.eol.map(|eol| eol.as_str());

                Some(spaces.into_iter().chain(line).chain(eol))
            } else {
                None
            }
        })
        .flatten()
    }
}

macro_rules! index {
    (mut $Type:ident$(<$lifetime:lifetime>)?) => {
        impl<$($lifetime,)? I> IndexMut<I> for $Type$(<$lifetime>)?
        where
            I: SliceIndex<[u8]>,
        {
            fn index_mut(&mut self, index: I) -> &mut I::Output {
                &mut self.bytes[index]
            }
        }
    };
    ($Type:ident$(<$lifetime:lifetime>)?) => {
        impl<$($lifetime,)? I> Index<I> for $Type$(<$lifetime>)?
        where
            I: SliceIndex<[u8]>,
        {
            type Output = I::Output;

            fn index(&self, index: I) -> &I::Output {
                &self.bytes[index]
            }
        }
    };
}

index!(Page);
index!(PageRef<'a>);
index!(PageMut<'a>);
index!(RcPage);
index!(mut Page);
index!(mut PageMut<'a>);

impl From<Page> for RcPage {
    fn from(page: Page) -> Self {
        Self {
            offset: page.offset,
            first:  page.first,
            end:    page.end,
            len:    page.len,
            chars:  page.chars,
            lines:  page.lines,
            bytes:  Rc::new(page.bytes),
        }
    }
}

impl ToString for Page {
    fn to_string(&self) -> String {
        use super::LineMeta;

        let mut string = String::with_capacity(self.len as usize);
        let mut bytes = &self.bytes[..];

        loop {
            if bytes.len() <= BYTES - self.end as usize {
                break;
            }

            let (meta, after) = LineMeta::deserialize(bytes);
            let (line, after) = after.split_at(meta.len as usize);
            bytes = after;

            (0..meta.spaces).for_each(|_| string.push(' '));
            string.push_str(unsafe { utf8!(line) });
            meta.eol.map(|eol| string.push_str(eol.as_str()));
        }

        string
    }
}
