use super::page;
use super::Eol;
use super::Page;
use super::Text;
use crate::line::LineMeta;
use crate::Offset;

#[derive(Default, Debug)]
pub struct TextBuilder {
    text: Text,
    page: Page,
}

impl TextBuilder {
    pub fn push(&mut self, str: &str) {}

    pub fn done(mut self) -> Text {
        self.text
    }
}

// =================

#[derive(Default, Debug)]
pub struct PageBuilder {
    page:  Page,
    index: usize,
    meta:  LineMeta,
}

impl PageBuilder {
    pub fn push_str<'a>(&mut self, mut str: &'a str) -> Option<&'a str> {
        if let Some(eol) = self.meta.eol {
            if eol == Eol::CR && str.starts_with("\n") {
                self.meta.eol = Some(Eol::CRLF);
                str = &str[1..];
            }

            self.advance();
            return Some(str);
        } else if self.meta.spaces == 0 && self.meta.len == 0 {
            return Some(str);
        } else {
            if self.meta.len == 0 {
                str = self.push_spaces(str);
            }

            let end = self.index + LineMeta::BYTES + self.meta.len as usize;
            str = self.push_line(str, page::BYTES - end);

            if str.is_empty() {
                return None;
            } else {
                Some(str)
            }
        }
    }

    pub fn push<'a>(&'a mut self, mut str: &'a str) -> impl 'a + Iterator<Item = Page> {
        dbg!(str);

        let mut first = None;

        if let Some(eol) = self.meta.eol {
            dbg!(eol);
            if eol == Eol::CR && str.starts_with("\n") {
                self.meta.eol = Some(Eol::CRLF);
                str = &str[1..];
            }
        } else {
            // TODO if at start of line only
            println!("no eol");
            if let (Some(eol), tail) = Eol::starts_with(str) {
                if self.index < page::BYTES {
                    self.push_eol(Some(eol));
                    str = tail;
                } else {
                    first = Some(self.flush());
                }
            } else {
                let end = self.index + LineMeta::BYTES + self.meta.len as usize;
                if end < page::BYTES {
                    println!("la fisdep");

                    if self.meta.len == 0 {
                        str = self.push_spaces(str);
                    }
                    str = self.push_line(str, page::BYTES - end);
                    // self.advance();
                } else {
                    println!("batar {} {:#?}", self.index, self.meta);
                    first = Some(self.flush());
                }
            };
        }

        dbg!(str);
        dbg!(first.is_some());

        first.into_iter().chain(std::iter::from_fn(move || {
            loop {
                println!("LOOP {:?} {:#?}", str, self.meta);
                if str.is_empty() {
                    return None;
                }

                // if self.index != 0 {
                // self.advance();
                // }

                str = if let (Some(eol), str) = Eol::starts_with(str) {
                    if self.index < page::BYTES {
                        // self.meta.eol = Some(eol);
                        self.push_eol(Some(eol));
                        str
                    } else {
                        println!("flush 1");
                        return Some(self.flush());
                    }
                } else {
                    println!("ici");
                    // str = self.push_spaces(str);

                    debug_assert!(self.meta.len == 0);
                    let end = self.index + LineMeta::BYTES;
                    if end < page::BYTES {
                        str = self.push_spaces(str);

                        self.push_line(str, page::BYTES - end)
                    } else {
                        println!("flush 4");
                        return Some(self.flush());
                    }
                };
                // self.advance();
            }

            None
        }))
    }

    pub fn done(mut self) -> Page {
        // if self.meta.is_valid() {
        self.advance();
        // }
        self.page
    }

    fn push_spaces<'a>(&mut self, str: &'a str) -> &'a str {
        debug_assert!(self.meta.len == 0);
        debug_assert!(self.meta.eol.is_none());

        let max = LineMeta::SPACES_MAX - self.meta.spaces;
        let (spaces, str) = split_spaces(str, max as usize);

        self.meta.spaces += spaces.len() as u8;
        str
    }

    fn push_line<'a>(&mut self, str: &'a str, at: usize) -> &'a str {
        let (left, right) = split_at(str, at);
        let (line, eol) = Eol::split(left);

        self.write(line);

        let (eol, str) = if let Some((eol, _)) = eol {
            (Some(eol), &str[line.len() + eol.as_bytes().len()..])
        } else if let (Some(eol), str) = Eol::starts_with(right) {
            (Some(eol), str)
        } else {
            (None, right)
        };

        // self.meta.eol = eol;
        self.push_eol(eol);
        str
    }

    fn push_eol(&mut self, eol: Option<Eol>) {
        self.meta.eol = eol;
        self.advance();
    }

    fn flush(&mut self) -> Page {
        let page = std::mem::take(&mut self.page);

        self.index = 0;
        self.page.offset.len += page.len as usize;
        self.page.offset.chars += page.chars as usize;
        self.page.offset.lines += page.lines as usize;

        page
    }

    fn write(&mut self, str: &str) {
        let end = self.index + LineMeta::BYTES + self.meta.len as usize;

        self.page[end..][..str.len()].copy_from_slice(str.as_bytes());
        self.meta.len += str.len() as u16;
        self.meta.chars += str.chars().count() as u16;
    }

    fn advance(&mut self) {
        debug_assert!(self.meta.is_valid());

        let meta = &self.meta.serialize()[..self.meta.width()];
        self.page[self.index..][..meta.len()].copy_from_slice(meta);

        let spaces = self.meta.spaces as u16;
        let (eol_len, eol_chars) = self
            .meta
            .eol
            .map(|eol| (eol.as_bytes().len() as u16, eol.as_chars().len() as u16))
            .unwrap_or((0, 0));

        self.index += self.meta.width() + self.meta.len as usize;
        self.page.len += spaces + self.meta.len + eol_len;
        self.page.chars += spaces + self.meta.chars + eol_chars;
        self.page.lines += 1;
        self.page.end = self.index as u16;

        self.meta = LineMeta::default();
    }

    fn split<'a>(&self, str: &'a str) -> (&'a str, Option<Eol>, &'a str) {
        let end = self.index + LineMeta::BYTES + self.meta.len as usize;
        debug_assert!(end < page::BYTES);

        let (left, right) = split_at(str, end);
        let (line, eol) = Eol::split(left);

        if let Some((eol, _)) = eol {
            (line, Some(eol), &str[line.len() + eol.as_bytes().len()..])
        } else if let (Some(eol), str) = Eol::starts_with(right) {
            (line, Some(eol), str)
        } else {
            (line, None, right)
        }
    }
}

fn split_at(str: &str, max: usize) -> (&str, &str) {
    let mut max = max.min(str.len());

    while !str.is_char_boundary(max) {
        max -= 1;
    }

    (&str[..max], &str[max..])
}

fn split_spaces(str: &str, max: usize) -> (&str, &str) {
    let i = str[..max.min(str.len())]
        .as_bytes()
        .iter()
        .take_while(|&&b| b == b' ')
        .count();

    (&str[..i], &str[i..])
}
