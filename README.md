RustChip8 is a Rust interpreter and assembler for the Chip-8 virtual machine.
Essentially a rewrite of my previous project PyChip8.

[![Build Status](https://travis-ci.org/DavidSpickett/RustChip8.svg?branch=master)](https://travis-ci.org/DavidSpickett/RustChip8)

![INVADERS](/screenshots/invaders.png) <img height="10" hspace="10"/> ![BLINKY](/screenshots/blinky.png)

Examples
--------
```
  rchip8 -i roms/INVADERS 10 (interpret)
  
  rchip8 -a example.s example.0 (assemble)
```
    
Input
-----

|Chip8 keys  |RustChip8 keys|
|------------|------------|
|1	2	3	C|1 2 3 4     |
|4	5	6	D|q w e r     |
|7	8	9	E|a s d f     |
|A	0	B	F|z x c v     |

To exit press 'esc'.

Command Line Options
--------------------

```
rchip8 <mode> <file> <scaling factor (-i) / output file name (-a)>
```

* 'mode' is one of '-i' or 'a' for interpret or assemble.

* 'file' is a rom for interpret and an assembly file for assemble.

* 'scaling factor' increases the size of each Chip8 pixel.
e.g. 5 means that each of the Chip8's 64x32 pixels is drawn as
a 5x5 square.

* 'output file name' is the binary file that assembly results
are written to.

Assembler
---------

The assembler follows the ISA laid out in the Cowgod docs [1].
It does not do any relocations so the first instruction in the file
is assumed to be at 0x200, which is where the system is out of reset.

Syntax is fairly forgiving, mixed case is accepted for mnemonics and
register names. V registers can be specified with hex or decimal, although
the disassembler always uses decimal.

### Examples

* [Simple game](https://github.com/DavidSpickett/RustChip8/tree/master/asm_example)
* system/test.rs
* asm/test.rs

### Comments

Single line C style comments can be used like so.

```
//Comment on a new line
CLS // Comment on the end of a line
// Commenting something out
// ADD V0, V1
```

### Labels
These can be declared at any position. Meaning that you
can jump backward to a previous label, or forward to a label
declared later.

```
init:
  CLS
  JP game
<...data...>
game:
  <...logic...>
  SNE V0, 0x00
  JP init
<...data...>
end:
  JP END
```

Labels can be used with CALL, JP, and LD when setting the I register.

### 16 Bit Address Values

Mostly for fun and as an artifact of this implementation, the assembler
can emit a sequence to allow you to load a >12 bit address into the I register.

```
LD I, 0x1234
```

Assuming the address is > 0xFFF this will be transformed into...

```
LD I, 0xFFF
LD V14, 0xFF
ADD I, V14
<repeated>
LD V14, <remainder>
```

There's no real use for this and you have to trash V14 to get it,
but it was fun to think about.

### .word Directive

The .word directive allows you to insert arbitrary 16 bit values
into the assembly like so.

```
  JP game
sprite_data:
.word 0x1234
.word 0x5678
<...etc etc...>
```

With this you can embed data for sprites without needing to know
what address it will end up at.

References
----------

[1] http://devernay.free.fr/hacks/chip8/C8TECH10.HTM (Technical docs)

http://www.pong-story.com/chip8/ (Homebrew Roms)
