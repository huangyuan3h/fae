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

pub struct LLMClient {
    base_url: String,
    provider_type: String,
    api_key: String,
    http_client: reqwest::Client,
}

impl LLMClient {
    pub fn new(base_url: String, provider_type: String, api_key: String) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("OpenClaw-Gateway/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            base_url,
            provider_type,
            api_key,
            http_client,
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
        if self.provider_type == "openai" {
            self.chat_stream_openai(model, messages, tools).await
        } else {
            self.chat_stream_ollama(model, messages, tools).await
        }
    }

    async fn chat_stream_openai(
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
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<ChatMessage>,
            #[serde(skip_serializing_if = "Option::is_none")]
            tools: Option<Vec<ToolDefinition>>,
            stream: bool,
        }

        let model = model.to_string();
        let request = OpenAIRequest {
            model: model.clone(),
            messages,
            tools,
            stream: true,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        tracing::info!("Sending request to OpenAI API: {} with model {}", url, model);
        
        let mut req_builder = self.http_client.post(&url).json(&request);
        
        if !self.api_key.is_empty() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("OpenAI API error: {} - {}", status, body).into());
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut done_tx = Some(done_tx);
            let mut tool_call_buffers: std::collections::HashMap<i32, (String, String, String)> = std::collections::HashMap::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line_str = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                            
                            if line_str.starts_with("data: ") {
                                let data = &line_str[6..];
                                if data == "[DONE]" {
                                    if let Some(dtx) = done_tx.take() {
                                        let _ = dtx.send(());
                                    }
                                    break;
                                }
                                
                                match serde_json::from_str::<OpenAIStreamResponse>(data) {
                                    Ok(openai_resp) => {
                                        for choice in openai_resp.choices {
                                            let content = choice.delta.content.clone().unwrap_or_default();
                                            
                                            let tool_calls: Option<Vec<ToolCall>> = choice.delta.tool_calls.as_ref().map(|tcs| {
                                                tcs.iter().filter_map(|tc| {
                                                    let index = tc.index;
                                                    let entry = tool_call_buffers.entry(index).or_insert((
                                                        tc.id.clone().unwrap_or_default(),
                                                        String::new(),
                                                        String::new(),
                                                    ));
                                                    
                                                    if let Some(ref id) = tc.id {
                                                        entry.0 = id.clone();
                                                    }
                                                    if let Some(ref func) = tc.function {
                                                        if let Some(ref name) = func.name {
                                                            entry.1 = name.clone();
                                                        }
                                                        if let Some(ref args) = func.arguments {
                                                            entry.2.push_str(args);
                                                        }
                                                    }
                                                    
                                                    if !entry.2.is_empty() {
                                                        Some(ToolCall {
                                                            id: entry.0.clone(),
                                                            tool_type: "function".to_string(),
                                                            function: FunctionCall {
                                                                name: entry.1.clone(),
                                                                arguments: entry.2.clone(),
                                                            },
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                }).collect()
                                            });

                                            let ollama_resp = OllamaStreamResponse {
                                                model: model.to_string(),
                                                created_at: chrono::Utc::now().to_rfc3339(),
                                                message: Some(ChatMessage {
                                                    role: "assistant".to_string(),
                                                    content,
                                                    tool_calls,
                                                }),
                                                done: choice.finish_reason.is_some(),
                                                total_duration: None,
                                                eval_count: None,
                                            };

                                            if tx.send(ollama_resp).await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse OpenAI response: {} - line: {}", e, data);
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

    async fn chat_stream_ollama(
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