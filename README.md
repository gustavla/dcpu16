[![Crates.io](https://img.shields.io/crates/v/dcpu16.svg)](https://crates.io/crates/dcpu16)

# DCPU-16

DCPU-16 assembler, disassembler and emulator written in Rust 1.0.

More info about the DCPU-16:

* https://en.wikipedia.org/wiki/0x10c
* https://raw.githubusercontent.com/gatesphere/demi-16/master/docs/dcpu-specs/dcpu-1-7.txt

To run DCPU-16 programs with hardware devices (such as a monitor), use:

* [dcpu16-gui](https://github.com/gustavla/dcpu16-gui)

## Completed features

* Assembler (feature complete)
  * Labels
  * String literals
  * Arithmetic literals (e.g. `SET A, 0x8000+100*3`)
  * Readable error messages
* Disassembler (feature complete)
  * Separate tokenizer
  * Colorized output
* Emulator
  * Basic instructions
  * Conditionals
  * Jumps
  * Interrupts
  * Some hardware support

## Planned

* Emulator
  * Better support for hardware

## Binaries

Run `cargo build --release` and add `dcpu16/target/release` to your `PATH`:

* assembler
  * `$ dcpu16-assembler program.asm -o program.bin`
* disassembler
  * `$ dcpu16-disassembler program.bin`
* tokenizer
  * `$ dcpu16-tokenizer program.bin`
* emulator
  * `$ dcpu16 -p program.bin`

## Library

Apart from providing binaries, this crate can also be used as a library and
embedded into other programs. An example of this can be seen in
[dcpu16-gui](https://github.com/gustavla/dcpu16-gui).

## Extentions

Some extensions (possibly temporary):

    --- Special opcodes: (5 bits) --------------------------------------------------
     C | VAL  | NAME  | DESCRIPTION
    ---+------+-------+-------------------------------------------------------------
     0 | 0x13 | OUT a | prints a null-terminated string located at a in memory
     0 | 0x14 | OUV a | prints a value in decimal and newline
    ---+------+-------+-------------------------------------------------------------

Since hardware is not supported, you can use `OUT` to print to regular standard
output. Another temporary behavior is that the CPU is terminated if it reads a
`0x00` instruction.

Extensions to the assembler:

    -- Assembler instructions ------------------------------------------------------
     FORMAT       | DESCRIPTION
    --------------+-----------------------------------------------------------------
     DAF c, v     | DATA FILL - repeats a value a certain number of times
                  | c (count) and v (value) must be numerical literals
                  | e.g. DAF 256, 0xffff  ; Fill 256 words with -1
    --------------+-----------------------------------------------------------------

## Example

Save the following as `prog.dasm16`:

                OUT hello                   ; Print the string defined at 'hello'
                DAT 0                       ; This will terminate the program    

    :hello      DAT "Hello World!\n", 0

Assemble the program:

    $ assembler prog.dasm16 -o prog.bin

Run it:

    $ emulator prog.bin
    Hello World!
