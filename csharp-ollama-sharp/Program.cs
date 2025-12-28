using System.IO.Pipelines;
using dotenv.net;
using Experiment;
using Microsoft.Extensions.AI;

// TODO use proper logging

static async Task<(string, ReadOnlyMemory<float>, int)> CreateNextChunkEmbedding(
    Db db,
    EmbeddingLlm llm,
    string inputPath,
    string outputDir,
    string key,
    int firstPage,
    int maxChunkPageCount
)
{
    var pageCount = await Pdf.GetPageCount(inputPath);
    var lastPage = Math.Min(firstPage + maxChunkPageCount, pageCount);
    while (true)
    {
        var currentPageCount = lastPage - firstPage + 1;

        var chunkPath = await Pdf.ExtractPagesIntoNewFile(
            inputPath,
            outputDir,
            firstPage,
            lastPage
        );
        Console.WriteLine($"chunk path: {chunkPath}, page range: {firstPage}-{lastPage}");
        var chunkTextContent = await Pdf.GetText(chunkPath);
        // TODO do we really need the augmented content thing?
        var embeddingText =
            $"key: {key}\nfirst page: {firstPage}\nlast page: {lastPage}\ncontent: {chunkTextContent}";
        var embedding = await llm.CreateEmbedding(embeddingText);
        await db.Insert(new(llm.ModelId, key, firstPage, lastPage, embeddingText, embedding));
        return (embeddingText, embedding, lastPage);
    }
}

static async Task<string> ChunkPdf(
    Db db,
    EmbeddingLlm llm,
    string inputPath,
    string outputDir,
    int maxChunkPageCount
)
{
    Console.WriteLine($"chunking pdf: {inputPath}");
    Console.WriteLine($"output dir: {outputDir}");
    var pageCount = await Pdf.GetPageCount(inputPath);
    Console.WriteLine($"page count: {pageCount}");

    var key = inputPath;

    // skip if we already have all pages
    var existing = await db.FindAllByKey(llm.ModelId, key).ToListAsync();
    Console.WriteLine($"found {existing.Count} existing document chunks");
    var (firstExistingPage, lastExistingPage) = existing.Aggregate(
        (int.MaxValue, 0),
        (totals, e) =>
        {
            var (first, last) = totals;
            return (Math.Min(first, e.FirstPage), Math.Max(last, e.LastPage));
        }
    );
    Console.WriteLine($"existing page range: {firstExistingPage}-{lastExistingPage}");
    if (firstExistingPage == 1 && lastExistingPage == pageCount)
    {
        Console.WriteLine("existing page range appears to be covered in db, skipping embedding");
    }
    else
    {
        var firstPage = 1;
        while (true)
        {
            var lastPage = existing.Find(e => e.FirstPage == firstPage)?.LastPage;
            if (lastPage.HasValue)
            {
                Console.WriteLine(
                    $"skipping pages for key: {key}, existing page range {firstPage}-{lastPage.Value}"
                );
            }
            else
            {
                // we need to actually generate this chunk
                var (textContent, embedding, actualLastPage) = await CreateNextChunkEmbedding(
                    db,
                    llm,
                    inputPath,
                    outputDir,
                    key,
                    firstPage,
                    maxChunkPageCount
                );
                Console.WriteLine(
                    $"successfully created embedding for key: {key}, page range {firstPage}-{actualLastPage}, text content len: {textContent.Length}, embedding len: {embedding.Length}"
                );
                lastPage = actualLastPage;
            }
            firstPage = Math.Max(lastPage.Value - 1, firstPage + 1);
            if (lastPage.Value >= pageCount)
            {
                break;
            }
        }
    }
    return key;
}

DotEnv.Load();

await using var db = await Db.Create(
    Env.AssertString("PG_HOST"),
    Env.AssertInt("PG_PORT"),
    Env.AssertString("PG_USERNAME"),
    Env.AssertString("PG_PASSWORD"),
    Env.AssertString("PG_DATABASE")
);

var llmProvider = Llm.Provider.Ollama;

var embeddingLlm = new EmbeddingLlm(llmProvider);

var tempDir = Path.Join(Path.GetTempPath(), "experiment");
var documentKey = await ChunkPdf(
    db,
    embeddingLlm,
    "/home/jeff/scratch/games/source_material/free_or_stolen/World of Darkness (Classic)/v20 Vampire The Masquerade - 20th Anniversary Edition.pdf",
    tempDir,
    5
);

var summarizeLlm = new ChatLlm(
    llmProvider,
    """
    You're going to be given a user-provided question and some snippets. Figure out the answer to that question.
    """,
    []
);

var chatLlm = new ChatLlm(
    llmProvider,
    """
    You're an assistant intended to help run a table top RPG. Use provided reference documents to look up answers to questions.

    Your default behavior should be to try to answer the question concisely. Prefer short sentence fragments, rule snippets, or bullet point lists.

    Don't include emojis unless directly quoting source text that has them.

    Most questions are going to be things like:
    - Look up some rule, or explain some game mechanic. In this case you're looking for rule snippets, examples of what dice to role, or concise explanations of what might be relevant in this scene.
    - Look up some piece of game lore. In this case you're looking for descriptions of characters, places, abilities, etc. Include any relevant rules (e.g. how to cast this spell) but also include at least a summary of the descriptive text.
    """,
    [
        AIFunctionFactory.Create(
            async (string searchTerm, string wholeQuestion) =>
            {
                Console.WriteLine($"TODO searchTerm: {searchTerm}");
                Console.WriteLine($"TODO wholeQuestion: {wholeQuestion}");
                try
                {
                    var embedding = await embeddingLlm.CreateEmbedding(searchTerm);
                    var results = await db.Search(embeddingLlm.ModelId, documentKey, embedding, 5)
                        .ToListAsync();
                    var response = await summarizeLlm.GetResponseAsync([
                        .. results.Select(x => new ChatMessage(
                            ChatRole.Assistant,
                            $"This is a snippet from the document:\n{x.Content}"
                        )),
                        new ChatMessage(
                            ChatRole.Assistant,
                            $"This is the original question: {wholeQuestion}"
                        ),
                    ]);
                    return response.Text;
                }
                catch (Exception e)
                {
                    Console.WriteLine($"TODO oops: {e.Message}\n{e.StackTrace}");
                    throw;
                }
            },
            "documentLookup",
            "Look up information in reference documents."
        ),
    ]
);

List<ChatMessage> history = [];
while (true)
{
    Console.Write("> ");
    string? message = null;
    while (string.IsNullOrWhiteSpace(message))
    {
        message = Console.ReadLine();
    }

    // TODO handle history getting too big by summarizing

    history.Add(new ChatMessage(ChatRole.User, message));
    var response = await chatLlm.GetResponseAsync(history);
    history.AddMessages(response);

    Console.WriteLine($"model response: {response}");
}
