pub mod anthropic;
pub mod openai;
pub mod prompts;

pub use prompts::PROMPT_DIAGNOSIS_SYSTEM;

use anyhow::Result;
use async_trait::async_trait;

/// A message in a chat conversation
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Role of a message sender
#[derive(Debug, Clone)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// LLM provider trait for post-processing
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Process raw transcript through LLM
    async fn post_process(&self, raw_transcript: &str) -> Result<String>;

    /// Send a chat conversation to the LLM and get a response
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
}
