# run_command Tool

## Purpose
Use this tool to execute shell commands in the terminal environment.

## When to Use
- When you need to run system commands or executables 
- When you need to query information from the system
- When you need to install packages or software
- When you need to execute scripts or build commands

## When Not to Use
- When the command could harm the system or delete important files
- When the command would take too long to execute
- When you can accomplish the task with a different, more specific tool
- When the command requires user interaction

## Best Practices
1. **Validate inputs**: Ensure the command and arguments are properly formatted and safe
2. **Be specific**: Use precise commands with appropriate flags rather than complex one-liners
3. **Explain your intent**: Always explain what the command will do before running it
4. **Handle errors**: Be prepared to handle and interpret error messages
5. **Start simple**: Begin with simple commands and iteratively add complexity if needed

## Parameters
- `command`: The primary command to execute (required)
- `args`: An array of command-line arguments (optional)

## Example Usage
```
# To list files in a directory:
run_command("ls", ["-la", "/path/to/directory"])

# To check git status:
run_command("git", ["status"])

# To install a package:
run_command("npm", ["install", "--save", "package-name"])
```