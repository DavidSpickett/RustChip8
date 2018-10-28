This is a simple game to demonstrate the assembler. Controls are the standard W,A,S,D (real keys, not Chip8 keys). You are the large square, collect 11 smaller squares to win.

Some limitations encountered when writing this example:
* Defining constants would be useful, or labelling registers
* Unused labels are accepted without warning
* Duplicate labels are accepted without warning (I had two 'wait_timer's at one point)
* A specific breakpoint instr would be nice, though you can at least trace by inserting invalid opcodes with .word
