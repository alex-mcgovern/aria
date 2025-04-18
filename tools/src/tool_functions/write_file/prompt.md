# write_file Tool

## Purpose
Use this tool to create or modify files on the filesystem by writing text content.

## When to Use
- When creating new configuration files, source code, or documentation
- When modifying existing files with updated content
- When saving output data to a file for later use
- When generating files based on templates or transformations

## When Not to Use
- When the file content is sensitive (credentials, private keys)
- When writing large volumes of data that could impact system performance
- When writing binary data (this tool is meant for text content)
- When you don't have appropriate permissions to write to the specified location

## Best Practices
1. **Check file existence first**: Consider using read_file to check if the file exists before modifying
2. **Create backups**: Consider creating a backup of important files before modifying them
3. **Verify writes**: Confirm the file was written successfully and contains the expected content
4. **Handle errors**: Be prepared to handle permission errors or disk space issues
5. **Follow file format conventions**: Ensure written content adheres to the expected format for the file type

## Parameters
- `path`: The file path to write to (required)
- `contents`: The text content to write to the file (required)

## Example Usage
```
# To create a new configuration file:
write_file("/path/to/config.json", "{\n  \"setting\": \"value\"\n}")

# To update a source code file:
write_file("/path/to/src/main.rs", "fn main() {\n  println!(\"Hello, world!\");\n}")

# To create a markdown file:
write_file("/path/to/README.md", "# Project Title\n\nProject description goes here.")
```