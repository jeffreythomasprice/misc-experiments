namespace Experiment;

using Microsoft.Extensions.Logging;

public sealed class TempFilePath(string extension = "tmp") : IDisposable
{
    private static readonly Lazy<ILogger> Log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger<TempFilePath>()
    );

    public readonly string Path = System.IO.Path.Join(
        System.IO.Path.GetTempPath(),
        System.IO.Path.ChangeExtension($"tmp{DateTime.UtcNow:yyyy-MM-ddTHH:mm:sszzz}", extension)
    );

    public void Dispose()
    {
        try
        {
            if (File.Exists(Path))
            {
                File.Delete(Path);
            }
        }
        catch (Exception e)
        {
            Log.Value.LogError(e, "failed to clean up temporary file at path {TempFilePath}", Path);
        }
    }
}
