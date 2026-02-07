mod documents;
mod env;
mod pdf;
mod process;

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

use crate::{documents::insert_document, env::assert_env_var, pdf::extract_pdf_text};

/*
TODO figure out embeddings
https://github.com/0xPlaygrounds/rig/blob/main/rig-integrations/rig-postgres/examples/vector_search_postgres.rs

TODO figure out rig in general
https://docs.rig.rs/docs
*/

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

    // TODO only truncate if asked first?
    // sqlx::query("TRUNCATE documents")
    //     .execute(&postgres_pool)
    //     .await?;

    sqlx::migrate!("./migrations").run(&postgres_pool).await?;

    // TODO is this actually needed? do we need to depend on rig-postgres?
    let vector_store = PostgresVectorStore::new(
        embeddings_model.clone(),
        postgres_pool.clone(),
        Some("documents".to_string()),
        PgVectorDistanceFunction::Cosine,
    );

    {
        let path = Path::new(
            "/home/jeff/scratch/games/source_material/free_or_stolen/World of Darkness (Classic)/v20 Vampire The Masquerade - 20th Anniversary Edition.pdf",
        );
        chunk_pdf(
            embeddings_model.clone(),
            postgres_pool.clone(),
            path,
            temp_dir.path(),
            3,
        )
        .await?;
    }

    // let response = agent.prompt("test").await?;
    // info!("response: {response}");

    Ok(())
}

async fn chunk_pdf<Model>(
    model: Model,
    postgres_pool: sqlx::Pool<sqlx::Postgres>,
    input_path: &Path,
    temp_dir: &Path,
    chunk_page_count: u32,
) -> Result<()>
where
    Model: EmbeddingModel + Clone,
{
    info!("chunking pdf: {:?}", input_path);
    let page_count = pdf::get_page_count(input_path).await?;
    debug!("page count: {}", page_count);
    debug!("temp dir: {:?}", temp_dir);
    debug!("chunk page count: {}", chunk_page_count);

    let key = input_path.to_string_lossy().to_string();
    debug!("key: {}", key);

    if chunk_page_count < 2 {
        return Err(anyhow::anyhow!(
            "chunk_page_count must be at least 2 to overlap by one page"
        ));
    }

    let mut first_page = 1;
    let step = chunk_page_count - 1;
    while first_page <= page_count {
        let last_page = (first_page + chunk_page_count - 1).min(page_count);
        let output_path =
            pdf::extract_pdf_pages_into_new_pdf(input_path, temp_dir, first_page, last_page)
                .await?;
        info!(
            "chunked pages {}-{} into {}",
            first_page, last_page, output_path
        );
        if last_page == page_count {
            break;
        }
        first_page += step;

        let new_id = insert_document(
            &model,
            postgres_pool.clone(),
            InsertEmbedding {
                key: key.clone(),
                first_page: first_page as i32,
                last_page: last_page as i32,
                text: extract_pdf_text(Path::new(&output_path)).await?,
            },
        )
        .await
        .map_err(|e| {
            anyhow!(
                "failed to insert document for key: {}, pages: {}-{}: {:?}",
                key,
                first_page,
                last_page,
                e
            )
        })?;
        trace!(
            "inserted document for key: {}, pages: {}-{}, new id: {}",
            key, first_page, last_page, new_id
        );
    }

    Ok(())
}
