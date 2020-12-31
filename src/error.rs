use snafu::Snafu;

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
    #[snafu(display("Error while reading bytes: {}", source))]
    Io {
        site: String,
        source: crate::byte_iter::ByteError,
    },

    #[snafu(display("{}: The MIDI file is invalid: {}", site, description))]
    InvalidFile { site: String, description: String },

    #[snafu(display("{} unknown error", site))]
    Other { site: String },
}

macro_rules! site {
    () => {
        format!("{}:{}", file!(), line!())
    };
}

macro_rules! io {
    () => {
        crate::error::Io { site: site!() }
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
