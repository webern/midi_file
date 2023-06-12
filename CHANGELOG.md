# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.0.4] 2022-06-12
## Changed
- Add support for pitch bend events. Thank you, @robrennie [#15]

[#15]: https://github.com/webern/midi_file/issues/15

## [v0.0.3] 2022-12-13
## Changed
- Add some getters to access private fields. [#14]

[#14]: https://github.com/webern/midi_file/issues/14

## [v0.0.2] 2021-03-07
## Changed
- Fixed a bug where running status bytes were not parsed (thank you @zigazeljko). [#6]
- Added a `Settings` object for constructing a `MidiFile` object. [#6]
- Improved documentation. [#12]
- Clamp quarter note division values to allowable range. [#12]

[#6]: https://github.com/webern/midi_file/issues/6
[#12]: https://github.com/webern/midi_file/issues/12

## [v0.0.1] 2021-01-30
## Changed
- Re-organized modules and code files, added some documentation.


## [v0.0.0] 2021-01-18
## Added
- Everything: you can create simple MIDI files with this library.

<!-- version diff links -->
[Unreleased]: https://github.com/webern/midi_file/compare/v0.0.4...HEAD
[v0.0.3]: https://github.com/webern/midi_file/compare/v0.0.3...v0.0.4
[v0.0.3]: https://github.com/webern/midi_file/compare/v0.0.2...v0.0.3
[v0.0.2]: https://github.com/webern/midi_file/compare/v0.0.1...v0.0.2
[v0.0.1]: https://github.com/webern/midi_file/compare/v0.0.0...v0.0.1
[v0.0.0]: https://github.com/webern/midi_file/releases/tag/v0.0.0
