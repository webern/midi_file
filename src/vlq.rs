use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub(crate) struct Vlq {
    bytes: u32,
}

impl Vlq {
    pub(crate) fn new(value: u32) -> Self {
        Self { bytes: value }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        encode_u32(self.bytes)
    }
}

impl TryFrom<u64> for Vlq {
    type Error = VlqError;

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        Ok(u32::try_from(value).map_err(|_| VlqError::Overflow)?.into())
    }
}

impl From<u32> for Vlq {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<u16> for Vlq {
    fn from(value: u16) -> Self {
        Self::new(value.into())
    }
}

impl From<u8> for Vlq {
    fn from(value: u8) -> Self {
        Self::new(value.into())
    }
}

impl Into<u64> for Vlq {
    fn into(self) -> u64 {
        self.bytes.into()
    }
}

impl Into<u32> for Vlq {
    fn into(self) -> u32 {
        self.bytes
    }
}

impl TryInto<u16> for Vlq {
    type Error = VlqError;

    fn try_into(self) -> Result<u16, Self::Error> {
        u16::try_from(self.bytes).map_err(|_| VlqError::Overflow)
    }
}

impl TryInto<u8> for Vlq {
    type Error = VlqError;

    fn try_into(self) -> Result<u8, Self::Error> {
        u8::try_from(self.bytes).map_err(|_| VlqError::Overflow)
    }
}

#[derive(Debug, PartialEq)]
pub enum VlqError {
    IncompleteNumber,
    Overflow,
    UnexpectedMultipleVlqs,
}

impl Display for VlqError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for VlqError {}

/// 0x7f, 127: The largest 7 bit number.
const MAX_7BIT: u8 = 0b0111_1111;

/// 0x80, 128: The highest bit is set, this bit indicates the last byte of a sequence.
pub(crate) const CONTINUE: u8 = 0b1000_0000;

/// Convert a list of numbers to a stream of bytes encoded with variable length encoding.
pub fn to_bytes(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|x| encode_u32(*x)).collect()
}

fn encode_u32(mut value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let mut result = Vec::new();
    while value > 0 {
        // get the value of the right-most seven bits
        let mut v = (value & MAX_7BIT as u32) as u8;

        // set MSB
        if !result.is_empty() {
            // ? why xor?
            // does this always flip whatever bit was there?
            // this has something to do with us later reversing the bytes
            v ^= 0x80;
        }

        result.push(v);
        // ? why 7 and not 8?
        value >>= 7;
    }
    result.reverse();
    result
}

/// Given a stream of bytes, extract all numbers which are encoded in there.
pub fn from_bytes(bytes: &[u8]) -> std::result::Result<Vec<u32>, VlqError> {
    let mut current = Vec::new();
    let mut result = Vec::new();
    for b in bytes {
        current.push(*b);
        if b & 0x80 == 0x00 {
            result.push(decode_slice(&current)?);
            current.clear();
        }
    }
    if result.is_empty() {
        return Err(VlqError::IncompleteNumber);
    }
    Ok(result)
}

pub(crate) fn decode_slice(bytes: &[u8]) -> std::result::Result<u32, VlqError> {
    let mut result: u32 = 0;

    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            if (result.rotate_left(7)) & 0x7F > 0 {
                return Err(VlqError::Overflow);
            }
            result <<= 7;
        }
        result ^= (b & 0x7F) as u32; // mask out MSB
    }

    Ok(result)
}

#[test]
fn to_single_byte() {
    assert_eq!(&[0x00], to_bytes(&[0x00]).as_slice());
    assert_eq!(&[0x40], to_bytes(&[0x40]).as_slice());
    assert_eq!(&[0x7f], to_bytes(&[0x7f]).as_slice());
}

#[test]
fn to_double_byte() {
    assert_eq!(&[0x81, 0x00], to_bytes(&[0x80]).as_slice());
    assert_eq!(&[0xc0, 0x00], to_bytes(&[0x2000]).as_slice());
    assert_eq!(&[0xff, 0x7f], to_bytes(&[0x3fff]).as_slice());
}

#[test]
fn to_triple_byte() {
    assert_eq!(&[0x81, 0x80, 0x00], to_bytes(&[0x4000]).as_slice());
    assert_eq!(&[0xc0, 0x80, 0x00], to_bytes(&[0x10_0000]).as_slice());
    assert_eq!(&[0xff, 0xff, 0x7f], to_bytes(&[0x1f_ffff]).as_slice());
}

#[test]
fn to_quadruple_byte() {
    assert_eq!(&[0x81, 0x80, 0x80, 0x00], to_bytes(&[0x20_0000]).as_slice());
    assert_eq!(
        &[0xc0, 0x80, 0x80, 0x00],
        to_bytes(&[0x0800_0000]).as_slice()
    );
    assert_eq!(
        &[0xff, 0xff, 0xff, 0x7f],
        to_bytes(&[0x0fff_ffff]).as_slice()
    );
}

#[test]
fn to_quintuple_byte() {
    assert_eq!(
        &[0x81, 0x80, 0x80, 0x80, 0x00],
        to_bytes(&[0x1000_0000]).as_slice()
    );
    assert_eq!(
        &[0x8f, 0xf8, 0x80, 0x80, 0x00],
        to_bytes(&[0xff00_0000]).as_slice()
    );
    assert_eq!(
        &[0x8f, 0xff, 0xff, 0xff, 0x7f],
        to_bytes(&[0xffff_ffff]).as_slice()
    );
}

#[test]
fn from_bytes_test() {
    assert_eq!(Ok(vec![0x7f]), from_bytes(&[0x7f]));
    assert_eq!(Ok(vec![0x2000]), from_bytes(&[0xc0, 0x00]));
    assert_eq!(Ok(vec![0x1f_ffff]), from_bytes(&[0xff, 0xff, 0x7f]));
    assert_eq!(Ok(vec![0x20_0000]), from_bytes(&[0x81, 0x80, 0x80, 0x00]));
    assert_eq!(
        Ok(vec![0xffff_ffff]),
        from_bytes(&[0x8f, 0xff, 0xff, 0xff, 0x7f])
    );
}

#[test]
fn to_bytes_multiple_values() {
    assert_eq!(&[0x40, 0x7f], to_bytes(&[0x40, 0x7f]).as_slice());
    assert_eq!(
        &[0x81, 0x80, 0x00, 0xc8, 0xe8, 0x56],
        to_bytes(&[0x4000, 0x12_3456]).as_slice()
    );
    assert_eq!(
        &[
            0xc0, 0x00, 0xc8, 0xe8, 0x56, 0xff, 0xff, 0xff, 0x7f, 0x00, 0xff, 0x7f, 0x81, 0x80,
            0x00,
        ],
        to_bytes(&[0x2000, 0x12_3456, 0x0fff_ffff, 0x00, 0x3fff, 0x4000]).as_slice()
    );
}

#[test]
fn from_bytes_multiple_values() {
    assert_eq!(
        Ok(vec![0x2000, 0x12_3456, 0x0fff_ffff, 0x00, 0x3fff, 0x4000]),
        from_bytes(&[
            0xc0, 0x00, 0xc8, 0xe8, 0x56, 0xff, 0xff, 0xff, 0x7f, 0x00, 0xff, 0x7f, 0x81, 0x80,
            0x00,
        ])
    );
}

#[test]
fn incomplete_byte_sequence() {
    assert_eq!(Err(VlqError::IncompleteNumber), from_bytes(&[0xff]));
}

#[test]
fn zero_incomplete_byte_sequence() {
    assert_eq!(Err(VlqError::IncompleteNumber), from_bytes(&[0x80]));
}

#[test]
fn overflow_u32() {
    assert_eq!(
        Err(VlqError::Overflow),
        from_bytes(&[0xff, 0xff, 0xff, 0xff, 0x7f])
    );
}

#[test]
fn chained_execution_is_identity() {
    let test = &[0xf2, 0xf6, 0x96, 0x9c, 0x3b, 0x39, 0x2e, 0x30, 0xb3, 0x24];
    assert_eq!(Ok(Vec::from(&test[..])), from_bytes(&to_bytes(test)));
}

#[test]
fn im_stupid_right_7() {
    let somebits: u32 = 0b1111_0000_1111_0000_1111_0000_1111_0000;
    let expected: u32 = 0b0000_0001_1110_0001_1110_0001_1110_0001;
    let actual = somebits >> 7;
    assert_eq!(expected, actual);
}

#[test]
fn im_stupid_left_7() {
    let somebits: u32 = 0b1111_0000_1111_0000_1111_0000_1111_0000;
    let expected: u32 = 0b0111_1000_0111_1000_0111_1000_0000_0000;
    let actual = somebits << 7;
    assert_eq!(expected, actual);
}
