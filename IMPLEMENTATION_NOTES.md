# CXP Query Command Implementation

## Overview

Implemented the `cxp query` command for text-based searching within CXP archives.

## Files Modified

- `/Users/einarjaeger/Documents/GitHub/cpx.datei typ/cxp-cli/src/main.rs`

## Changes Made

### 1. Added Query Command to CLI

```rust
Query {
    file: PathBuf,        // CXP file to search
    term: String,         // Search term
    limit: usize,         // Max files to show (default: 10)
    context: usize,       // Lines of context (default: 2)
}
```

### 2. Implemented query_files() Function

**Key Features:**
- Case-insensitive text search
- Line-by-line matching with configurable context
- ANSI color highlighting for better readability
- Automatic binary file skipping (UTF-8 validation)
- Efficient processing (stops after limit reached)

**Output Formatting:**
- Green: File paths
- Yellow: Matching lines
- Red: Highlighted search terms
- Dim gray: Context lines

### 3. Implemented highlight_term() Helper

Highlights all occurrences of the search term within a line using ANSI escape codes.

## Usage

```bash
# Basic search
cxp query project.cxp "function"

# With options
cxp query project.cxp "TODO" --limit 5 --context 3

# Help
cxp query --help
```

## Technical Details

### Search Algorithm

1. Open CXP archive using CxpReader
2. Get sorted list of all file paths
3. For each file:
   - Read and decompress content
   - Validate UTF-8 encoding
   - Split into lines
   - Search each line (case-insensitive)
   - If matches found, display with context
4. Stop after reaching --limit files with matches

### Performance Considerations

- Files are processed sequentially (no parallel processing needed for grep)
- Only matching files are fully decompressed
- Early termination when limit is reached
- Memory efficient (processes one file at a time)

### Color Codes Used

- `\x1b[1;32m` - Bold Green (file paths)
- `\x1b[1;33m` - Bold Yellow (matching lines)
- `\x1b[1;31m` - Bold Red (highlighted terms)
- `\x1b[2m` - Dim (context lines)
- `\x1b[0m` - Reset

## Future Enhancements

Possible improvements:
1. Regex pattern support
2. Case-sensitive search option
3. Whole word matching option
4. File type filtering
5. Parallel file processing for large archives
6. Export results to file
7. Integration with embedding-based semantic search

## Testing

To test the implementation:

```bash
# Build the CLI
cd cxp-cli
cargo build --release

# Create a test CXP file
./target/release/cxp build /path/to/source test.cxp

# Query the archive
./target/release/cxp query test.cxp "search_term"

# Test with options
./target/release/cxp query test.cxp "pattern" --limit 20 --context 5
```

## Compatibility

- Works with existing CXP file format
- No changes required to cxp-core library
- Uses existing CxpReader API
- Compatible with all CXP v1.0.0 archives
