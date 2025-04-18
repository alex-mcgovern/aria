use crate::graph::models::{CurrentNode, Deps, GraphError, NodeRunner, NodeTransition, State};
use crate::graph::nodes::{CallTools, End, Start, UserRequest};
use providers::{Provider, Role};

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
    pub fn new(deps: Deps<P>, user_prompt: String) -> Self {
        let state = State {
            messages: Vec::new(),
            current_user_prompt: user_prompt,
            tool_outputs: std::collections::HashMap::new(),
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
        tools: Option<Vec<providers::Tool>>,
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
