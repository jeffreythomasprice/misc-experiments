namespace Experiment;

using System.Diagnostics;
using Microsoft.Extensions.Logging;

public static class Exec
{
    private static readonly Lazy<ILogger> log = new(() =>
        LoggerUtils.Factory.Value.CreateLogger(typeof(Exec).ToString())
    );

    public static string Run(string command, IEnumerable<string> arguments)
    {
        using var process = new Process();
        process.StartInfo.UseShellExecute = false;
        process.StartInfo.CreateNoWindow = true;
        process.StartInfo.RedirectStandardOutput = true;
        process.StartInfo.RedirectStandardError = true;
        process.StartInfo.WindowStyle = ProcessWindowStyle.Hidden;
        process.StartInfo.FileName = command;
        foreach (var arg in arguments)
        {
            process.StartInfo.ArgumentList.Add(arg);
        }
        var stdout = "";
        var stderr = "";
        process.OutputDataReceived += (sender, args) =>
        {
            stdout += args.Data + "\n";
        };
        process.ErrorDataReceived += (sender, args) =>
        {
            stderr += args.Data + "\n";
        };
        process.Start();
        process.BeginOutputReadLine();
        process.BeginErrorReadLine();
        process.WaitForExit();
        log.Value.LogTrace(
            "process {Command} [{Arguments}] exited with code {ExitCode}",
            command,
            arguments,
            process.ExitCode
        );
        if (process.ExitCode != 0)
        {
            log.Value.LogError(
                "process {Command} [{Arguments}] stderr:\n{Stderr}",
                command,
                arguments,
                stderr
            );
            throw new Exception(
                $"process {command} exited with non-0 exit code {process.ExitCode}"
            );
        }
        return stdout;
    }
}
