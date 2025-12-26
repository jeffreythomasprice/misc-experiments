mod db;
mod env;
mod llm;
mod pdf;
mod process;
mod prompt;

use std::{
    cell::RefCell,
    path::Path,
    sync::{Arc, Mutex},
};

use ::llm::{
    LLMProvider,
    builder::FunctionBuilder,
    chat::{ChatMessage, ChatMessageBuilder, ChatResponse, MessageType},
};
use anyhow::Result;
use anyhow::anyhow;
use dotenvy::dotenv;
use futures::future::BoxFuture;
use prompt::prompt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tempdir::TempDir;
use tiktoken_rs::o200k_base;
use tracing::*;

use crate::{
    db::FindAllByKeyResult,
    llm::{EmbeddingLLM, LLMs, Provider, Tool, create_embedding},
    pdf::{extract_pdf_pages_into_new_pdf, extract_pdf_text},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let pkg_name = env!("CARGO_PKG_NAME").replace("-", "_");
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(format!("info,{pkg_name}=trace"))
        .init();

    let temp_dir = TempDir::new("experiment")?;
    info!("temp dir: {:?}", temp_dir.path());

    let pg_client = db::init().await?;

    let document_key: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let llms = LLMs::new(
        Provider::Ollama,
        r"
        You're trying to assist running a table top game. You have access to a document store containing relevant rule books.
        ",
        vec![
            Tool {
                name: "time".to_string(),
                description: "Gets the current time".to_string(),
                json_schema: None,
                callback: Box::new(move |_llms, _tool_call| {
                    let result = chrono::Local::now().to_rfc3339();
                    debug!("time: {}", result);
                    Box::pin(async { Ok(result) })
                }),
            },
            Tool {
                name: "document_search".to_string(),
                description: r"
                Find relevant document snippets by a search term

                search_term - a set of keywords to search for

                Result is a list of relevant snippets from documents in the database.
                "
                .to_string(),
                json_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "search_term": {"type": "string"}
                    },
                    "required": ["search_term"],
                    "additionalProperties": false
                })),
                callback: {
                    let document_key = document_key.clone();
                    Box::new(move |llms, tool_call| {
                        #[derive(Deserialize)]
                        struct Args {
                            search_term: String,
                        }
                        // TODO no unwrap
                        let args: Args = serde_json::from_str(&tool_call.function.arguments).unwrap();
                        info!("TODO search_term = {}", args.search_term);
                        let document_key = document_key.clone();
                        Box::pin(async move {
                            // match &*document_key.lock().unwrap() {
                            //     Some(document_key) => {
                            //         info!("TODO document_key = {}", document_key);
                            //         let search_results =
                            //             find_relevant_documents(&llms.embedding, pg_client, &document_key, &args.search_term, 3).await?;
                            //         // for r in search_results.iter() {
                            //         //     messages.push(
                            //         //         ChatMessage::assistant()
                            //         //             .content(format!(
                            //         //                 "this is an excerpt from a potentially relevant document\nkey: {}\npage range: {}-{}\ncontent: {}",
                            //         //                 r.key, r.first_page, r.last_page, r.content
                            //         //             ))
                            //         //             .build(),
                            //         //     );
                            //         // }
                            //         todo!()
                            //     }
                            //     None => Err(anyhow!(
                            //         "no document key available, did we fail to load the pdf into the document store?"
                            //     ))?,
                            // }
                            todo!()
                        })
                    })
                },
            },
        ],
    )
    .await?;

    {
        let path = Path::new(
            "/home/jeff/scratch/games/source_material/free_or_stolen/World of Darkness (Classic)/v20 Vampire The Masquerade - 20th Anniversary Edition.pdf",
        );
        let document_key = &mut *document_key.lock().unwrap();
        document_key.replace(
            chunk_pdf(
                &llms.embedding,
                &pg_client,
                path,
                temp_dir.path(),
                // TODO what should the max page size be?
                5,
            )
            .await?,
        );
    }

    // TODO conversation loop should be a impl method of LLMs
    let mut conversation_history = vec![];
    loop {
        let user_response = prompt("> ");
        conversation_history.push(ChatMessage::user().content(user_response).build());

        // TODO check if error is context too big and remove/summarize old messages

        // send input to llm and force tool use
        let mut llm_response = llms.chat.llm.chat_with_tools(&conversation_history, llms.chat.llm.tools()).await?;

        // loop until we're done thinking
        loop {
            trace!("llm response: {llm_response:?}");

            if let Some(thinking) = llm_response.thinking() {
                debug!("thinking: {}", thinking);
            }

            if let Some(text) = llm_response.text() {
                println!("{}", text);
                conversation_history.push(ChatMessage::assistant().content(text).build());
            }

            if let Some(tool_calls) = llm_response.tool_calls() {
                // remember the tools for the next time so it can see tool use followed immediately by tool results
                conversation_history.push(ChatMessage::assistant().tool_use(tool_calls.clone()).build());
                // actually call the tools
                let result = llms.handle_tool_calls(tool_calls).await?;
                conversation_history.push(result);
                // send that response back to the llm and get a final thought
                conversation_history.push(ChatMessage::assistant().content("Respond to the tool use").build());
                llm_response = llms.chat.llm.chat(&conversation_history).await?;
                continue;
            }

            // nothing more to think about so we're done
            break;
        }
    }
}

async fn chunk_pdf(
    embedding_llm: &EmbeddingLLM,
    pg_client: &tokio_postgres::Client,
    input_path: &Path,
    temp_dir: &Path,
    max_chunk_page_count: u32,
) -> Result<String> {
    info!("chunking pdf: {:?}", input_path);
    let page_count = pdf::get_page_count(input_path).await?;
    debug!("page count: {}", page_count);
    debug!("temp dir: {:?}", temp_dir);
    debug!("max chunk page count: {}", max_chunk_page_count);

    let key = input_path.to_string_lossy().to_string();

    // skip if we already have all pages
    let existing = db::find_all_by_key(pg_client, &embedding_llm, &key).await?;
    let existing_range = existing.iter().fold(None, |totals: Option<(u32, u32)>, e| {
        Some(match totals {
            Some((first_page, last_page)) => (first_page.min(e.first_page), last_page.max(e.last_page)),
            None => (e.first_page, e.last_page),
        })
    });
    trace!("existing page range: {:?}", existing_range);
    match existing_range {
        Some((first_existing_page, last_existing_page)) if first_existing_page == 1 && last_existing_page == page_count => {
            debug!("existing page range appears to be covered in db, skipping embedding");
            return Ok(key.to_owned());
        }
        _ => (),
    }

    let mut first_page = 1;
    loop {
        let last_page = if let Some(&FindAllByKeyResult {
            id: _,
            embedding_llm_name: _,
            key: _,
            first_page: _,
            last_page,
        }) = existing.iter().find(|e| e.first_page == first_page)
        {
            // we already have a block of pages here, skip
            debug!(
                "skipping pages for key: {}, existing page range {}..={}",
                key, first_page, last_page
            );
            last_page
        } else {
            // we need to actually generate this chunk
            let (text_content, embedding, last_page) =
                create_next_chunk_embedding(embedding_llm, input_path, temp_dir, &key, first_page, max_chunk_page_count).await?;
            debug!(
                "successfully created embedding for key: {}, page range {}..={}, text content len: {}, embedding len: {}",
                key,
                first_page,
                last_page,
                text_content.len(),
                embedding.len()
            );
            db::insert(pg_client, &embedding_llm, &key, first_page, last_page, &text_content, embedding).await?;
            last_page
        };
        first_page = (last_page - 1).max(first_page + 1);
        if last_page >= page_count {
            break;
        }
    }

    Ok(key.to_owned())
}

async fn create_next_chunk_embedding(
    embedding_llm: &EmbeddingLLM,
    input_path: &Path,
    temp_dir: &Path,
    key: &str,
    first_page: u32,
    max_chunk_page_count: u32,
) -> Result<(String, Vec<f32>, u32)> {
    let page_count = pdf::get_page_count(input_path).await?;
    let mut last_page = (first_page + max_chunk_page_count).min(page_count);
    loop {
        let current_page_count = last_page - first_page + 1;
        match create_embedding_from_pages(embedding_llm, input_path, temp_dir, key, first_page, last_page).await? {
            EmbeddingResult::Success { text_content, embedding } => {
                debug!(
                    "successfully created embedding for key: {}, page range: {}..={}",
                    key, first_page, last_page
                );
                return Ok((text_content, embedding, last_page));
            }
            EmbeddingResult::Failure {
                estimated_token_count,
                embedding: e,
            } => {
                // abort if we're already at the minimim size and still failing
                if current_page_count <= 1 {
                    Err(e)?;
                }
                // TODO context window length is hard coded, and the estimated token count may not be the right algorithm anyway, just decrement? binary search the possible token lengths?
                let estimated_page_count = (((embedding_llm.context_window_length as f64) * (current_page_count as f64)
                    / (estimated_token_count as f64))
                    // do ceil instead of floor because our estimate might be a little off and we can always do one more iteration to get down to where we need be
                    .ceil() as u32)
                    .max(1);
                last_page = (first_page + estimated_page_count - 1).min(last_page - 1);
            }
        }
    }
}

enum EmbeddingResult {
    Success {
        text_content: String,
        embedding: Vec<f32>,
    },
    Failure {
        estimated_token_count: usize,
        embedding: anyhow::Error,
    },
}

async fn create_embedding_from_pages(
    embedding_llm: &EmbeddingLLM,
    input_path: &Path,
    temp_dir: &Path,
    key: &str,
    first_page: u32,
    last_page: u32,
) -> Result<EmbeddingResult> {
    let chunk_path = extract_pdf_pages_into_new_pdf(input_path, temp_dir, first_page, last_page).await?;
    info!("chunk path: {}", chunk_path);
    let chunk_text_content = extract_pdf_text(Path::new(&chunk_path)).await?;
    // TODO do we really need the augmented content thing?
    let embedding_text = format!(
        "key: ${}\nfirst page: {}\nlast page: {}\ncontent: {}",
        key, first_page, last_page, chunk_text_content
    );
    let estimated_token_count = o200k_base()?.encode_with_special_tokens(&embedding_text).len();
    debug!("chunk estimated token count: {}", estimated_token_count);
    Ok(match create_embedding(&embedding_llm, embedding_text).await {
        Ok(embedding) => EmbeddingResult::Success {
            text_content: chunk_text_content,
            embedding,
        },
        Err(embedding) => EmbeddingResult::Failure {
            estimated_token_count,
            embedding,
        },
    })
}

async fn find_relevant_documents(
    embedding_llm: &EmbeddingLLM,
    pg_client: tokio_postgres::Client,
    key: &str,
    search: &str,
    limit: u32,
) -> Result<Vec<db::SearchResult>> {
    let embedding = create_embedding(&embedding_llm, search.to_owned()).await?;
    Ok(db::search(&pg_client, embedding_llm, &[key], embedding, limit).await?)
}
