use crate::error::LibResult;
use crate::scribe::Scribe;
use crate::{Division, Error};
use snafu::ResultExt;
use std::convert::TryFrom;
use std::io::Write;

/// 2.1 - Header Chunks
/// The header chunk at the beginning of the file specifies some basic information about the data in
/// the file. Here's the syntax of the complete chunk:
/// `<Header Chunk> = <chunk type><length><format><ntrks><division>`
///
/// As described above, <chunk type> is the four ASCII characters 'MThd'; <length> is a 32-bit
/// representation of the number 6 (high byte first).
///
/// The data section contains three 16-bit words, stored most-significant byte first.
///
/// The first word, <format>, specifies the overall organisation of the file.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Header {
    format: Format,
    division: Division,
}

impl Header {
    /// Create a new `Header` object.
    pub fn new(format: Format, division: Division) -> Self {
        Self { format, division }
    }

    /// A getter for the `format` field.
    pub fn format(&self) -> &Format {
        &self.format
    }

    /// A getter for the `division` field.
    pub fn division(&self) -> &Division {
        &self.division
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>, ntracks: u16) -> LibResult<()> {
        // write the header chunk identifier
        write!(w, "MThd").context(wr!())?;

        // write the header chunk length (always 6)
        w.write_all(&6u32.to_be_bytes()).context(wr!())?;

        // write the format indicator
        w.write_all(&(self.format as u16).to_be_bytes())
            .context(wr!())?;

        // write the number of tracks
        w.write_all(&ntracks.to_be_bytes()).context(wr!())?;

        // write the division value
        self.division.write(w)?;
        Ok(())
    }
}

/// The first word, <format>, specifies the overall organisation of the file. Only three values of
/// `<format>` are specified:
///
/// 0-the file contains a single multichannel track
/// 1-the file contains one or more simultaneous tracks (or MIDI outputs) of a sequence
/// 2-the file contains one or more sequentially independent single-track patterns
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub enum Format {
    /// 0 the file contains a single multi-channel track
    Single = 0,
    /// 1 the file contains one or more simultaneous tracks (or MIDI outputs) of a sequence
    #[default]
    Multi = 1,
    /// 2 the file contains one or more sequentially independent single-track patterns
    Sequential = 2,
}

impl Format {
    pub(crate) fn from_u16(value: u16) -> LibResult<Self> {
        match value {
            0 => Ok(Format::Single),
            1 => Ok(Format::Multi),
            2 => Ok(Format::Sequential),
            _ => crate::error::OtherSnafu { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u16> for Format {
    type Error = Error;

    fn try_from(value: u16) -> crate::Result<Self> {
        Ok(Self::from_u16(value)?)
    }
}
