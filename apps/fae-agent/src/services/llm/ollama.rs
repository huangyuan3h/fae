use super::models::*;
use tokio::sync::mpsc;
use futures_util::StreamExt;

pub struct OllamaClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent("OpenClaw-Gateway/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            base_url,
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

        let (tx, rx) = mpsc::channel(100);
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_client_default_url() {
        let client = OllamaClient::new("http://127.0.0.1:11434".to_string());
        assert!(client.base_url.contains("11434"));
    }

    #[test]
    fn test_ollama_request_for_stream() {
        let request = OllamaRequest {
            model: "llama2".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "test".to_string(),
                tool_calls: None,
            }],
            tools: None,
            stream: true,
        };
        
        assert!(request.stream);
        assert_eq!(request.model, "llama2");
    }

    #[test]
    fn test_ollama_request_for_non_stream() {
        let request = OllamaRequest {
            model: "llama2".to_string(),
            messages: vec![],
            tools: None,
            stream: false,
        };
        
        assert!(!request.stream);
    }
}