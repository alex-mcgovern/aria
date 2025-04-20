use crate::graph::models::{CurrentNode, Deps, GraphError, NodeRunner, NodeTransition, State};
use crate::graph::nodes::{CallTools, End, ModelRequest, Start, UserRequest};
use providers::{models::ContentBlock, BaseProvider, Role};

/// A struct to hold the state of a graph iteration
pub struct GraphIter<P: BaseProvider> {
    deps: Deps<P>,
    state: State,
    current_node: CurrentNode,
    finished: bool,
    result: Option<String>,
}

impl<P: BaseProvider> GraphIter<P> {
    /// Create a new graph iterator
    pub fn new(deps: Deps<P>, user_prompt: String, use_streaming: bool) -> Self {
        let state = State {
            message_history: Vec::new(),
            current_user_prompt: user_prompt,
            tool_outputs: std::collections::HashMap::new(),
            use_streaming,
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
                        NodeTransition::ToModelRequest => {
                            self.current_node = CurrentNode::ModelRequest;
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
            CurrentNode::ModelRequest => {
                let result = ModelRequest.run(&mut self.state, &self.deps).await;
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
                                "Invalid transition from ModelRequest".to_string(),
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
                        NodeTransition::ToModelRequest => {
                            self.current_node = CurrentNode::ModelRequest;
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
                if let Some(last_message) = self.state.message_history.last() {
                    if last_message.role == Role::Assistant {
                        // Look for text blocks in the content array
                        for content_block in &last_message.content {
                            if let ContentBlock::Text { text } = content_block {
                                self.result = Some(text.clone());
                                break;
                            }
                        }
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
