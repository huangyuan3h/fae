#![forbid(unsafe_code)]
//! fae-agent
//! Rust-based backend service for the FAE platform.
//!
//! Endpoints:
//! - GET /health - Health check
//! - GET /api/status - Service status
//! - POST /api/chat/stream - Chat stream endpoint
//! - GET /api/ws/chat - WebSocket chat endpoint (future AI streaming)

use anyhow::Result;
use config::Settings;
use tracing_subscriber::{EnvFilter, fmt};
use sqlx::sqlite::SqliteConnectOptions;
use std::sync::Arc;
use sqlx::SqlitePool;

pub mod api;
pub mod config;
pub mod models;
pub mod services;
pub mod agents_api;

// Define AppState for sharing resources across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<SqlitePool>,
    pub llm_log_dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let settings = Settings::new().expect("Failed to load settings");
    
    tracing::info!("Starting fae-agent server on {}", settings.server_addr());
    
    // Initialize database pool
    let conn_opts = SqliteConnectOptions::new()
        .filename(&settings.database_url.replace("sqlite:", "").replace("file:", ""))
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);  // WAL mode
        
    let db_pool = sqlx::SqlitePool::connect_with(conn_opts).await
        .expect("Failed to connect to database");
        
    // Run migrations    
    sqlx::migrate!("./migrations").run(&db_pool).await
        .expect("Failed to run migrations");

tracing::info!("Database initialized successfully");
    
    if services::folders_api::get_base_folder(&db_pool).await.is_none() {
        match services::folders_api::create_default_base_folder(&db_pool).await {
            Ok(folder) => {
                tracing::info!("Created default workspace folder: {}", folder.path);
            }
            Err(e) => {
                tracing::error!("Failed to create default workspace folder: {}", e);
            }
        }
    }

    // Load skills automatically from the skills directory
    // First try relative to current working directory, then relative to parent if not found
    let mut skills_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("skills")
        .to_string_lossy()
        .to_string();
        
    if !std::path::Path::new(&skills_dir).exists() {
        // Look in project root as fallback (common when running from apps/fae-agent/)
        // Go up two levels from apps/fae-agent to reach project root
        let proj_root_skills_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("skills"))
            .unwrap_or_else(|| std::path::PathBuf::from("../../skills"))
            .to_string_lossy()
            .to_string();
            
        if std::path::Path::new(&proj_root_skills_dir).exists() {
            skills_dir = proj_root_skills_dir;
            tracing::info!("Using skills directory from project root: {}", skills_dir);
        }
    }
    
    match services::skills::load_skills_from_directory(&skills_dir, &db_pool).await {
        Ok(loaded_skills) => {
            tracing::info!("Successfully loaded {} skills from {}", loaded_skills.len(), skills_dir);
        }
        Err(e) => {
            tracing::error!("Failed to load skills: {}", e);
        }
    }

    // Create shared state
    let app_state = AppState {
        db_pool: Arc::new(db_pool),
        llm_log_dir: settings.llm_log_dir.clone(),
    };
    
    let app = api::create_app(app_state);
    
    let listener = tokio::net::TcpListener::bind(settings.server_addr()).await?;
    
    tracing::info!("Server starting on {}", settings.server_addr());
    
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
        
    Ok(())
}