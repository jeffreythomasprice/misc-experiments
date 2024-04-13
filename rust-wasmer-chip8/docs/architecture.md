# The Basic Idea

Define "target" as the device being emulated.

Define "host" as the device or environment running the emulator.

Define "wasm" as the webassembly environment.

The goal is to turn a program for some target device into a wasm program that executes on the host device via a wasm runtime.

Define an "instruction sequence" as one or more instructions in the target instruction set.

Define an "execute function" as a wasm function that executes an instruction sequence.
- For each instruction in the sequence equivalent wasm instructions will be executed to operate on globals that represent the various
registers.
- It's likely that this needs no input or output, since it expects to operate on registers.
- It's expected that this function will leave the PC pointing to the next instruction.

An execution function is defined for any given address by taking the instruction sequence starting at that address and ending at the first
instruction that is any of:
- HALT
- illegal instruction
- jump of any kind, conditional or always, relative or absolute

Define the "jump table" as a mapping of addresses to execute functions.

Define the "main function" as a wasm function that loops while the target CPU is not halted, and while the current PC is in the jump table.
Every loop it executes the execution function for the current PC.

The host will hold a jump table that starts empty. It will implement a loop where it assembles the full program and executes it. The full
program includes:
- the current jump table
- an exported main function

Every time the main function returns the host checks if the CPU is halted. If not, it adds a new execution function to the jump table.

# Psuedocode

```
// wasm

// entries in the jump table all look like this
fn exec_fn_addr() {
	// execute each instruction in sequence
	// make sure PC and whether the target CPU is halted are set by the end
}

fn main() {
	while target CPU is not halted and jumpTable[PC] exists {
		jumpTable[PC]()
	}
}
```

```
// host
fn run() {
	jumpTable = {}
	while target CPU is not halted {
		jumpTable[PC] = build a new execution function
		wasmBinary = build from jumpTable
		exec main on wasmBinary
	}
}
```

# Interrupts

Each instruction in an execution function can be interleaved with interrupt code. This can make sure to update the PC or other flags as
needed to make the main loop execute an interrupt handler next.

If an interrupt is triggered by some automatic process that executes every clock tick (e.g. clock tick timers or similar) then the
interleaved code can include whatever auto increment or decrements are necessary to trigger interrupts, as well as the check for whether it
has triggered.

If an interrupt is triggered by some outside process, e.g. input from the host, then the host can simply update the wasm memory from another
thread. By the next instruction the relevant flag will be set and the next check in an execution function will see it.

TODO safety issues assocciated with shared memory on the host and wasm?

# Display Output

Depending on how video works on the target one of several strategies might be used.

If video is written in discrete chunks, e.g. a draw sprite instruction, then handling that instruction might call a host function with the
sprite data.

If video is written in scan lines then a host function will be called when a line is complete or on some interval. The address of
the start of the line in video memory will be passed.

Separately a host function can be called on an interval to trigger a screen refresh. This makes it easy to implement double buffering.

# Input

Instructions that wait for input can be implemented by returning from the main function. i.e. this can be a terminal condition for an
instruction sequence, and the host loop can check for whether it's waiting on input.

Input registers, memory-mapped input devices, or reading from ports can all be implemented by having the host write to shared memory.

Input that triggers interrupts can have the host write interrupt state to shared memory when the even occurs.
