# Rust MIDI File Library

The purpose of this library is primarily to be able to author and write MIDI files in Rust.
The library also parses MIDI files and thus can "round trip" files.

The status is un-released.
Note that the current name `midi` is taken on crates.io,
so the name will need to be changed if published.

It feels like most features are implemented, except for `sysex` messages,
which I haven't happened to encounter yet in the files I have downloaded.

### Unimplemented Features

- `sysex` messages
- sequence number messages
- sequencer specific messages
- some horrible bug (see below)

### Horrible Lurking Bug

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

### Mess

Though I'm relatively happy with the data structure representing MIDI,
the code is messy and needs to be re-organized (and documented).
I'll get to it.

### Limited Interface

All the bytes, messages and such are represented with pub structs and enums,
but the structs have private members. 
So to create a basic file, as I have done in an [example], I have added functions at the `Track` 
level for pushing things onto the `Vec` without needing to know the underlying data structure.
This is may be a direction I want to continue with.

The API contract that I'm going for here is low-level-ish.
You need to understand MIDI in order to create a meaningful MIDI file,
but any file you create with the library will be technically valid per the spec.
You *do not* need to know what bytes equal what or what the bounds of an allowable value are.
This is all constrained with types.

[example]: https://github.com/webern/midi/blob/main/examples/main.rs