use snafu::Snafu;
use std::num::TryFromIntError;
use std::path::PathBuf;

/// The public Error type for this library.
#[derive(Debug, Snafu)]
pub struct Error(LibError);

/// The public Result type for this library.
pub type Result<T> = std::result::Result<T, Error>;

/// The internal Result type for this library.
pub(crate) type LibResult<T> = std::result::Result<T, LibError>;

/// The internal Error type for this library.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum LibError {
    #[snafu(display("{} Error creating file '{}': {}", site, path.display(), source))]
    Create {
        site: String,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("{}: The MIDI file is invalid: {}", site, description))]
    InvalidFile { site: String, description: String },

    #[snafu(display("{} unknown error", site))]
    Other { site: String },

    #[snafu(display("{} Error while reading data: {}", site, source))]
    Read {
        site: String,
        source: crate::byte_iter::ByteError,
    },

    #[snafu(display("{} The string is too long and overflows a u32: {}", site, source))]
    StringTooLong {
        site: String,
        source: TryFromIntError,
    },

    #[snafu(display("{} There are too many tracks for a 16-byte uint: {}", site, source))]
    TooManyTracks {
        site: String,
        source: TryFromIntError,
    },

    #[snafu(display("{} The track is too long and overflows a u32: {}", site, source))]
    TrackTooLong {
        site: String,
        source: TryFromIntError,
    },

    #[snafu(display("{} Error while writing data: {}", site, source))]
    Write {
        site: String,
        source: std::io::Error,
    },
}

macro_rules! site {
    () => {
        format!("{}:{}", file!(), line!())
    };
}

macro_rules! io {
    () => {
        crate::error::Read { site: site!() }
    };
}

macro_rules! wr {
    () => {
        crate::error::Write { site: site!() }
    };
}

macro_rules! invalid_file_s {
    () => {
        crate::error::InvalidFile {
            site: site!(),
            description: "[no description]",
        }
    };
    ($msg:expr) => {
        crate::error::InvalidFile {
            site: site!(),
            description: $msg,
        }
    };
    ($fmt:expr, $($arg:expr),+) => {
        crate::error::InvalidFile {
            site: site!(),
            description: format!($fmt, $($arg),+),
        }
    };
}

macro_rules! invalid_file_e {
    () => {
        invalid_file_s!().build()
    };
    ($msg:expr) => {
        invalid_file_s!($msg).build()
    };
    ($fmt:expr, $($arg:expr),+) => {
        invalid_file_s!($fmt, $($arg),+).build()
    };
}

macro_rules! invalid_file_r {
    () => {
        Err(invalid_file_e!())
    };
    ($msg:expr) => {
        Err(invalid_file_e!($msg))
    };
    ($fmt:expr, $($arg:expr),+) => {
        Err(invalid_file_e!($fmt, $($arg),+))
    };
}

macro_rules! invalid_file {
    () => {
        return invalid_file_r!();
    };
    ($msg:expr) => {
        return invalid_file_r!($msg)
    };
    ($fmt:expr, $($arg:expr),+) => {
        return invalid_file_r!($fmt, $($arg),+)
    };
}

#[test]
fn site_test() {
    let line = line!() + 1;
    let site = site!();
    assert!(site.contains("error.rs"));
    assert!(site.contains(format!("{}", line).as_str()));
}

#[test]
fn invalid_file_macros_test_no_message() {
    fn foo() -> LibResult<u64> {
        invalid_file!();
    }
    let result = foo();
    assert!(result.is_err());
    let message = format!("{}", result.err().unwrap());
    assert!(message.as_str().contains("The MIDI file is invalid"));
}

#[test]
fn invalid_file_macros_test_message() {
    fn foo() -> LibResult<u64> {
        let flerbin = String::from("flerbin");
        invalid_file!(flerbin);
    }
    let result = foo();
    assert!(result.is_err());
    let message = format!("{}", result.err().unwrap());
    assert!(message.as_str().contains("flerbin"));
}

#[test]
fn invalid_file_macros_test_fmt() {
    fn foo() -> LibResult<u64> {
        invalid_file!("hello {}, {}", "world", String::from("foo"));
    }
    let result = foo();
    assert!(result.is_err());
    let message = format!("{}", result.err().unwrap());
    assert!(message.as_str().contains("hello world, foo"));
}
