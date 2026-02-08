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
use serde::Serialize;
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

pub struct InsertEmbedding {
    pub path: String,
    pub key: String,
    pub first_page: i32,
    pub last_page: i32,
    pub text: String,
}

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
    info!(
        "TODO document, first_page: {}, last_page: {}",
        document.first_page, document.last_page
    );
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

// TODO also implement the VectorStoreIndex trait?

// impl<Model> VectorStoreIndex for PostgresVectorStore<Model>
// where
//     Model: EmbeddingModel,
// {
//     type Filter = PgSearchFilter;

//     /// Get the top n documents based on the distance to the given query.
//     /// The result is a list of tuples of the form (score, id, document)
//     async fn top_n<T: for<'a> Deserialize<'a> + Send>(
//         &self,
//         req: VectorSearchRequest<PgSearchFilter>,
//     ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
//         if req.samples() > i64::MAX as u64 {
//             return Err(VectorStoreError::DatastoreError(
//                 format!(
//                     "The maximum amount of samples to return with the `rig` Postgres integration cannot be larger than {}",
//                     i64::MAX
//                 )
//                 .into(),
//             ));
//         }

//         let embedded_query: pgvector::Vector = self
//             .model
//             .embed_text(req.query())
//             .await?
//             .vec
//             .iter()
//             .map(|&x| x as f32)
//             .collect::<Vec<f32>>()
//             .into();

//         let (search_query, params) = self.search_query_full(&req);
//         let builder = sqlx::query_as(search_query.as_str())
//             .bind(embedded_query)
//             .bind(req.samples() as i64);

//         let builder = params.iter().cloned().fold(builder, bind_value);

//         let rows = builder
//             .fetch_all(&self.pg_pool)
//             .await
//             .map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

//         let rows: Vec<(f64, String, T)> = rows
//             .into_iter()
//             .flat_map(SearchResult::into_result)
//             .collect();

//         Ok(rows)
//     }

//     /// Same as `top_n` but returns the document ids only.
//     async fn top_n_ids(
//         &self,
//         req: VectorSearchRequest<PgSearchFilter>,
//     ) -> Result<Vec<(f64, String)>, VectorStoreError> {
//         if req.samples() > i64::MAX as u64 {
//             return Err(VectorStoreError::DatastoreError(
//                 format!(
//                     "The maximum amount of samples to return with the `rig` Postgres integration cannot be larger than {}",
//                     i64::MAX
//                 )
//                 .into(),
//             ));
//         }
//         let embedded_query: pgvector::Vector = self
//             .model
//             .embed_text(req.query())
//             .await?
//             .vec
//             .iter()
//             .map(|&x| x as f32)
//             .collect::<Vec<f32>>()
//             .into();

//         let (search_query, params) = self.search_query_only_ids(&req);
//         let builder = sqlx::query_as(search_query.as_str())
//             .bind(embedded_query)
//             .bind(req.samples() as i64);

//         let builder = params.iter().cloned().fold(builder, bind_value);

//         let rows: Vec<SearchResultOnlyId> = builder
//             .fetch_all(&self.pg_pool)
//             .await
//             .map_err(|e| VectorStoreError::DatastoreError(Box::new(e)))?;

//         let rows: Vec<(f64, String)> = rows
//             .into_iter()
//             .map(|row| (row.distance, row.id.to_string()))
//             .collect();

//         Ok(rows)
//     }
// }
