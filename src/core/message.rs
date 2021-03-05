use crate::byte_iter::ByteIter;
use crate::core::{Channel, ControlValue, NoteNumber, Program, StatusType, Velocity, U7};
use crate::error::{self, LibResult};
use crate::scribe::Scribe;
use log::trace;
use log::warn;
use snafu::{OptionExt, ResultExt};
use std::convert::TryFrom;
use std::io::{Read, Write};

pub(crate) trait WriteBytes {
    fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()>;
}

/// Represents the data that is common, and required for both [`Message::NoteOn`] and
/// [`Message::NoteOff`] messages.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NoteMessage {
    pub(crate) channel: Channel,
    pub(crate) note_number: NoteNumber,
    pub(crate) velocity: Velocity,
}

impl NoteMessage {
    fn parse<R: Read>(iter: &mut ByteIter<R>, channel: Channel) -> LibResult<Self> {
        Ok(NoteMessage {
            channel,
            note_number: iter.read_or_die().context(io!())?.into(),
            velocity: iter.read_or_die().context(io!())?.into(),
        })
    }

    fn write<W: Write>(&self, w: &mut Scribe<W>, st: StatusType) -> LibResult<()> {
        write_status_byte(w, st, self.channel)?;
        w.write_all(&self.note_number.get().to_be_bytes())
            .context(wr!())?;
        w.write_all(&self.velocity.get().to_be_bytes())
            .context(wr!())?;
        Ok(())
    }
}

/// Provides the ability to change an instrument (sound, patch, etc.) by specifying the affected
/// channel number and the new program value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProgramChangeValue {
    pub(crate) channel: Channel,
    pub(crate) program: Program,
}

impl ProgramChangeValue {
    /// Get the channel value.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Get the program value.
    pub fn program(&self) -> &Program {
        &self.program
    }
}

impl WriteBytes for ProgramChangeValue {
    fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        write_status_byte(w, StatusType::Program, self.channel)?;
        write_u8!(w, self.program.get())?;
        Ok(())
    }
}

// TODO - unused?
/// Maybe unused.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ChannelPressureMessage {}

// TODO - unused?
/// Maybe unused.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PitchBendMessage {}

/// Some complicated MIDI thing.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum ModeMessage {
    AllSoundsOff(Channel),
    ResetAllControllers(Channel),
    LocalControl(LocalControlValue),
    AllNotesOff(Channel),
    OmniModeOff(Channel),
    OmniModeOn(Channel),
    MonoModeOn(MonoModeOnValue),
    PolyModeOn,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum OnOff {
    On = 127,
    Off = 0,
}

impl Default for OnOff {
    fn default() -> Self {
        OnOff::Off
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LocalControlValue {
    channel: Channel,
    on_off: OnOff,
}

impl Default for ModeMessage {
    fn default() -> Self {
        ModeMessage::AllSoundsOff(Channel::new(0))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MonoModeOnValue {
    channel: Channel,
    mono_mode_channels: U7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum SystemCommonMessage {
    MidiTimeCodeQuarterFrame(MidiTimeCodeQuarterFrameMessage),
    SongPositionPointer(SongPositionPointerMessage),
    SongSelect(SongSelectMessage),
    TuneRequest,
    EndOfSysexFlag,
}

impl Default for SystemCommonMessage {
    fn default() -> Self {
        SystemCommonMessage::MidiTimeCodeQuarterFrame(MidiTimeCodeQuarterFrameMessage::default())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MidiTimeCodeQuarterFrameMessage {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SongPositionPointerMessage {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SongSelectMessage {}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SystemRealtimeMessage {
    TimingClock = 0xf8,
    Undefined1 = 0xf9,
    Start = 0xfa,
    Continue = 0xfb,
    Stop = 0xfc,
    Undefined2 = 0xfd,
    ActiveSensing = 0xfe,
    SystemReset = 0xff,
}

impl Default for SystemRealtimeMessage {
    fn default() -> Self {
        SystemRealtimeMessage::TimingClock
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[allow(dead_code)]
pub enum SystemMessage {
    Common(SystemCommonMessage),
    Realtime(SystemRealtimeMessage),
}

impl Default for SystemMessage {
    fn default() -> Self {
        SystemMessage::Common(SystemCommonMessage::default())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Message {
    NoteOff(NoteMessage),
    NoteOn(NoteMessage),
    PolyPressure(NoteMessage),
    Control(ControlChangeValue),
    ProgramChange(ProgramChangeValue),
    ChannelPressure(ChannelPressureMessage),
    PitchBend(PitchBendMessage),
    AllSoundsOff(Channel),
    ResetAllControllers(Channel),
    LocalControlOff(Channel),
    LocalControlOn(Channel),
    AllNotesOff(Channel),
    OmniModeOff(Channel),
    OmniModeOn(Channel),
    MonoModeOn(MonoModeOnValue),
    PolyModeOn(Channel),
    MidiTimeCodeQuarterFrame(MidiTimeCodeQuarterFrameMessage),
    SongPositionPointer(SongPositionPointerMessage),
    SongSelect(SongSelectMessage),
    TuneRequest,
    EndOfSysexFlag,
    TimingClock,
    Undefined1,
    Start,
    Continue,
    Stop,
    Undefined2,
    ActiveSensing,
    SystemReset,
}

impl Default for Message {
    fn default() -> Self {
        Message::AllSoundsOff(Channel::default())
    }
}

impl Message {
    pub(crate) fn parse<R: Read>(iter: &mut ByteIter<R>) -> LibResult<Self> {
        // check if the first byte is a status byte. if not, then this should be a running status
        // message.
        let byte = if matches!(iter.peek_or_die().context(io!())?, 0x00..=0x7F) {
            iter.set_running_status_detected();
            let running_status = iter
                .latest_message_byte()
                .context(error::RunningStatus { site: site!() })?;
            trace!("running status byte {:#x}", running_status);
            running_status
        } else {
            let byte = iter.read_or_die().context(io!())?;
            iter.set_latest_message_byte(Some(byte));
            byte
        };

        // first check if the message is a sysex or realtime message (using the whole byte).
        match byte {
            x if SystemRealtimeMessage::TimingClock as u8 == x => return Ok(Message::TimingClock),
            x if SystemRealtimeMessage::Undefined1 as u8 == x => return Ok(Message::Undefined1),
            x if SystemRealtimeMessage::Start as u8 == x => return Ok(Message::Start),
            x if SystemRealtimeMessage::Continue as u8 == x => return Ok(Message::Continue),
            x if SystemRealtimeMessage::Stop as u8 == x => return Ok(Message::Stop),
            x if SystemRealtimeMessage::Undefined2 as u8 == x => return Ok(Message::Undefined2),
            x if SystemRealtimeMessage::ActiveSensing as u8 == x => {
                return Ok(Message::ActiveSensing)
            }
            x if SystemRealtimeMessage::SystemReset as u8 == x => return Ok(Message::SystemReset),
            0xf0 => noimpl!("sysex"),
            _ => {}
        }
        // now check if it is a channel voice message or channel mode message
        let (status_type, channel) = split_byte(byte)?;
        match status_type {
            StatusType::NoteOff => Ok(Message::NoteOff(NoteMessage::parse(iter, channel)?)),
            StatusType::NoteOn => Ok(Message::NoteOn(NoteMessage::parse(iter, channel)?)),
            StatusType::PolyPressure => {
                Ok(Message::PolyPressure(NoteMessage::parse(iter, channel)?))
            }
            StatusType::ControlOrSelectChannelMode => parse_0xb(iter, channel),
            StatusType::Program => {
                let program: Program = iter.read_or_die().context(io!())?.into();
                Ok(Message::ProgramChange(ProgramChangeValue {
                    channel,
                    program,
                }))
            }
            StatusType::ChannelPressure => noimpl!("channel pressure"),
            StatusType::PitchBend => noimpl!("pitch bend"),
            StatusType::System => noimpl!("system"),
        }
    }

    pub(crate) fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        match self {
            Message::NoteOff(value) => value.write(w, StatusType::NoteOff),
            Message::NoteOn(value) => value.write(w, StatusType::NoteOn),
            Message::PolyPressure(value) => value.write(w, StatusType::PolyPressure),
            Message::Control(value) => value.write(w),
            Message::ProgramChange(value) => value.write(w),
            Message::ChannelPressure(_) => noimpl!("ChannelPressure"),
            Message::PitchBend(_) => noimpl!("PitchBend"),
            Message::AllSoundsOff(channel) => write_chanmod(w, *channel, CONTROL_ALL_SOUNDS_OFF, 0),
            Message::ResetAllControllers(channel) => {
                write_chanmod(w, *channel, CONTROL_RESET_ALL_CONTROLLERS, 0)
            }
            Message::LocalControlOff(channel) => {
                write_chanmod(w, *channel, CONTROL_LOCAL_CONTROL, 0)
            }
            Message::LocalControlOn(channel) => {
                write_chanmod(w, *channel, CONTROL_LOCAL_CONTROL, 127)
            }
            Message::AllNotesOff(channel) => write_chanmod(w, *channel, CONTROL_ALL_NOTES_OFF, 0),
            Message::OmniModeOff(channel) => write_chanmod(w, *channel, CONTROL_OMNI_MODE_OFF, 0),
            Message::OmniModeOn(channel) => write_chanmod(w, *channel, CONTROL_OMNI_MODE_ON, 0),
            Message::MonoModeOn(m) => write_chanmod(
                w,
                m.channel,
                CONTROL_MONO_MODE_ON,
                m.mono_mode_channels.get(),
            ),
            Message::PolyModeOn(channel) => write_chanmod(w, *channel, CONTROL_POLY_MODE_ON, 0),
            Message::MidiTimeCodeQuarterFrame(_) => noimpl!("MidiTimeCodeQuarterFrame"),
            Message::SongPositionPointer(_) => noimpl!("SongPositionPointer"),
            Message::SongSelect(_) => noimpl!("SongSelect"),
            Message::TuneRequest => noimpl!("TuneRequest"),
            Message::EndOfSysexFlag => noimpl!("EndOfSysexFlag"),
            Message::TimingClock => noimpl!("TimingClock"),
            Message::Undefined1 => noimpl!("Undefined1"),
            Message::Start => noimpl!("Start"),
            Message::Continue => noimpl!("Continue"),
            Message::Stop => noimpl!("Stop"),
            Message::Undefined2 => noimpl!(""),
            Message::ActiveSensing => noimpl!("ActiveSensing"),
            Message::SystemReset => noimpl!("SystemReset"),
        }
    }
}

pub(crate) const CONTROL_ALL_SOUNDS_OFF: u8 = 120;
pub(crate) const CONTROL_RESET_ALL_CONTROLLERS: u8 = 121;
pub(crate) const CONTROL_LOCAL_CONTROL: u8 = 122;
pub(crate) const CONTROL_ALL_NOTES_OFF: u8 = 123;
pub(crate) const CONTROL_OMNI_MODE_OFF: u8 = 124;
pub(crate) const CONTROL_OMNI_MODE_ON: u8 = 125;
pub(crate) const CONTROL_MONO_MODE_ON: u8 = 126;
pub(crate) const CONTROL_POLY_MODE_ON: u8 = 127;

/// Returns (4-bit status part, 4-bit channel).
fn split_byte(status_byte: u8) -> LibResult<(StatusType, Channel)> {
    let status_type_val = status_byte >> 4;
    let status_type = StatusType::from_u8(status_type_val)?;
    let channel_value = status_byte & 0b0000_1111;
    let channel: Channel = channel_value.into();
    Ok((status_type, channel))
}

/// Combines the status part and channel part of a channel voice message.
fn merge_byte(status: StatusType, channel: Channel) -> u8 {
    let status_number = status as u8;
    let status_bits = status_number << 4;
    let channel_bits = channel.get();
    status_bits | channel_bits
}

/// Combines then writes the status part and channel part of a channel voice message.
fn write_status_byte<W: Write>(
    w: &mut Scribe<W>,
    status: StatusType,
    channel: Channel,
) -> LibResult<()> {
    let data = merge_byte(status, channel);
    w.write_status_byte(data)
}

fn parse_0xb<R: Read>(iter: &mut ByteIter<R>, channel: Channel) -> LibResult<Message> {
    let first_data_byte = iter.read_or_die().context(io!())?;
    match first_data_byte {
        0..=119 => parse_control(iter, channel, first_data_byte),
        120..=127 => parse_chanmod(iter, channel, first_data_byte),
        _ => invalid_file!("expected value between 0 and 127, got {}", first_data_byte),
    }
}

fn parse_chanmod<R>(it: &mut ByteIter<R>, chan: Channel, first_byte: u8) -> LibResult<Message>
where
    R: Read,
{
    let second_byte = it.read_or_die().context(io!())?;
    match first_byte {
        CONTROL_ALL_SOUNDS_OFF => Ok(Message::AllSoundsOff(chan)),
        CONTROL_RESET_ALL_CONTROLLERS => Ok(Message::ResetAllControllers(chan)),
        CONTROL_LOCAL_CONTROL => {
            if second_byte == 0 {
                Ok(Message::LocalControlOff(chan))
            } else {
                if second_byte != 127 {
                    warn!(
                        "unexpected local control on value, {}, setting to 127",
                        second_byte
                    )
                }
                Ok(Message::LocalControlOn(chan))
            }
        }
        CONTROL_ALL_NOTES_OFF => Ok(Message::AllNotesOff(chan)),
        CONTROL_OMNI_MODE_OFF => Ok(Message::OmniModeOff(chan)),
        CONTROL_OMNI_MODE_ON => Ok(Message::OmniModeOn(chan)),
        CONTROL_MONO_MODE_ON => Ok(Message::MonoModeOn(MonoModeOnValue {
            channel: chan,
            mono_mode_channels: U7::new(second_byte),
        })),
        CONTROL_POLY_MODE_ON => Ok(Message::PolyModeOn(chan)),
        _ => invalid_file!("Bad channel mode value {:#04X}", first_byte),
    }
}

fn write_chanmod<W>(w: &mut Scribe<W>, channel: Channel, controller: u8, value: u8) -> LibResult<()>
where
    W: Write,
{
    debug_assert!(matches!(controller, 120..=127));
    debug_assert!(matches!(value, 0..=127));
    let status_byte = 0xB0u8 | channel.get();
    w.write_status_byte(status_byte)?;
    write_u8!(w, controller)?;
    write_u8!(w, value)?;
    Ok(())
}

fn parse_control<R>(it: &mut ByteIter<R>, chan: Channel, first_data_byte: u8) -> LibResult<Message>
where
    R: Read,
{
    let control = Control::try_from_u8(first_data_byte)?;
    let value: ControlValue = it.read_or_die().context(io!())?.into();
    Ok(Message::Control(ControlChangeValue {
        channel: chan,
        control,
        value,
    }))
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Control {
    BankSelect = 0,
    ModWheel = 1,
    BreathController = 2,
    Undefined3 = 3,
    FootController = 4,
    PortamentoTime = 5,
    DataEntryMsb = 6,
    ChannelVolume = 7,
    Balance = 8,
    Undefined9 = 9,
    Pan = 10,
    ExpressionController = 11,
    EffectControl1 = 12,
    EffectControl2 = 13,
    Undefined14,
    Undefined15,
    GeneralPurpose1 = 16,
    GeneralPurpose2 = 17,
    GeneralPurpose3 = 18,
    GeneralPurpose4 = 19,
    Undefined20 = 20,
    Undefined21 = 21,
    Undefined22 = 22,
    Undefined23 = 23,
    Undefined24 = 24,
    Undefined25 = 25,
    Undefined26 = 26,
    Undefined27 = 27,
    Undefined28 = 28,
    Undefined29 = 29,
    Undefined30 = 30,
    Undefined31 = 31,

    // These represent the "LSB" for items 0-31. When a 0-31 message is larger than one byte,
    // two messages are sent, one with the MSB and one with the LSB.
    BankSelectLsb = 32,
    ModWheelLsb = 33,
    BreathControllerLsb = 34,
    Undefined3Lsb = 35,
    FootControllerLsb = 36,
    PortamentoTimeLsb = 37,
    DataEntryMsbLsb = 38,
    ChannelVolumeLsb = 39,
    BalanceLsb = 40,
    Undefined9Lsb = 41,
    PanLsb = 42,
    ExpressionControllerLsb = 43,
    EffectControl1Lsb = 44,
    EffectControl2Lsb = 45,
    Undefined14Lsb = 46,
    Undefined15Lsb = 47,
    GeneralPurpose1Lsb = 48,
    GeneralPurpose2Lsb = 49,
    GeneralPurpose3Lsb = 50,
    GeneralPurpose4Lsb = 51,
    Undefined20Lsb = 52,
    Undefined21Lsb = 53,
    Undefined22Lsb = 54,
    Undefined23Lsb = 55,
    Undefined24Lsb = 56,
    Undefined25Lsb = 57,
    Undefined26Lsb = 58,
    Undefined27Lsb = 59,
    Undefined28Lsb = 60,
    Undefined29Lsb = 61,
    Undefined30Lsb = 62,
    Undefined31Lsb = 63,

    DamperPedalSustain = 64,
    PortamentoOnOff = 65,
    Sostenuto = 66,
    SoftPedal = 67,
    LegatoFootswitch = 68,
    Hold2 = 69,
    SoundVariation = 70,
    HarmonicIntensity = 71,
    ReleaseTime = 72,
    AttackTime = 73,
    Brightness = 74,
    SoundControllers6 = 75,
    SoundControllers7 = 76,
    SoundControllers8 = 77,
    SoundControllers9 = 78,
    SoundControllers10 = 79,
    GeneralPurpose5 = 80,
    GeneralPurpose6 = 81,
    GeneralPurpose7 = 82,
    GeneralPurpose8 = 83,
    PortamentoControl = 84,
    Undefined85 = 85,
    Undefined86 = 86,
    Undefined87 = 87,
    Undefined88 = 88,
    Undefined89 = 89,
    Undefined90 = 90,
    Effects1Depth = 91,
    Effects2Depth = 92,
    Effects3Depth = 93,
    Effects4Depth = 94,
    Effects5Depth = 95,
    DataIncrement = 96,
    DataDecrement = 97,
    NonRegisteredParameterNumberLsb = 98,
    NonRegisteredParameterNumberMsb = 99,
    RegisteredParameterNumberLsb = 100,
    RegisteredParameterNumberMsb = 101,
    Undefined102 = 102,
    Undefined103 = 103,
    Undefined104 = 104,
    Undefined105 = 105,
    Undefined106 = 106,
    Undefined107 = 107,
    Undefined108 = 108,
    Undefined109 = 109,
    Undefined110 = 110,
    Undefined111 = 111,
    Undefined112 = 112,
    Undefined113 = 113,
    Undefined114 = 114,
    Undefined115 = 115,
    Undefined116 = 116,
    Undefined117 = 117,
    Undefined118 = 118,
    Undefined119 = 119,
}

impl Default for Control {
    fn default() -> Self {
        Control::BankSelect
    }
}

impl Control {
    pub(crate) fn try_from_u8(value: u8) -> LibResult<Self> {
        match value {
            x if x == Control::BankSelect as u8 => Ok(Control::BankSelect),
            x if x == Control::ModWheel as u8 => Ok(Control::ModWheel),
            x if x == Control::BreathController as u8 => Ok(Control::BreathController),
            x if x == Control::Undefined3 as u8 => Ok(Control::Undefined3),
            x if x == Control::FootController as u8 => Ok(Control::FootController),
            x if x == Control::PortamentoTime as u8 => Ok(Control::PortamentoTime),
            x if x == Control::DataEntryMsb as u8 => Ok(Control::DataEntryMsb),
            x if x == Control::ChannelVolume as u8 => Ok(Control::ChannelVolume),
            x if x == Control::Balance as u8 => Ok(Control::Balance),
            x if x == Control::Undefined9 as u8 => Ok(Control::Undefined9),
            x if x == Control::Pan as u8 => Ok(Control::Pan),
            x if x == Control::ExpressionController as u8 => Ok(Control::ExpressionController),
            x if x == Control::EffectControl1 as u8 => Ok(Control::EffectControl1),
            x if x == Control::EffectControl2 as u8 => Ok(Control::EffectControl2),
            x if x == Control::Undefined14 as u8 => Ok(Control::Undefined14),
            x if x == Control::Undefined15 as u8 => Ok(Control::Undefined15),
            x if x == Control::GeneralPurpose1 as u8 => Ok(Control::GeneralPurpose1),
            x if x == Control::GeneralPurpose2 as u8 => Ok(Control::GeneralPurpose2),
            x if x == Control::GeneralPurpose3 as u8 => Ok(Control::GeneralPurpose3),
            x if x == Control::GeneralPurpose4 as u8 => Ok(Control::GeneralPurpose4),
            x if x == Control::Undefined20 as u8 => Ok(Control::Undefined20),
            x if x == Control::Undefined21 as u8 => Ok(Control::Undefined21),
            x if x == Control::Undefined22 as u8 => Ok(Control::Undefined22),
            x if x == Control::Undefined23 as u8 => Ok(Control::Undefined23),
            x if x == Control::Undefined24 as u8 => Ok(Control::Undefined24),
            x if x == Control::Undefined25 as u8 => Ok(Control::Undefined25),
            x if x == Control::Undefined26 as u8 => Ok(Control::Undefined26),
            x if x == Control::Undefined27 as u8 => Ok(Control::Undefined27),
            x if x == Control::Undefined28 as u8 => Ok(Control::Undefined28),
            x if x == Control::Undefined29 as u8 => Ok(Control::Undefined29),
            x if x == Control::Undefined30 as u8 => Ok(Control::Undefined30),
            x if x == Control::Undefined31 as u8 => Ok(Control::Undefined31),
            x if x == Control::BankSelectLsb as u8 => Ok(Control::BankSelectLsb),
            x if x == Control::ModWheelLsb as u8 => Ok(Control::ModWheelLsb),
            x if x == Control::BreathControllerLsb as u8 => Ok(Control::BreathControllerLsb),
            x if x == Control::Undefined3Lsb as u8 => Ok(Control::Undefined3Lsb),
            x if x == Control::FootControllerLsb as u8 => Ok(Control::FootControllerLsb),
            x if x == Control::PortamentoTimeLsb as u8 => Ok(Control::PortamentoTimeLsb),
            x if x == Control::DataEntryMsbLsb as u8 => Ok(Control::DataEntryMsbLsb),
            x if x == Control::ChannelVolumeLsb as u8 => Ok(Control::ChannelVolumeLsb),
            x if x == Control::BalanceLsb as u8 => Ok(Control::BalanceLsb),
            x if x == Control::Undefined9Lsb as u8 => Ok(Control::Undefined9Lsb),
            x if x == Control::PanLsb as u8 => Ok(Control::PanLsb),
            x if x == Control::ExpressionControllerLsb as u8 => {
                Ok(Control::ExpressionControllerLsb)
            }
            x if x == Control::EffectControl1Lsb as u8 => Ok(Control::EffectControl1Lsb),
            x if x == Control::EffectControl2Lsb as u8 => Ok(Control::EffectControl2Lsb),
            x if x == Control::Undefined14Lsb as u8 => Ok(Control::Undefined14Lsb),
            x if x == Control::Undefined15Lsb as u8 => Ok(Control::Undefined15Lsb),
            x if x == Control::GeneralPurpose1Lsb as u8 => Ok(Control::GeneralPurpose1Lsb),
            x if x == Control::GeneralPurpose2Lsb as u8 => Ok(Control::GeneralPurpose2Lsb),
            x if x == Control::GeneralPurpose3Lsb as u8 => Ok(Control::GeneralPurpose3Lsb),
            x if x == Control::GeneralPurpose4Lsb as u8 => Ok(Control::GeneralPurpose4Lsb),
            x if x == Control::Undefined20Lsb as u8 => Ok(Control::Undefined20Lsb),
            x if x == Control::Undefined21Lsb as u8 => Ok(Control::Undefined21Lsb),
            x if x == Control::Undefined22Lsb as u8 => Ok(Control::Undefined22Lsb),
            x if x == Control::Undefined23Lsb as u8 => Ok(Control::Undefined23Lsb),
            x if x == Control::Undefined24Lsb as u8 => Ok(Control::Undefined24Lsb),
            x if x == Control::Undefined25Lsb as u8 => Ok(Control::Undefined25Lsb),
            x if x == Control::Undefined26Lsb as u8 => Ok(Control::Undefined26Lsb),
            x if x == Control::Undefined27Lsb as u8 => Ok(Control::Undefined27Lsb),
            x if x == Control::Undefined28Lsb as u8 => Ok(Control::Undefined28Lsb),
            x if x == Control::Undefined29Lsb as u8 => Ok(Control::Undefined29Lsb),
            x if x == Control::Undefined30Lsb as u8 => Ok(Control::Undefined30Lsb),
            x if x == Control::Undefined31Lsb as u8 => Ok(Control::Undefined31Lsb),
            x if x == Control::DamperPedalSustain as u8 => Ok(Control::DamperPedalSustain),
            x if x == Control::PortamentoOnOff as u8 => Ok(Control::PortamentoOnOff),
            x if x == Control::Sostenuto as u8 => Ok(Control::Sostenuto),
            x if x == Control::SoftPedal as u8 => Ok(Control::SoftPedal),
            x if x == Control::LegatoFootswitch as u8 => Ok(Control::LegatoFootswitch),
            x if x == Control::Hold2 as u8 => Ok(Control::Hold2),
            x if x == Control::SoundVariation as u8 => Ok(Control::SoundVariation),
            x if x == Control::HarmonicIntensity as u8 => Ok(Control::HarmonicIntensity),
            x if x == Control::ReleaseTime as u8 => Ok(Control::ReleaseTime),
            x if x == Control::AttackTime as u8 => Ok(Control::AttackTime),
            x if x == Control::Brightness as u8 => Ok(Control::Brightness),
            x if x == Control::SoundControllers6 as u8 => Ok(Control::SoundControllers6),
            x if x == Control::SoundControllers7 as u8 => Ok(Control::SoundControllers7),
            x if x == Control::SoundControllers8 as u8 => Ok(Control::SoundControllers8),
            x if x == Control::SoundControllers9 as u8 => Ok(Control::SoundControllers9),
            x if x == Control::SoundControllers10 as u8 => Ok(Control::SoundControllers10),
            x if x == Control::GeneralPurpose5 as u8 => Ok(Control::GeneralPurpose5),
            x if x == Control::GeneralPurpose6 as u8 => Ok(Control::GeneralPurpose6),
            x if x == Control::GeneralPurpose7 as u8 => Ok(Control::GeneralPurpose7),
            x if x == Control::GeneralPurpose8 as u8 => Ok(Control::GeneralPurpose8),
            x if x == Control::PortamentoControl as u8 => Ok(Control::PortamentoControl),
            x if x == Control::Undefined85 as u8 => Ok(Control::Undefined85),
            x if x == Control::Undefined86 as u8 => Ok(Control::Undefined86),
            x if x == Control::Undefined87 as u8 => Ok(Control::Undefined87),
            x if x == Control::Undefined88 as u8 => Ok(Control::Undefined88),
            x if x == Control::Undefined89 as u8 => Ok(Control::Undefined89),
            x if x == Control::Undefined90 as u8 => Ok(Control::Undefined90),
            x if x == Control::Effects1Depth as u8 => Ok(Control::Effects1Depth),
            x if x == Control::Effects2Depth as u8 => Ok(Control::Effects2Depth),
            x if x == Control::Effects3Depth as u8 => Ok(Control::Effects3Depth),
            x if x == Control::Effects4Depth as u8 => Ok(Control::Effects4Depth),
            x if x == Control::Effects5Depth as u8 => Ok(Control::Effects5Depth),
            x if x == Control::DataIncrement as u8 => Ok(Control::DataIncrement),
            x if x == Control::DataDecrement as u8 => Ok(Control::DataDecrement),
            x if x == Control::NonRegisteredParameterNumberLsb as u8 => {
                Ok(Control::NonRegisteredParameterNumberLsb)
            }
            x if x == Control::NonRegisteredParameterNumberMsb as u8 => {
                Ok(Control::NonRegisteredParameterNumberMsb)
            }
            x if x == Control::RegisteredParameterNumberLsb as u8 => {
                Ok(Control::RegisteredParameterNumberLsb)
            }
            x if x == Control::RegisteredParameterNumberMsb as u8 => {
                Ok(Control::RegisteredParameterNumberMsb)
            }
            x if x == Control::Undefined102 as u8 => Ok(Control::Undefined102),
            x if x == Control::Undefined103 as u8 => Ok(Control::Undefined103),
            x if x == Control::Undefined104 as u8 => Ok(Control::Undefined104),
            x if x == Control::Undefined105 as u8 => Ok(Control::Undefined105),
            x if x == Control::Undefined106 as u8 => Ok(Control::Undefined106),
            x if x == Control::Undefined107 as u8 => Ok(Control::Undefined107),
            x if x == Control::Undefined108 as u8 => Ok(Control::Undefined108),
            x if x == Control::Undefined109 as u8 => Ok(Control::Undefined109),
            x if x == Control::Undefined110 as u8 => Ok(Control::Undefined110),
            x if x == Control::Undefined111 as u8 => Ok(Control::Undefined111),
            x if x == Control::Undefined112 as u8 => Ok(Control::Undefined112),
            x if x == Control::Undefined113 as u8 => Ok(Control::Undefined113),
            x if x == Control::Undefined114 as u8 => Ok(Control::Undefined114),
            x if x == Control::Undefined115 as u8 => Ok(Control::Undefined115),
            x if x == Control::Undefined116 as u8 => Ok(Control::Undefined116),
            x if x == Control::Undefined117 as u8 => Ok(Control::Undefined117),
            x if x == Control::Undefined118 as u8 => Ok(Control::Undefined118),
            x if x == Control::Undefined119 as u8 => Ok(Control::Undefined119),
            _ => error::Other { site: site!() }.fail(),
        }
    }
}

impl TryFrom<u8> for Control {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self::try_from_u8(value)?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ControlChangeValue {
    channel: Channel,
    control: Control,
    value: ControlValue,
}

impl ControlChangeValue {
    pub fn channel(&self) -> Channel {
        self.channel
    }
    pub fn control(&self) -> Control {
        self.control
    }
    pub fn value(&self) -> ControlValue {
        self.value
    }
}

impl WriteBytes for ControlChangeValue {
    fn write<W: Write>(&self, w: &mut Scribe<W>) -> LibResult<()> {
        write_status_byte(w, StatusType::ControlOrSelectChannelMode, self.channel)?;
        write_u8!(w, self.control as u8)?;
        write_u8!(w, self.value.get())?;
        Ok(())
    }
}
