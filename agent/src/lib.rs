use providers::BaseProvider;

pub mod graph;
pub use graph::{CurrentNode, Deps, GraphError, GraphIter, NodeRunner, NodeTransition, State};
use tools::{ListFilesTool, ReadFileTool, RunCommandTool, ToolType, TreeTool, WriteFileTool};

pub struct Agent<P: BaseProvider> {
    provider: P,
}

impl<P: BaseProvider> Agent<P> {
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
        let tools: Vec<ToolType> = vec![
            ToolType::ListFiles(ListFilesTool),
            ToolType::ReadFile(ReadFileTool),
            ToolType::RunCommand(RunCommandTool),
            ToolType::Tree(TreeTool),
            ToolType::WriteFile(WriteFileTool),
        ];
        
        // Create the dependencies for GraphIter
        let deps = Deps {
            provider: self.provider.clone(),
            tools: Some(tools),
            system_prompt: system_prompt.to_string(),
            max_tokens,
            temperature,
        };
        
        // Create GraphIter (streaming is now the default behavior)
        GraphIter::new(deps, user_prompt.to_string())
    }
}
