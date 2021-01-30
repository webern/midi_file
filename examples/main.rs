use midi_file::core::{Channel, Clocks, DurationName, GeneralMidi, NoteNumber, Velocity};
use midi_file::file::{Division, Format, QuartersPerMinute, Track};
use midi_file::MidiFile;

// durations
const QUARTER: u32 = 1024;
const EIGHTH: u32 = QUARTER / 2;
const DOTTED_QUARTER: u32 = QUARTER + EIGHTH;

// pitches
const C4: NoteNumber = NoteNumber::new(72);
const D4: NoteNumber = NoteNumber::new(74);
const E4: NoteNumber = NoteNumber::new(76);

// some arbitrary velocity
const V: Velocity = Velocity::new(64);

// channel zero (displayed as channel 1 in any sequencer UI)
const CH: Channel = Channel::new(0);

fn main() {
    let mut mfile = MidiFile::new(Format::Multi, Division::default());

    // set up track metadata
    let mut track = Track::default();
    track.set_name("Singer").unwrap();
    track.set_instrument_name("Alto").unwrap();
    track.set_general_midi(CH, GeneralMidi::SynthVoice).unwrap();

    // set time signature and tempo
    track
        .push_time_signature(0, 6, DurationName::Eighth, Clocks::DottedQuarter)
        .unwrap();
    track.push_tempo(0, QuartersPerMinute::new(116)).unwrap();

    // measure 1 ///////////////////////////////////////////////////////////////////////////////////

    // create the first note
    // we don't have any rests, all of our lyric and note-on events will be at delta time zero
    track.push_lyric(0, "Row").unwrap();
    track.push_note_on(0, CH, C4, V).unwrap();
    // the note-off event determines the duration of the note
    track
        .push_note_off(DOTTED_QUARTER, CH, C4, Velocity::default())
        .unwrap();

    track.push_lyric(0, "row").unwrap();
    track.push_note_on(0, CH, C4, V).unwrap();
    track.push_note_off(DOTTED_QUARTER, CH, C4, V).unwrap();

    // measure 2 ///////////////////////////////////////////////////////////////////////////////////

    track.push_lyric(0, "row").unwrap();
    track.push_note_on(0, CH, C4, V).unwrap();
    // the note-off event determines the duration of the note
    track.push_note_off(QUARTER, CH, C4, V).unwrap();

    track.push_lyric(0, "your").unwrap();
    track.push_note_on(0, CH, D4, V).unwrap();
    track.push_note_off(EIGHTH, CH, D4, V).unwrap();

    track.push_lyric(0, "boat").unwrap();
    track.push_note_on(0, CH, E4, V).unwrap();
    // the note-off event determines the duration of the note
    track.push_note_off(DOTTED_QUARTER, CH, E4, V).unwrap();

    // measure 3, etc.

    // finish and write the file ///////////////////////////////////////////////////////////////////

    // add the track to the file
    mfile.push_track(track).unwrap();

    // write the file (can also be written to a file with mfile.save(path))
    let mut bytes = Vec::new();
    mfile.write(&mut bytes).unwrap();

    // assert the library is not broken! ///////////////////////////////////////////////////////////

    let expected: [u8; 144] = [
        // header: MThd, len 6 bytes, format 1, ntracks 1, divisions 1024
        0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x01, 0x04, 0x00,
        // track: MTrk, len 121 bytes
        0x4D, 0x54, 0x72, 0x6B, 0x00, 0x00, 0x00, 0x7A, //
        // DeltaTime: 0, ProgramChange/Channel: 0, Value: 0x37
        0x00, 0xC0, 0x37, //
        // DeltaTime: 0, InstrumentName, len 4 bytes, "Alto"
        0x00, 0xFF, 0x04, 0x04, 0x41, 0x6C, 0x74, 0x6F, //
        // DeltaTime: 0, TrackName, len 6 bytes, "Singer"
        0x00, 0xFF, 0x03, 0x06, 0x53, 0x69, 0x6E, 0x67, 0x65, 0x72, //
        // DeltaTime: 0, TimeSignature
        0x00, 0xFF, 0x58, 0x04, 0x06, 0x03, 0x20, 0x00, //
        // DeltaTime: 0, SetTempo
        0x00, 0xFF, 0x51, 0x03, 0x07, 0xE4, 0x79, //
        // DeltaTime: 0, Lyric: "Row"
        0x00, 0xFF, 0x05, 0x03, 0x52, 0x6F, 0x77, //
        // NoteOn
        0x00, 0x90, 0x48, 0x40, //
        // NoteOff
        0x8C, 0x00, 0x80, 0x48, 0x48, //
        // Lyric: "row"
        0x00, 0xFF, 0x05, 0x03, 0x72, 0x6F, 0x77, //
        // NoteOn
        0x00, 0x90, 0x48, 0x40, //
        // NoteOff
        0x8C, 0x00, 0x80, 0x48, 0x40, //
        // Lyric:  "row"
        0x00, 0xFF, 0x05, 0x03, 0x72, 0x6F, 0x77, //
        // NoteOn
        0x00, 0x90, 0x48, 0x40, //
        // NoteOff
        0x88, 0x00, 0x80, 0x48, 0x40, //
        // Lyric: "your"
        0x00, 0xFF, 0x05, 0x04, 0x79, 0x6F, 0x75, 0x72, //
        // NoteOn
        0x00, 0x90, 0x4A, 0x40, //
        // NoteOff
        0x84, 0x00, 0x80, 0x4A, 0x40, //
        // Lyric: "boat"
        0x00, 0xFF, 0x05, 0x04, 0x62, 0x6F, 0x61, 0x74, //
        // NoteOn
        0x00, 0x90, 0x4C, 0x40, //
        // NoteOff
        0x8C, 0x00, 0x80, 0x4C, 0x40, //
        // EndOfTrack marker
        0x00, 0xFF, 0x2F, 0x00,
    ];

    assert_eq!(bytes.len(), expected.len());
    for (ix, &byte) in bytes.iter().enumerate() {
        let ex = expected[ix];
        assert_eq!(
            ex, byte,
            "mismatch at byte index {}, expected {:#04X}, got {:#04X}",
            ix, ex, byte
        );
    }
}
