use crate::error::LibResult;
use crate::Error;
use std::convert::TryFrom;

/// `DurationName` is used when specifying the denominator of a [`crate::file::TimeSignatureValue`].
/// When defining time signatures, the MIDI file spec says:
/// ```text
/// The denominator is a negative power of two: 2 represents a quarter-note, 3 represents an'
/// eighth-note, etc.
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum DurationName {
    /// Whole Note / Semibreve
    Whole = 0,

    /// Half Note / Minim
    Half = 1,

    /// Quarter Note / Crotchet
    Quarter = 2,

    /// Eighth Note / Quaver
    Eighth = 3,

    /// Sixteenth note / Semiquaver
    Sixteenth = 4,

    /// Thirty-Second Note / Demisemiquaver
    D32 = 5,

    /// Sixty-Fourth Note / Hemidemisemiquaver
    D64 = 6,

    /// One-Twenty-Eighth Note / Semihemidemisemiquaver
    D128 = 7,

    /// Two-Fifty-Sixth Note / Demisemihemidemisemiquaver
    D256 = 8,

    /// Five-Twelfth Note
    D512 = 9,

    /// One Thousand Twenty-Fourth Note
    D1024 = 10,
}

impl Default for DurationName {
    fn default() -> Self {
        DurationName::Quarter
    }
}

impl DurationName {
    pub(crate) fn from_u8(v: u8) -> LibResult<Self> {
        match v {
            v if DurationName::Whole as u8 == v => Ok(DurationName::Whole),
            v if DurationName::Half as u8 == v => Ok(DurationName::Half),
            v if DurationName::Quarter as u8 == v => Ok(DurationName::Quarter),
            v if DurationName::Eighth as u8 == v => Ok(DurationName::Eighth),
            v if DurationName::Sixteenth as u8 == v => Ok(DurationName::Sixteenth),
            v if DurationName::D32 as u8 == v => Ok(DurationName::D32),
            v if DurationName::D64 as u8 == v => Ok(DurationName::D64),
            v if DurationName::D128 as u8 == v => Ok(DurationName::D128),
            v if DurationName::D256 as u8 == v => Ok(DurationName::D256),
            v if DurationName::D512 as u8 == v => Ok(DurationName::D512),
            v if DurationName::D1024 as u8 == v => Ok(DurationName::D1024),
            _ => crate::error::Other { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u8> for DurationName {
    type Error = Error;

    fn try_from(value: u8) -> crate::Result<Self> {
        Ok(Self::from_u8(value)?)
    }
}
