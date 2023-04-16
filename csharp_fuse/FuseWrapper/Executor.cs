using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Experiment.FuseWrapper;

internal static class Executor
{
	/// <summary>
	/// Blocks until the mount point is unmounted or the given cancellation token is cancelled.
	/// </summary>
	/// <param name="serviceProvider"></param>
	/// <param name="mountName">for humans, the name that shows up when you run 'mount'</param>
	/// <param name="mountPoint">the directory path to mount to</param>
	/// <param name="cancellationToken"></param>
	/// <exception cref="Exception"></exception>
	public static void Mount(
		ServiceProvider serviceProvider,
		string mountName,
		string mountPoint,
		Natives.FuseOperations ops,
		CancellationToken cancellationToken = default
	)
	{
		var logger = serviceProvider.GetService<ILogger<Program>>()!;
		using var cLibLogger = new FuseWrapper.Logger(serviceProvider.GetService<ILoggerFactory>()!.CreateLogger($"{typeof(Program)}.c-lib"));

		var fuseData = IntPtr.Zero;

		var exited = false;
		var exitOnce = () =>
		{
			if (!exited)
			{
				exited = true;
				Natives.UnmountAndExit(cLibLogger.Handle, fuseData);
			}
		};

		var cancelTokenSource = new CancellationTokenSource();
		var cancelThread = new Thread(() =>
		{
			cancellationToken.WaitHandle.WaitOne();
			exitOnce();
		});
		cancelThread.Start();

		var result = Natives.MountAndRun(
			cLibLogger.Handle,
			3,
			new[] {
				mountName,
				mountPoint,
				// foreground mode
				"-f"
			},
			ref ops,
			(data) =>
			{
				fuseData = data;
			}
		);

		// if we haven't been intentionally exited the mount point might have been unmounted externally
		// make sure we clean up and stop waiting
		exitOnce();

		cancelThread.Join();

		if (result != 0)
		{
			// TODO custom exception type
			throw new Exception($"fuse exit code was non-0: {result}");
		}
	}
}