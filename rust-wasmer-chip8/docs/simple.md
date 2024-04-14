# The Basic Idea

Concerning an emulator that doesn't make use of AOT or JIT systems.

Define the "main loop" as a function that executes whlie the target CPU is not halted. It executes the instruction pointed to by the current
PC, leaving the PC pointing to the next instruction.

Define a "callback function" as any hook into the emulator that triggers based on some event. For example, a timer might trigger every so
many clock cycles.

# Interrupts

After each instruction check whether an interrupt has occurred. If it has, update the PC and any other state so that the next thing the
main loop does is execute the interrupt handler.

# Time

Keep a global that represents a number of clock ticks. Every time we execute an instruction we can increment that by the time spent on that
instruction.

The main function can take a target clock rate. When the total clock ticks reaches the right value, execute a callback. The callback can
then sleep for some amount of time to make the real time sync with the target time.

# Display Output

Depending on how video works on the target one of several strategies might be used.

If video is written in discrete chunks, e.g. a draw sprite instruction, then handling that instruction might call a callback function with
the sprite data.

If video is written on some interval then this could hook into the time system. The host function that executes periodically can be an
opportunity to copy from video memory to the actual display.

# Input

Instructions that wait for input can be implemented by blocking on an event, e.g. a mutex or a channel. Some other thread should either
always be running or spun up as necessary to provide that input.

Input registers, memory-mapped input devices, or reading from ports can all be implemented by having some other thread write to shared
memory.

Input that triggers interrupts can have the input thread write interrupt state to shared memory when the event occurs.

# Psuedocode

TODO update psuedocode to use better langage

```
fn main() {
	clockCycleCount = 0
	while not halted {
		exec(memory[PC])
		
		if an interrupt has occurred {
			handle interrupt, pushing PC on stack and setting to interrupt handler, etc.
		}

		add to clockCycleCount based on the type of instruction
		if clockCycleCount > clockCyclesPerExternalCallback {
			timeCallback(clockCycleCount)
			clockCycleCount -= clockCyclesPerExternalCallback
		}
	}
}

fn exec(instruction) {
	do whatever is necessary for this instruction
	illegal instructions halt with some error state
	most instructions set the PC to the start of the next instruction
	jumps set the PC to whereever they go
}
```