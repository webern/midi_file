// one per line to simplify commenting certain ones out during development
#![deny(arithmetic_overflow)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(dead_code)]
// TODO - maybe document all pub(crate) types
// #![deny(missing_crate_level_docs)]
// TODO - document all
// #![deny(missing_docs)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unreachable_patterns)]
#![deny(unused_imports)]
#![deny(unused_variables)]

#[macro_use]
mod error;
#[macro_use]
mod macros;

use crate::byte_iter::ByteIter;
use std::convert::TryFrom;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

mod byte_iter;
pub mod core;
pub mod file;
mod scribe;
mod text;

use crate::error::LibResult;
use crate::file::{ensure_end_of_track, Division, Format, Header, Track};
use crate::scribe::{Scribe, ScribeSettings};
pub use crate::text::Text;
pub use error::{Error, Result};
use log::trace;
use snafu::{ensure, ResultExt};
use std::fs::File;

/// Optionally provide settings to the [`MidiFile`]. This is a 'builder' struct.
///
/// # Example
/// ```
/// use midi_file::{MidiFile, Settings};
/// use midi_file::file::{Format, Division, QuarterNoteDivision};
///
/// let settings = Settings::new()
///     .running_status(true)
///     .format(Format::Single)
///     .divisions(Division::QuarterNote(QuarterNoteDivision::new(244)));
/// let _m = MidiFile::new_with_settings(settings);
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Settings {
    /// The type of MIDI file. Defaults to `1`, i.e. `Multi`.
    format: Format,
    /// Defaults to a reasonable QuarterNote value.
    division: Division,
    /// Whether or not we should omit redundant status bytes.
    running_status: bool,
}

impl Settings {
    /// Create a `Settings` object with default settings.
    pub fn new() -> Self {
        Self {
            format: Format::default(),
            division: Division::default(),
            running_status: false,
        }
    }

    /// Set the `running_status` setting. When this is `true`, the [`MidiFile`] will not write
    /// redundant status bytes.
    pub fn running_status(mut self, value: bool) -> Self {
        self.running_status = value;
        self
    }

    /// Set the `format` setting. MIDI files can be one of three types, see [`Format`].
    pub fn format(mut self, value: Format) -> Self {
        self.format = value;
        self
    }

    /// Set the `division` setting, see [`Division`].
    pub fn divisions(mut self, value: Division) -> Self {
        self.division = value;
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a MIDI file, which consists of a header identifying the type of MIDI file, and tracks
/// with MIDI data.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct MidiFile {
    header: Header,
    tracks: Vec<Track>,
    running_status: bool,
}

impl Default for MidiFile {
    fn default() -> Self {
        Self::new()
    }
}

impl MidiFile {
    /// Create a new `MidiFile` with reasonable default [`Settings`].
    pub fn new() -> Self {
        Self::new_with_settings(Settings::new())
    }

    /// A getter for the `header` field.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// A getter for the `running_status` field.
    pub fn running_status(&self) -> bool {
        self.running_status
    }

    /// Create a new `MidiFile` with customizable [`Settings`].
    pub fn new_with_settings(settings: Settings) -> Self {
        Self {
            header: Header::new(settings.format, settings.division),
            tracks: Vec::new(),
            running_status: settings.running_status,
        }
    }

    /// Read a `MidiFile` from bytes.
    pub fn read<R: Read>(r: R) -> Result<Self> {
        let bytes = r.bytes();
        let iter = ByteIter::new(bytes).context(io!())?;
        Ok(Self::read_inner(iter)?)
    }

    /// Load a `MidiFile` from a file path.
    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self> {
        Ok(Self::read_inner(ByteIter::new_file(file).context(io!())?)?)
    }

    /// Write a `MidiFile` to bytes.
    pub fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        let ntracks =
            u16::try_from(self.tracks.len()).context(error::TooManyTracks { site: site!() })?;
        let mut scribe = Scribe::new(
            w,
            ScribeSettings {
                running_status: self.running_status,
            },
        );
        self.header.write(&mut scribe, ntracks)?;
        for track in self.tracks() {
            track.write(&mut scribe)?;
        }
        Ok(())
    }

    /// Save a `MidiFile` to a file path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path).context(error::Create {
            site: site!(),
            path,
        })?;
        let w = BufWriter::new(file);
        let mut scribe = Scribe::new(
            w,
            ScribeSettings {
                running_status: self.running_status,
            },
        );
        self.write(&mut scribe)
    }

    pub fn tracks_len(&self) -> u32 {
        u32::try_from(self.tracks.len()).unwrap_or(u32::MAX)
    }

    /// An iterator over the tracks in the file.
    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    /// Get a reference to the track at `index` if it exists.
    pub fn track(&self, index: u32) -> Option<&Track> {
        let i = match usize::try_from(index) {
            Ok(ok) => ok,
            Err(_) => return None,
        };
        self.tracks.get(i)
    }

    pub fn push_track(&mut self, track: Track) -> Result<()> {
        ensure!(self.tracks_len() < u32::MAX, error::Other { site: site!() });
        if *self.header().format() == Format::Single {
            ensure!(self.tracks_len() <= 1, error::Other { site: site!() });
        }
        self.tracks.push(ensure_end_of_track(track)?);
        Ok(())
    }

    pub fn insert_track(&mut self, index: u32, track: Track) -> Result<()> {
        ensure!(self.tracks_len() < u32::MAX, error::Other { site: site!() });
        if *self.header().format() == Format::Single {
            ensure!(self.tracks_len() <= 1, error::Other { site: site!() });
        }
        ensure!(index < self.tracks_len(), error::Other { site: site!() });
        self.tracks.insert(
            usize::try_from(index).context(error::TooManyTracks { site: site!() })?,
            ensure_end_of_track(track)?,
        );
        Ok(())
    }

    pub fn remove_track(&mut self, index: u32) -> Result<Track> {
        ensure!(index < self.tracks_len(), error::Other { site: site!() });
        let i = usize::try_from(index).context(error::TooManyTracks { site: site!() })?;
        Ok(self.tracks.remove(i))
    }

    fn read_inner<R: Read>(mut iter: ByteIter<R>) -> LibResult<Self> {
        trace!("parsing header chunk");
        iter.expect_tag("MThd").context(io!())?;
        let chunk_length = iter.read_u32().context(io!())?;
        // header chunk length is always 6
        if chunk_length != 6 {
            return error::Other { site: site!() }.fail();
        }
        let format_word = iter.read_u16().context(io!())?;
        let num_tracks = iter.read_u16().context(io!())?;
        let division_data = iter.read_u16().context(io!())?;
        let format = Format::from_u16(format_word)?;
        let header = Header::new(format, Division::from_u16(division_data)?);
        let mut tracks = Vec::new();
        for i in 0..num_tracks {
            trace!("parsing track chunk {} (zero-based) of {}", i, num_tracks);
            tracks.push(Track::parse(&mut iter)?)
        }
        Ok(Self {
            running_status: iter.is_running_status_detected(),
            header,
            tracks,
        })
    }
}
