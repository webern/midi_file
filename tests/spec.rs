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

/// https://github.com/webern/midi_file/issues/35
/// A file that ends in the middle of a pitch bend message must error, not panic.
#[test]
fn truncated_pitch_bend_errors() {
    let bytes = file_with_track_data(&[0x00, 0xE0, 0x40]);
    assert!(MidiFile::read(bytes.as_slice()).is_err());
}
