/*!
The `file` module is for types and concepts strictly related to MIDI *files*.
These are kept separate from types and concepts that are also used in realtime MIDI (`core`).
!*/

mod division;
mod header;
mod smpte_offset;
mod time_signature;
mod track;
mod track_event;

pub use division::Division;
pub use header::{Format, Header};
