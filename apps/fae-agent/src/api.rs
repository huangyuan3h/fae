use axum::{
    extract::DefaultBodyLimit,
    response::Html,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

pub fn create_app(state: crate::AppState) -> Router {
    let cors_layer = CorsLayer::new()
        .allow_origin(get_allowed_origins())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
        .allow_headers(Any);

    Router::new()
        // Health check endpoint
        .route("/health", get(handle_health))
        // Authentication API routes
        .route("/api/auth/login", post(super::services::auth::handle_auth_login))  // Using POST for login
        .route("/api/auth/dev-login", post(super::services::auth::handle_dev_login))  // Development login
        // Basic API routes
        .route("/api/status", get(super::services::status_handler))
        // Agent API routes
        .route("/api/agents", get(super::agents_api::get_agents_handler))
        .route("/api/agents", post(super::agents_api::create_agent_handler))
        .route("/api/agents/:id", put(super::agents_api::update_agent_handler))
        .route("/api/agents/:id", delete(super::agents_api::delete_agent_handler))
        // API routes for chat services
        .route("/api/chat", post(super::services::agent_chat_handler))
        .route("/api/chat/stream", post(super::services::agent_stream_chat_handler))
        // Provider API routes
        .route("/api/settings/providers", get(super::services::providers_api::get_providers_handler))
        .route("/api/settings/providers", put(super::services::providers_api::update_providers_handler))
        .route("/api/settings/ollama", get(super::services::providers_api::get_ollama_settings_handler))
        .route("/api/settings/ollama", put(super::services::providers_api::update_ollama_settings_handler))
        .route("/api/settings/folders", get(super::services::folders_api::get_folders_handler))
        .route("/api/settings/folders", put(super::services::folders_api::update_folders_handler))
        // Skill API routes 
        .route("/api/skills", get(super::services::skills_api::get_skills_handler))
        .route("/api/skills/:id", put(super::services::skills_api::update_skill_handler))
        .route("/api/skills/refresh", post(super::services::skills_api::refresh_skills_handler))
        // WebSocket route for future AI streaming
        .route("/api/ws/chat", get(super::services::chat_ws_handler))
        // Add state
        .with_state(state)
        // Add limit to prevent large payloads
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        // Add CORS layer (added last to be outermost)
        .layer(cors_layer)
}

fn get_allowed_origins() -> tower_http::cors::AllowOrigin {
    let origins = [
        "http://localhost:5173",
        "http://localhost:3000",
        "http://localhost:3001",
        "http://localhost:8080",
        "http://localhost:8081",
    ];
    tower_http::cors::AllowOrigin::list(origins.iter().map(|s| s.parse().unwrap()))
}

async fn handle_health() -> Html<&'static str> {
    Html("<h1>OK</h1>")
}