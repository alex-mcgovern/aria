use anyhow::Result;
use providers::{Provider, Tool};

pub mod graph;
pub use graph::{Deps, GraphError, GraphRunner, NodeRunner, NodeTransition, State};

pub struct Agent<P: Provider> {
    provider: P,
}

impl<P: Provider> Agent<P> {
    pub fn new(provider: P) -> Self {
        Agent { provider }
    }

    /// Process input using the graph-based state machine
    pub async fn run(
        &self,
        input: &str,
        system_prompt: &str,
        max_tokens: u32,
        temperature: Option<f64>,
    ) -> std::result::Result<String, GraphError>
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

        let graph_runner = GraphRunner::new(
            self.provider.clone(),
            system_prompt.to_string(),
            max_tokens,
            temperature,
            Some(tools),
        );

        graph_runner.run(input.to_string()).await
    }
}
