namespace Experiment;

using System.Reflection.Metadata;
using Microsoft.Extensions.Logging;

public static class LoggerUtils
{
    public static readonly Lazy<ILoggerFactory> Factory = new(() =>
    {
        return LoggerFactory.Create(builder =>
        {
            builder
                .SetMinimumLevel(LogLevel.Trace)
                .AddSimpleConsole(options =>
                {
                    options.IncludeScopes = true;
                    options.SingleLine = true;
                    options.TimestampFormat = "yyyy-MM-ddTHH:mm:sszzz ";
                });
        });
    });
}
