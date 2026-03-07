use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Define the exact shape needed to match Next.js expectations in JSON response
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct ProviderSettings {
    #[serde(rename = "providerConfigs")] // Map Rust snake_case field to Next.js camelCase property
    pub provider_configs: Vec<ProviderConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String, // Should be validated to match ProviderType constants
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ProviderConfigRequest {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "modelId")]
    pub model_id: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateProviderSettingsRequest {
    #[serde(rename = "providerConfigs")]
    pub provider_configs: Vec<ProviderConfigRequest>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatPayload {
    pub agent_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
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
            provider_type: "ollama".to_string(),
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
            provider_type: "openai".to_string(),
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
            provider_type: "google".to_string(),
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
            provider_type: "alibaba".to_string(),
            api_key: "".to_string(),
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            model_id: "qwen-turbo".to_string(),
            enabled: true,
        }
    }
}
