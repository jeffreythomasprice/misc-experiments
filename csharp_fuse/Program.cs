namespace Experiment;

using System.Runtime.InteropServices;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

internal class Program
{
	private static void Main(string[] args)
	{
		using var serviceProvider = new ServiceCollection().AddLogging(builder =>
		{
			builder
				.AddSimpleConsole(options =>
				{
					options.SingleLine = true;
					options.TimestampFormat = "yyyy-mm-ddTHH:MM:sszzzz ";
				})
				.AddFile("log/output.log", options =>
				{
					options.Append = true;
					options.MaxRollingFiles = 7;
					options.FileSizeLimitBytes = 1024 * 1024 * 100;
				})
				.AddFilter("Experiment", LogLevel.Trace)
				.AddFilter("Microsoft", LogLevel.Warning)
				.AddFilter("System", LogLevel.Warning);
		})
			.BuildServiceProvider();

		var logger = serviceProvider.GetService<ILogger<Program>>()!;

		var cancelTokenSource = new CancellationTokenSource();
		using var _ = PosixSignalRegistration.Create(PosixSignal.SIGINT, (context) =>
		{
			logger.LogDebug($"handling {context.Signal}");
			context.Cancel = true;
			cancelTokenSource.Cancel();
		});

		FuseWrapper.Executor.Mount(
			serviceProvider,
			"experiment",
			"/home/jeff/mount_points/test",
			new FuseWrapper.Natives.FuseOperations
			{
				// TODO implement some real stuff
				getattr = (path, stat) =>
				{
					logger.LogDebug($"TODO getattr {path}");
					return -FuseWrapper.Natives.ENOENT;
				},
				readdir = (string path, IntPtr data, FuseWrapper.Natives.FuseFillDirFunc callback, Int64 off, ref FuseWrapper.Natives.FuseFileInfo info) =>
				{
					logger.LogDebug($"TODO readdir {path}");
					return -FuseWrapper.Natives.ENOENT;
				},
			},
			cancelTokenSource.Token
		);

		logger.LogDebug("program exit");

		// disposing of the log service is supposed to flush but it doesn't
		// writing an unprintable character to stdout seems to force that stream to flush though
		Console.Write("\0");
	}
}