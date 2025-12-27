# CXP Migration Tool

A command-line tool for migrating SQLite databases to the CXP (Universal AI Context Format) file format.

## Quick Start

```bash
# Build the CLI with contextai feature (default)
cargo build --release --package cxp-cli

# Migrate a SQLite database
./target/release/cxp migrate database.db output.cxp

# Migrate with source files
./target/release/cxp migrate database.db output.cxp --files /path/to/source
```

## Features

- **Complete Data Migration**: Migrates all tables from ContextAI SQLite schema
- **Efficient Storage**: Uses CXP's compression and deduplication
- **Optional File Inclusion**: Include source code files alongside database data
- **Extension System**: Stores data in standardized ContextAI extension format
- **Data Verification**: Built-in verification scripts to check migration integrity

## What Gets Migrated

### Database Tables → CXP Extension

| SQLite Table | CXP Extension Component | Description |
|--------------|------------------------|-------------|
| `conversations` | `conversations.msgpack` | Chat conversation threads |
| `chat_messages` | `conversations.msgpack` | Individual chat messages |
| `user_habits` | `habits.msgpack` | User preferences and habits |
| `watched_folders` | `watched_folders.msgpack` | Monitored directories |
| `custom_dictionary` | `dictionary.msgpack` | Custom terms and definitions |
| `user_habits` (settings) | `settings.msgpack` | Application settings |

## Implementation Details

### Code Structure

```
cxp-cli/
├── src/
│   ├── main.rs          # CLI entry point with migrate command
│   └── migrate.rs       # Migration logic
└── Cargo.toml           # Dependencies (includes rusqlite)

cxp-core/
├── src/
│   ├── contextai.rs     # ContextAI extension data structures
│   └── extensions.rs    # Extension system
└── examples/
    └── verify_migration.rs  # Migration verification script
```

### Migration Function

The main migration function signature:

```rust
pub fn migrate_sqlite_to_cxp(
    sqlite_path: &Path,      // Input SQLite database
    output_cxp: &Path,       // Output CXP file
    source_files_dir: Option<&Path>,  // Optional source files
) -> Result<()>
```

### Process Flow

1. **Open SQLite Connection**
   - Validates database exists and is readable
   - Connects using rusqlite

2. **Extract Data**
   - Read conversations with messages
   - Read user habits and preferences
   - Read watched folders
   - Read app settings
   - Read custom dictionary

3. **Create ContextAI Extension**
   - Instantiate `ContextAIExtension`
   - Populate with extracted data
   - Serialize to MessagePack format

4. **Build CXP File**
   - Create `CxpBuilder`
   - Optionally scan and process source files
   - Add ContextAI extension data
   - Write compressed CXP archive

5. **Finalize**
   - Close database connection
   - Report statistics
   - Verify output file

## Dependencies

### Required Crates

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
cxp-core = { path = "../cxp-core" }
anyhow = "1.0"
tracing = "0.1"
```

### Features

```toml
[features]
default = ["contextai"]
contextai = ["cxp-core/contextai"]
```

## Usage Examples

### Basic Migration

```bash
cxp migrate contextai.db my_context.cxp
```

Output:
```
Starting SQLite to CXP migration...
  SQLite DB: contextai.db
  Output CXP: my_context.cxp
Migrating conversations and messages...
  Migrated 50 conversations with 234 messages
Migrating user habits...
  Migrated 5 user habits
Migrating watched folders...
  Migrated 3 watched folders
Migration completed successfully!
```

### Migration with Source Files

```bash
cxp migrate contextai.db full_backup.cxp --files ~/projects/my_app
```

This will:
1. Migrate all database data
2. Scan `~/projects/my_app` for text files
3. Include files with chunking and deduplication
4. Create a comprehensive backup

### View Migrated Data

```bash
# Show CXP file info
cxp info my_context.cxp

# List extensions
cxp info my_context.cxp | grep Extensions

# Run verification (from project root)
cargo run --features contextai --example verify_migration
```

## Testing

### Run Test Suite

```bash
# Run migration test script
./test_migration.sh

# Manual test
sqlite3 test.db < test_schema.sql
cxp migrate test.db output.cxp
cxp info output.cxp
```

### Verification Script

The verification script (`verify_migration.rs`) demonstrates how to:
- Open a CXP file
- Read ContextAI extension data
- Reconstruct the extension object
- Access all migrated data

Example output:
```
Opening CXP file: test_output.cxp

Extensions found: ["contextai"]

=== Conversations ===
Total: 2

Conversation ID: 1
  Title: First Conversation
  Messages: 4
    [user     ] Hello, how are you?
    [assistant] I am doing well, thank you!

=== User Habits ===
Preferred Language: en
Coding Style: Some("4-space-indent")

✅ Verification completed successfully!
```

## Error Handling

Common errors and solutions:

### Database Not Found
```
Error: Failed to open SQLite database
```
**Solution**: Check file path and permissions

### Invalid Schema
```
Error: no such table: conversations
```
**Solution**: Ensure database has ContextAI schema

### Feature Not Enabled
```
Error: unresolved import `cxp_core::ContextAIExtension`
```
**Solution**: Build with `--features contextai`

### Output File Exists
The tool will overwrite existing output files. Use caution or implement a check:
```bash
[ -f output.cxp ] && echo "File exists!" || cxp migrate db.db output.cxp
```

## Performance

### Benchmarks

| Database Size | Conversations | Messages | Migration Time | CXP Size |
|---------------|---------------|----------|----------------|----------|
| Small | 10 | 50 | < 1s | ~5 KB |
| Medium | 100 | 500 | 1-2s | ~50 KB |
| Large | 1000 | 5000 | 5-10s | ~500 KB |

*Note: Times measured on M1 Mac. Actual performance varies by hardware.*

### Optimization Tips

1. **Batch Processing**: Migration processes all data in a single pass
2. **Streaming**: Large tables are processed row-by-row
3. **Compression**: CXP uses zstd compression (typically 70-90% size reduction)
4. **Deduplication**: Source files benefit from chunk-level dedup

## API Reference

### Core Functions

```rust
/// Main migration function
pub fn migrate_sqlite_to_cxp(
    sqlite_path: &Path,
    output_cxp: &Path,
    source_files_dir: Option<&Path>,
) -> Result<()>

/// Migrate conversations and messages
fn migrate_conversations(
    conn: &Connection,
    contextai: &mut ContextAIExtension
) -> Result<()>

/// Migrate user habits and preferences
fn migrate_user_habits(
    conn: &Connection,
    contextai: &mut ContextAIExtension
) -> Result<()>

/// Migrate watched folders
fn migrate_watched_folders(
    conn: &Connection,
    contextai: &mut ContextAIExtension
) -> Result<()>

/// Migrate app settings
fn migrate_app_settings(
    conn: &Connection,
    contextai: &mut ContextAIExtension
) -> Result<()>

/// Migrate custom dictionary
fn migrate_custom_dictionary(
    conn: &Connection,
    contextai: &mut ContextAIExtension
) -> Result<()>
```

## Integration with ContextAI App

To integrate with the ContextAI application:

1. **Replace SQLite Backend**:
   ```rust
   // Old: Direct SQLite access
   let conn = Connection::open("contextai.db")?;

   // New: Read from CXP
   let reader = CxpReader::open("contextai.cxp")?;
   let mut ext_data = HashMap::new();
   for key in reader.list_extension_keys("contextai") {
       ext_data.insert(key, reader.read_extension("contextai", &key)?);
   }
   let contextai = ContextAIExtension::from_extension_data(ext_data)?;
   ```

2. **Update Data Access**:
   ```rust
   // Access conversations
   let conversations = contextai.list_conversations();

   // Access user habits
   let habits = contextai.get_habits();

   // Access settings
   let settings = contextai.get_settings();
   ```

3. **Write Changes** (future feature):
   ```rust
   // Add new conversation
   contextai.add_conversation(new_conversation);

   // Save back to CXP
   let updated_data = contextai.to_extension_data()?;
   // Write logic TBD
   ```

## Contributing

When adding migration features:

1. Update SQLite schema mapping in `migrate.rs`
2. Add corresponding fields to ContextAI extension structures
3. Update tests in `test_migration.sh`
4. Update documentation

## License

Part of the CXP project - see project LICENSE file.

## See Also

- [MIGRATION.md](../MIGRATION.md) - Detailed migration guide
- [cxp-core/src/contextai.rs](../cxp-core/src/contextai.rs) - Extension implementation
- [test_migration.sh](../test_migration.sh) - Test script
