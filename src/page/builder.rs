use super::Chunk;
use super::BYTES;
use crate::text::LineError;
use crate::text::LineMeta;
use crate::text::Split;
use crate::Eol;
use crate::Text;

#[derive(Default, Debug)]
pub struct ChunkBuilder {
    chunk:  Chunk,
    index:  usize,
    len:    u16,
    spaces: u8,
}

impl ChunkBuilder {
    pub fn push(&mut self, split: Split) -> Result<Option<Chunk>, LineError> {
        match split {
            Split::Spaces(spaces) => return Ok(self.push_spaces(spaces)),
            Split::Bytes(bytes) => return self.push_bytes(bytes),
            Split::Eol(eol) => self.push_eol(Some(eol)),
        }

        Ok(None)
    }

    pub fn done(mut self) -> Chunk {
        if self.spaces != 0 || self.len != 0 {
            self.push_eol(None);
        }

        self.chunk
    }

    fn push_spaces(&mut self, spaces: u8) -> Option<Chunk> {
        self.spaces += spaces;
        debug_assert!(self.spaces <= LineMeta::MAX_SPACES);

        if self.spaces == 0 {
            None
        } else {
            self.dump(&[])
        }
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> Result<Option<Chunk>, LineError> {
        self.has_error(bytes)?;

        let dump = self.dump(bytes);
        let cursor = self.cursor();
        self.chunk[cursor..][..bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len() as u16;

        Ok(dump)
    }

    fn push_eol(&mut self, eol: Option<Eol>) {
        let cursor = self.cursor();
        let eol_len = eol.map(|eol| eol.as_str().len()).unwrap_or(0);

        let ser = LineMeta::new(self.len, self.spaces, eol).serialize();
        self.chunk[self.index] = ser.0[0];
        self.chunk[self.index + 1] = ser.0[1];
        self.chunk[cursor] = ser.1;

        self.chunk.lines += 1;
        self.chunk.len += self.len + self.spaces as u16 + eol_len as u16;
        self.index = cursor + 1;
        self.len = 0;
        self.spaces = 0;
    }

    fn dump(&mut self, bytes: &[u8]) -> Option<Chunk> {
        if self.cursor() + bytes.len() + 1 >= BYTES {
            let chunk = std::mem::take(&mut self.chunk);

            // Copy last (incomplete) line to new chunk
            if self.index + 2 <= BYTES {
                let line = &chunk[self.index + 2..][..self.len as usize];
                self.chunk[2..][..self.len as usize].copy_from_slice(line);
            } else {
                debug_assert!(self.len == 0);
            }

            // New chunk's offset
            self.chunk.offset.len = chunk.offset.len + chunk.len as usize;
            self.chunk.offset.lines = chunk.offset.lines + chunk.lines as usize;

            self.index = 0;
            debug_assert!(chunk.len != 0);
            Some(chunk)
        } else {
            None
        }
    }

    fn has_error(&self, bytes: &[u8]) -> Result<(), LineError> {
        if self.len as usize + bytes.len() <= Text::LINE_MAX_LEN {
            Ok(())
        } else {
            Err(LineError)
        }
    }

    fn cursor(&self) -> usize {
        debug_assert!(self.index <= BYTES);
        self.index + 2 + self.len as usize
    }
}
