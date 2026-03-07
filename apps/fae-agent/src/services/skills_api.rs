use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::skills::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSkillPayload {
    pub enabled: bool,
}

pub async fn get_skills_handler(
    State(state): State<crate::AppState>,
) -> Result<Json<SkillApiResponse<Vec<Skill>>>, StatusCode> {
    match get_all_skills(&state.db_pool).await {
        Ok(skills) => Ok(Json(SkillApiResponse {
            ok: true,
            data: Some(skills),
            error: None,
        })),
        Err(e) => {
            eprintln!("Failed to get skills: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_skill_handler(
    State(state): State<crate::AppState>,
    Path(skill_id): Path<String>,
    Json(payload): Json<UpdateSkillPayload>,
) -> Result<Json<SkillApiResponse<Skill>>, StatusCode> {
    info!("Updating skill: {} to enabled: {}", skill_id, payload.enabled);
    
    match update_skill(&state.db_pool, &skill_id, payload.enabled).await {
        Ok(Some(updated_skill)) => Ok(Json(SkillApiResponse {
            ok: true,
            data: Some(updated_skill),
            error: None,
        })),
        Ok(None) => {
            // Skill not found
            Ok(Json(SkillApiResponse {
                ok: false,
                data: None,
                error: Some(ApiError {
                    code: "NOT_FOUND".to_string(),
                    message: "Skill not found".to_string(),
                }),
            }))
        },
        Err(e) => {
            eprintln!("Failed to update skill: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}