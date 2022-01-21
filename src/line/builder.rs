use super::super::Split;
use super::LineMeta;
use crate::utils::Validator;
use crate::Eol;
use std::str::from_utf8;
use std::str::from_utf8_unchecked;

#[derive(Copy, Clone, Default, Debug)]
pub struct LineBuilder {
    spaces:    u8,
    len:       u16,
    validator: Validator,
    eol:       bool,
}

impl LineBuilder {
    const SPACES: &'static str = unsafe { from_utf8_unchecked(&[b' '; u8::MAX as usize]) };

    pub fn push_spaces<'a>(&mut self, chunk: &mut [u8], spaces: u8) -> Option<&'a str> {
        debug_assert!(!self.eol);
        debug_assert!(chunk.len() >= LineMeta::BYTES);
        debug_assert!(self.spaces <= LineMeta::MAX_SPACES);

        if self.len == 0 {
            let available = LineMeta::MAX_SPACES - self.spaces;

            if spaces <= available {
                self.spaces += spaces;
                None
            } else {
                self.spaces = LineMeta::MAX_SPACES;
                self.push_str(chunk, &Self::SPACES[..(spaces - available) as usize])
            }
        } else {
            self.push_str(chunk, &Self::SPACES[..spaces as usize])
        }
    }

    pub fn push_str<'a>(&mut self, chunk: &mut [u8], str: &'a str) -> Option<&'a str> {
        debug_assert!(!self.eol);
        debug_assert!(chunk.len() >= LineMeta::BYTES);
        debug_assert!(!self.validator.has_joint()); // This should be an assert, really.

        let (str, more) = split(str, chunk.len() - self.end());

        // TODO count spaces
        if self.len == 0 {
        } else {
        }

        self.write(chunk, str.as_bytes());

        if more.is_empty() {
            None
        } else {
            self.push_eol(chunk, None);
            Some(more)
        }
    }

    pub fn push_bytes<'a>(&mut self, chunk: &mut [u8], bytes: &'a [u8]) -> Option<&'a [u8]> {
        debug_assert!(!self.eol);
        debug_assert!(chunk.len() >= LineMeta::BYTES);
        None
    }

    pub fn push_eol(&mut self, chunk: &mut [u8], eol: Option<Eol>) {
        debug_assert!(!self.eol);
        debug_assert!(chunk.len() >= LineMeta::BYTES);

        self.eol = true;
    }

    fn write<'a, T: AsRef<[u8]>>(&mut self, chunk: &mut [u8], bytes: T) {
        let bytes = bytes.as_ref();

        debug_assert!(!self.eol);
        debug_assert!(self.len() + bytes.len() <= LineMeta::MAX_LEN as usize);
        debug_assert!(self.end() + bytes.len() <= chunk.len());
        debug_assert!(from_utf8(bytes).is_ok());

        chunk[self.end()..][..bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len() as u16;
    }

    fn len(&self) -> usize {
        self.len as usize
    }

    fn end(&self) -> usize {
        self.len() + LineMeta::BYTES
    }
}

fn split(str: &str, mut at: usize) -> (&str, &str) {
    if at >= str.len() {
        (str, "")
    } else {
        while !str.is_char_boundary(at) {
            at -= 1;
        }

        str.split_at(at)
    }
}

// pub fn push<'a>(&mut self, chunk: &mut [u8], split: Split<'a>) -> Option<&'a
// [u8]> { debug_assert!(chunk.len() >= 3);
// let len = self.len as usize;
//
// match split {
// Split::Spaces(spaces) => {}
// Split::Bytes(bytes) => {
// debug_assert!(chunk.len() >= len + 3);
// let chunk = &mut chunk[len + 3..];
//
// return if chunk.len() >= bytes.len() {
// chunk[..bytes.len()].copy_from_slice(bytes);
// None
// } else {
// chunk.copy_from_slice(&bytes[..chunk.len()]);
// Some(&bytes[chunk.len()..])
// };
// }
// Split::Eol(eol) => {}
// }
//
// None
// }
