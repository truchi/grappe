mod splitter;

pub use splitter::*;

use std::fmt::Debug;

#[cfg(test)]
mod tests;

// ========================================================================== //
//                                    CONSTANTS                               //
// ========================================================================== //

pub(super) const NEL0: u8 = 0xC2;
pub(super) const NEL1: u8 = 0x85;
pub(super) const S0: u8 = 0xE2;
pub(super) const S1: u8 = 0x80;
pub(super) const LS2: u8 = 0xA8;
pub(super) const PS2: u8 = 0xA9;

/// Line Feed (`0x0A` aka `'\n'`).
pub const LF: u8 = 0x0A;
/// Vertical Tab (`0x0B`).
pub const VT: u8 = 0x0B;
/// Form Feed (`0x0C`).
pub const FF: u8 = 0x0C;
/// Carriage Return (`0x0D` aka `'\r'`).
pub const CR: u8 = 0x0D;
/// Next Line (`[0xC2, 0x85]` aka `'\u{0085}'`).
pub const NEL: [u8; 2] = [NEL0, NEL1];
/// Line Separator (`[0xE2, 0x80, 0xA8]` aka `'\u{2028}'`).
pub const LS: [u8; 3] = [S0, S1, LS2];
/// Paragraph Separator (`[0xE2, 0x80, 0xA9]` aka `'\u{2029}'`).
pub const PS: [u8; 3] = [S0, S1, PS2];
/// Carriage Return + Line Feed (`[0x0D, 0x0A]` aka `"\r\n"`).
pub const CRLF: [u8; 2] = [CR, LF];

// ========================================================================== //
//                                       Eol                                  //
// ========================================================================== //

/// Line endings Unicode sequences.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Eol {
    /// Line Feed (`'\u{000A}'` aka `'\n'`).
    LF,
    /// Vertical Tab (`'\u{000B}'`).
    VT,
    /// Form Feed (`'\u{000C}'`).
    FF,
    /// Carriage Return (`'\u{000D}'` aka `'\r'`).
    CR,
    /// Next Line (`'\u{0085}'`).
    NEL,
    /// Line Separator (`'\u{2028}'`).
    LS,
    /// Paragraph Separator (`'\u{2029}'`).
    PS,
    /// Carriage Return + Line Feed (`"\u{000D}\u{000A}"` aka `"\r\n"`).
    CRLF,
}

impl Eol {
    /// Returns the underlying `&[u8]`.
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Self::LF => &[LF],
            Self::VT => &[VT],
            Self::FF => &[FF],
            Self::CR => &[CR],
            Self::NEL => &NEL,
            Self::LS => &LS,
            Self::PS => &PS,
            Self::CRLF => &CRLF,
        }
    }

    /// Returns the underlying `&str`.
    pub fn as_str(&self) -> &'static str {
        unsafe { utf8!(self.as_bytes()) }
    }

    pub fn as_chars(&self) -> &'static [char] {
        match self {
            Self::LF => &['\u{A}'],
            Self::VT => &['\u{B}'],
            Self::FF => &['\u{C}'],
            Self::CR => &['\u{D}'],
            Self::NEL => &['\u{85}'],
            Self::LS => &['\u{2028}'],
            Self::PS => &['\u{2029}'],
            Self::CRLF => &['\u{D}', '\u{A}'],
        }
    }

    /// Spits `str` at first `Eol`, if any.
    ///
    /// # Example
    ///
    /// ```
    /// # use grappe::Eol;
    /// assert!(Eol::split("No line breaks") == ("No line breaks", None));
    /// assert!(Eol::split("Hello\nUnix") == ("Hello", Some((Eol::LF, "Unix"))));
    /// assert!(Eol::split("Goodbye\r\nWindows\r\n") == ("Goodbye", Some((Eol::CRLF, "Windows\r\n"))));
    /// ```
    pub fn split(str: &str) -> (&str, Option<(Self, &str)>) {
        macro_rules! ret {
            ($str:ident, $i:ident, $len:literal, $eol:ident) => {
                return (&$str[..$i], Some((Self::$eol, &$str[$i + $len..])))
            };
        }

        let bytes = str.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            let is = |byte, len| bytes.get(i + len) == Some(&byte);

            match byte {
                LF => ret!(str, i, 1, LF),
                CR if is(LF, 1) => ret!(str, i, 2, CRLF),
                CR => ret!(str, i, 1, CR),
                VT => ret!(str, i, 1, VT),
                FF => ret!(str, i, 1, FF),
                NEL0 if is(NEL1, 1) => ret!(str, i, 2, NEL),
                S0 if is(S1, 1) =>
                    if is(LS2, 2) {
                        ret!(str, i, 3, LS)
                    } else if is(PS2, 2) {
                        ret!(str, i, 3, PS)
                    },
                _ => {}
            }
        }

        (str, None)
    }

    pub fn starts_with(str: &str) -> (Option<Self>, &str) {
        macro_rules! ret {
            ($str:ident, $i:ident, $len:literal, $eol:ident) => {
                return (Some(Self::$eol), &$str[$len..])
            };
        }

        let bytes = str.as_bytes();
        let is = |byte, len| bytes.get(len) == Some(&byte);

        match bytes.get(0) {
            Some(&LF) => ret!(str, i, 1, LF),
            Some(&CR) if is(LF, 1) => ret!(str, i, 2, CRLF),
            Some(&CR) => ret!(str, i, 1, CR),
            Some(&VT) => ret!(str, i, 1, VT),
            Some(&FF) => ret!(str, i, 1, FF),
            Some(&NEL0) if is(NEL1, 1) => ret!(str, i, 2, NEL),
            Some(&S0) if is(S1, 1) =>
                if is(LS2, 2) {
                    ret!(str, i, 3, LS)
                } else if is(PS2, 2) {
                    ret!(str, i, 3, PS)
                },
            _ => {}
        }

        (None, str)
    }
}

impl AsRef<[u8]> for Eol {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for Eol {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
