//! The `byte_iter` module provides a wrapper for iterating over the bytes of a MIDI file.

use crate::core::vlq::{decode_slice, VlqError, CONTINUE};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Bytes, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::str::{from_utf8, Utf8Error};

// TODO - make this less weird
/// The internals are weird, it's basically a debugging thing to be able to see the next few values
/// at a breakpoint.
pub(crate) struct ByteIter<R: Read> {
    iter: Bytes<R>,
    position: Option<u64>,
    current: Option<u8>,
    peek1: Option<u8>,
    peek2: Option<u8>,
    peek3: Option<u8>,
    position_limit: Option<u64>,
    /// To help with 'running status', you can save a byte you need to remember here.
    latest_message_byte: Option<u8>,
    running_status_detected: bool,
}

/// A failure while reading bytes, located by byte position.
#[derive(Debug)]
pub(crate) enum ByteError {
    Io {
        position: u64,
        source: std::io::Error,
    },
    End {
        position: u64,
    },
    Str {
        position: u64,
        source: Utf8Error,
    },
    Tag {
        expected: String,
        found: String,
        position: u64,
    },
    VlqTooBig {
        position: u64,
    },
    VlqDecode {
        position: u64,
        source: VlqError,
    },
    ReadExpect {
        expected: u8,
        found: u8,
        position: u64,
    },
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Display for ByteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteError::Io { position, source } => {
                write!(f, "io error around byte {}: {}", position, source)
            }
            ByteError::End { position } => {
                write!(f, "unexpected end reached around byte {}", position)
            }
            ByteError::Str { position, source } => write!(
                f,
                "expected string but found non-utf8 encoded bytes around {}: {}",
                position, source
            ),
            ByteError::Tag {
                expected,
                found,
                position,
            } => write!(
                f,
                "expected tag '{}' but found '{}' near position {}",
                expected, found, position
            ),
            ByteError::VlqTooBig { position } => {
                write!(f, "too many bytes while reading vlq around {}", position)
            }
            ByteError::VlqDecode { position, source } => {
                write!(f, "problem decoding vlq around {}: {}", position, source)
            }
            ByteError::ReadExpect {
                expected,
                found,
                position,
            } => write!(
                f,
                "incorrect byte value around {}: expected '{:#X}', found '{:#X}'",
                position, expected, found
            ),
            ByteError::FileOpen { path, source } => {
                write!(f, "unable to open '{}': {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for ByteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ByteError::Io { source, .. } | ByteError::FileOpen { source, .. } => Some(source),
            ByteError::Str { source, .. } => Some(source),
            ByteError::VlqDecode { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub(crate) type ByteResult<T> = std::result::Result<T, ByteError>;

const BYTE_SIZE: usize = 8;
const KB: usize = BYTE_SIZE * 1024;
const MB: usize = KB * 1024;

impl ByteIter<BufReader<File>> {
    pub(crate) fn new_file<P: AsRef<Path>>(path: P) -> ByteResult<Self> {
        let path = path.as_ref();
        let f = File::open(path).map_err(|source| ByteError::FileOpen {
            path: path.to_path_buf(),
            source,
        })?;
        let buf = BufReader::with_capacity(MB, f);
        Self::new(buf.bytes())
    }
}

impl<R: Read> ByteIter<R> {
    pub(crate) fn new(mut iter: Bytes<R>) -> ByteResult<Self> {
        let peek1 = Self::next_impl(&mut iter, 0)?;
        let peek2 = Self::next_impl(&mut iter, 0)?;
        let peek3 = Self::next_impl(&mut iter, 0)?;
        Ok(Self {
            iter,
            position: None,
            current: None,
            peek1,
            peek2,
            peek3,
            position_limit: None,
            latest_message_byte: None,
            running_status_detected: false,
        })
    }

    /// The byte offset the iter is currently sitting on.
    fn pos(&self) -> u64 {
        self.position.unwrap_or(0)
    }

    fn next_impl(iter: &mut Bytes<R>, position: u64) -> ByteResult<Option<u8>> {
        match iter.next() {
            None => Ok(None),
            Some(result) => match result {
                Ok(val) => Ok(Some(val)),
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
                Err(source) => Err(ByteError::Io { position, source }),
            },
        }
    }

    /// Read a single byte and advance the iter.
    pub(crate) fn read(&mut self) -> ByteResult<Option<u8>> {
        if let Some(position_limit) = self.position_limit {
            if let Some(position) = self.position {
                if position >= position_limit {
                    return Ok(None);
                }
            }
        }
        if self.current.is_none() {
            self.position = Some(0);
        } else if self.current.is_some() {
            self.position = Some(self.position.unwrap_or(0) + 1);
        }
        let return_val = self.peek1;
        self.current = self.peek1;
        self.peek1 = self.peek2;
        self.peek2 = self.peek3;
        match self.iter.next() {
            None => self.peek3 = None,
            Some(Ok(byte)) => self.peek3 = Some(byte),
            Some(Err(e)) if e.kind() == ErrorKind::UnexpectedEof => self.peek3 = None,
            Some(Err(source)) => {
                return Err(ByteError::Io {
                    position: self.pos(),
                    source,
                })
            }
        }
        Ok(return_val)
    }

    pub(crate) fn read_or_die(&mut self) -> ByteResult<u8> {
        let position = self.pos();
        self.read()?.ok_or(ByteError::End { position })
    }

    pub(crate) fn read2(&mut self) -> ByteResult<[u8; 2]> {
        Ok([self.read_or_die()?, self.read_or_die()?])
    }

    pub(crate) fn read4(&mut self) -> ByteResult<[u8; 4]> {
        Ok([
            self.read_or_die()?,
            self.read_or_die()?,
            self.read_or_die()?,
            self.read_or_die()?,
        ])
    }

    pub(crate) fn read_u16(&mut self) -> ByteResult<u16> {
        let bytes: [u8; 2] = self.read2()?;
        Ok(u16::from_be_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self) -> ByteResult<u32> {
        let bytes = self.read4()?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub(crate) fn read_vlq_bytes(&mut self) -> ByteResult<Vec<u8>> {
        let mut retval = Vec::new();
        // initialize with the continue bit set
        let mut current_byte = CONTINUE;
        let mut byte_count = 0u8;
        while current_byte & CONTINUE == CONTINUE {
            // the spec allows at most four bytes (0x0FFFFFFF)
            if byte_count >= 4 {
                return Err(ByteError::VlqTooBig {
                    position: self.pos(),
                });
            }
            current_byte = self.read_or_die()?;
            retval.push(current_byte);
            byte_count += 1;
        }
        Ok(retval)
    }

    pub(crate) fn read_vlq_u32(&mut self) -> ByteResult<u32> {
        let bytes = self.read_vlq_bytes()?;
        decode_slice(&bytes).map_err(|source| ByteError::VlqDecode {
            position: self.pos(),
            source,
        })
    }

    pub(crate) fn current(&self) -> Option<u8> {
        self.current
    }

    pub(crate) fn peek_or_die(&self) -> ByteResult<u8> {
        self.peek1.ok_or(ByteError::End {
            position: self.pos(),
        })
    }

    pub(crate) fn is_end(&self) -> bool {
        if let Some(limit) = self.position_limit {
            debug_assert!(self.position.is_some());
            debug_assert!(self.position.unwrap_or(0) <= limit);
            if self.position.unwrap_or(0) >= limit {
                return true;
            }
        }
        self.current.is_none()
    }

    pub(crate) fn expect_tag(&mut self, expected_tag: &str) -> ByteResult<()> {
        let tag_bytes = self.read4()?;
        let actual_tag = from_utf8(&tag_bytes).map_err(|source| ByteError::Str {
            position: self.pos(),
            source,
        })?;
        if expected_tag != actual_tag {
            return Err(ByteError::Tag {
                expected: expected_tag.to_string(),
                found: actual_tag.to_string(),
                position: self.pos(),
            });
        }
        Ok(())
    }

    /// When this is set, the ByteIter will report that it is at the end when `size` bytes have been
    /// read.
    pub(crate) fn set_size_limit(&mut self, size: u64) {
        self.position_limit = Some(self.position.unwrap_or(0) + size)
    }

    pub(crate) fn clear_size_limit(&mut self) {
        self.position_limit = None
    }

    pub(crate) fn read_expect(&mut self, expected: u8) -> ByteResult<()> {
        let found = self.read_or_die()?;
        if expected != found {
            return Err(ByteError::ReadExpect {
                expected,
                found,
                position: self.pos(),
            });
        }
        Ok(())
    }

    pub(crate) fn read_n(&mut self, num_bytes: usize) -> ByteResult<Vec<u8>> {
        let mut bytes = Vec::with_capacity(num_bytes);
        for _ in 0..num_bytes {
            bytes.push(self.read_or_die()?)
        }
        debug_assert_eq!(num_bytes, bytes.len());
        Ok(bytes)
    }

    /// Read and discard `num_bytes` bytes.
    pub(crate) fn skip_n(&mut self, num_bytes: usize) -> ByteResult<()> {
        for _ in 0..num_bytes {
            self.read_or_die()?;
        }
        Ok(())
    }

    pub(crate) fn set_latest_message_byte(&mut self, value: Option<u8>) {
        self.latest_message_byte = value;
    }

    pub(crate) fn latest_message_byte(&self) -> Option<u8> {
        self.latest_message_byte
    }

    pub(crate) fn set_running_status_detected(&mut self) {
        self.running_status_detected = true;
    }

    pub(crate) fn is_running_status_detected(&self) -> bool {
        self.running_status_detected
    }
}

#[test]
fn byte_iter_test() {
    use std::io::Cursor;
    let bytes = [0x00u8, 0x01, 0x02, 0x03, 0x04, 0x10, 0x20, 0x30, 0x40];
    let cursor = Cursor::new(bytes);
    let mut iter = ByteIter::new(cursor.bytes()).unwrap();
    assert!(iter.current.is_none());
    assert_eq!(0x00, iter.read().unwrap().unwrap());
    assert_eq!(0x00, iter.current.unwrap());
    assert_eq!(0x01, iter.peek1.unwrap());
    assert_eq!(0x02, iter.peek2.unwrap());
    assert_eq!(0x03, iter.peek3.unwrap());

    assert_eq!([0x01, 0x02], iter.read2().unwrap());
    assert_eq!(2, iter.position.unwrap());
    iter.set_size_limit(2);
    assert!(!iter.is_end());
    assert_eq!(0x03, iter.read().unwrap().unwrap());
    assert_eq!(0x04, iter.read().unwrap().unwrap());
    assert_eq!(0x04, iter.current().unwrap());
    assert!(iter.read().unwrap().is_none());
    iter.clear_size_limit();
    assert_eq!(0x10, iter.read().unwrap().unwrap());
}
