// one per line to simplify commenting certain ones out during development
#![deny(arithmetic_overflow)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(dead_code)]
// #![deny(missing_crate_level_docs)]
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
mod text;

use crate::error::LibResult;
use crate::file::{ensure_end_of_track, Division, Format, Header, Track};
pub use crate::text::Text;
pub use error::{Error, Result};
use log::trace;
use snafu::{ensure, ResultExt};
use std::fs::File;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct MidiFile {
    header: Header,
    tracks: Vec<Track>,
}

impl MidiFile {
    pub fn new(format: Format, division: Division) -> Self {
        Self {
            header: Header::new(format, division),
            tracks: Vec::new(),
        }
    }

    pub fn read<R: Read>(r: R) -> Result<Self> {
        let bytes = r.bytes();
        let iter = ByteIter::new(bytes).context(io!())?;
        Ok(Self::read_inner(iter)?)
    }

    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self> {
        Ok(Self::read_inner(ByteIter::new_file(file).context(io!())?)?)
    }

    pub fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        let ntracks =
            u16::try_from(self.tracks.len()).context(error::TooManyTracks { site: site!() })?;
        self.header.write(w, ntracks)?;
        for track in self.tracks() {
            track.write(w)?;
        }
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(&path).context(error::Create {
            site: site!(),
            path,
        })?;
        let mut w = BufWriter::new(file);
        self.write(&mut w)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn tracks_len(&self) -> u32 {
        u32::try_from(self.tracks.len()).unwrap_or(u32::MAX)
    }

    pub fn tracks(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

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
        Ok(Self { header, tracks })
    }
}
