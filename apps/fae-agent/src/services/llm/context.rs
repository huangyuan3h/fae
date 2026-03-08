use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub avatar_url: Option<String>,
    pub skills: Vec<SkillInfo>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

impl AgentContext {
    pub fn new(id: String, name: String, provider: String, model: String) -> Self {
        Self {
            id,
            name,
            provider,
            model,
            system_prompt: None,
            avatar_url: None,
            skills: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    pub fn with_avatar_url(mut self, url: String) -> Self {
        self.avatar_url = Some(url);
        self
    }

    pub fn with_skills(mut self, skills: Vec<SkillInfo>) -> Self {
        self.skills = skills;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn get_enabled_skills(&self) -> Vec<&SkillInfo> {
        self.skills.iter().filter(|s| s.enabled).collect()
    }

    pub fn has_skills(&self) -> bool {
        !self.skills.is_empty()
    }

    pub fn get_skill_names(&self) -> Vec<&str> {
        self.skills.iter().map(|s| s.id.as_str()).collect()
    }
}

impl From<crate::models::Agent> for AgentContext {
    fn from(agent: crate::models::Agent) -> Self {
        let skills = agent
            .skills
            .into_iter()
            .map(|skill_id| SkillInfo {
                id: skill_id.clone(),
                name: skill_id.clone(),
                description: String::new(),
                enabled: true,
            })
            .collect();

        AgentContext {
            id: agent.id,
            name: agent.name,
            provider: agent.provider,
            model: agent.model,
            system_prompt: agent.system_prompt,
            avatar_url: agent.avatar_url,
            skills,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context_creation() {
        let ctx = AgentContext::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        );

        assert_eq!(ctx.id, "agent-1");
        assert_eq!(ctx.name, "Test Agent");
        assert!(ctx.system_prompt.is_none());
        assert!(ctx.skills.is_empty());
    }

    #[test]
    fn test_agent_context_with_system_prompt() {
        let ctx = AgentContext::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        )
        .with_system_prompt("You are helpful".to_string());

        assert_eq!(ctx.system_prompt, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_agent_context_with_skills() {
        let skills = vec![
            SkillInfo {
                id: "search".to_string(),
                name: "Search".to_string(),
                description: "Search the web".to_string(),
                enabled: true,
            },
            SkillInfo {
                id: "calculate".to_string(),
                name: "Calculate".to_string(),
                description: "Do math".to_string(),
                enabled: false,
            },
        ];

        let ctx = AgentContext::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        )
        .with_skills(skills);

        assert_eq!(ctx.skills.len(), 2);
        assert_eq!(ctx.get_enabled_skills().len(), 1);
        assert!(ctx.has_skills());
    }

    #[test]
    fn test_agent_context_metadata() {
        let ctx = AgentContext::new(
            "agent-1".to_string(),
            "Test Agent".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        )
        .with_metadata("version".to_string(), "1.0".to_string())
        .with_metadata("env".to_string(), "production".to_string());

        assert_eq!(ctx.metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(ctx.metadata.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_agent_context_from_model() {
        let agent = crate::models::Agent {
            id: "agent-1".to_string(),
            name: "Test Agent".to_string(),
            provider: "ollama".to_string(),
            provider_config_id: None,
            model: "llama2".to_string(),
            system_prompt: Some("Be helpful".to_string()),
            avatar_url: None,
            skills: vec!["search".to_string(), "calculate".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let ctx = AgentContext::from(agent);

        assert_eq!(ctx.id, "agent-1");
        assert_eq!(ctx.name, "Test Agent");
        assert_eq!(ctx.system_prompt, Some("Be helpful".to_string()));
        assert_eq!(ctx.skills.len(), 2);
        assert_eq!(ctx.skills[0].id, "search");
    }
}
