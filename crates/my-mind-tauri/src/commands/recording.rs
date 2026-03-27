use crate::events;
use crate::state::AppState;
use my_mind_core::asr::whisper_api::WhisperApiEngine;
use my_mind_core::input::{ClipboardManager, FocusManager, InputSimulator};
use my_mind_core::llm::anthropic::AnthropicProvider;
use my_mind_core::llm::openai::OpenAiProvider;
use my_mind_core::pipeline::{Pipeline, PipelineEvent};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Inner function called by both Tauri command and global shortcut handler
pub async fn start_recording_inner(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    let mut is_recording = state.is_recording.lock().await;
    if *is_recording {
        return Err("Already recording".to_string());
    }
    *is_recording = true;
    drop(is_recording);

    let config = state.config.lock().await.clone();

    // Create stop channel
    let (stop_tx, stop_rx) = mpsc::channel::<()>(1);
    *state.pipeline_stop_tx.lock().await = Some(stop_tx);

    // Create event channel for frontend updates
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<PipelineEvent>();

    // Shared slot to capture ASR text for history saving
    let asr_text_captured: Arc<std::sync::Mutex<Option<String>>> =
        Arc::new(std::sync::Mutex::new(None));
    let asr_text_for_events = asr_text_captured.clone();

    // Build ASR engine
    let asr_engine: Arc<dyn my_mind_core::asr::AsrEngine> =
        Arc::new(WhisperApiEngine::new(
            config.asr.online.api_key.clone(),
            config.asr.online.api_base_url.clone(),
            config.asr.online.model.clone(),
        ));

    // Build LLM provider (if enabled)
    let llm_provider: Option<Arc<dyn my_mind_core::llm::LlmProvider>> = if config.llm.enabled {
        let prompt = config.llm.effective_prompt().to_string();
        let provider: Arc<dyn my_mind_core::llm::LlmProvider> = match config.llm.provider.as_str() {
            "anthropic" | "claude" => Arc::new(AnthropicProvider::new(
                config.llm.api_key.clone(),
                config.llm.api_base_url.clone(),
                Some(config.llm.model.clone()),
                Some(config.llm.temperature),
                Some(config.llm.max_tokens),
                prompt,
            )),
            // "openai", "openrouter", "alibaba" 等 OpenAI 兼容格式统一走 OpenAiProvider
            _ => Arc::new(OpenAiProvider::new(
                config.llm.api_key.clone(),
                config.llm.api_base_url.clone(),
                Some(config.llm.model.clone()),
                Some(config.llm.temperature),
                Some(config.llm.max_tokens),
                prompt,
            )),
        };
        Some(provider)
    } else {
        None
    };

    let pipeline = Pipeline::new(asr_engine, llm_provider);
    let auto_paste = config.output.auto_paste;

    // Forward pipeline events to Tauri frontend
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match &event {
                PipelineEvent::StateChanged(s) => {
                    let _ = app_handle.emit(events::EVENT_PIPELINE_STATE, s);
                }
                PipelineEvent::AsrResult(text) => {
                    *asr_text_for_events.lock().unwrap() = Some(text.clone());
                    let _ = app_handle.emit(events::EVENT_ASR_RESULT, text);
                }
                PipelineEvent::LlmResult(text) => {
                    let _ = app_handle.emit(events::EVENT_LLM_RESULT, text);
                }
                PipelineEvent::Done(text) => {
                    let _ = app_handle.emit(events::EVENT_PIPELINE_DONE, text);
                }
                PipelineEvent::Error(msg) => {
                    let _ = app_handle.emit(events::EVENT_PIPELINE_ERROR, msg);
                }
                _ => {}
            }
        }
    });

    // Run pipeline in background
    let app_handle = app.clone();
    tokio::spawn(async move {
        match pipeline.run(stop_rx, event_tx).await {
            Ok(final_text) => {
                info!("[output] Pipeline completed, text length={}, content: {:?}", final_text.len(), final_text);

                // Get previously active app for focus restoration
                let previous_app = {
                    let state = app_handle.state::<AppState>();
                    let result = state.previous_app.lock().unwrap().take();
                    result
                };

                // Save to history
                if !final_text.is_empty() {
                    let asr = asr_text_captured.lock().unwrap().take().unwrap_or_default();
                    let state = app_handle.state::<AppState>();
                    if let Err(e) = state.history.insert(&asr, &final_text, previous_app.as_deref()) {
                        error!("[history] Failed to save record: {}", e);
                    }
                }

                info!("[output] auto_paste={}, text_empty={}", auto_paste, final_text.is_empty());

                if auto_paste && !final_text.is_empty() {
                    info!("[output] Starting clipboard + paste flow...");

                    let mut clipboard = ClipboardManager::new();

                    info!("[output] Step 1/3: Saving current clipboard content...");
                    match clipboard.save() {
                        Ok(_) => info!("[output] Step 1/3: Clipboard saved successfully"),
                        Err(e) => error!("[output] Step 1/3: Failed to save clipboard: {}", e),
                    }

                    info!("[output] Step 2/3: Setting text to clipboard ({} chars)...", final_text.len());
                    match clipboard.set_text(&final_text) {
                        Ok(_) => {
                            info!("[output] Step 2/3: Text set to clipboard successfully");

                            // Step 3: Activate target app + paste
                            // Use spawn_blocking to avoid blocking the tokio runtime
                            // (activate_and_paste uses std::thread::sleep + blocking I/O)
                            info!("[output] Step 3/3: Activate + paste...");
                            let paste_bundle = previous_app.clone();
                            let paste_ok;
                            let paste_result = tokio::task::spawn_blocking(move || {
                                if let Some(ref bundle_id) = paste_bundle {
                                    InputSimulator::activate_and_paste(bundle_id)
                                } else {
                                    Err(anyhow::anyhow!("未检测到目标应用，文本已复制到剪贴板，请手动粘贴"))
                                }
                            }).await;
                            match paste_result {
                                Ok(Ok(_)) => {
                                    info!("[output] Step 3/3: Activate + paste succeeded");
                                    paste_ok = true;
                                }
                                Ok(Err(e)) => {
                                    error!("[output] Step 3/3: Activate + paste failed: {}", e);
                                    paste_ok = false;
                                    // Show error on overlay
                                    let _ = app_handle.emit(events::EVENT_PIPELINE_ERROR, e.to_string());
                                }
                                Err(e) => {
                                    error!("[output] Step 3/3: Paste task panicked: {}", e);
                                    paste_ok = false;
                                    let _ = app_handle.emit(events::EVENT_PIPELINE_ERROR, "粘贴异常，文本已复制到剪贴板，请手动粘贴".to_string());
                                }
                            }

                            if paste_ok {
                                // Paste succeeded — hide overlay, wait for target app to consume clipboard, then restore
                                if let Some(window) = app_handle.get_webview_window("overlay") {
                                    let _ = window.hide();
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                info!("[output] Restoring original clipboard...");
                                match clipboard.restore() {
                                    Ok(_) => info!("[output] Clipboard restored successfully"),
                                    Err(e) => error!("[output] Failed to restore clipboard: {}", e),
                                }
                            } else {
                                // Paste failed — keep text in clipboard, overlay stays visible
                                // until user presses Esc or starts a new recording
                                warn!("[output] Paste failed, leaving text in clipboard for manual paste");
                            }
                        }
                        Err(e) => {
                            error!("[output] Step 2/3: Failed to set clipboard text: {}", e);
                            let _ = app_handle.emit(events::EVENT_PIPELINE_ERROR, format!("剪贴板写入失败: {}", e));
                        }
                    }

                    info!("[output] Clipboard + paste flow completed");
                } else {
                    // No paste needed, hide overlay and restore focus
                    if let Some(window) = app_handle.get_webview_window("overlay") {
                        let _ = window.hide();
                    }
                    if let Some(ref bundle_id) = previous_app {
                        info!("[focus] Restoring focus to: {}", bundle_id);
                        if let Err(e) = FocusManager::activate_app(bundle_id) {
                            warn!("[focus] Failed to restore focus: {}", e);
                        }
                    }
                    info!("[output] Skipping paste (auto_paste={}, text_empty={})", auto_paste, final_text.is_empty());
                }
            }
            Err(e) => {
                error!("Pipeline failed: {}", e);
                let _ = app_handle.emit(events::EVENT_PIPELINE_ERROR, e.to_string());

                // Show error briefly then hide overlay
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if let Some(window) = app_handle.get_webview_window("overlay") {
                    let _ = window.hide();
                }

                // Restore focus to the previously active application
                let previous_app = {
                    let state = app_handle.state::<AppState>();
                    let result = state.previous_app.lock().unwrap().take();
                    result
                };
                if let Some(ref bundle_id) = previous_app {
                    info!("[focus] Restoring focus after error to: {}", bundle_id);
                    if let Err(e) = FocusManager::activate_app(bundle_id) {
                        warn!("[focus] Failed to restore focus: {}", e);
                    }
                }
            }
        }
        // Reset recording state
        let state = app_handle.state::<AppState>();
        *state.is_recording.lock().await = false;
        *state.pipeline_stop_tx.lock().await = None;
    });

    Ok(())
}

/// Inner function to stop recording, called from shortcut handler
pub async fn stop_recording_inner(state: &AppState) -> Result<(), String> {
    let stop_tx = state.pipeline_stop_tx.lock().await;
    if let Some(tx) = stop_tx.as_ref() {
        tx.send(()).await.map_err(|e| e.to_string())?;
        info!("Stop signal sent");
    }
    Ok(())
}

/// Cancel recording and hide the overlay window
pub async fn cancel_recording_inner(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    // Stop pipeline if running
    let is_recording = *state.is_recording.lock().await;
    if is_recording {
        stop_recording_inner(&state).await?;
    }

    // Hide overlay window
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }

    // Restore focus to the previously active application
    let previous_app = state.previous_app.lock().unwrap().take();
    if let Some(ref bundle_id) = previous_app {
        info!("[focus] Restoring focus after cancel to: {}", bundle_id);
        if let Err(e) = FocusManager::activate_app(bundle_id) {
            warn!("[focus] Failed to restore focus: {}", e);
        }
    }

    // Reset state
    *state.is_recording.lock().await = false;
    *state.pipeline_stop_tx.lock().await = None;

    info!("Recording cancelled");
    Ok(())
}

#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    start_recording_inner(&app).await
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    stop_recording_inner(&state).await
}

#[tauri::command]
pub async fn cancel_recording(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    cancel_recording_inner(&app).await
}

#[tauri::command]
pub async fn is_recording(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_recording.lock().await)
}
