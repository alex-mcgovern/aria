use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::{models::ContentBlock, Message, BaseProvider, Role};
use serde_json::Value;
use tools::{
    models::{ToolName, ToolResult},
    ListFilesInput, ReadFileInput, RunCommandInput, Tool, ToolType, TreeInput, WriteFileInput,
};

/// The tool calling node
#[derive(Debug)]
pub struct CallTools;

impl<P: BaseProvider> NodeRunner<P> for CallTools {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Check if the last message is from the assistant and contains a tool use request
        let last_msg = state
            .message_history
            .last()
            .ok_or_else(|| GraphError::Other(anyhow::anyhow!("No messages in history")))?;

        // Only process if the last message is from the assistant
        if last_msg.role != Role::Assistant {
            return Err(GraphError::InvalidStateTransition(
                "Last message is not from assistant".to_string(),
            ));
        }

        // Extract tool use content block and process it
        for content_block in &last_msg.content {
            if let ContentBlock::ToolUse { id, name, input } = content_block {
                // Make sure we have tools available
                let tools = deps.tools.as_ref().ok_or_else(|| {
                    GraphError::Other(anyhow::anyhow!(
                        "No tools available in the agent's dependencies"
                    ))
                })?;

                // Execute the tool
                let tool_result = execute_tool(name, input, tools)
                    .await
                    .map_err(|e| GraphError::Other(e))?;

                // Create result message text
                let result_content = match tool_result.is_error {
                    true => format!("Error: {}", tool_result.content),
                    false => format!("{}", tool_result.content),
                };

                // Store the tool output in the state's tool_outputs HashMap
                state
                    .tool_outputs
                    .insert(id.clone(), result_content.clone());

                // Add the tool result message to the message history
                state.message_history.push(Message {
                    role: Role::User,
                    content: vec![ContentBlock::ToolResult {
                        tool_use_id: id.clone(),
                        content: result_content,
                    }],
                });

                // Found and processed a tool, transition to the model request node
                return Ok(NodeTransition::ToModelRequest);
            }
        }

        // If we get here, no tool use was found
        Err(GraphError::InvalidStateTransition(
            "No tool use request found in the last message".to_string(),
        ))
    }
}

/// Execute a tool based on its name and input
async fn execute_tool(
    tool_name: &ToolName,
    input: &Value,
    tools: &Vec<ToolType>,
) -> anyhow::Result<ToolResult> {
    // Execute the tool based on its name
    match tool_name {
        ToolName::ListFiles => {
            // Find the ListFiles tool in the tools vec
            let tool = tools
                .iter()
                .find_map(|t| {
                    if let ToolType::ListFiles(tool) = t {
                        Some(tool)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("ListFiles tool not found"))?;

            // Parse the input
            let input: ListFilesInput = serde_json::from_value(input.clone())?;

            println!("ListFiles input: {:?}", input);

            // Execute the tool
            Ok(tool.run(input).await)
        }
        ToolName::ReadFile => {
            // Find the ReadFile tool in the tools vec
            let tool = tools
                .iter()
                .find_map(|t| {
                    if let ToolType::ReadFile(tool) = t {
                        Some(tool)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("ReadFile tool not found"))?;

            // Parse the input
            let input: ReadFileInput = serde_json::from_value(input.clone())?;

            println!("ListFiles input: {:?}", input);

            // Execute the tool
            Ok(tool.run(input).await)
        }
        ToolName::RunCommand => {
            // Find the RunCommand tool in the tools vec
            let tool = tools
                .iter()
                .find_map(|t| {
                    if let ToolType::RunCommand(tool) = t {
                        Some(tool)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("RunCommand tool not found"))?;

            // Parse the input
            let input: RunCommandInput = serde_json::from_value(input.clone())?;

            println!("ListFiles input: {:?}", input);

            // Execute the tool
            Ok(tool.run(input).await)
        }
        ToolName::Tree => {
            // Find the Tree tool in the tools vec
            let tool = tools
                .iter()
                .find_map(|t| {
                    if let ToolType::Tree(tool) = t {
                        Some(tool)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("Tree tool not found"))?;

            // Parse the input
            let input: TreeInput = serde_json::from_value(input.clone())?;

            println!("ListFiles input: {:?}", input);

            // Execute the tool
            Ok(tool.run(input).await)
        }
        ToolName::WriteFile => {
            // Find the WriteFile tool in the tools vec
            let tool = tools
                .iter()
                .find_map(|t| {
                    if let ToolType::WriteFile(tool) = t {
                        Some(tool)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("WriteFile tool not found"))?;

            // Parse the input
            let input: WriteFileInput = serde_json::from_value(input.clone())?;

            println!("ListFiles input: {:?}", input);

            // Execute the tool
            Ok(tool.run(input).await)
        }
    }
}
