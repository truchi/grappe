use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str::from_utf8;

const CAPACITY: usize = 8 * 1024;

/// Reads and validates (UTF-8) bytes.
#[derive(Copy, Clone, Debug)]
pub struct Reader<T: Read> {
    reader: T,
    buffer: [u8; CAPACITY],
    len:    usize,
    valid:  usize,
}

impl Reader<File> {
    /// Creates a new `Reader` from a file path.
    pub fn open<P: AsRef<Path>>(file: P) -> io::Result<Self> {
        Ok(Self {
            reader: File::open(file)?,
            buffer: [0; CAPACITY],
            len:    0,
            valid:  CAPACITY,
        })
    }
}

impl<T: Read> Reader<T> {
    /// Creates a new `Reader` from `reader`.
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            buffer: [0; CAPACITY],
            len: 0,
            valid: CAPACITY,
        }
    }

    /// Reads from the reader, returning:
    /// - `Err`: io error
    /// - `Ok`:
    ///   - `None`: end of stream
    ///   - `Some`:
    ///     - `None`: UTF-8 validation error
    ///     - `Some`: `&str`
    pub fn read(&mut self) -> io::Result<Option<Option<&str>>> {
        // TODO: do not split `\r\n`s!

        let invalid = self.prepare_buffer();
        let len = self.reader.read(&mut self.buffer[invalid..])?;
        let bytes = &self.buffer[..len];

        Ok(if !bytes.is_empty() {
            Some(match from_utf8(bytes) {
                Ok(str) => Some(str),
                Err(err) => match err.error_len() {
                    Some(_) => None,
                    None => {
                        self.valid = err.valid_up_to();
                        Some(unsafe { utf8!(&bytes[..self.valid]) })
                    }
                },
            })
        } else {
            None
        })
    }

    /// Copies invalid bytes at the end of the buffer to the start,
    /// returning the amout.
    fn prepare_buffer(&mut self) -> usize {
        // NOTE: this works only if CAPACITY >= 8
        let (left, right) = self.buffer.split_at_mut(self.valid);
        left[..right.len()].copy_from_slice(right);

        right.len()
    }
}
