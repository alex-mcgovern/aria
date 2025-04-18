use anyhow::Context;
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
    pub current_user_prompt: String,
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
            content: state.current_user_prompt.clone(),
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
            .send_prompt(&state.current_user_prompt, deps.tools.clone())
            .await
            .context("Failed to send prompt to provider")?;

        // Add the response to messages
        state.messages.push(Message {
            role: Role::Assistant,
            content: response.content.clone(),
        });

        println!("Assistant response: {}", response.content);
        println!("[GRAPH] Stop reason: {:?}", response.stop_reason);

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

/// Enum representing the current node in the graph
#[derive(Debug, Clone)]
pub enum CurrentNode {
    Start,
    UserRequest,
    CallTools,
    End,
}

/// A struct to hold the state of a graph iteration
pub struct GraphIter<P: Provider> {
    deps: Deps<P>,
    state: State,
    current_node: CurrentNode,
    finished: bool,
    result: Option<String>,
}

impl<P: Provider> GraphIter<P> {
    /// Create a new graph iterator
    fn new(deps: Deps<P>, user_prompt: String) -> Self {
        let state = State {
            messages: Vec::new(),
            current_user_prompt: user_prompt,
            tool_outputs: HashMap::new(),
        };

        GraphIter {
            deps,
            state,
            current_node: CurrentNode::Start,
            finished: false,
            result: None,
        }
    }

    /// Get the result of the graph execution
    pub fn get_result(&self) -> Option<&str> {
        self.result.as_deref()
    }

    /// Run the next node in the graph
    pub async fn next(&mut self) -> Option<std::result::Result<CurrentNode, GraphError>> {
        if self.finished {
            return None;
        }

        let transition_result = match self.current_node {
            CurrentNode::Start => {
                let result = Start.run(&mut self.state, &self.deps).await;
                self.current_node = CurrentNode::UserRequest;
                result.map(|_| self.current_node.clone())
            }
            CurrentNode::UserRequest => {
                let result = UserRequest.run(&mut self.state, &self.deps).await;
                match &result {
                    Ok(transition) => match transition {
                        NodeTransition::ToCallTools => {
                            self.current_node = CurrentNode::CallTools;
                        }
                        NodeTransition::ToEnd => {
                            self.current_node = CurrentNode::End;
                        }
                        _ => {
                            return Some(Err(GraphError::InvalidStateTransition(
                                "Invalid transition from UserRequest".to_string(),
                            )));
                        }
                    },
                    Err(_) => {
                        // On error, we'll return the error and mark as finished
                        self.finished = true;
                    }
                }
                result.map(|_| self.current_node.clone())
            }
            CurrentNode::CallTools => {
                let result = CallTools.run(&mut self.state, &self.deps).await;
                match &result {
                    Ok(transition) => match transition {
                        NodeTransition::ToUserRequest => {
                            self.current_node = CurrentNode::UserRequest;
                        }
                        NodeTransition::ToEnd => {
                            self.current_node = CurrentNode::End;
                        }
                        _ => {
                            return Some(Err(GraphError::InvalidStateTransition(
                                "Invalid transition from CallTools".to_string(),
                            )));
                        }
                    },
                    Err(_) => {
                        // On error, we'll return the error and mark as finished
                        self.finished = true;
                    }
                }
                result.map(|_| self.current_node.clone())
            }
            CurrentNode::End => {
                let result = End.run(&mut self.state, &self.deps).await;

                // Store the result if we've reached the end
                if let Some(last_message) = self.state.messages.last() {
                    if last_message.role == Role::Assistant {
                        self.result = Some(last_message.content.clone());
                    }
                }

                self.finished = true;
                result.map(|_| self.current_node.clone())
            }
        };

        Some(transition_result)
    }

    /// Get the current state
    pub fn state(&self) -> &State {
        &self.state
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

    /// Create a new graph iterator
    pub fn create_iter(&self, user_prompt: String) -> GraphIter<P>
    where
        P: Clone,
    {
        let deps = Deps {
            provider: self.deps.provider.clone(),
            tools: self.deps.tools.clone(),
            system_prompt: self.deps.system_prompt.clone(),
            max_tokens: self.deps.max_tokens,
            temperature: self.deps.temperature,
        };

        GraphIter::new(deps, user_prompt)
    }
}
