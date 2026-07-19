//! The `error` module provides the error type used throughout the library.

use std::fmt::{Display, Formatter};

/// The public Result type for this library.
pub type Result<T> = std::result::Result<T, Error>;

/// Identifies what went wrong. The human-readable detail is in the [`Error`]'s `Display` output.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorType {
    /// An underlying reader or writer failed.
    Io,

    /// The data ended while more bytes were expected.
    End,

    /// Bytes that should have been a string were not valid UTF-8.
    Str,

    /// A four-character chunk tag, such as `MThd`, was not the expected one.
    Tag,

    /// A variable-length quantity ran past its four-byte maximum.
    VlqTooBig,

    /// A variable-length quantity could not be decoded.
    VlqDecode,

    /// A byte did not have the one value the spec allows at that position.
    ReadExpect,

    /// A file could not be opened for reading.
    FileOpen,

    /// A file could not be created for writing.
    FileCreate,

    /// The bytes are not a well-formed MIDI file.
    InvalidFile,

    /// A message relied on running status, but no status byte had been seen yet.
    RunningStatus,

    /// A string's length overflows the `u32` its length field is stored in.
    StringTooLong,

    /// The track count overflows the `u16` the header stores it in.
    TooManyTracks,

    /// A track's length overflows the `u32` its length field is stored in.
    TrackTooLong,

    /// Something went wrong that has no more specific classification.
    Other,
}

impl ErrorType {
    /// The generic description used when an [`Error`] carries no message of its own.
    fn description(self) -> &'static str {
        match self {
            ErrorType::Io => "an I/O error occurred",
            ErrorType::End => "the data ended unexpectedly",
            ErrorType::Str => "expected a string but found non-UTF-8 bytes",
            ErrorType::Tag => "the chunk tag was not the expected one",
            ErrorType::VlqTooBig => "too many bytes while reading a variable-length quantity",
            ErrorType::VlqDecode => "unable to decode a variable-length quantity",
            ErrorType::ReadExpect => "a byte did not have the required value",
            ErrorType::FileOpen => "unable to open the file",
            ErrorType::FileCreate => "unable to create the file",
            ErrorType::InvalidFile => "The MIDI file is invalid",
            ErrorType::RunningStatus => "expected a running status byte but found none",
            ErrorType::StringTooLong => "the string is too long and overflows a u32",
            ErrorType::TooManyTracks => "there are too many tracks for a 16-bit uint",
            ErrorType::TrackTooLong => "the track is too long and overflows a u32",
            ErrorType::Other => "unknown error",
        }
    }
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

/// The error type for this library.
#[derive(Debug)]
pub struct Error {
    /// The error type. Clients can access this.
    error_type: ErrorType,

    /// The relative path to the file, and line number, where the error
    /// occured in this library. For example: `src/lib.rs:31`.
    site: String,

    /// The contextual message for the error.
    message: Option<String>,

    /// The underlying error, if any
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    /// Create an `Error` with no message and no underlying cause. `site` is a `file.rs:line`
    /// string, as produced by the [`site`] macro.
    pub fn new<S: Into<String>>(error_type: ErrorType, site: S) -> Self {
        Self {
            error_type,
            site: site.into(),
            message: None,
            source: None,
        }
    }

    /// Attach a human-readable message describing this particular failure.
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Attach the underlying error that caused this one.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// A getter for the `error_type` field.
    pub fn error_type(&self) -> ErrorType {
        self.error_type
    }

    /// Where in this library the error was raised, e.g. `src/lib.rs:31`.
    pub fn site(&self) -> &str {
        &self.site
    }

    /// The message describing this particular failure, if one was attached.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.site, self.error_type)?;
        if let Some(message) = &self.message {
            write!(f, ": {}", message)?;
        }
        if let Some(source) = &self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

/// Everything needed to build an [`Error`] except the underlying cause.
pub struct ErrorContext {
    error_type: ErrorType,
    site: String,
    message: Option<String>,
}

impl ErrorContext {
    /// Create an `ErrorContext`. The [`ctx`] macro fills in `site`.
    pub fn new<S: Into<String>>(error_type: ErrorType, site: S) -> Self {
        Self {
            error_type,
            site: site.into(),
            message: None,
        }
    }

    /// Attach a human-readable message describing this particular failure.
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Build the [`Error`] with no underlying cause.
    pub fn into_error(self) -> Error {
        let error = Error::new(self.error_type, self.site);
        match self.message {
            Some(message) => error.with_message(message),
            None => error,
        }
    }

    /// Build the [`Error`] and wrap it in `Err`, for use as a function's return value.
    pub fn fail<T>(self) -> Result<T> {
        Err(self.into_error())
    }
}

/// Converts a failure into this library's [`Error`], attaching the type, site and message carried by
/// an [`ErrorContext`]. Implemented for both `Result` and `Option`.
///
/// The context closure runs only on failure.
pub trait Context<T> {
    /// Convert the failure case into an [`Error`] described by `context`.
    fn context<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext;
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| context().into_error().with_source(e))
    }
}

impl<T> Context<T> for Option<T> {
    fn context<F>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.ok_or_else(|| context().into_error())
    }
}

/// The `file.rs:line` where this macro is expanded.
macro_rules! site {
    () => {
        format!("{}:{}", file!(), line!())
    };
}

/// An [`crate::error::ErrorContext`] of the given type, sited at the call, with an optional
/// `format!`-style message. Expands to a closure, so eager uses must call it.
macro_rules! ctx {
    ($error_type:expr) => {
        || crate::error::ErrorContext::new($error_type, site!())
    };
    ($error_type:expr, $msg:expr) => {
        || crate::error::ErrorContext::new($error_type, site!()).with_message($msg)
    };
    ($error_type:expr, $fmt:expr, $($arg:expr),+) => {
        || crate::error::ErrorContext::new($error_type, site!())
            .with_message(format!($fmt, $($arg),+))
    };
}

/// Shorthand for the context used when reading bytes fails.
macro_rules! io {
    () => {
        ctx!(crate::error::ErrorType::Io)
    };
}

/// Shorthand for the context used when a write fails.
macro_rules! wr {
    () => {
        ctx!(crate::error::ErrorType::Io)
    };
}

/// Return early with an error unless the condition holds.
macro_rules! ensure {
    ($condition:expr, $context:expr) => {
        if !$condition {
            return $context().fail();
        }
    };
}

macro_rules! invalid_file_e {
    () => {
        ctx!(crate::error::ErrorType::InvalidFile)().into_error()
    };
    ($msg:expr) => {
        ctx!(crate::error::ErrorType::InvalidFile, $msg)().into_error()
    };
    ($fmt:expr, $($arg:expr),+) => {
        ctx!(crate::error::ErrorType::InvalidFile, $fmt, $($arg),+)().into_error()
    };
}

/// Return early, reporting that the bytes are not a well-formed MIDI file.
macro_rules! invalid_file {
    () => {
        return Err(invalid_file_e!())
    };
    ($msg:expr) => {
        return Err(invalid_file_e!($msg))
    };
    ($fmt:expr, $($arg:expr),+) => {
        return Err(invalid_file_e!($fmt, $($arg),+))
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
    fn foo() -> Result<u64> {
        invalid_file!();
    }
    let result = foo();
    assert!(result.is_err());
    let message = format!("{}", result.err().unwrap());
    assert!(message.as_str().contains("The MIDI file is invalid"));
}

#[test]
fn invalid_file_macros_test_message() {
    fn foo() -> Result<u64> {
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
    fn foo() -> Result<u64> {
        invalid_file!("hello {}, {}", "world", String::from("foo"));
    }
    let result = foo();
    assert!(result.is_err());
    let message = format!("{}", result.err().unwrap());
    assert!(message.as_str().contains("hello world, foo"));
}
