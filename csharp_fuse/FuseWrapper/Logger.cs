using Microsoft.Extensions.Logging;

namespace Experiment.FuseWrapper;

internal class Logger : IDisposable
{
	public readonly IntPtr Handle;

	public Logger(ILogger logger)
	{
		this.Handle = Natives.CreateLogger(
			(s) => logger.LogTrace(s),
			(s) => logger.LogDebug(s),
			(s) => logger.LogInformation(s),
			(s) => logger.LogWarning(s),
			(s) => logger.LogError(s),
			(s) => logger.LogCritical(s)
		);
	}

	public void Dispose()
	{
		Natives.FreeLogger(Handle);
	}
}