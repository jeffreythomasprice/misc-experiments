mod documents;
mod env;
mod pdf;
mod process;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use rig::{
    Embed,
    client::{CompletionClient, EmbeddingsClient, ProviderClient},
    completion::{Chat, Prompt},
    embeddings::{EmbeddingModel, EmbeddingsBuilder, embed},
    providers::openai,
    vector_store::{self, InsertDocuments},
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tempdir::TempDir;
use tracing::*;

use crate::{
    documents::{
        InsertEmbedding, delete_all_documents, delete_documents_by_key, delete_documents_by_path,
        get_document_by_path_and_pages, insert_document, list_all_documents, top_n_documents,
    },
    env::assert_env_var,
    pdf::extract_pdf_text,
};

/*
TODO figure out embeddings
https://github.com/0xPlaygrounds/rig/blob/main/rig-integrations/rig-postgres/examples/vector_search_postgres.rs

TODO figure out rig in general
https://docs.rig.rs/docs
*/

#[derive(Clone)]
struct AppState<EmbeddingModelT>
where
    EmbeddingModelT: EmbeddingModel,
{
    embeddings_model: EmbeddingModelT,
    postgres_pool: sqlx::Pool<sqlx::Postgres>,
    temp_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    // TODO also filter on list documents?
    ListDocuments,
    DeleteDocuments {
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        key: Option<String>,
    },
    InsertDocument {
        #[arg(long)]
        path: String,
        #[arg(long)]
        chunk_page_count: u32,
    },
    SearchDocuments {
        #[arg(long)]
        query: String,
    },
    // TODO start a chat session
}

impl Commands {
    async fn exec<EmbeddingModelT>(&self, app_state: AppState<EmbeddingModelT>) -> Result<()>
    where
        EmbeddingModelT: EmbeddingModel + Clone,
    {
        match self {
            Commands::ListDocuments => {
                list_all_documents_command(&app_state).await?;
                Ok(())
            }
            Commands::DeleteDocuments { path, key } => match (path, key) {
                (None, None) => Ok(delete_all_documents_command(&app_state).await?),
                (None, Some(key)) => {
                    Ok(delete_documents_by_key_command(&app_state, key.clone()).await?)
                }
                (Some(path), None) => {
                    Ok(delete_documents_by_path_command(&app_state, path.clone()).await?)
                }
                (Some(_), Some(_)) => Err(anyhow!("provide exactly one of path or key"))?,
            },
            Commands::InsertDocument {
                path,
                chunk_page_count,
            } => {
                insert_document_command(&app_state, Path::new(path), *chunk_page_count).await?;
                Ok(())
            }
            Commands::SearchDocuments { query } => {
                search_documents_command(&app_state, query.clone()).await?;
                Ok(())
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    let temp_dir = TempDir::new("experiment")?;
    info!("temp dir: {:?}", temp_dir.path());

    let openai_client = openai::Client::from_env();

    let embeddings_model = openai_client.embedding_model("text-embedding-3-small");

    let agent = openai_client
        .agent("gpt-5-nano-2025-08-07")
        .preamble("You're an agent for helping run a table-top gaming session.")
        .build();

    let postgres_pool = PgPoolOptions::new()
        .max_connections(50)
        .idle_timeout(Duration::from_secs(5))
        .connect(&assert_env_var("DATABASE_URL")?)
        .await?;

    sqlx::migrate!("./migrations").run(&postgres_pool).await?;

    let app_state = AppState {
        embeddings_model,
        postgres_pool,
        temp_dir: temp_dir.into_path(),
    };

    let args = Cli::parse();
    args.command.exec(app_state.clone()).await?;

    Ok(())
}

async fn list_all_documents_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    info!("listing all documents");
    let documents = list_all_documents(&app_state.postgres_pool).await?;
    info!("found {} documents", documents.len());
    for document in documents.iter() {
        info!("document: {document:#?}");
    }
    Ok(())
}

async fn delete_all_documents_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    info!("deleting all documents");
    delete_all_documents(&app_state.postgres_pool).await?;
    Ok(())
}

async fn delete_documents_by_path_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
    path: String,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    info!("deleting documents with path: {}", path);
    delete_documents_by_path(&app_state.postgres_pool, path).await?;
    Ok(())
}

async fn delete_documents_by_key_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
    key: String,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    info!("deleting documents with key: {}", key);
    delete_documents_by_key(&app_state.postgres_pool, key).await?;
    Ok(())
}

async fn insert_document_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
    input_path: &Path,
    chunk_page_count: u32,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    info!(
        "chunking pdf, input_path: {:?}, temp_dir: {:?}, chunk_page_count: {}",
        input_path, app_state.temp_dir, chunk_page_count
    );
    let path_str = input_path.to_string_lossy().to_string();
    let page_count = pdf::get_page_count(input_path).await?;
    let key = input_path.to_string_lossy().to_string();
    debug!(
        "path: {}, key: {}, page_count: {}",
        path_str, key, page_count
    );

    if chunk_page_count < 2 {
        return Err(anyhow::anyhow!(
            "chunk_page_count must be at least 2 to overlap by one page"
        ));
    }

    let mut first_page = 1;
    let step = chunk_page_count - 1;
    while first_page <= page_count {
        let last_page = (first_page + chunk_page_count - 1).min(page_count);
        let chunk_description = format!(
            "chunk(path: {}, pages: {}-{})",
            path_str, first_page, last_page
        );
        let output_path = pdf::extract_pdf_pages_into_new_pdf(
            input_path,
            &app_state.temp_dir,
            first_page,
            last_page,
        )
        .await?;
        debug!("chunking {}", chunk_description);
        if last_page == page_count {
            break;
        }

        if get_document_by_path_and_pages(
            &app_state.postgres_pool,
            path_str.clone(),
            first_page as i32,
            last_page as i32,
        )
        .await?
        .is_some()
        {
            debug!("{} already exists, skipping", chunk_description);
            continue;
        }

        let new_id = insert_document(
            &app_state.embeddings_model,
            app_state.postgres_pool.clone(),
            InsertEmbedding {
                path: path_str.clone(),
                key: key.clone(),
                first_page: first_page as i32,
                last_page: last_page as i32,
                text: extract_pdf_text(Path::new(&output_path)).await?,
            },
        )
        .await
        .map_err(|e| anyhow!("failed to insert document {}: {:?}", chunk_description, e))?;

        trace!(
            "inserted document {}, new id: {}",
            chunk_description, new_id
        );

        first_page += step;
    }

    info!("chunking complete, path: {:?}", input_path);

    Ok(())
}

async fn search_documents_command<EmbeddingModelT>(
    app_state: &AppState<EmbeddingModelT>,
    query: String,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
{
    let search_result = top_n_documents(
        &app_state.embeddings_model,
        app_state.postgres_pool.clone(),
        "thaumaturgy cauldren of blood rules".to_string(),
    )
    .await?;
    info!("search result: {search_result:#?}");
    Ok(())
}
