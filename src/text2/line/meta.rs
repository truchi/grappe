use crate::Eol;

pub struct LineMeta {
    pub len:    u16,
    pub spaces: u8,
    pub eol:    Option<Eol>,
}

impl LineMeta {
    pub const LEN_BITS: u16 = 10;
    pub const LEN_MASK: u16 = u16::MAX >> Self::SPACES_BITS;
    pub const MAX_LEN: u16 = 2u16.pow(Self::LEN_BITS as u32) - 1;
    pub const MAX_SPACES: u8 = 2u8.pow(Self::SPACES_BITS as u32) - 1;
    pub const SPACES_BITS: u16 = 6;
    pub const SPACES_MASK: u16 = u16::MAX << Self::LEN_BITS;

    pub fn new(len: u16, spaces: u8, eol: Option<Eol>) -> Self {
        Self { len, spaces, eol }
    }

    pub fn serialize(&self) -> ([u8; 2], u8) {
        debug_assert!(self.len <= Self::MAX_LEN);
        debug_assert!(self.spaces <= Self::MAX_SPACES);

        (
            (self.len | (self.spaces as u16) << Self::LEN_BITS).to_le_bytes(),
            eol_to_u8(self.eol),
        )
    }

    pub fn deserialize(bytes: &[u8]) -> (Self, &str, &[u8]) {
        debug_assert!(bytes.len() >= 3);
        let (len, spaces) = Self::deserialize_len([bytes[0], bytes[1]]);

        debug_assert!(bytes.len() >= len as usize + 3);
        let eol = Self::deserialize_eol(bytes[len as usize + 2]);

        (
            Self { spaces, len, eol },
            std::str::from_utf8(&bytes[2..][..len as usize]).unwrap(),
            &bytes[len as usize + 3..],
        )
    }

    fn deserialize_len(bytes: [u8; 2]) -> (u16, u8) {
        let bytes = u16::from_le_bytes(bytes);
        let spaces = (bytes >> Self::LEN_BITS);
        let len = bytes & Self::LEN_MASK;

        debug_assert!(spaces <= u8::MAX as u16);
        (len, spaces as u8)
    }

    fn deserialize_eol(byte: u8) -> Option<Eol> {
        u8_to_eol(byte)
    }
}

macro_rules! eol_from_to_u8 {
    (None => $none:literal, $($Eol:ident => $u8:literal,)*) => {
        fn eol_to_u8(eol: Option<Eol>) -> u8 {
            match eol {
                None => $none,
                $(Some(Eol::$Eol) => $u8,)*
            }
        }

        fn u8_to_eol(u8: u8) -> Option<Eol> {
            match u8 {
                $none => None,
                $($u8 => Some(Eol::$Eol),)*
                _ => unreachable!(),
            }
        }
    };
}

eol_from_to_u8!(
    None => 0,
    LF   => 1,
    VT   => 2,
    FF   => 3,
    CR   => 4,
    NEL  => 5,
    LS   => 6,
    PS   => 7,
    CRLF => 8,
);
