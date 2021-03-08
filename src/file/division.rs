use crate::error::LibResult;
use crate::scribe::Scribe;
use crate::Error;
use snafu::ResultExt;
use std::convert::TryFrom;
use std::io::Write;

clamp!(
    /// The allowable values for [`Division`] when using the quarter note method. It is a positive
    /// `u14` and thus has the range 1 to 16,383. The default value is 1024.
    QuarterNoteDivision,
    u16,
    1,
    16383,
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
    /// Frame rate and resolution within the frame. Caution, this may not be implemented correctly.
    /// https://github.com/webern/midi_file/issues/11
    Smpte(SmpteRate),
}

impl Default for Division {
    fn default() -> Self {
        Division::QuarterNote(QuarterNoteDivision::default())
    }
}

const DIVISION_TYPE_BIT: u16 = 0b1000000000000000;

impl Division {
    pub(crate) fn from_u16(value: u16) -> LibResult<Self> {
        if value & DIVISION_TYPE_BIT == DIVISION_TYPE_BIT {
            // TODO - implement SMPTE division
            crate::error::Other { site: site!() }.fail()
        } else {
            Ok(Division::QuarterNote(QuarterNoteDivision::new(value)))
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        match self {
            Division::QuarterNote(q) => Ok(w.write_all(&q.get().to_be_bytes()).context(wr!())?),
            Division::Smpte(_) => crate::error::Other { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u16> for Division {
    type Error = Error;

    fn try_from(value: u16) -> crate::Result<Self> {
        Ok(Division::from_u16(value)?)
    }
}

/// <division> Bits 14 thru 8 contain one of the four values -24, -25, -29, or -30, corresponding to
/// the four standard SMPTE and MIDI time code formats (-29 corresponds to 30 drop frame), and
/// represents the number of frames per second. These negative numbers are stored in two's
/// complement form.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum FrameRate {
    /// 24 frames per second
    N24,
    /// 25 frames per second
    N25,
    /// 30 drop
    N29,
    /// 30 frames per second
    N30,
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::N24
    }
}

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
