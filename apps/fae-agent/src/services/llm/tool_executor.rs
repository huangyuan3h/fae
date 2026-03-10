use std::collections::HashMap;
use std::sync::Arc;
use sqlx::SqlitePool;

use super::models::{ToolCall, ToolDefinition};
use super::tools::{Tool, ToolResult, BashTool, ReadFileTool, WriteFileTool, ListDirectoryTool, SkillTool};
use super::folder_validator::FolderValidator;

pub struct ToolExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
    working_directory: String,
    folder_validator: Option<FolderValidator>,
}

impl ToolExecutor {
    pub fn new(working_directory: String) -> Self {
        let mut executor = Self {
            tools: HashMap::new(),
            working_directory,
            folder_validator: None,
        };

        executor.register_default_tools();
        executor
    }

    pub async fn with_folder_validation(db_pool: &SqlitePool) -> Self {
        let validator = FolderValidator::new(db_pool).await;
        let working_directory = validator.get_working_directory();
        
        let mut executor = Self {
            tools: HashMap::new(),
            working_directory: working_directory.clone(),
            folder_validator: Some(validator),
        };

        executor.register_default_tools();
        executor
    }

    fn register_default_tools(&mut self) {
        self.register_tool(Arc::new(BashTool::new(self.working_directory.clone())));
        self.register_tool(Arc::new(ReadFileTool::new(self.working_directory.clone())));
        self.register_tool(Arc::new(WriteFileTool::new(self.working_directory.clone())));
        self.register_tool(Arc::new(ListDirectoryTool::new(self.working_directory.clone())));
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn register_skill(&mut self, skill_name: String, skill_description: String, skill_path: String) {
        let skill_tool = Arc::new(SkillTool::new(skill_name.clone(), skill_description, skill_path));
        self.tools.insert(skill_name, skill_tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.to_tool_definition())
            .collect()
    }

    fn validate_path_for_tool(&self, tool_name: &str, arguments: &mut serde_json::Value) -> Result<(), String> {
        if let Some(validator) = &self.folder_validator {
            if let Some(path) = arguments.get("path").and_then(|p| p.as_str()) {
                let validated_path = validator.validate_path(path)?;
                if let Some(args) = arguments.as_object_mut() {
                    args.insert("path".to_string(), serde_json::json!(validated_path));
                }
            }
            
            if tool_name == "bash" {
                if let Some(args) = arguments.get("args").and_then(|a| a.as_str()) {
                    let validated_args = self.validate_bash_args(args, validator)?;
                    if let Some(obj) = arguments.as_object_mut() {
                        obj.insert("args".to_string(), serde_json::json!(validated_args));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_bash_args(&self, args: &str, validator: &FolderValidator) -> Result<String, String> {
        let path_patterns = [
            ("cd ", 3),
            ("ls ", 3),
            ("cat ", 4),
            ("mkdir ", 6),
            ("touch ", 6),
            ("rm ", 3),
            ("cp ", 3),
            ("mv ", 3),
            ("grep ", 5),
            ("find ", 5),
            ("head ", 5),
            ("tail ", 5),
        ];

        let mut validated_args = args.to_string();

        for (cmd, len) in path_patterns.iter() {
            if args.starts_with(cmd) {
                let path_part = &args[*len..];
                if !path_part.trim().is_empty() {
                    let validated_path = validator.validate_path(path_part.trim())?;
                    validated_args = format!("{}{}", cmd, validated_path);
                }
                break;
            }
        }

        Ok(validated_args)
    }

    pub async fn execute_tool(&self, tool_name: &str, mut arguments: serde_json::Value) -> ToolResult {
        if let Err(e) = self.validate_path_for_tool(tool_name, &mut arguments) {
            return ToolResult::error(e);
        }

        match self.tools.get(tool_name) {
            Some(tool) => tool.execute(arguments).await,
            None => ToolResult::error(format!("Unknown tool: {}", tool_name)),
        }
    }

    pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> ToolResult {
        let arguments: serde_json::Value = match serde_json::from_str(&tool_call.function.arguments) {
            Ok(args) => args,
            Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
        };

        self.execute_tool(&tool_call.function.name, arguments).await
    }

    pub async fn execute_tool_calls(&self, tool_calls: &[ToolCall]) -> Vec<(String, ToolResult)> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            let result = self.execute_tool_call(tool_call).await;
            results.push((tool_call.id.clone(), result));
        }

        results
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new(std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_executor() -> (ToolExecutor, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let executor = ToolExecutor::new(temp_dir.path().to_string_lossy().to_string());
        (executor, temp_dir)
    }

    #[test]
    fn test_tool_executor_creation() {
        let (executor, _temp_dir) = create_test_executor();
        let tool_names = executor.list_tools();
        
        assert!(tool_names.contains(&"bash"));
        assert!(tool_names.contains(&"read_file"));
        assert!(tool_names.contains(&"write_file"));
        assert!(tool_names.contains(&"list_directory"));
    }

    #[test]
    fn test_get_tool() {
        let (executor, _temp_dir) = create_test_executor();
        
        assert!(executor.get_tool("bash").is_some());
        assert!(executor.get_tool("read_file").is_some());
        assert!(executor.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_get_tool_definitions() {
        let (executor, _temp_dir) = create_test_executor();
        let definitions = executor.get_tool_definitions();
        
        assert_eq!(definitions.len(), 4);
        
        let bash_def = definitions.iter().find(|d| d.function.name == "bash");
        assert!(bash_def.is_some());
        assert_eq!(bash_def.unwrap().tool_type, "function");
    }

    #[tokio::test]
    async fn test_execute_tool() {
        let (executor, temp_dir) = create_test_executor();
        
        let result = executor.execute_tool(
            "bash",
            serde_json::json!({
                "command": "echo",
                "args": "hello"
            })
        ).await;
        
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let (executor, _temp_dir) = create_test_executor();
        
        let result = executor.execute_tool(
            "unknown_tool",
            serde_json::json!({})
        ).await;
        
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_execute_tool_call() {
        let (executor, temp_dir) = create_test_executor();
        
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: super::super::models::FunctionCall {
                name: "bash".to_string(),
                arguments: r#"{"command":"echo","args":"test"}"#.to_string(),
            },
        };
        
        let result = executor.execute_tool_call(&tool_call).await;
        
        assert!(result.success);
        assert!(result.output.contains("test"));
    }

    #[tokio::test]
    async fn test_execute_tool_call_invalid_args() {
        let (executor, _temp_dir) = create_test_executor();
        
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: super::super::models::FunctionCall {
                name: "bash".to_string(),
                arguments: "invalid json".to_string(),
            },
        };
        
        let result = executor.execute_tool_call(&tool_call).await;
        
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn test_execute_multiple_tool_calls() {
        let (executor, temp_dir) = create_test_executor();
        
        tokio::fs::write(temp_dir.path().join("test.txt"), "content")
            .await
            .unwrap();
        
        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                tool_type: "function".to_string(),
                function: super::super::models::FunctionCall {
                    name: "list_directory".to_string(),
                    arguments: "{}".to_string(),
                },
            },
            ToolCall {
                id: "call_2".to_string(),
                tool_type: "function".to_string(),
                function: super::super::models::FunctionCall {
                    name: "read_file".to_string(),
                    arguments: r#"{"path":"test.txt"}"#.to_string(),
                },
            },
        ];
        
        let results = executor.execute_tool_calls(&tool_calls).await;
        
        assert_eq!(results.len(), 2);
        
        let (id1, result1) = &results[0];
        assert_eq!(id1, "call_1");
        assert!(result1.success);
        assert!(result1.output.contains("test.txt"));
        
        let (id2, result2) = &results[1];
        assert_eq!(id2, "call_2");
        assert!(result2.success);
        assert_eq!(result2.output, "content");
    }

    #[test]
    fn test_tool_executor_default() {
        let executor = ToolExecutor::default();
        assert!(!executor.list_tools().is_empty());
    }
}