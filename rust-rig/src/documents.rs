use std::{path::Path, time::Duration};

use anyhow::{Result, anyhow};
use rig::{
    Embed,
    client::{CompletionClient, EmbeddingsClient, ProviderClient},
    completion::{Chat, Prompt},
    embeddings::{EmbeddingModel, EmbeddingsBuilder, embed},
    providers::openai,
    vector_store::{self, InsertDocuments, VectorStoreIndex},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    postgres::{self, PgPoolOptions},
};
use tempdir::TempDir;
use tracing::*;
use uuid::Uuid;

/*
This is my version of what rig-postgres implements. Lots of stuff taken directly from there, but they didn't give enough flexibility in
the way the underlying documents table is structured.
*/

#[derive(Debug, FromRow)]
pub struct DocumentListResult {
    pub id: Uuid,
    pub path: String,
    pub key: String,
    pub first_page: i32,
    pub last_page: i32,
}

#[derive(Debug)]
pub struct InsertEmbedding {
    pub path: String,
    pub key: String,
    pub first_page: i32,
    pub last_page: i32,
    pub text: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub path: String,
    pub key: String,
    pub first_page: i32,
    pub last_page: i32,
    pub text: String,
    pub distance: f64,
}

#[derive(Clone)]
pub struct VectorStore<EmbeddingModelT>
where
    EmbeddingModelT: EmbeddingModel,
{
    model: EmbeddingModelT,
    pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(Debug, Clone)]
pub struct VectorStoreSearchFilter {}

pub async fn list_all_documents(
    postgres_pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Vec<DocumentListResult>> {
    Ok(sqlx::query_as::<_, DocumentListResult>(
        "SELECT id, path, key, first_page, last_page FROM documents",
    )
    .fetch_all(postgres_pool)
    .await
    .map_err(|e| anyhow!("failed to list documents: {e:?}"))?)
}

pub async fn get_document_by_path_and_pages(
    postgres_pool: &sqlx::Pool<sqlx::Postgres>,
    path: String,
    first_page: i32,
    last_page: i32,
) -> Result<Option<DocumentListResult>> {
    Ok(sqlx::query_as::<_, DocumentListResult>(
        "SELECT id, path, key, first_page, last_page FROM documents WHERE path = $1 AND first_page = $2 AND last_page = $3",
    )
    .bind(&path)
    .bind(first_page)
    .bind(last_page)
    .fetch_optional(postgres_pool)
    .await
    .map_err(|e| anyhow!("failed to get document by path ({}) and pages ({}-{}): {:?}", path, first_page, last_page, e))?)
}

pub async fn delete_all_documents(postgres_pool: &sqlx::Pool<sqlx::Postgres>) -> Result<()> {
    sqlx::query("TRUNCATE documents")
        .execute(postgres_pool)
        .await
        .map_err(|e| anyhow!("failed to delete all documents: {e:?}"))?;
    Ok(())
}

pub async fn delete_documents_by_path(
    postgres_pool: &sqlx::Pool<sqlx::Postgres>,
    path: String,
) -> Result<()> {
    sqlx::query("DELETE FROM documents WHERE path = $1")
        .bind(path)
        .execute(postgres_pool)
        .await
        .map_err(|e| anyhow!("failed to delete documents by path: {e:?}"))?;
    Ok(())
}

pub async fn delete_documents_by_key(
    postgres_pool: &sqlx::Pool<sqlx::Postgres>,
    key: String,
) -> Result<()> {
    sqlx::query("DELETE FROM documents WHERE key = $1")
        .bind(key)
        .execute(postgres_pool)
        .await
        .map_err(|e| anyhow!("failed to delete documents by key: {e:?}"))?;
    Ok(())
}

pub async fn insert_document<Model>(
    model: &Model,
    postgres_pool: sqlx::Pool<sqlx::Postgres>,
    document: InsertEmbedding,
) -> Result<Uuid>
where
    Model: EmbeddingModel,
{
    let embedding = model
        .embed_text(&document.text)
        .await
        .map_err(|e| anyhow!("failed to create embedding: {e:?}"))?;
    let result = sqlx::query_scalar(
                r#"
                INSERT INTO documents (path, key, first_page, last_page, text, embedding) VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id;
                "#
                )
                .bind(&document.path)
                .bind(&document.key)
                .bind(document.first_page)
                .bind(document.last_page)
                .bind(&document.text)
                .bind(&embedding.vec)
                .fetch_one(&postgres_pool)
                .await.map_err(|e| anyhow!("failed to insert document: {e:?}"))?;
    Ok(result)
}

pub async fn top_n_documents<Model>(
    model: &Model,
    postgres_pool: sqlx::Pool<sqlx::Postgres>,
    search_text: String,
    n: u32,
) -> Result<Vec<SearchResult>>
where
    Model: EmbeddingModel,
{
    let search_embedding = model
        .embed_text(&search_text)
        .await
        .map_err(|e| anyhow!("failed to create embedding: {e:?}"))?;
    Ok(sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, path, key, first_page, last_page, text, embedding <-> $1::vector AS distance
        FROM documents
        ORDER BY distance
        LIMIT $2;
        "#,
    )
    .bind(&search_embedding.vec)
    .bind(n as i32)
    .fetch_all(&postgres_pool)
    .await?)
}

impl<EmbeddingModelT> VectorStore<EmbeddingModelT>
where
    EmbeddingModelT: EmbeddingModel,
{
    pub fn new(model: EmbeddingModelT, pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { model, pool }
    }
}

impl rig::vector_store::request::SearchFilter for VectorStoreSearchFilter {
    type Value = VectorStoreSearchFilter;

    fn eq(key: impl AsRef<str>, value: Self::Value) -> Self {
        todo!()
    }

    fn gt(key: impl AsRef<str>, value: Self::Value) -> Self {
        todo!()
    }

    fn lt(key: impl AsRef<str>, value: Self::Value) -> Self {
        todo!()
    }

    fn and(self, rhs: Self) -> Self {
        todo!()
    }

    fn or(self, rhs: Self) -> Self {
        todo!()
    }
}

impl<EmbeddingModelT> VectorStoreIndex for VectorStore<EmbeddingModelT>
where
    EmbeddingModelT: EmbeddingModel,
{
    type Filter = VectorStoreSearchFilter;

    async fn top_n<T: for<'a> serde::Deserialize<'a> + rig::wasm_compat::WasmCompatSend>(
        &self,
        req: vector_store::VectorSearchRequest<Self::Filter>,
    ) -> std::result::Result<Vec<(f64, String, T)>, vector_store::VectorStoreError> {
        let results = top_n_documents(
            &self.model,
            self.pool.clone(),
            req.query().to_string(),
            req.samples() as u32,
        )
        .await
        .map_err(|e| vector_store::VectorStoreError::DatastoreError(e.into()))?;
        Ok(results
            .into_iter()
            .map(
                |r| -> std::result::Result<(f64, String, T), vector_store::VectorStoreError> {
                    // TODO can we omit the serialize and deserialize when we know T = SearchResult?
                    let x = serde_json::to_string(&r)?;
                    let t: T = serde_json::from_str(&x)?;
                    Ok((r.distance, r.text, t))
                },
            )
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn top_n_ids(
        &self,
        req: vector_store::VectorSearchRequest<Self::Filter>,
    ) -> std::result::Result<Vec<(f64, String)>, vector_store::VectorStoreError> {
        // TODO only return distance and id
        todo!();
    }
}
