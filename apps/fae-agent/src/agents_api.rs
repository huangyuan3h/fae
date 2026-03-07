use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use sqlx::Row;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};

/// Response wrapper expected by frontend (ok + data)
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
}

// Helper function to convert DB row to Agent with proper JSON deserialization
async fn db_row_to_agent(row: &sqlx::sqlite::SqliteRow) -> Result<Agent, Box<dyn std::error::Error>> {
    let id: String = row.get("id");
    let name: String = row.get("name");
    let provider: String = row.get("provider");
    let provider_config_id: Option<String> = row.get("provider_config_id");
    let model: String = row.get("model");
    let system_prompt: Option<String> = row.get("system_prompt");
    let avatar_url: Option<String> = row.get("avatar_url");
    let skills_json: String = row.get("skills_json");  // DB column is actually skills_json
    let created_at_timestamp: i64 = row.get("created_at");
    let updated_at_timestamp: i64 = row.get("updated_at");

    let skills: Vec<String> = serde_json::from_str(&skills_json)?;
    let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());

    Ok(Agent {
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

// Helper function to serialize skills for database storage
fn serialize_skills_for_db(skills: &Vec<String>) -> String {
    serde_json::to_string(skills).unwrap_or_else(|_| "[]".to_string())
}

// GET /api/agents - 获取所有agents
pub async fn get_agents_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<AgentApiResponse<Vec<Agent>>>, StatusCode> {
    info!("Received request to get all agents");

    let rows = sqlx::query(
        r#"SELECT id, name, provider, provider_config_id, model, system_prompt, avatar_url, skills_json, created_at, updated_at 
           FROM agents ORDER BY created_at DESC"#
    )
    .fetch_all(&*state.db_pool)
    .await
    .map_err(|e| {
        eprintln!("Database error when fetching agents: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut agents = Vec::new();
    for row in rows {
        match db_row_to_agent(&row).await {
            Ok(agent) => agents.push(agent),
            Err(e) => {
                eprintln!("Error converting DB row to agent: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Ok(Json(AgentApiResponse {
        ok: true,
        data: Some(agents),
    }))
}

// POST /api/agents - 创建新的agent
pub async fn create_agent_handler(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<Json<AgentApiResponse<Agent>>, StatusCode> {
    info!("Received request to create agent: {}", payload.name);
    
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp(); // 使用时间戳
    let skills_json = serialize_skills_for_db(&payload.skills);
    
    // 保存到数据库
    let result = sqlx::query(
        r#"
        INSERT INTO agents (id, name, provider, provider_config_id, model, system_prompt, avatar_url, skills_json, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.provider)
    .bind(&payload.provider_config_id)
    .bind(&payload.model)
    .bind(&payload.system_prompt)
    .bind(&payload.avatar_url)
    .bind(&skills_json)
    .bind(now)
    .bind(now)
    .execute(&*state.db_pool)
    .await;
    
    match result {
        Ok(_) => {
            // 查询数据库以返回最新创建的agent，保证与数据库中的实际值一致
            let row = sqlx::query(
                r#"SELECT 
                    id, name, provider, provider_config_id, model, system_prompt, avatar_url,
                    skills_json, created_at, updated_at
                FROM agents WHERE id = ?"#
            )
            .bind(&id)
            .fetch_one(&*state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            match db_row_to_agent(&row).await {
                Ok(agent) => Ok(Json(AgentApiResponse {
                    ok: true,
                    data: Some(agent),
                })),
                Err(e) => {
                    eprintln!("Error converting DB row to agent: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error when creating agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// PUT /api/agents/:id - 更新现有agent
pub async fn update_agent_handler(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAgentRequest>,
) -> Result<Json<AgentApiResponse<Agent>>, StatusCode> {
    info!("Received request to update agent: {}", id);
    
    let now = chrono::Utc::now().timestamp();
    let skills_json = serialize_skills_for_db(&payload.skills);
    
    // 更新数据库中的记录
    let result = sqlx::query(
        r#"
        UPDATE agents 
        SET name = ?, provider = ?, provider_config_id = ?, model = ?, 
            system_prompt = ?, avatar_url = ?, skills_json = ?, updated_at = ?
        WHERE id = ?
        "#,
        )
    .bind(&payload.name)
    .bind(&payload.provider)
    .bind(&payload.provider_config_id)
    .bind(&payload.model)
    .bind(&payload.system_prompt)
    .bind(&payload.avatar_url)
    .bind(&skills_json)
    .bind(now)
    .bind(&id)
    .execute(&*state.db_pool)
    .await;
    
    match result {
        Ok(update_result) => {
            if update_result.rows_affected() == 0 {
                return Err(StatusCode::NOT_FOUND);
            }
            
            // 获取更新后的数据
            let row = sqlx::query(
                r#"SELECT 
                    id, name, provider, provider_config_id, model, system_prompt, avatar_url,
                    skills_json, created_at, updated_at
                FROM agents WHERE id = ?"#
            )
            .bind(&id)
            .fetch_one(&*state.db_pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            match db_row_to_agent(&row).await {
                Ok(agent) => Ok(Json(AgentApiResponse {
                    ok: true,
                    data: Some(agent),
                })),
                Err(e) => {
                    eprintln!("Error converting DB row to agent: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Database error when updating agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// DELETE /api/agents/:id - 删除agent
pub async fn delete_agent_handler(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Received request to delete agent: {}", id);
    
    // 首先检查记录是否存在
    let exists_result = sqlx::query("SELECT COUNT(*) as count FROM agents WHERE id = ?")
        .bind(&id)
        .fetch_one(&*state.db_pool)
        .await;
        
    let exists = match exists_result {
        Ok(row) => {
            let count: i64 = row.get("count");
            count > 0
        },
        Err(_) => false
    };
    
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // 从数据库删除记录
    let result = sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(&id)
        .execute(&*state.db_pool)
        .await;
        
    match result {
        Ok(_) => Ok(Json(json!({"id": id, "deleted": true}))),
        Err(e) => {
            eprintln!("Database error when deleting agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}