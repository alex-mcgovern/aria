use anyhow::{anyhow, Context};
use providers::{Message, Provider, Role, StopReason, Tool};
use std::collections::HashMap;
use std::fmt::{Debug, Display};

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
    pub messages: Vec<Message>,
    pub current_input: String,
    pub tool_outputs: HashMap<String, String>,
}

/// Dependencies that nodes need to function
pub struct Deps<P: Provider> {
    pub provider: P,
    pub tools: Option<Vec<Tool>>,
    pub system_prompt: String,
    pub max_tokens: u32,
    pub temperature: Option<f64>,
}

/// A trait for running node logic without the associated type
/// This allows us to use dynamic dispatch with trait objects
pub trait NodeRunner<P: Provider>: Debug {
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
    ToCallTools,
    ToEnd,
    Terminal,
}

/// The starting node
#[derive(Debug)]
pub struct Start;

/// The user request node
#[derive(Debug)]
pub struct UserRequest;

/// The tool calling node
#[derive(Debug)]
pub struct CallTools;

/// The end node
#[derive(Debug)]
pub struct End;

impl<P: Provider> NodeRunner<P> for Start {
    async fn run(
        &self,
        state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Setup initial state with user input
        state.messages.push(Message {
            role: Role::User,
            content: state.current_input.clone(),
        });

        Ok(NodeTransition::ToUserRequest)
    }
}

impl<P: Provider> NodeRunner<P> for UserRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Send the current messages to the LLM provider
        let response = deps
            .provider
            .send_prompt(&state.current_input, deps.tools.clone())
            .await
            .context("Failed to send prompt to provider")?;

        // Add the response to messages
        state.messages.push(Message {
            role: Role::Assistant,
            content: response.content.clone(),
        });

        // Route based on stop reason
        match response.stop_reason {
            Some(StopReason::MaxTokens) => {
                return Err(GraphError::MaxTokens);
            }
            Some(StopReason::ToolUse) => Ok(NodeTransition::ToCallTools),
            _ => {
                // EndTurn, StopSequence, or None
                Ok(NodeTransition::ToEnd)
            }
        }
    }
}

impl<P: Provider> NodeRunner<P> for CallTools {
    async fn run(
        &self,
        state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // This would normally parse the tool request from the LLM response
        // and execute the requested tool

        println!("Tool request received but not implemented yet");
        println!("Messages: {:?}", state.messages);

        // Just a placeholder - in a real implementation we would:
        // 1. Extract tool name and parameters from the last message
        // 2. Call the tool
        // 3. Store the result in tool_outputs
        // 4. Create a new message with the tool result

        Err(GraphError::ToolNotImplemented(
            "Tools not implemented yet".to_string(),
        ))
    }
}

impl<P: Provider> NodeRunner<P> for End {
    async fn run(
        &self,
        _state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // End node doesn't transition to any other node
        Ok(NodeTransition::Terminal)
    }
}

/// The graph runner
pub struct GraphRunner<P: Provider> {
    deps: Deps<P>,
}

impl<P: Provider> GraphRunner<P> {
    pub fn new(
        provider: P,
        system_prompt: String,
        max_tokens: u32,
        temperature: Option<f64>,
        tools: Option<Vec<Tool>>,
    ) -> Self {
        let deps = Deps {
            provider,
            tools,
            system_prompt,
            max_tokens,
            temperature,
        };

        GraphRunner { deps }
    }

    pub async fn run(&self, input: String) -> std::result::Result<String, GraphError> {
        let mut state = State {
            messages: Vec::new(),
            current_input: input,
            tool_outputs: HashMap::new(),
        };

        // Start with the Start node and progress through the state machine
        let mut current_transition = Start.run(&mut state, &self.deps).await?;

        loop {
            match current_transition {
                NodeTransition::ToUserRequest => {
                    current_transition = UserRequest.run(&mut state, &self.deps).await?;
                }
                NodeTransition::ToCallTools => {
                    current_transition = CallTools.run(&mut state, &self.deps).await?;
                }
                NodeTransition::ToEnd => {
                    // Run the End node and then exit the loop
                    End.run(&mut state, &self.deps).await?;
                    break;
                }
                NodeTransition::Terminal => {
                    // This is used by the End node - we've reached a terminal state
                    break;
                }
            }
        }

        // Return the assistant's response (last message)
        if let Some(last_message) = state.messages.last() {
            if last_message.role == Role::Assistant {
                return Ok(last_message.content.clone());
            }
        }

        Err(GraphError::Other(anyhow!("No assistant response found")))
    }
}
