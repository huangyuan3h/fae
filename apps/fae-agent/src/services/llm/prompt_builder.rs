use super::context::AgentContext;
use super::models::{ToolDefinition, ToolFunction};
use super::system_context::SystemContext;

pub struct PromptBuilder {
    base_prompt: String,
    include_agent_info: bool,
    include_skill_list: bool,
    include_system_context: bool,
    custom_instructions: Vec<String>,
    system_context: Option<SystemContext>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            base_prompt: "You are a helpful assistant.".to_string(),
            include_agent_info: true,
            include_skill_list: true,
            include_system_context: true,
            custom_instructions: Vec::new(),
            system_context: None,
        }
    }

    pub fn with_base_prompt(mut self, prompt: String) -> Self {
        self.base_prompt = prompt;
        self
    }

    pub fn with_agent_info(mut self, include: bool) -> Self {
        self.include_agent_info = include;
        self
    }

    pub fn with_skill_list(mut self, include: bool) -> Self {
        self.include_skill_list = include;
        self
    }

    pub fn with_system_context(mut self, include: bool) -> Self {
        self.include_system_context = include;
        self
    }

    pub fn set_system_context(mut self, ctx: SystemContext) -> Self {
        self.system_context = Some(ctx);
        self
    }

    pub fn add_custom_instruction(mut self, instruction: String) -> Self {
        self.custom_instructions.push(instruction);
        self
    }

    pub fn build(&self, context: &AgentContext) -> String {
        let mut parts = Vec::new();

        if self.include_system_context {
            if let Some(ref sys_ctx) = self.system_context {
                parts.push(sys_ctx.to_prompt_section());
            }
        }

        if self.include_agent_info {
            parts.push(self.build_agent_info(context));
        }

        if context.system_prompt.is_some() {
            parts.push(context.system_prompt.clone().unwrap());
        } else if !self.base_prompt.is_empty() {
            parts.push(self.base_prompt.clone());
        }

        if self.include_skill_list && context.has_skills() {
            parts.push(self.build_skill_section(context));
        }

        for instruction in &self.custom_instructions {
            parts.push(instruction.clone());
        }

        parts.join("\n\n")
    }

    fn build_agent_info(&self, context: &AgentContext) -> String {
        format!(
            "You are {} (ID: {}), an AI assistant powered by {} running on {} model.",
            context.name, context.id, context.provider, context.model
        )
    }

    fn build_skill_section(&self, context: &AgentContext) -> String {
        let enabled_skills = context.get_enabled_skills();

        if enabled_skills.is_empty() {
            return String::new();
        }

        let skill_list: Vec<String> = enabled_skills
            .iter()
            .map(|s| {
                if s.description.is_empty() {
                    format!("- {}: {}", s.id, s.name)
                } else {
                    format!("- {}: {}", s.id, s.description)
                }
            })
            .collect();

        format!(
            "You have access to the following tools/skills:\n{}\n\n\
            You should use tools when they would help answer the user's question. \
            Think step by step about whether you need to use any tools. \
            If you use a tool, explain what you're doing and why.",
            skill_list.join("\n")
        )
    }

    pub fn build_tools(&self, context: &AgentContext) -> Vec<ToolDefinition> {
        context
            .get_enabled_skills()
            .into_iter()
            .map(|skill| ToolDefinition {
                tool_type: "function".to_string(),
                function: Some(ToolFunction {
                    name: skill.id.clone(),
                    description: if skill.description.is_empty() {
                        format!("Execute the {} skill", skill.name)
                    } else {
                        skill.description.clone()
                    },
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "input": {
                                "type": "string",
                                "description": format!("Input for the {} skill", skill.name)
                            }
                        },
                        "required": ["input"]
                    }),
                }),
            })
            .collect()
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::llm::context::SkillInfo;

    fn create_test_context() -> AgentContext {
        AgentContext::new(
            "agent-123".to_string(),
            "TestBot".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        )
        .with_system_prompt("You are a test assistant.".to_string())
        .with_skills(vec![
            SkillInfo {
                id: "search".to_string(),
                name: "Search".to_string(),
                description: "Search the web".to_string(),
                enabled: true,
            },
            SkillInfo {
                id: "calculate".to_string(),
                name: "Calculate".to_string(),
                description: "Perform calculations".to_string(),
                enabled: true,
            },
        ])
    }

    #[test]
    fn test_prompt_builder_basic() {
        let context = create_test_context();
        let builder = PromptBuilder::new();
        let prompt = builder.build(&context);

        assert!(prompt.contains("TestBot"));
        assert!(prompt.contains("agent-123"));
        assert!(prompt.contains("openai"));
        assert!(prompt.contains("gpt-4"));
    }

    #[test]
    fn test_prompt_builder_with_custom_instruction() {
        let context = create_test_context();
        let builder = PromptBuilder::new().add_custom_instruction("Always be polite.".to_string());

        let prompt = builder.build(&context);
        assert!(prompt.contains("Always be polite."));
    }

    #[test]
    fn test_prompt_builder_without_agent_info() {
        let context = create_test_context();
        let builder = PromptBuilder::new().with_agent_info(false);

        let prompt = builder.build(&context);
        assert!(!prompt.contains("You are TestBot"));
    }

    #[test]
    fn test_prompt_builder_without_skills() {
        let context = create_test_context();
        let builder = PromptBuilder::new().with_skill_list(false);

        let prompt = builder.build(&context);
        assert!(!prompt.contains("You have access to the following tools"));
    }

    #[test]
    fn test_prompt_builder_tools() {
        let context = create_test_context();
        let builder = PromptBuilder::new();
        let tools = builder.build_tools(&context);

        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].function.as_ref().unwrap().name, "search");
        assert_eq!(tools[1].function.as_ref().unwrap().name, "calculate");
    }

    #[test]
    fn test_prompt_builder_no_skills() {
        let context = AgentContext::new(
            "agent-1".to_string(),
            "TestBot".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        );

        let builder = PromptBuilder::new();
        let tools = builder.build_tools(&context);

        assert!(tools.is_empty());
    }

    #[test]
    fn test_prompt_builder_skill_descriptions() {
        let context = AgentContext::new(
            "agent-1".to_string(),
            "TestBot".to_string(),
            "openai".to_string(),
            "gpt-4".to_string(),
        )
        .with_skills(vec![
            SkillInfo {
                id: "skill1".to_string(),
                name: "Skill One".to_string(),
                description: "Description for skill 1".to_string(),
                enabled: true,
            },
            SkillInfo {
                id: "skill2".to_string(),
                name: "Skill Two".to_string(),
                description: String::new(),
                enabled: true,
            },
        ]);

        let builder = PromptBuilder::new();
        let tools = builder.build_tools(&context);

        assert_eq!(
            tools[0].function.as_ref().unwrap().description,
            "Description for skill 1"
        );
        assert_eq!(
            tools[1].function.as_ref().unwrap().description,
            "Execute the Skill Two skill"
        );
    }

    #[test]
    fn test_prompt_builder_with_system_context() {
        use crate::services::llm::system_context::{LocationContext, SystemContext};

        let context = create_test_context();
        let sys_ctx = SystemContext::new(
            "Asia/Shanghai".to_string(),
            Some(LocationContext {
                city: "Shanghai".to_string(),
                country: "China".to_string(),
                country_code: "CN".to_string(),
                latitude: Some(31.2304),
                longitude: Some(121.4737),
            }),
        );

        let builder = PromptBuilder::new().set_system_context(sys_ctx);
        let prompt = builder.build(&context);

        assert!(prompt.contains("Current date and time"));
        assert!(prompt.contains("User location: Shanghai, China"));
        assert!(prompt.contains("Timezone: Asia/Shanghai"));
    }

    #[test]
    fn test_prompt_builder_without_system_context() {
        let context = create_test_context();
        let builder = PromptBuilder::new().with_system_context(false);
        let prompt = builder.build(&context);

        assert!(!prompt.contains("System Context"));
        assert!(!prompt.contains("Current date and time"));
    }
}
