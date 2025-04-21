use agent::{Agent, CurrentNode};
use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{load_config_file, Config, ProviderType};
use providers::{models::ContentBlock, Role};
use providers::{BaseProvider, Provider};
use std::io::{self, Write};

// Import the stream wrapper
mod stream_wrapper;
use stream_wrapper::CliStreamWrapper;

// Constants for the process_input_with_graph parameters
const DEFAULT_SYSTEM_PROMPT: &str = "You are an AI assistant helping with code editing tasks. \
The user will provide a request, and you can use tools to help them. \
Always explain what you're doing before using tools.";

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

    // Load config from file
    let config = match load_config_file() {
        Ok(config) => {
            println!("Loaded configuration from file");
            config
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to load config: {}", e));
        }
    };

    // Create provider based on config
    let provider = create_provider_from_config(&config)?;

    // Create agent
    let agent = Agent::new(provider);

    // Handle commands
    match &cli.command {
        Some(Commands::Interactive { dir }) => {
            if let Some(dir_path) = dir {
                std::env::set_current_dir(dir_path)?;
                println!("Working directory set to: {}", dir_path);
            }
            interactive_loop(&agent, &config).await?;
        }
        Some(Commands::Exec { prompt, dir }) => {
            if let Some(dir_path) = dir {
                std::env::set_current_dir(dir_path)?;
                println!("Working directory set to: {}", dir_path);
            }
            execute_with_graph_iter(&agent, prompt, &config).await?;
        }
        None => {
            // Default to interactive mode if no command specified
            interactive_loop(&agent, &config).await?;
        }
    }

    Ok(())
}

// Create a provider from config without relying on TryFrom implementation
fn create_provider_from_config(config: &Config) -> Result<Provider> {
    // Create provider based on config type
    match &config.provider {
        ProviderType::Anthropic => Provider::new_anthropic(
            config.api_key.clone(),
            config.model.clone(),
            config.provider_base_url.clone(),
        ),
    }
}

async fn execute_with_graph_iter<P: BaseProvider>(
    agent: &Agent<P>,
    input: &str,
    config: &Config,
) -> Result<()>
where
    P: Clone,
{
    // Create a CLI stream wrapper
    let stream_wrapper = Box::new(CliStreamWrapper);

    // Create graph iterator with our stream wrapper
    let mut graph_iter = agent.iter(
        input,
        DEFAULT_SYSTEM_PROMPT,
        config.response_max_tokens,
        Some(config.temperature as f64),
        Some(stream_wrapper),
    );

    // Process each node
    while let Some(node_result) = graph_iter.next().await {
        match node_result {
            Ok(node) => {
                // All streaming is handled by the CliStreamWrapper, so we don't need to do anything here
                // with the stream_receiver anymore
                if matches!(node, CurrentNode::UserRequest) {
                    if let Some(last_message) = graph_iter.state().message_history.last() {
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

    Ok(())
}

async fn interactive_loop<P: BaseProvider>(agent: &Agent<P>, config: &Config) -> Result<()>
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
        if let Err(e) = execute_with_graph_iter(agent, input, config).await {
            eprintln!("Error: {}", e);
            std::io::stdout().flush().expect("Failed to flush stdout");
        }
    }

    Ok(())
}
