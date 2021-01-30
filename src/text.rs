use log::warn;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};

/// The MIDI spec does not state what encoding should be used for strings. Since Rust strings are
/// UTF-8 encoded, we try to parse text as a `String` and hope for the best. But if we get an error
/// then we store the original bytes to facilitate lossless parsing.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum Text {
    Utf8(String),
    Other(Vec<u8>),
}

impl Default for Text {
    fn default() -> Self {
        Text::Utf8(String::new())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Text::Utf8(s) => Display::fmt(s, f),
            Text::Other(b) => write!(f, "{}", String::from_utf8_lossy(b)),
        }
    }
}

impl From<Vec<u8>> for Text {
    fn from(bytes: Vec<u8>) -> Self {
        match String::from_utf8(bytes.clone()) {
            Ok(s) => Text::Utf8(s),
            Err(_) => {
                warn!("non UTF-8 string encountered, encoding unknown");
                Text::Other(bytes)
            }
        }
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::Utf8(s)
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text::Utf8(s.into())
    }
}

/// Caution, this will be 'lossy' if the `Text` is not UTF-8 encoded.
impl Into<String> for Text {
    fn into(self) -> String {
        match self {
            Text::Utf8(s) => s,
            Text::Other(b) => String::from_utf8_lossy(&b).into(),
        }
    }
}

impl Text {
    pub fn new<S: Into<String>>(s: S) -> Self {
        Text::Utf8(s.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Text::Utf8(s) => s.as_bytes(),
            Text::Other(b) => b.as_slice(),
        }
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Text::Utf8(s) => Cow::Borrowed(s.as_str()),
            Text::Other(b) => String::from_utf8_lossy(b),
        }
    }
}
