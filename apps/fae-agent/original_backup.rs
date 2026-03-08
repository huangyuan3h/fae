use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    Json,
};
use axum::response::sse::{Event, KeepAlive};
use futures_util::Stream;
use std::time::Duration;
use tokio::time::sleep;
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

// Define the types of events in a stream
#[derive(Debug, Clone, Serialize)]
pub enum ChatStreamEvent {
    Chunk { content: String },
    Thinking { content: String },
    ToolInputStart { tool_call_id: String, tool_name: String },
    ToolInputDelta { tool_call_id: String, delta: String },
    ToolCall { tool_call_id: String, tool_name: String, input: serde_json::Value },
    ToolResult { tool_call_id: String, output: serde_json::Value },
    ToolError { tool_call_id: String, message: String },
    Done,
    Error { message: String },
}

pub async fn chat_stream_handler(
    State(state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    // Convert agent_id to lowercase for lookup purposes
    let _agent_id = payload.agent_id;
    let message = payload.message;
    
    // For this demo implementation, we'll simulate responses using streams
    let stream = async_stream::stream! {
        yield Ok(Event::default().event("think").data(&format!("Thinking about your question: \"{}\"...", message)));
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        yield Ok(Event::default().event("think").data("Identifying necessary actions..."));
        
        // Yield simulated thinking traces and responses in chunks
        let response_parts = [
            "Okay, I understand. ",
            " Let me process your request. ",
            " Based on your query: ",
            &format!("\"{}\", ", message),
            "I can provide some insights. "
        ];
        
        // Yield initial partial response
        for part in response_parts {
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield Ok(Event::default().event("chunk").data(part.to_string()));
        }
        
        // Add complete response in chunks
        let full_response = format!("This is a complete response to your query about '{}'. The system has processed your request successfully.", message);
        let words: Vec<&str> = full_response.split(' ').collect();
        
        for word in words {
            tokio::time::sleep(Duration::from_millis(50)).await;
            yield Ok(Event::default().event("chunk").data(format!("{} ", word)));
        }
        
        // Add tool simulation
        let tool_call_id = format!("call_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        yield Ok(Event::default()
            .event("tool-call")
            .data(format!(
                r#"{{"toolCallId": "{}", "toolName": "search_tool", "input": {{"query": "{}"}}}}"#,
                tool_call_id, message
            )));
            
        tokio::time::sleep(Duration::from_millis(600)).await;
        
        yield Ok(Event::default()
            .event("tool-result")
            .data(format!(
                r#"{{"toolCallId": "{}", "output": {{"result": "Simulated search results for '{}'"}}}}"#, 
                tool_call_id, message
            )));
        
        // Send completion signals
        tokio::time::sleep(Duration::from_millis(300)).await;
        yield Ok(Event::default().event("done").data("[DONE]"));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Handler specifically for Next.js client expectations (camelCase params) - Returns JSON
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

// Handler for streaming chat responses to match /api/chat/stream expectation - returns SSE
pub async fn agent_stream_chat_handler(
// Handler for streaming chat responses to match /api/chat/stream expectation - returns SSE
pub async fn agent_stream_chat_handler(
    State(state): State<crate::AppState>,
    AxumJson(payload): AxumJson<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let agent_id = payload.agent_id;
    let message = payload.message;
    
    // For now simulating based on skills in a realistic manner
    // Later could extend with real agent lookups
    let has_skills = agent_id.contains("skill") || message.to_lowercase().contains("skill");
    
    let stream = async_stream::stream! {
        // Send thinking stages to simulate AI processing
        let thinking_steps = vec![
            "Processing user request. ",
            "Analyzing the query: \"" + &message + "\". ", 
            "Preparing a structured response. "
        ];
        
        for step in thinking_steps {
            tokio::time::sleep(Duration::from_millis(200)).await;
            yield Ok(Event::default()
                .event("data")
                .data(format!(r#"{"type":"think","content":"{}"}"#, step)));
        }

        // Conditionally simulate tool calls based on whether agent has skills
        if has_skills {
            let simulated_skills = ["file_reader", "internet_search", "calculator"];
            for skill_id in simulated_skills.iter().filter(|s| message.to_lowercase().contains(*s) || **s == "example") {
                // Generate a unique tool call ID
                let tool_call_id = format!("call_{}", &uuid::Uuid::new_v4().to_string()[..8]);
                
                yield Ok(Event::default()
                    .event("data")
                    .data(format!(
                        r#"{"type":"tool-input-start","toolCallId":"{}","toolName":"{}"}"#, 
                        tool_call_id, skill_id
                    )));
                
                tokio::time::sleep(Duration::from_millis(300)).await;
                
                let esc_message = message.replace("\"", "'");
                yield Ok(Event::default()
                    .event("data")
                    .data(format!(
                        r#"{"type":"tool-call","toolCallId":"{}","toolName":"{}","input":{"query":"{}","taskId":"analyze"}}"#, 
                        tool_call_id, skill_id, esc_message
                    )));
                
                tokio::time::sleep(Duration::from_millis(500)).await;
                
                yield Ok(Event::default()
                    .event("data")
                    .data(format!(
                        r#"{"type":"tool-result","toolCallId":"{}","toolName":"{}","output":{"status":"success","data":"Simulated results for {} on {}", "query":"{}", "metadata":{"execTime":123, "source":"simulated"}}"#, 
                        tool_call_id, skill_id, skill_id, esc_message, esc_message
                    )));
            }
        }

        // Generate response segments based on the input message
        let response_segments = vec![
            format!("I have processed your request about ("),
            message.clone(),
            format!("). "),
            format!("Based on skills available to agent {}, ", agent_id),
            format!("I have analyzed the information as requested. "),
            "Is there anything else I can assist with?".to_string()
        ];
        
        for segment in response_segments {
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield Ok(Event::default()
                .event("data")
                .data(format!(r#"{"type":"chunk","content":"{}"}"#, segment))); 
        }
    };
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}
    State(_state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let _agent_id = payload.agent_id;
    let message = payload.message;
    
    let stream = async_stream::stream! {
        yield Ok(Event::default().event("think").data(&format!("Thinking about your question: \"{}\"...", message)));
        
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        // Send thinking updates in fragments
        yield Ok(Event::default().event("think").data("Analyzing query structure..."));
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        yield Ok(Event::default().event("think").data(&format!("Preparing response based on available information for query: '{}'", message)));
        tokio::time::sleep(Duration::from_millis(400)).await;
        
        // Split the final response into streamable chunks
        let greeting = "Okay, I've processed your message.";
        yield Ok(Event::default().event("chunk").data(greeting));
        
        // Stream a complete response in parts
        let response_parts = [
            format!(" You asked about: \"{}\"", message),
            ". ".to_string(),
            "I can see you're looking for information.".to_string(),
            " Let me share my thoughts on this topic.".to_string()
        ];
        
        for part in response_parts {
            tokio::time::sleep(Duration::from_millis(150)).await;
            yield Ok(Event::default().event("chunk").data(part));
        }
        
        // Add tool simulation
        let tool_call_id = format!("call_{}", &uuid::Uuid::new_v4().to_string()[..8]);
        yield Ok(Event::default()
            .event("tool-call")
            .data(format!(
                r#"{{"toolCallId": "{}", "toolName": "simulated_tool", "input": {{"request": "{}"}}}}"#,
                tool_call_id, message
            )));
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        yield Ok(Event::default()
            .event("tool-result")
            .data(format!(
                r#"{{"toolCallId": "{}", "output": {{"response": "Simulated result for your query about '{}'."}}}}"#, 
                tool_call_id, message
            )));
        
        // Finalize the stream
        tokio::time::sleep(Duration::from_millis(200)).await;
        yield Ok(Event::default().event("done").data("[DONE]"));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn chat_ws_handler(
    _state: State<crate::AppState>,
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
pub use super::agents_api;

pub mod auth; // Authentication handlers

pub mod providers;      // Provider core logic
pub mod providers_api;  // Provider API handlers
pub mod skills;         // Skills functionality
pub mod skills_api;     // Skill API handlers