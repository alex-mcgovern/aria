use providers::{Message, BaseProvider};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use tools::ToolType;

/// Custom error type for the graph
#[derive(Debug)]
pub enum GraphError {
    MaxTokens,
    ToolNotImplemented(String),
    InvalidStateTransition(String),
    Other(anyhow::Error),
}

impl Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::MaxTokens => write!(f, "Max tokens reached"),
            GraphError::ToolNotImplemented(tool) => write!(f, "Tool not implemented: {}", tool),
            GraphError::InvalidStateTransition(msg) => {
                write!(f, "Invalid state transition: {}", msg)
            }
            GraphError::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for GraphError {}

impl From<anyhow::Error> for GraphError {
    fn from(err: anyhow::Error) -> Self {
        GraphError::Other(err)
    }
}

/// State shared between nodes
#[derive(Debug)]
pub struct State {
    pub message_history: Vec<Message>,
    pub current_user_prompt: String,
    pub tool_outputs: HashMap<String, String>,
}

/// Dependencies that nodes need to function
pub struct Deps<P: BaseProvider> {
    pub provider: P,
    pub tools: Option<Vec<ToolType>>,
    pub system_prompt: String,
    pub max_tokens: u32,
    pub temperature: Option<f64>,
}

/// A trait for running node logic without the associated type
/// This allows us to use dynamic dispatch with trait objects
pub trait NodeRunner<P: BaseProvider>: Debug {
    /// Run the node's logic
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError>;
}

/// Enum to represent all possible node transitions
#[derive(Debug)]
pub enum NodeTransition {
    ToUserRequest,
    ToModelRequest,
    ToCallTools,
    ToEnd,
    Terminal,
}

/// Enum representing the current node in the graph
#[derive(Debug, Clone)]
pub enum CurrentNode {
    Start,
    UserRequest,
    ModelRequest,
    CallTools,
    End,
}
