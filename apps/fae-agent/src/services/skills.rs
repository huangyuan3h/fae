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

// Simple function to extract YAML frontmatter from markdown
fn extract_yaml_frontmatter(content: &str) -> Option<(std::collections::HashMap<String, String>, String)> {
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut frontmatter_end = None;
    
    // Find the end of frontmatter (first line that equals "---" after the first one)
    for (i, line) in lines.iter().enumerate().skip(1) { // skip first line which is "---"
        if *line == "---" {
            frontmatter_end = Some(i);
            break;
        }
    }

    if let Some(end_idx) = frontmatter_end {
        let frontmatter_content = lines[1..end_idx].join("\n");
        
        // Very simple YAML parser for basic key-value pairs (without pulling in yaml library dependency)
        let mut metadata = std::collections::HashMap::new();
        for line in frontmatter_content.lines() {
            if line.contains(":") {
                let parts: Vec<&str> = line.splitn(2, ':').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim().trim_matches('"');
                    metadata.insert(key.to_string(), value.to_string());
                }
            }
        }

        // Remaining content after frontmatter
        let remaining_content = lines[(end_idx + 1)..].join("\n").trim().to_string();
        
        return Some((metadata, remaining_content));
    }
    
    None
}

pub async fn load_skills_from_directory(
    skills_dir: &str,
    db_pool: &SqlitePool,
) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
    let mut loaded_skills = Vec::new();
    
    // Check if skills directory exists
    if !Path::new(skills_dir).exists() {
        info!("Skills directory not found: {}", skills_dir);
        // Even if the directory doesn't exist, return all existing skills from the database
        let all_skills = sqlx::query_as::<_, Skill>("SELECT id, name, enabled FROM skills ORDER BY id")
            .fetch_all(db_pool)
            .await?;
        return Ok(all_skills);
    }
    
    // Get list of expected skills (those in the directory)
    let mut expected_skills = Vec::new();
    
    // Iterate over directory entries to collect expected skills in directory
    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // We only process directories which may contain a SKILL.md file
        if path.is_dir() {
            let skill_md_path = path.join("SKILL.md");
            
            if skill_md_path.exists() {
                let content = fs::read_to_string(&skill_md_path)?;
                
                // Try to parse YAML frontmatter and extract name/description
                let (skill_name, skill_description) = if let Some((metadata, _)) = extract_yaml_frontmatter(&content) {
                    let extracted_name = metadata.get("name")
                        .cloned()
                        .unwrap_or_else(|| {
                            // Use directory name as fallback
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or_default()
                                .to_string()
                        });
                    
                    let extracted_description = metadata.get("description")
                        .cloned()
                        .unwrap_or_else(|| "No description provided".to_string());
                    
                    (extracted_name, extracted_description)
                } else {
                    // If no frontmatter found, use directory name and default description
                    let fallback_name = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .to_string();
                    (fallback_name, "No description provided".to_string())
                };

                expected_skills.push((skill_name.clone(), skill_description.clone()));
                
                // Upsert this skill in the database
                sqlx::query(
                    "INSERT OR REPLACE INTO skills (id, name, enabled) VALUES (?1, ?2, COALESCE((SELECT enabled FROM skills WHERE id = ?1), 1))"
                )
                .bind(&skill_name)
                .bind(&skill_description)
                .execute(db_pool)
                .await?;
                
                let skill = Skill {
                    id: skill_name,
                    name: skill_description,
                    enabled: 1, // Default to enabled
                };
                
                loaded_skills.push(skill);
            }
        }
    }
    
    // Find and delete skills that are in the database but no longer exist in the directory
    let existing_db_skills: std::collections::HashSet<String> = sqlx::query_scalar("SELECT id FROM skills")
        .fetch_all(db_pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|id: String| id)
        .collect();
    
    let expected_db_skills: std::collections::HashSet<String> = expected_skills
        .iter()
        .map(|(id, _)| id.clone())
        .collect();
    
    let skills_to_delete: Vec<&String> = existing_db_skills.difference(&expected_db_skills).collect();
    
    for skill_id in skills_to_delete {
        sqlx::query("DELETE FROM skills WHERE id = ?")
            .bind(skill_id)
            .execute(db_pool)
            .await?;
        info!("Deleted skill '{}' as it no longer exists in {}", skill_id, skills_dir);
    }
    
    info!("Scanned {} and found {} skills from directory", skills_dir, loaded_skills.len());
    
    // Now also load all skills from the database (this will reflect the deletion/cleaning too)
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

pub async fn refresh_skills_from_directory(
    skills_dir: &str,
    db_pool: &SqlitePool,
) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
    load_skills_from_directory(skills_dir, db_pool).await
}