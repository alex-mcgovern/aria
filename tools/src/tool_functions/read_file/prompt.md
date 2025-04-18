# read_file Tool

## Purpose
Use this tool to read the contents of a file from the filesystem.

## When to Use
- When you need to inspect the contents of a specific file
- When you need to analyze code, configuration files, or text data
- When you need to check logs or output files
- When you need to verify file contents before making changes

## When Not to Use
- When dealing with very large files that might cause performance issues
- When accessing sensitive files like private keys or credentials
- When reading binary files (this tool expects text files)
- When you've already read the file recently and don't need to read it again

## Best Practices
1. **Verify path first**: Ensure the file exists before trying to read it
2. **Handle potential errors**: Be prepared for file access permissions or file not found errors
3. **Process content appropriately**: Handle the file content based on its format (JSON, YAML, code, etc.)
4. **Consider context**: Only read files relevant to the current task
5. **Be judicious**: Don't read more files than necessary to complete a task

## Parameters
- `path`: The file path to read (required)

## Example Usage
```
# To read a configuration file:
read_file("/path/to/config.json")

# To read a source code file:
read_file("/path/to/src/main.rs")

# To read a log file:
read_file("/path/to/logs/application.log")
```