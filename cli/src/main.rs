use agent::{Agent, CurrentNode};
use anyhow::Result;
use clap::{Parser, Subcommand};
use providers::Provider;
use providers::{anthropic::AnthropicProvider, models::ContentBlock, Role};
use std::io::{self, Write};

// Constants for the process_input_with_graph parameters
const DEFAULT_SYSTEM_PROMPT: &str = "You are an AI assistant helping with code editing tasks. \
The user will provide a request, and you can use tools to help them. \
Always explain what you're doing before using tools.";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_TEMPERATURE: f64 = 0.7;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run an interactive session with the agent
    Interactive {
        /// The directory to work in
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Execute a single command
    Exec {
        /// The command to execute
        #[arg(required = true)]
        prompt: String,

        /// The directory to work in
        #[arg(short, long)]
        dir: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
        anyhow::anyhow!("ANTHROPIC_API_KEY environment variable not set. Please export ANTHROPIC_API_KEY='your-key-here' and try again.")
    })?;

    // Create provider and agent
    let provider = AnthropicProvider::new(api_key, "claude-3-7-sonnet-20250219".to_string())?;
    let agent = Agent::new(provider);

    // Handle commands
    match &cli.command {
        Some(Commands::Interactive { dir }) => {
            if let Some(dir_path) = dir {
                std::env::set_current_dir(dir_path)?;
                println!("Working directory set to: {}", dir_path);
            }

            interactive_loop(&agent).await?;
        }
        Some(Commands::Exec { prompt, dir }) => {
            if let Some(dir_path) = dir {
                std::env::set_current_dir(dir_path)?;
                println!("Working directory set to: {}", dir_path);
            }

            execute_with_graph_iter(&agent, prompt).await?;
        }
        None => {
            // Default to interactive mode if no command specified
            interactive_loop(&agent).await?;
        }
    }

    Ok(())
}

async fn execute_with_graph_iter<P: Provider>(agent: &Agent<P>, input: &str) -> Result<()>
where
    P: Clone,
{
    println!("Processing input: {}", input);

    // Create graph iterator
    let mut graph_iter = agent.iter(
        input,
        DEFAULT_SYSTEM_PROMPT,
        DEFAULT_MAX_TOKENS,
        Some(DEFAULT_TEMPERATURE),
    );

    // Process each node
    while let Some(node_result) = graph_iter.next().await {
        match node_result {
            Ok(node) => {
                println!("Processing node: {:?}", node);

                // Special handling for UserRequest node
                if matches!(node, CurrentNode::UserRequest) {
                    if let Some(last_message) = graph_iter.state().messages.last() {
                        if last_message.role == Role::Assistant {
                            // Look for text content in the array
                            for content_block in &last_message.content {
                                if let ContentBlock::Text { text } = content_block {
                                    println!("Response received: {}", text);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing node: {:?}", e);
                return Err(anyhow::anyhow!("Graph processing error: {:?}", e));
            }
        }
    }

    // Get the final result
    if let Some(result) = graph_iter.get_result() {
        println!("Final result: {}", result);
    } else {
        println!("No final result available");
    }

    Ok(())
}

async fn interactive_loop<P: Provider>(agent: &Agent<P>) -> Result<()>
where
    P: Clone,
{
    println!("Interactive mode. Enter 'exit' or 'quit' to end the session.");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Use the graph iterator
        if let Err(e) = execute_with_graph_iter(agent, input).await {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}
