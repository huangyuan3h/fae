use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;
use std::fs;
use tracing::info;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub enabled: i32,  // Using SQLite integer convention
}

#[derive(Deserialize)]
pub struct UpdateSkillRequest {
    pub enabled: bool,
}

#[derive(Serialize, Debug)]
pub struct SkillApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize, Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

pub async fn load_skills_from_directory(
    skills_dir: &str,
    db_pool: &SqlitePool,
) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
    let mut loaded_skills = Vec::new();
    
    // Check if skills directory exists
    if !Path::new(skills_dir).exists() {
        info!("Skills directory not found: {}", skills_dir);
        return Ok(loaded_skills);
    }
    
    // Iterate over directory entries
    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Process files or directories
        if path.is_file() {
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
                
            // Only process ts/js/mjs files
            if matches!(extension, "ts" | "js" | "mjs") {
                // For now, since this is Rust we can't execute TS/JS dynamically,
                // in production we would need to parse these files for metadata
                // For now, extract from filename as a placeholder
                let skill_id = path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
                    .to_string();
                    
                let skill_name = skill_id.clone(); // Default name = ID
                
                // Insert/update skill in database
                _ = sqlx::query(
                    "INSERT OR REPLACE INTO skills (id, name, enabled) VALUES (?1, ?2, COALESCE((SELECT enabled FROM skills WHERE id = ?1), 1))"
                )
                .bind(&skill_id)
                .bind(&skill_name)
                .execute(db_pool)
                .await?;
                
                let skill = Skill {
                    id: skill_id,
                    name: skill_name,
                    enabled: 1, // Default to enabled
                };
                
                loaded_skills.push(skill);
            }
        } else if path.is_dir() {
            // Could potentially look for SKILL.md files inside subdirectories
            // This allows for the extended skill format mentioned in the documentation
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                // For now just add the directory as a skill
                if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {
                    let skill_id = dir_name.to_string();
                    let skill_name = dir_name.to_string();
                    
                    sqlx::query(
                        "INSERT OR REPLACE INTO skills (id, name, enabled) VALUES (?1, ?2, COALESCE((SELECT enabled FROM skills WHERE id = ?1), 1))"
                    )
                    .bind(&skill_id)
                    .bind(&skill_name)
                    .execute(db_pool)
                    .await?;
                    
                    let skill = Skill {
                        id: skill_id,
                        name: skill_name,
                        enabled: 1, // Default to enabled
                    };
                    
                    loaded_skills.push(skill);
                }
            }
        }
    }
    
    info!("Scanned {} for skills", skills_dir);
    
    // Now also load all skills from the database to ensure we have them all
    let all_skills = sqlx::query_as::<_, Skill>("SELECT id, name, enabled FROM skills ORDER BY id")
        .fetch_all(db_pool)
        .await?;
    
    Ok(all_skills)
}

pub async fn get_all_skills(db_pool: &SqlitePool) -> Result<Vec<Skill>, sqlx::Error> {
    sqlx::query_as::<_, Skill>("SELECT id, name, enabled FROM skills ORDER BY id")
        .fetch_all(db_pool)
        .await
}

pub async fn update_skill(db_pool: &SqlitePool, skill_id: &str, enabled: bool) -> Result<Option<Skill>, sqlx::Error> {
    // Update enabled status
    sqlx::query("UPDATE skills SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(skill_id)
        .execute(db_pool)
        .await?;
    
    // Retrieve and return updated skill
    sqlx::query_as::<_, Skill>("SELECT id, name, enabled FROM skills WHERE id = ?")
        .bind(skill_id)
        .fetch_optional(db_pool)
        .await
}

// Function to get enabled skills for use by the agents
pub async fn get_enabled_skills(db_pool: &SqlitePool) -> Result<Vec<Skill>, sqlx::Error> {
    sqlx::query_as::<_, Skill>("SELECT id, name, enabled FROM skills WHERE enabled = 1 ORDER BY id")
        .fetch_all(db_pool)
        .await
}