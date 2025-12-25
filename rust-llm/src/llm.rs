use anyhow::{Result, anyhow};
use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
};

use crate::env::assert_env_var;

pub struct NamedLLM {
    pub llm: Box<dyn LLMProvider>,
    pub name: String,
}

pub struct EmbeddingLLM {
    pub llm: NamedLLM,
    pub context_window: u32,
    pub embedding_vector_length: usize,
}

pub struct LLMs {
    pub chat: NamedLLM,
    pub embedding: EmbeddingLLM,
}

pub async fn create_openai() -> Result<LLMs> {
    Ok(LLMs {
        chat: create_openai_llm()?,
        embedding: create_openai_embedding_llm().await?,
    })
}

pub async fn create_ollama() -> Result<LLMs> {
    Ok(LLMs {
        chat: create_ollama_llm()?,
        embedding: create_ollama_embedding_llm().await?,
    })
}

pub async fn create_embedding(llm: &EmbeddingLLM, input: String) -> Result<Vec<f32>> {
    Ok(create_embedding_from_llm(&llm.llm, input).await?)
}

fn create_openai_llm() -> Result<NamedLLM> {
    let name = "gpt-5-nano".to_string();
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .model(&name)
        .api_key(get_openai_api_key()?)
        .build()?;
    Ok(NamedLLM { llm, name })
}

async fn create_openai_embedding_llm() -> Result<EmbeddingLLM> {
    let name = "text-embedding-3-small".to_string();
    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .model(&name)
        .api_key(get_openai_api_key()?)
        .build()?;
    let llm = NamedLLM { llm, name };
    Ok(create_embedding_llm(llm, 8192).await?)
}

fn create_ollama_llm() -> Result<NamedLLM> {
    let name = "llama3.2:1b".to_string();
    let llm = LLMBuilder::new()
        .backend(LLMBackend::Ollama)
        .base_url(get_ollama_base_url())
        .model(&name)
        .build()?;
    Ok(NamedLLM { llm, name })
}

async fn create_ollama_embedding_llm() -> Result<EmbeddingLLM> {
    let name = "bge-m3:latest".to_string();
    let llm = LLMBuilder::new()
        .backend(LLMBackend::Ollama)
        .base_url(get_ollama_base_url())
        .model(&name)
        .build()?;
    let llm = NamedLLM { llm, name };
    Ok(create_embedding_llm(llm, 8192).await?)
}

async fn create_embedding_llm(llm: NamedLLM, context_window: u32) -> Result<EmbeddingLLM> {
    let test_embedding = create_embedding_from_llm(&llm, "Hello, World!".to_owned()).await?;
    Ok(EmbeddingLLM {
        llm,
        context_window,
        embedding_vector_length: test_embedding.len(),
    })
}

async fn create_embedding_from_llm(llm: &NamedLLM, input: String) -> Result<Vec<f32>> {
    let mut result = llm.llm.embed(vec![input]).await?;
    let len = result.len();
    let result = result.pop();
    match (len, result) {
        (1, Some(result)) => Ok(result),
        (0, _) => Err(anyhow!("didn't get any embedding results")),
        _ => Err(anyhow!("got multiple embedding results when we only expected one")),
    }
}

fn get_openai_api_key() -> Result<String> {
    assert_env_var("OPENAI_API_KEY")
}

fn get_ollama_base_url() -> String {
    "http://127.0.0.1:11434".to_owned()
}
