mod builder;

pub use builder::*;

use super::Offset;
use std::ops::Index;
use std::ops::IndexMut;
use std::rc::Rc;
use std::slice::SliceIndex;

pub(super) const BYTES: usize = 1024;

pub type ChunkRef<'a> = Chunk<&'a [u8]>;
pub type ChunkMut<'a> = Chunk<&'a mut [u8]>;
pub type RcChunk = Chunk<Rc<[u8; BYTES]>>;

#[derive(Copy, Clone, Debug)]
pub struct Chunk<T = [u8; BYTES]> {
    pub(super) len:    u16,
    pub(super) lines:  u16,
    pub(super) offset: Offset,
    pub(super) chunk:  T,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            len:    0,
            lines:  0,
            offset: Offset::default(),
            chunk:  [0; BYTES],
        }
    }
}

macro_rules! index {
    (mut $Type:ident$(<$lifetime:lifetime>)?) => {
        impl<$($lifetime,)? I> IndexMut<I> for $Type$(<$lifetime>)?
        where
            I: SliceIndex<[u8]>,
        {
            fn index_mut(&mut self, index: I) -> &mut I::Output {
                &mut self.chunk[index]
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
                &self.chunk[index]
            }
        }
    };
}

index!(Chunk);
index!(ChunkRef<'a>);
index!(ChunkMut<'a>);
index!(RcChunk);
index!(mut Chunk);
index!(mut ChunkMut<'a>);

impl From<Chunk> for RcChunk {
    fn from(chunk: Chunk) -> Self {
        Self {
            len:    chunk.len,
            lines:  chunk.lines,
            offset: chunk.offset,
            chunk:  Rc::new(chunk.chunk),
        }
    }
}
