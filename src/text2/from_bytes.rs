use super::super::eol;
use super::super::eol::Split;
use super::super::eol::Splitter;
use super::chunk;
use super::Chunk;
use super::Eol;
use super::LineMeta;
use super::Text;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct FromBytes {
    splitter: Splitter,
    inner:    FromBytesInner,
}

impl FromBytes {
    pub fn feed(mut self, bytes: &[u8]) -> Result<Self, ()> {
        self.splitter.feed(bytes, |split| self.inner.split(split))?;
        Ok(self)
    }

    pub fn done(mut self) -> Result<Text, ()> {
        self.splitter.done(|split| self.inner.split(split))?;
        Ok(self.inner.done())
    }
}

#[derive(Default, Debug)]
struct FromBytesInner {
    text:   Text,
    chunk:  Chunk,
    start:  usize,
    len:    u16,
    spaces: u8,
}

impl FromBytesInner {
    fn split(&mut self, split: Split) -> Result<(), ()> {
        match split {
            Split::Bytes(bytes) => return self.bytes(bytes),
            Split::Eol(eol) => self.eol(Some(eol)),
        }

        Ok(())
    }

    fn done(mut self) -> Text {
        if self.spaces != 0 || self.len != 0 {
            self.eol(None);
        }

        self.push();
        self.text
    }

    fn eol(&mut self, eol: Option<Eol>) {
        let cursor = self.cursor();

        let ser = LineMeta::new(self.len, self.spaces, eol).serialize();
        self.chunk[self.start] = ser.0[0];
        self.chunk[self.start + 1] = ser.0[1];
        self.chunk[cursor] = ser.1;

        let eol_len = eol.map(|eol| eol.as_str().len()).unwrap_or(0);

        self.chunk.lines += 1;
        self.chunk.len += self.len + self.spaces as u16 + eol_len as u16;
        self.start = cursor + 1;
        self.len = 0;
        self.spaces = 0;
    }

    fn bytes(&mut self, mut bytes: &[u8]) -> Result<(), ()> {
        if self.len == 0 {
            bytes = self.spaces(bytes);
        }

        if self.len as usize + bytes.len() > Text::LINE_MAX_LEN {
            return Err(());
        }

        if self.cursor() + bytes.len() + 1 >= chunk::BYTES {
            self.push();
        }

        let cursor = self.cursor();
        self.chunk[cursor..][..bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len() as u16;

        Ok(())
    }

    fn push(&mut self) {
        let chunk = std::mem::take(&mut self.chunk);

        // Copy last (incomplete) line to new chunk
        if self.start + 2 <= chunk::BYTES {
            let line = &chunk[self.start + 2..][..self.len as usize];
            self.chunk[2..][..self.len as usize].copy_from_slice(line);
        } else {
            debug_assert!(self.len == 0);
        }

        self.text.push(chunk.into());
        self.start = 0;
    }

    fn cursor(&self) -> usize {
        debug_assert!(self.start <= chunk::BYTES);
        self.start + 2 + self.len as usize
    }

    fn spaces<'a>(&mut self, bytes: &'a [u8]) -> &'a [u8] {
        debug_assert!(self.spaces <= LineMeta::MAX_SPACES);

        let mut count = 0;
        let max = ((LineMeta::MAX_SPACES - self.spaces) as usize).min(bytes.len());

        for &b in &bytes[..max] {
            if b == b' ' {
                count += 1;
            } else {
                break;
            }
        }

        self.spaces += count;

        if let Some(byte) = bytes.get(count as usize) {
            if !byte.is_ascii() {
                self.spaces = self.spaces.saturating_sub(1);
                count = count.saturating_sub(1);
            }
        }

        return &bytes[count as usize..];
    }
}

pub fn leading<'a>(f: fn(&u8) -> bool, bytes: &'a [u8], initial: u8) -> u8 {
    let mut count = 0;
    let max = ((u8::MAX - initial) as usize).min(bytes.len());

    for b in &bytes[..max] {
        if f(b) {
            count += 1;
        } else {
            break;
        }
    }

    return count;
}
