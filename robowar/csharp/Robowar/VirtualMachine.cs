namespace Robowar;

public class VirtualMachine
{
	private UInt64[] generalPurposeRegistersU64 = new UInt64[8];
	private double[] generalPurposeRegistersF64 = new double[8];
	private Vector2<double> position = new(0, 0);
	private Vector2<double> velocity = new(0, 0);
	// TODO should be angle type
	private double turretAngle = 0;
	// TODO should be angle type
	private double turretAngularVelocity = 0;
	private double health = 0;
	private double energy = 0;

	public void Step()
	{
		throw new NotImplementedException("VirtualMachine.Step is not implemented yet.");
	}

	private UInt64 GetRegisterValue(ReadRegisterU64 register)
	{
		return register switch
		{
			ReadRegisterU64.GeneralPurpose0 => generalPurposeRegistersU64[0],
			ReadRegisterU64.GeneralPurpose1 => generalPurposeRegistersU64[1],
			ReadRegisterU64.GeneralPurpose2 => generalPurposeRegistersU64[2],
			ReadRegisterU64.GeneralPurpose3 => generalPurposeRegistersU64[3],
			ReadRegisterU64.GeneralPurpose4 => generalPurposeRegistersU64[4],
			ReadRegisterU64.GeneralPurpose5 => generalPurposeRegistersU64[5],
			ReadRegisterU64.GeneralPurpose6 => generalPurposeRegistersU64[6],
			ReadRegisterU64.GeneralPurpose7 => generalPurposeRegistersU64[7],
			_ => throw new ArgumentOutOfRangeException(nameof(register), "Invalid register"),
		};
	}

	private void SetRegisterValue(WriteRegisterU64 register, UInt64 value)
	{
		switch (register)
		{
			case WriteRegisterU64.GeneralPurpose0:
				generalPurposeRegistersU64[0] = value;
				break;
			case WriteRegisterU64.GeneralPurpose1:
				generalPurposeRegistersU64[1] = value;
				break;
			case WriteRegisterU64.GeneralPurpose2:
				generalPurposeRegistersU64[2] = value;
				break;
			case WriteRegisterU64.GeneralPurpose3:
				generalPurposeRegistersU64[3] = value;
				break;
			case WriteRegisterU64.GeneralPurpose4:
				generalPurposeRegistersU64[4] = value;
				break;
			case WriteRegisterU64.GeneralPurpose5:
				generalPurposeRegistersU64[5] = value;
				break;
			case WriteRegisterU64.GeneralPurpose6:
				generalPurposeRegistersU64[6] = value;
				break;
			case WriteRegisterU64.GeneralPurpose7:
				generalPurposeRegistersU64[7] = value;
				break;
			default:
				throw new ArgumentOutOfRangeException(nameof(register), "Invalid register");
		}
	}

	private double GetRegisterValue(ReadRegisterF64 register)
	{
		return register switch
		{
			ReadRegisterF64.GeneralPurpose0 => generalPurposeRegistersF64[0],
			ReadRegisterF64.GeneralPurpose1 => generalPurposeRegistersF64[1],
			ReadRegisterF64.GeneralPurpose2 => generalPurposeRegistersF64[2],
			ReadRegisterF64.GeneralPurpose3 => generalPurposeRegistersF64[3],
			ReadRegisterF64.GeneralPurpose4 => generalPurposeRegistersF64[4],
			ReadRegisterF64.GeneralPurpose5 => generalPurposeRegistersF64[5],
			ReadRegisterF64.GeneralPurpose6 => generalPurposeRegistersF64[6],
			ReadRegisterF64.GeneralPurpose7 => generalPurposeRegistersF64[7],
			ReadRegisterF64.PositionX => position.X,
			ReadRegisterF64.PositionY => position.Y,
			ReadRegisterF64.VelocityX => velocity.X,
			ReadRegisterF64.VelocityY => velocity.Y,
			ReadRegisterF64.TurretAngle => turretAngle,
			ReadRegisterF64.TurretAngularVelocity => turretAngularVelocity,
			ReadRegisterF64.Health => health,
			ReadRegisterF64.Energy => energy,
			_ => throw new ArgumentOutOfRangeException(nameof(register), "Invalid register"),
		};
	}

	private void SetRegisterValue(WriteRegisterF64 register, double value)
	{
		switch (register)
		{
			case WriteRegisterF64.GeneralPurpose0:
				generalPurposeRegistersF64[0] = value;
				break;
			case WriteRegisterF64.GeneralPurpose1:
				generalPurposeRegistersF64[1] = value;
				break;
			case WriteRegisterF64.GeneralPurpose2:
				generalPurposeRegistersF64[2] = value;
				break;
			case WriteRegisterF64.GeneralPurpose3:
				generalPurposeRegistersF64[3] = value;
				break;
			case WriteRegisterF64.GeneralPurpose4:
				generalPurposeRegistersF64[4] = value;
				break;
			case WriteRegisterF64.GeneralPurpose5:
				generalPurposeRegistersF64[5] = value;
				break;
			case WriteRegisterF64.GeneralPurpose6:
				generalPurposeRegistersF64[6] = value;
				break;
			case WriteRegisterF64.GeneralPurpose7:
				generalPurposeRegistersF64[7] = value;
				break;
			case WriteRegisterF64.VelocityX:
				velocity.X = value;
				break;
			case WriteRegisterF64.VelocityY:
				velocity.Y = value;
				break;
			case WriteRegisterF64.TurretAngularVelocity:
				turretAngularVelocity = value;
				break;
			default:
				throw new ArgumentOutOfRangeException(nameof(register), "Invalid register");
		}
	}
}