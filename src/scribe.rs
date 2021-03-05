use crate::error::LibResult;
use snafu::ResultExt;
use std::io::Write;

#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub(crate) struct ScribeSettings {
    pub(crate) running_status: bool,
}

/// A wrapper for any `Write`, which provides a setting for running status, and allows for the
/// storing of the most recent status byte.
pub(crate) struct Scribe<W: Write> {
    w: W,
    settings: ScribeSettings,
    running_status_byte: Option<u8>,
}

impl<W: Write> Write for Scribe<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.w.flush()
    }
}

impl<W: Write> Scribe<W> {
    /// Create a new `Scribe`.
    pub(crate) fn new(w: W, settings: ScribeSettings) -> Self {
        Self {
            w,
            settings,
            running_status_byte: None,
        }
    }

    /// Write a status byte. If `running_status` is `true`, and the `status` byte is the same as
    /// `previous_status`, then nothing happens.
    pub(crate) fn write_status_byte(&mut self, status: u8) -> LibResult<()> {
        match self.running_status() {
            Some(previous_status) if previous_status == status => Ok(()),
            _ => {
                write_u8!(self.w, status)?;
                self.set_running_status(status);
                Ok(())
            }
        }
    }

    /// If the `running_status` setting is true, and a previous status byte has been written, then
    /// the previous status byte is returned.
    pub(crate) fn running_status(&self) -> Option<u8> {
        if self.use_running_status() {
            self.running_status_byte
        } else {
            None
        }
    }

    /// If the `running_status` setting is true, sets the `running_status_byte`, otherwise does
    /// nothing.
    pub(crate) fn set_running_status(&mut self, value: u8) {
        if self.use_running_status() {
            self.running_status_byte = Some(value)
        }
    }

    /// Returns true if the settings are set to use `running_status`.
    pub(crate) fn use_running_status(&self) -> bool {
        self.settings.running_status
    }
}
