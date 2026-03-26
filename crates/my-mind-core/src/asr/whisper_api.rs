use super::{AsrEngine, TranscribeResult};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::multipart;
use serde::Deserialize;
use tracing::{debug, info};

pub struct WhisperApiEngine {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    language: Option<String>,
}

impl WhisperApiEngine {
    pub fn new(api_key: String, base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "whisper-1".to_string()),
        }
    }
}

#[async_trait]
impl AsrEngine for WhisperApiEngine {
    async fn transcribe(
        &self,
        audio_wav: &[u8],
        language: Option<&str>,
    ) -> Result<TranscribeResult> {
        let url = format!("{}/audio/transcriptions", self.base_url);

        let file_part = multipart::Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let mut form = multipart::Form::new()
            .part("file", file_part)
            .text("model", self.model.clone());

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        debug!("Sending audio to Whisper API ({} bytes)", audio_wav.len());

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .context("Failed to send request to Whisper API")?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            anyhow::bail!("Whisper API error ({}): {}", status, error_body);
        }

        let body = response
            .text()
            .await
            .context("Failed to read Whisper API response")?;

        // Try parsing as JSON first, fall back to plain text
        let (text, language) = if let Ok(parsed) = serde_json::from_str::<WhisperResponse>(&body) {
            (parsed.text, parsed.language)
        } else {
            (body.trim().to_string(), None)
        };

        info!("Whisper transcription: \"{}\"", text);

        Ok(TranscribeResult { text, language })
    }
}
