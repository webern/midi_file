//! The `file` module is for types and concepts strictly related to MIDI *files*.
//! These are separated from types and concepts that are also used in realtime MIDI (`core`).

mod division;
mod event;
mod header;
mod meta_event;
mod sysex;
mod track;

pub use division::Division;
pub use event::{Event, TrackEvent};
pub use header::{Format, Header};
pub use meta_event::{MetaEvent, MicrosecondsPerQuarter, QuartersPerMinute, TimeSignatureValue};
pub use sysex::{SysexEvent, SysexEventType};
pub use track::Track;

pub(crate) use track::ensure_end_of_track;
