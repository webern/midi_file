/// There are 24 MIDI Clocks in every quarter note. (12 MIDI Clocks in an eighth note, 6 MIDI Clocks
/// in a 16th, etc). One example of using this enum is in the `TimeSignature`, where we can specify
/// the frequency of the metronome click.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Clocks {
    /// 142 MIDI clocks.
    DottedWhole,

    /// 96 MIDI clocks.
    Whole,

    /// 72 MIDI clocks.
    DottedHalf,

    /// 48 MIDI clocks.
    Half,

    /// 32 MIDI clocks.
    DottedQuarter,

    /// 24 MIDI clocks.
    Quarter,

    /// 18 MIDI clocks.
    DottedEighth,

    /// 12 MIDI clocks.
    Eighth,

    /// 9 MIDI clocks.
    DottedSixteenth,

    /// 6 MIDI clocks.
    Sixteenth,

    /// Any number of MIDI clocks, intended for durations not named above.
    Other(u8),
}

impl Default for Clocks {
    fn default() -> Self {
        Clocks::Quarter
    }
}

impl Clocks {
    /// Create a new `Clocks` value from a `u8`, choosing one of the named variants if possible, and
    /// falling back to `Other` if the value does not correspond to one of the named variants.
    pub(crate) fn from_u8(v: u8) -> Clocks {
        match v {
            142 => Clocks::DottedWhole,
            96 => Clocks::Whole,
            72 => Clocks::DottedHalf,
            48 => Clocks::Half,
            32 => Clocks::DottedQuarter,
            24 => Clocks::Quarter,
            18 => Clocks::DottedEighth,
            12 => Clocks::Eighth,
            9 => Clocks::DottedSixteenth,
            6 => Clocks::Sixteenth,
            _ => Clocks::Other(v),
        }
    }

    // Get the `u8` value represented by the enum.
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Clocks::DottedWhole => 142,
            Clocks::Whole => 96,
            Clocks::DottedHalf => 72,
            Clocks::Half => 48,
            Clocks::DottedQuarter => 32,
            Clocks::Quarter => 24,
            Clocks::DottedEighth => 18,
            Clocks::Eighth => 12,
            Clocks::DottedSixteenth => 9,
            Clocks::Sixteenth => 6,
            Clocks::Other(v) => v,
        }
    }

    /// Create a new `Clocks` value from a `u8`, choosing one of the named variants if possible, and
    /// falling back to `Other` if the value does not correspond to one of the named variants.
    pub fn new(clocks: u8) -> Self {
        Self::from_u8(clocks)
    }

    /// If you create a `Clocks` value with a standard value, this will resolve the `Clocks` value
    /// to a named variant instead of `Other`. For example:
    /// ```
    /// use midi_file::core::Clocks;
    /// let mut clocks = Clocks::Other(24);
    /// clocks.resolve();
    /// assert!(matches!(clocks, Clocks::Quarter));
    /// ```
    pub fn resolve(&mut self) {
        *self = Self::from_u8(self.to_u8())
    }
}
