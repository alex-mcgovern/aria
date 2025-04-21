use providers::BaseProvider;

pub mod graph;
pub use graph::models::StreamWrapper;
pub use graph::{CurrentNode, Deps, GraphError, GraphIter, NodeRunner, NodeTransition, State};
use tools::{ListFilesTool, ReadFileTool, RunCommandTool, ToolType, TreeTool, WriteFileTool};

pub struct Agent<P: BaseProvider> {
    provider: P,
}

impl<P: BaseProvider> Agent<P> {
    pub fn new(provider: P) -> Self {
        Agent { provider }
    }

    pub fn iter(
        &self,
        user_prompt: &str,
        system_prompt: &str,
        max_tokens: u32,
        temperature: Option<f64>,
        stream_wrapper: Option<Box<dyn StreamWrapper>>,
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

        let deps = Deps::new(
            self.provider.clone(),
            Some(tools),
            system_prompt.to_string(),
            max_tokens,
            temperature,
            stream_wrapper,
        );

        GraphIter::new(deps, user_prompt.to_string())
    }
}
