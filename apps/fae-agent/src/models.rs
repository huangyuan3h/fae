// Database models will go here
// Example structure for future use:

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a message in the system
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
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

pub mod providers;
