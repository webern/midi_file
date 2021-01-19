/// There are 24 MIDI Clocks in every quarter note. (12 MIDI Clocks in an eighth note, 6 MIDI Clocks in a 16th, etc).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Clocks {
    DottedWhole,
    Whole,
    DottedHalf,
    Half,
    DottedQuarter,
    Quarter,
    DottedEighth,
    Eighth,
    DottedSixteenth,
    Sixteenth,
    Other(u8),
}

impl Default for Clocks {
    fn default() -> Self {
        Clocks::Quarter
    }
}

impl Clocks {
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

    pub(crate) fn to_u8(&self) -> u8 {
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
            Clocks::Other(v) => *v,
        }
    }

    pub fn new(clocks: u8) -> Self {
        Self::from_u8(clocks)
    }

    pub fn resolve(&mut self) {
        *self = Self::from_u8(self.to_u8())
    }
}
