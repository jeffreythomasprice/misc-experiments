use crate::env::assert_env_var;
use anyhow::{Result, anyhow};
use tokio_postgres::{Client, Config, NoTls};
use tracing::*;

pub struct FindAllByKeyResult {
    pub id: i32,
    pub embedding_llm_name: String,
    pub key: String,
    pub first_page: u32,
    pub last_page: u32,
}

pub struct SearchResult {
    pub id: i32,
    pub embedding_llm_name: String,
    pub key: String,
    pub first_page: u32,
    pub last_page: u32,
    pub content: String,
}

pub async fn init() -> Result<Client> {
    let host = assert_env_var("PG_HOST")?;
    let port = assert_env_var("PG_PORT")?
        .parse()
        .map_err(|e| anyhow!("failed to parse port into int: {e:?}"))?;
    let username = assert_env_var("PG_USERNAME")?;
    let password = assert_env_var("PG_PASSWORD")?;
    let database = assert_env_var("PG_DATABASE")?;

    let (client, connection) = Config::new()
        .host(host)
        .port(port)
        .user(username)
        .password(password)
        .dbname(database)
        .connect(NoTls)
        .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    debug!("connected to db");

    client
        .batch_execute(
            r"
		CREATE EXTENSION IF NOT EXISTS vector;

		CREATE TABLE IF NOT EXISTS document_chunk (
			id SERIAL PRIMARY KEY,
            embedding_llm_name TEXT NOT NULL,
			key TEXT NOT NULL,
			first_page INT NOT NULL,
			last_page INT NOT NULL,
			content TEXT NOT NULL,
			-- TODO determine length of embedding vector dynamically
			embedding vector(1024) NOT NULL,
			UNIQUE (embedding_llm_name, key, first_page, last_page)
		);

		CREATE INDEX IF NOT EXISTS idx_document_chunk_embedding ON document_chunk USING hnsw (embedding vector_l2_ops);
	",
        )
        .await?;

    debug!("db initialized");

    Ok(client)
}

pub async fn find_all_by_key(client: &Client, embedding_llm_name: &str, key: &str) -> Result<Vec<FindAllByKeyResult>> {
    let results = client
        .query(
            r"
            SELECT id, embedding_llm_name, key, first_page, last_page
            FROM document_chunk
            WHERE embedding_llm_name = $1 AND key = $2
            ",
            &[&embedding_llm_name, &key],
        )
        .await?;
    Ok(results
        .iter()
        .map(|row| FindAllByKeyResult {
            id: row.get(0),
            embedding_llm_name: row.get(1),
            key: row.get(2),
            first_page: row.get::<_, i32>(3) as u32,
            last_page: row.get::<_, i32>(4) as u32,
        })
        .collect())
}

pub async fn insert(
    client: &Client,
    embedding_llm_name: &str,
    key: &str,
    first_page: u32,
    last_page: u32,
    content: &str,
    embedding: Vec<f32>,
) -> Result<()> {
    let first_page = first_page as i32;
    let last_page = last_page as i32;
    let embedding = pgvector::Vector::from(embedding);
    client
        .execute(
            r"
            INSERT INTO document_chunk
            (embedding_llm_name, key, first_page, last_page, content, embedding)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
            &[&embedding_llm_name, &key, &first_page, &last_page, &content, &embedding],
        )
        .await?;
    Ok(())
}

pub async fn search(client: &Client, keys: &[&str], embedding: Vec<f32>, limit: u32) -> Result<Vec<SearchResult>> {
    let embedding = pgvector::Vector::from(embedding);
    let limit = limit as i64;
    let results = client
        .query(
            r"
            SELECT id, embedding_llm_name, key, first_page, last_page, content
            FROM document_chunk
            WHERE key = ANY ($1)
            ORDER BY embedding <-> $2
            LIMIT $3
            ",
            &[&keys, &embedding, &limit],
        )
        .await?;
    Ok(results
        .iter()
        .map(|row| SearchResult {
            id: row.get(0),
            embedding_llm_name: row.get(1),
            key: row.get(2),
            first_page: row.get::<_, i32>(3) as u32,
            last_page: row.get::<_, i32>(4) as u32,
            content: row.get(5),
        })
        .collect())
}
