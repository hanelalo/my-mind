use super::{ChatMessage, LlmProvider, MessageRole};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: u16,
    prompt: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ApiChatMessage>,
    temperature: f32,
    max_tokens: u16,
}

#[derive(Serialize)]
struct ApiChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ApiResponseMessage,
}

#[derive(Deserialize)]
struct ApiResponseMessage {
    content: Option<String>,
}

impl OpenAiProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u16>,
        prompt: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            temperature: temperature.unwrap_or(0.3),
            max_tokens: max_tokens.unwrap_or(2048),
            prompt,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn post_process(&self, raw_transcript: &str) -> Result<String> {
        if raw_transcript.trim().is_empty() {
            return Ok(String::new());
        }

        let url = format!("{}/chat/completions", self.base_url);

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ApiChatMessage {
                    role: "system".to_string(),
                    content: self.prompt.clone(),
                },
                ApiChatMessage {
                    role: "user".to_string(),
                    content: raw_transcript.to_string(),
                },
            ],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        debug!(
            "Sending transcript to LLM ({} chars, model: {})",
            raw_transcript.len(),
            self.model
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error ({}): {}", status, error_body);
        }

        let result: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        let text = result
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        info!("LLM post-processed: \"{}\"", text);
        Ok(text)
    }

    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        if messages.is_empty() {
            return Ok(String::new());
        }

        let url = format!("{}/chat/completions", self.base_url);

        let api_messages: Vec<ApiChatMessage> = messages
            .into_iter()
            .map(|m| ApiChatMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: api_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        debug!("Sending chat request to LLM (model: {})", self.model);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request to LLM API")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM API error ({}): {}", status, error_body);
        }

        let result: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        let text = result
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        info!("LLM chat response received");
        Ok(text)
    }
}
