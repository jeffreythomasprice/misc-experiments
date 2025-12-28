using System.Text.RegularExpressions;

public static class Pdf
{
    public static async Task<int> GetPageCount(string path)
    {
        var r = new Regex(@"^NumberOfPages:\s*([0-9]+)$");
        return (await Exec.Run("pdftk", [path, "dump_data"]))
            .Split("\n")
            .Select(s => s.Trim())
            .Select(s =>
            {
                var m = r.Match(s);
                if (m.Success && int.TryParse(m.Groups[1].Value, out var result))
                {
                    return result as int?;
                }
                return null;
            })
            .Single(x => x != null)!
            .Value;
    }

    public static async Task<string> ExtractPagesIntoNewFile(
        string inputPath,
        string outputDir,
        int firstPage,
        int lastPage
    )
    {
        Directory.CreateDirectory(outputDir);
        var outputFileName =
            $"{Path.GetFileNameWithoutExtension(inputPath)}-{firstPage}-{lastPage}{Path.GetExtension(inputPath)}";
        var outputPath = Path.Join(outputDir, outputFileName);
        await Exec.Run(
            "pdftk",
            [inputPath, "cat", $"{firstPage}-{lastPage}", "output", outputPath]
        );
        return outputPath;
    }

    public static async Task<string> GetText(string path)
    {
        return await Exec.Run("pdftotext", [path, "-"]);
    }
}
