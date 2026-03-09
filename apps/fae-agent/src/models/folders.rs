use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AllowedFolder {
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(rename = "isBase")]
    pub is_base: bool,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FolderSettings {
    #[serde(rename = "folderConfigs")]
    pub folder_configs: Vec<AllowedFolder>,
}

#[derive(Serialize, Deserialize)]
pub struct AllowedFolderRequest {
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(rename = "isBase")]
    pub is_base: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateFolderSettingsRequest {
    #[serde(rename = "folderConfigs")]
    pub folder_configs: Vec<AllowedFolderRequest>,
}

impl AllowedFolder {
    pub fn new(path: String, name: String, is_base: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            name,
            is_base,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

impl FolderSettings {
    pub fn empty() -> Self {
        Self {
            folder_configs: vec![],
        }
    }

    pub fn get_base_folder(&self) -> Option<&AllowedFolder> {
        self.folder_configs.iter().find(|f| f.is_base)
    }
}
