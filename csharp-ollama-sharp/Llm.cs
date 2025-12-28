namespace Experiment;

using System.Globalization;
using Microsoft.Extensions.AI;
using OllamaSharp;

public class Llm
{
    public enum Provider
    {
        Ollama,
    }

    private readonly IChatClient chatClient;
    private readonly ChatOptions chatOptions;

    private readonly IEmbeddingGenerator<string, Embedding<float>> embeddingClient;
    private readonly EmbeddingGenerationOptions embeddingOptions;

    private readonly List<ChatMessage> history;

    public Llm(Provider provider, string systemPrompt, IEnumerable<AIFunction> tools)
    {
        ((chatClient, chatOptions), (embeddingClient, embeddingOptions)) = provider switch
        {
            Provider.Ollama => (
                CreateChatClient(CreateOllamaChatClient, systemPrompt, tools),
                CreateOllamaEmbeddingClient()
            ),
        };
        history = [];
    }

    public string ChatModelId => chatOptions.ModelId!;

    public string EmbeddingModelId => embeddingOptions.ModelId!;

    public async Task<ChatResponse> SendUserMessageAndGetResponseAsync(string userMessage)
    {
        // TODO handle history getting too big by summarizing
        history.Add(new ChatMessage(ChatRole.User, userMessage));
        var response = await chatClient.GetResponseAsync(history, chatOptions);
        history.AddMessages(response);
        return response;
    }

    public async Task<ReadOnlyMemory<float>> CreateEmbedding(string input)
    {
        return await embeddingClient.GenerateVectorAsync(input, embeddingOptions);
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

    private static (IChatClient, ChatOptions) CreateOllamaChatClient()
    {
        var client = new OllamaApiClient(new Uri("http://localhost:11434"));
        var options = new ChatOptions() { ModelId = "qwen3-vl:8b-instruct-q8_0" };
        return (client, options);
    }

    private static (
        IEmbeddingGenerator<string, Embedding<float>>,
        EmbeddingGenerationOptions
    ) CreateOllamaEmbeddingClient()
    {
        var client = new OllamaApiClient(new Uri("http://localhost:11434"));
        var options = new EmbeddingGenerationOptions() { ModelId = "bge-m3:latest" };
        return (client, options);
    }

    // TODO CreateOpenAIChatClient
    // openai chat = gpt-5-nano
    // return new OpenAIChatClient(new OpenAI.OpenAIClient(arguments.ApiKey), arguments.Model);

    // TODO CreateOpenAIEmbeddingCLient
    // openai embedding = text-embedding-3-small
}
