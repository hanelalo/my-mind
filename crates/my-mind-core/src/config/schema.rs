use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub asr: AsrConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    /// "online", "offline", or "auto"
    #[serde(default = "default_asr_mode")]
    pub mode: String,
    /// "zh", "en", or "auto"
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub online: AsrOnlineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrOnlineConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_base_url: Option<String>,
    /// ASR model name, e.g. "whisper-1", "FunAudioLLM/SenseVoiceSmall"
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// "openai", "anthropic", "ollama", "custom"
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_base_url: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u16,
    /// Enable LLM post-processing
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Global shortcut for recording (default: "Option+Space")
    #[serde(default = "default_record_shortcut")]
    pub record: String,
    /// Recording mode: "push_to_talk" or "toggle"
    #[serde(default = "default_record_mode")]
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Automatically paste after processing
    #[serde(default = "default_true")]
    pub auto_paste: bool,
}

// Default value functions
fn default_asr_mode() -> String { "online".to_string() }
fn default_language() -> String { "zh".to_string() }
fn default_llm_provider() -> String { "openai".to_string() }
fn default_llm_model() -> String { "gpt-4o-mini".to_string() }
fn default_temperature() -> f32 { 0.3 }
fn default_max_tokens() -> u16 { 2048 }
fn default_true() -> bool { true }
fn default_record_shortcut() -> String { "Alt+Space".to_string() }
fn default_record_mode() -> String { "push_to_talk".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            asr: AsrConfig::default(),
            llm: LlmConfig::default(),
            shortcuts: ShortcutConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            mode: default_asr_mode(),
            language: default_language(),
            online: AsrOnlineConfig::default(),
        }
    }
}

impl Default for AsrOnlineConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base_url: None,
            model: None,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_llm_provider(),
            model: default_llm_model(),
            api_key: String::new(),
            api_base_url: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            enabled: true,
        }
    }
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            record: default_record_shortcut(),
            mode: default_record_mode(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            auto_paste: true,
        }
    }
}

impl AppConfig {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?;
        Ok(config_dir.join("my-mind").join("config.toml"))
    }

    /// Load config from file, or return default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
