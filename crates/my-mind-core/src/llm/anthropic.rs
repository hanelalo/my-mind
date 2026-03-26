use super::{prompts, LlmProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    temperature: f32,
    max_tokens: u16,
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    system: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u16,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

impl AnthropicProvider {
    pub fn new(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        temperature: Option<f32>,
        max_tokens: Option<u16>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            temperature: temperature.unwrap_or(0.3),
            max_tokens: max_tokens.unwrap_or(2048),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn post_process(&self, raw_transcript: &str) -> Result<String> {
        if raw_transcript.trim().is_empty() {
            return Ok(String::new());
        }

        let url = format!("{}/v1/messages", self.base_url);

        let request = MessagesRequest {
            model: self.model.clone(),
            system: prompts::POST_PROCESS_PROMPT.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: raw_transcript.to_string(),
            }],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        debug!(
            "Sending transcript to Anthropic ({} chars, model: {})",
            raw_transcript.len(),
            self.model
        );

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, error_body);
        }

        let result: MessagesResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        let text = result
            .content
            .iter()
            .filter(|b| b.content_type == "text")
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join("");

        info!("Anthropic post-processed: \"{}\"", text);
        Ok(text)
    }
}
