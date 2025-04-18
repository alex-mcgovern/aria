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

    pub async fn process_input(&self, input: &str) -> Result<String> {
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

        // Format the prompt with instructions
        let prompt = format!(
            "You are an AI assistant helping with code editing tasks. \
             The user will provide a request, and you can use tools to help them. \
             Always explain what you're doing before using tools. \
             Request: {}",
            input
        );

        // Send the prompt to the provider
        let response = self.provider.send_prompt(&prompt, Some(tools)).await?;
        Ok(response.content)
    }

    /// Create a graph-based runner for more complex interactions with state tracking
    pub fn create_graph_runner(
        &self,
        system_prompt: String,
        max_tokens: u32,
        temperature: Option<f64>,
        tools: Option<Vec<Tool>>,
    ) -> GraphRunner<P>
    where
        P: Clone,
    {
        GraphRunner::new(
            self.provider.clone(),
            system_prompt,
            max_tokens,
            temperature,
            tools,
        )
    }

    /// Process input using the graph-based state machine
    pub async fn process_input_with_graph(
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

        let graph_runner = self.create_graph_runner(
            system_prompt.to_string(),
            max_tokens,
            temperature,
            Some(tools),
        );

        graph_runner.run(input.to_string()).await
    }
}
