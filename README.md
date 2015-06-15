# DCPU-16

DCPU-16 assembler, disassembler and emulator written in Rust 1.0.

More info about the DCPU-16:

* https://en.wikipedia.org/wiki/0x10c
* https://raw.githubusercontent.com/gatesphere/demi-16/master/docs/dcpu-specs/dcpu-1-7.txt

## Completed features

* Assembler
  * Labels
  * String literals
  * Good error messages
  * PUSH, POP and PEEK
* Disassembler (feature complete)
  * Separate tokenizer
  * Colorized output
* Emulator
  * Basic instructions
  * Conditionals
  * Jumps
  * Some hardware support

## Planned

* Assembler
  * PICK
  * Label and numeric literal addition (e.g. `[label + 10]`)
* Emulator
  * ADX, SBX, STI, STD
  * INT, IAG, IAS, RFI, IAQ
  * Better support for hardware

## Binaries

* assembler
  * `$ assembler program.asm -o program.bin`
* disassembler
  * `$ disassembler program.bin`
* tokenizer
  * `$ tokenizer program.bin`
* emulator
  * `$ emulator -p program.bin`

## Library

Apart from providing binaries, this crate can also be used as a library and embedded into other programs.

## Extentions

Some extensions (possibly temporary):

    --- Special opcodes: (5 bits) --------------------------------------------------
     C | VAL  | NAME  | DESCRIPTION
    ---+------+-------+-------------------------------------------------------------
     0 | 0x13 | OUT a | prints a null-terminated string located at a in memory
    ---+------+-------+-------------------------------------------------------------
    
Since hardware is not supported, you can use `OUT` to print to regular standard output. Another temporary behavior is that the CPU is terminated if it reads a `0x00` instruction.

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
