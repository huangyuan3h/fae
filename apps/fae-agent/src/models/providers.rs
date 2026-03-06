use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow, PartialEq)]
pub struct ProviderType(String);

impl ProviderType {
    pub const OLLAMA: &'static str = "ollama";
    pub const OPENAI: &'static str = "openai";
    pub const GOOGLE: &'static str = "google";
    pub const ALIBABA: &'static str = "alibaba";

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(s: &str) -> Option<Self> {
        match s {
            "ollama" | "openai" | "google" | "alibaba" => Some(ProviderType(s.to_string())),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String, // Should be validated to match ProviderType constants
    pub api_key: String,
    pub base_url: String,
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub provider_configs: Vec<ProviderConfig>,
    pub default_provider: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfigRequest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub api_key: String,
    pub base_url: String,
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProviderSettingsRequest {
    pub provider_configs: Vec<ProviderConfigRequest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatPayload {
    pub agent_id: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub message: String,
    pub model: String,
    pub timestamp: i64,
}

// Default values for various providers
impl ProviderConfig {
    pub fn ollama_default() -> Self {
        ProviderConfig {
            id: "default-ollama".to_string(),
            name: "Local Ollama".to_string(),
            provider_type: ProviderType::OLLAMA.to_string(),
            api_key: "".to_string(),
            base_url: "http://127.0.0.1:11434".to_string(),
            model_id: "qwen3:8b".to_string(),
            enabled: true,
        }
    }

    pub fn openai_default() -> Self {
        ProviderConfig {
            id: "default-openai".to_string(),
            name: "OpenAI Default".to_string(),
            provider_type: ProviderType::OPENAI.to_string(),
            api_key: "".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model_id: "gpt-4o-mini".to_string(),
            enabled: true,
        }
    }

    pub fn google_default() -> Self {
        ProviderConfig {
            id: "default-google".to_string(),
            name: "Google Gemini".to_string(),
            provider_type: ProviderType::GOOGLE.to_string(),
            api_key: "".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            model_id: "gemini-2.5-flash".to_string(),
            enabled: true,
        }
    }

    pub fn alibaba_default() -> Self {
        ProviderConfig {
            id: "default-alibaba".to_string(),
            name: "Alibaba Qwen".to_string(),
            provider_type: ProviderType::ALIBABA.to_string(),
            api_key: "".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model_id: "qwen-turbo".to_string(),
            enabled: true,
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
