use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::info;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};

// GET /api/agents - 获取所有agents
pub async fn get_agents_handler(
    State(_state): State<crate::AppState>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    info!("Received request to get all agents");
    
    // 模拟数据 - 在实际实现中会是从数据库获取  
    let mock_agents = vec![
        Agent {
            id: "1".to_string(),
            name: "Market Research Agent".to_string(),
            provider: "ollama".to_string(),
            provider_config_id: Some("ollama-default".to_string()),
            model: "qwen2.5:14b".to_string(),
            system_prompt: Some("You are a helpful market research analyst.".to_string()),
            avatar_url: Some("emoji:📊".to_string()),
            skills: vec!["web_search".to_string(), "data_analysis".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Agent {
            id: "2".to_string(),
            name: "HR Virtual Assistant".to_string(),
            provider: "openai".to_string(),
            provider_config_id: Some("openai-default".to_string()),
            model: "gpt-4o".to_string(),
            system_prompt: Some("You are a helpful HR assistant.".to_string()),
            avatar_url: Some("emoji:🤖".to_string()),
            skills: vec!["document_processing".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    
    Ok(Json(mock_agents))
}

// POST /api/agents - 创建新的agent
pub async fn create_agent_handler(
    State(_state): State<crate::AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
    info!("Received request to create agent: {}", payload.name);
    
    // 在实际实现中，这会保存到数据库
    let new_agent = Agent {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        provider: payload.provider,
        provider_config_id: payload.provider_config_id,
        model: payload.model,
        system_prompt: payload.system_prompt,
        avatar_url: payload.avatar_url,
        skills: payload.skills,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    Ok(Json(new_agent))
}

// PUT /api/agents/:id - 更新现有agent
pub async fn update_agent_handler(
    State(_state): State<crate::AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
    info!("Received request to update agent: {}", id);
    
    // 在实际实现中，这会更新数据库中的记录
    let updated_agent = Agent {
        id,
        name: payload.name,
        provider: payload.provider,
        provider_config_id: payload.provider_config_id,
        model: payload.model,
        system_prompt: payload.system_prompt,
        avatar_url: payload.avatar_url,
        skills: payload.skills,
        created_at: chrono::Utc::now() - chrono::Duration::seconds(1), // 使用过去的时间作为创建时间
        updated_at: chrono::Utc::now(),
    };
    
    Ok(Json(updated_agent))
}

// DELETE /api/agents/:id - 删除agent
pub async fn delete_agent_handler(
    State(_state): State<crate::AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Received request to delete agent: {}", id);
    
    // 在实际实现中，这会从数据库删除记录
    Ok(Json(json!({"id": id, "deleted": true})))
}