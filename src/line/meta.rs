use crate::Eol;

// 32 bits:
// 1  is_empty
// 4  Option<Eol>
// 10 len
// 10 chars
// 7  spaces

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct LineMeta {
    pub len:    u16,
    pub chars:  u16,
    pub spaces: u8,
    pub eol:    Option<Eol>,
}

macro_rules! consts {
    ($($name:ident($ty:ty): $MAX:ident $MASK:ident $BITS:ident ($bits:expr) $SHL:ident $(($prev:ident))?)*) => {
        mod consts { $(
            pub(super) mod $name {
                use super::super::*;
                pub const BITS: u32 = $bits;
                pub const SHL: u32 = 0 $(+ super::$prev::BITS + super::$prev::SHL)?;
                pub const MASK: u32 = u32::MAX >> (u32::BITS - BITS) ;
                pub const MAX: $ty = (2_u32.pow(BITS) - 1) as $ty;
            }
        )* }

        impl LineMeta {
            $(
                pub const $BITS: u32 = consts::$name::BITS;
                pub const $MAX: $ty = consts::$name::MAX;
                const $SHL: u32 = consts::$name::SHL;
                const $MASK: u32 = consts::$name::MASK;
            )*

            fn deser_u32(u32: u32) -> ($($ty),*) {
                $(
                    let u32 = u32 $(>> consts::$prev::BITS)?;
                    let $name = u32 & consts::$name::MASK;
                    debug_assert!($name <= consts::$name::MAX as u32);
                    let $name = $name as $ty;
                )*

                ($($name),*)
            }
        }
    };
}

consts!(
    spaces(u8) : SPACES_MAX SPACES_MASK SPACES_BITS ( 7) SPACES_SHL
    chars (u16): CHARS_MAX  CHARS_MASK  CHARS_BITS  (10) CHARS_SHL  (spaces)
    len   (u16): LEN_MAX    LEN_MASK    LEN_BITS    (10) LEN_SHL    (chars)
    eol   (u8) : EOL_MAX    EOL_MASK    EOL_BITS    ( 4) EOL_SHL    (len)
);

impl LineMeta {
    pub const BYTES: usize = 4;
    pub const IS_EMPTY_MASK: u8 = 1 << (u8::BITS - 1);

    pub fn new(len: u16, chars: u16, spaces: u8, eol: Option<Eol>) -> Self {
        debug_assert!(len <= Self::LEN_MAX);
        debug_assert!(chars <= Self::CHARS_MAX);
        debug_assert!(spaces <= Self::SPACES_MAX);
        debug_assert!(if chars == 0 { len == 0 } else { true });
        debug_assert!(if len == 0 { chars == 0 } else { true });
        debug_assert!(if len == 0 && spaces == 0 {
            eol.is_some() // Or the line would be void (no bytes nor eol)
        } else {
            true
        });

        Self {
            len,
            chars,
            spaces,
            eol,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.len <= Self::LEN_MAX
            && self.chars <= Self::CHARS_MAX
            && self.spaces <= Self::SPACES_MAX
            && if self.chars == 0 { self.len == 0 } else { true }
            && if self.len == 0 { self.chars == 0 } else { true }
            && if self.len == 0 && self.spaces == 0 {
                self.eol.is_some() // Or the line would be void
                                   // (no bytes nor eol)
            } else {
                true
            }
    }

    pub fn is_empty(&self) -> bool {
        if self.len == 0 && self.spaces == 0 {
            true
        } else {
            false
        }
    }

    pub fn width(&self) -> usize {
        if self.is_empty() {
            1
        } else {
            4
        }
    }

    pub fn serialize(&self) -> [u8; 4] {
        if self.is_empty() {
            [Self::IS_EMPTY_MASK | eol_to_u8(self.eol) << 3, 0, 0, 0]
        } else {
            (0 | (self.spaces as u32) << Self::SPACES_SHL
                | (self.chars as u32) << Self::CHARS_SHL
                | (self.len as u32) << Self::LEN_SHL
                | (eol_to_u8(self.eol) as u32) << Self::EOL_SHL)
                .to_be_bytes()
        }
    }

    pub fn deserialize(bytes: &[u8]) -> (Self, &[u8]) {
        if bytes.len() >= 4 {
            if bytes[0] >= Self::IS_EMPTY_MASK {
                (Self::deserialize_1(bytes[0]), &bytes[1..])
            } else {
                (Self::deserialize_4(bytes), &bytes[4..])
            }
        } else {
            assert!(!bytes.is_empty());
            assert!(bytes[0] >= Self::IS_EMPTY_MASK);
            (Self::deserialize_1(bytes[0]), &bytes[1..])
        }
    }

    pub fn deserialize_1(byte: u8) -> Self {
        debug_assert!(byte >= Self::IS_EMPTY_MASK);

        let eol = u8_to_eol(byte << 1 >> 4);
        debug_assert!(eol.is_some());

        Self::new(0, 0, 0, eol)
    }

    pub fn deserialize_4(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() >= 4);

        let mut u32 = [0; 4];
        u32.copy_from_slice(&bytes[0..4]);

        let (spaces, chars, len, eol) = Self::deser_u32(u32::from_be_bytes(u32));
        debug_assert!(spaces as u16 + chars + len > 0);

        Self::new(len, chars, spaces, u8_to_eol(eol))
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
