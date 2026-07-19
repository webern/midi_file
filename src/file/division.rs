use crate::error::{Context, Result};
use crate::scribe::Scribe;
use crate::Error;
use std::convert::TryFrom;
use std::io::Write;

clamp!(
    /// The allowable values for [`Division`] when using the quarter note method. The spec gives it
    /// bits 14 thru 0 of the division word, so it is a positive 15-bit value with the range 1 to
    /// 32,767. The default value is 1024.
    QuarterNoteDivision,
    u16,
    1,
    32767,
    1024,
    pub
);

/// Specifies the meaning of the delta-times. It has two formats, one for metrical time, and one for
/// time-code-based time:
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Division {
    /// If bit 15 of <division> is a zero, the bits 14 thru 0 represent the number of delta-time
    /// "ticks" which make up a quarter-note. For instance, if <division> is 96, then a time
    /// interval of an eighth-note between two events in the file would be 48.
    QuarterNote(QuarterNoteDivision),
    /// Frame rate and resolution within the frame.
    Smpte(SmpteRate),
}

impl Default for Division {
    fn default() -> Self {
        Division::QuarterNote(QuarterNoteDivision::default())
    }
}

const DIVISION_TYPE_BIT: u16 = 0b1000000000000000;

impl Division {
    pub(crate) fn from_u16(value: u16) -> Result<Self> {
        if value & DIVISION_TYPE_BIT == DIVISION_TYPE_BIT {
            // the high byte holds the frame rate as a negative two's complement number and the low
            // byte holds the ticks-per-frame resolution, e.g. E250 for 30 fps at 80 ticks/frame.
            let frame_rate = FrameRate::from_i8((value >> 8) as u8 as i8)?;
            let resolution = (value & 0x00FF) as u8;
            Ok(Division::Smpte(SmpteRate::new(frame_rate, resolution)))
        } else {
            // never silently alter the value: zero ticks-per-quarter is the only in-format value
            // that QuarterNoteDivision cannot represent, and it is meaningless, so error.
            ensure!(value != 0, ctx!(crate::error::ErrorType::Other));
            Ok(Division::QuarterNote(QuarterNoteDivision::new(value)))
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> Result<()> {
        match self {
            Division::QuarterNote(q) => Ok(w.write_all(&q.get().to_be_bytes()).context(wr!())?),
            Division::Smpte(s) => {
                let hi = s.frame_rate().as_i8() as u8;
                Ok(w.write_all(&[hi, s.resolution()]).context(wr!())?)
            }
        }
    }
}

impl TryFrom<u16> for Division {
    type Error = Error;

    fn try_from(value: u16) -> crate::Result<Self> {
        Division::from_u16(value)
    }
}

/// <division> Bits 14 thru 8 contain one of the four values -24, -25, -29, or -30, corresponding to
/// the four standard SMPTE and MIDI time code formats (-29 corresponds to 30 drop frame), and
/// represents the number of frames per second. These negative numbers are stored in two's
/// complement form.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub enum FrameRate {
    /// 24 frames per second
    #[default]
    N24,
    /// 25 frames per second
    N25,
    /// 30 drop
    N29,
    /// 30 frames per second
    N30,
}

impl FrameRate {
    /// The negative number that represents this frame rate in the division word.
    pub(crate) fn as_i8(self) -> i8 {
        match self {
            FrameRate::N24 => -24,
            FrameRate::N25 => -25,
            FrameRate::N29 => -29,
            FrameRate::N30 => -30,
        }
    }

    pub(crate) fn from_i8(value: i8) -> Result<Self> {
        match value {
            -24 => Ok(FrameRate::N24),
            -25 => Ok(FrameRate::N25),
            -29 => Ok(FrameRate::N29),
            -30 => Ok(FrameRate::N30),
            _ => invalid_file!("invalid SMPTE frame rate {}", value),
        }
    }
}

/// The SMPTE form of [`Division`]: a frame rate and a resolution within the frame. This allows
/// delta-times to correspond to subdivisions of a second, in a way consistent with SMPTE and MIDI
/// time code.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SmpteRate {
    /// The number of frames per second.
    frame_rate: FrameRate,
    /// The <division> second byte (stored positive) is the resolution within a frame: typical
    /// values may be 4 (MIDI time code resolution), 8, 10, 80 (bit resolution), or 100. This system
    /// allows exact specification of time-code-based tracks, but also allows millisecond-based
    /// tracks by specifying 25 frames/sec and a resolution of 40 units per frame. If the events in
    /// a file are stored with bit resolution of thirty-frame time code, the division word would be
    /// E250 hex.
    resolution: u8,
}

impl Default for SmpteRate {
    fn default() -> Self {
        // This is the 'millisecond-based tracks' example given by the spec.
        SmpteRate {
            frame_rate: FrameRate::N25,
            resolution: 40,
        }
    }
}

impl SmpteRate {
    /// Create a new `SmpteRate`.
    pub fn new(frame_rate: FrameRate, resolution: u8) -> Self {
        Self {
            frame_rate,
            resolution,
        }
    }

    /// A getter for the `frame_rate` field.
    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    /// A getter for the `resolution` field.
    pub fn resolution(&self) -> u8 {
        self.resolution
    }
}
