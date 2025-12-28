namespace Experiment;

using System.Security.Cryptography.X509Certificates;
using Microsoft.Extensions.AI;
using OllamaSharp;

public abstract class Llm
{
    public enum Provider
    {
        Ollama,
    }

    public abstract string ModelId { get; }
}

public class ChatLlm : Llm
{
    private readonly IChatClient client;
    private readonly ChatOptions options;

    public ChatLlm(Provider provider, string systemPrompt, IEnumerable<AIFunction> tools)
    {
        (client, options) = provider switch
        {
            Provider.Ollama => CreateChatClient(CreateOllama, systemPrompt, tools),
        };
    }

    public override string ModelId => options.ModelId!;

    public async Task<ChatResponse> GetResponseAsync(IEnumerable<ChatMessage> history)
    {
        return await client.GetResponseAsync(history, options);
    }

    public async Task<ChatResponse> GetResponseUsingSpecificTool(
        IEnumerable<ChatMessage> history,
        string toolName
    )
    {
        var options = this.options.Clone();
        options.ToolMode = ChatToolMode.RequireSpecific(toolName);
        return await client.GetResponseAsync(history, options);
    }

    private static (IChatClient, ChatOptions) CreateChatClient(
        Func<(IChatClient, ChatOptions)> f,
        string systemPrompt,
        IEnumerable<AIFunction> tools
    )
    {
        var (client, options) = f();

        client = ChatClientBuilderChatClientExtensions
            .AsBuilder(client)
            .UseFunctionInvocation()
            .Build();

        options.Tools = tools.ToArray();
        options.Instructions = systemPrompt;
        options.Temperature = 0;

        return (client, options);
    }

    private static (IChatClient, ChatOptions) CreateOllama()
    {
        var client = new OllamaApiClient(new Uri("http://localhost:11434"));
        var options = new ChatOptions() { ModelId = "qwen3-vl:8b-instruct-q8_0" };
        return (client, options);
    }

    // TODO CreateOpenAI
    // openai chat = gpt-5-nano
    // return new OpenAIChatClient(new OpenAI.OpenAIClient(arguments.ApiKey), arguments.Model);
}

public class EmbeddingLlm : Llm
{
    private readonly IEmbeddingGenerator<string, Embedding<float>> client;
    private readonly EmbeddingGenerationOptions options;

    public EmbeddingLlm(Provider provider)
    {
        (client, options) = provider switch
        {
            Provider.Ollama => CreateOllama(),
        };
    }

    public override string ModelId => options.ModelId!;

    public async Task<ReadOnlyMemory<float>> CreateEmbedding(string input)
    {
        return await client.GenerateVectorAsync(input, options);
    }

    private static (
        IEmbeddingGenerator<string, Embedding<float>>,
        EmbeddingGenerationOptions
    ) CreateOllama()
    {
        var client = new OllamaApiClient(new Uri("http://localhost:11434"));
        var options = new EmbeddingGenerationOptions() { ModelId = "bge-m3:latest" };
        return (client, options);
    }

    // TODO CreateOpenAI
    // openai embedding = text-embedding-3-small
}
