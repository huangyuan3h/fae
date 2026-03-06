#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{SqlitePool, Connection};
    use std::collections::HashMap;

    fn sample_providers() -> Vec<ProviderConfig> {
        vec![
            ProviderConfig {
                id: "test-1".to_string(),
                name: "Test Provider 1".to_string(),
                provider_type: "openai".to_string(),
                api_key: "test-key-1".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                model_id: "gpt-4".to_string(),
                enabled: true,
            },
            ProviderConfig {
                id: "test-2".to_string(),
                name: "Test Provider 2".to_string(),
                provider_type: "ollama".to_string(),
                api_key: "".to_string(),
                base_url: "http://127.0.0.1:11434".to_string(),
                model_id: "llama2".to_string(),
                enabled: true,
            }
        ]
    }

    #[tokio::test]
    async fn test_resolver_get_provider_settings_empty_db() {
        let db_url = "sqlite::memory:";
        let pool = SqlitePool::connect(db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let settings = ProviderResolver::get_provider_settings(&pool).await.unwrap();
        
        assert_eq!(settings.provider_configs.len(), 3); // Default legacy configs
        assert_eq!(settings.default_provider, "ollama");
    }

    #[tokio::test]
    async fn test_resolver_save_and_get_settings() {
        let db_url = "sqlite::memory:";
        let pool = SqlitePool::connect(db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let original_providers = sample_providers();
        let original_settings = ProviderSettings {
            provider_configs: original_providers.clone(),
            default_provider: "openai".to_string(),
        };
        
        // Save settings
        ProviderResolver::save_provider_settings(&pool, &original_settings).await.unwrap();
        
        // Get settings back
        let retrieved_settings = ProviderResolver::get_provider_settings(&pool).await.unwrap();
        
        assert_eq!(retrieved_settings.provider_configs.len(), 2);
        assert_eq!(retrieved_settings.default_provider, "openai");
        
        // Check first provider
        let first_provider = &retrieved_settings.provider_configs[0];
        assert_eq!(first_provider.id, "test-1");
        assert_eq!(first_provider.provider_type, "openai");
        assert_eq!(first_provider.enabled, true);
    }

    #[tokio::test]
    async fn test_resolver_update_existing_settings() {
        let db_url = "sqlite::memory:";
        let pool = SqlitePool::connect(db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        // Initial settings
        let initial_settings = ProviderSettings {
            provider_configs: vec![ProviderConfig {
                id: "initial".to_string(),
                name: "Initial Provider".to_string(),
                provider_type: "ollama".to_string(),
                api_key: "".to_string(),
                base_url: "http://127.0.0.1:11434".to_string(),
                model_id: "llama2".to_string(),
                enabled: true,
            }],
            default_provider: "ollama".to_string()
        };
        
        ProviderResolver::save_provider_settings(&pool, &initial_settings).await.unwrap();
        let retrieved = ProviderResolver::get_provider_settings(&pool).await.unwrap();
        assert_eq!(retrieved.provider_configs.len(), 1);
        
        // Updated settings
        let updated_settings = ProviderSettings {
            provider_configs: sample_providers(),
            default_provider: "openai".to_string(),
        };
        
        ProviderResolver::save_provider_settings(&pool, &updated_settings).await.unwrap();
        let updated = ProviderResolver::get_provider_settings(&pool).await.unwrap();
        
        assert_eq!(updated.provider_configs.len(), 2);
        assert_eq!(updated.default_provider, "openai");
    }

    #[tokio::test]
    async fn test_resolver_get_setting_not_exists() {
        let db_url = "sqlite::memory:";
        let pool = SqlitePool::connect(db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let result = ProviderResolver::get_setting(&pool, "non-existent-setting").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_resolver_get_specific_setting() {
        let db_url = "sqlite::memory:";
        let pool = SqlitePool::connect(db_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        // First, manually insert setting to db
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
            .bind("test-setting")
            .bind("test-value")
            .execute(&pool)
            .await
            .unwrap();
        
        let result = ProviderResolver::get_setting(&pool, "test-setting").await.unwrap();
        assert_eq!(result, Some("test-value".to_string()));
    }

    #[test]
    fn test_resolve_provider_default_behavior() {
        let test_providers = sample_providers();
        let settings = ProviderSettings {
            provider_configs: test_providers,
            default_provider: "ollama".to_string(),
        };
        
        // When no preferred provider is given, should return default
        let resolved = ProviderResolver::resolve_provider(None, &settings, None);
        assert_eq!(resolved, "ollama");
        
        // When preferred is given and exists in settings, should return preferred
        let resolved = ProviderResolver::resolve_provider(Some("openai".to_string()), &settings, None);
        assert_eq!(resolved, "openai");
    }

    #[test]
    fn test_resolve_provider_with_config_id() {
        let test_providers = sample_providers();
        let settings = ProviderSettings {
            provider_configs: test_providers,
            default_provider: "ollama".to_string(),
        };
        
        // If config ID matches a provider, should return that provider type regardless of preferred
        let resolved = ProviderResolver::resolve_provider(Some("openai".to_string()), &settings, Some("test-2".to_string()));
        assert_eq!(resolved, "ollama"); // Config ID "test-2" is an "ollama" type
    }
    
    #[test]
    fn test_resolve_provider_config_id_priority_over_pref() {
        let test_providers = sample_providers();
        let settings = ProviderSettings {
            provider_configs: test_providers,
            default_provider: "ollama".to_string(),
        };
        
        // Config ID has priority over preferred provider
        let resolved = ProviderResolver::resolve_provider(
            Some("google".to_string()),  // preferred
            &settings,
            Some("test-1".to_string())  // config specifies openai
        );
        assert_eq!(resolved, "openai"); // Should be openai (from config) not google (preferred)
    }
    
    #[test]
    fn test_resolve_provider_returns_pref_if_no_valid_config_id() {
        let test_providers = sample_providers();
        let settings = ProviderSettings {
            provider_configs: test_providers,
            default_provider: "ollama".to_string(),
        };
        
        // If config ID doesn't match, fall back to preferred provider
        let resolved = ProviderResolver::resolve_provider(
            Some("openai".to_string()),  // preferred
            &settings,
            Some("nonexistent-id".to_string())  // doesn't exist
        );
        assert_eq!(resolved, "openai");
    }
    
    #[test]
    fn test_resolve_provider_defaults() {
        let settings = ProviderSettings {
            provider_configs: vec![], // Empty configs
            default_provider: "google".to_string(),
        };
        
        // If no preferred and no matching config, return setting default
        let resolved = ProviderResolver::resolve_provider(None, &settings, None);
        assert_eq!(resolved, "google");
    }
}