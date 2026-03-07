use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::info;
use sqlx::Row;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};

// GET /api/agents - 获取所有agents
pub async fn get_agents_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    info!("Received request to get all agents");
    
    // 从数据库获取 - 我们直接使用Sqlx::query获取原始数据然后手动构建对象
    let rows = sqlx::query(
        r#"SELECT 
            id, name, provider, provider_config_id, model, system_prompt, avatar_url,
            skills_json, created_at, updated_at
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
        let id: String = row.get("id");
        let name: String = row.get("name");
        let provider: String = row.get("provider");
        let provider_config_id: Option<String> = row.get("provider_config_id");
        let model: String = row.get("model");
        let system_prompt: Option<String> = row.get("system_prompt");
        let avatar_url: Option<String> = row.get("avatar_url");
        let skills_json: String = row.get("skills_json");
        let created_at_i64: i64 = row.get("created_at");
        let updated_at_i64: i64 = row.get("updated_at");

        let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_else(|_| vec![]);
        let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at_i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at_i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        let agent = Agent {
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
        };

        agents.push(agent);
    }
    
    Ok(Json(agents))
}

// POST /api/agents - 创建新的agent
pub async fn create_agent_handler(
    State(state): State<crate::AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
    info!("Received request to create agent: {}", payload.name);
    
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp(); // 使用时间戳
    let skills_json = serde_json::to_string(&payload.skills).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
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
            // 返回创建的agent
            let new_agent = Agent {
                id,
                name: payload.name,
                provider: payload.provider,
                provider_config_id: payload.provider_config_id,
                model: payload.model,
                system_prompt: payload.system_prompt,
                avatar_url: payload.avatar_url,
                skills: payload.skills,
                created_at: chrono::DateTime::<chrono::Utc>::from_timestamp(now, 0).unwrap(),
                updated_at: chrono::DateTime::<chrono::Utc>::from_timestamp(now, 0).unwrap(),
            };
            
            Ok(Json(new_agent))
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
) -> Result<Json<Agent>, StatusCode> {
    info!("Received request to update agent: {}", id);
    
    let now = chrono::Utc::now().timestamp();
    let skills_json = serde_json::to_string(&payload.skills).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 更新数据库中的记录
    let result = sqlx::query!(
        r#"
        UPDATE agents 
        SET name = ?, provider = ?, provider_config_id = ?, model = ?, 
            system_prompt = ?, avatar_url = ?, skills_json = ?, updated_at = ?
        WHERE id = ?
        "#,
        payload.name,
        payload.provider,
        payload.provider_config_id,
        payload.model,
        payload.system_prompt,
        payload.avatar_url,
        skills_json,
        now,
        &id
    )
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

            let id_val: String = row.get("id");
            let name: String = row.get("name");
            let provider: String = row.get("provider");
            let provider_config_id: Option<String> = row.get("provider_config_id");
            let model: String = row.get("model");
            let system_prompt: Option<String> = row.get("system_prompt");
            let avatar_url: Option<String> = row.get("avatar_url");
            let skills_json_val: String = row.get("skills_json");
            let created_at_i64: i64 = row.get("created_at");
            let updated_at_i64: i64 = row.get("updated_at");

            let skills: Vec<String> = serde_json::from_str(&skills_json_val).unwrap_or_else(|_| vec![]);
            let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at_i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at_i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());

            let updated_agent = Agent {
                id: id_val,
                name,
                provider,
                provider_config_id,
                model,
                system_prompt,
                avatar_url,
                skills,
                created_at,
                updated_at,
            };

            Ok(Json(updated_agent))
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