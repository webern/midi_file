mod utils;

use crate::utils::{PITCH_BEND, PITCH_BEND_TWO_BYTES};
use midi_file::core::{Clocks, Control, DurationName, Message};
use midi_file::file::{Division, Event, Format, MetaEvent, QuarterNoteDivision};
use midi_file::MidiFile;
use std::fs::File;
use std::io::Read;
use tempfile::tempdir;
use utils::{enable_logging, test_file, AVE_MARIS_STELLA};

#[test]
fn ave_maris_stella_finale_export() {
    enable_logging();
    let midi_file = MidiFile::load(test_file(AVE_MARIS_STELLA)).unwrap();
    assert_eq!(*midi_file.header().format(), Format::Multi);
    assert_eq!(
        *midi_file.header().division(),
        Division::QuarterNote(QuarterNoteDivision::new(1024))
    );
    assert_eq!(midi_file.tracks_len(), 2);
    let mut tracks = midi_file.tracks();
    let track = tracks.next().unwrap();
    assert_eq!(29, track.events_len());
    let mut events = track.events();
    let track_event = events.next().unwrap();
    assert_eq!(0, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert!(matches!(data, MetaEvent::SmpteOffset(_)));

    // advance to the next event
    let track_event = events.next().unwrap();
    assert_eq!(0, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let MetaEvent::TimeSignature(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert_eq!(4, data.numerator());
    assert_eq!(DurationName::Quarter, data.denominator());
    assert_eq!(Clocks::Quarter, data.click());

    // advance somewhat randomly to sample another event
    let mut events = events.skip(20);
    let track_event = events.next().unwrap();
    assert_eq!(256, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let MetaEvent::SetTempo(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert_eq!(674576, data.get());

    // advance to the last event
    let mut events = events.skip(5);
    let track_event = events.next().unwrap();
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert!(matches!(data, MetaEvent::EndOfTrack));

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // next track
    let track = tracks.next().unwrap();
    assert_eq!(230, track.events_len());
    let mut events = track.events();
    let track_event = events.next().unwrap();
    assert_eq!(0, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let MetaEvent::DeviceName(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert_eq!("SmartMusic SoftSynth 1", data.as_str());

    // advance to the next event
    let track_event = events.next().unwrap();
    assert_eq!(0, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Meta(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let MetaEvent::TrackName(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    assert_eq!("[Staff 1]", data.as_str());

    // advance to the next event
    let track_event = events.next().unwrap();
    assert_eq!(0, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Midi(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let Message::ProgramChange(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };

    assert_eq!(0, data.channel().get());
    assert_eq!(0, data.program().get());

    // advance a ways into the track
    let mut events = events.skip(200);
    let track_event = events.next().unwrap();
    assert_eq!(80, track_event.delta_time());
    let data = track_event.event();
    let data = if let Event::Midi(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };
    let data = if let Message::Control(inner) = data {
        inner
    } else {
        panic!("wrong variant, got {:?}", data);
    };

    assert_eq!(0, data.channel().get());
    assert_eq!(Control::ChannelVolume, data.control());
    assert_eq!(83, data.value().get());

    // no more tracks
    assert!(tracks.next().is_none());

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // save file
    let mut written_bytes: Vec<u8> = Vec::new();
    midi_file.write(&mut written_bytes).unwrap();
    let mut original_bytes = Vec::new();
    let _ = File::open(test_file(AVE_MARIS_STELLA))
        .unwrap()
        .read_to_end(&mut original_bytes)
        .unwrap();

    // TODO - remove this
    //std::fs::write("/Users/mjb/Desktop/bad.mid", &written_bytes).unwrap();

    // assert files are the same size
    assert_eq!(written_bytes.len(), original_bytes.len());

    // assert files are exactly the same
    for (index, original) in original_bytes.iter().enumerate() {
        let written = written_bytes.get(index).unwrap();
        assert_eq!(original, written);
    }
}

#[test]
fn pitch_bend() {
    enable_logging();
    let midi_file = MidiFile::load(test_file(PITCH_BEND)).unwrap();
    let track = midi_file.tracks().next().unwrap();

    fn assert_pitch_bend(event: &Event, expected: u16) {
        let message = match event {
            Event::Midi(message) => message,
            _ => panic!("wrong event type {:?}", event),
        };
        let pitch_bend_message = match message {
            Message::PitchBend(p) => p,
            _ => panic!("wrong message type {:?}", message),
        };
        assert_eq!(pitch_bend_message.pitch_bend().get(), expected);
    }

    // The file was created with Logic Pro, which treats Pitch Bend values as a single 7-bit number,
    // from 0-127 instead of using the full range. If we multiply by 128 then we get the actual,
    // written 14-bit value instead of the value displayed in Logic's UI.
    assert_pitch_bend(track.events().nth(8).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(9).unwrap().event(), 20 * 128);
    assert_pitch_bend(track.events().nth(10).unwrap().event(), 40 * 128);
    assert_pitch_bend(track.events().nth(11).unwrap().event(), 127 * 128);
    assert_pitch_bend(track.events().nth(12).unwrap().event(), 125 * 128);
    assert_pitch_bend(track.events().nth(13).unwrap().event(), 101 * 128);
    assert_pitch_bend(track.events().nth(14).unwrap().event(), 40 * 128);
    assert_pitch_bend(track.events().nth(15).unwrap().event(), 20 * 128);

    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("file.mid");
    midi_file.save(&path).unwrap();
    let midi_file = MidiFile::load(&path).unwrap();
    let track = midi_file.tracks().next().unwrap();
    assert_pitch_bend(track.events().nth(8).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(9).unwrap().event(), 20 * 128);
    assert_pitch_bend(track.events().nth(10).unwrap().event(), 40 * 128);
    assert_pitch_bend(track.events().nth(11).unwrap().event(), 127 * 128);
    assert_pitch_bend(track.events().nth(12).unwrap().event(), 125 * 128);
    assert_pitch_bend(track.events().nth(13).unwrap().event(), 101 * 128);
    assert_pitch_bend(track.events().nth(14).unwrap().event(), 40 * 128);
    assert_pitch_bend(track.events().nth(15).unwrap().event(), 20 * 128);
}

#[test]
fn pitch_bend_two_byte() {
    enable_logging();
    let midi_file = MidiFile::load(test_file(PITCH_BEND_TWO_BYTES)).unwrap();
    let track = midi_file.tracks().nth(1).unwrap();

    fn assert_pitch_bend(event: &Event, expected: u16) {
        let message = match event {
            Event::Midi(message) => message,
            _ => panic!("wrong event type {:?}", event),
        };
        let pitch_bend_message = match message {
            Message::PitchBend(p) => p,
            _ => panic!("wrong message type {:?}", message),
        };
        assert_eq!(pitch_bend_message.pitch_bend().get(), expected);
    }

    assert_pitch_bend(track.events().nth(1).unwrap().event(), 8192);
    assert_pitch_bend(track.events().nth(3).unwrap().event(), 8292);
    assert_pitch_bend(track.events().nth(4).unwrap().event(), 8092);
    assert_pitch_bend(track.events().nth(5).unwrap().event(), 16383);
    assert_pitch_bend(track.events().nth(6).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(7).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(8).unwrap().event(), 1);

    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("file.mid");
    midi_file.save(&path).unwrap();
    let midi_file = MidiFile::load(&path).unwrap();
    let track = midi_file.tracks().nth(1).unwrap();
    assert_pitch_bend(track.events().nth(1).unwrap().event(), 8192);
    assert_pitch_bend(track.events().nth(3).unwrap().event(), 8292);
    assert_pitch_bend(track.events().nth(4).unwrap().event(), 8092);
    assert_pitch_bend(track.events().nth(5).unwrap().event(), 16383);
    assert_pitch_bend(track.events().nth(6).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(7).unwrap().event(), 0);
    assert_pitch_bend(track.events().nth(8).unwrap().event(), 1);
}
