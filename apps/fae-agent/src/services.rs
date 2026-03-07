use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::models::ChatCompletion;

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub timestamp: i64,
}

pub async fn status_handler(
    State(_state): State<crate::AppState>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let status = StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    Ok(Json(status))
}

#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub agent_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AgentChatRequest {
    #[serde(rename = "agentId")]  // Maps from camelCase 'agentId' in JSON to snake_case field
    pub agent_id: String,
    pub message: String,
}

pub async fn chat_stream_handler(
    State(_state): State<crate::AppState>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatCompletion>, StatusCode> {
    info!("Received chat request for agent: {}", payload.agent_id);
    
    // Placeholder - would normally interact with DB through state
    // For now, return mock response
    let model = "mock-model".to_string();
    
    let response_message = format!(
        "Processed chat request for agent {}: {}",
        payload.agent_id, payload.message
    );
    
    let response = ChatCompletion {
        id: uuid::Uuid::new_v4().to_string(),
        message: response_message,
        model,
        timestamp: chrono::Utc::now().timestamp(),
    };

    Ok(Json(response))
}

// Handler specifically for Next.js client expectations (camelCase params)
pub async fn agent_chat_handler(
    State(_state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Result<Json<ChatCompletion>, StatusCode> {
    info!("Received agent chat request for agent: {}", payload.agent_id);
    
    // Placeholder - same response just different parameter name
    let model = "mock-model".to_string();
    
    let response_message = format!(
        "Processed agent chat request for {}: {}",
        payload.agent_id, payload.message
    );
    
    let response = ChatCompletion {
        id: uuid::Uuid::new_v4().to_string(),
        message: response_message,
        model,
        timestamp: chrono::Utc::now().timestamp(),
    };

    Ok(Json(response))
}

pub async fn chat_ws_handler(
    State(_state): State<crate::AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|websocket| async move {
        handle_socket(websocket).await;
    })
}

async fn handle_socket(mut websocket: axum::extract::ws::WebSocket) {
    while let Some(msg) = websocket.recv().await {
        if let Ok(axum::extract::ws::Message::Text(text)) = msg {
            // Process chat message
            let response = format!("Echo via WS: {}", text);
            
            if websocket.send(axum::extract::ws::Message::Text(response)).await.is_err() {
                // Client disconnected
                break;
            }
        } else if msg.is_err() {
            // Error occurred
            break;
        }
    }
}

// Initialize database function remains for future use
pub async fn initialize_db(_db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(()) // Handled in main now
}

// Export needed for API access
pub use providers::ProviderResolver; // Export provider resolver itself  
pub use providers_api::{ 
    get_providers_handler, 
    update_providers_handler, 
    get_ollama_settings_handler, 
    update_ollama_settings_handler 
};

pub mod providers;      // Provider core logic
pub mod providers_api;  // Provider API handlers
pub mod skills;         // Skills functionality
pub mod skills_api;     // Skill API handlers