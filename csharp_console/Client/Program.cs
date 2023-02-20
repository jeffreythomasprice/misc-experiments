using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

using var logger = new StreamWriter("log", append: true);
try
{
	using var screen = new Screen(logger);
	logger.WriteLine($"size = {screen.Size}");
	logger.WriteLine($"cursor = {screen.Cursor}");

	screen.Cursor = new(Row: 1, Column: 1);
	logger.WriteLine($"cursor = {screen.Cursor}");

	screen.WaitForEnter();
	logger.WriteLine("done");
}
catch (Exception e)
{
	logger.WriteLine($"oops {e}");
}

readonly record struct Size(
	int Rows,
	int Columns
);

readonly record struct Position(
	int Row,
	int Column
);

static class LibC
{
	[StructLayout(LayoutKind.Sequential)]
	public struct Winsize
	{
		public ushort ws_row;
		public ushort ws_col;
		public ushort ws_xpixel;
		public ushort ws_ypixel;
	}

	[StructLayout(LayoutKind.Sequential)]
	public struct Termios
	{
		public uint c_iflag;
		public uint c_oflag;
		public uint c_cflag;
		public uint c_lflag;
		public byte c_line;
		[MarshalAs(UnmanagedType.ByValArray, SizeConst = NCCS)]
		public byte[] c_cc;
		public uint c_ispeed;
		public uint c_ospeed;

		public Termios()
		{
			c_cc = new byte[NCCS];
		}
	}

	public const int NCCS = 32;

	public const int STDIN_FILENO = 0;
	public const int STDOUT_FILENO = 1;
	public const int STDERR_FILENO = 1;

	public const int TIOCGWINSZ = 21523;

	public const uint ISIG = 1;
	public const uint ICANON = 2;
	public const uint ECHO = 8;

	public const uint OPOST = 1;

	public const uint BRKINT = 2;
	public const uint IGNPAR = 4;
	public const uint ISTRIP = 32;
	public const uint ICRNL = 256;
	public const uint IXON = 1024;

	public const uint VTIME = 5;
	public const uint VMIN = 6;

	public const uint TCSANOW = 0;
	public const uint TCSADRAIN = 1;

	[DllImport("libc", EntryPoint = "ioctl", SetLastError = true)]
	public static extern int Ioctl(IntPtr handle, uint request, ref Winsize destination);

	[DllImport("libc", EntryPoint = "tcgetattr", SetLastError = true)]
	public static extern int Tcgetattr(IntPtr handle, ref Termios termios);

	[DllImport("libc", EntryPoint = "tcsetattr", SetLastError = true)]
	public static extern int Tcsetattr(IntPtr handle, uint optionalActions, ref Termios termios);

	[DllImport("libc", EntryPoint = "getchar", SetLastError = true)]
	public static extern int Getchar();
}

class Screen : IDisposable
{
	private TextWriter logger;
	private Stream stdout;

	private LibC.Termios backupTermios;

	public Screen(TextWriter logger)
	{
		this.logger = logger;
		stdout = Console.OpenStandardOutput();

		backupTermios = Termios;

		SetRaw();
	}

	public void Dispose()
	{
		try
		{
			SetCooked();
		}
		finally
		{
			stdout.Close();
			Console.Out.WriteLine();
		}
	}

	public Size Size
	{
		get
		{
			var result = IoctlWinsize;
			return new(result.ws_row, result.ws_col);
		}
	}

	public Position Cursor
	{
		get
		{
			stdout.Write(new byte[] { 0x1b, (byte)'[', (byte)'6', (byte)'n' });
			Expect(new byte[] { 0x1b, (byte)'[' });
			var row = int.Parse(Encoding.ASCII.GetString(ExpectUntil((byte)';', 4)));
			var column = int.Parse(Encoding.ASCII.GetString(ExpectUntil((byte)'R', 4)));
			return new(Row: row, Column: column);
		}
		set
		{
			stdout.WriteByte(0x1b);
			stdout.Write(Encoding.ASCII.GetBytes($"[{value.Row};{value.Column}H"));
		}
	}

	public void WaitForEnter()
	{
		// TODO JEFF bring sanity to this
		while (true)
		{
			logger.WriteLine($"TODO JEFF about to call ReadByte");
			logger.Flush();
			var next = ReadByte();
			logger.WriteLine($"TODO JEFF {Convert.ToString(next, 16).PadLeft(2, '0')}");
			logger.Flush();
			if (next == '\r')
			{
				break;
			}
		}
		// while (ReadByte() != (byte)'\r') { }
	}

	private byte ReadByte()
	{
		return (byte)LibC.Getchar();
	}

	private void Expect(byte value)
	{
		var result = ReadByte();
		if (result != value)
		{
			// TODO special exception type
			throw new Exception($"expected {Convert.ToString(value, 16).PadLeft(2, '0')} but got {Convert.ToString(result, 16).PadLeft(2, '0')}");
		}
	}

	private void Expect(byte[] values)
	{
		foreach (var value in values)
		{
			Expect(value);
		}
	}

	private byte[] ExpectUntil(byte separator, int max)
	{
		var results = new List<byte>(max);
		while (results.Count < max)
		{
			var value = ReadByte();
			if (value == separator)
			{
				break;
			}
			results.Add((byte)value);
		}
		return results.ToArray();
	}

	private LibC.Winsize IoctlWinsize
	{
		get
		{
			LibC.Winsize result = new();
			int ioctlResult = LibC.Ioctl(new IntPtr(LibC.STDIN_FILENO), LibC.TIOCGWINSZ, ref result);
			if (ioctlResult != 0)
			{
				// TODO special exception type
				throw new Exception($"ioctl failed: {ioctlResult}");
			}
			return result;
		}
	}

	private LibC.Termios Termios
	{
		get
		{
			LibC.Termios result = new();
			int tcgetattrResult = LibC.Tcgetattr(new IntPtr(LibC.STDIN_FILENO), ref result);
			if (tcgetattrResult != 0)
			{
				// TODO special exception type
				throw new Exception($"tcgetattr failed: {tcgetattrResult}");
			}
			return result;
		}
	}

	private void Tcsetattr(uint optionalActions, LibC.Termios termios)
	{
		int result = LibC.Tcsetattr(0, optionalActions, ref termios);
		if (result != 0)
		{
			// TODO special exception type
			throw new Exception($"tcsetattr failed: {result}");
		}
	}

	private void SetRaw()
	{
		// set raw mode
		// https://github.com/wertarbyte/coreutils/blob/master/src/stty.c#L1180
		var termios = Termios;
		termios.c_iflag = 0;
		termios.c_oflag &= ~LibC.OPOST;
		termios.c_lflag &= ~(LibC.ISIG | LibC.ICANON | LibC.ECHO);
		termios.c_cc[LibC.VMIN] = 1;
		termios.c_cc[LibC.VTIME] = 0;
		Tcsetattr(LibC.TCSADRAIN, termios);
	}

	private void SetCooked()
	{
		// set cooked mode
		// https://github.com/wertarbyte/coreutils/blob/master/src/stty.c#L1167
		// TODO do we need a backup or just get current termios again?
		var termios = backupTermios;
		termios.c_iflag |= LibC.BRKINT | LibC.IGNPAR | LibC.ISTRIP | LibC.ICRNL | LibC.IXON;
		termios.c_oflag |= LibC.OPOST;
		termios.c_lflag |= LibC.ISIG | LibC.ICANON | LibC.ECHO;
		Tcsetattr(LibC.TCSANOW, termios);
	}
}