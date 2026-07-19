# MIDI File

The goal of this project is to be compliant with the MIDI file specification v1.1.

The spec is checked-in here: `./docs/spec-midimusic.md`. We MUST always check our code against the
spec for correctness.

Test files are in `tests/data`. We expect these to roundtrip through the library byte-perfectly.

The `Makefile` provides canonical build commands. `make ci` is the gate used by continuous
integration.

No `dependencies`: this library does not require dependencies in its Cargo.toml. Keep it that way.