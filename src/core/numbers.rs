clamp!(
    /// Represents the MIDI channel. The minimum value is `0`, the maximum value is `15`. This type
    /// will clamp values to the valid range.
    Channel,
    u8,
    0,
    15,
    0,
    pub
);

clamp!(
    /// Represents the MIDI note number (`C4` is `60`, for example). The minimum value is `0`,
    /// the maximum value is `127` (i.e. `u7`). This type will clamp values to the valid range.
    NoteNumber,
    u8,
    0,
    127,
    60,
    pub
);

clamp!(
    /// Represents the MIDI velocity. The minimum value is `0`, the maximum value is `127` (i.e.
    /// `u7`). This type will clamp values to the valid range.
    Velocity,
    u8,
    0,
    127,
    72,
    pub
);

clamp!(
    /// Represents the MIDI program number. The minimum value is `0`, the maximum value is `127`
    /// (i.e. `u7`). This type will clamp values to the valid range.
    Program,
    u8,
    0,
    127,
    0,
    pub
);

clamp!(
    /// Represents the number of channels in mono mode. The minimum value is `0`, the maximum value
    /// is `127` (i.e. `u7`). This type will clamp values to the valid range.
    MonoModeChannels,
    u8,
    0,
    127,
    0,
    pub
);

clamp!(
    /// Represents a MIDI control value. The minimum value is `0`, the maximum value  is `127` (i.e.
    /// `u7`). This type will clamp values to the valid range.
    ControlValue,
    u8,
    0,
    127,
    0,
    pub
);

clamp!(
    /// The [port](http://midi.teragonaudio.com/tech/midifile/obsolete.htm) number. The minimum
    /// value is `0`, maximum value is `255` (i.e. `u7`). The default value is `0`.
    PortValue,
    u8,
    0,
    127,
    0,
    pub
);

clamp!(
    /// Represents the MIDI pitch bend value. The minimum value is `0`, the maximum value is `16383`
    /// (i.e. `u14`). This type will clamp values to the valid range.
    PitchBendValue,
    u16,
    0,
    16383,
    8192,
    pub
);
