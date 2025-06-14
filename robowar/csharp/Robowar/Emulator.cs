using System.Collections;
using System.Runtime.CompilerServices;
using System.Security.Permissions;

namespace Robowar;

public enum Instruction
{
	Nop = 0,
	AddI32_Register_Register,
	AddI32_Register_Literal,
	SubI32_Register_Register,
	SubI32_Register_Literal,
	MulI32_Register_Register,
	MulI32_Register_Literal,
	DivI32_Register_Register,
	DivI32_Register_Literal,
	// TODO push/pop
	// TODO compares
	// TODO jumps
}

public class ExecutionHaltException(string message) : Exception(message) { }

public class UnrecognizedInstructionException(Instruction i) : ExecutionHaltException($"unrecognized instruction: {(int)i}") { }

public class Emulator
{
	private readonly byte[] program;
	private int programPointer;

	private readonly byte[] stack;
	private int stackPointer;

	private Int32[] registersI32;

	private UInt64 clock;

	private bool halted;
	private string? haltReason;

	public Emulator(byte[] program)
	{
		this.program = program;
		programPointer = 0;

		stack = new byte[65536];
		stackPointer = 0;

		registersI32 = new Int32[8];

		clock = 0;

		halted = false;
		haltReason = null;
	}

	public void ExecuteNextInstruction()
	{
		try
		{
			var instruction = ReadInstruction();
			switch (instruction)
			{
				case Instruction.Nop:
					break;
				case Instruction.AddI32_Register_Register:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = registersI32[ReadInstructionDataRegisterI32Index()];
						registersI32[destinationIndex] = left + right;
						clock += 3;
					}
					break;
				case Instruction.AddI32_Register_Literal:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = ReadInstructionDataI32();
						registersI32[destinationIndex] = left + right;
						clock += 3;
					}
					break;
				case Instruction.SubI32_Register_Register:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = registersI32[ReadInstructionDataRegisterI32Index()];
						registersI32[destinationIndex] = left - right;
						clock += 3;
					}
					break;
				case Instruction.SubI32_Register_Literal:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = ReadInstructionDataI32();
						registersI32[destinationIndex] = left + right;
						clock += 3;
					}
					break;
				case Instruction.MulI32_Register_Register:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = registersI32[ReadInstructionDataRegisterI32Index()];
						registersI32[destinationIndex] = left * right;
						clock += 3;
					}
					break;
				case Instruction.MulI32_Register_Literal:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = ReadInstructionDataI32();
						registersI32[destinationIndex] = left * right;
						clock += 3;
					}
					break;
				case Instruction.DivI32_Register_Register:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = registersI32[ReadInstructionDataRegisterI32Index()];
						registersI32[destinationIndex] = left / right;
						clock += 3;
					}
					break;
				case Instruction.DivI32_Register_Literal:
					{
						var destinationIndex = ReadInstructionDataRegisterI32Index();
						var left = registersI32[ReadInstructionDataRegisterI32Index()];
						var right = ReadInstructionDataI32();
						registersI32[destinationIndex] = left / right;
						clock += 3;
					}
					break;
				default:
					throw new UnrecognizedInstructionException(instruction);
			}
		}
		catch (ExecutionHaltException e)
		{
			halted = true;
			haltReason = e.Message;
		}
	}

	private Instruction ReadInstruction() => (Instruction)ReadInstructionDataU8();

	private int ReadInstructionDataRegisterI32Index()
	{
		var result = (int)ReadInstructionDataU8();
		if (result > registersI32.Length)
		{
			throw new IndexOutOfRangeException($"no such i32 register: {result}");
		}
		return result;
	}

	private byte ReadInstructionDataU8() => ReadU8(program, ref programPointer);

	private Int32 ReadInstructionDataI32() => ReadI32(program, ref programPointer);

	private void PushI32(Int32 value) => WriteI32(stack, ref stackPointer, value);

	private Int32 PopI32() => ReadI32(stack, ref stackPointer);

	private static byte ReadU8(byte[] memory, ref int pointer)
	{
		if (pointer - sizeof(byte) < 0)
		{
			throw new ExecutionHaltException($"can't read byte, not enough bytes left in memory");
		}
		pointer -= sizeof(byte);
		unsafe
		{
			fixed (byte* p = &memory[pointer])
			{
				return *p;
			}
		}
	}

	private static Int32 ReadI32(byte[] memory, ref int pointer)
	{
		if (pointer - sizeof(Int32) < 0)
		{
			throw new ExecutionHaltException($"can't read Int32, not enough bytes left in memory");
		}
		pointer -= sizeof(Int32);
		unsafe
		{
			fixed (byte* p = &memory[pointer])
			{
				return *(Int32*)p;
			}
		}
	}

	private static void WriteI32(byte[] memory, ref int pointer, Int32 value)
	{
		if (pointer + sizeof(Int32) > memory.Length)
		{
			throw new ExecutionHaltException($"can't write Int32, not enough bytes left in memory");
		}
		unsafe
		{
			fixed (byte* p = &memory[pointer])
			{
				*(Int32*)p = value;
			}
		}
		pointer += sizeof(Int32);
	}
}