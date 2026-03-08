use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct LLMClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl LLMClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn chat_stream(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<
        (
            tokio::sync::mpsc::Receiver<OllamaStreamResponse>,
            tokio::sync::oneshot::Receiver<()>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = OllamaRequest {
            model: model.to_string(),
            messages,
            tools,
            stream: true,
        };

        let url = format!("{}/api/chat", self.base_url);
        
        tracing::info!("Sending request to Ollama: {} with model {}", url, model);
        
        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("Ollama API error: {} - {}", status, body).into());
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut done_tx = Some(done_tx);

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line_str = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            if !line_str.is_empty() {
                                match serde_json::from_str::<OllamaStreamResponse>(&line_str) {
                                    Ok(response) => {
                                        tracing::debug!("Ollama response: done={}, has_message={}", 
                                            response.done, response.message.is_some());
                                        
                                        if tx.send(response.clone()).await.is_err() {
                                            break;
                                        }
                                        
                                        if response.done {
                                            if let Some(dtx) = done_tx.take() {
                                                let _ = dtx.send(());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse Ollama response: {} - line: {}", e, line_str);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Stream error: {}", e);
                        break;
                    }
                }
            }

            if let Some(dtx) = done_tx.take() {
                let _ = dtx.send(());
            }
        });

        Ok((rx, done_rx))
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<OllamaChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = OllamaRequest {
            model: model.to_string(),
            messages,
            tools,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);
        
        tracing::info!("Sending non-streaming request to Ollama: {}", url);

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("Ollama API error: {} - {}", status, body).into());
        }

        let chat_response = response.json::<OllamaChatResponse>().await?;
        Ok(chat_response)
    }
}

pub fn skills_to_tools(skills: &[String], skill_definitions: &HashMap<String, String>) -> Vec<ToolDefinition> {
    skills
        .iter()
        .filter_map(|skill_name| {
            let description = skill_definitions
                .get(skill_name)
                .cloned()
                .unwrap_or_else(|| format!("Execute the {} skill", skill_name));
            
            Some(ToolDefinition {
                tool_type: "function".to_string(),
                function: ToolFunction {
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
                },
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