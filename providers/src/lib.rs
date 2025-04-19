pub mod claude;
pub mod models;

// Re-export common types and traits from models
pub use models::{Message, Provider, ProviderResponse, Request, ResponseContent, Role, StopReason};

// Re-export the ClaudeProvider for easier access
pub use claude::ClaudeProvider;
