use agent::Agent;
use anyhow::Result;
use clap::{Parser, Subcommand};
use providers::claude::ClaudeProvider;
use providers::Provider;
use std::io::{self, Write};

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
    let provider = ClaudeProvider::new(api_key, "claude-3-7-sonnet-20250219".to_string())?;
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

            let response = agent.process_input(&prompt).await?;
            println!("{}", response);
        }
        None => {
            // Default to interactive mode if no command specified
            interactive_loop(&agent).await?;
        }
    }

    Ok(())
}

async fn interactive_loop<P: Provider>(agent: &Agent<P>) -> Result<()> {
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

        match agent.process_input(input).await {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
