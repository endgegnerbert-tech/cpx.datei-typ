# ContextAI Extension for CXP

The ContextAI extension enables storing all application data for the ContextAI app directly in CXP files, replacing the need for SQLite databases.

## Overview

Instead of maintaining a separate SQLite database, ContextAI can store all its data (conversations, user habits, settings, watched folders, etc.) in the `extensions/` directory of a CXP file.

## Features

The ContextAI extension provides:

1. **Conversation Management** - Store chat conversations with full message history
2. **User Habits** - Save user preferences, coding styles, and custom instructions
3. **App Settings** - Persist UI theme, auto-index settings, and context limits
4. **Watched Folders** - Track folders to automatically index
5. **Dictionary** - Custom vocabulary/terms for the user

## Usage

### Enabling the Feature

Add to your `Cargo.toml`:

```toml
[dependencies]
cxp-core = { version = "0.1", features = ["contextai"] }
```

### Basic Example

```rust
use cxp_core::contextai::{
    ContextAIExtension, Conversation, ChatMessage, UserHabits,
    WatchedFolder, AppSettings,
};

// Create a new extension
let mut ext = ContextAIExtension::new();

// Add a conversation
let conv = Conversation {
    id: "conv-123".to_string(),
    title: "My Conversation".to_string(),
    created_at: "2025-01-15T10:00:00Z".to_string(),
    updated_at: "2025-01-15T10:00:00Z".to_string(),
    messages: vec![],
};
ext.add_conversation(conv);

// Add a message
let message = ChatMessage {
    id: "msg-1".to_string(),
    role: "user".to_string(),
    content: "Hello!".to_string(),
    timestamp: "2025-01-15T10:01:00Z".to_string(),
};
ext.add_message("conv-123", message).unwrap();

// Set user habits
let habits = UserHabits {
    preferred_language: "en".to_string(),
    coding_style: Some("2-space-indent".to_string()),
    custom_instructions: vec!["Be concise".to_string()],
};
ext.set_habits(habits);

// Configure settings
let settings = AppSettings {
    theme: "dark".to_string(),
    auto_index: true,
    max_context_files: 50,
};
ext.set_settings(settings);
```

## Data Structures

### Conversation

Represents a chat conversation with messages:

```rust
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
}
```

### ChatMessage

A single message in a conversation:

```rust
pub struct ChatMessage {
    pub id: String,
    pub role: String,  // "user" or "assistant"
    pub content: String,
    pub timestamp: String,
}
```

### UserHabits

User preferences and coding style:

```rust
pub struct UserHabits {
    pub preferred_language: String,
    pub coding_style: Option<String>,
    pub custom_instructions: Vec<String>,
}
```

### WatchedFolder

A folder to automatically index:

```rust
pub struct WatchedFolder {
    pub path: String,
    pub enabled: bool,
    pub last_scan: Option<String>,
}
```

### AppSettings

Application settings:

```rust
pub struct AppSettings {
    pub theme: String,
    pub auto_index: bool,
    pub max_context_files: u32,
}
```

## Serialization

The extension can be serialized to/from the CXP file format:

```rust
// Serialize to extension data for storage in CXP
let extension_data = ext.to_extension_data()?;

// This creates a HashMap with the following files:
// - conversations.msgpack
// - habits.msgpack
// - watched_folders.msgpack
// - settings.msgpack
// - dictionary.msgpack

// Deserialize from extension data
let restored = ContextAIExtension::from_extension_data(extension_data)?;
```

### JSON Export (for debugging)

```rust
// Export to JSON for human reading
let json = ext.to_json()?;
println!("{}", json);

// Import from JSON
let restored = ContextAIExtension::from_json(&json)?;
```

## Storage in CXP Files

When stored in a CXP file, the structure looks like:

```
my-project.cxp (ZIP)
├── manifest.msgpack
├── file_map.msgpack
├── chunks/
│   └── ...
└── extensions/
    └── contextai/
        ├── conversations.msgpack
        ├── habits.msgpack
        ├── watched_folders.msgpack
        ├── settings.msgpack
        └── dictionary.msgpack
```

## API Reference

### Conversation Management

```rust
// Add a conversation
ext.add_conversation(conv: Conversation);

// Get a conversation by ID
let conv = ext.get_conversation(id: &str) -> Option<&Conversation>;

// List all conversations
let convs = ext.list_conversations() -> &[Conversation];

// Update a conversation
ext.update_conversation(id: &str, updated: Conversation) -> Result<()>;

// Delete a conversation
ext.delete_conversation(id: &str) -> Result<()>;

// Add a message to a conversation
ext.add_message(conversation_id: &str, message: ChatMessage) -> Result<()>;
```

### Habits Management

```rust
// Set user habits
ext.set_habits(habits: UserHabits);

// Get user habits
let habits = ext.get_habits() -> &UserHabits;

// Get mutable habits
let habits = ext.get_habits_mut() -> &mut UserHabits;
```

### Settings Management

```rust
// Set settings
ext.set_settings(settings: AppSettings);

// Get settings
let settings = ext.get_settings() -> &AppSettings;

// Get mutable settings
let settings = ext.get_settings_mut() -> &mut AppSettings;
```

### Watched Folders Management

```rust
// Add a watched folder
ext.add_watched_folder(folder: WatchedFolder);

// Get all watched folders
let folders = ext.get_watched_folders() -> &[WatchedFolder];

// Get mutable watched folders
let folders = ext.get_watched_folders_mut() -> &mut Vec<WatchedFolder>;

// Remove a watched folder by path
ext.remove_watched_folder(path: &str) -> Result<()>;
```

### Dictionary Management

```rust
// Add a word to the dictionary
ext.add_to_dictionary(word: String);

// Get the dictionary
let words = ext.get_dictionary() -> &[String];

// Remove a word from the dictionary
ext.remove_from_dictionary(word: &str) -> Result<()>;
```

## Integration with CXP Files

To integrate ContextAI extension with CXP file reading/writing, you would extend the `CxpBuilder` and `CxpReader` to handle extensions:

```rust
// When building a CXP file
let mut builder = CxpBuilder::new(source_dir);
builder.scan()?.process()?;

// Add ContextAI extension
let mut ext = ContextAIExtension::new();
// ... configure extension ...

let extension_data = ext.to_extension_data()?;
builder.add_extension("contextai", extension_data)?;

builder.build("output.cxp")?;

// When reading a CXP file
let reader = CxpReader::open("output.cxp")?;

// Read ContextAI extension
if let Some(ext_data) = reader.read_extension("contextai")? {
    let ext = ContextAIExtension::from_extension_data(ext_data)?;
    // ... use extension ...
}
```

## Migration from SQLite

If you have existing data in SQLite, you can migrate it:

1. Read data from SQLite tables
2. Convert to ContextAI data structures
3. Serialize to CXP format

Example:

```rust
// Pseudocode for migration
let conversations = read_conversations_from_sqlite()?;
let mut ext = ContextAIExtension::new();

for conv in conversations {
    ext.add_conversation(conv);
}

// Save to CXP
let extension_data = ext.to_extension_data()?;
// ... store in CXP file ...
```

## Benefits

1. **Single File Storage** - No separate database files to manage
2. **Version Control Friendly** - CXP files can be versioned with Git
3. **Portable** - Everything in one file, easy to backup/share
4. **Compression** - MessagePack + Zstandard for efficient storage
5. **Structured** - Clear separation of concerns in extensions/ directory

## Testing

Run the tests with:

```bash
cargo test --features contextai contextai
```

Run the example:

```bash
cargo run --example contextai_example --features contextai
```

## Performance Considerations

- MessagePack serialization is fast and compact
- All data is loaded into memory when reading
- For large conversation histories, consider pagination or lazy loading
- The dictionary uses a simple Vec, consider a HashSet for large dictionaries

## Future Enhancements

Potential future improvements:

1. **Incremental Updates** - Update single conversations without rewriting entire file
2. **Compression** - Compress old conversations separately
3. **Search Index** - Add full-text search for conversations
4. **Encryption** - Optional encryption for sensitive conversations
5. **Attachments** - Store file attachments in conversations

## License

Same as the CXP project.
