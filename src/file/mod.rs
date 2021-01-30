/*!
The `file` module is for types and concepts strictly related to MIDI *files*.
These are kept separate from types and concepts that are also used in realtime MIDI (`core`).
!*/

mod division;
mod header;
mod sysex;
mod track;
mod track_event;

pub use division::Division;
pub use header::{Format, Header};
pub use sysex::{SysexEvent, SysexEventType};
pub(crate) use track::ensure_end_of_track;
pub use track::Track;
pub use track_event::{Event, TrackEvent};
