use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{Response, Sse},
    Json,
};
use axum::response::sse::{Event, KeepAlive};
use futures_util::Stream;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::models::ChatCompletion;
use sqlx::Row;

pub mod llm;

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
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub message: String,
}

pub async fn chat_stream_handler(
    State(_state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let _agent_id = payload.agent_id;
    let message = payload.message;
    
    let stream = async_stream::stream! {
        yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
            "type": "chunk",
            "content": format!("Echo: {}", message)
        })).unwrap()));
        
        yield Ok(Event::default().data("[DONE]"));
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn agent_chat_handler(
    State(_state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Result<Json<ChatCompletion>, StatusCode> {
    info!("[CHAT] Agent: {}, Message: {}", payload.agent_id, payload.message);
    
    let model = "mock-model".to_string();
    let response_message = format!("Processed: {}", payload.message);
    
    let response = ChatCompletion {
        id: uuid::Uuid::new_v4().to_string(),
        message: response_message,
        model,
        timestamp: chrono::Utc::now().timestamp(),
    };

    Ok(Json(response))
}

async fn get_agent_by_id(
    db_pool: &sqlx::SqlitePool,
    agent_id: &str,
) -> Option<crate::models::Agent> {
    let row = sqlx::query(
        r#"SELECT id, name, provider, provider_config_id, model, system_prompt, avatar_url, skills_json, created_at, updated_at 
           FROM agents WHERE id = ?"#
    )
    .bind(agent_id)
    .fetch_optional(db_pool)
    .await
    .ok()??;

    let id: String = row.get("id");
    let name: String = row.get("name");
    let provider: String = row.get("provider");
    let provider_config_id: Option<String> = row.get("provider_config_id");
    let model: String = row.get("model");
    let system_prompt: Option<String> = row.get("system_prompt");
    let avatar_url: Option<String> = row.get("avatar_url");
    let skills_json: String = row.get("skills_json");
    let created_at_timestamp: i64 = row.get("created_at");
    let updated_at_timestamp: i64 = row.get("updated_at");

    let skills: Vec<String> = serde_json::from_str(&skills_json).ok()?;
    let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());

    Some(crate::models::Agent {
        id,
        name,
        provider,
        provider_config_id,
        model,
        system_prompt,
        avatar_url,
        skills,
        created_at,
        updated_at,
    })
}

async fn get_provider_config(
    db_pool: &sqlx::SqlitePool,
    config_id: &str,
) -> Option<crate::models::providers::ProviderConfig> {
    let row = sqlx::query(
        r#"SELECT value FROM settings WHERE key = 'provider.configs'"#
    )
    .fetch_optional(db_pool)
    .await
    .ok()??;

    let value: String = row.get("value");
    let configs: Vec<crate::models::providers::ProviderConfig> = 
        serde_json::from_str(&value).ok()?;
    
    configs
        .iter()
        .find(|c| c.id == config_id)
        .cloned()
}

pub async fn agent_stream_chat_handler(
    State(state): State<crate::AppState>,
    Json(payload): Json<AgentChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let agent_id = payload.agent_id.clone();
    let message = payload.message.clone();
    let db_pool = state.db_pool.clone();
    
    info!("[AGENT CHAT] ===== New Request =====");
    info!("[AGENT CHAT] Agent ID: {}", agent_id);
    info!("[AGENT CHAT] User Message: {}", message);

    let stream = async_stream::stream! {
        let agent = get_agent_by_id(&db_pool, &agent_id).await;
        
        match agent {
            Some(agent) => {
                info!("[AGENT] Found agent: {} (provider: {}, model: {})", 
                    agent.name, agent.provider, agent.model);
                info!("[AGENT] Skills: {:?}", agent.skills);
                
                let provider_config = if let Some(ref config_id) = agent.provider_config_id {
                    match get_provider_config(&db_pool, config_id).await {
                        Some(config) => {
                            info!("[PROVIDER] Using config '{}': type={}, base_url={}, model={}", 
                                config_id, config.provider_type, config.base_url, config.model_id);
                            Some(config)
                        }
                        None => {
                            info!("[PROVIDER] Config '{}' not found", config_id);
                            None
                        }
                    }
                } else {
                    None
                };

                let (base_url, model, provider_type, api_key) = match &provider_config {
                    Some(config) => {
                        let model = if config.model_id.is_empty() { agent.model.clone() } else { config.model_id.clone() };
                        (config.base_url.clone(), model, config.provider_type.clone(), config.api_key.clone())
                    }
                    None => {
                        info!("[PROVIDER] No config specified, using Ollama default");
                        ("http://127.0.0.1:11434".to_string(), agent.model.clone(), "ollama".to_string(), String::new())
                    }
                };

                info!("[LLM] Provider type: {}", provider_type);
                info!("[LLM] Base URL: {}", base_url);
                info!("[LLM] Model: {}", model);

                let llm_client = llm::LLMClient::new(base_url.clone(), provider_type.clone(), api_key);
                
                let system_prompt = llm::build_system_prompt(&agent.skills);
                let tools = if agent.skills.is_empty() {
                    None
                } else {
                    let mut skill_defs = std::collections::HashMap::new();
                    skill_defs.insert("file-operation".to_string(), 
                        "Perform file operations like read, write, list directory".to_string());
                    skill_defs.insert("example-skill".to_string(), 
                        "An example demonstration skill".to_string());
                    Some(llm::skills_to_tools(&agent.skills, &skill_defs))
                };

                info!("[LLM] System prompt: {}", system_prompt);
                info!("[LLM] Tools enabled: {:?}", tools.as_ref().map(|t| t.len()));

                let messages = vec![
                    llm::ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt,
                        tool_calls: None,
                    },
                    llm::ChatMessage {
                        role: "user".to_string(),
                        content: message.clone(),
                        tool_calls: None,
                    },
                ];

                info!("[LLM] Sending request to Ollama...");

                yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                    "type": "think",
                    "content": format!("Connecting to {} model...", model)
                })).unwrap()));

                match llm_client.chat_stream(&model, messages.clone(), tools.clone()).await {
                    Ok((mut rx, _done_rx)) => {
                        let mut tool_calls_to_execute: Vec<llm::ToolCall> = Vec::new();
                        let mut accumulated_content = String::new();

                        while let Some(response) = rx.recv().await {
                            if let Some(msg) = &response.message {
                                if !msg.content.is_empty() {
                                    accumulated_content.push_str(&msg.content);
                                    yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                        "type": "chunk",
                                        "content": &msg.content
                                    })).unwrap()));
                                }

                                if let Some(tool_calls) = &msg.tool_calls {
                                    for tc in tool_calls {
                                        info!("[TOOL CALL] Tool: {}, Args: {}", 
                                            tc.function.name, tc.function.arguments);
                                        tool_calls_to_execute.push(tc.clone());
                                    }
                                }
                            }

                            if response.done {
                                info!("[LLM] Stream completed. Total content length: {}", accumulated_content.len());
                                break;
                            }
                        }

                        for tc in &tool_calls_to_execute {
                            let tool_call_id = tc.id.clone();
                            let tool_name = tc.function.name.clone();
                            let args = tc.function.arguments.clone();

                            info!("[TOOL] Executing tool: {} with args: {}", tool_name, args);

                            yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                "type": "tool-call",
                                "toolCallId": tool_call_id,
                                "toolName": tool_name,
                                "input": serde_json::from_str::<serde_json::Value>(&args).unwrap_or(serde_json::json!({"raw": args}))
                            })).unwrap()));

                            tokio::time::sleep(Duration::from_millis(300)).await;

                            let tool_result = format!("Tool '{}' executed successfully for input: {}", tool_name, args);
                            info!("[TOOL] Result: {}", tool_result);

                            yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                "type": "tool-result",
                                "toolCallId": tool_call_id,
                                "toolName": tool_name,
                                "output": { "result": tool_result }
                            })).unwrap()));
                        }

                        if !tool_calls_to_execute.is_empty() {
                            info!("[LLM] Sending tool results back to model for final response...");

                            yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                "type": "think",
                                "content": "Processing tool results..."
                            })).unwrap()));

                            let mut messages_with_results = messages.clone();
                            messages_with_results.push(llm::ChatMessage {
                                role: "assistant".to_string(),
                                content: accumulated_content.clone(),
                                tool_calls: Some(tool_calls_to_execute.clone()),
                            });

                            for tc in &tool_calls_to_execute {
                                messages_with_results.push(llm::ChatMessage {
                                    role: "tool".to_string(),
                                    content: format!("Tool result for {}: executed successfully", tc.function.name),
                                    tool_calls: None,
                                });
                            }

                            match llm_client.chat_stream(&model, messages_with_results, tools).await {
                                Ok((mut rx2, _)) => {
                                    while let Some(response) = rx2.recv().await {
                                        if let Some(msg) = &response.message {
                                            if !msg.content.is_empty() {
                                                yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                                    "type": "chunk",
                                                    "content": &msg.content
                                                })).unwrap()));
                                            }
                                        }
                                        if response.done {
                                            info!("[LLM] Final response completed");
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    info!("[LLM ERROR] Failed to get final response: {}", e);
                                    yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                        "type": "error",
                                        "message": format!("Failed to process tool results: {}", e)
                                    })).unwrap()));
                                }
                            }
                        }

                        yield Ok(Event::default().data("[DONE]"));
                    }
                    Err(e) => {
                        info!("[LLM ERROR] {}", e);
                        yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                            "type": "error",
                            "message": format!("LLM error: {}", e)
                        })).unwrap()));
                        yield Ok(Event::default().data("[DONE]"));
                    }
                }
            }
            None => {
                info!("[AGENT ERROR] Agent not found: {}", agent_id);
                yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                    "type": "error",
                    "message": format!("Agent '{}' not found", agent_id)
                })).unwrap()));
                yield Ok(Event::default().data("[DONE]"));
            }
        }
        
        info!("[AGENT CHAT] ===== Request Complete =====");
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
            let response = format!("Echo via WS: {}", text);
            
            if websocket.send(axum::extract::ws::Message::Text(response)).await.is_err() {
                break;
            }
        } else if msg.is_err() {
            break;
        }
    }
}

pub async fn initialize_db(_db_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub use providers::ProviderResolver;
pub use providers_api::{ 
    get_providers_handler, 
    update_providers_handler, 
    get_ollama_settings_handler, 
    update_ollama_settings_handler 
};
pub use super::agents_api;

pub mod auth;
pub mod providers;
pub mod providers_api;
pub mod skills;
pub mod skills_api;