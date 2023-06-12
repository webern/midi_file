/*!
The `core` module is for types and concepts that are *not* strictly related to MIDI *files*.
These types and concepts could be used for realtime MIDI as well.
!*/

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
    Control, LocalControlValue, Message, MonoModeOnValue, NoteMessage, PitchBendMessage,
    ProgramChangeValue,
};
pub use numbers::{
    Channel, ControlValue, MonoModeChannels, NoteNumber, PitchBendValue, PortValue, Program,
    Velocity,
};
pub use status_type::StatusType;
