/*!
The `core` module is for types and concepts that are *not* strictly related to MIDI *files*.
These types and concepts could be used for realtime MIDI as well.
!*/

mod bits;
mod clocks;
mod duration_name;
mod general_midi;
mod message;
mod numbers;
mod status_type;
pub(crate) mod vlq;

pub use clocks::Clocks;
pub use duration_name::DurationName;
pub use general_midi::GeneralMidi;
pub use message::{
    ChannelPressureMessage, Control, ControlChangeValue, LocalControlValue, Message,
    MidiTimeCodeQuarterFrameMessage, MonoModeOnValue, NoteMessage, OnOff, PitchBendMessage,
    ProgramChangeValue, SongPositionPointerMessage, SongSelectMessage,
};
pub use numbers::{
    Channel, ControlValue, MonoModeChannels, NoteNumber, PitchBendValue, PortValue, PressureValue,
    Program, QuarterFrameValue, SongNumber, SongPosition, Velocity,
};
pub use status_type::StatusType;
