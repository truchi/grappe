use std::str::from_utf8;

/// An error raised when trying to validate non UTF-8 bytes.
#[derive(Copy, Clone, Default, Debug)]
pub struct Utf8Error;

/// Validates byte chunks.
///
/// # Usage
///
/// ```
/// # use grappe::utils::Validator;
/// let mut validator = Validator::default();
///
/// // The file we are reading
/// let file = "Hello 🦀!".as_bytes(); // Crab emoji is 1 char, 4 bytes
///
/// // Let's pretend we are using a BufReader...
/// let mut chunks = file.chunks(2);
///
/// // Feeds next chunk to the validator and test for validity
/// let mut validate = || {
///     validator
///         .validate(chunks.next().unwrap())
///         .expect("be valid")
/// };
///
/// // Feed chunks
/// assert!(validate() == (None, "He"));
/// assert!(validate() == (None, "ll"));
/// assert!(validate() == (None, "o "));
/// assert!(validate() == (None, "")); // Waiting for more bytes...
/// assert!(validate() == (Some('🦀'), ""));
/// assert!(validate() == (None, "!"));
///
/// // After feeding all chunks
/// validator.done().expect("be valid");
/// ```
#[derive(Copy, Clone, Default, Debug)]
pub struct Validator {
    joint: Joint,
}

impl Validator {
    /// Feeds `bytes` into the validator.
    ///
    /// Returns the eventual overlapping `char` from the previous chunk
    /// and the valid part of this chunk.
    pub fn validate<'a, 'b>(
        &'a mut self,
        mut bytes: &'b [u8],
    ) -> Result<(Option<char>, &'b str), Utf8Error> {
        if self.joint.is_empty() {
            Ok((None, self.valid(bytes)?))
        } else if self.joint.is_full() {
            let char = self.joint.validate()?;
            debug_assert!(char.is_some());
            Ok((char, self.valid(bytes)?))
        } else {
            loop {
                debug_assert!(!self.joint.is_full());

                if let Some(&byte) = bytes.get(0) {
                    bytes = &bytes[1..];

                    if let Some(char) = self.joint.push(byte).validate()? {
                        return Ok((Some(char), self.valid(bytes)?));
                    }
                } else {
                    return Ok((None, ""));
                }
            }
        }
    }

    /// Tests if there are overlapping bytes.
    ///
    /// Call at the end of the chunk stream to complete validation.
    /// State is reset to allow reuse for another chunk stream.
    pub fn done(&mut self) -> Result<(), Utf8Error> {
        if self.joint.is_empty() {
            Ok(())
        } else {
            self.joint.clear();
            Err(Utf8Error)
        }
    }

    fn valid<'a>(&mut self, bytes: &'a [u8]) -> Result<&'a str, Utf8Error> {
        let (str, joint): (&str, &[u8]) = match from_utf8(bytes) {
            Ok(str) => (str, &[]),
            Err(err) => {
                if err.error_len().is_some() {
                    return Err(Utf8Error);
                }

                let valid = err.valid_up_to();
                (unsafe { utf8!(&bytes[..valid]) }, &bytes[valid..])
            }
        };

        self.joint = Joint::new(joint);
        Ok(str)
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct Joint {
    len:   usize,
    bytes: [u8; Self::BYTES],
}

impl Joint {
    const BYTES: usize = std::mem::size_of::<char>();

    fn new(bytes: &[u8]) -> Self {
        let mut joint = Self::default();
        joint.bytes[..bytes.len()].copy_from_slice(bytes);
        joint.len = bytes.len();

        joint
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn is_full(&self) -> bool {
        self.len == Self::BYTES
    }

    fn push(&mut self, byte: u8) -> &mut Self {
        self.bytes[self.len] = byte;
        self.len += 1;
        self
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn validate(&self) -> Result<Option<char>, Utf8Error> {
        debug_assert!(!self.is_empty());

        match from_utf8(&self.bytes[..self.len]) {
            Ok(str) => {
                debug_assert!(str.chars().count() == 1);
                Ok(str.chars().next())
            }
            Err(err) =>
                if err.error_len().is_some() {
                    Err(Utf8Error)
                } else {
                    debug_assert!(err.valid_up_to() == 0);
                    Ok(None)
                },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    test_files!("../../../../");

    #[test]
    fn test() {
        const SIZES: &[usize] = &[1, 2, 3, 4, 512, 8 * 1024];

        for &file in FILES {
            for &size in SIZES {
                let mut string = String::new();
                let mut chunks = file.as_bytes().chunks(size);
                let mut validator = Validator::default();

                for chunk in chunks {
                    let (char, str) = validator.validate(chunk).expect("be valid");

                    char.map(|char| string.push(char));
                    string.push_str(str);
                }

                validator.done().expect("be valid");
                assert!(file == string);
            }
        }
    }
}
