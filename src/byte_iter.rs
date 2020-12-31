use crate::vlq::{decode_slice, VlqError, CONTINUE};
use log::trace;
use snafu::{ensure, OptionExt, ResultExt, Snafu};
use std::fs::File;
use std::io::{BufReader, Bytes, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::str::{from_utf8, Utf8Error};

pub(crate) struct ByteIter<R: Read> {
    iter: Bytes<R>,
    position: Option<u64>,
    current: Option<u8>,
    peek1: Option<u8>,
    peek2: Option<u8>,
    peek3: Option<u8>,
    position_limit: Option<u64>,
}

#[derive(Debug, Snafu)]
pub(crate) enum ByteError {
    #[snafu(display("io error around byte {}: {}", position, source))]
    Io {
        position: u64,
        source: std::io::Error,
    },

    #[snafu(display("unexpended end reached around byte {}", position))]
    End { position: u64 },

    #[snafu(display(
        "expected string but found non-utf8 encoded bytes around {}: {}",
        position,
        source
    ))]
    Str { position: u64, source: Utf8Error },

    #[snafu(display(
        "expected tag '{}' but found '{}' near position {}",
        expected,
        found,
        position
    ))]
    Tag {
        expected: String,
        found: String,
        position: u64,
    },

    #[snafu(display("too many bytes while reading vlq around {}", position))]
    VlqTooBig { position: u64 },

    #[snafu(display("problem decoding vlq around {}: {}", position, source))]
    VlqDecode { position: u64, source: VlqError },

    #[snafu(display(
        "incorrect byte value around {}: expected '{:#X}', found '{:#X}'",
        position,
        expected,
        found,
    ))]
    ReadExpect {
        expected: u8,
        found: u8,
        position: u64,
    },

    #[snafu(display("unable to open '{}': {}", path.display(), source,))]
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub(crate) type ByteResult<T> = std::result::Result<T, ByteError>;

const BYTE_SIZE: usize = 8;
const KB: usize = BYTE_SIZE * 1024;
const MB: usize = KB * 1024;

impl ByteIter<BufReader<File>> {
    pub(crate) fn new_file<P: AsRef<Path>>(path: P) -> ByteResult<Self> {
        let path = path.as_ref();
        let f = File::open(path).context(FileOpen { path })?;
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
        })
    }

    fn next_impl(iter: &mut Bytes<R>, position: u64) -> ByteResult<Option<u8>> {
        match iter.next() {
            None => Ok(None),
            Some(result) => match result {
                Ok(val) => Ok(Some(val)),
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
                Err(e) => Err(e).context(Io { position }),
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
        let next_opt = self.iter.next();
        let next_result = match next_opt {
            None => {
                self.peek3 = None;
                trace!(
                    "read {:#x} at position {}",
                    return_val.unwrap_or(0),
                    self.position.unwrap_or(0)
                );
                return Ok(return_val);
            }
            Some(r) => r,
        };

        let e = match next_result {
            Ok(ok) => {
                self.peek3 = Some(ok);
                trace!(
                    "read {:#x} at position {}",
                    return_val.unwrap_or(0),
                    self.position.unwrap_or(0)
                );
                return Ok(return_val);
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    self.peek3 = None;
                    trace!(
                        "read {:#x} at position {}",
                        return_val.unwrap_or(0),
                        self.position.unwrap_or(0)
                    );
                    return Ok(return_val);
                }
                e
            }
        };
        Err(e).context(Io {
            position: self.position.unwrap_or(0),
        })
    }

    pub(crate) fn read_or_die(&mut self) -> ByteResult<u8> {
        self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })
    }

    pub(crate) fn read2(&mut self) -> ByteResult<[u8; 2]> {
        let mut retval = [0u8; 2];
        retval[0] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        retval[1] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        Ok(retval)
    }

    pub(crate) fn read4(&mut self) -> ByteResult<[u8; 4]> {
        let mut retval = [0u8; 4];
        retval[0] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        retval[1] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        retval[2] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        retval[3] = self.read()?.context(End {
            position: self.position.unwrap_or(0),
        })?;
        Ok(retval)
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
            ensure!(
                byte_count <= 4,
                VlqTooBig {
                    position: self.position.unwrap_or(0)
                }
            );
            current_byte = self.read_or_die()?;
            retval.push(current_byte);
            byte_count += 1;
        }
        Ok(retval)
    }

    pub(crate) fn read_vlq_u32(&mut self) -> ByteResult<u32> {
        let bytes = self.read_vlq_bytes()?;
        let decoded = decode_slice(&bytes).context(VlqDecode {
            position: self.position.unwrap_or(0),
        })?;
        trace!("decoded vlq value {} from {} bytes", decoded, bytes.len());
        Ok(decoded)
    }

    pub(crate) fn current(&self) -> Option<u8> {
        self.current
    }

    pub(crate) fn peek_or_die(&self) -> ByteResult<u8> {
        self.peek1.context(End {
            position: self.position.unwrap_or(0),
        })
    }

    /// Get the next value without advancing the iterator:
    /// ```text
    ///       peek
    ///       v   
    /// 0x00, 0x01, 0x02, 0x03
    /// ^ current
    /// ```
    pub(crate) fn peek(&self) -> Option<u8> {
        self.peek1
    }

    /// Get the value after the next value without advancing the iterator:
    /// ```text
    ///             peek2
    ///             v   
    /// 0x00, 0x01, 0x02, 0x03
    /// ^ current
    /// ```
    pub(crate) fn peek2(&self) -> Option<u8> {
        self.peek2
    }

    /// Get the value after the value after the next value without advancing the iterator:
    /// ```text
    ///                   peek3
    ///                   v   
    /// 0x00, 0x01, 0x02, 0x03
    /// ^ current
    /// ```
    pub(crate) fn peek3(&self) -> Option<u8> {
        self.peek3
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
        let actual_tag = from_utf8(&tag_bytes).context(Str {
            position: self.position.unwrap_or(0),
        })?;
        ensure!(
            expected_tag == actual_tag,
            Tag {
                expected: expected_tag,
                found: actual_tag,
                position: self.position.unwrap_or(0)
            }
        );
        Ok(())
    }

    /// Returns true if `current()` is the start of `expected_tag`.
    pub(crate) fn is_tag(&self, expected_tag: &str) -> bool {
        let mut tag_bytes = [0u8; 4];
        tag_bytes[0] = match self.current {
            None => return false,
            Some(val) => val,
        };
        tag_bytes[1] = match self.peek1 {
            None => return false,
            Some(val) => val,
        };
        tag_bytes[2] = match self.peek2 {
            None => return false,
            Some(val) => val,
        };
        tag_bytes[3] = match self.peek3 {
            None => return false,
            Some(val) => val,
        };
        let found = match from_utf8(&tag_bytes) {
            Ok(val) => val,
            Err(_) => return false,
        };
        expected_tag == found
    }

    // pub(crate) fn read_tag(&mut self) -> ByteResult<String> {
    //     let tag_bytes = self.read4()?;
    //     Ok(from_utf8(&tag_bytes)
    //         .context(Str {
    //             position: self.position,
    //         })?
    //         .to_owned())
    // }

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
        ensure!(
            expected == found,
            ReadExpect {
                expected,
                found,
                position: self.position.unwrap_or(0)
            }
        );
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
