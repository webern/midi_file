/*!
The `core` module is for types and concepts that are *not* strictly related to MIDI *files*.
These types and concepts could be used for realtime MIDI as well.
!*/

mod clocks;
mod duration_name;
mod general_midi;

pub use clocks::Clocks;
pub use duration_name::DurationName;
pub use general_midi::GeneralMidi;
