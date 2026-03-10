use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<ToolFunction>,
}

impl ToolDefinition {
    pub fn function_tool(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: Some(ToolFunction {
                name,
                description,
                parameters,
            }),
        }
    }

    pub fn builtin_tool(tool_type: &str) -> Self {
        Self {
            tool_type: tool_type.to_string(),
            function: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    pub stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaStreamResponse {
    pub model: String,
    pub created_at: String,
    pub message: Option<ChatMessage>,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIStreamChoice {
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIToolCall {
    pub index: i32,
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<OpenAIFunctionCall>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIStreamResponse {
    pub choices: Vec<OpenAIStreamChoice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_chat_message_deserialization() {
        let json = r#"{"role":"assistant","content":"Hi there"}"#;
        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "Hi there");
        assert_eq!(msg.tool_calls, None);
    }

    #[test]
    fn test_chat_message_with_tool_calls() {
        let msg = ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
            tool_calls: Some(vec![ToolCall {
                id: "call_123".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "search".to_string(),
                    arguments: r#"{"query":"test"}"#.to_string(),
                },
            }]),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_calls.unwrap().len(), 1);
    }

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition {
            tool_type: "function".to_string(),
            function: Some(ToolFunction {
                name: "get_weather".to_string(),
                description: "Get weather info".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }),
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"type\":\"function\""));
        assert!(json.contains("\"name\":\"get_weather\""));
    }

    #[test]
    fn test_builtin_tool_definition() {
        let tool = ToolDefinition::builtin_tool("web_search_preview");

        let json = serde_json::to_string(&tool).unwrap();
        assert_eq!(json, r#"{"type":"web_search_preview"}"#);
        assert!(!json.contains("function"));
    }

    #[test]
    fn test_function_tool_helper() {
        let tool = ToolDefinition::function_tool(
            "search".to_string(),
            "Search the web".to_string(),
            serde_json::json!({"type": "object"}),
        );

        assert_eq!(tool.tool_type, "function");
        assert!(tool.function.is_some());
        let func = tool.function.as_ref().unwrap();
        assert_eq!(func.name, "search");
        assert_eq!(func.description, "Search the web");
    }

    #[test]
    fn test_ollama_request() {
        let request = OllamaRequest {
            model: "llama2".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                tool_calls: None,
            }],
            tools: None,
            stream: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"llama2\""));
        assert!(json.contains("\"stream\":true"));
    }

    #[test]
    fn test_ollama_stream_response() {
        let json = r#"{
            "model": "llama2",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello back"
            },
            "done": false
        }"#;

        let response: OllamaStreamResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.model, "llama2");
        assert!(!response.done);
        assert_eq!(response.message.unwrap().content, "Hello back");
    }

    #[test]
    fn test_openai_stream_response() {
        let json = r#"{
            "choices": [{
                "delta": {
                    "content": "Hello",
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_123",
                        "function": {
                            "name": "search",
                            "arguments": "{\"q\":\"test\""
                        }
                    }]
                },
                "finish_reason": null
            }]
        }"#;

        let response: OpenAIStreamResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_chat_message_default_content() {
        let json = r#"{"role":"user"}"#;
        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.content, "");
    }

    #[test]
    fn test_ollama_stream_response_optional_fields() {
        let json = r#"{
            "model": "llama2",
            "created_at": "2024-01-01T00:00:00Z",
            "done": true,
            "total_duration": 1000,
            "eval_count": 50
        }"#;

        let response: OllamaStreamResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_duration, Some(1000));
        assert_eq!(response.eval_count, Some(50));
    }
}
