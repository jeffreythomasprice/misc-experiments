use crate::env::assert_env_var;
use anyhow::{Result, anyhow};
use futures::future::BoxFuture;
use llm::{
    LLMProvider, ToolCall,
    builder::{FunctionBuilder, LLMBackend, LLMBuilder, ParamBuilder},
    chat::ChatMessage,
};
use tracing::*;

pub enum Provider {
    OpenAI,
    Ollama,
}

pub struct Tool {
    pub name: String,
    pub description: String,
    pub json_schema: Option<serde_json::Value>,
    pub callback: Box<dyn Fn(&ToolCall) -> BoxFuture<'static, Result<String>>>,
}

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
    tools: Vec<Tool>,
}

impl LLMs {
    pub async fn new(provider: Provider, system_prompt: &str, tools: Vec<Tool>) -> Result<Self> {
        Ok(Self {
            chat: create_llm(
                match provider {
                    Provider::OpenAI => create_openai_llm,
                    Provider::Ollama => create_ollama_llm,
                },
                system_prompt,
                &tools,
            )?,
            embedding: create_openai_embedding_llm().await?,
            tools,
        })
    }

    pub async fn handle_tool_calls(&self, tool_calls: Vec<ToolCall>) -> Result<ChatMessage> {
        let mut content = Vec::new();
        for tool_call in tool_calls.iter() {
            let result = self.handle_tool_call(&tool_call).await?;
            content.push(format!(
                "tool call id: {}\ntool name: {}\nresult: {}",
                tool_call.id, tool_call.function.name, result
            ));
        }
        trace!("combined tool call results: {:?}", content);
        Ok(ChatMessage::assistant()
            .tool_result(tool_calls)
            .content(content.join("\n\n"))
            .build())
    }

    pub async fn handle_tool_call(&self, tool_call: &ToolCall) -> Result<String> {
        trace!("tool call: {tool_call:?}");
        match self.tools.iter().find(|tool| tool.name == tool_call.function.name) {
            Some(tool) => {
                let result = (tool.callback)(tool_call).await?;
                trace!("tool call {} result = {}", tool_call.id, result);
                Ok(result)
            }
            None => Err(anyhow!("no tool found for tool call: {:?}", tool_call)),
        }
    }
}

pub async fn create_embedding(llm: &EmbeddingLLM, input: String) -> Result<Vec<f32>> {
    Ok(create_embedding_from_llm(&llm.llm, input).await?)
}

fn create_llm<F>(f: F, system_prompt: &str, tools: &[Tool]) -> Result<NamedLLM>
where
    F: FnOnce() -> Result<(String, LLMBuilder)>,
{
    let (name, llm_builder) = f()?;

    let mut llm_builder = llm_builder.system(system_prompt);

    for tool in tools {
        let mut function_builder = FunctionBuilder::new(tool.name.clone()).description(tool.description.clone());
        if let Some(json_schema) = tool.json_schema.clone() {
            function_builder = function_builder.json_schema(json_schema);
        }
        llm_builder = llm_builder.function(function_builder);
    }

    let llm = NamedLLM {
        llm: llm_builder.build()?,
        name,
    };
    Ok(llm)
}

fn create_openai_llm() -> Result<(String, LLMBuilder)> {
    let name = "gpt-5-nano".to_string();
    Ok((
        name.clone(),
        LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .model(name)
            .api_key(get_openai_api_key()?),
    ))
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

fn create_ollama_llm() -> Result<(String, LLMBuilder)> {
    let name = "llama3.2:1b".to_string();
    Ok((
        name.clone(),
        LLMBuilder::new()
            .backend(LLMBackend::Ollama)
            .base_url(get_ollama_base_url())
            .model(&name),
    ))
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
