# Rust MIDI File Library

The purpose of this library is primarily to be able to author and write MIDI files in Rust.
The library also parses MIDI files and thus can "round trip" files.

### Unimplemented Features

- `sysex` messages
- sequence number messages
- sequencer specific messages
- some horrible bug (see below)

### Bug

I need help figuring out why I am seeing what I expect to be [midi messages],
but which start with status bytes that I don't recognize.
Several files I have found trigger this problem.
There are currently three tests `#[ignore]`ed which we fail to parse.
I don't get it.

Fortunately, for my intended use-case (authoring and writing files) this is not a problem.
We simply won't be writing the MIDI messages that I don't understand.

Please see issue [#1] and help if you understand MIDI.

[#1]: https://github.com/webern/midi/issues/1
[midi messages]: http://www.music.mcgill.ca/~ich/classes/mumt306/StandardMIDIfileformat.html#BMA1_

### Interface

All the bytes, messages and such are represented with pub structs and enums, but the structs have private members. 
To create a basic file, as I have done in an [example], I have added functions at the `Track` level.
With these functions you can build up a file without as much knowledge of the underlying data structure.

You need to understand MIDI in order to create a meaningful MIDI file, but any file you create with the library should
be technically valid per the spec.
You do not need to understand the meaning of any particular byte's numeric value. 

[example]: https://github.com/webern/midi/blob/main/examples/main.rs
