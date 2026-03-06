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

pub mod api;
pub mod config;
pub mod models;
pub mod services;

// Define AppState for sharing resources across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<SqlitePool>,
}

use sqlx::SqlitePool;

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
    
    // Create shared state
    let app_state = AppState {
        db_pool: Arc::new(db_pool),
    };
    
    let app = api::create_app(app_state);
    
    let listener = tokio::net::TcpListener::bind(settings.server_addr()).await?;
    
    tracing::info!("Server starting on {}", settings.server_addr());
    
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
        
    Ok(())
}