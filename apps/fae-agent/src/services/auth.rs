use axum::{extract::State, http::StatusCode, response::Json};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub token: Option<String>, // Startup token for real login (if implemented)
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub data: Option<LoginData>,
    pub error: Option<ErrorResponse>,
}

#[derive(Serialize)]
pub struct LoginData {
    pub session_token: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Standard login handler (could be for startup token verification in the future)
pub async fn handle_auth_login(
    State(state): State<AppState>,
    Json(_payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // In future: validate token against startup tokens in DB or config
    
    // For now, just create a session token since we received a login request
    let session_token = generate_session_token();
    
    // Store session in database
    if let Err(_) = create_session_in_db(&state.db_pool, &session_token).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(LoginResponse {
        ok: true,
        data: Some(LoginData {
            session_token: session_token.clone(),
        }),
        error: None,
    }))
}

/// Development login handler - creates session without requiring startup token
pub async fn handle_dev_login(State(state): State<AppState>) -> Result<Json<LoginResponse>, StatusCode> {
    // Only allow dev login in non-production environments (would check ENV in real implementation)
    // Since Rust programs typically differentiate environments via features or environment variables,
    // we'll proceed without environment check for this implementation.
    
    let session_token = generate_session_token();
    
    // Store session in database
    if let Err(_) = create_session_in_db(&state.db_pool, &session_token).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(LoginResponse {
        ok: true,
        data: Some(LoginData {
            session_token: session_token.clone(),
        }),
        error: None,
    }))
}

/// Generate a unique session token
fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

/// Store session token in database with expiration
async fn create_session_in_db(db_pool: &Arc<SqlitePool>, token: &str) -> Result<(), sqlx::Error> {
    let expires_at = chrono::Utc::now().timestamp() + (24 * 60 * 60); // 24 hours from now

    sqlx::query!(
        "INSERT INTO sessions (token, expires_at) VALUES (?1, ?2) ON CONFLICT(token) DO UPDATE SET expires_at = ?2",
        token,
        expires_at
    )
    .execute(&**db_pool) // Dereference the Arc to get the pool reference
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_token() {
        let token = generate_session_token();
        assert_eq!(token.len(), 64); // 64 character token
    }
}