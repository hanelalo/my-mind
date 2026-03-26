use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SampleRate, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Raw audio chunk from microphone
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// PCM samples, mono, i16
    pub samples: Vec<i16>,
    /// Sample rate in Hz
    pub sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct AudioCaptureConfig {
    /// Target sample rate (default: 16000 for Whisper)
    pub sample_rate: u32,
}

impl Default for AudioCaptureConfig {
    fn default() -> Self {
        Self { sample_rate: 16000 }
    }
}

pub struct AudioCapture {
    config: AudioCaptureConfig,
}

impl AudioCapture {
    pub fn new(config: AudioCaptureConfig) -> Self {
        Self { config }
    }

    /// Get the default input device
    fn get_input_device() -> Result<Device> {
        let host = cpal::default_host();
        host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))
    }

    /// Start recording and return a receiver for audio chunks.
    /// The cpal Stream is kept alive on a dedicated thread (it is not Send).
    pub fn start(&self) -> Result<(mpsc::UnboundedReceiver<AudioChunk>, AudioCaptureHandle)> {
        let device = Self::get_input_device()?;
        let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
        info!("Using input device: {}", device_name);

        let supported_config = device.default_input_config()?;
        let sample_format = supported_config.sample_format();
        let device_sample_rate = supported_config.sample_rate().0;
        debug!(
            "Device config: format={:?}, rate={}, channels={}",
            sample_format,
            device_sample_rate,
            supported_config.channels()
        );

        let stream_config = StreamConfig {
            channels: 1, // mono
            sample_rate: SampleRate(device_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = mpsc::unbounded_channel::<AudioChunk>();
        let is_recording = Arc::new(AtomicBool::new(true));
        let recording_flag = is_recording.clone();
        let target_sample_rate = self.config.sample_rate;

        // Spawn a dedicated thread to own the cpal Stream (which is !Send)
        std::thread::spawn(move || {
            let err_fn = |err| {
                error!("Audio stream error: {}", err);
            };

            let recording_for_cb = recording_flag.clone();

            let stream_result = match sample_format {
                SampleFormat::I16 => {
                    let tx = tx;
                    device.build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            if !recording_for_cb.load(Ordering::SeqCst) {
                                return;
                            }
                            let chunk = AudioChunk {
                                samples: data.to_vec(),
                                sample_rate: device_sample_rate,
                            };
                            let _ = tx.send(chunk);
                        },
                        err_fn,
                        None,
                    )
                }
                SampleFormat::F32 => {
                    let tx = tx;
                    let recording_for_cb = recording_flag.clone();
                    device.build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            if !recording_for_cb.load(Ordering::SeqCst) {
                                return;
                            }
                            let samples: Vec<i16> = data
                                .iter()
                                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                                .collect();
                            let chunk = AudioChunk {
                                samples,
                                sample_rate: device_sample_rate,
                            };
                            let _ = tx.send(chunk);
                        },
                        err_fn,
                        None,
                    )
                }
                _ => {
                    error!("Unsupported sample format: {:?}", sample_format);
                    return;
                }
            };

            match stream_result {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        error!("Failed to play stream: {}", e);
                        return;
                    }
                    info!(
                        "Recording started (device rate: {}Hz, target: {}Hz)",
                        device_sample_rate, target_sample_rate
                    );
                    // Keep the stream alive until recording is stopped
                    while recording_flag.load(Ordering::SeqCst) {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    // Stream is dropped here
                    info!("Audio stream thread exiting");
                }
                Err(e) => {
                    error!("Failed to build input stream: {}", e);
                }
            }
        });

        let handle = AudioCaptureHandle {
            is_recording,
            device_sample_rate,
            target_sample_rate,
        };

        Ok((rx, handle))
    }
}

/// Handle to control recording. This is Send+Sync safe because
/// the cpal Stream lives on a dedicated thread.
pub struct AudioCaptureHandle {
    is_recording: Arc<AtomicBool>,
    pub device_sample_rate: u32,
    pub target_sample_rate: u32,
}

impl AudioCaptureHandle {
    /// Stop the recording
    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::SeqCst);
        info!("Recording stopped");
    }
}
