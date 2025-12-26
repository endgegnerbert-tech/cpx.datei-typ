# CXP Query Command

The `cxp query` command allows you to search through files in a CXP archive using simple text-based search.

## Usage

```bash
cxp query <file.cxp> "search term" [OPTIONS]
```

## Options

- `--limit N` - Maximum number of files to show with matches (default: 10)
- `--context N` - Number of lines of context to show around each match (default: 2)
- `--verbose` - Show detailed output during search

## Examples

### Basic search

Search for the term "function" in all files:

```bash
cxp query project.cxp "function"
```

### Limit results

Show only the first 5 files with matches:

```bash
cxp query project.cxp "TODO" --limit 5
```

### More context

Show 5 lines of context around each match:

```bash
cxp query project.cxp "error" --context 5
```

### Combined options

```bash
cxp query project.cxp "async fn" --limit 20 --context 3
```

## Output Format

The query command displays:

1. **File header** - Shows the file path and number of matches in green
2. **Matching lines** - Highlighted in yellow with the search term in red
3. **Context lines** - Shown in dim gray above and below matches
4. **Summary** - Total matches and files found

Example output:

```
Searching for 'CxpReader' in 15 files...

src/format.rs (3 matches)
--------------------------------------------------------------------------------
 245 | /// Reader for CXP files
 246 | pub struct CxpReader {
 247 |     /// The manifest
    ...
 255 |
 256 | impl CxpReader {
 257 |     /// Open a CXP file for reading

Summary: 3 matches in 1 files
```

## Features

- **Case-insensitive search** - Searches are case-insensitive by default
- **UTF-8 text files only** - Binary files are automatically skipped
- **ANSI color highlighting** - Makes results easy to scan
- **Efficient** - Only decompresses files as needed from the CXP archive

## Implementation Notes

This is a simple text-based search implementation. For semantic search using embeddings, see the full CXP documentation.

The query engine:
1. Opens the CXP archive and reads the file map
2. Iterates through files in sorted order
3. For each file, reads and decompresses its content
4. Searches line-by-line for matches (case-insensitive)
5. Displays matches with configurable context
6. Stops after finding matches in N files (based on --limit)
