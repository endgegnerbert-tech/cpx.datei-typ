# CXP Query - Quick Start Guide

## Installation

```bash
cd cxp-cli
cargo build --release
```

The binary will be at: `./target/release/cxp`

## Basic Usage

### 1. Create a CXP file (if you don't have one)

```bash
cxp build /path/to/your/project project.cxp
```

### 2. Search for a term

```bash
cxp query project.cxp "search_term"
```

## Common Examples

### Find all TODO comments
```bash
cxp query project.cxp "TODO"
```

### Search for function definitions
```bash
cxp query project.cxp "fn " --limit 20
```

### Find error handling with more context
```bash
cxp query project.cxp "Result<" --context 5
```

### Search for a struct definition
```bash
cxp query project.cxp "struct CxpReader"
```

### Find imports
```bash
cxp query project.cxp "use std::" --limit 30
```

## Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--limit` | `-l` | 10 | Max number of files to show |
| `--context` | `-c` | 2 | Lines of context around matches |
| `--verbose` | `-v` | false | Show detailed output |
| `--help` | `-h` | - | Show help |

## Understanding the Output

```
src/format.rs (3 matches)
--------------------------------------------------------------------------------
 245 | /// Reader for CXP files
 246 | pub struct CxpReader {
 247 |     /// The manifest
    ...
```

- **Green text**: File path
- **Yellow text**: Matching line
- **Red text**: The search term itself
- **Gray text**: Context lines
- **Number**: Line number

## Tips

1. **Use quotes** for multi-word searches:
   ```bash
   cxp query project.cxp "async fn"
   ```

2. **Increase limit** for comprehensive searches:
   ```bash
   cxp query project.cxp "error" --limit 50
   ```

3. **More context** for understanding:
   ```bash
   cxp query project.cxp "impl" --context 10
   ```

4. **Combine options** for best results:
   ```bash
   cxp query project.cxp "test" --limit 15 --context 3
   ```

## Common Searches

### Development
```bash
# Find tests
cxp query project.cxp "#[test]"

# Find async functions
cxp query project.cxp "async fn"

# Find panics
cxp query project.cxp "panic!"

# Find unwraps (potential errors)
cxp query project.cxp ".unwrap()"
```

### Documentation
```bash
# Find TODOs
cxp query project.cxp "TODO"

# Find FIXMEs
cxp query project.cxp "FIXME"

# Find doc comments
cxp query project.cxp "///"
```

### Code Review
```bash
# Find unsafe code
cxp query project.cxp "unsafe"

# Find commented code
cxp query project.cxp "//"

# Find hardcoded values
cxp query project.cxp "TODO: replace"
```

## Troubleshooting

### No matches found
- Check spelling (search is case-insensitive)
- Try a shorter/broader search term
- Verify the CXP file contains the expected files

### Binary file errors
- Binary files are automatically skipped
- No action needed

### Too many results
- Use `--limit` to see more files:
  ```bash
  cxp query project.cxp "fn" --limit 100
  ```

### Not enough context
- Use `--context` for more lines:
  ```bash
  cxp query project.cxp "struct" --context 10
  ```

## Next Steps

- Read the full documentation: `QUERY_USAGE.md`
- View implementation details: `IMPLEMENTATION_NOTES.md`
- See all CLI commands: `cxp --help`

## Integration with Workflow

### VS Code Integration
Add to `.vscode/tasks.json`:
```json
{
  "label": "Search CXP",
  "type": "shell",
  "command": "cxp query project.cxp \"${input:searchTerm}\"",
  "presentation": {
    "reveal": "always",
    "panel": "new"
  }
}
```

### Shell Alias
Add to `.bashrc` or `.zshrc`:
```bash
alias cxpq='cxp query'
```

Then use:
```bash
cxpq project.cxp "search term"
```

### Git Hook
Search for TODOs before commit:
```bash
#!/bin/bash
# .git/hooks/pre-commit
cxp build . .git/temp.cxp
cxp query .git/temp.cxp "TODO" --limit 100
```

## Performance Notes

- Sequential file processing (one file at a time)
- Fast for typical projects (< 1000 files)
- Efficient decompression (only matching files)
- Low memory usage

## Limitations

- No regex support (simple text matching only)
- Always case-insensitive
- No AND/OR operators
- No file type filtering in query (use `cxp list` first)

For advanced search needs, consider the embedding-based semantic search feature (separate implementation).
