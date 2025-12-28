using System.Diagnostics;

public static class Exec
{
    public static async Task<string> Run(string command, IEnumerable<string> arguments)
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
        await process.WaitForExitAsync();
        if (process.ExitCode != 0)
        {
            Console.Error.WriteLine($"process {command} stderr:\n{stderr}");
            throw new Exception(
                $"process {command} exited with non-0 exit code {process.ExitCode}"
            );
        }
        return stdout;
    }
}
