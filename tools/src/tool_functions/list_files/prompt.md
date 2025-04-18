# list_files Tool

## Purpose
Use this tool to list all files and directories in a specific directory.

## When to Use
- When you need to explore the contents of a directory
- When you need to find specific files in a directory
- When you need to verify file existence before operations
- When you want to enumerate available resources in a location

## When Not to Use
- When you need to recursively list all files (use tree tool instead)
- When dealing with directories containing thousands of files
- When you only need to check if a specific file exists
- When you already have the file list from a recent call

## Best Practices
1. **Start with important directories**: Focus on directories most relevant to the task
2. **Filter mentally**: Process the results to focus on relevant files
3. **Follow up with specific actions**: Use the results to guide your next steps
4. **Handle potential errors**: Be prepared for permission denied errors
5. **Consider recursion needs**: If you need recursive listing, use the tree tool instead

## Parameters
- `dir`: The directory path to list (required)

## Example Usage
```
# To list files in the current project directory:
list_files("/path/to/project")

# To list configuration files:
list_files("/path/to/config")

# To list source code:
list_files("/path/to/src")
```