use crate::error::{self, LibResult};
use std::convert::TryFrom;

/// To extract the channel number from a status byte. The right most (least-significant?) four bits
/// of a status byte represent the channel number.
pub(crate) const _STATUS_CHANNEL_MASK: u8 = 0b0000_1111;

/// Message type mask. The left most (most-significant?) four bits of a status byte message
/// represent the message type.
pub(crate) const _STATUS_TYPE_MASK: u8 = 0b1111_0000;

/// `0x8`: The bits that represent a `Note Off` message.
pub(crate) const _STATUS_NOTE_ON: u8 = 0b1000;

/// `0x9`: The bits that represent a `Note On (a velocity of 0 = Note Off)` message.
pub(crate) const _STATUS_NOTE_OFF: u8 = 0b1001;

/// `0xA`: The bits that represent a `Polyphonic key pressure/Aftertouch` message.
pub(crate) const _STATUS_POLY_PRESSURE: u8 = 0b1010;

/// `0xB`: The bits that represent a `Control change` message or a `Channel Mode` message. Channel
/// Mode messages are sent under the same Status Byte as the Control Change messages (BnH). They are
/// differentiated by the first data byte which will have a value from 121 to 127 for Channel Mode
/// messages.
pub(crate) const _STATUS_CONTROL_OR_CHANNEL: u8 = 0b1011;

/// `0xC`:The bits that represent a `Program change` message.
pub(crate) const _STATUS_PROGRAM: u8 = 0b1100;

/// `0xD`: The bits that represent a `Channel pressure/After touch` message.
pub(crate) const _STATUS_CHAN_PRESSUE: u8 = 0b1101;

/// `0xE`: The bits that represent a `Pitch bend change` message.
pub(crate) const _STATUS_PITCH_BEND: u8 = 0b1110;

/// `0xF`: The bits that represent a `System Message`.
pub(crate) const _STATUS_SYSTEM: u8 = 0b1111;

/// `0xFF`: File Spec: All meta-events begin with FF, then have an event type byte (which is always
/// less than 128)
pub(crate) const FILE_META_EVENT: u8 = 0b1111_1111;

/// `0xF0`: File Spec: `F0 <length> <bytes to be transmitted after F0>`
pub(crate) const FILE_SYSEX_F0: u8 = 0b1111_0000;

/// `0xF7`: File Spec: `F7 <length> <all bytes to be transmitted>`
pub(crate) const FILE_SYSEX_F7: u8 = 0b1111_0111;

/// Represents the status byte types in Table I "Summary of Status Bytes" from the MIDI
/// specification.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum StatusType {
    /// `0x8`: a `Note Off` message.
    NoteOff = _STATUS_NOTE_OFF,

    /// `0x9`: a `Note On (a velocity of 0 = Note Off)` message.
    NoteOn = _STATUS_NOTE_ON,

    /// `0xA`: a `Polyphonic key pressure/Aftertouch` message.
    PolyPressure = _STATUS_POLY_PRESSURE,

    /// `0xB`: a `Control change` message or a `Channel Mode` message. Channel Mode messages are
    /// sent under the same Status Byte as the Control Change messages (BnH). They are
    /// differentiated by the first data byte which will have a value from 121 to 127 for Channel
    /// Mode messages.
    ControlOrSelectChannelMode = _STATUS_CONTROL_OR_CHANNEL,

    /// `0xC`: a `Program change` message.
    Program = _STATUS_PROGRAM,

    /// `0xD`: a `Channel pressure/After touch` message.
    ChannelPressure = _STATUS_CHAN_PRESSUE,

    /// `0xE`: a `Pitch bend change` message.
    PitchBend = _STATUS_PITCH_BEND,

    /// `0xF`: a `System Message`.
    System = _STATUS_SYSTEM,
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
            _ => error::Other { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u8> for StatusType {
    type Error = crate::Error;

    fn try_from(value: u8) -> crate::Result<Self> {
        Ok(StatusType::from_u8(value)?)
    }
}
