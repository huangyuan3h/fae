use std::collections::HashMap;

use super::models::{ToolDefinition, ToolFunction};

pub fn skills_to_tools(
    skills: &[String],
    skill_definitions: &HashMap<String, String>,
) -> Vec<ToolDefinition> {
    skills
        .iter()
        .filter_map(|skill_name| {
            let description = skill_definitions
                .get(skill_name)
                .cloned()
                .unwrap_or_else(|| format!("Execute the {} skill", skill_name));

            Some(ToolDefinition {
                tool_type: "function".to_string(),
                function: Some(ToolFunction {
                    name: skill_name.clone(),
                    description,
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "input": {
                                "type": "string",
                                "description": "Input for the skill"
                            }
                        },
                        "required": ["input"]
                    }),
                }),
            })
        })
        .collect()
}

pub fn build_system_prompt(skills: &[String]) -> String {
    if skills.is_empty() {
        return "You are a helpful assistant.".to_string();
    }

    format!(
        "You are a helpful assistant with access to tools. You should use tools when they would help answer the user's question. \
        Available tools: {}. \
        Think step by step about whether you need to use any tools to answer the user's question. \
        If you use a tool, explain what you're doing and why.",
        skills.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_to_tools_empty() {
        let skills = vec![];
        let definitions = HashMap::new();
        let tools = skills_to_tools(&skills, &definitions);
        assert!(tools.is_empty());
    }

    #[test]
    fn test_skills_to_tools_single() {
        let skills = vec!["search".to_string()];
        let mut definitions = HashMap::new();
        definitions.insert("search".to_string(), "Search the web".to_string());

        let tools = skills_to_tools(&skills, &definitions);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.as_ref().unwrap().name, "search");
        assert_eq!(
            tools[0].function.as_ref().unwrap().description,
            "Search the web"
        );
    }

    #[test]
    fn test_skills_to_tools_multiple() {
        let skills = vec!["search".to_string(), "calculate".to_string()];
        let mut definitions = HashMap::new();
        definitions.insert("search".to_string(), "Search the web".to_string());
        definitions.insert("calculate".to_string(), "Perform calculations".to_string());

        let tools = skills_to_tools(&skills, &definitions);
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_skills_to_tools_missing_definition() {
        let skills = vec!["unknown".to_string()];
        let definitions = HashMap::new();

        let tools = skills_to_tools(&skills, &definitions);
        assert_eq!(tools.len(), 1);
        assert_eq!(
            tools[0].function.as_ref().unwrap().description,
            "Execute the unknown skill"
        );
    }

    #[test]
    fn test_build_system_prompt_empty() {
        let skills = vec![];
        let prompt = build_system_prompt(&skills);
        assert_eq!(prompt, "You are a helpful assistant.");
    }

    #[test]
    fn test_build_system_prompt_with_skills() {
        let skills = vec!["search".to_string(), "calculate".to_string()];
        let prompt = build_system_prompt(&skills);
        assert!(prompt.contains("Available tools: search, calculate"));
        assert!(prompt.contains("You are a helpful assistant with access to tools"));
    }

    #[test]
    fn test_tool_parameters_structure() {
        let skills = vec!["test".to_string()];
        let definitions = HashMap::new();
        let tools = skills_to_tools(&skills, &definitions);

        let params = &tools[0].function.as_ref().unwrap().parameters;
        assert_eq!(params["type"], "object");
        assert!(params["properties"]["input"]["type"].is_string());
        assert!(params["required"].is_array());
    }
}
