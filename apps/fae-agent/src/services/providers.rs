use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use crate::models::providers::{ProviderConfig, ProviderSettings};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
struct Setting {
    key: String,
    value: String,
}

// AI Provider resolution module - handles provider configuration and selection
pub struct ProviderResolver;

impl ProviderResolver {
    // Gets all provider settings from the database
    pub async fn get_provider_settings(db_pool: &SqlitePool) -> Result<ProviderSettings, sqlx::Error> {
        // Try to get provider configs from settings
        match Self::get_setting(db_pool, "provider.configs").await {
            Ok(Some(provider_configs_raw)) => {
                // Try to decode the JSON settings
                match serde_json::from_str::<Vec<ProviderConfig>>(&provider_configs_raw).map(to_settings) {
                    Ok(settings) => Ok(settings),
                    Err(_) => {
                        // If JSON parsing fails, fall through to legacy format
                        Self::get_legacy_settings(db_pool).await
                    }
                }
            }
            Ok(None) | Err(_) => {
                // If there are no JSON configs, use defaults
                Self::get_legacy_settings(db_pool).await
            }
        }
    }
    
    // Gets individual setting from the database  
    pub async fn get_setting(db_pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row_option = sqlx::query_as::<_, Setting>("SELECT key, value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(db_pool)
            .await?;
            
        Ok(row_option.map(|row| row.value))
    }
    
    // Implementation for getting legacy settings (compatibility)
    async fn get_legacy_settings(db_pool: &SqlitePool) -> Result<ProviderSettings, sqlx::Error> {
        let ollama_base_url = Self::get_setting(db_pool, "provider.ollama.base_url")
            .await?
            .unwrap_or_else(|| "http://127.0.0.1:11434".to_string());
            
        let openai_api_key = Self::get_setting(db_pool, "provider.openai.api_key")
            .await?
            .unwrap_or_default();
            
        let openai_base_url = Self::get_setting(db_pool, "provider.openai.base_url")
            .await?
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
            
        let google_api_key = Self::get_setting(db_pool, "provider.google.api_key")
            .await?
            .unwrap_or_default();
            
        let google_base_url = Self::get_setting(db_pool, "provider.google.base_url")
            .await?
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
            

        let provider_configs = vec![
            ProviderConfig {
                id: "legacy-ollama".to_string(),
                name: "Ollama Local".to_string(),
                provider_type: "ollama".to_string(),
                api_key: "".to_string(),
                base_url: ollama_base_url,
                model_id: "".to_string(),
                enabled: true,
            },
            ProviderConfig {
                id: "legacy-openai".to_string(),
                name: "OpenAI Default".to_string(),
                provider_type: "openai".to_string(),
                api_key: openai_api_key,
                base_url: openai_base_url,
                model_id: "".to_string(),
                enabled: true,
            },
            ProviderConfig {
                id: "legacy-google".to_string(),
                name: "Google Default".to_string(),
                provider_type: "google".to_string(),
                api_key: google_api_key,
                base_url: google_base_url,
                model_id: "".to_string(),
                enabled: true,
            },
        ];

        Ok(to_settings(provider_configs))
    }
    
    // Resolves which provider to use based on preference and availability
    #[allow(dead_code)]  // This function is now available for export
    pub fn resolve_provider(
        preferred: Option<String>,
        settings: &ProviderSettings,
        provider_config_id: Option<String>,
    ) -> String {
        // If user specified a config ID, try to use that first
        if let Some(config_id) = provider_config_id {
            for config in &settings.provider_configs {
                if config.id == config_id {
                    return config.provider_type.clone();
                }
            }
        }
        
        // Otherwise use preferred provider or fallback to default
        preferred.unwrap_or_else(|| {
            settings.provider_configs.first()
                .map(|config| config.provider_type.clone())
                .unwrap_or_else(|| "ollama".to_string())
        })
    }
    
    // Updates provider settings in the database
    #[allow(dead_code)]  // This function is now available for export
    pub async fn save_provider_settings(
        db_pool: &SqlitePool,
        settings: &ProviderSettings,
    ) -> Result<(), sqlx::Error> {
        // Use the actual field for db storage and serialization
        let providers_for_storage = &settings.provider_configs;
        
        let json_value = match serde_json::to_string(providers_for_storage) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize provider settings: {}", e);
                return Err(sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Serialization failed"
                )));
            }
        };
        
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind("provider.configs")
            .bind(&json_value)
            .execute(&*db_pool)  // Dereference the Arc here as well
            .await?;
        
        Ok(())
    }
}

// Simple conversion helper function for provider configs
fn to_settings(provider_configs: Vec<ProviderConfig>) -> ProviderSettings {
    ProviderSettings {
        provider_configs,  // Return the new structure with the appropriate JSON mapping
    }
}