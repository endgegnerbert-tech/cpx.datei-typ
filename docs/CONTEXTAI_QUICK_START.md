# ContextAI Extension - Quick Start Guide

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
cxp-core = { version = "0.1", features = ["contextai"] }
```

## Basic Usage

```rust
use cxp_core::contextai::*;

// Create extension
let mut ext = ContextAIExtension::new();
```

## Common Operations

### Create a Conversation

```rust
let conv = Conversation {
    id: "conv-1".to_string(),
    title: "My Chat".to_string(),
    created_at: "2025-01-15T10:00:00Z".to_string(),
    updated_at: "2025-01-15T10:00:00Z".to_string(),
    messages: vec![],
};
ext.add_conversation(conv);
```

### Add a Message

```rust
let msg = ChatMessage {
    id: "msg-1".to_string(),
    role: "user".to_string(),
    content: "Hello!".to_string(),
    timestamp: "2025-01-15T10:01:00Z".to_string(),
};
ext.add_message("conv-1", msg)?;
```

### Set User Preferences

```rust
ext.set_habits(UserHabits {
    preferred_language: "en".to_string(),
    coding_style: Some("2-space-indent".to_string()),
    custom_instructions: vec!["Be concise".to_string()],
});
```

### Configure Settings

```rust
ext.set_settings(AppSettings {
    theme: "dark".to_string(),
    auto_index: true,
    max_context_files: 50,
});
```

### Add Watched Folder

```rust
ext.add_watched_folder(WatchedFolder {
    path: "/path/to/folder".to_string(),
    enabled: true,
    last_scan: None,
});
```

### Manage Dictionary

```rust
ext.add_to_dictionary("TypeScript".to_string());
ext.add_to_dictionary("React".to_string());

// Remove a word
ext.remove_from_dictionary("React")?;
```

## Serialization

### Save to CXP File

```rust
// Serialize to HashMap<String, Vec<u8>>
let data = ext.to_extension_data()?;

// Store in CXP file's extensions/ directory
// (Integration with CxpBuilder required)
```

### Load from CXP File

```rust
// Read from CXP file's extensions/ directory
// (Integration with CxpReader required)
let data = read_extension_data()?;

// Deserialize
let ext = ContextAIExtension::from_extension_data(data)?;
```

### Debug with JSON

```rust
// Export to JSON
let json = ext.to_json()?;
println!("{}", json);

// Import from JSON
let ext = ContextAIExtension::from_json(&json)?;
```

## Data Access

### Read Conversations

```rust
// Get all conversations
for conv in ext.list_conversations() {
    println!("{}: {}", conv.id, conv.title);
    for msg in &conv.messages {
        println!("  {}: {}", msg.role, msg.content);
    }
}

// Get specific conversation
if let Some(conv) = ext.get_conversation("conv-1") {
    println!("{}", conv.title);
}
```

### Read Settings

```rust
let settings = ext.get_settings();
println!("Theme: {}", settings.theme);

let habits = ext.get_habits();
println!("Language: {}", habits.preferred_language);
```

### Read Watched Folders

```rust
for folder in ext.get_watched_folders() {
    println!("{} (enabled: {})", folder.path, folder.enabled);
}
```

## Error Handling

```rust
use cxp_core::Result;

fn my_function(ext: &mut ContextAIExtension) -> Result<()> {
    // Operations that return Result<()>
    ext.add_message("conv-1", message)?;
    ext.delete_conversation("conv-2")?;
    ext.remove_watched_folder("/path")?;

    Ok(())
}
```

## Storage Format

Data is stored in MessagePack format:

- `conversations.msgpack` - All conversations with messages
- `habits.msgpack` - User habits and preferences
- `settings.msgpack` - App settings
- `watched_folders.msgpack` - Watched folders list
- `dictionary.msgpack` - Custom dictionary

## Examples

Run the example:

```bash
cargo run --example contextai_example --features contextai
```

## Testing

Run tests:

```bash
# Unit tests
cargo test --features contextai contextai

# Integration tests
cargo test --features contextai --test contextai_integration_test
```

## Defaults

Default values when creating a new extension:

- **Language:** `en`
- **Theme:** `auto`
- **Auto-index:** `true`
- **Max context files:** `50`
- **Conversations:** Empty
- **Watched folders:** Empty
- **Dictionary:** Empty

## Common Patterns

### Update Conversation Title

```rust
if let Some(conv) = ext.get_conversation_mut("conv-1") {
    conv.title = "New Title".to_string();
    conv.updated_at = current_timestamp();
}
```

### Toggle Folder Watching

```rust
for folder in ext.get_watched_folders_mut() {
    if folder.path == "/specific/path" {
        folder.enabled = !folder.enabled;
    }
}
```

### Add Custom Instruction

```rust
let habits = ext.get_habits_mut();
habits.custom_instructions.push("New instruction".to_string());
```

## Performance Tips

1. Use `get_conversation_mut()` for in-place updates
2. Batch multiple operations before serialization
3. For large conversations, consider pagination
4. Use `Vec::with_capacity()` for known sizes

## Documentation

See full documentation: `docs/CONTEXTAI_EXTENSION.md`

## Support

For issues or questions, please refer to the main CXP documentation.
