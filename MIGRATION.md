# SQLite to CXP Migration Guide

This document explains how to migrate a ContextAI SQLite database to the CXP format.

## Overview

The migration tool converts a ContextAI SQLite database into a CXP file with the ContextAI extension. This allows you to:

- Store all your chat history, conversations, and settings in a single CXP file
- Benefit from CXP's efficient compression and deduplication
- Optionally include source files alongside your data
- Use the CXP format across different platforms

## Command Line Usage

### Basic Migration

Migrate a SQLite database to CXP format:

```bash
cxp migrate <sqlite.db> <output.cxp>
```

**Example:**
```bash
cxp migrate contextai.db my_context.cxp
```

### Migration with Source Files

Include source files from a directory:

```bash
cxp migrate <sqlite.db> <output.cxp> --files <source-directory>
```

**Example:**
```bash
cxp migrate contextai.db my_project.cxp --files /path/to/project
```

This will:
1. Migrate all database data (conversations, messages, habits, etc.)
2. Scan the source directory for text files
3. Include all text files in the CXP archive
4. Apply chunking and deduplication to the files

## What Gets Migrated

The migration tool extracts and converts the following data from the SQLite database:

### 1. Conversations and Messages
- All conversation threads
- Chat messages (both user and assistant)
- Message timestamps and metadata

**SQLite Tables:**
- `conversations` → `ContextAIExtension.conversations`
- `chat_messages` → `Conversation.messages`

### 2. User Habits and Preferences
- Preferred language
- Coding style preferences
- Custom instructions and other habits

**SQLite Table:**
- `user_habits` → `ContextAIExtension.habits`

### 3. Watched Folders
- Folder paths being monitored
- Enable/disable status
- Last scan timestamps

**SQLite Table:**
- `watched_folders` → `ContextAIExtension.watched_folders`

### 4. App Settings
- UI theme preference
- Auto-indexing settings
- Maximum context file limit

**SQLite Table:**
- `user_habits` (specific keys) → `ContextAIExtension.settings`

### 5. Custom Dictionary
- Custom terms and definitions
- User-defined vocabulary for AI context

**SQLite Table:**
- `custom_dictionary` → `ContextAIExtension.dictionary`

## CXP File Structure

After migration, your CXP file will contain:

```
output.cxp (ZIP archive)
├── manifest.msgpack                   # CXP metadata
├── file_map.msgpack                   # File-to-chunk mapping (if files included)
├── chunks/                            # Compressed file chunks (if files included)
│   ├── 0001.zst
│   ├── 0002.zst
│   └── ...
└── extensions/                        # Extension data
    └── contextai/                     # ContextAI namespace
        ├── manifest.msgpack           # Extension metadata
        ├── conversations.msgpack      # All conversations and messages
        ├── habits.msgpack             # User habits and preferences
        ├── watched_folders.msgpack    # Watched folder configuration
        ├── settings.msgpack           # App settings
        └── dictionary.msgpack         # Custom dictionary entries
```

## Verifying Migration

After migration, you can verify the data:

### 1. View CXP Information

```bash
cxp info output.cxp
```

This shows:
- File statistics
- Extensions included
- Compression ratio
- File types (if files were included)

### 2. Run Verification Script

```bash
cargo run --features contextai --example verify_migration
```

This will:
- Open the CXP file
- Extract all ContextAI extension data
- Display conversations, habits, settings, etc.
- Verify data integrity

## SQLite Database Schema Requirements

The migration tool expects the following SQLite tables:

```sql
-- Conversations
CREATE TABLE conversations (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Chat messages
CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY,
    conversation_id INTEGER,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- User habits
CREATE TABLE user_habits (
    id INTEGER PRIMARY KEY,
    habit_key TEXT UNIQUE NOT NULL,
    habit_value TEXT NOT NULL,
    confidence REAL NOT NULL,
    updated_at TEXT NOT NULL
);

-- Watched folders
CREATE TABLE watched_folders (
    id INTEGER PRIMARY KEY,
    folder_path TEXT UNIQUE NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

-- Custom dictionary
CREATE TABLE custom_dictionary (
    id INTEGER PRIMARY KEY,
    term TEXT UNIQUE NOT NULL,
    definition TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## Reading Migrated Data

### Using Rust

```rust
use cxp_core::{CxpReader, contextai::ContextAIExtension};
use std::collections::HashMap;

// Open CXP file
let reader = CxpReader::open("output.cxp")?;

// Read ContextAI extension data
let mut extension_data: HashMap<String, Vec<u8>> = HashMap::new();
for key in reader.list_extension_keys("contextai") {
    let data = reader.read_extension("contextai", &key)?;
    extension_data.insert(key, data);
}

// Reconstruct ContextAI extension
let contextai = ContextAIExtension::from_extension_data(extension_data)?;

// Access conversations
for conv in contextai.list_conversations() {
    println!("Conversation: {}", conv.title);
    for msg in &conv.messages {
        println!("  [{:9}] {}", msg.role, msg.content);
    }
}

// Access user habits
let habits = contextai.get_habits();
println!("Preferred Language: {}", habits.preferred_language);

// Access settings
let settings = contextai.get_settings();
println!("Theme: {}", settings.theme);
```

## Error Handling

The migration tool will fail with an error if:

- SQLite database file doesn't exist or is corrupted
- SQLite database doesn't have the required schema
- Output CXP file path is invalid or unwritable
- Source files directory (if specified) doesn't exist

## Performance Considerations

- **Migration speed**: Depends on database size and number of conversations
- **CXP file size**: Much smaller than original SQLite file due to compression
- **With source files**: File scanning and chunking takes additional time

Typical migration times:
- Small database (< 100 conversations): < 1 second
- Medium database (100-1000 conversations): 1-5 seconds
- Large database (> 1000 conversations): 5-30 seconds

## Example Workflow

Complete migration workflow:

```bash
# 1. Migrate SQLite database
cxp migrate contextai.db my_context.cxp

# 2. Verify migration
cxp info my_context.cxp

# 3. (Optional) Run verification script
cargo run --features contextai --example verify_migration

# 4. (Optional) Migrate with source files
cxp migrate contextai.db full_backup.cxp --files ~/projects/my_app

# 5. View backed up files
cxp list full_backup.cxp
```

## Testing

To test the migration tool:

```bash
# Run the test script (creates test database and migrates it)
./test_migration.sh

# This will:
# - Create a test SQLite database with sample data
# - Run migration
# - Show CXP file information
# - Suggest cleanup commands
```

## Troubleshooting

### Error: "Failed to open SQLite database"
- Check that the database file exists
- Verify you have read permissions
- Ensure the file is a valid SQLite database

### Error: "no such table: conversations"
- Your SQLite database doesn't have the expected schema
- Make sure you're using a ContextAI database

### Error: "Extension namespace 'contextai' not registered"
- Build the CLI with the `contextai` feature:
  ```bash
  cargo build --features contextai
  ```

### Migration completes but CXP file is very small
- This is normal if you didn't include source files
- The extension data (conversations, settings) is highly compressed
- Use `cxp info` to verify data was included

## Integration with ContextAI App

To use the migrated CXP file with the ContextAI app:

1. Migrate your SQLite database to CXP
2. Update the ContextAI app to read from CXP instead of SQLite
3. Use `CxpReader` and `ContextAIExtension` to access data
4. All existing functionality should work seamlessly

## Future Enhancements

Planned features:
- Bidirectional migration (CXP → SQLite)
- Incremental migration (sync changes)
- Migration of browser history and context log
- Compression options for migration output
