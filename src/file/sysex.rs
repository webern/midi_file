use crate::byte_iter::ByteIter;
use crate::error::LibResult;
use std::io::{Read, Write};

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct SysexEvent {
    t: SysexEventType,
    data: Vec<u8>,
}

impl SysexEvent {
    pub(crate) fn parse<R: Read>(_first_byte: u8, _r: &mut ByteIter<R>) -> LibResult<Self> {
        noimpl!("SysexEvent::parse")
    }

    pub(crate) fn write<W: Write>(&self, _w: &mut W) -> LibResult<()> {
        noimpl!("SysexEvent::write")
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum SysexEventType {
    F0 = 0xf0,
    F7 = 0xf7,
}

impl Default for SysexEventType {
    fn default() -> Self {
        SysexEventType::F0
    }
}
