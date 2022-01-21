mod builder;
// mod from_bytes;
// mod splitter;
mod reader;

pub use builder::*;
// pub use splitter::*;
pub use reader::*;

use super::Eol;
use crate::page;
use crate::page::*;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

#[derive(Clone, Default, Debug)]
pub struct Text {
    pub len:   usize,
    pub chars: usize,
    pub lines: usize,
    pub pages: Vec<RcPage>,
}

impl Text {
    // Max len of line (without counting leading spaces and eol)
    pub const LINE_MAX_LEN: usize = page::BYTES - 3;

    /*
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
    */

    // pub fn push(&mut self, mut page: RcPage) {
    // page.offset.len = self.len;
    // page.offset.lines = self.lines;
    //
    // self.len += page.len as usize;
    // self.lines += page.lines as usize;
    //
    // self.pages.push(page);
    // }
}

/*
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

        for page in &self.pages {
            let mut bytes = &page[..];

            for _ in (0..page.lines).enumerate() {
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
*/
