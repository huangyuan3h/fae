mod models;
mod openai;
mod ollama;
mod client;
mod utils;
mod context;
mod memory;
mod prompt_builder;
mod service;
mod tools;
mod tool_executor;
pub mod log;

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
pub use tools::{Tool, ToolResult, BashTool, ReadFileTool, WriteFileTool, ListDirectoryTool};
pub use tool_executor::ToolExecutor;
pub use log::{LLMLogger, LLMLogEntry, LLMEventType, LLMEventData, generate_session_id};