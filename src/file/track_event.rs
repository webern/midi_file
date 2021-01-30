use crate::byte_iter::ByteIter;
use crate::constants::{FILE_META_EVENT, FILE_SYSEX_F0, FILE_SYSEX_F7};
use crate::core::vlq::Vlq;
use crate::error::LibResult;
use crate::{Message, MetaEvent, SysexEvent};
use log::trace;
use snafu::ResultExt;
use std::io::{Read, Write};

/// <MTrk event> = <delta-time> <event>
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TrackEvent {
    /// <delta-time> is stored as a variable-length quantity. It represents the amount of time
    /// before the following event. If the first event in a track occurs at the very beginning of a
    /// track, or if two events occur simultaneously, a delta-time of zero is used. Delta-times are
    /// always present. Delta-time is in ticks as specified in the header chunk.
    delta_time: u32,
    event: Event,
}

impl TrackEvent {
    pub fn new(delta_time: u32, event: Event) -> Self {
        Self { delta_time, event }
    }

    pub fn delta_time(&self) -> u32 {
        self.delta_time
    }

    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Returns true if the track event is a [`MetaEvent::EndOfTrack`].
    pub(crate) fn is_end(&self) -> bool {
        matches!(&self.event, Event::Meta(meta) if matches!(meta, MetaEvent::EndOfTrack))
    }

    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        let delta_time = iter.read_vlq_u32().context(io!())?;
        trace!("delta_time {}", delta_time);
        let event = Event::parse(iter)?;
        Ok(Self { delta_time, event })
    }

    pub(crate) fn write<W: Write>(&self, w: &mut W) -> LibResult<()> {
        let delta = Vlq::new(self.delta_time).to_bytes();
        w.write_all(&delta).context(wr!())?;
        self.event.write(w)
    }
}

/// <event> = <MIDI event> | <sysex event> | <meta-event>
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Event {
    /// <MIDI event> is any MIDI channel message. Running status is used.
    Midi(Message),
    /// <sysex event> is used to specify a MIDI system exclusive message.
    Sysex(SysexEvent),
    /// <meta-event> specifies non-MIDI information useful to this format or to sequencers.
    Meta(MetaEvent),
}

impl Default for Event {
    fn default() -> Self {
        Event::Midi(Message::default())
    }
}

impl Event {
    fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        let status_byte = iter.peek_or_die().context(io!())?;
        match status_byte {
            FILE_SYSEX_F7 | FILE_SYSEX_F0 => {
                Ok(Event::Sysex(SysexEvent::parse(status_byte, iter)?))
            }
            FILE_META_EVENT => {
                trace!("I peeked at {:#x}, a MetaEvent!", status_byte);
                Ok(Event::Meta(MetaEvent::parse(iter)?))
            }
            _ => {
                trace!(
                    "I peeked at {:#x}, neither a SysEx nor a MetaEvent, it must be a MIDI Message!",
                    status_byte
                );
                Ok(Event::Midi(Message::parse(iter)?))
            }
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut W) -> LibResult<()> {
        match self {
            Event::Midi(md) => md.write(w),
            Event::Sysex(sx) => sx.write(w),
            Event::Meta(mt) => mt.write(w),
        }
    }
}
