use axum::{
    extract::{State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::models::providers::{ProviderConfig, ProviderSettings};

use super::providers::ProviderResolver;
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

pub async fn get_providers_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ProviderSettings>>, StatusCode> {
    match ProviderResolver::get_provider_settings(&state.db_pool).await {
        Ok(settings) => Ok(Json(ApiResponse {
            ok: true,
            data: Some(settings),
            error: None,
        })),
        Err(e) => {
            eprintln!("Failed to get provider settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateProvidersPayload {
    pub provider_configs: Vec<ProviderConfig>,
}

pub async fn update_providers_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateProvidersPayload>,
) -> Result<Json<ApiResponse<ProviderSettings>>, StatusCode> {
    let settings = ProviderSettings {
        provider_configs: payload.provider_configs,
        default_provider: "ollama".to_string(), // Will be set based on first valid config
    };
    
    match ProviderResolver::save_provider_settings(&state.db_pool, &settings).await {
        Ok(()) => {
            info!("Provider settings updated successfully");
            Ok(Json(ApiResponse {
                ok: true,
                data: Some(settings),
                error: None,
            }))
        },
        Err(e) => {
            eprintln!("Failed to save provider settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_ollama_settings_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<OllamaSettings>>, StatusCode> {
    // Get the current Ollama base URL from settings
    match ProviderResolver::get_setting(&state.db_pool, "provider.ollama.base_url").await {
        Ok(Some(base_url)) => {
            let settings = OllamaSettings { base_url };
            Ok(Json(ApiResponse {
                ok: true,
                data: Some(settings),
                error: None,
            }))
        },
        Ok(None) => {
            // Return default value
            let settings = OllamaSettings { 
                base_url: "http://127.0.0.1:11434".to_string() 
            };
            Ok(Json(ApiResponse {
                ok: true,
                data: Some(settings),
                error: None,
            }))
        },
        Err(e) => {
            eprintln!("Error retrieving Ollama settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OllamaSettings {
    pub base_url: String,
}

#[derive(Deserialize)]
pub struct UpdateOllamaPayload {
    pub base_url: String,
}

pub async fn update_ollama_settings_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateOllamaPayload>,
) -> Result<Json<ApiResponse<OllamaSettings>>, StatusCode> {
    // Save the new base URL in settings
    if let Err(e) = sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind("provider.ollama.base_url")
        .bind(&payload.base_url)
        .execute(&*state.db_pool)  // Dereference the Arc here
        .await {
            eprintln!("Failed to save Ollama settings: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Ollama settings updated to: {}", payload.base_url);
    
    Ok(Json(ApiResponse {
        ok: true,
        data: Some(OllamaSettings {
            base_url: payload.base_url,
        }),
        error: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{SqlitePool};
    use crate::AppState;
    use tokio;

    #[tokio::test]
    async fn test_update_ollama_settings_handler() {
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();
        
        let state = AppState {
            db_pool: std::sync::Arc::new(db_pool.clone()),
        };
        
        let new_url = UpdateOllamaPayload {
            base_url: "http://new-ollama:11434".to_string()
        };
        
        let result = update_ollama_settings_handler(State(state), Json(new_url)).await.unwrap();
        
        assert_eq!(result.ok, true);
        assert!(result.data.is_some());
        assert_eq!(result.data.clone().unwrap().base_url, "http://new-ollama:11434");
        
        // Verify it was persisted
        let persisted_url = ProviderResolver::get_setting(&db_pool, "provider.ollama.base_url")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(persisted_url, "http://new-ollama:11434");
    }
    
    #[tokio::test]
    async fn test_get_ollama_settings_handler_default() {
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();
        
        let state = AppState {
            db_pool: std::sync::Arc::new(db_pool),
        };
        
        let result = get_ollama_settings_handler(State(state)).await.unwrap();
        
        assert_eq!(result.ok, true);
        assert!(result.data.is_some());
        assert_eq!(result.data.clone().unwrap().base_url, "http://127.0.0.1:11434");
    }
    
    #[tokio::test]
    async fn test_update_and_get_new_providers() {
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();
        
        let state = AppState {
            db_pool: std::sync::Arc::new(db_pool.clone()),
        };
        
        // Create new provider configuration
        let new_provider_config = ProviderConfig {
            id: "test-provider".to_string(),
            name: "Test Provider".to_string(),
            provider_type: "openai".to_string(),
            api_key: "fake-key".to_string(),
            base_url: "https://api.test.com/v1".to_string(),
            model_id: "gpt-4-test".to_string(),
            enabled: true,
        };
        
        let update_payload = UpdateProvidersPayload {
            provider_configs: vec![new_provider_config],
        };
        
        let result = update_providers_handler(State(state), Json(update_payload)).await.unwrap();
        
        assert_eq!(result.ok, true);
        assert!(result.data.is_some());
        let settings = result.data.clone().unwrap();
        assert_eq!(settings.provider_configs.len(), 1);
        assert_eq!(settings.provider_configs[0].name, "Test Provider");
        assert_eq!(settings.provider_configs[0].provider_type, "openai");
        
        // Now get the providers back to verify they were saved
        let state = AppState {
            db_pool: std::sync::Arc::new(db_pool),
        };
        
        let get_result = get_providers_handler(State(state)).await.unwrap();
        assert_eq!(get_result.ok, true);
        assert!(get_result.data.is_some());
        let saved_settings = get_result.data.clone().unwrap();
        assert_eq!(saved_settings.provider_configs.len(), 1);
        assert_eq!(saved_settings.provider_configs[0].id, "test-provider");
    }
}