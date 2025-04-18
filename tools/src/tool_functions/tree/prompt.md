# tree Tool

## Purpose
Use this tool to recursively list all files and directories within a directory and its subdirectories.

## When to Use
- When you need a complete view of a directory structure
- When searching for files that could be nested in subdirectories
- When you need to understand the organization of a project
- When you need to verify the presence of files deep in a directory tree

## When Not to Use
- When dealing with very large directory structures that could impact performance
- When you only need files from a single directory (use list_files instead)
- When you've already explored the directory structure recently
- When the specific file locations are already known

## Best Practices
1. **Start with specific directories**: Use on targeted directories rather than root directories
2. **Filter results**: Focus on relevant files and directories in the results
3. **Analyze structure**: Look for patterns in the directory organization
4. **Handle large results**: Be prepared to process potentially large lists of files
5. **Be specific**: Start with the deepest common directory for your task

## Parameters
- `dir`: The directory path to explore recursively (required)

## Example Usage
```
# To explore a project's source code:
tree("/path/to/src")

# To list all configuration files recursively:
tree("/path/to/config")

# To explore the entire project structure:
tree("/path/to/project")
```