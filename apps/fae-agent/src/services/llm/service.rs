use super::context::AgentContext;
use super::memory::ConversationMemory;
use super::models::*;
use super::prompt_builder::PromptBuilder;
use super::client::LLMClient;
use super::tool_executor::ToolExecutor;

pub struct LLMService {
    client: LLMClient,
    context: AgentContext,
    memory: ConversationMemory,
    prompt_builder: PromptBuilder,
    tool_executor: ToolExecutor,
}

impl LLMService {
    pub fn new(context: AgentContext) -> Self {
        let provider_type = context.provider.clone();
        let _model = context.model.clone();
        
        let (base_url, api_key) = {
            ("http://127.0.0.1:11434".to_string(), String::new())
        };

        let client = LLMClient::new(base_url, provider_type, api_key);
        let working_directory = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();
        
        Self {
            client,
            context,
            memory: ConversationMemory::new(),
            prompt_builder: PromptBuilder::new(),
            tool_executor: ToolExecutor::new(working_directory),
        }
    }

    pub fn with_working_directory(mut self, dir: String) -> Self {
        self.tool_executor = ToolExecutor::new(dir);
        self
    }

    pub fn with_memory(mut self, memory: ConversationMemory) -> Self {
        self.memory = memory;
        self
    }

    pub fn with_prompt_builder(mut self, builder: PromptBuilder) -> Self {
        self.prompt_builder = builder;
        self
    }

    pub fn with_provider_config(mut self, base_url: String, api_key: String) -> Self {
        self.client = LLMClient::new(
            base_url,
            self.context.provider.clone(),
            api_key,
        );
        self
    }

    pub fn get_context(&self) -> &AgentContext {
        &self.context
    }

    pub fn get_context_mut(&mut self) -> &mut AgentContext {
        &mut self.context
    }

    pub fn get_memory(&self) -> &ConversationMemory {
        &self.memory
    }

    pub fn get_memory_mut(&mut self) -> &mut ConversationMemory {
        &mut self.memory
    }

    pub fn build_system_prompt(&self) -> String {
        self.prompt_builder.build(&self.context)
    }

    pub fn build_tools(&self) -> Option<Vec<ToolDefinition>> {
        let mut tools = self.prompt_builder.build_tools(&self.context);
        let system_tools = self.tool_executor.get_tool_definitions();
        tools.extend(system_tools);
        
        if tools.is_empty() {
            None
        } else {
            Some(tools)
        }
    }

    pub fn get_tool_executor(&self) -> &ToolExecutor {
        &self.tool_executor
    }

    pub fn get_tool_executor_mut(&mut self) -> &mut ToolExecutor {
        &mut self.tool_executor
    }

    pub async fn execute_tool_call(&self, tool_call: &ToolCall) -> super::tools::ToolResult {
        self.tool_executor.execute_tool_call(tool_call).await
    }

    pub async fn execute_tool_calls(&self, tool_calls: &[ToolCall]) -> Vec<(String, super::tools::ToolResult)> {
        self.tool_executor.execute_tool_calls(tool_calls).await
    }

    pub async fn chat(
        &mut self,
        user_message: String,
    ) -> Result<
        (
            tokio::sync::mpsc::Receiver<OllamaStreamResponse>,
            tokio::sync::oneshot::Receiver<()>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let system_prompt = self.build_system_prompt();
        let tools = self.build_tools();

        let mut messages = Vec::new();

        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
            tool_calls: None,
        });

        messages.extend(self.memory.get_messages_for_api());

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_message.clone(),
            tool_calls: None,
        });

        self.memory.add_user_message(user_message);

        let result = self.client.chat_stream(&self.context.model, messages, tools).await;
        
        result
    }

    pub async fn chat_with_history(
        &mut self,
        user_message: String,
    ) -> Result<
        (
            tokio::sync::mpsc::Receiver<OllamaStreamResponse>,
            tokio::sync::oneshot::Receiver<()>,
        ),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.chat(user_message).await
    }

    pub fn add_assistant_response(&mut self, content: String) {
        self.memory.add_assistant_message(content);
    }

    pub fn add_assistant_response_with_tools(&mut self, content: String, tool_calls: Vec<ToolCall>) {
        self.memory.add_assistant_message_with_tools(content, tool_calls);
    }

    pub fn add_tool_result(&mut self, tool_call_id: String, tool_name: String, result: String) {
        self.memory.add_tool_result(tool_call_id, tool_name, result);
    }

    pub fn clear_conversation(&mut self) {
        self.memory.clear();
    }

    pub fn get_conversation_summary(&self) -> String {
        self.memory.get_conversation_summary()
    }

    pub fn update_context<F>(&mut self, f: F)
    where
        F: FnOnce(&mut AgentContext),
    {
        f(&mut self.context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> LLMService {
        let context = AgentContext::new(
            "agent-1".to_string(),
            "TestBot".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
        );

        LLMService::new(context)
    }

    #[test]
    fn test_llm_service_creation() {
        let service = create_test_service();

        assert_eq!(service.get_context().id, "agent-1");
        assert_eq!(service.get_context().name, "TestBot");
        assert!(service.get_memory().is_empty());
    }

    #[test]
    fn test_llm_service_system_prompt() {
        let service = create_test_service();
        let prompt = service.build_system_prompt();

        assert!(prompt.contains("TestBot"));
        assert!(prompt.contains("agent-1"));
    }

    #[test]
    fn test_llm_service_tools_with_system_tools() {
        let service = create_test_service();
        let tools = service.build_tools();

        assert!(tools.is_some());
        let tools = tools.unwrap();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.function.name == "bash"));
        assert!(tools.iter().any(|t| t.function.name == "read_file"));
        assert!(tools.iter().any(|t| t.function.name == "write_file"));
        assert!(tools.iter().any(|t| t.function.name == "list_directory"));
    }

    #[test]
    fn test_llm_service_add_messages() {
        let mut service = create_test_service();

        service.add_assistant_response("Hello".to_string());
        assert_eq!(service.get_memory().len(), 1);

        service.add_tool_result(
            "call_123".to_string(),
            "search".to_string(),
            "results".to_string(),
        );
        assert_eq!(service.get_memory().len(), 2);
    }

    #[test]
    fn test_llm_service_clear_conversation() {
        let mut service = create_test_service();

        service.add_assistant_response("Hello".to_string());
        assert!(!service.get_memory().is_empty());

        service.clear_conversation();
        assert!(service.get_memory().is_empty());
    }

    #[test]
    fn test_llm_service_conversation_summary() {
        let mut service = create_test_service();

        service.get_memory_mut().add_user_message("Hi".to_string());
        service.get_memory_mut().add_assistant_message("Hello".to_string());

        let summary = service.get_conversation_summary();
        assert!(summary.contains("1 user messages"));
        assert!(summary.contains("1 assistant messages"));
    }

    #[test]
    fn test_llm_service_update_context() {
        let mut service = create_test_service();

        service.update_context(|ctx| {
            ctx.metadata.insert("version".to_string(), "2.0".to_string());
        });

        assert_eq!(
            service.get_context().metadata.get("version"),
            Some(&"2.0".to_string())
        );
    }
}