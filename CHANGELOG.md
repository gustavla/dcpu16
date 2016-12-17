# Changelog

## Next release
Released: TBD
* Moved devices from `dcpu16-gui` crate to here
  * `DeviceMonitorLEM1802`
  * `DeviceKeyboardGeneric`
  * `DeviceFloppyM35FD`

## 0.3.0
Released: 2016-12-16
* Add `run` to `Device`, allowing asynchronous devices (e.g. floppy drive)
* Add `--help` and `--version` to all binaries

## 0.2.0
Released: 2016-12-15
* Fixed bug in JSR (fix was incomplete in 0.1.0)
* Changed ownership structure of devices in `DCPU`
* Renamed `Hardware` to `Device`
* Added `as_any_mut` to `Device`

## 0.1.0
Released: 2016-12-14
* Added `DAF`, a data fill assembly instruction (e.g. `DAF 256, 0xffff`)
* Assembler will resolve simple arithmetic operations (addition, multiplication,
  divison) in numerical literals. Subtraction can be done through a hack
  using addition and negatives (e.g. `0x8000 + -10`).
* The Tokenizer CLI can now take `-c` to indicate that arithmetic literals
  should be resolved
* Fixed bug in JSR
* Fixed underflow bug in SP
* Fixed minor tokenization alignment bug for hexadecimals
* Renamed `load_from_assembly_file` to `load_from_binary_file`
* Removed `get_data` from `Hardware` trait
* Added `as_any` to `Hardware` trait

## 0.0.7
Released: 2016-12-06
* Critical memory bug fix (memory of DCPU-16 was one word too short)
