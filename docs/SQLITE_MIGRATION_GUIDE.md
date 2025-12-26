# SQLite to ContextAI Extension Migration Guide

This guide helps you migrate from SQLite storage to the ContextAI CXP extension.

## Why Migrate?

**Benefits of CXP Extension:**
- Single file storage (no separate .db files)
- Better compression (MessagePack + Zstandard)
- Version control friendly
- Type-safe at compile time
- Cross-platform without database drivers
- Portable and self-contained

## Migration Overview

The migration process involves:
1. Reading data from SQLite
2. Converting to ContextAI data structures
3. Serializing to CXP format
4. Storing in CXP file

## SQLite Schema Mapping

### Conversations Table

**SQLite Schema:**
```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

**Maps to:**
```rust
Conversation {
    id: String,
    title: String,
    created_at: String,
    updated_at: String,
    messages: Vec<ChatMessage>,
}
```

### Messages Table

**SQLite Schema:**
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
```

**Maps to:**
```rust
ChatMessage {
    id: String,
    role: String,
    content: String,
    timestamp: String,
}
```

### User Preferences Table

**SQLite Schema:**
```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**Maps to:**
```rust
UserHabits {
    preferred_language: String,
    coding_style: Option<String>,
    custom_instructions: Vec<String>,
}
```

### Settings Table

**SQLite Schema:**
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**Maps to:**
```rust
AppSettings {
    theme: String,
    auto_index: bool,
    max_context_files: u32,
}
```

### Watched Folders Table

**SQLite Schema:**
```sql
CREATE TABLE watched_folders (
    path TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL,
    last_scan TEXT
);
```

**Maps to:**
```rust
WatchedFolder {
    path: String,
    enabled: bool,
    last_scan: Option<String>,
}
```

### Dictionary Table

**SQLite Schema:**
```sql
CREATE TABLE dictionary (
    word TEXT PRIMARY KEY
);
```

**Maps to:**
```rust
Vec<String>  // in ContextAIExtension.dictionary
```

## Migration Code

### Complete Migration Function

```rust
use rusqlite::{Connection, Result as SqlResult};
use cxp_core::contextai::*;
use std::collections::HashMap;

fn migrate_from_sqlite(db_path: &str) -> cxp_core::Result<ContextAIExtension> {
    let conn = Connection::open(db_path)
        .map_err(|e| cxp_core::CxpError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        ))?;

    let mut ext = ContextAIExtension::new();

    // 1. Migrate conversations and messages
    migrate_conversations(&conn, &mut ext)?;

    // 2. Migrate user preferences
    migrate_preferences(&conn, &mut ext)?;

    // 3. Migrate settings
    migrate_settings(&conn, &mut ext)?;

    // 4. Migrate watched folders
    migrate_watched_folders(&conn, &mut ext)?;

    // 5. Migrate dictionary
    migrate_dictionary(&conn, &mut ext)?;

    Ok(ext)
}

fn migrate_conversations(
    conn: &Connection,
    ext: &mut ContextAIExtension
) -> cxp_core::Result<()> {
    // Read all conversations
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations"
    ).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let conversations = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,  // id
            row.get::<_, String>(1)?,  // title
            row.get::<_, String>(2)?,  // created_at
            row.get::<_, String>(3)?,  // updated_at
        ))
    }).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    for conv_result in conversations {
        let (id, title, created_at, updated_at) = conv_result.map_err(|e| {
            cxp_core::CxpError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )
        })?;

        // Read messages for this conversation
        let mut msg_stmt = conn.prepare(
            "SELECT id, role, content, timestamp FROM messages WHERE conversation_id = ?1"
        ).map_err(|e| cxp_core::CxpError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        ))?;

        let messages = msg_stmt.query_map([&id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
            })
        }).map_err(|e| cxp_core::CxpError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        ))?;

        let messages: Vec<ChatMessage> = messages
            .filter_map(|m| m.ok())
            .collect();

        ext.add_conversation(Conversation {
            id,
            title,
            created_at,
            updated_at,
            messages,
        });
    }

    Ok(())
}

fn migrate_preferences(
    conn: &Connection,
    ext: &mut ContextAIExtension
) -> cxp_core::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT key, value FROM user_preferences"
    ).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let prefs = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let mut pref_map: HashMap<String, String> = HashMap::new();
    for pref_result in prefs {
        let (key, value) = pref_result.map_err(|e| {
            cxp_core::CxpError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )
        })?;
        pref_map.insert(key, value);
    }

    let habits = UserHabits {
        preferred_language: pref_map
            .get("preferred_language")
            .cloned()
            .unwrap_or_else(|| "en".to_string()),
        coding_style: pref_map.get("coding_style").cloned(),
        custom_instructions: pref_map
            .get("custom_instructions")
            .map(|s| s.split('\n').map(|s| s.to_string()).collect())
            .unwrap_or_default(),
    };

    ext.set_habits(habits);
    Ok(())
}

fn migrate_settings(
    conn: &Connection,
    ext: &mut ContextAIExtension
) -> cxp_core::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT key, value FROM settings"
    ).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let settings_rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let mut settings_map: HashMap<String, String> = HashMap::new();
    for setting_result in settings_rows {
        let (key, value) = setting_result.map_err(|e| {
            cxp_core::CxpError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )
        })?;
        settings_map.insert(key, value);
    }

    let settings = AppSettings {
        theme: settings_map
            .get("theme")
            .cloned()
            .unwrap_or_else(|| "auto".to_string()),
        auto_index: settings_map
            .get("auto_index")
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        max_context_files: settings_map
            .get("max_context_files")
            .and_then(|s| s.parse().ok())
            .unwrap_or(50),
    };

    ext.set_settings(settings);
    Ok(())
}

fn migrate_watched_folders(
    conn: &Connection,
    ext: &mut ContextAIExtension
) -> cxp_core::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT path, enabled, last_scan FROM watched_folders"
    ).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let folders = stmt.query_map([], |row| {
        Ok(WatchedFolder {
            path: row.get(0)?,
            enabled: row.get::<_, i32>(1)? != 0,
            last_scan: row.get(2).ok(),
        })
    }).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    for folder_result in folders {
        let folder = folder_result.map_err(|e| {
            cxp_core::CxpError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )
        })?;
        ext.add_watched_folder(folder);
    }

    Ok(())
}

fn migrate_dictionary(
    conn: &Connection,
    ext: &mut ContextAIExtension
) -> cxp_core::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT word FROM dictionary"
    ).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    let words = stmt.query_map([], |row| {
        row.get::<_, String>(0)
    }).map_err(|e| cxp_core::CxpError::Io(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;

    for word_result in words {
        let word = word_result.map_err(|e| {
            cxp_core::CxpError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            )
        })?;
        ext.add_to_dictionary(word);
    }

    Ok(())
}
```

## Usage

```rust
// Migrate from SQLite
let ext = migrate_from_sqlite("path/to/database.db")?;

// Save to CXP file
let extension_data = ext.to_extension_data()?;
// ... store in CXP file ...
```

## Migration Checklist

- [ ] Backup your SQLite database
- [ ] Install `rusqlite` dependency for migration
- [ ] Run migration code
- [ ] Verify data integrity
- [ ] Test all features with new storage
- [ ] Remove SQLite dependency
- [ ] Delete old database file (after verification)

## Testing Migration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration() {
        // Migrate
        let ext = migrate_from_sqlite("test.db").unwrap();

        // Verify conversations
        assert_eq!(ext.list_conversations().len(), expected_count);

        // Verify messages
        let conv = ext.get_conversation("conv-id").unwrap();
        assert_eq!(conv.messages.len(), expected_messages);

        // Verify settings
        assert_eq!(ext.get_settings().theme, expected_theme);

        // ... more verifications ...
    }
}
```

## Rollback Plan

If migration fails or you need to rollback:

1. Keep your original SQLite database
2. Export CXP data to JSON for inspection:
   ```rust
   let json = ext.to_json()?;
   std::fs::write("backup.json", json)?;
   ```
3. If needed, you can convert JSON back to SQLite

## Performance Comparison

**SQLite:**
- Requires database driver
- Separate .db file
- SQL query overhead
- Index maintenance

**CXP Extension:**
- No external dependencies
- Single file
- Direct memory access
- Efficient MessagePack serialization

**Storage Size Comparison** (approximate):
```
SQLite:           ~100 KB (with indexes)
CXP MessagePack:  ~30-40 KB (60-70% smaller)
```

## Troubleshooting

### Issue: Missing Data After Migration

Check the migration logs and verify:
```rust
println!("Migrated {} conversations", ext.list_conversations().len());
println!("Migrated {} folders", ext.get_watched_folders().len());
println!("Migrated {} dictionary words", ext.get_dictionary().len());
```

### Issue: Invalid Timestamps

Ensure SQLite timestamps are in ISO 8601 format:
```sql
-- Convert if needed
UPDATE conversations
SET created_at = datetime(created_at, 'unixepoch')
WHERE created_at NOT LIKE '%-%';
```

### Issue: Large Database Performance

For databases with >10,000 conversations, consider:
1. Migrating in batches
2. Using parallel processing
3. Optimizing SQL queries with indexes

## Post-Migration

After successful migration:

1. **Verify Functionality:**
   - Test all app features
   - Verify data integrity
   - Check performance

2. **Clean Up:**
   ```bash
   # Remove SQLite dependency from Cargo.toml
   # Delete old database file
   rm database.db
   ```

3. **Update Code:**
   - Replace SQLite queries with ContextAI API
   - Remove database connection code
   - Update initialization logic

## Example: Before/After Code

### Before (SQLite)

```rust
// Open database
let conn = Connection::open("app.db")?;

// Query conversations
let mut stmt = conn.prepare(
    "SELECT id, title FROM conversations WHERE user_id = ?"
)?;
let conversations = stmt.query_map([user_id], |row| {
    Ok((row.get(0)?, row.get(1)?))
})?;
```

### After (ContextAI)

```rust
// Open CXP file
let reader = CxpReader::open("app.cxp")?;
let ext_data = reader.read_extension("contextai")?;
let ext = ContextAIExtension::from_extension_data(ext_data)?;

// Get conversations
for conv in ext.list_conversations() {
    println!("{}: {}", conv.id, conv.title);
}
```

## Support

For migration issues, please check:
- This migration guide
- Main ContextAI documentation
- Example code in `examples/contextai_example.rs`

## Next Steps

After migration:
1. Read the [Quick Start Guide](CONTEXTAI_QUICK_START.md)
2. Review the [Full Documentation](CONTEXTAI_EXTENSION.md)
3. Run the example: `cargo run --example contextai_example --features contextai`
