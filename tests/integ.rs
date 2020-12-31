mod utils;

use midi::{Division, Format, MidiFile};
use std::path::PathBuf;
use utils::enable_logging;

fn path(filename: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(filename);
    p.canonicalize()
        .unwrap_or_else(|_| panic!("bad path '{}'", p.display()))
}

#[test]
fn ave_maris_stella_finale_export() {
    enable_logging();
    let midi_file = MidiFile::load(path("ave_maris_stella_finale_export.midi")).unwrap();
    assert_eq!(*midi_file.header().format(), Format::Multi);
    assert_eq!(*midi_file.header().division(), Division::QuarterNote(1024));
    assert_eq!(midi_file.tracks_len(), 2);
}
