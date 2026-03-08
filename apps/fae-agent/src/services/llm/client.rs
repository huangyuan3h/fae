use super::models::*;
use super::openai::OpenAIClient;
use super::ollama::OllamaClient;

pub struct LLMClient {
    provider_type: String,
    openai_client: Option<OpenAIClient>,
    ollama_client: Option<OllamaClient>,
}

impl LLMClient {
    pub fn new(base_url: String, provider_type: String, api_key: String) -> Self {
        let (openai_client, ollama_client) = if provider_type == "openai" {
            (
                Some(OpenAIClient::new(base_url, api_key)),
                None,
            )
        } else {
            (
                None,
                Some(OllamaClient::new(base_url)),
            )
        };

        Self {
            provider_type,
            openai_client,
            ollama_client,
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
            if let Some(ref client) = self.openai_client {
                client.chat_stream(model, messages, tools).await
            } else {
                Err("OpenAI client not initialized".into())
            }
        } else {
            if let Some(ref client) = self.ollama_client {
                client.chat_stream(model, messages, tools).await
            } else {
                Err("Ollama client not initialized".into())
            }
        }
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<OllamaChatResponse, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref client) = self.ollama_client {
            client.chat(model, messages, tools).await
        } else {
            Err("Ollama client not initialized for non-streaming chat".into())
        }
    }

    pub fn provider_type(&self) -> &str {
        &self.provider_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_openai_creation() {
        let client = LLMClient::new(
            "https://api.openai.com/v1".to_string(),
            "openai".to_string(),
            "test-key".to_string(),
        );
        
        assert_eq!(client.provider_type(), "openai");
        assert!(client.openai_client.is_some());
        assert!(client.ollama_client.is_none());
    }

    #[test]
    fn test_llm_client_ollama_creation() {
        let client = LLMClient::new(
            "http://localhost:11434".to_string(),
            "ollama".to_string(),
            String::new(),
        );
        
        assert_eq!(client.provider_type(), "ollama");
        assert!(client.openai_client.is_none());
        assert!(client.ollama_client.is_some());
    }

    #[test]
    fn test_llm_client_other_provider() {
        let client = LLMClient::new(
            "http://localhost:11434".to_string(),
            "other".to_string(),
            String::new(),
        );
        
        assert_eq!(client.provider_type(), "other");
        assert!(client.openai_client.is_none());
        assert!(client.ollama_client.is_some());
    }
}