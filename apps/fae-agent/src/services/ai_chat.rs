use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub agent_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AgentChatRequest {
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub message: String,
}

pub struct AIChatService {}

impl AIChatService {
    pub fn new(_db_pool: &SqlitePool) -> Self {
        Self {}
    }

    pub async fn get_agent_by_id(&self, _agent_id: &str) -> Result<crate::models::Agent, sqlx::Error> {
        // We don't need to access the DB in our current streaming implementation.
        // This function exists to keep struct compatibility.
        todo!("Not used in current implementation")
    }
}