mod models;
mod openai;
mod ollama;
mod client;
mod utils;
mod context;
mod memory;
mod prompt_builder;
mod service;

pub use models::{
    ChatMessage,
    ToolCall,
    FunctionCall,
    ToolDefinition,
    ToolFunction,
    OllamaRequest,
    OllamaStreamResponse,
    OllamaChatResponse,
    OpenAIStreamChoice,
    OpenAIDelta,
    OpenAIToolCall,
    OpenAIFunctionCall,
    OpenAIStreamResponse,
};

pub use client::LLMClient;
pub use utils::{skills_to_tools, build_system_prompt};
pub use context::{AgentContext, SkillInfo};
pub use memory::ConversationMemory;
pub use prompt_builder::PromptBuilder;
pub use service::LLMService;