use anyhow::{Result, anyhow};
use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
};

use crate::env::assert_env_var;

pub struct EmbeddingLLM {
    pub llm: Box<dyn LLMProvider>,
    pub context: u32,
}

pub fn create_openai_llm() -> Result<Box<dyn LLMProvider>> {
    Ok(LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .model("gpt-5-nano")
        .api_key(assert_env_var("OPENAI_API_KEY")?)
        .build()?)
}

pub fn create_ollama_llm() -> Result<Box<dyn LLMProvider>> {
    Ok(LLMBuilder::new()
        .backend(LLMBackend::Ollama)
        .base_url(get_ollama_base_url())
        .model("llama3.2:1b")
        // TODO do we need updated params?
        // .max_tokens(1000)
        // .temperature(0.5)
        .build()?)
}

pub fn create_ollama_embedding_llm() -> Result<EmbeddingLLM> {
    Ok(EmbeddingLLM {
        llm: LLMBuilder::new()
            .backend(LLMBackend::Ollama)
            .base_url(get_ollama_base_url())
            .model("bge-m3:latest")
            .build()?,
        context: 2048,
    })
}

pub async fn create_embedding(llm: &Box<dyn LLMProvider>, input: String) -> Result<Vec<f32>> {
    let mut result = llm.embed(vec![input]).await?;
    let len = result.len();
    let result = result.pop();
    match (len, result) {
        (1, Some(result)) => Ok(result),
        (0, _) => Err(anyhow!("didn't get any embedding results")),
        _ => Err(anyhow!("got multiple embedding results when we only expected one")),
    }
}

fn get_ollama_base_url() -> String {
    "http://127.0.0.1:11434".to_owned()
}
