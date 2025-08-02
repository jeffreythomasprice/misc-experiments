implemented in javascript, so the data type is "number", i.e. 64-bit numbers
e.g. bit shifts force all operands to unsigned 32-bit integers, and return an unsigned 32-bit result

there are 8 general purpose registers, r0 through r7
there are the following special purpose registers:
positionX (read only)
positionY (read only)
velocityX
velocityY
turretAngle
radarAngle
radarDistance (read only)
health (read only)
energy (read only)

there is a stack with a configurable max size, which holds numbers

there is a heap of configurable size, which holds numbers
each address is a whole number; individual bytes are not addressable

when an argument is a literal the next number is treated as the number
when an argument is a register the next number is treated as:
0 through 7 = general purpose register 0 through 7
any other number = fault

instruction set syntax:
rX = register X, e.g. r0 = first register, r1 = second register, etc.
[rX] = memory at address pointed to by rX
i = immediate, i.e. a numeric literal, or an identifier that evaluates to a constant value, e.g. a label

opcode - instruction - description
00 - nop - do nothing
01 - halt - force halt
03 - set [rd], rs - put value of register rs into memory at address in register rd
03 - set rd, [rs] - put value of memory at address in register rs into register rd
04 - set rd, i - put value of immediate i into register rd
05 - add rd, ra, rb - set register rd = register a + register b
06 - sub rd, ra, rb - set register rd = register a - register b
07 - mul rd, ra, rb - set register rd = register a * register b
08 - div rd, ra, rb - set register rd = register a / register b
09 - mod rd, ra, rb - set register rd = register a % register b
0a - shl rd, ra, rb - set register rd = register a left shifted by register b bits
0b - shr rd, ra, rb - set register rd = register a right shifted by register b bits
0c - jump rd - jump to address in register rd
0d - jeq rd, ra, rb - jump to address in register rd if register a == register b
0e - jne rd, ra, rb - jump to address in register rd if register a != register b
0f - jlt rd, ra, rb - jump to address in register rd if register a < register b
10 - jle rd, ra, rb - jump to address in register rd if register a <= register b
11 - jgt rd, ra, rb - jump to address in register rd if register a > register b
12 - jge rd, ra, rb - jump to address in register rd if register a >= register b
13 - push rx - push value in register x to the stack
14 - pop rx - pop value from stack and put in register x
15 - fire rx - fire a bullet with energy in register x

labels are identifiers followed by ":"
