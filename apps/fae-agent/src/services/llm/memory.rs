use super::models::{ChatMessage, ToolCall};

#[derive(Debug, Clone)]
pub struct ConversationMemory {
    messages: Vec<ChatMessage>,
    max_messages: Option<usize>,
}

impl ConversationMemory {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: None,
        }
    }

    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = Some(max);
        self
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);

        if let Some(max) = self.max_messages {
            if self.messages.len() > max {
                let remove_count = self.messages.len() - max;
                self.messages.drain(0..remove_count);
            }
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.add_message(ChatMessage {
            role: "user".to_string(),
            content,
            tool_calls: None,
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.add_message(ChatMessage {
            role: "assistant".to_string(),
            content,
            tool_calls: None,
        });
    }

    pub fn add_assistant_message_with_tools(&mut self, content: String, tool_calls: Vec<ToolCall>) {
        self.add_message(ChatMessage {
            role: "assistant".to_string(),
            content,
            tool_calls: Some(tool_calls),
        });
    }

    pub fn add_tool_result(&mut self, _tool_call_id: String, tool_name: String, result: String) {
        self.add_message(ChatMessage {
            role: "tool".to_string(),
            content: format!("Tool result for {}: {}", tool_name, result),
            tool_calls: None,
        });
    }

    pub fn add_system_message(&mut self, content: String) {
        self.messages.insert(
            0,
            ChatMessage {
                role: "system".to_string(),
                content,
                tool_calls: None,
            },
        );
    }

    pub fn get_messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn get_messages_for_api(&self) -> Vec<ChatMessage> {
        self.messages.clone()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn last_user_message(&self) -> Option<&ChatMessage> {
        self.messages.iter().rev().find(|m| m.role == "user")
    }

    pub fn last_assistant_message(&self) -> Option<&ChatMessage> {
        self.messages.iter().rev().find(|m| m.role == "assistant")
    }

    pub fn get_conversation_summary(&self) -> String {
        let user_count = self.messages.iter().filter(|m| m.role == "user").count();
        let assistant_count = self
            .messages
            .iter()
            .filter(|m| m.role == "assistant")
            .count();
        let tool_count = self.messages.iter().filter(|m| m.role == "tool").count();

        format!(
            "Conversation: {} user messages, {} assistant messages, {} tool calls",
            user_count, assistant_count, tool_count
        )
    }

    pub fn truncate_to_last_n_messages(&mut self, n: usize) {
        if self.messages.len() > n {
            let system_messages: Vec<ChatMessage> = self
                .messages
                .iter()
                .filter(|m| m.role == "system")
                .cloned()
                .collect();

            let recent_messages: Vec<ChatMessage> =
                self.messages.iter().rev().take(n).rev().cloned().collect();

            self.messages = system_messages;
            self.messages.extend(recent_messages);
        }
    }
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::models::FunctionCall;
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = ConversationMemory::new();
        assert!(memory.is_empty());
        assert_eq!(memory.len(), 0);
    }

    #[test]
    fn test_add_user_message() {
        let mut memory = ConversationMemory::new();
        memory.add_user_message("Hello".to_string());

        assert_eq!(memory.len(), 1);
        assert_eq!(memory.last_user_message().unwrap().content, "Hello");
    }

    #[test]
    fn test_add_assistant_message() {
        let mut memory = ConversationMemory::new();
        memory.add_assistant_message("Hi there".to_string());

        assert_eq!(memory.len(), 1);
        assert_eq!(memory.last_assistant_message().unwrap().content, "Hi there");
    }

    #[test]
    fn test_add_tool_result() {
        let mut memory = ConversationMemory::new();
        memory.add_tool_result(
            "call_123".to_string(),
            "search".to_string(),
            "Found results".to_string(),
        );

        assert_eq!(memory.len(), 1);
        let msg = &memory.get_messages()[0];
        assert_eq!(msg.role, "tool");
        assert!(msg.content.contains("search"));
        assert!(msg.content.contains("Found results"));
    }

    #[test]
    fn test_max_messages_limit() {
        let mut memory = ConversationMemory::new().with_max_messages(3);

        memory.add_user_message("1".to_string());
        memory.add_user_message("2".to_string());
        memory.add_user_message("3".to_string());
        memory.add_user_message("4".to_string());

        assert_eq!(memory.len(), 3);
        assert_eq!(memory.get_messages()[0].content, "2");
        assert_eq!(memory.get_messages()[2].content, "4");
    }

    #[test]
    fn test_system_message_always_first() {
        let mut memory = ConversationMemory::new();
        memory.add_user_message("Hello".to_string());
        memory.add_system_message("You are helpful".to_string());

        assert_eq!(memory.get_messages()[0].role, "system");
        assert_eq!(memory.get_messages()[1].role, "user");
    }

    #[test]
    fn test_conversation_summary() {
        let mut memory = ConversationMemory::new();
        memory.add_user_message("Hi".to_string());
        memory.add_assistant_message("Hello".to_string());
        memory.add_user_message("How are you?".to_string());
        memory.add_assistant_message("Good".to_string());

        let summary = memory.get_conversation_summary();
        assert!(summary.contains("2 user messages"));
        assert!(summary.contains("2 assistant messages"));
    }

    #[test]
    fn test_truncate_messages() {
        let mut memory = ConversationMemory::new();
        memory.add_system_message("System".to_string());
        memory.add_user_message("1".to_string());
        memory.add_user_message("2".to_string());
        memory.add_user_message("3".to_string());
        memory.add_user_message("4".to_string());

        memory.truncate_to_last_n_messages(2);

        assert_eq!(memory.len(), 3);
        assert_eq!(memory.get_messages()[0].role, "system");
        assert_eq!(memory.get_messages()[1].content, "3");
        assert_eq!(memory.get_messages()[2].content, "4");
    }

    #[test]
    fn test_clear_memory() {
        let mut memory = ConversationMemory::new();
        memory.add_user_message("Hello".to_string());
        memory.add_assistant_message("Hi".to_string());

        memory.clear();

        assert!(memory.is_empty());
    }

    #[test]
    fn test_assistant_with_tools() {
        let mut memory = ConversationMemory::new();
        let tool_calls = vec![ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "search".to_string(),
                arguments: r#"{"q":"test"}"#.to_string(),
            },
        }];

        memory.add_assistant_message_with_tools("Searching".to_string(), tool_calls);

        let msg = memory.last_assistant_message().unwrap();
        assert!(msg.tool_calls.is_some());
        assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);
    }
}
