// channel is 0-15, displayed to users as 1-16.
clamp!(Channel, u8, 0, 15, 0, pub);

clamp!(NoteNumber, u8, 0, 127, 60, pub);
clamp!(Velocity, u8, 0, 127, 72, pub);
clamp!(Program, u8, 0, 127, 0, pub);
clamp!(U7, u8, 0, 127, 0, pub);
clamp!(ControlValue, u8, 0, 127, 0, pub);
clamp!(PortValue, u8, 0, 15, 0, pub);
