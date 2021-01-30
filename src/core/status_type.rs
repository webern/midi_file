use crate::error::LibResult;
use std::convert::TryFrom;

/// Represents the status byte types in Table I "Summary of Status Bytes" from the MIDI
/// specification.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum StatusType {
    /// `0x8`: a `Note Off` message.
    NoteOff = 0x8,

    /// `0x9`: a `Note On (a velocity of 0 = Note Off)` message.
    NoteOn = 0x9,

    /// `0xA`: a `Polyphonic key pressure/Aftertouch` message.
    PolyPressure = 0xA,

    /// `0xB`: a `Control change` message or a `Channel Mode` message. Channel Mode messages are
    /// sent under the same Status Byte as the Control Change messages (BnH). They are
    /// differentiated by the first data byte which will have a value from 121 to 127 for Channel
    /// Mode messages.
    ControlOrSelectChannelMode = 0xB,

    /// `0xC`: a `Program change` message.
    Program = 0xC,

    /// `0xD`: a `Channel pressure/After touch` message.
    ChannelPressure = 0xD,

    /// `0xE`: a `Pitch bend change` message.
    PitchBend = 0xE,

    /// `0xF`: a `System Message`.
    System = 0xF,
}

impl Default for StatusType {
    fn default() -> Self {
        StatusType::NoteOff
    }
}

impl StatusType {
    pub(crate) fn from_u8(value: u8) -> LibResult<Self> {
        match value {
            x if StatusType::NoteOff as u8 == x => Ok(StatusType::NoteOff),
            x if StatusType::NoteOn as u8 == x => Ok(StatusType::NoteOn),
            x if StatusType::PolyPressure as u8 == x => Ok(StatusType::PolyPressure),
            x if StatusType::ControlOrSelectChannelMode as u8 == x => {
                Ok(StatusType::ControlOrSelectChannelMode)
            }
            x if StatusType::Program as u8 == x => Ok(StatusType::Program),
            x if StatusType::ChannelPressure as u8 == x => Ok(StatusType::ChannelPressure),
            x if StatusType::PitchBend as u8 == x => Ok(StatusType::PitchBend),
            x if StatusType::System as u8 == x => Ok(StatusType::System),
            _ => invalid_file!("unrecognized status byte {:#04X}", value),
        }
    }
}

impl TryFrom<u8> for StatusType {
    type Error = crate::Error;

    fn try_from(value: u8) -> crate::Result<Self> {
        Ok(StatusType::from_u8(value)?)
    }
}
