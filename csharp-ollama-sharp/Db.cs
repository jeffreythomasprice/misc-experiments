namespace Experiment;

using System.ComponentModel;
using System.Text.RegularExpressions;
using Npgsql;
using Pgvector;

public record FindAllByKeyResult(
    int Id,
    string EmbeddingLlmName,
    string Key,
    int FirstPage,
    int LastPage
);

public record SearchResult(
    int Id,
    string EmbeddingLlmName,
    string Key,
    int FirstPage,
    int LastPage,
    string Content
);

public record Insert(
    string EmbeddingLlmName,
    string Key,
    int FirstPage,
    int LastPage,
    string Content,
    ReadOnlyMemory<float> Embedding
);

public class Db : IAsyncDisposable
{
    private readonly NpgsqlDataSource dataSource;

    public static async Task<Db> Create(
        string host,
        int port,
        string username,
        string password,
        string database
    )
    {
        var connectionStringBuilder = new NpgsqlConnectionStringBuilder();
        connectionStringBuilder.Host = host;
        connectionStringBuilder.Port = port;
        connectionStringBuilder.Username = username;
        connectionStringBuilder.Password = password;
        connectionStringBuilder.Database = database;
        var result = new Db(connectionStringBuilder);

        try
        {
            await using var testCommand = result.dataSource.CreateCommand("SELECT 1");
            var testResult = await testCommand.ExecuteScalarAsync();
            if (testResult?.Equals(1) != true)
            {
                // TODO proper logging
                Console.WriteLine(
                    $"testResult = {testResult}, typeof testResult = {testResult?.GetType()}"
                );
                throw new Exception("test command failed");
            }
        }
        catch
        {
            await result.DisposeAsync();
            throw;
        }

        await result
            .dataSource.CreateCommand("CREATE EXTENSION IF NOT EXISTS vector")
            .ExecuteNonQueryAsync();

        return result;
    }

    private Db(NpgsqlConnectionStringBuilder connectionStringBuilder)
    {
        var builder = new NpgsqlDataSourceBuilder(connectionStringBuilder.ToString());
        builder.UseVector();
        dataSource = builder.Build();
    }

    public async ValueTask DisposeAsync()
    {
        try
        {
            await dataSource.DisposeAsync();
        }
        catch
        {
            // TODO log
        }
    }

    public async IAsyncEnumerable<FindAllByKeyResult> FindAllByKey(
        string embeddingLlmName,
        string key
    )
    {
        var (tableExists, tableName) = await CheckIfTableExists(embeddingLlmName);
        if (tableExists)
        {
            var command = dataSource.CreateCommand(
                $"""
                SELECT id, embedding_llm_name, key, first_page, last_page
                    FROM {tableName}
                    WHERE embedding_llm_name = $1 AND key = $2
                """
            );
            command.Parameters.AddWithValue(embeddingLlmName);
            command.Parameters.AddWithValue(key);
            await using var results = await command.ExecuteReaderAsync();
            while (await results.ReadAsync())
            {
                yield return new(
                    results.GetInt32(0),
                    results.GetString(1),
                    results.GetString(2),
                    results.GetInt32(3),
                    results.GetInt32(4)
                );
            }
        }
    }

    public async IAsyncEnumerable<SearchResult> Search(
        string embeddingLlmName,
        string key,
        ReadOnlyMemory<float> embedding,
        int limit
    )
    {
        var (tableExists, tableName) = await CheckIfTableExists(embeddingLlmName);
        if (tableExists)
        {
            var command = dataSource.CreateCommand(
                $"""
                SELECT id, embedding_llm_name, key, first_page, last_page, content
                    FROM {tableName}
                    WHERE embedding_llm_name = $1 AND key = $2
                    ORDER BY embedding <-> $3
                    LIMIT $4
                """
            );
            command.Parameters.AddWithValue(embeddingLlmName);
            command.Parameters.AddWithValue(key);
            command.Parameters.AddWithValue(new Vector(embedding));
            command.Parameters.AddWithValue(limit);
            await using var results = await command.ExecuteReaderAsync();
            while (await results.ReadAsync())
            {
                yield return new(
                    results.GetInt32(0),
                    results.GetString(1),
                    results.GetString(2),
                    results.GetInt32(3),
                    results.GetInt32(4),
                    results.GetString(5)
                );
            }
        }
    }

    public async Task Insert(Insert data)
    {
        var tableName = await CreateDocumentChunkTableIfNeeded(
            data.EmbeddingLlmName,
            data.Embedding.Length
        );
        var command = dataSource.CreateCommand(
            $"""
            INSERT INTO {tableName}
                (embedding_llm_name, key, first_page, last_page, content, embedding)
                VALUES ($1, $2, $3, $4, $5, $6)
            """
        );
        command.Parameters.AddWithValue(data.EmbeddingLlmName);
        command.Parameters.AddWithValue(data.Key);
        command.Parameters.AddWithValue(data.FirstPage);
        command.Parameters.AddWithValue(data.LastPage);
        command.Parameters.AddWithValue(data.Content);
        command.Parameters.AddWithValue(new Vector(data.Embedding));
        await command.ExecuteNonQueryAsync();
    }

    private async Task<string> CreateDocumentChunkTableIfNeeded(
        string embeddingLlmName,
        int embeddingVectorLength
    )
    {
        var tableName = GetTableName(embeddingLlmName);
        await dataSource
            .CreateCommand(
                $"""
                CREATE TABLE IF NOT EXISTS {tableName} (
                    id SERIAL PRIMARY KEY,
                    embedding_llm_name TEXT NOT NULL,
                    key TEXT NOT NULL,
                    first_page INT NOT NULL,
                    last_page INT NOT NULL,
                    content TEXT NOT NULL,
                    embedding vector({embeddingVectorLength}) NOT NULL,
                    UNIQUE (embedding_llm_name, key, first_page, last_page)
                );

                CREATE INDEX IF NOT EXISTS idx_{tableName}_embedding ON {tableName} USING hnsw (embedding vector_l2_ops);
                """
            )
            .ExecuteNonQueryAsync();
        return tableName;
    }

    private async Task<(bool, string)> CheckIfTableExists(string embeddingLlmName)
    {
        var tableName = GetTableName(embeddingLlmName);
        try
        {
            var result = await dataSource
                .CreateCommand(
                    $"""
                    SELECT count(*) FROM {tableName} LIMIT 1;
                    """
                )
                .ExecuteScalarAsync();
            return (true, tableName);
        }
        catch (PostgresException e)
        {
            if (e.Message.Contains($"relation \"{tableName}\" does not exist"))
            {
                return (false, tableName);
            }
            throw;
        }
    }

    private string GetTableName(string embeddingLlmName)
    {
        return Regex.Replace(embeddingLlmName, @"[^a-zA-Z0-9_]", "_", RegexOptions.Multiline);
    }
}
