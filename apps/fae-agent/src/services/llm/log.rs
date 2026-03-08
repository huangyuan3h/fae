use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMLogEntry {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub event_type: LLMEventType,
    pub data: LLMEventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMEventType {
    SessionStart,
    SessionEnd,
    SystemPrompt,
    UserMessage,
    AssistantMessage,
    Thinking,
    ToolCall,
    ToolResult,
    Error,
    LLMRequest,
    LLMResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallData {
    pub tool_call_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultData {
    pub tool_call_id: String,
    pub tool_name: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequestData {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub messages_count: usize,
    pub tools_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponseData {
    pub content: String,
    pub tool_calls: Vec<ToolCallData>,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LLMEventData {
    Text { content: String },
    SystemPrompt { prompt: String },
    ToolCall(ToolCallData),
    ToolResult(ToolResultData),
    LLMRequest(LLMRequestData),
    LLMResponse(LLMResponseData),
    Error { message: String },
    Empty {},
}

pub struct LLMLogger {
    log_file: Arc<Mutex<File>>,
    session_id: String,
    agent_id: String,
    agent_name: String,
}

impl LLMLogger {
    pub fn new(
        log_dir: &str,
        session_id: &str,
        agent_id: &str,
        agent_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let log_path = Self::get_log_path(log_dir, session_id);

        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        info!("[LLMLogger] Created log file: {:?}", log_path);

        Ok(Self {
            log_file: Arc::new(Mutex::new(log_file)),
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            agent_name: agent_name.to_string(),
        })
    }

    fn get_log_path(log_dir: &str, session_id: &str) -> PathBuf {
        PathBuf::from(log_dir).join(format!("{}.jsonl", session_id))
    }

    pub fn log(&self, event_type: LLMEventType, data: LLMEventData) {
        let entry = LLMLogEntry {
            timestamp: Utc::now(),
            session_id: self.session_id.clone(),
            agent_id: self.agent_id.clone(),
            agent_name: self.agent_name.clone(),
            event_type,
            data,
        };

        self.write_entry(&entry);
    }

    fn write_entry(&self, entry: &LLMLogEntry) {
        match serde_json::to_string(entry) {
            Ok(json) => {
                if let Ok(mut file) = self.log_file.lock() {
                    let _ = writeln!(file, "{}", json);
                    let _ = file.flush();
                }
            }
            Err(e) => {
                info!("[LLMLogger] Failed to serialize log entry: {}", e);
            }
        }
    }

    pub fn log_session_start(&self) {
        self.log(LLMEventType::SessionStart, LLMEventData::Empty {});
    }

    pub fn log_session_end(&self) {
        self.log(LLMEventType::SessionEnd, LLMEventData::Empty {});
    }

    pub fn log_system_prompt(&self, prompt: &str) {
        self.log(
            LLMEventType::SystemPrompt,
            LLMEventData::SystemPrompt {
                prompt: prompt.to_string(),
            },
        );
    }

    pub fn log_user_message(&self, content: &str) {
        self.log(
            LLMEventType::UserMessage,
            LLMEventData::Text {
                content: content.to_string(),
            },
        );
    }

    pub fn log_assistant_message(&self, content: &str) {
        self.log(
            LLMEventType::AssistantMessage,
            LLMEventData::Text {
                content: content.to_string(),
            },
        );
    }

    pub fn log_thinking(&self, content: &str) {
        self.log(
            LLMEventType::Thinking,
            LLMEventData::Text {
                content: content.to_string(),
            },
        );
    }

    pub fn log_tool_call(&self, tool_call_id: &str, tool_name: &str, arguments: &str) {
        let args = serde_json::from_str(arguments).unwrap_or(serde_json::json!({"raw": arguments}));
        self.log(
            LLMEventType::ToolCall,
            LLMEventData::ToolCall(ToolCallData {
                tool_call_id: tool_call_id.to_string(),
                tool_name: tool_name.to_string(),
                arguments: args,
            }),
        );
    }

    pub fn log_tool_result(&self, tool_call_id: &str, tool_name: &str, result: &str) {
        self.log(
            LLMEventType::ToolResult,
            LLMEventData::ToolResult(ToolResultData {
                tool_call_id: tool_call_id.to_string(),
                tool_name: tool_name.to_string(),
                result: result.to_string(),
            }),
        );
    }

    pub fn log_llm_request(
        &self,
        provider: &str,
        model: &str,
        base_url: &str,
        messages_count: usize,
        tools_count: Option<usize>,
    ) {
        self.log(
            LLMEventType::LLMRequest,
            LLMEventData::LLMRequest(LLMRequestData {
                provider: provider.to_string(),
                model: model.to_string(),
                base_url: base_url.to_string(),
                messages_count,
                tools_count,
            }),
        );
    }

    pub fn log_llm_response(&self, content: &str, tool_calls: Vec<ToolCallData>, done: bool) {
        self.log(
            LLMEventType::LLMResponse,
            LLMEventData::LLMResponse(LLMResponseData {
                content: content.to_string(),
                tool_calls,
                done,
            }),
        );
    }

    pub fn log_error(&self, message: &str) {
        self.log(
            LLMEventType::Error,
            LLMEventData::Error {
                message: message.to_string(),
            },
        );
    }
}

pub fn generate_session_id() -> String {
    format!("session_{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_llm_logger_creates_file() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let logger = LLMLogger::new(dir_path, "test-session", "agent-1", "TestAgent").unwrap();

        logger.log_session_start();
        logger.log_system_prompt("You are a helpful assistant.");
        logger.log_user_message("Hello!");
        logger.log_assistant_message("Hi there!");
        logger.log_session_end();

        let log_path = PathBuf::from(dir_path).join("test-session.jsonl");
        assert!(log_path.exists());

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 5);

        for line in lines {
            let entry: LLMLogEntry = serde_json::from_str(line).unwrap();
            assert_eq!(entry.session_id, "test-session");
            assert_eq!(entry.agent_id, "agent-1");
        }
    }

    #[test]
    fn test_tool_call_logging() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let logger = LLMLogger::new(dir_path, "test-tool", "agent-1", "TestAgent").unwrap();

        logger.log_tool_call("call_123", "search", r#"{"query": "rust"}"#);
        logger.log_tool_result("call_123", "search", "Found 10 results");

        let log_path = PathBuf::from(dir_path).join("test-tool.jsonl");
        let content = fs::read_to_string(&log_path).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let tool_call_entry: LLMLogEntry = serde_json::from_str(lines[0]).unwrap();
        match tool_call_entry.data {
            LLMEventData::ToolCall(tc) => {
                assert_eq!(tc.tool_name, "search");
                assert_eq!(tc.arguments["query"], "rust");
            }
            _ => panic!("Expected ToolCall data"),
        }

        let tool_result_entry: LLMLogEntry = serde_json::from_str(lines[1]).unwrap();
        match tool_result_entry.data {
            LLMEventData::ToolResult(tr) => {
                assert_eq!(tr.tool_name, "search");
                assert_eq!(tr.result, "Found 10 results");
            }
            _ => panic!("Expected ToolResult data"),
        }
    }
}
