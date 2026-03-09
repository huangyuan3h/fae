use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::models::folders::{AllowedFolder, FolderSettings};

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

pub async fn get_folders_handler(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<FolderSettings>>, StatusCode> {
    match get_folder_settings(&state.db_pool).await {
        Ok(settings) => Ok(Json(ApiResponse {
            ok: true,
            data: Some(settings),
            error: None,
        })),
        Err(e) => {
            eprintln!("Failed to get folder settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateFoldersPayload {
    #[serde(rename = "folderConfigs")]
    pub folder_configs: Vec<AllowedFolderRequest>,
}

#[derive(Deserialize)]
pub struct AllowedFolderRequest {
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(rename = "isBase")]
    pub is_base: bool,
}

pub async fn update_folders_handler(
    State(state): State<AppState>,
    Json(payload): Json<UpdateFoldersPayload>,
) -> Result<Json<ApiResponse<FolderSettings>>, StatusCode> {
    match save_folder_settings(&state.db_pool, &payload.folder_configs).await {
        Ok(settings) => {
            info!("Folder settings updated successfully");
            Ok(Json(ApiResponse {
                ok: true,
                data: Some(settings),
                error: None,
            }))
        },
        Err(e) => {
            eprintln!("Failed to save folder settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_folder_settings(
    db_pool: &sqlx::SqlitePool,
) -> Result<FolderSettings, sqlx::Error> {
    let rows = sqlx::query_as::<_, AllowedFolder>(
        "SELECT id, path, name, is_base as is_base, created_at FROM allowed_folders ORDER BY created_at DESC"
    )
    .fetch_all(db_pool)
    .await?;

    Ok(FolderSettings {
        folder_configs: rows,
    })
}

pub async fn save_folder_settings(
    db_pool: &sqlx::SqlitePool,
    folder_configs: &[AllowedFolderRequest],
) -> Result<FolderSettings, sqlx::Error> {
    let mut tx = db_pool.begin().await?;

    sqlx::query("DELETE FROM allowed_folders")
        .execute(&mut *tx)
        .await?;

    for config in folder_configs {
        let is_base = if config.is_base { 1 } else { 0 };
        sqlx::query(
            "INSERT INTO allowed_folders (id, path, name, is_base, created_at) VALUES (?, ?, ?, ?, strftime('%s', 'now'))"
        )
        .bind(&config.id)
        .bind(&config.path)
        .bind(&config.name)
        .bind(is_base)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    get_folder_settings(db_pool).await
}

pub async fn get_base_folder(db_pool: &sqlx::SqlitePool) -> Option<AllowedFolder> {
    match sqlx::query_as::<_, AllowedFolder>(
        "SELECT id, path, name, is_base as is_base, created_at FROM allowed_folders WHERE is_base = 1 LIMIT 1"
    )
    .fetch_optional(db_pool)
    .await
    {
        Ok(Some(folder)) => Some(folder),
        _ => None,
    }
}

pub async fn create_default_base_folder(db_pool: &sqlx::SqlitePool) -> Result<AllowedFolder, sqlx::Error> {
    let default_path = std::env::current_dir()
        .map(|p| p.join("fae-workspace"))
        .unwrap_or_else(|_| std::path::PathBuf::from("./fae-workspace"));

    let path_str = default_path.to_string_lossy().to_string();

    if !default_path.exists() {
        std::fs::create_dir_all(&default_path)
            .map_err(|e| sqlx::Error::Protocol(format!("Failed to create default workspace: {}", e)))?;
    }

    let folder = AllowedFolder::new(path_str.clone(), "Default Workspace".to_string(), true);

    let is_base = if folder.is_base { 1 } else { 0 };
    sqlx::query(
        "INSERT INTO allowed_folders (id, path, name, is_base, created_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&folder.id)
    .bind(&folder.path)
    .bind(&folder.name)
    .bind(is_base)
    .bind(folder.created_at)
    .execute(db_pool)
    .await?;

    Ok(folder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqlitePool;

    #[tokio::test]
    async fn test_get_empty_folder_settings() {
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();

        let settings = get_folder_settings(&db_pool).await.unwrap();
        assert_eq!(settings.folder_configs.len(), 0);
    }

    #[tokio::test]
    async fn test_save_and_get_folder_settings() {
        let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db_pool).await.unwrap();

        let configs = vec![
            AllowedFolderRequest {
                id: "folder-1".to_string(),
                path: "/tmp/test1".to_string(),
                name: "Test Folder 1".to_string(),
                is_base: true,
            },
            AllowedFolderRequest {
                id: "folder-2".to_string(),
                path: "/tmp/test2".to_string(),
                name: "Test Folder 2".to_string(),
                is_base: false,
            },
        ];

        let saved = save_folder_settings(&db_pool, &configs).await.unwrap();
        assert_eq!(saved.folder_configs.len(), 2);

        let loaded = get_folder_settings(&db_pool).await.unwrap();
        assert_eq!(loaded.folder_configs.len(), 2);
        assert_eq!(loaded.folder_configs[0].name, "Test Folder 1");
    }
}