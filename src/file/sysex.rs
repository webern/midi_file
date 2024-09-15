use crate::byte_iter::ByteIter;
use crate::error::LibResult;
use crate::scribe::Scribe;
use std::io::{Read, Write};

// TODO - implement sysex messages
/// Caution: Sysex messages are [not implemented](https://github.com/webern/midi_file/issues/7) and
/// will error.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SysexEvent {
    t: SysexEventType,
    data: Vec<u8>,
}

impl SysexEvent {
    // TODO - implement a `new` function.
    // TODO - implement getter functions.

    pub(crate) fn parse<R: Read>(_first_byte: u8, _r: &mut ByteIter<R>) -> LibResult<Self> {
        noimpl!("SysexEvent::parse")
    }

    pub(crate) fn write<W: Write>(&self, _w: &mut Scribe<W>) -> LibResult<()> {
        noimpl!("SysexEvent::write")
    }
}

/// `<sysex event>` is used to specify a MIDI system exclusive message, either as one unit or in
/// packets, or as an "escape" to specify any arbitrary bytes to be transmitted. See Appendix 1 -
/// MIDI Messages. A normal complete system exclusive message is stored in a MIDI File in this way:
#[repr(u8)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub enum SysexEventType {
    /// F0 `<length>` `<bytes to be transmitted after F0>`
    ///
    /// The length is stored as a variable-length quantity. It specifies the number of bytes which
    /// follow it, not including the F0 or the length itself. For instance, the transmitted message
    /// `F0 43 12 00 07 F7` would be stored in a MIDI File as `F0 05 43 12 00 07 F7`. It is required
    /// to include the `F7` at the end so that the reader of the MIDI File knows that it has read
    /// the entire message.
    #[default]
    F0 = 0xf0,

    /// F7 <length> <all bytes to be transmitted>
    ///
    /// Unfortunately, some synthesiser manufacturers specify that their system exclusive messages
    /// are to be transmitted as little packets. Each packet is only part of an entire syntactical
    /// system exclusive message, but the times they are transmitted are important. Examples of this
    /// are the bytes sent in a CZ patch dump, or the FB-01's "system exclusive mode" in which
    /// microtonal data can be transmitted. The F0 and F7 sysex events may be used together to break
    /// up syntactically complete system exclusive messages into timed packets.
    ///
    /// An F0 sysex event is used for the first packet in a series -- it is a message in which the
    /// F0 should be transmitted. An F7 sysex event is used for the remainder of the packets, which
    /// do not begin with F0. (Of course, the F7 is not considered part of the system exclusive
    /// message).
    ///
    /// A syntactic system exclusive message must always end with an F7, even if the real-life
    /// device didn't send one, so that you know when you've reached the end of an entire sysex
    /// message without looking ahead to the next event in the MIDI File. If it's stored in one
    /// complete F0 sysex event, the last byte must be an F7. There also must not be any
    /// transmittable MIDI events in between the packets of a multi-packet system exclusive message.
    F7 = 0xf7,
}
