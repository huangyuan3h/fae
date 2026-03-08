use anyhow::Result;
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub cors_origins: Vec<String>,
    pub llm_log_dir: String,
}

impl Settings {
    pub fn new() -> Result<Self> {
        dotenv().ok(); // Load .env file if present

        Ok(Settings {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".to_string()),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            llm_log_dir: env::var("LLM_LOG_DIR").unwrap_or_else(|_| "./logs/llm".to_string()),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
