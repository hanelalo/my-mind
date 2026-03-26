pub mod anthropic;
pub mod openai;
pub mod prompts;

use anyhow::Result;
use async_trait::async_trait;

/// LLM provider trait for post-processing
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Process raw transcript through LLM
    async fn post_process(&self, raw_transcript: &str) -> Result<String>;
}
