use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::models::{ToolDefinition, ToolFunction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    async fn execute(&self, arguments: serde_json::Value) -> ToolResult;
    
    fn to_tool_definition(&self) -> ToolDefinition {
        ToolDefinition {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: self.name().to_string(),
                description: self.description().to_string(),
                parameters: self.parameters(),
            },
        }
    }
}

pub struct BashTool {
    allowed_commands: Vec<String>,
    working_directory: String,
}

impl BashTool {
    pub fn new(working_directory: String) -> Self {
        Self {
            allowed_commands: vec![
                "ls".to_string(),
                "cat".to_string(),
                "pwd".to_string(),
                "echo".to_string(),
                "mkdir".to_string(),
                "touch".to_string(),
                "rm".to_string(),
                "cp".to_string(),
                "mv".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
            ],
            working_directory,
        }
    }

    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }

    fn is_command_allowed(&self, command: &str) -> bool {
        let command_name = command.split_whitespace().next().unwrap_or("");
        self.allowed_commands.contains(&command_name.to_string())
    }

    #[allow(dead_code)]
    fn validate_path(&self, path: &str) -> Result<String, String> {
        let resolved_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            Path::new(&self.working_directory).join(path)
        };

        let canonical_path = resolved_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        if !canonical_path.starts_with(&self.working_directory) {
            return Err("Path traversal not allowed".to_string());
        }

        Ok(canonical_path.to_string_lossy().to_string())
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute bash commands for file operations. Supported commands: ls, cat, pwd, echo, mkdir, touch, rm, cp, mv, grep, find, head, tail, wc. All operations are restricted to the working directory."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "args": {
                    "type": "string",
                    "description": "Arguments for the command"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolResult {
        let command = arguments["command"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let args = arguments["args"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if !self.is_command_allowed(&command) {
            return ToolResult::error(format!(
                "Command '{}' is not allowed. Allowed commands: {:?}",
                command, self.allowed_commands
            ));
        }

        let full_command = if args.is_empty() {
            command.clone()
        } else {
            format!("{} {}", command, args)
        };

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .current_dir(&self.working_directory)
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    ToolResult::success(stdout)
                } else {
                    ToolResult::error(format!("Command failed: {}", stderr))
                }
            }
            Err(e) => ToolResult::error(format!("Failed to execute command: {}", e)),
        }
    }
}

pub struct ReadFileTool {
    working_directory: String,
}

impl ReadFileTool {
    pub fn new(working_directory: String) -> Self {
        let canonical_working_dir = std::fs::canonicalize(&working_directory)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(working_directory);
        Self { working_directory: canonical_working_dir }
    }

    fn validate_path(&self, path: &str) -> Result<std::path::PathBuf, String> {
        let resolved_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            Path::new(&self.working_directory).join(path)
        };

        let canonical_path = resolved_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        if !canonical_path.starts_with(&self.working_directory) {
            return Err("Path traversal not allowed".to_string());
        }

        Ok(canonical_path)
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. The file path must be within the working directory."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolResult {
        let path = match arguments["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::error("Missing 'path' parameter".to_string()),
        };

        let validated_path = match self.validate_path(path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(e),
        };

        match tokio::fs::read_to_string(&validated_path).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
        }
    }
}

pub struct WriteFileTool {
    working_directory: String,
}

impl WriteFileTool {
    pub fn new(working_directory: String) -> Self {
        let canonical_working_dir = if std::path::Path::new(&working_directory).exists() {
            std::fs::canonicalize(&working_directory)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(working_directory)
        } else {
            working_directory
        };
        Self { working_directory: canonical_working_dir }
    }

    fn validate_path(&self, path: &str) -> Result<std::path::PathBuf, String> {
        let resolved_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            Path::new(&self.working_directory).join(path)
        };

        if let Ok(canonical_parent) = resolved_path.parent().unwrap_or(Path::new("")).canonicalize() {
            if !canonical_parent.starts_with(&self.working_directory) {
                return Err("Path traversal not allowed".to_string());
            }
        }

        Ok(resolved_path)
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. The file path must be within the working directory. Creates parent directories if they don't exist."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolResult {
        let path = match arguments["path"].as_str() {
            Some(p) => p,
            None => return ToolResult::error("Missing 'path' parameter".to_string()),
        };

        let content = match arguments["content"].as_str() {
            Some(c) => c,
            None => return ToolResult::error("Missing 'content' parameter".to_string()),
        };

        let validated_path = match self.validate_path(path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(e),
        };

        if let Some(parent) = validated_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return ToolResult::error(format!("Failed to create parent directories: {}", e));
            }
        }

        match tokio::fs::write(&validated_path, content).await {
            Ok(_) => ToolResult::success(format!("Successfully wrote to {}", path)),
            Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
        }
    }
}

pub struct ListDirectoryTool {
    working_directory: String,
}

impl ListDirectoryTool {
    pub fn new(working_directory: String) -> Self {
        let canonical_working_dir = std::fs::canonicalize(&working_directory)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(working_directory);
        Self { working_directory: canonical_working_dir }
    }

    fn validate_path(&self, path: &str) -> Result<std::path::PathBuf, String> {
        let resolved_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            Path::new(&self.working_directory).join(path)
        };

        let canonical_path = resolved_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        if !canonical_path.starts_with(&self.working_directory) {
            return Err("Path traversal not allowed".to_string());
        }

        Ok(canonical_path)
    }
}

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the contents of a directory. Returns a list of files and directories."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the directory to list (defaults to working directory)"
                }
            },
            "required": []
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolResult {
        let path = arguments["path"]
            .as_str()
            .unwrap_or(&self.working_directory);

        let validated_path = match self.validate_path(path) {
            Ok(p) => p,
            Err(e) => return ToolResult::error(e),
        };

        let mut entries = match tokio::fs::read_dir(&validated_path).await {
            Ok(entries) => entries,
            Err(e) => return ToolResult::error(format!("Failed to read directory: {}", e)),
        };

        let mut result = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry.file_type().await.unwrap();
            let type_str = if file_type.is_dir() { "DIR" } else { "FILE" };
            result.push(format!("{}: {}", type_str, name));
        }

        ToolResult::success(result.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("output".to_string());
        assert!(result.success);
        assert_eq!(result.output, "output");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("error message".to_string());
        assert!(!result.success);
        assert!(result.output.is_empty());
        assert_eq!(result.error, Some("error message".to_string()));
    }

    #[test]
    fn test_bash_tool_creation() {
        let tool = BashTool::new("/tmp".to_string());
        assert_eq!(tool.name(), "bash");
        assert!(tool.description().contains("Execute bash commands"));
    }

    #[test]
    fn test_bash_tool_is_command_allowed() {
        let tool = BashTool::new("/tmp".to_string());
        assert!(tool.is_command_allowed("ls"));
        assert!(tool.is_command_allowed("cat file.txt"));
        assert!(tool.is_command_allowed("rm"));
        assert!(!tool.is_command_allowed("sudo ls"));
        assert!(!tool.is_command_allowed("chmod"));
    }

    #[tokio::test]
    async fn test_bash_tool_execute_allowed_command() {
        let temp_dir = TempDir::new().unwrap();
        let tool = BashTool::new(temp_dir.path().to_string_lossy().to_string());

        let args = serde_json::json!({
            "command": "echo",
            "args": "hello world"
        });

        let result = tool.execute(args).await;
        assert!(result.success);
        assert!(result.output.contains("hello world"));
    }

    #[tokio::test]
    async fn test_bash_tool_execute_denied_command() {
        let tool = BashTool::new("/tmp".to_string());

        let args = serde_json::json!({
            "command": "sudo",
            "args": "ls"
        });

        let result = tool.execute(args).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not allowed"));
    }

    #[test]
    fn test_read_file_tool_creation() {
        let tool = ReadFileTool::new("/tmp".to_string());
        assert_eq!(tool.name(), "read_file");
        assert!(tool.description().contains("Read the contents"));
    }

    #[tokio::test]
    async fn test_read_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "test content").await.unwrap();

        let tool = ReadFileTool::new(temp_dir.path().to_string_lossy().to_string());

        let args = serde_json::json!({
            "path": "test.txt"
        });

        let result = tool.execute(args).await;
        if !result.success {
            eprintln!("Test failed. Error: {:?}", result.error);
        }
        assert!(result.success, "Tool execution failed: {:?}", result.error);
        assert_eq!(result.output, "test content");
    }

    #[tokio::test]
    async fn test_read_file_tool_nonexistent() {
        let tool = ReadFileTool::new("/tmp".to_string());

        let args = serde_json::json!({
            "path": "nonexistent.txt"
        });

        let result = tool.execute(args).await;
        assert!(!result.success);
    }

    #[test]
    fn test_write_file_tool_creation() {
        let tool = WriteFileTool::new("/tmp".to_string());
        assert_eq!(tool.name(), "write_file");
        assert!(tool.description().contains("Write content to a file"));
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let tool = WriteFileTool::new(temp_dir.path().to_string_lossy().to_string());

        let args = serde_json::json!({
            "path": "test.txt",
            "content": "test content"
        });

        let result = tool.execute(args).await;
        assert!(result.success);

        let content = tokio::fs::read_to_string(temp_dir.path().join("test.txt"))
            .await
            .unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_write_file_tool_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let tool = WriteFileTool::new(temp_dir.path().to_string_lossy().to_string());

        let args = serde_json::json!({
            "path": "subdir/test.txt",
            "content": "test content"
        });

        let result = tool.execute(args).await;
        assert!(result.success);

        let content = tokio::fs::read_to_string(temp_dir.path().join("subdir/test.txt"))
            .await
            .unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_list_directory_tool_creation() {
        let tool = ListDirectoryTool::new("/tmp".to_string());
        assert_eq!(tool.name(), "list_directory");
        assert!(tool.description().contains("List the contents"));
    }

    #[tokio::test]
    async fn test_list_directory_tool() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("file1.txt"), "content1")
            .await
            .unwrap();
        tokio::fs::write(temp_dir.path().join("file2.txt"), "content2")
            .await
            .unwrap();
        tokio::fs::create_dir(temp_dir.path().join("subdir"))
            .await
            .unwrap();

        let tool = ListDirectoryTool::new(temp_dir.path().to_string_lossy().to_string());

        let args = serde_json::json!({});

        let result = tool.execute(args).await;
        assert!(result.success);
        assert!(result.output.contains("FILE: file1.txt"));
        assert!(result.output.contains("FILE: file2.txt"));
        assert!(result.output.contains("DIR: subdir"));
    }

    #[test]
    fn test_tool_to_tool_definition() {
        let tool = BashTool::new("/tmp".to_string());
        let definition = tool.to_tool_definition();

        assert_eq!(definition.tool_type, "function");
        assert_eq!(definition.function.name, "bash");
        assert!(definition.function.description.contains("Execute bash commands"));
    }

    #[test]
    fn test_skill_tool_creation() {
        let tool = SkillTool::new("weather".to_string(), "Get weather information".to_string(), "/skills/weather".to_string());
        assert_eq!(tool.name(), "weather");
        assert!(tool.description().contains("weather"));
    }
}

pub struct SkillTool {
    skill_name: String,
    skill_description: String,
    skill_path: String,
}

impl SkillTool {
    pub fn new(skill_name: String, skill_description: String, skill_path: String) -> Self {
        Self {
            skill_name,
            skill_description,
            skill_path,
        }
    }
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        &self.skill_name
    }

    fn description(&self) -> &str {
        &self.skill_description
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": format!("Input for the {} skill", self.skill_name)
                }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> ToolResult {
        let input = arguments["input"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let script_path = Path::new(&self.skill_path);
        let script_file = if script_path.is_dir() {
            script_path.join(format!("{}.js", self.skill_name))
        } else {
            script_path.to_path_buf()
        };

        if !script_file.exists() {
            return ToolResult::error(format!(
                "Skill script not found: {}",
                script_file.display()
            ));
        }

        let output = tokio::process::Command::new("node")
            .arg(&script_file)
            .arg(&input)
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    ToolResult::success(stdout)
                } else {
                    ToolResult::error(format!("Skill execution failed: {}", stderr))
                }
            }
            Err(e) => ToolResult::error(format!("Failed to execute skill: {}", e)),
        }
    }
}