# A.R.I.A - Agentic Reasoning and Implementation Assistant

Aria is a lightweight agentic codegen assistant that runs in your terminal (for now).

## Demo

https://github.com/user-attachments/assets/dddc0ff0-a389-44dd-a062-b5a78e1f15a4

## Features

- CLI-based interface with both interactive and command execution modes
- Built with Rust for performance and reliability
- Integrated tool functions for file operations and command execution:
  - Reading and writing files
  - Listing files and directory structures
  - Executing shell commands

## Supported Providers

Currently supported:
- Anthropic (Claude)

Coming soon (maybe):
- OpenAI
- Ollama
- OpenRouter

## Installation

> [!NOTE]  
> This is a "toy" project, not intended for distribution, if you wish to use it locally, you will need to install it from source.

Install from the project root directory:

```bash
cargo install --path cli
```

## Configuration

Aria is configured via an `aria.yml` file, which can be placed in:
- The current project directory
- `~/.config/aria/` directory

### Reference Configuration

```yaml
provider: Anthropic
api_key: "your_api_key_here"  # Optional (default: None) — some providers may require it
model: "claude-3-7-sonnet-20250219"
max_tokens: 8192  # Optional (default: 4096)
temperature: 0.7  # Optional (default: 0.7)
provider_base_url: "https://api.anthropic.com"  # Optional — a default is provided for each provider
```

## Usage

```bash
# Start an interactive session
aria interactive

# Execute a single command
aria exec "refactor this function to be more efficient"

# Work in a specific directory
aria interactive --dir /path/to/your/project
```

## Status

This project is still under active development. The current focus is on improving the core functionality and adding more provider support.

### Roadmap

- API for integration with other tools
- UI interface for easier interaction
- Additional provider support

## License

MIT
