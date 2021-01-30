/*!
The `core` module is for types and concepts that are *not* strictly related to MIDI *files*.
These types and concepts could be used for realtime MIDI as well.
!*/

mod clocks;
mod duration_name;
mod general_midi;
mod numbers;
mod status_type;
pub(crate) mod vlq;

pub use clocks::Clocks;
pub use duration_name::DurationName;
pub use general_midi::GeneralMidi;
pub use status_type::StatusType;

pub use numbers::{Channel, NoteNumber, Program, Velocity, U7};
