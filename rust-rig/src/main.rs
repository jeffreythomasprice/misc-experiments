mod documents;
mod env;
mod pdf;
mod process;

use std::{
    io::{self, Write},
    ops::Deref,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use futures::{TryStreamExt, stream};
use rig::{
    Embed,
    agent::Agent,
    client::{CompletionClient, EmbeddingsClient, ProviderClient},
    completion::{Chat, CompletionModel, Prompt},
    embeddings::{EmbeddingModel, EmbeddingsBuilder, embed},
    message::Message,
    providers::openai,
    streaming::StreamingChat,
    vector_store::{
        self, InsertDocuments, VectorSearchRequest, VectorStoreIndex, VectorStoreIndexDyn,
    },
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tempdir::TempDir;
use tracing::*;

use crate::{
    documents::{
        InsertEmbedding, SearchResult, VectorStore, delete_all_documents, delete_documents_by_key,
        delete_documents_by_path, get_document_by_path_and_pages, insert_document,
        list_all_documents, top_n_documents,
    },
    env::assert_env_var,
    pdf::extract_pdf_text,
};

/*
TODO figure out rig in general
https://docs.rig.rs/docs
*/

#[derive(Clone)]
struct AppState<EmbeddingModelT, CompletionModelT>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
{
    embeddings_model: EmbeddingModelT,
    vector_store: VectorStore<EmbeddingModelT>,
    agent: Agent<CompletionModelT>,
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
        #[arg(long)]
        count: u32,
    },
    Chat,
}

impl Commands {
    async fn exec<EmbeddingModelT, CompletionModelT>(
        &self,
        app_state: AppState<EmbeddingModelT, CompletionModelT>,
    ) -> Result<()>
    where
        EmbeddingModelT: EmbeddingModel + Clone,
        CompletionModelT: CompletionModel + 'static,
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
            Commands::SearchDocuments { query, count } => {
                search_documents_command(&app_state, query.clone(), *count).await?;
                Ok(())
            }
            Commands::Chat => {
                chat_command(&app_state).await?;
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
        // .with_env_filter(format!("info,{pkg_name}=trace"))
        .with_env_filter("debug")
        .init();

    let temp_dir = TempDir::new("experiment")?;
    info!("temp dir: {:?}", temp_dir.path());

    let postgres_pool = PgPoolOptions::new()
        .max_connections(50)
        .idle_timeout(Duration::from_secs(5))
        .connect(&assert_env_var("DATABASE_URL")?)
        .await?;

    let openai_client = openai::Client::from_env();

    let embeddings_model = openai_client.embedding_model("text-embedding-3-small");

    let vector_store = VectorStore::new(embeddings_model.clone(), postgres_pool.clone());

    let agent = openai_client
        .agent("gpt-5-nano-2025-08-07")
        .preamble(
            r#"
        You're an agent for helping run a table-top gaming session.

        You have access to a document store containing the rule book for the game we're playing.
        "#,
        )
        .dynamic_context(5, vector_store.clone())
        .build();

    sqlx::migrate!("./migrations").run(&postgres_pool).await?;

    let app_state = AppState {
        embeddings_model,
        vector_store,
        agent,
        postgres_pool,
        temp_dir: temp_dir.into_path(),
    };

    let args = Cli::parse();
    args.command.exec(app_state.clone()).await?;

    Ok(())
}

async fn list_all_documents_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
{
    info!("listing all documents");
    let documents = list_all_documents(&app_state.postgres_pool).await?;
    info!("found {} documents", documents.len());
    for document in documents.iter() {
        info!("document: {document:#?}");
    }
    Ok(())
}

async fn delete_all_documents_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
{
    info!("deleting all documents");
    delete_all_documents(&app_state.postgres_pool).await?;
    Ok(())
}

async fn delete_documents_by_path_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
    path: String,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
{
    info!("deleting documents with path: {}", path);
    delete_documents_by_path(&app_state.postgres_pool, path).await?;
    Ok(())
}

async fn delete_documents_by_key_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
    key: String,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
{
    info!("deleting documents with key: {}", key);
    delete_documents_by_key(&app_state.postgres_pool, key).await?;
    Ok(())
}

async fn insert_document_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
    input_path: &Path,
    chunk_page_count: u32,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel,
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

async fn search_documents_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
    query: String,
    n: u32,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel + Clone,
    CompletionModelT: CompletionModel,
{
    let search_request = VectorSearchRequest::builder()
        .query(query)
        .samples(5)
        .build()
        .map_err(|e| anyhow!("failed to build vector search request: {e:?}"))?;
    let results = app_state.vector_store.top_n(search_request).await?;
    info!("found {} search results", results.len());
    for (distance, text, _) in results.iter() {
        info!("search result, distance: {}, text: {}", distance, text);
    }
    Ok(())
}

async fn chat_command<EmbeddingModelT, CompletionModelT>(
    app_state: &AppState<EmbeddingModelT, CompletionModelT>,
) -> Result<()>
where
    EmbeddingModelT: EmbeddingModel,
    CompletionModelT: CompletionModel + 'static,
{
    let mut chat_history = Vec::new();
    loop {
        let mut input = String::new();
        let input = loop {
            print!("> ");
            io::stdout()
                .flush()
                .map_err(|e| anyhow!("failed to flush stdout: {e:?}"))?;

            io::stdin()
                .read_line(&mut input)
                .map_err(|e| anyhow!("failed to read line: {e:?}"))?;
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            break input;
        };

        let mut response_stream = app_state
            .agent
            .stream_chat(Message::user(input), chat_history.clone())
            .await;

        chat_history.push(Message::user(input.to_string()));

        while let Some(stream_message) = response_stream.try_next().await? {
            match stream_message {
                rig::agent::MultiTurnStreamItem::StreamAssistantItem(
                    streamed_assistant_content,
                ) => {
                    match streamed_assistant_content {
                        // TODO is this actually how tool call results are going to come to us?
                        rig::streaming::StreamedAssistantContent::ToolCall {
                            tool_call,
                            internal_call_id,
                        } => {
                            let m = Message::tool_result_with_call_id(
                                internal_call_id,
                                Some(tool_call.id.clone()),
                                format!("{:?}", tool_call),
                            );
                            trace!("tool call result: {:?}", m);
                            chat_history.push(m);
                        }
                        _ => (),
                    };
                }
                rig::agent::MultiTurnStreamItem::FinalResponse(final_response) => {
                    debug!("final response: {:?}", final_response);
                    chat_history.push(Message::assistant(final_response.response()));
                    println!("AI response: {}", final_response.response());
                }
                // type we're matching over is marked as non-exhaustive, and we don't handle all cases anyway
                _ => (),
            }
        }
    }
}
