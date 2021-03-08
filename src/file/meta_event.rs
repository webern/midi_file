use crate::byte_iter::ByteIter;
use crate::core::vlq::Vlq;
use crate::core::{Channel, Clocks, DurationName, PortValue};
use crate::error::{self, LibResult};
use crate::scribe::Scribe;
use crate::{Result, Text};
use snafu::{ensure, OptionExt, ResultExt};
use std::convert::TryFrom;
use std::io::{Read, Write};

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
    SequenceNumber, // TODO - some value here

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
    OtherText(Text),

    /// `FF 02 len text`: Contains a copyright notice as printable ASCII text. The notice should contain the characters
    /// (C), the year of the copyright, and the owner of the copyright. If several pieces of music are in the same MIDI
    /// file, all of the copyright notices should be placed together in this event so that it will be at the beginning
    /// of the file. This event should be the first event in the first track chunk, at time 0.
    Copyright(Text),

    /// `FF 03 len text`: If in a format 0 track, or the first track in a format 1 file, the name of the sequence.
    /// Otherwise, the name of the track.
    TrackName(Text),

    /// `FF 04 len text`: A description of the type of instrumentation to be used in that track. May be used with the
    /// MIDI Prefix meta-event to specify which MIDI channel the description applies to, or the channel may be specified
    /// as text in the event itself.
    InstrumentName(Text),

    /// `FF 05 len text`: A lyric to be sung. Generally, each syllable will be a separate lyric event which begins at
    /// the event's time.
    Lyric(Text),

    /// `FF 06 len text`: Normally in a format 0 track, or the first track in a format 1 file. The name of that point in
    /// the sequence, such as a rehearsal letter or section name ("First Verse", etc.).
    Marker(Text),

    /// `FF 07 len text`: A description of something happening on a film or video screen or stage at that point in the
    /// musical score ("Car crashes into house", "curtain opens", "she slaps his face", etc.)
    CuePoint(Text),

    /// `FF 08 length text`: Weird, I found it here http://www.somascape.org/midi/tech/mfile.html
    ProgramName(Text),

    /// `FF 09 length text`: Weird, I found it here http://www.somascape.org/midi/tech/mfile.html
    DeviceName(Text),

    /// `FF 20 01 cc`: The MIDI channel (0-15) contained in this event may be used to associate a MIDI channel with all
    /// events which follow, including System Exclusive and meta-events. This channel is "effective" until the next
    /// normal MIDI event (which contains a channel) or the next MIDI Channel Prefix meta-event. If MIDI channels refer
    /// to "tracks", this message may help jam several tracks into a format 0 file, keeping their non-MIDI data
    /// associated with a track. This capability is also present in Yamaha's ESEQ file format.
    MidiChannelPrefix(Channel),

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
    /// sf = -1: 1 flat
    /// sf=0: keyofC
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
    Sequencer, // TODO - value

    /// `FF 0x21 0x01 value`: https://mido.readthedocs.io/en/latest/meta_message_types.html
    Port(PortValue),
}

impl Default for MetaEvent {
    fn default() -> Self {
        MetaEvent::EndOfTrack
    }
}

impl MetaEvent {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        iter.read_expect(0xff).context(io!())?;
        let meta_type_byte = iter.read_or_die().context(io!())?;
        match meta_type_byte {
            META_SEQUENCE_NUM => {
                noimpl!("Sequence Number: https://github.com/webern/midi_file/issues/8")
            }
            META_TEXT..=META_DEVICE_NAME => MetaEvent::parse_text(iter),
            META_CHAN_PREFIX => {
                iter.read_expect(LEN_META_CHAN_PREFIX).context(io!())?;
                Ok(MetaEvent::MidiChannelPrefix(Channel::new(
                    iter.read_or_die().context(io!())?,
                )))
            }
            META_END_OF_TRACK => Ok(MetaEvent::parse_end_of_track(iter)?),
            META_SET_TEMPO => Ok(MetaEvent::SetTempo(MicrosecondsPerQuarter::parse(iter)?)),
            META_SMTPE_OFFSET => Ok(MetaEvent::SmpteOffset(SmpteOffsetValue::parse(iter)?)),
            META_TIME_SIG => Ok(MetaEvent::TimeSignature(TimeSignatureValue::parse(iter)?)),
            META_KEY_SIG => Ok(MetaEvent::KeySignature(KeySignatureValue::parse(iter)?)),
            META_SEQ_SPECIFIC => {
                noimpl!("Sequencer-Specific: https://github.com/webern/midi_file/issues/9")
            }
            META_PORT => Ok(MetaEvent::Port(PortValue::new({
                iter.read_expect(1).context(io!())?;
                iter.read_or_die().context(io!())?
            }))),
            _ => invalid_file!("unrecognized byte {:#04X}", meta_type_byte),
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        w.write_all(&[0xff]).context(wr!())?;
        match self {
            MetaEvent::SequenceNumber => {
                noimpl!("Sequence Number: https://github.com/webern/midi_file/issues/8")
            }
            MetaEvent::OtherText(s) => write_text(w, 0x01, s),
            MetaEvent::Copyright(s) => write_text(w, 0x02, s),
            MetaEvent::TrackName(s) => write_text(w, 0x03, s),
            MetaEvent::InstrumentName(s) => write_text(w, 0x04, s),
            MetaEvent::Lyric(s) => write_text(w, 0x05, s),
            MetaEvent::Marker(s) => write_text(w, 0x06, s),
            MetaEvent::CuePoint(s) => write_text(w, 0x07, s),
            MetaEvent::ProgramName(s) => write_text(w, 0x08, s),
            MetaEvent::DeviceName(s) => write_text(w, 0x09, s),
            MetaEvent::MidiChannelPrefix(channel) => {
                write_u8!(w, META_CHAN_PREFIX)?;
                write_u8!(w, LEN_META_CHAN_PREFIX)?;
                write_u8!(w, channel.get())
            }
            MetaEvent::EndOfTrack => {
                write_u8!(w, META_END_OF_TRACK)?;
                write_u8!(w, LEN_META_END_OF_TRACK)?;
                Ok(())
            }
            MetaEvent::SetTempo(value) => {
                // meta event type
                write_u8!(w, META_SET_TEMPO)?;
                // data length
                write_u8!(w, LEN_META_SET_TEMPO)?;
                // we are encoding a 24-bit be number, so first get it as be bytes
                let bytes = u32::to_be_bytes(value.get());
                // my ide doesn't seem to know if this is guaranteed to be len 4
                debug_assert_eq!(bytes.len(), 4);
                // skip the first byte and write the rest
                w.write_all(&bytes[1..]).context(wr!())
            }
            MetaEvent::SmpteOffset(value) => value.write(w),
            MetaEvent::TimeSignature(value) => value.write(w),
            MetaEvent::KeySignature(value) => value.write(w),
            MetaEvent::Sequencer => {
                noimpl!("Sequencer-Specific: https://github.com/webern/midi_file/issues/9")
            }
            MetaEvent::Port(value) => {
                write_u8!(w, META_PORT)?;
                write_u8!(w, 1)?;
                write_u8!(w, value.get())
            }
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
        let s: Text = bytes.into();
        match text_type {
            META_TEXT => Ok(MetaEvent::OtherText(s)),
            META_COPYRIGHT => Ok(MetaEvent::Copyright(s)),
            META_TRACK_NAME => Ok(MetaEvent::TrackName(s)),
            META_INSTR_NAME => Ok(MetaEvent::InstrumentName(s)),
            META_LYRIC => Ok(MetaEvent::Lyric(s)),
            META_MARKER => Ok(MetaEvent::Marker(s)),
            META_CUE_POINT => Ok(MetaEvent::CuePoint(s)),
            META_PROG_NAME => Ok(MetaEvent::ProgramName(s)),
            META_DEVICE_NAME => Ok(MetaEvent::DeviceName(s)),
            _ => invalid_file!("unrecognized byte {:#04X}", text_type),
        }
    }
}

fn write_text<W: Write>(w: &mut Scribe<W>, text_type: u8, text: &Text) -> LibResult<()> {
    w.write_all(&text_type.to_be_bytes()).context(wr!())?;
    let bytes = text.as_bytes();
    let size_u32 = u32::try_from(bytes.len()).context(error::StringTooLong { site: site!() })?;
    let size = Vlq::new(size_u32).to_bytes();
    w.write_all(&size).context(wr!())?;
    w.write_all(&bytes).context(wr!())?;
    Ok(())
}

// TODO - create some interface for this, constrict it's values, etc.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SmpteOffsetValue {
    // TODO - these are held as raw bytes for now without caring about their meaning or signedness.
    pub(crate) hr: u8,
    pub(crate) mn: u8,
    pub(crate) se: u8,
    pub(crate) fr: u8,
    pub(crate) ff: u8,
}

impl SmpteOffsetValue {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // after 0x54 we should see 0x05
        iter.read_expect(LEN_META_SMTPE_OFFSET).context(io!())?;
        Ok(Self {
            hr: iter.read_or_die().context(io!())?,
            mn: iter.read_or_die().context(io!())?,
            se: iter.read_or_die().context(io!())?,
            fr: iter.read_or_die().context(io!())?,
            ff: iter.read_or_die().context(io!())?,
        })
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        write_u8!(w, META_SMTPE_OFFSET)?;
        write_u8!(w, LEN_META_SMTPE_OFFSET)?;
        write_u8!(w, self.hr)?;
        write_u8!(w, self.mn)?;
        write_u8!(w, self.se)?;
        write_u8!(w, self.fr)?;
        write_u8!(w, self.ff)?;
        Ok(())
    }
}
pub(crate) const META_SEQUENCE_NUM: u8 = 0x00;
pub(crate) const META_TEXT: u8 = 0x01;
pub(crate) const META_COPYRIGHT: u8 = 0x02;
pub(crate) const META_TRACK_NAME: u8 = 0x03;
pub(crate) const META_INSTR_NAME: u8 = 0x04;
pub(crate) const META_LYRIC: u8 = 0x05;
pub(crate) const META_MARKER: u8 = 0x06;
pub(crate) const META_CUE_POINT: u8 = 0x07;
pub(crate) const META_PROG_NAME: u8 = 0x08;
pub(crate) const META_DEVICE_NAME: u8 = 0x09;
pub(crate) const META_CHAN_PREFIX: u8 = 0x20;
pub(crate) const META_END_OF_TRACK: u8 = 0x2f;
pub(crate) const META_SET_TEMPO: u8 = 0x51;
pub(crate) const META_SMTPE_OFFSET: u8 = 0x54;
pub(crate) const META_TIME_SIG: u8 = 0x58;
pub(crate) const META_KEY_SIG: u8 = 0x59;
pub(crate) const META_SEQ_SPECIFIC: u8 = 0x7f;
/// https://groups.google.com/u/2/g/comp.music.midi/c/_MIjgi-8xQQ
/// http://www.verycomputer.com/47_f2ad3c41e745127b_1.htm
pub(crate) const META_PORT: u8 = 0x21;

// #[allow(dead_code)] // TODO - implement
pub(crate) const LEN_META_CHAN_PREFIX: u8 = 1;
pub(crate) const LEN_META_END_OF_TRACK: u8 = 0;
pub(crate) const LEN_META_SET_TEMPO: u8 = 3;
pub(crate) const LEN_META_SMTPE_OFFSET: u8 = 5;
pub(crate) const LEN_META_TIME_SIG: u8 = 4;
pub(crate) const LEN_META_KEY_SIG: u8 = 2;

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
    click: Clocks,

    /// The number of 32nd notes per quarter. This should normally be 8. The spec apologizes for its
    /// existence: "The bb parameter expresses the number of notated 32nd-notes in what MIDI thinks
    /// of as a quarter-note (24 MIDI Clocks). This was added because there are already multiple
    /// programs which allow the user to specify that what MIDI thinks of as a quarter-note (24
    /// clocks) is to be notated as, or related to in terms of, something else."
    tpq: u8,
}

impl TimeSignatureValue {
    pub fn new(numerator: u8, denominator: DurationName, click: Clocks) -> Result<Self> {
        ensure!(numerator > 0, error::Other { site: site!() });
        Ok(Self {
            numerator,
            denominator,
            click,
            ..Self::default()
        })
    }

    pub fn numerator(&self) -> u8 {
        self.numerator
    }

    pub fn denominator(&self) -> DurationName {
        self.denominator
    }

    pub fn click(&self) -> Clocks {
        self.click
    }

    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        iter.read_expect(LEN_META_TIME_SIG).context(io!())?;
        Ok(Self {
            numerator: iter.read_or_die().context(io!())?,
            denominator: DurationName::from_u8(iter.read_or_die().context(io!())?)?,
            click: Clocks::from_u8(iter.read_or_die().context(io!())?),
            tpq: iter.read_or_die().context(io!())?,
        })
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        write_u8!(w, META_TIME_SIG)?;
        write_u8!(w, LEN_META_TIME_SIG)?;
        write_u8!(w, self.numerator)?;
        write_u8!(w, self.denominator as u8)?;
        write_u8!(w, self.click.to_u8())?;
        write_u8!(w, self.tpq)?;
        Ok(())
    }
}

// -7 is 7 flats, +7 is 7 sharps.
clamp!(
    /// Represents the number of flats or sharps in a key signature. For example `-2` means
    /// "2 flats". The valid range is from -7 to 7.
    KeyAccidentals,
    i8,
    -7,
    7,
    0,
    pub
);

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
        iter.read_expect(LEN_META_KEY_SIG).context(io!())?;
        let raw_accidentals_byte = iter.read_or_die().context(io!())?;
        let casted_accidentals = raw_accidentals_byte as i8;
        Ok(Self {
            accidentals: casted_accidentals.into(),
            mode: match iter.read_or_die().context(io!())? {
                1 => KeyMode::Minor,
                _ => KeyMode::Major,
            },
        })
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        write_u8!(w, META_KEY_SIG)?;
        write_u8!(w, LEN_META_KEY_SIG)?;
        write_u8!(w, self.accidentals.get() as u8)?;
        write_u8!(w, self.mode as u8)?;
        Ok(())
    }
}

pub(crate) const DEFAULT_MICROSECONDS_PER_QUARTER: u32 = 500_000;
pub(crate) const MAX_24BIT_UINT_VALUE: u32 = 16_777_215;

clamp!(
    /// In MIDI tempos are given as microseconds per quarter note. Tempo microseconds are given by a
    /// 6-byte integer, hence the weird upper-bound (16,777,215). The default tempo is 120 beats per
    /// minute, which is `500_000` microseconds per beat. The minimum value is `1` since `0`
    /// microseconds per beat would be an infinitely fast tempo.
    ///
    /// # Examples
    ///
    /// ## Quarter Note at 92 Beats per Minute
    ///
    /// - that is 1 minute ÷ 92 => 0.010869565217391 minutes per beat (mpb)
    /// - 0.010869565217391 mpb * 60 seconds per minute => 0.652173913043478 seconds per beat (spb)
    /// - 0.652173913043478 spb * 1000000 => 652173.91304347803816 microseconds per beat
    ///
    /// ## Quarter Note at 120 Beats per Minute
    ///
    /// - that is 1 minute ÷ 120 beats per minute => 0.008333333333333 minutes per beat (mpb)
    /// - 0.008333333333333 mpb * 60 seconds => 0.5 seconds per beat (spb)
    /// - 0.652173913043478 spb * 1000000 => 500000 microseconds per beat
    MicrosecondsPerQuarter,
    u32,
    1,
    MAX_24BIT_UINT_VALUE,
    DEFAULT_MICROSECONDS_PER_QUARTER,
    pub
);

impl MicrosecondsPerQuarter {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        iter.read_expect(LEN_META_SET_TEMPO).context(io!())?;
        let bytes = iter.read_n(LEN_META_SET_TEMPO as usize).context(io!())?;
        // bytes is a big-endian u24. fit it into a big-endian u32 then parse it
        let beu32 = [0u8, bytes[0], bytes[1], bytes[2]];
        let parsed_number = u32::from_be_bytes(beu32);
        Ok(MicrosecondsPerQuarter::new(parsed_number))
    }
}

clamp!(
    /// A more convenient way to specify tempo, not part of the MIDI spec. This is closer to the way
    /// we think of tempo, e.g. "120 Beats per Minute". This type is locked to quarter-notes so you
    /// will have to translate if your "beat" is not a quarter note. Any `u8` greater than zero is
    /// valid.
    QuartersPerMinute,
    u8,
    1,
    u8::MAX,
    120,
    pub
);
