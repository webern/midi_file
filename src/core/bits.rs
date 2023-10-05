/*!

This module is for dealing with bit operations that were hard to figure out.

!*/

#[inline]
pub(crate) fn decode_14_bit_number(bits: u16) -> u16 {
    ((extract_high_bits(bits) as u16) << 7) | (extract_low_bits(bits) as u16)
}

#[inline]
pub(crate) fn encode_14_bit_number(value: u16) -> u16 {
    let hi_bits = value & 0b0011111110000000;
    let lo_bits = value & 0b0000000001111111;
    let lo_moved = lo_bits << 8;
    let hi_moved = hi_bits >> 7;
    lo_moved | hi_moved
}

#[inline]
fn extract_low_bits(bits: u16) -> u8 {
    ((bits >> 8) as u8) & 0b0000000001111111
}

#[inline]
fn extract_high_bits(bits: u16) -> u8 {
    (bits & 0b0000000001111111) as u8
}

#[cfg(test)]
mod bit_tests {
    use super::*;

    struct Number14Bit {
        encoded: u16,
        decoded: u16,
        lo_bits: u8,
        hi_bits: u8,
    }
    const NUMBER_14_BIT_08192: Number14Bit = Number14Bit {
        encoded: 0b0000000001000000,
        decoded: 0b0010000000000000,
        lo_bits: 0b0000000,
        hi_bits: 0b1000000,
    };

    const NUMBER_14_BIT_08292: Number14Bit = Number14Bit {
        encoded: 0b0110010001000000,
        decoded: 0b0010000001100100,
        lo_bits: 0b1100100,
        hi_bits: 0b1000000,
    };
    const NUMBER_14_BIT_08092: Number14Bit = Number14Bit {
        encoded: 0b0001110000111111,
        decoded: 0b0001111110011100,
        lo_bits: 0b0011100,
        hi_bits: 0b0111111,
    };
    const NUMBER_14_BIT_16383: Number14Bit = Number14Bit {
        encoded: 0b0111111101111111,
        decoded: 0b0011111111111111,
        lo_bits: 0b1111111,
        hi_bits: 0b1111111,
    };
    const NUMBER_14_BIT_00001: Number14Bit = Number14Bit {
        encoded: 0b0000000100000000,
        decoded: 0b0000000000000001,
        lo_bits: 0b0000001,
        hi_bits: 0b0000000,
    };

    #[test]
    fn test_14_bit_08192() {
        let data = NUMBER_14_BIT_08192;
        assert_eq!(extract_low_bits(data.encoded), data.lo_bits);
        assert_eq!(extract_high_bits(data.encoded), data.hi_bits);
        assert_eq!(decode_14_bit_number(data.encoded), data.decoded);
        assert_eq!(encode_14_bit_number(data.decoded), data.encoded);
    }

    #[test]
    fn test_14_bit_08292() {
        let data = NUMBER_14_BIT_08292;
        assert_eq!(extract_low_bits(data.encoded), data.lo_bits);
        assert_eq!(extract_high_bits(data.encoded), data.hi_bits);
        assert_eq!(decode_14_bit_number(data.encoded), data.decoded);
        assert_eq!(encode_14_bit_number(data.decoded), data.encoded);
    }

    #[test]
    fn test_14_bit_08092() {
        let data = NUMBER_14_BIT_08092;
        assert_eq!(extract_low_bits(data.encoded), data.lo_bits);
        assert_eq!(extract_high_bits(data.encoded), data.hi_bits);
        assert_eq!(decode_14_bit_number(data.encoded), data.decoded);
        assert_eq!(encode_14_bit_number(data.decoded), data.encoded);
    }
    #[test]
    fn test_14_bit_16383() {
        let data = NUMBER_14_BIT_16383;
        assert_eq!(extract_low_bits(data.encoded), data.lo_bits);
        assert_eq!(extract_high_bits(data.encoded), data.hi_bits);
        assert_eq!(decode_14_bit_number(data.encoded), data.decoded);
        assert_eq!(encode_14_bit_number(data.decoded), data.encoded);
    }

    #[test]
    fn test_14_bit_00001() {
        let data = NUMBER_14_BIT_00001;
        assert_eq!(extract_low_bits(data.encoded), data.lo_bits);
        assert_eq!(extract_high_bits(data.encoded), data.hi_bits);
        assert_eq!(decode_14_bit_number(data.encoded), data.decoded);
        assert_eq!(encode_14_bit_number(data.decoded), data.encoded);
    }

    #[test]
    fn test_14_all() {
        for i in 0..=16383u16 {
            let original = i;
            let encoded = encode_14_bit_number(original);
            let decoded = decode_14_bit_number(encoded);
            assert_eq!(original, decoded);
            if original != 0 {
                assert_ne!(
                    encoded, decoded,
                    "encoded should not equal decoded, {} == {}",
                    encoded, decoded
                );
            }
        }
    }
}
