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
    let llm_log_dir = state.llm_log_dir.clone();
    let skills_dir = state.skills_dir.clone();
    let session_id = llm::generate_session_id();
    
    info!("[AGENT CHAT] ===== New Request =====");
    info!("[AGENT CHAT] Agent ID: {}", agent_id);
    info!("[AGENT CHAT] User Message: {}", message);
    info!("[AGENT CHAT] Session ID: {}", session_id);

    let stream = async_stream::stream! {
        let agent = get_agent_by_id(&db_pool, &agent_id).await;
        
        match agent {
            Some(agent) => {
                info!("[AGENT] Found agent: {} (provider: {}, model: {})", 
                    agent.name, agent.provider, agent.model);
                info!("[AGENT] Skills: {:?}", agent.skills);
                
                let logger = match llm::LLMLogger::new(&llm_log_dir, &session_id, &agent_id, &agent.name) {
                    Ok(l) => Some(l),
                    Err(e) => {
                        info!("[LLMLogger] Failed to create logger: {}", e);
                        None
                    }
                };
                
                if let Some(ref logger) = logger {
                    logger.log_session_start();
                }
                
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
                
                let system_settings = match system_settings_api::get_system_settings(&db_pool).await {
                    Ok(s) => s,
                    Err(_) => crate::models::system_settings::SystemSettings::default(),
                };
                
                let system_context = llm::SystemContext::from(system_settings);
                
                let mut system_prompt = llm::build_system_prompt(&agent.skills);
                
                system_prompt = format!("{}\n\n{}", system_context.to_prompt_section(), system_prompt);
                
                let mut all_tools = Vec::new();
                
                let mut tool_executor = llm::ToolExecutor::with_folder_validation(&db_pool).await;
                
                let agent_skills = agent.skills.clone();
                let skill_defs: std::collections::HashMap<String, String> = if !agent.skills.is_empty() {
                    let all_db_skills = match skills::get_all_skills(&db_pool).await {
                        Ok(s) => s,
                        Err(e) => {
                            info!("[SKILLS] Failed to load skills from database: {}", e);
                            Vec::new()
                        }
                    };
                    
                    let defs: std::collections::HashMap<String, String> = all_db_skills
                        .into_iter()
                        .map(|s| (s.id, s.name))
                        .collect();
                    
                    for skill_id in &agent.skills {
                        if let Some(desc) = defs.get(skill_id) {
                            let skill_path = std::path::PathBuf::from(&skills_dir)
                                .join(skill_id);
                            tool_executor.register_skill(skill_id.clone(), desc.clone(), skill_path.to_string_lossy().to_string());
                        }
                    }
                    
                    all_tools.extend(llm::skills_to_tools(&agent.skills, &defs));
                    defs
                } else {
                    std::collections::HashMap::new()
                };
                
                all_tools.extend(tool_executor.get_tool_definitions());
                
                let tools = if all_tools.is_empty() {
                    None
                } else {
                    Some(all_tools)
                };

                info!("[LLM] System prompt: {}", system_prompt);
                info!("[LLM] Tools enabled: {:?}", tools.as_ref().map(|t| t.len()));
                
                if let Some(ref logger) = logger {
                    logger.log_system_prompt(&system_prompt);
                    logger.log_user_message(&message);
                    logger.log_llm_request(&provider_type, &model, &base_url, 2, tools.as_ref().map(|t| t.len()));
                }

                let messages = vec![
                    llm::ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt.clone(),
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
                                        if let Some(ref logger) = logger {
                                            logger.log_tool_call(&tc.id, &tc.function.name, &tc.function.arguments);
                                        }
                                        tool_calls_to_execute.push(tc.clone());
                                    }
                                }
                            }

                            if response.done {
                                info!("[LLM] Stream completed. Total content length: {}", accumulated_content.len());
                                if let Some(ref logger) = logger {
                                    logger.log_assistant_message(&accumulated_content);
                                }
                                break;
                            }
                        }

                        let mut tool_results: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                        
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

                            let mut tool_executor = llm::ToolExecutor::with_folder_validation(&db_pool).await;
                            
                            // Register skills for this tool executor
                            for skill_id in &agent_skills {
                                if let Some(desc) = skill_defs.get(skill_id) {
                                    let skill_path = std::path::PathBuf::from(&skills_dir)
                                        .join(skill_id);
                                    tool_executor.register_skill(skill_id.clone(), desc.clone(), skill_path.to_string_lossy().to_string());
                                }
                            }
                            
                            let tool_result = match tool_executor.execute_tool_call(tc).await {
                                llm::ToolResult { success, output, error } => {
                                    if success {
                                        output
                                    } else {
                                        format!("Error: {}", error.unwrap_or_else(|| "Unknown error".to_string()))
                                    }
                                }
                            };
                            
                            tool_results.insert(tool_call_id.clone(), tool_result.clone());
                            
                            info!("[TOOL] Result: {}", tool_result);
                            
                            if let Some(ref logger) = logger {
                                logger.log_tool_result(&tool_call_id, &tool_name, &tool_result);
                            }

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
                            
                            if let Some(ref logger) = logger {
                                logger.log_thinking("Processing tool results...");
                            }

                            let mut messages_with_results = messages.clone();
                            messages_with_results.push(llm::ChatMessage {
                                role: "assistant".to_string(),
                                content: accumulated_content.clone(),
                                tool_calls: Some(tool_calls_to_execute.clone()),
                            });

                            for tc in &tool_calls_to_execute {
                                let result = tool_results.get(&tc.id).cloned().unwrap_or_else(|| "No result".to_string());
                                messages_with_results.push(llm::ChatMessage {
                                    role: "tool".to_string(),
                                    content: result,
                                    tool_calls: None,
                                });
                            }

                            match llm_client.chat_stream(&model, messages_with_results, tools).await {
                                Ok((mut rx2, _)) => {
                                    let mut final_content = String::new();
                                    while let Some(response) = rx2.recv().await {
                                        if let Some(msg) = &response.message {
                                            if !msg.content.is_empty() {
                                                final_content.push_str(&msg.content);
                                                yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                                    "type": "chunk",
                                                    "content": &msg.content
                                                })).unwrap()));
                                            }
                                        }
                                        if response.done {
                                            info!("[LLM] Final response completed");
                                            if let Some(ref logger) = logger {
                                                logger.log_assistant_message(&final_content);
                                            }
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    info!("[LLM ERROR] Failed to get final response: {}", e);
                                    if let Some(ref logger) = logger {
                                        logger.log_error(&format!("Failed to process tool results: {}", e));
                                    }
                                    yield Ok(Event::default().data(serde_json::to_string(&serde_json::json!({
                                        "type": "error",
                                        "message": format!("Failed to process tool results: {}", e)
                                    })).unwrap()));
                                }
                            }
                        }

                        yield Ok(Event::default().data("[DONE]"));
                        if let Some(ref logger) = logger {
                            logger.log_session_end();
                        }
                    }
                    Err(e) => {
                        info!("[LLM ERROR] {}", e);
                        if let Some(ref logger) = logger {
                            logger.log_error(&format!("LLM error: {}", e));
                        }
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
pub mod folders_api;
pub mod system_settings_api;