// Database models will go here
// Example structure for future use:

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a message in the system
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    /// Unique identifier for the message
    pub id: i32,
    /// Content of the message
    pub message: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request payload for chat interactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatPayload {
    /// Agent ID for the chat
    pub agent_id: String,
    /// User's message
    pub message: String,
}

/// Response payload for chat interactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    /// Unique identifier for the response
    pub id: String,
    /// Chat completion message
    pub message: String,
    /// Model used for completion
    pub model: String,
    /// Timestamp of creation
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[serde(rename = "provider_config_id")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "system_prompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatar_url")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// 自定义实现从数据库行解析
impl FromRow<'_, SqliteRow> for Agent {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let skills_json: String = row.try_get("skills")?; // 注意：实际DB列名是skills_json
        let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_else(|_| vec![]);
        let created_timestamp: i64 = row.try_get("created_at")?;
        let updated_timestamp: i64 = row.try_get("updated_at")?;

        let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(created_timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        Ok(Agent {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            provider: row.try_get("provider")?,
            provider_config_id: row.try_get("provider_config_id")?,
            model: row.try_get("model")?,
            system_prompt: row.try_get("system_prompt")?,
            avatar_url: row.try_get("avatar_url")?,
            skills,
            created_at,
            updated_at,
        })
    }
}

/// 存入数据库时的格式
impl Agent {
    pub fn to_db_params(
        &self,
    ) -> (
        String,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
        i64,
    ) {
        let skills_json = serde_json::to_string(&self.skills).unwrap_or_else(|_| "[]".to_string());
        let created_ts = self.created_at.timestamp();
        let updated_ts = self.updated_at.timestamp();

        (
            self.id.clone(),
            self.name.clone(),
            self.provider.clone(),
            self.provider_config_id.clone(),
            self.model.clone(),
            self.system_prompt.clone(),
            self.avatar_url.clone(),
            skills_json, // 存储为JSON字符串
            created_ts,
            updated_ts,
        )
    }

    pub fn from_db_columns(
        id: String,
        name: String,
        provider: String,
        provider_config_id: Option<String>,
        model: String,
        system_prompt: Option<String>,
        avatar_url: Option<String>,
        skills_json: String,
        created_at: i64,
        updated_at: i64,
    ) -> Self {
        let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_else(|_| vec![]);
        let created_dt = chrono::DateTime::<chrono::Utc>::from_timestamp(created_at, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let updated_dt = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        Agent {
            id,
            name,
            provider,
            provider_config_id,
            model,
            system_prompt,
            avatar_url,
            skills,
            created_at: created_dt,
            updated_at: updated_dt,
        }
    }
}

impl<'de> Deserialize<'de> for DbAgentRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let temp = TempAgent::deserialize(deserializer)?;
        Ok(DbAgentRow {
            id: temp.id,
            name: temp.name,
            provider: temp.provider,
            provider_config_id: temp.provider_config_id,
            model: temp.model,
            system_prompt: temp.system_prompt,
            avatar_url: temp.avatar_url,
            skills: temp.skills,
            created_at: temp.created_at,
            updated_at: temp.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct DbAgentRow {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[sqlx(rename = "provider_config_id")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[sqlx(rename = "system_prompt")]
    pub system_prompt: Option<String>,
    #[sqlx(rename = "avatar_url")]
    pub avatar_url: Option<String>,
    #[sqlx(rename = "skills_json")]
    pub skills: String, // raw json string from DB
    #[sqlx(rename = "created_at")]
    pub created_at: i64,
    #[sqlx(rename = "updated_at")]
    pub updated_at: i64,
}

impl DbAgentRow {
    pub fn to_agent(self) -> Agent {
        let skills: Vec<String> = serde_json::from_str(&self.skills).unwrap_or_else(|_| vec![]);
        let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp(self.created_at, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let updated_at = chrono::DateTime::<chrono::Utc>::from_timestamp(self.updated_at, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        Agent {
            id: self.id,
            name: self.name,
            provider: self.provider,
            provider_config_id: self.provider_config_id,
            model: self.model,
            system_prompt: self.system_prompt,
            avatar_url: self.avatar_url,
            skills,
            created_at,
            updated_at,
        }
    }

    pub fn from_agent(agent: &Agent, timestamp: i64) -> Self {
        let skills_json = serde_json::to_string(&agent.skills).unwrap_or_else(|_| "[]".to_string());
        let current_time = timestamp;

        DbAgentRow {
            id: agent.id.clone(),
            name: agent.name.clone(),
            provider: agent.provider.clone(),
            provider_config_id: agent.provider_config_id.clone(),
            model: agent.model.clone(),
            system_prompt: agent.system_prompt.clone(),
            avatar_url: agent.avatar_url.clone(),
            skills: skills_json,
            created_at: current_time,
            updated_at: current_time,
        }
    }
}

// Helper struct to match JSON request format
#[derive(Deserialize)]
struct TempAgent {
    id: String,
    name: String,
    provider: String,
    #[serde(rename = "provider_config_id")]
    provider_config_id: Option<String>,
    model: String,
    #[serde(rename = "system_prompt")]
    system_prompt: Option<String>,
    #[serde(rename = "avatar_url")]
    avatar_url: Option<String>,
    skills: Vec<String>,
    #[serde(rename = "created_at")]
    created_at: i64,
    #[serde(rename = "updated_at")]
    updated_at: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct DbAgent {
    pub id: String,
    pub name: String,
    pub provider: String,
    #[sqlx(rename = "provider_config_id")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[sqlx(rename = "system_prompt")]
    pub system_prompt: Option<String>,
    #[sqlx(rename = "avatar_url")]
    pub avatar_url: Option<String>,
    #[sqlx(rename = "skills_json")]
    pub skills_json: String,
    #[sqlx(rename = "created_at")]
    pub created_at_timestamp: i64,
    #[sqlx(rename = "updated_at")]
    pub updated_at_timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputedFields {
    #[serde(rename = "skills")]
    pub skills: Vec<String>,
    #[serde(rename = "created_at")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updated_at")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DbAgent> for Agent {
    fn from(db_agent: DbAgent) -> Self {
        let skills: Vec<String> =
            serde_json::from_str(&db_agent.skills_json).unwrap_or_else(|_| vec![]);

        let created_at =
            chrono::DateTime::<chrono::Utc>::from_timestamp(db_agent.created_at_timestamp, 0)
                .unwrap_or_else(|| chrono::Utc::now());
        let updated_at =
            chrono::DateTime::<chrono::Utc>::from_timestamp(db_agent.updated_at_timestamp, 0)
                .unwrap_or_else(|| chrono::Utc::now());

        Agent {
            id: db_agent.id,
            name: db_agent.name,
            provider: db_agent.provider,
            provider_config_id: db_agent.provider_config_id,
            model: db_agent.model,
            system_prompt: db_agent.system_prompt,
            avatar_url: db_agent.avatar_url,
            skills_json: db_agent.skills_json,
            computed_fields: ComputedFields {
                skills,
                created_at,
                updated_at,
            },
        }
    }
}

impl Agent {
    pub fn new_with_skills(
        id: String,
        name: String,
        provider: String,
        provider_config_id: Option<String>,
        model: String,
        system_prompt: Option<String>,
        avatar_url: Option<String>,
        skills: Vec<String>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        let skills_json = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string());

        Agent {
            id,
            name,
            provider,
            provider_config_id,
            model,
            system_prompt,
            avatar_url,
            skills_json,
            computed_fields: ComputedFields {
                skills,
                created_at,
                updated_at,
            },
        }
    }

    pub fn to_db_format(&self) -> DbAgent {
        DbAgent {
            id: self.id.clone(),
            name: self.name.clone(),
            provider: self.provider.clone(),
            provider_config_id: self.provider_config_id.clone(),
            model: self.model.clone(),
            system_prompt: self.system_prompt.clone(),
            avatar_url: self.avatar_url.clone(),
            skills_json: self.skills_json.clone(),
            created_at_timestamp: self.computed_fields.created_at.timestamp(),
            updated_at_timestamp: self.computed_fields.updated_at.timestamp(),
        }
    }

    pub fn get_skills(&self) -> Vec<String> {
        self.computed_fields.skills.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub provider: String,
    #[serde(rename = "providerConfigId")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: String,
    pub provider: String,
    #[serde(rename = "providerConfigId")]
    pub provider_config_id: Option<String>,
    pub model: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub skills: Vec<String>,
}

pub mod providers;
