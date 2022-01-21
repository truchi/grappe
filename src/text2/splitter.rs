use super::LineMeta;
use crate::eol;
use crate::Eol;

#[derive(Copy, Clone, Debug)]
pub enum Split<'a> {
    Spaces(u8),
    Bytes(&'a [u8]),
    Eol(Eol),
}

#[derive(Default, Debug)]
pub struct Splitter {
    state:    State,
    splitter: eol::Splitter,
}

impl Splitter {
    pub fn feed<'a, E, T>(&mut self, bytes: &'a [u8], mut f: T) -> Result<(), E>
    where
        T: FnMut(Split<'a>) -> Result<(), E>,
    {
        self.splitter.feed::<E, _>(bytes, |split| match split {
            eol::Split::Bytes(bytes) => self.state.feed_bytes(bytes, &mut f),
            eol::Split::Eol(eol) => self.state.eol(eol, &mut f),
        })
    }

    pub fn done<E, T>(mut self, mut f: T) -> Result<(), E>
    where
        T: FnMut(Split<'static>) -> Result<(), E>,
    {
        self.splitter.done::<E, _>(|split| match split {
            eol::Split::Bytes(bytes) => self.state.done_bytes(bytes, &mut f),
            eol::Split::Eol(eol) => self.state.eol(eol, &mut f),
        })
    }
}

#[derive(Debug)]
enum State {
    Spaces(u8),
    Bytes,
}

impl Default for State {
    fn default() -> Self {
        Self::Spaces(0)
    }
}

impl State {
    fn feed_bytes<'a, E, T>(&mut self, bytes: &'a [u8], mut f: T) -> Result<(), E>
    where
        T: FnMut(Split<'a>) -> Result<(), E>,
    {
        match *self {
            State::Spaces(spaces) => {
                let count = leading_spaces(spaces, bytes);

                if let Some(byte) = bytes.get(count as usize) {
                    let bytes = if byte.is_ascii() {
                        f(Split::Spaces(spaces + count))?;

                        &bytes[count as usize..]
                    } else if count == 0 {
                        if spaces != 0 {
                            f(Split::Spaces(spaces - 1))?;
                            f(Split::Bytes(&[b' ']))?;
                        }

                        bytes
                    } else {
                        let count = count - 1;
                        f(Split::Spaces(spaces + count))?;

                        &bytes[count as usize..]
                    };

                    *self = State::Bytes;
                    f(Split::Bytes(bytes))
                } else {
                    *self = State::Spaces(spaces + count);
                    Ok(())
                }
            }
            State::Bytes => f(Split::Bytes(bytes)),
        }
    }

    fn done_bytes<'a, E, T>(&mut self, bytes: &'a [u8], mut f: T) -> Result<(), E>
    where
        T: FnMut(Split<'a>) -> Result<(), E>,
    {
        debug_assert!(!bytes[0].is_ascii());

        match *self {
            State::Spaces(spaces) => {
                if spaces != 0 {
                    f(Split::Spaces(spaces - 1))?;
                    f(Split::Bytes(&[b' ']))?;
                }
                f(Split::Bytes(bytes))
            }
            State::Bytes => f(Split::Bytes(bytes)),
        }
    }

    fn eol<'a, E, T>(&mut self, eol: Eol, mut f: T) -> Result<(), E>
    where
        T: FnMut(Split<'a>) -> Result<(), E>,
    {
        if let State::Spaces(spaces) = *self {
            f(Split::Spaces(spaces))?;
        }

        *self = State::Spaces(0);
        f(Split::Eol(eol))
    }
}

fn leading_spaces(spaces: u8, bytes: &[u8]) -> u8 {
    let mut count = 0;
    let max = LineMeta::MAX_SPACES - spaces;
    let max = (max as usize).min(bytes.len());

    for b in &bytes[..max] {
        if *b == b' ' {
            count += 1;
        } else {
            break;
        }
    }

    return count;
}
