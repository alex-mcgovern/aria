use providers::{Provider, Tool};

pub mod graph;
pub use graph::{CurrentNode, Deps, GraphError, GraphIter, NodeRunner, NodeTransition, State};

pub struct Agent<P: Provider> {
    provider: P,
}

impl<P: Provider> Agent<P> {
    pub fn new(provider: P) -> Self {
        Agent { provider }
    }

    /// Process input using the graph-based state machine and return an iterator
    pub fn iter(
        &self,
        user_prompt: &str,
        system_prompt: &str,
        max_tokens: u32,
        temperature: Option<f64>,
    ) -> GraphIter<P>
    where
        P: Clone,
    {
        // Create tool definitions for the provider
        let tool_schemas = tools::get_tool_schemas();
        let tools: Vec<Tool> = tool_schemas
            .into_iter()
            .map(|schema| Tool {
                name: schema["name"].as_str().unwrap().to_string(),
                description: schema["description"].as_str().unwrap().to_string(),
                input_schema: schema["input_schema"].clone(),
            })
            .collect();

        // Create the dependencies for GraphIter
        let deps = Deps {
            provider: self.provider.clone(),
            tools: Some(tools),
            system_prompt: system_prompt.to_string(),
            max_tokens,
            temperature,
        };

        // Create GraphIter directly
        GraphIter::new(deps, user_prompt.to_string())
    }
}
