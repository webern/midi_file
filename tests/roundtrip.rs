mod utils;

use midi::MidiFile;
use std::fmt::{Debug, Display, Formatter};
use tempfile::TempDir;
use utils::{
    enable_logging, test_file, ADESTE_FIDELES, ALS_DIE_ROEMER, AVE_MARIS_STELLA, BARITONE_SAX,
    B_GUAJEO, LATER_FOLIA, LOGIC_PRO, PHOBOS_DORICO, TOBEFREE,
};

type RtResult = std::result::Result<(), RtErr>;

enum RtErr {
    BadByteValue(BadByte),
    Length(WrongLength),
    NotEqual(String),
}

impl Display for RtErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RtErr::BadByteValue(x) => Display::fmt(x, f),
            RtErr::Length(x) => Display::fmt(x, f),
            RtErr::NotEqual(x) => write!(
                f,
                "after reloading the saved file, it was found to be not-equal to the original \
                    file (using MidiFile::Eq), filename: {}",
                x
            ),
        }
    }
}

macro_rules! impldebug {
    ($symbol:ident) => {
        impl Debug for $symbol {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self, f)
            }
        }
    };
}

impldebug!(RtErr);

struct BadByte {
    file: String,
    byte_position: usize,
    expected: u8,
    actual: u8,
}

impl Display for BadByte {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "round trip test error, byte position: {}, expected: {:#04X}, actual: {:#04X}, filepath: {}",
            self.byte_position, self.expected, self.actual, self.file
        )
    }
}

impldebug!(BadByte);

struct WrongLength {
    file: String,
    expected: usize,
    actual: usize,
}

impl Display for WrongLength {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "round trip test error, expected {} bytes, got {} bytes, filepath: {}",
            self.expected, self.actual, self.file
        )
    }
}

impldebug!(WrongLength);

macro_rules! rtfail {
    ($file:expr, $ix:expr, $expected:expr, $actual:expr) => {
        return Err(RtErr::BadByteValue(BadByte {
            file: $file.as_ref().into(),
            byte_position: $ix,
            expected: $expected,
            actual: $actual,
        }));
    };
}

/// Asserts that a well-formed file can be deserialized then serialized to the exact same bytes.
fn round_trip_test<S: AsRef<str>>(filename: S) -> RtResult {
    enable_logging();
    let td = TempDir::new().unwrap();
    let out_path = td.path().join("output.mid");
    let in_path = test_file(&filename);
    let mf = MidiFile::load(&in_path).unwrap();
    mf.save(&out_path).unwrap();

    let original_bytes = std::fs::read(&in_path).unwrap();
    let saved_bytes = std::fs::read(&out_path).unwrap();

    if original_bytes.len() != saved_bytes.len() {
        return Err(RtErr::Length(WrongLength {
            file: filename.as_ref().into(),
            expected: original_bytes.len(),
            actual: saved_bytes.len(),
        }));
    }

    for (ix, expected) in original_bytes.iter().enumerate() {
        let actual = saved_bytes[ix];
        if actual != *expected {
            rtfail!(filename, ix, *expected, actual);
        }
    }

    let reloaded = MidiFile::load(&out_path).unwrap();
    if mf != reloaded {
        return Err(RtErr::NotEqual(filename.as_ref().into()));
    }
    Ok(())
}

type BadFileTestResult = std::result::Result<(), BadFileTestError>;
struct BadFileTestError {
    filename: String,
}

impl Display for BadFileTestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "loading file '{}' was expected to error, but did not",
            self.filename
        )
    }
}

impldebug!(BadFileTestError);

/// Asserts that loading a malformed file will return an error.
fn bad_file_test<S: AsRef<str>>(filename: S) -> BadFileTestResult {
    enable_logging();
    match MidiFile::load(filename.as_ref()) {
        Ok(_) => Err(BadFileTestError {
            filename: filename.as_ref().into(),
        }),
        Err(_) => Ok(()),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn adeste_fideles() {
    round_trip_test(ADESTE_FIDELES).unwrap();
}

// TODO - https://github.com/webern/midi/issues/1
#[test]
#[ignore]
fn als_die_roemer() {
    round_trip_test(ALS_DIE_ROEMER).unwrap();
}

#[test]
fn ave_maris_stella() {
    round_trip_test(AVE_MARIS_STELLA).unwrap();
}

#[test]
fn b_guajeo() {
    round_trip_test(B_GUAJEO).unwrap();
}

#[test]
fn baritone_sax() {
    bad_file_test(BARITONE_SAX).unwrap();
}

#[test]
fn later_folia() {
    round_trip_test(LATER_FOLIA).unwrap();
}

#[test]
fn logic_pro() {
    round_trip_test(LOGIC_PRO).unwrap();
}

// TODO - https://github.com/webern/midi/issues/1
#[test]
#[ignore]
fn phobos_dorico() {
    round_trip_test(PHOBOS_DORICO).unwrap();
}

// TODO - https://github.com/webern/midi/issues/1
#[test]
#[ignore]
fn tobeefree() {
    round_trip_test(TOBEFREE).unwrap();
}
