#![allow(dead_code)]

#[macro_use]
mod error;
#[macro_use]
mod clamp;
#[macro_use]
mod macros;

use crate::byte_iter::ByteIter;
use std::convert::TryFrom;
use std::io::Read;
use std::path::Path;

mod byte_iter;
pub mod channel;

pub mod constants;
pub mod message;
pub mod vlq;

use crate::constants::{FILE_META_EVENT, FILE_SYSEX_F0, FILE_SYSEX_F7};
use crate::error::LibResult;
use crate::message::Message;
use crate::MetaEvent::EndOfTrack;
pub use error::{Error, Result};
use log::{debug, trace};
use snafu::{OptionExt, ResultExt};

// https://www.music.mcgill.ca/~gary/306/week9/smf.html
// https://github.com/Shkyrockett/midi-unit-test-cases

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct MidiFile {
    header: Header,
    tracks: Vec<Track>,
}

impl MidiFile {
    fn read_inner<R: Read>(mut iter: ByteIter<R>) -> LibResult<Self> {
        trace!("parsing header chunk");
        iter.expect_tag("MThd")
            .context(error::Io { site: site!() })?;
        let chunk_length = iter.read_u32().context(error::Io { site: site!() })?;
        // header chunk length is always 6
        if chunk_length != 6 {
            return error::Other { site: site!() }.fail();
        }
        let format_word = iter.read_u16().context(error::Io { site: site!() })?;
        let num_tracks = iter.read_u16().context(error::Io { site: site!() })?;
        let division_data = iter.read_u16().context(error::Io { site: site!() })?;
        let format = Format::from_u16(format_word)?;
        let header = Header {
            format,
            division: Division::from_u16(division_data)?,
        };
        let mut tracks = Vec::new();
        for i in 0..num_tracks {
            trace!("parsing track chunk {} (zero-based) of {}", i, num_tracks);
            tracks.push(Track::parse(&mut iter)?)
        }
        Ok(Self { header, tracks })
    }

    pub fn read<R: Read>(r: R) -> Result<Self> {
        let bytes = r.bytes();
        let iter = ByteIter::new(bytes).context(error::Io { site: site!() })?;
        Ok(Self::read_inner(iter)?)
    }

    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self> {
        Ok(Self::read_inner(
            ByteIter::new_file(file).context(error::Io { site: site!() })?,
        )?)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn tracks_len(&self) -> usize {
        self.tracks.len()
    }

    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    pub fn track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }
}

// fn expect_tag<R: Read>(i: &mut ByteIter<R>, tag: &str) -> Result<()> {
//     let expected = tag.as_bytes();
//     if expected.len() != 4 {
//         return Err(Error::Badness);
//     }
//     let mut tag: [u8; 4] = [0; 4];
//     r.read_exact(&mut tag).map_err(|_| Error::Badness)?;
//     for i in 0..4usize {
//         if tag[i] != expected[i] {
//             return Err(Error::Badness);
//         }
//     }
//     Ok(())
// }

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Header {
    format: Format,
    division: Division,
}

impl Header {
    pub fn format(&self) -> &Format {
        &self.format
    }

    pub fn division(&self) -> &Division {
        &self.division
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Format {
    /// 0 the file contains a single multi-channel track
    Single = 0,
    /// 1 the file contains one or more simultaneous tracks (or MIDI outputs) of a sequence
    Multi = 1,
    /// 2 the file contains one or more sequentially independent single-track patterns
    Sequential = 2,
}

impl Default for Format {
    fn default() -> Self {
        Format::Multi
    }
}

impl Format {
    pub(crate) fn from_u16(value: u16) -> LibResult<Self> {
        match value {
            0 => Ok(Format::Single),
            1 => Ok(Format::Multi),
            2 => Ok(Format::Sequential),
            _ => error::Other { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u16> for Format {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(Self::from_u16(value)?)
    }
}

/// <division>, specifies the meaning of the delta-times. It has two formats, one for metrical time,
/// and one for time-code-based time:
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Division {
    /// If bit 15 of <division> is a zero, the bits 14 thru 0 represent the number of delta-time
    /// "ticks" which make up a quarter-note. For instance, if <division> is 96, then a time
    /// interval of an eighth-note between two events in the file would be 48.
    QuarterNote(u16), // TODO - clamp this to the allowable range
    /// Frame rate and resolution within the frame.
    Smpte(SmpteRate),
}

impl Default for Division {
    fn default() -> Self {
        Division::QuarterNote(1024)
    }
}

const DIVISION_TYPE_BIT: u16 = 0b1000000000000000;

impl Division {
    pub(crate) fn from_u16(value: u16) -> LibResult<Self> {
        if value & DIVISION_TYPE_BIT == DIVISION_TYPE_BIT {
            // TODO - implement SMPTE division
            error::Other { site: site!() }.fail()
        } else {
            Ok(Division::QuarterNote(value))
        }
    }
}

impl TryFrom<u16> for Division {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(Division::from_u16(value)?)
    }
}

/// <division> Bits 14 thru 8 contain one of the four values -24, -25, -29, or -30, corresponding to
/// the four standard SMPTE and MIDI time code formats (-29 corresponds to 30 drop frame), and
/// represents the number of frames per second. These negative numbers are stored in two's
/// complement form.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum FrameRate {
    /// 24 frames per second
    N24,
    /// 25 frames per second
    N25,
    /// 30 drop
    N29,
    /// 30 frames per second
    N30,
}

impl Default for FrameRate {
    fn default() -> Self {
        FrameRate::N24
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SmpteRate {
    /// The number of frames per second.
    frame_rate: FrameRate,
    /// The <division> second byte (stored positive) is the resolution within a frame: typical
    /// values may be 4 (MIDI time code resolution), 8, 10, 80 (bit resolution), or 100. This system
    /// allows exact specification of time-code-based tracks, but also allows millisecond-based
    /// tracks by specifying 25 frames/sec and a resolution of 40 units per frame. If the events in
    /// a file are stored with bit resolution of thirty-frame time code, the division word would be
    /// E250 hex.
    resolution: u8,
}

impl Default for SmpteRate {
    fn default() -> Self {
        // This is the 'millisecond-based tracks' example given by the spec.
        SmpteRate {
            frame_rate: FrameRate::N25,
            resolution: 40,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Track {}

impl Track {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        iter.expect_tag("MTrk")
            .context(error::Io { site: site!() })?;
        let chunk_length = iter.read_u32().context(error::Io { site: site!() })?;
        iter.set_size_limit(chunk_length as u64);
        let mut events = Vec::new();
        loop {
            if iter.is_end() {
                invalid_file!("end of track bytes reached before EndOfTrack event.");
            }
            let event = TrackEvent::parse(iter)?;
            trace!("parsed {:?}", event);
            let is_track_end = event.is_end();
            events.push(event);
            if is_track_end {
                debug!("end of track event");
                if !iter.is_end() {
                    invalid_file!("EndOfTrack event before end of track bytes.");
                }
                break;
            }
        }
        iter.clear_size_limit();
        Ok(Self {})
    }
}

/// <MTrk event> = <delta-time> <event>
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TrackEvent {
    // TODO - what is the actual maximum size of this value?
    // http://www.ccarh.org/courses/253/handout/vlv/
    /// <delta-time> is stored as a variable-length quantity. It represents the amount of time
    /// before the following event. If the first event in a track occurs at the very beginning of a
    /// track, or if two events occur simultaneously, a delta-time of zero is used. Delta-times are
    /// always present. (Not storing delta-times of 0 requires at least two bytes for any other
    /// value, and most delta-times aren't zero.) Delta-time is in ticks as specified in the header
    /// chunk.
    delta_time: u32,
    event: Event,
}

impl TrackEvent {
    fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        let delta_time = iter.read_vlq_u32().context(error::Io { site: site!() })?;
        trace!("delta_time {}", delta_time);
        let event = Event::parse(iter)?;
        Ok(Self { delta_time, event })
    }

    /// Returns true if the track event is a [`MetaEvent::EndOfTrack`].
    pub(crate) fn is_end(&self) -> bool {
        matches!(&self.event, Event::Meta(meta) if matches!(meta, MetaEvent::EndOfTrack))
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
        let status_byte = iter.peek_or_die().context(error::Io { site: site!() })?;
        // let status_val = status_byte >> 4;
        match status_byte {
            FILE_SYSEX_F7 | FILE_SYSEX_F0 => unimplemented!(),
            FILE_META_EVENT => {
                // meta events start with 0xff. we have already seen the first f, but we need to
                // read the next f and verify before parsing the MetaEvent.
                // r.read_exact(&mut one_byte).map_err(|_| Error::Io)?;
                // if one_byte[0] != FILE_META_EVENT {
                //     return Err(Error::Badness);
                // }
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
}

// /// MIDI communication is achieved through multi-byte "messages" consisting of one Status byte
// /// followed by one or two Data bytes. Real-Time and Exclusive messages are exception. A MIDI event
// /// is transmitted as a "message" and consists of one or more bytes.
// /// `Byte = Status Byte (80H - FFH) | Data Byte (00H - 7FH)`
// ///
// #[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
// pub struct MidiEvent {}

// impl MidiEvent {
//     fn parse<R: Read>(_first_byte: u8, _r: &mut R) -> Result<Self> {
//         unimplemented!()
//     }
// }

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SysexEvent {
    t: SysexEventType,
    data: Vec<u8>,
}

impl SysexEvent {
    fn parse<R: Read>(_first_byte: u8, _r: &mut R) -> LibResult<Self> {
        unimplemented!()
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum SysexEventType {
    F0 = 0xf0,
    F7 = 0xf7,
}

impl Default for SysexEventType {
    fn default() -> Self {
        SysexEventType::F0
    }
}

/// Meta Events seem to only exist in the MIDI File Spec. Here is what it says about them:
/// A few meta-events are defined herein. It is not required for every program to support every meta-event.
///
/// In the syntax descriptions for each of the meta-events a set of conventions is used to describe parameters of the
/// events. The FF which begins each event, the type of each event, and the lengths of events which do not have a
/// variable amount of data are given directly in hexadecimal. A notation such as dd or se, which consists of two
/// lower-case letters, mnemonically represents an 8-bit value. Four identical lower-case letters such as wwww refer to
/// a 16-bit value, stored most-significant-byte first. Six identical lower-case letters such as tttttt refer to a
/// 24-bit value, stored most-significant-byte first. The notation len refers to the length portion of the meta-event
/// syntax, that is, a number, stored as a variable-length quantity, which specifies how many data bytes follow it in
/// the meta-event. The notations text and data refer to however many bytes of (possibly text) data were just specified
/// by the length.
///
/// In general, meta-events in a track which occur at the same time may occur in any order. If a copyright event is
/// used, it should be placed as early as possible in the file, so it will be noticed easily. Sequence Number and
/// Sequence/Track Name events, if present, must appear at time 0. An end-of-track event must occur as the last event in
/// the track.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum MetaEvent {
    /// `FF 00 02 ssss`: This optional event, which must occur at the beginning of a track, before any nonzero delta-
    /// times, and before any transmittable MIDI events, specifies the number of a sequence. In a format 2 MIDI file, it
    /// is used to identify each "pattern" so that a "song" sequence using the Cue message to refer to the patterns. If
    /// the ID numbers are omitted, the sequences' locations in order in the file are used as defaults. In a format 0 or
    /// 1 MIDI file, which only contain one sequence, this number should be contained in the first (or only) track. If
    /// transfer of several multitrack sequences is required, this must be done as a group of format 1 files, each with
    /// a different sequence number.
    SequenceNumber,

    /// `FF 01 len text`: Any amount of text describing anything. It is a good idea to put a text event right at the
    /// beginning of a track, with the name of the track, a description of its intended orchestration, and any other
    /// information which the user wants to put there. Text events may also occur at other times in a track, to be used
    /// as lyrics, or descriptions of cue points. The text in this event should be printable ASCII characters for
    /// maximum interchange. However, other character codes using the high-order bit may be used for interchange of
    /// files between different programs on the same computer which supports an extended character set. Programs on a
    /// computer which does not support non-ASCII characters should ignore those characters.
    ///
    /// Meta event types 01 through 0F are reserved for various types of text events, each of which meets the
    /// specification of text events(above) but is used for a different purpose:
    Text(String),

    /// `FF 02 len text`: Contains a copyright notice as printable ASCII text. The notice should contain the characters
    /// (C), the year of the copyright, and the owner of the copyright. If several pieces of music are in the same MIDI
    /// file, all of the copyright notices should be placed together in this event so that it will be at the beginning
    /// of the file. This event should be the first event in the first track chunk, at time 0.
    Copyright(String),

    /// `FF 03 len text`: If in a format 0 track, or the first track in a format 1 file, the name of the sequence.
    /// Otherwise, the name of the track.
    TrackName(String),

    /// `FF 04 len text`: A description of the type of instrumentation to be used in that track. May be used with the
    /// MIDI Prefix meta-event to specify which MIDI channel the description applies to, or the channel may be specified
    /// as text in the event itself.
    InstrumentName(String),

    /// `FF 05 len text`: A lyric to be sung. Generally, each syllable will be a separate lyric event which begins at
    /// the event's time.
    Lyric(String),

    /// `FF 06 len text`: Normally in a format 0 track, or the first track in a format 1 file. The name of that point in
    /// the sequence, such as a rehearsal letter or section name ("First Verse", etc.).
    Marker(String),

    /// `FF 07 len text`: A description of something happening on a film or video screen or stage at that point in the
    /// musical score ("Car crashes into house", "curtain opens", "she slaps his face", etc.)
    CuePoint(String),

    /// `FF 08 length text`: Weird, I found it here http://www.somascape.org/midi/tech/mfile.html
    ProgramName(String),

    /// `FF 09 length text`: Weird, I found it here http://www.somascape.org/midi/tech/mfile.html
    DeviceName(String),

    /// `FF 20 01 cc`: The MIDI channel (0-15) contained in this event may be used to associate a MIDI channel with all
    /// events which follow, including System Exclusive and meta-events. This channel is "effective" until the next
    /// normal MIDI event (which contains a channel) or the next MIDI Channel Prefix meta-event. If MIDI channels refer
    /// to "tracks", this message may help jam several tracks into a format 0 file, keeping their non-MIDI data
    /// associated with a track. This capability is also present in Yamaha's ESEQ file format.
    MidiChannelPrefix,

    /// `FF 2F 00`: This event is not optional. It is included so that an exact ending point may be specified for the
    /// track, so that it has an exact length, which is necessary for tracks which are looped or concatenated.
    EndOfTrack,

    /// `FF 51 03 tttttt`: Set Tempo, in microseconds per MIDI quarter-note. This event indicates a tempo change.
    /// Another way of putting "microseconds per quarter-note" is "24ths of a microsecond per MIDI clock". Representing
    /// tempos as time per beat instead of beat per time allows absolutely exact long-term synchronization with a time-
    /// based sync protocol such as SMPTE time code or MIDI time code. This amount of accuracy provided by this tempo
    /// resolution allows a four-minute piece at 120 beats per minute to be accurate within 500 usec at the end of the
    /// piece. Ideally, these events should only occur where MIDI clocks would be located — this convention is intended
    /// to guarantee, or at least increase the likelihood, of compatibility with other synchronization devices so that a
    /// time signature/tempo map stored in this format may easily be transferred to another device.
    SetTempo(MicrosecondsPerQuarter),

    /// `FF 54 05 hr mn se fr ff`: This event, if present, designates the SMPTE time at which the track chunk is
    /// supposed to start. It should be present at the beginning of the track, that is, before any nonzero delta-times,
    /// and before any transmittable MIDI events. The hour must be encoded with the SMPTE format, just as it is in MIDI
    /// Time Code. In a format 1 file, the SMPTE Offset must be stored with the tempo map, and has no meaning in any of
    /// the other tracks. The ff field contains fractional frames, in 100ths of a frame, even in SMPTE- based tracks
    /// which specify a different frame subdivision for delta-times.
    SmpteOffset(SmpteOffsetValue),

    /// `FF 58 04 nn dd cc bb`: The time signature is expressed as four numbers. nn and dd represent the numerator and
    /// denominator of the time signature as it would be notated. The denominator is a negative power of two: 2
    /// represents a quarter-note, 3 represents an eighth-note, etc. The cc parameter expresses the number of MIDI
    /// clocks in a metronome click. The bb parameter expresses the number of notated 32nd-notes in what MIDI thinks of
    /// as a quarter-note (24 MIDI Clocks). This was added because there are already multiple programs which allow the
    /// user to specify that what MIDI thinks of as a quarter-note (24 clocks) is to be notated as, or related to in
    /// terms of, something else.
    ///
    /// Therefore, the complete event for 6/8 time, where the metronome clicks every three eighth-notes, but there are
    /// 24 clocks per quarter-note, 72 to the bar, would be (in hex): `FF 58 04 06 03 24 08`. That is, 6/8 time (8 is 2
    /// to the 3rd power, so this is 06 03), 36 MIDI clocks per dotted- quarter (24 hex!), and eight notated 32nd-notes
    /// per MIDI quarter note.
    TimeSignature(TimeSignatureValue),

    /// `FF 59 02 sf mi`:
    /// ```text
    /// sf = -7: 7 flats
    /// sf = -1: 1 flat sf=0: keyofC
    /// sf =  1: 1 sharp
    /// sf =  7: 7 sharps
    /// -----------------
    /// mi = 0: major key
    /// mi = 1: minor key
    /// ```
    KeySignature(KeySignatureValue),

    /// `FF 7f len data`: Special requirements for particular sequencers may use this event type: the first byte or
    /// bytes of data is a manufacturer ID (these are one byte, or, if the first byte is 00, three bytes). As with MIDI
    /// System Exclusive, manufacturers who define something using this meta-event should publish it so that others may
    /// know how to use it. After all, this is an interchange format. This type of event may be used by a sequencer
    /// which elects to use this as its only file format; sequencers with their established feature-specific formats
    /// should probably stick to the standard features when using this format.
    Sequencer,
}

impl Default for MetaEvent {
    fn default() -> Self {
        EndOfTrack
    }
}

impl MetaEvent {
    fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        iter.read_expect(0xff)
            .context(error::Io { site: site!() })?;
        let meta_type_byte = iter.read_or_die().context(error::Io { site: site!() })?;
        match meta_type_byte {
            0x01..=0x09 => MetaEvent::parse_text(iter),
            0x20 => panic!("{:?}", MetaEvent::MidiChannelPrefix),
            0x2f => Ok(MetaEvent::parse_end_of_track(iter)?),
            0x51 => Ok(MetaEvent::SetTempo(MicrosecondsPerQuarter::parse(iter)?)),
            0x54 => Ok(MetaEvent::SmpteOffset(SmpteOffsetValue::parse(iter)?)),
            0x58 => Ok(MetaEvent::TimeSignature(TimeSignatureValue::parse(iter)?)),
            0x59 => Ok(MetaEvent::KeySignature(KeySignatureValue::parse(iter)?)),
            0x7f => panic!("{:?}", MetaEvent::Sequencer),
            _ => error::Other { site: site!() }.fail(),
        }
    }

    pub(crate) fn parse_end_of_track<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x2f we should see 0x00
        iter.read_expect(0x00).context(io!())?;
        Ok(MetaEvent::EndOfTrack)
    }

    pub(crate) fn parse_text<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // we should be on a type-byte with a value between 0x01 and 0x09 (the text range).
        let text_type = iter.current().context(error::Other { site: site!() })?;
        let length = iter.read_vlq_u32().context(io!())?;
        let bytes = iter.read_n(length as usize).context(io!())?;
        // the spec does not strictly specify what encoding should be used for strings
        let s = String::from_utf8_lossy(&bytes).to_string();
        match text_type {
            0x01 => Ok(MetaEvent::Text(s)),
            0x02 => Ok(MetaEvent::Copyright(s)),
            0x03 => Ok(MetaEvent::TrackName(s)),
            0x04 => Ok(MetaEvent::InstrumentName(s)),
            0x05 => Ok(MetaEvent::Lyric(s)),
            0x06 => Ok(MetaEvent::Marker(s)),
            0x07 => Ok(MetaEvent::CuePoint(s)),
            0x08 => Ok(MetaEvent::ProgramName(s)),
            0x09 => Ok(MetaEvent::DeviceName(s)),
            _ => error::Other { site: site!() }.fail(),
        }
    }
}

// TODO - create some interface for this, constrict it's values, etc.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SmpteOffsetValue {
    // TODO - these are held as raw bytes for now without caring about their meaning or actual type.
    pub(crate) hr: u8,
    pub(crate) mn: u8,
    pub(crate) se: u8,
    pub(crate) fr: u8,
    pub(crate) ff: u8,
}

impl SmpteOffsetValue {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x54 we should see 0x05
        iter.read_expect(0x05)
            .context(error::Io { site: site!() })?;
        Ok(Self {
            hr: iter.read_or_die().context(error::Io { site: site!() })?,
            mn: iter.read_or_die().context(error::Io { site: site!() })?,
            se: iter.read_or_die().context(error::Io { site: site!() })?,
            fr: iter.read_or_die().context(error::Io { site: site!() })?,
            ff: iter.read_or_die().context(error::Io { site: site!() })?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TimeSignatureValue {
    /// The upper part of a time signature. For example, in 6/8, the `numerator` is 6.
    numerator: u8,

    /// The lower part of a time signature. For example, in 6/8, the `denominator` is
    /// [`DurationName::Eighth`].
    denominator: DurationName,

    /// The spec says, "The cc parameter expresses the number of MIDI clocks in a metronome click."
    /// This tells us on which beats of the bar a metronome should click, i.e. this is unrelated to
    /// tempo.
    ///
    /// [This](http://www.somascape.org/midi/tech/mfile.html) source says, "There are 24 MIDI Clocks
    /// per quarter-note." Where is this coming from? The main MIDI Spec?
    clocks_per_click: Clocks,

    /// The number of 32nd notes per quarter. This should normally be 8. The spec apologizes for its
    /// existence: "The bb parameter expresses the number of notated 32nd-notes in what MIDI thinks
    /// of as a quarter-note (24 MIDI Clocks). This was added because there are already multiple
    /// programs which allow the user to specify that what MIDI thinks of as a quarter-note (24
    /// clocks) is to be notated as, or related to in terms of, something else."
    ///
    /// I don't understand why the spec says "(24 MIDI Clocks)" in the above description. Isn't the
    /// number of MIDI Clocks per Quarter specified by the header chunk?
    tpq: u8,
}

impl TimeSignatureValue {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x58 we should see 0x04
        iter.read_expect(0x04)
            .context(error::Io { site: site!() })?;
        Ok(Self {
            numerator: iter.read_or_die().context(error::Io { site: site!() })?,
            denominator: DurationName::from_u8(
                iter.read_or_die().context(error::Io { site: site!() })?,
            )?,
            clocks_per_click: Clocks::from_u8(
                iter.read_or_die().context(error::Io { site: site!() })?,
            ),
            tpq: iter.read_or_die().context(error::Io { site: site!() })?,
        })
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum DurationName {
    /// Whole Note / Semibreve
    Whole = 0,

    /// Half Note / Minim
    Half = 1,

    /// Quarter Note / Crotchet
    Quarter = 2,

    /// Eighth Note / Quaver
    Eighth = 3,

    /// Sixteenth note / Semiquaver
    Sixteenth = 4,

    /// Thirty-Second Note / Demisemiquaver
    D32 = 5,

    /// Sixty-Fourth Note / Hemidemisemiquaver
    D64 = 6,

    /// One-Twenty-Eighth Note / Semihemidemisemiquaver
    D128 = 7,

    /// Two-Fifty-Sixth Note / Demisemihemidemisemiquaver
    D256 = 8,

    /// Five-Twelfth Note
    D512 = 9,

    /// One Thousand, Twenty-Fourth Note
    D1024 = 10,
}

impl Default for DurationName {
    fn default() -> Self {
        DurationName::Quarter
    }
}

impl DurationName {
    pub(crate) fn from_u8(v: u8) -> LibResult<Self> {
        match v {
            v if DurationName::Whole as u8 == v => Ok(DurationName::Whole),
            v if DurationName::Half as u8 == v => Ok(DurationName::Half),
            v if DurationName::Quarter as u8 == v => Ok(DurationName::Quarter),
            v if DurationName::Eighth as u8 == v => Ok(DurationName::Eighth),
            v if DurationName::Sixteenth as u8 == v => Ok(DurationName::Sixteenth),
            v if DurationName::D32 as u8 == v => Ok(DurationName::D32),
            v if DurationName::D64 as u8 == v => Ok(DurationName::D64),
            v if DurationName::D128 as u8 == v => Ok(DurationName::D128),
            v if DurationName::D256 as u8 == v => Ok(DurationName::D256),
            v if DurationName::D512 as u8 == v => Ok(DurationName::D512),
            v if DurationName::D1024 as u8 == v => Ok(DurationName::D1024),
            _ => error::Other { site: site!() }.fail(),
        }
    }

    pub(crate) fn to_u8(&self) -> u8 {
        *self as u8
    }

    /// i.e. in 4/4, the denominator is [`DurationName::Quarter`].
    pub(crate) fn to_notated_number(&self) -> u8 {
        self.to_u8() + 2
    }
}

impl TryFrom<u8> for DurationName {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Ok(Self::from_u8(value)?)
    }
}

/// There are 24 MIDI Clocks in every quarter note. (12 MIDI Clocks in an eighth note, 6 MIDI Clocks in a 16th, etc).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Clocks {
    DottedWhole,
    Whole,
    DottedHalf,
    Half,
    DottedQuarter,
    Quarter,
    DottedEighth,
    Eighth,
    DottedSixteenth,
    Sixteenth,
    Other(u8),
}

impl Default for Clocks {
    fn default() -> Self {
        Clocks::Quarter
    }
}

impl Clocks {
    pub(crate) fn from_u8(v: u8) -> Clocks {
        match v {
            142 => Clocks::DottedWhole,
            96 => Clocks::Whole,
            72 => Clocks::DottedHalf,
            48 => Clocks::Half,
            32 => Clocks::DottedQuarter,
            24 => Clocks::Quarter,
            18 => Clocks::DottedEighth,
            12 => Clocks::Eighth,
            9 => Clocks::DottedSixteenth,
            6 => Clocks::Sixteenth,
            _ => Clocks::Other(v),
        }
    }

    pub(crate) fn to_u8(&self) -> u8 {
        match self {
            Clocks::DottedWhole => 142,
            Clocks::Whole => 96,
            Clocks::DottedHalf => 72,
            Clocks::Half => 48,
            Clocks::DottedQuarter => 32,
            Clocks::Quarter => 24,
            Clocks::DottedEighth => 18,
            Clocks::Eighth => 12,
            Clocks::DottedSixteenth => 9,
            Clocks::Sixteenth => 6,
            Clocks::Other(v) => *v,
        }
    }

    pub fn new(clocks: u8) -> Self {
        Self::from_u8(clocks)
    }

    pub fn resolve(&mut self) {
        *self = Self::from_u8(self.to_u8())
    }
}

// -7 is 7 flats, +7 is 7 sharps.
clamp!(KeyAccidentals, i8, -7, 7, 0, pub);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum KeyMode {
    Major,
    Minor,
}

impl Default for KeyMode {
    fn default() -> Self {
        KeyMode::Major
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct KeySignatureValue {
    accidentals: KeyAccidentals,
    mode: KeyMode,
}

impl KeySignatureValue {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x59 we should see 0x02
        iter.read_expect(0x02)
            .context(error::Io { site: site!() })?;
        let raw_accidentals_byte = iter.read_or_die().context(error::Io { site: site!() })?;
        let casted_accidentals = raw_accidentals_byte as i8;
        Ok(Self {
            accidentals: casted_accidentals.into(),
            mode: match iter.read_or_die().context(error::Io { site: site!() })? {
                1 => KeyMode::Minor,
                _ => KeyMode::Major,
            },
        })
    }
}

pub(crate) const DEFAULT_MICROSECONDS_PER_QUARTER: u32 = 500_000;
pub(crate) const MAX_24BIT_UINT_VALUE: u32 = 16_777_215;

// Tempo microseconds are given by a 6-byte integer, hence the weird upper-bound. Default tempo is
// 120 beats per minute, which is 500_000 microseconds per beat.
//
// examples
//
// ave_maris_stella_finale_export.midi is Q=92
// that is 1/92 => 0.010869565217391 minutes per beat
// 0.010869565217391 * 60 => 0.652173913043478 seconds per beat
// 0.652173913043478 * 1000000 => 652173.91304347803816 microseconds per beat
//
// standard tempo is Q=120
// that is 1/120 => 0.008333333333333 minutes per beat
// 0.008333333333333 * 60 => 0.5 seconds per beat
// 0.652173913043478 * 1000000 => 500000 microseconds per beat
clamp!(
    MicrosecondsPerQuarter,
    u32,
    1,
    MAX_24BIT_UINT_VALUE,
    DEFAULT_MICROSECONDS_PER_QUARTER,
    pub
);

impl MicrosecondsPerQuarter {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x51 we should see 0x03
        iter.read_expect(0x03).context(io!())?;
        let bytes = iter.read_n(3).context(io!())?;
        // bytes is a big-endian u24. fit it into a big-endian u32 then parse it
        let beu32 = [0u8, bytes[0], bytes[1], bytes[2]];
        let parsed_number = u32::from_be_bytes(beu32);
        Ok(MicrosecondsPerQuarter::new(parsed_number))
    }
}
