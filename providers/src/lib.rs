pub mod models;
pub mod claude;

// Re-export common types and traits from models
pub use models::{Provider, ProviderResponse, Role, StopReason, Tool, Message, Request};

// Re-export the ClaudeProvider for easier access
pub use claude::ClaudeProvider;
