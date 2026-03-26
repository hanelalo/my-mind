pub mod state;

pub use state::PipelineState;

use crate::asr::AsrEngine;
use crate::audio::encoder::{resample_linear, WavEncoder};
use crate::audio::{AudioCapture, AudioCaptureConfig};
use crate::llm::LlmProvider;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Pipeline event sent to the frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PipelineEvent {
    /// Recording started
    RecordingStarted,
    /// Pipeline state changed
    StateChanged(String),
    /// ASR raw transcript available
    AsrResult(String),
    /// LLM post-processed result
    LlmResult(String),
    /// Pipeline completed with final text
    Done(String),
    /// Error occurred
    Error(String),
}

/// Main pipeline orchestrator
pub struct Pipeline {
    asr_engine: Arc<dyn AsrEngine>,
    llm_provider: Option<Arc<dyn LlmProvider>>,
    audio_config: AudioCaptureConfig,
}

impl Pipeline {
    pub fn new(
        asr_engine: Arc<dyn AsrEngine>,
        llm_provider: Option<Arc<dyn LlmProvider>>,
    ) -> Self {
        Self {
            asr_engine,
            llm_provider,
            audio_config: AudioCaptureConfig::default(),
        }
    }

    /// Run the full pipeline: record → ASR → LLM → return final text
    /// The event_tx is used to push status updates to the frontend.
    pub async fn run(
        &self,
        mut stop_rx: mpsc::Receiver<()>,
        event_tx: mpsc::UnboundedSender<PipelineEvent>,
    ) -> Result<String> {
        // 1. Start recording
        let capture = AudioCapture::new(self.audio_config.clone());
        let (mut audio_rx, handle) = capture.start()?;
        let _ = event_tx.send(PipelineEvent::RecordingStarted);
        let _ = event_tx.send(PipelineEvent::StateChanged("recording".to_string()));

        // 2. Collect audio chunks until stop signal
        let mut all_samples: Vec<i16> = Vec::new();
        let device_rate = handle.device_sample_rate;
        let target_rate = handle.target_sample_rate;

        loop {
            tokio::select! {
                chunk = audio_rx.recv() => {
                    match chunk {
                        Some(c) => all_samples.extend_from_slice(&c.samples),
                        None => break,
                    }
                }
                _ = stop_rx.recv() => {
                    info!("Stop signal received");
                    handle.stop();
                    // Drain remaining chunks
                    while let Ok(c) = audio_rx.try_recv() {
                        all_samples.extend_from_slice(&c.samples);
                    }
                    break;
                }
            }
        }

        if all_samples.is_empty() {
            let _ = event_tx.send(PipelineEvent::Error("No audio recorded".to_string()));
            anyhow::bail!("No audio recorded");
        }

        info!("Recorded {} samples at {}Hz", all_samples.len(), device_rate);

        // 3. Resample if needed
        let _ = event_tx.send(PipelineEvent::StateChanged("processing".to_string()));
        let samples = if device_rate != target_rate {
            info!("Resampling from {}Hz to {}Hz", device_rate, target_rate);
            resample_linear(&all_samples, device_rate, target_rate)
        } else {
            all_samples
        };

        // 4. Encode to WAV
        let wav_bytes = WavEncoder::encode(&samples, target_rate)?;

        // 5. ASR
        let _ = event_tx.send(PipelineEvent::StateChanged("transcribing".to_string()));
        let asr_result = self.asr_engine.transcribe(&wav_bytes, Some("zh")).await?;
        info!("ASR result: {}", asr_result.text);
        let _ = event_tx.send(PipelineEvent::AsrResult(asr_result.text.clone()));

        if asr_result.text.trim().is_empty() {
            let _ = event_tx.send(PipelineEvent::Done(String::new()));
            return Ok(String::new());
        }

        // 6. LLM post-processing (if enabled)
        let final_text = if let Some(llm) = &self.llm_provider {
            let _ = event_tx.send(PipelineEvent::StateChanged("post_processing".to_string()));
            match llm.post_process(&asr_result.text).await {
                Ok(processed) => {
                    info!("LLM processed: {}", processed);
                    let _ = event_tx.send(PipelineEvent::LlmResult(processed.clone()));
                    processed
                }
                Err(e) => {
                    error!("LLM post-processing failed, using raw ASR: {}", e);
                    let _ = event_tx.send(PipelineEvent::Error(format!(
                        "LLM failed, using raw transcript: {}",
                        e
                    )));
                    asr_result.text
                }
            }
        } else {
            asr_result.text
        };

        let _ = event_tx.send(PipelineEvent::Done(final_text.clone()));
        Ok(final_text)
    }
}
