namespace Robowar;

public enum ReadRegisterU64
{
	GeneralPurpose0,
	GeneralPurpose1,
	GeneralPurpose2,
	GeneralPurpose3,
	GeneralPurpose4,
	GeneralPurpose5,
	GeneralPurpose6,
	GeneralPurpose7,
}

public enum WriteRegisterU64
{
	GeneralPurpose0,
	GeneralPurpose1,
	GeneralPurpose2,
	GeneralPurpose3,
	GeneralPurpose4,
	GeneralPurpose5,
	GeneralPurpose6,
	GeneralPurpose7,
}

public enum ReadRegisterF64
{
	GeneralPurpose0,
	GeneralPurpose1,
	GeneralPurpose2,
	GeneralPurpose3,
	GeneralPurpose4,
	GeneralPurpose5,
	GeneralPurpose6,
	GeneralPurpose7,
	PositionX,
	PositionY,
	VelocityX,
	VelocityY,
	TurretAngle,
	TurretAngularVelocity,
	Health,
	Energy,
}

public enum WriteRegisterF64
{
	GeneralPurpose0,
	GeneralPurpose1,
	GeneralPurpose2,
	GeneralPurpose3,
	GeneralPurpose4,
	GeneralPurpose5,
	GeneralPurpose6,
	GeneralPurpose7,
	VelocityX,
	VelocityY,
	TurretAngularVelocity,
}

interface Instruction
{
	public record AddU64(WriteRegisterU64 Destination, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record AddF64(WriteRegisterF64 Destination, ReadRegisterF64 Left, ReadRegisterF64 Right) : Instruction;
	public record SubU64(WriteRegisterU64 Destination, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record SubF64(WriteRegisterF64 Destination, ReadRegisterF64 Left, ReadRegisterF64 Right) : Instruction;
	public record MulU64(WriteRegisterU64 Destination, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record MulF64(WriteRegisterF64 Destination, ReadRegisterF64 Left, ReadRegisterF64 Right) : Instruction;
	public record DivU64(WriteRegisterU64 Destination, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record DivF64(WriteRegisterF64 Destination, ReadRegisterF64 Left, ReadRegisterF64 Right) : Instruction;
	public record ModU64(WriteRegisterU64 Destination, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record ModF64(WriteRegisterF64 Destination, ReadRegisterF64 Left, ReadRegisterF64 Right) : Instruction;
	public record Jump(ReadRegisterU64 Target) : Instruction;
	public record JumpIfEqual(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record JumpIfNotEqual(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record JumpIfLessThan(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record JumpIfLessThanOrEqual(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record JumpIfGreaterThan(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
	public record JumpIfGreaterThanOrEqual(ReadRegisterU64 Target, ReadRegisterU64 Left, ReadRegisterU64 Right) : Instruction;
}
