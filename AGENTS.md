# MIDI File

The goal of this project is to be compliant with the MIDI file speicification, checked-in here:
`./docs/spec-tekartik.txt` and to provide a usable interface on top of it.

The `Makefile` provides cannonical build commands. `make ci` is the gate used by continuous
integration.

Test files are in `tests/data`. We expect these to roundtrip through the library correctly.

