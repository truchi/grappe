mod builder;
mod chunk;
mod from_bytes;
mod line;
mod splitter;

pub use builder::*;
pub use chunk::*;
pub use line::*;
pub use splitter::*;

use super::Eol;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

#[derive(Copy, Clone, Default, Debug)]
pub struct LineError;

#[derive(Copy, Clone, Default, Debug)]
pub struct Offset {
    len:   usize,
    lines: usize,
}

#[derive(Clone, Default, Debug)]
pub struct Text {
    pub chunks: Vec<RcChunk>,
    pub len:    usize,
    pub lines:  usize,
}

impl Text {
    // Max len of line (without counting leading spaces and eol)
    pub const LINE_MAX_LEN: usize = chunk::BYTES - 3;

    pub fn open<T: AsRef<Path>>(file: T) -> io::Result<Result<Self, LineError>> {
        Self::read(BufReader::new(File::open(file)?))
    }

    pub fn read<T: BufRead>(mut reader: T) -> io::Result<Result<Self, LineError>> {
        let mut splitter = Splitter::default();
        let mut builder = TextBuilder::default();

        loop {
            let buf = reader.fill_buf()?;
            let len = buf.len();

            if buf.is_empty() {
                break;
            }

            if let Err(err) = splitter.feed(buf, |split| builder.push(split)) {
                return Ok(Err(err));
            }

            reader.consume(len);
        }

        if let Err(err) = splitter.done(|split| builder.push(split)) {
            return Ok(Err(err));
        }

        Ok(Ok(builder.done()))
    }

    pub fn push(&mut self, mut chunk: RcChunk) {
        chunk.offset.len = self.len;
        chunk.offset.lines = self.lines;

        self.len += chunk.len as usize;
        self.lines += chunk.lines as usize;

        self.chunks.push(chunk);
    }
}

impl FromStr for Text {
    type Err = LineError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut splitter = Splitter::default();
        let mut builder = TextBuilder::default();

        splitter.feed(str.as_bytes(), |split| builder.push(split))?;
        splitter.done(|split| builder.push(split))?;

        Ok(builder.done())
    }
}

impl ToString for Text {
    fn to_string(&self) -> String {
        let mut string = String::with_capacity(self.len);

        for chunk in &self.chunks {
            let mut bytes = &chunk[..];

            for _ in (0..chunk.lines).enumerate() {
                let (meta, str, next_bytes) = LineMeta::deserialize(bytes);
                bytes = next_bytes;

                (0..meta.spaces).for_each(|_| string.push(' '));
                string.push_str(str);
                meta.eol.map(|eol| string.push_str(eol.as_str()));
            }
        }

        string
    }
}
