pub mod whisper_api;

use anyhow::Result;
use async_trait::async_trait;

/// Result of ASR transcription
#[derive(Debug, Clone)]
pub struct TranscribeResult {
    pub text: String,
    pub language: Option<String>,
}

/// ASR engine trait - unified interface for online/offline engines
#[async_trait]
pub trait AsrEngine: Send + Sync {
    /// Transcribe complete audio (batch mode)
    async fn transcribe(&self, audio_wav: &[u8], language: Option<&str>) -> Result<TranscribeResult>;
}
