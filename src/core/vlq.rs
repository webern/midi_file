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
pub(crate) enum VlqError {
    // TODO - implement incomplete number check
    IncompleteNumber,
    Overflow,
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

        // if this is the last byte, the continue bit should not be set
        if i == bytes.len() - 1 {
            if b & CONTINUE != 0 {
                return Err(VlqError::IncompleteNumber);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test(vlq_bytes: &[u8], value: u32) {
        let encoded = encode_u32(value);
        assert_eq!(vlq_bytes, &encoded);
        let decoded = decode_slice(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn one_byte() {
        test(&[0x00], 0x00);
        test(&[0x40], 0x40);
        test(&[0x7f], 0x7f);
    }

    #[test]
    fn two_bytes() {
        test(&[0x81, 0x00], 0x80);
        test(&[0xc0, 0x00], 0x2000);
        test(&[0xff, 0x7f], 0x3fff);
    }

    #[test]
    fn three_bytes() {
        test(&[0x81, 0x80, 0x00], 0x4000);
        test(&[0xc0, 0x80, 0x00], 0x10_0000);
        test(&[0xff, 0xff, 0x7f], 0x1f_ffff);
    }

    #[test]
    fn four_bytes() {
        test(&[0x81, 0x80, 0x80, 0x00], 0x20_0000);
        test(&[0xc0, 0x80, 0x80, 0x00], 0x0800_0000);
        test(&[0xff, 0xff, 0xff, 0x7f], 0x0fff_ffff);
    }

    #[test]
    fn five_bytes() {
        test(&[0x81, 0x80, 0x80, 0x80, 0x00], 0x1000_0000);
        test(&[0x8f, 0xf8, 0x80, 0x80, 0x00], 0xff00_0000);
        test(&[0x8f, 0xff, 0xff, 0xff, 0x7f], 0xffff_ffff);
    }

    fn error_test(vlq_bytes: &[u8], x: VlqError) {
        let result = decode_slice(vlq_bytes);
        let e = result.err().unwrap();
        assert_eq!(x, e);
    }

    #[test]
    fn incomplete_0xff() {
        error_test(&[0xff], VlqError::IncompleteNumber);
    }

    #[test]
    fn incomplete_0x80() {
        error_test(&[0x80], VlqError::IncompleteNumber);
    }

    #[test]
    fn overflow_u32() {
        error_test(&[0xff, 0xff, 0xff, 0xff, 0x7f], VlqError::Overflow);
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
}
