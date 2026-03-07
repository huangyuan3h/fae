// Database models will go here
// Example structure for future use:

use serde::{Deserialize, Serialize};

/// Represents a message in the system
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    /// Unique identifier for the message
    pub id: i32,
    /// Content of the message
    pub message: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request payload for chat interactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatPayload {
    /// Agent ID for the chat
    pub agent_id: String,
    /// User's message
    pub message: String,
}

/// Response payload for chat interactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    /// Unique identifier for the response
    pub id: String,
    /// Chat completion message
    pub message: String,
    /// Model used for completion
    pub model: String,
    /// Timestamp of creation
    pub timestamp: i64,
}

// Agent结构体定义，但不会直接从数据库行反序列化整个结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(rename = "provider_config_id")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "system_prompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatar_url")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub provider: String,
    #[serde(rename = "providerConfigId")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: String,
    pub provider: String,
    #[serde(rename = "providerConfigId")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
}

pub mod providers;
