use serde::Serialize;
use super::models::*;
use tokio::sync::mpsc;
use futures_util::StreamExt;

pub struct OpenAIClient {
    base_url: String,
    api_key: String,
    http_client: reqwest::Client,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    stream: bool,
}

impl OpenAIClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("OpenClaw-Gateway/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            base_url,
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
            mpsc::Receiver<OllamaStreamResponse>,
            tokio::sync::oneshot::Receiver<()>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
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

        let (tx, rx) = mpsc::channel(100);
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
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
                                            
                                            // Accumulate tool call data
                                            if let Some(ref tcs) = choice.delta.tool_calls {
                                                for tc in tcs {
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
                                                }
                                            }

                                            // Only emit tool calls when finish_reason is present
                                            let tool_calls = if choice.finish_reason.is_some() && !tool_call_buffers.is_empty() {
                                                Some(tool_call_buffers.values().map(|(id, name, args)| {
                                                    ToolCall {
                                                        id: id.clone(),
                                                        tool_type: "function".to_string(),
                                                        function: FunctionCall {
                                                            name: name.clone(),
                                                            arguments: args.clone(),
                                                        },
                                                    }
                                                }).collect())
                                            } else {
                                                None
                                            };

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new(
            "https://api.openai.com/v1".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(client.base_url, "https://api.openai.com/v1");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_openai_client_empty_key() {
        let client = OpenAIClient::new(
            "https://api.openai.com/v1".to_string(),
            String::new(),
        );
        assert!(client.api_key.is_empty());
    }

    #[test]
    fn test_openai_request_serialization() {
        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                tool_calls: None,
            }],
            tools: None,
            stream: true,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"stream\":true"));
    }

    #[test]
    fn test_openai_request_with_tools() {
        let tools = vec![ToolDefinition {
            tool_type: "function".to_string(),
            function: Some(ToolFunction {
                name: "search".to_string(),
                description: "Search".to_string(),
                parameters: serde_json::json!({"type": "object"}),
            }),
        }];
        
        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            tools: Some(tools),
            stream: false,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"tools\":"));
    }
}