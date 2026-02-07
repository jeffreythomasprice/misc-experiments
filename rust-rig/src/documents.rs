use std::{path::Path, time::Duration};

use anyhow::{Result, anyhow};
use rig::{
    Embed,
    client::{CompletionClient, EmbeddingsClient, ProviderClient},
    completion::{Chat, Prompt},
    embeddings::{EmbeddingModel, EmbeddingsBuilder, embed},
    providers::openai,
    vector_store::{self, InsertDocuments},
};
use rig_postgres::{PgVectorDistanceFunction, PostgresVectorStore};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tempdir::TempDir;
use tracing::*;

pub struct InsertEmbedding {
    key: String,
    first_page: i32,
    last_page: i32,
    text: String,
}

pub async fn insert_document<Model>(
    model: &Model,
    postgres_pool: sqlx::Pool<sqlx::Postgres>,
    document: InsertEmbedding,
) -> Result<String>
where
    Model: EmbeddingModel,
{
    let embedding = model
        .embed_text(&document.text)
        .await
        .map_err(|e| anyhow!("failed to create embedding: {e:?}"))?;
    let result = sqlx::query_scalar::<_, String>(
                r#"
                INSERT INTO documents (key, first_page, last_page, content, embedding) VALUES ($1, $2, $3, $4, $5)
                RETURNING id;
                "#
                )
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
) -> Result<()>
where
    Model: EmbeddingModel,
{
    // async fn top_n<T: for<'a> Deserialize<'a> + Send>(
    //     &self,
    //     req: VectorSearchRequest<PgSearchFilter>,
    // ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
    //     if req.samples() > i64::MAX as u64 {
    //         return Err(VectorStoreError::DatastoreError(
    //             format!(
    //                 "The maximum amount of samples to return with the `rig` Postgres integration cannot be larger than {}",
    //                 i64::MAX
    //             )
    //             .into(),
    //         ));
    //     }

    //     let embedded_query: pgvector::Vector = self
    //         .model
    //         .embed_text(req.query())
    //         .await?
    //         .vec
    //         .iter()
    //         .map(|&x| x as f32)
    //         .collect::<Vec<f32>>()
    //         .into();

    //     let (search_query, params) = self.search_query_full(&req);
    //     let builder = sqlx::query_as(search_query.as_str())
    //         .bind(embedded_query)
    //         .bind(req.samples() as i64);

    //     let builder = params.iter().cloned().fold(builder, bind_value);

    //     let rows = builder
    //         .fetch_all(&self.pg_pool)
    //         .await
    //         .map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

    //     let rows: Vec<(f64, String, T)> = rows
    //         .into_iter()
    //         .flat_map(SearchResult::into_result)
    //         .collect();

    //     Ok(rows)
    // }

    todo!()
}
