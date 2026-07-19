//! Regression tests for compliance with the MIDI file specification (docs/spec-midimusic.md).

use midi_file::MidiFile;

/// Builds a MIDI file as bytes: a format 0 header (division 96) followed by a single MTrk chunk
/// with the given track data.
fn file_with_track_data(track_data: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![
        0x4D, 0x54, 0x68, 0x64, // MThd
        0x00, 0x00, 0x00, 0x06, // length 6
        0x00, 0x00, // format 0
        0x00, 0x01, // one track
        0x00, 0x60, // division 96
        0x4D, 0x54, 0x72, 0x6B, // MTrk
    ];
    bytes.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(track_data);
    bytes
}

/// Builds a MIDI file as bytes: a format 0 header with the given division word followed by a
/// single empty track.
fn file_with_division(division: &[u8; 2]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![
        0x4D, 0x54, 0x68, 0x64, // MThd
        0x00, 0x00, 0x00, 0x06, // length 6
        0x00, 0x00, // format 0
        0x00, 0x01, // one track
    ];
    bytes.extend_from_slice(division);
    bytes.extend_from_slice(&[
        0x4D, 0x54, 0x72, 0x6B, // MTrk
        0x00, 0x00, 0x00, 0x04, // length 4
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ]);
    bytes
}

/// https://github.com/webern/midi_file/issues/30
/// Spec 2.3: "programs must properly ignore meta-events which they do not recognise, and indeed
/// should expect to see them." Unknown meta events are retained as raw bytes and roundtrip.
#[test]
fn unknown_meta_events_are_preserved() {
    use midi_file::file::{Event, MetaEvent};
    let bytes = file_with_track_data(&[
        0x00, 0xFF, 0x60, 0x01, 0x7F, // unknown meta type 0x60, len 1
        0x00, 0xFF, 0x0A, 0x01, 0x41, // reserved text meta type 0x0A, len 1, "A"
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ]);
    let mf = MidiFile::read(bytes.as_slice()).unwrap();
    let track = mf.tracks().next().unwrap();
    let mut events = track.events();
    match events.next().unwrap().event() {
        Event::Meta(MetaEvent::Unknown(u)) => {
            assert_eq!(0x60, u.meta_type());
            assert_eq!(&[0x7F], u.data());
        }
        e => panic!("wrong event {:?}", e),
    }
    match events.next().unwrap().event() {
        Event::Meta(MetaEvent::Unknown(u)) => {
            assert_eq!(0x0A, u.meta_type());
            assert_eq!(&[0x41], u.data());
        }
        e => panic!("wrong event {:?}", e),
    }
    let mut out: Vec<u8> = Vec::new();
    mf.write(&mut out).unwrap();
    assert_eq!(bytes, out);
}

/// https://github.com/webern/midi_file/issues/30
/// A known meta type with a length other than the one the spec assigns is preserved verbatim as
/// an unknown meta event instead of being rejected or misinterpreted.
#[test]
fn known_meta_type_with_unexpected_length_is_preserved() {
    use midi_file::file::{Event, MetaEvent};
    // a set tempo event with four data bytes instead of three
    let bytes = file_with_track_data(&[
        0x00, 0xFF, 0x51, 0x04, 0x07, 0xA1, 0x20, 0x00, //
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ]);
    let mf = MidiFile::read(bytes.as_slice()).unwrap();
    let track = mf.tracks().next().unwrap();
    match track.events().next().unwrap().event() {
        Event::Meta(MetaEvent::Unknown(u)) => assert_eq!(0x51, u.meta_type()),
        e => panic!("wrong event {:?}", e),
    }
    let mut out: Vec<u8> = Vec::new();
    mf.write(&mut out).unwrap();
    assert_eq!(bytes, out);
}

/// https://github.com/webern/midi_file/issues/32
/// Spec 1.3: "Your programs should EXPECT alien chunks and treat them as if they weren't there."
#[test]
fn alien_chunks_are_skipped() {
    let bytes: Vec<u8> = vec![
        0x4D, 0x54, 0x68, 0x64, // MThd
        0x00, 0x00, 0x00, 0x06, // length 6
        0x00, 0x00, // format 0
        0x00, 0x01, // one track
        0x00, 0x60, // division 96
        0x58, 0x46, 0x49, 0x48, // alien chunk "XFIH"
        0x00, 0x00, 0x00, 0x02, // length 2
        0x01, 0x02, // alien data
        0x4D, 0x54, 0x72, 0x6B, // MTrk
        0x00, 0x00, 0x00, 0x04, // length 4
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ];
    let mf = MidiFile::read(bytes.as_slice()).unwrap();
    assert_eq!(1, mf.tracks_len());
}

/// https://github.com/webern/midi_file/issues/31
/// Spec 2.2: "it is important to read and honour the length, even if it is longer than 6."
#[test]
fn header_longer_than_six_bytes() {
    let bytes: Vec<u8> = vec![
        0x4D, 0x54, 0x68, 0x64, // MThd
        0x00, 0x00, 0x00, 0x08, // length 8
        0x00, 0x00, // format 0
        0x00, 0x01, // one track
        0x00, 0x60, // division 96
        0xAB, 0xCD, // two extra header bytes to be ignored
        0x4D, 0x54, 0x72, 0x6B, // MTrk
        0x00, 0x00, 0x00, 0x04, // length 4
        0x00, 0xFF, 0x2F, 0x00, // end of track
    ];
    let mf = MidiFile::read(bytes.as_slice()).unwrap();
    assert_eq!(1, mf.tracks_len());

    // a length shorter than 6 remains invalid
    let bytes: Vec<u8> = vec![
        0x4D, 0x54, 0x68, 0x64, // MThd
        0x00, 0x00, 0x00, 0x04, // length 4
        0x00, 0x00, // format 0
        0x00, 0x01, // one track
    ];
    assert!(MidiFile::read(bytes.as_slice()).is_err());
}

/// https://github.com/webern/midi_file/issues/37
/// Spec 1.1: "The largest number which is allowed is 0FFFFFFF", i.e. a delta-time vlq is at most
/// four bytes.
#[test]
fn vlq_capped_at_four_bytes() {
    use midi_file::file::{Event, Track};
    // a five-byte delta-time vlq must be rejected on read
    let bytes = file_with_track_data(&[
        0x81, 0x80, 0x80, 0x80, 0x00, // five-byte vlq for 0x10000000
        0xFF, 0x2F, 0x00, // end of track
    ]);
    assert!(MidiFile::read(bytes.as_slice()).is_err());

    // a delta time greater than 0x0FFFFFFF must be rejected on the way in
    let mut track = Track::default();
    assert!(track.push_event(0x1000_0000, Event::default()).is_err());
    assert!(track.push_event(0x0FFF_FFFF, Event::default()).is_ok());
}

/// https://github.com/webern/midi_file/issues/33
/// Spec 2.1: bits 14 thru 0 of the division word hold ticks per quarter, so values up to 32767
/// are valid and must roundtrip unaltered. A zero division must error, not silently become 1.
#[test]
fn division_full_15_bit_range() {
    use midi_file::file::Division;
    // division 20000 (0x4E20)
    let bytes = file_with_division(&[0x4E, 0x20]);
    let mf = MidiFile::read(bytes.as_slice()).unwrap();
    match mf.header().division() {
        Division::QuarterNote(q) => assert_eq!(20000, q.get()),
        d => panic!("wrong division {:?}", d),
    }
    let mut out: Vec<u8> = Vec::new();
    mf.write(&mut out).unwrap();
    assert_eq!(bytes, out);

    // division 0 is meaningless and must error
    let bytes = file_with_division(&[0x00, 0x00]);
    assert!(MidiFile::read(bytes.as_slice()).is_err());
}

/// https://github.com/webern/midi_file/issues/34
/// Spec 2.1: ntrks "will always be 1 for a format 0 file."
#[test]
fn format_0_rejects_second_track() {
    use midi_file::file::{Format, Track};
    use midi_file::Settings;
    let mut mf = MidiFile::new_with_settings(Settings::new().format(Format::Single));
    mf.push_track(Track::default()).unwrap();
    assert!(mf.push_track(Track::default()).is_err());
    assert!(mf.insert_track(0, Track::default()).is_err());
    assert_eq!(1, mf.tracks_len());
}

/// https://github.com/webern/midi_file/issues/36
/// Spec 3.1 gives 36 MIDI clocks per dotted quarter in its 6/8 example. A dotted whole is 144
/// (96 * 1.5).
#[test]
fn dotted_clocks_values() {
    use midi_file::core::Clocks;
    assert_eq!(Clocks::new(36), Clocks::DottedQuarter);
    assert_eq!(Clocks::new(144), Clocks::DottedWhole);
}

/// https://github.com/webern/midi_file/issues/35
/// A file that ends in the middle of a pitch bend message must error, not panic.
#[test]
fn truncated_pitch_bend_errors() {
    let bytes = file_with_track_data(&[0x00, 0xE0, 0x40]);
    assert!(MidiFile::read(bytes.as_slice()).is_err());
}
