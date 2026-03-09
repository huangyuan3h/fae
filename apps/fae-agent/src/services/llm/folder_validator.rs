use std::path::Path;
use sqlx::SqlitePool;
use crate::models::folders::{AllowedFolder, FolderSettings};
use super::folders_api::{get_folder_settings, get_base_folder, create_default_base_folder};

pub struct FolderValidator {
    allowed_folders: Vec<AllowedFolder>,
    base_folder: Option<AllowedFolder>,
}

impl FolderValidator {
    pub async fn new(db_pool: &SqlitePool) -> Self {
        let settings = get_folder_settings(db_pool).await.unwrap_or_else(|_| FolderSettings::empty());
        
        let base_folder = match get_base_folder(db_pool).await {
            Some(folder) => Some(folder),
            None => {
                match create_default_base_folder(db_pool).await {
                    Ok(folder) => Some(folder),
                    Err(_) => None,
                }
            }
        };

        Self {
            allowed_folders: settings.folder_configs,
            base_folder,
        }
    }

    pub fn from_settings(settings: FolderSettings) -> Self {
        let base_folder = settings.get_base_folder().cloned();
        Self {
            allowed_folders: settings.folder_configs,
            base_folder,
        }
    }

    pub fn validate_path(&self, path: &str) -> Result<String, String> {
        let resolved_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            match &self.base_folder {
                Some(base) => Path::new(&base.path).join(path),
                None => return Err("No base folder configured. Please set up a base folder in settings.".to_string()),
            }
        };

        let canonical_path = resolved_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        for folder in &self.allowed_folders {
            if canonical_path.starts_with(&folder.path) {
                return Ok(canonical_path.to_string_lossy().to_string());
            }
        }

        Err(format!(
            "Path '{}' is not in any allowed folder. Please add it to your allowed folders list in settings.",
            path
        ))
    }

    pub fn get_base_folder(&self) -> Option<&AllowedFolder> {
        self.base_folder.as_ref()
    }

    pub fn get_working_directory(&self) -> String {
        self.base_folder
            .as_ref()
            .map(|f| f.path.clone())
            .unwrap_or_else(|| std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string())
    }
}