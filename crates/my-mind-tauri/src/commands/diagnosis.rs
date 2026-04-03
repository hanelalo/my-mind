use crate::state::AppState;
use my_mind_core::llm::{
    anthropic::AnthropicProvider, openai::OpenAiProvider, ChatMessage, LlmProvider, MessageRole,
    PROMPT_DIAGNOSIS_SYSTEM,
};
use std::sync::Arc;
use tauri::State;

/// Request for prompt diagnosis
#[derive(serde::Deserialize)]
pub struct DiagnosisRequest {
    pub asr_text: String,
    pub final_text: String,
    pub user_message: String,
    pub conversation_history: Vec<DiagnosisMessage>,
}

/// A message in the diagnosis conversation
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct DiagnosisMessage {
    pub role: String,
    pub content: String,
}

/// Response from prompt diagnosis
#[derive(serde::Serialize)]
pub struct DiagnosisResponse {
    pub reply: String,
}

/// Diagnose a history record and get suggestions for prompt improvement
#[tauri::command]
pub async fn diagnose_prompt(
    state: State<'_, AppState>,
    request: DiagnosisRequest,
) -> Result<DiagnosisResponse, String> {
    let config = state.config.lock().await;
    let llm_config = &config.llm;

    // Clone necessary values before dropping the lock
    let provider_type = llm_config.provider.clone();
    let api_key = llm_config.api_key.clone();
    let api_base_url = llm_config.api_base_url.clone();
    let model = llm_config.model.clone();
    let temperature = llm_config.temperature;
    let max_tokens = llm_config.max_tokens;
    let effective_prompt = llm_config.effective_prompt().to_string();

    // Create LLM provider based on configuration
    let provider: Arc<dyn LlmProvider> = match provider_type.as_str() {
        "openai" => {
            if api_key.is_empty() {
                return Err("OpenAI API key is not configured".to_string());
            }
            Arc::new(OpenAiProvider::new(
                api_key,
                api_base_url,
                Some(model),
                Some(temperature),
                Some(max_tokens),
                effective_prompt.clone(),
            ))
        }
        "anthropic" => {
            if api_key.is_empty() {
                return Err("Anthropic API key is not configured".to_string());
            }
            Arc::new(AnthropicProvider::new(
                api_key,
                api_base_url,
                Some(model),
                Some(temperature),
                Some(max_tokens),
                effective_prompt.clone(),
            ))
        }
        _ => {
            return Err(format!(
                "LLM provider '{}' is not supported for diagnosis",
                provider_type
            ));
        }
    };

    drop(config);

    // Build the conversation messages
    let mut messages: Vec<ChatMessage> = vec![ChatMessage {
        role: MessageRole::System,
        content: PROMPT_DIAGNOSIS_SYSTEM.to_string(),
    }];

    // Add context about the current prompt and the record being diagnosed
    let context_message = format!(
        "I'm diagnosing an issue with my speech-to-text post-processing. Here's the context:

**Current Post-Processing Prompt:**
```
{}
```

**Original ASR Transcript:**
```
{}
```

**Final Processed Output:**
```
{}
```",
        effective_prompt,
        request.asr_text,
        request.final_text
    );

    messages.push(ChatMessage {
        role: MessageRole::User,
        content: context_message,
    });

    // Add conversation history
    for msg in request.conversation_history {
        let role = match msg.role.as_str() {
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
        };
        messages.push(ChatMessage {
            role,
            content: msg.content,
        });
    }

    // Add the current user message
    messages.push(ChatMessage {
        role: MessageRole::User,
        content: request.user_message,
    });

    // Send to LLM
    match provider.chat(messages).await {
        Ok(reply) => Ok(DiagnosisResponse { reply }),
        Err(e) => Err(format!("Failed to get diagnosis: {}", e)),
    }
}
