use serde::{Deserialize, Serialize};

/// Pipeline state machine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineState {
    /// Waiting for user to trigger recording
    Idle,
    /// Currently recording audio
    Recording,
    /// Processing: ASR transcription in progress
    Transcribing,
    /// Processing: LLM post-processing in progress
    PostProcessing,
    /// Done, final text ready
    Done,
    /// Error occurred
    Error,
}

impl std::fmt::Display for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineState::Idle => write!(f, "idle"),
            PipelineState::Recording => write!(f, "recording"),
            PipelineState::Transcribing => write!(f, "transcribing"),
            PipelineState::PostProcessing => write!(f, "post_processing"),
            PipelineState::Done => write!(f, "done"),
            PipelineState::Error => write!(f, "error"),
        }
    }
}
