# CXP Extension System

The CXP extension system allows applications to store custom, app-specific data alongside the indexed source code in CXP files.

## Overview

Extensions are stored in namespaced directories within the CXP file:

```
file.cxp (ZIP)
├── manifest.msgpack
├── file_map.msgpack
├── chunks/
│   └── ...
└── extensions/              # Extension data
    ├── contextai/           # Namespace for ContextAI app
    │   ├── manifest.msgpack # Extension metadata
    │   ├── conversations.msgpack
    │   ├── habits.msgpack
    │   └── settings.msgpack
    └── custom/              # Custom extension namespace
        └── ...
```

## Key Concepts

### 1. Extension Trait

All extensions must implement the `Extension` trait:

```rust
pub trait Extension {
    fn namespace(&self) -> &str;  // e.g., "contextai"
    fn version(&self) -> &str;    // e.g., "1.0.0"
}
```

### 2. Extension Manager

The `ExtensionManager` handles registration, storage, and retrieval of extension data:

- `register()` - Register an extension
- `write_data()` - Write data to a namespace
- `read_data()` - Read data from a namespace
- `list_extensions()` - List all registered extensions

### 3. Serialization

Extension data is serialized using **MessagePack** for efficiency and compactness.

## Usage

### Writing Extension Data (CxpBuilder)

```rust
use cxp_core::{CxpBuilder, Extension};
use std::collections::HashMap;

// Create your extension
struct MyExtension;

impl Extension for MyExtension {
    fn namespace(&self) -> &str { "myapp" }
    fn version(&self) -> &str { "1.0.0" }
}

// Prepare extension data
let mut data = HashMap::new();
data.insert(
    "config.msgpack".to_string(),
    rmp_serde::to_vec(&my_config)?,
);

// Build CXP file with extension
let mut builder = CxpBuilder::new("./src");
builder
    .scan()?
    .process()?
    .add_extension(&MyExtension, data)?
    .build("output.cxp")?;
```

### Reading Extension Data (CxpReader)

```rust
use cxp_core::CxpReader;

// Open CXP file
let reader = CxpReader::open("output.cxp")?;

// List all extensions
let extensions = reader.list_extensions();
println!("Extensions: {:?}", extensions);

// Read extension data
let data = reader.read_extension("myapp", "config.msgpack")?;
let config: MyConfig = rmp_serde::from_slice(&data)?;

// Get extension manifest
if let Some(manifest) = reader.get_extension_manifest("myapp") {
    println!("Version: {}", manifest.version);
}

// List all data keys in an extension
let keys = reader.list_extension_keys("myapp");
```

## ContextAI Extension

The built-in ContextAI extension (`feature = "contextai"`) demonstrates a complete implementation:

### Data Structure

- **Conversations** - Chat history with messages
- **User Habits** - Language preferences, coding style
- **Settings** - App configuration (theme, auto-index, etc.)
- **Watched Folders** - Directories to monitor
- **Dictionary** - Custom words/terms

### Example Usage

```rust
use cxp_core::contextai::{ContextAIExtension, Conversation, ChatMessage};

// Create extension
let mut ext = ContextAIExtension::new();

// Add a conversation
let conv = Conversation {
    id: "conv-1".to_string(),
    title: "Project Discussion".to_string(),
    created_at: "2025-01-15T10:00:00Z".to_string(),
    updated_at: "2025-01-15T10:00:00Z".to_string(),
    messages: vec![],
};
ext.add_conversation(conv);

// Add a message
let msg = ChatMessage {
    id: "msg-1".to_string(),
    role: "user".to_string(),
    content: "Hello!".to_string(),
    timestamp: "2025-01-15T10:01:00Z".to_string(),
};
ext.add_message("conv-1", msg)?;

// Serialize to extension data
let data = ext.to_extension_data()?;

// Add to CXP file
builder.add_extension(&ext, data)?;
```

### Reading Back

```rust
// Read all extension data from CXP file
let mut extension_data = HashMap::new();
for key in reader.list_extension_keys("contextai") {
    let data = reader.read_extension("contextai", &key)?;
    extension_data.insert(key, data);
}

// Reconstruct the extension
let ext = ContextAIExtension::from_extension_data(extension_data)?;

// Access the data
for conv in ext.list_conversations() {
    println!("Conversation: {}", conv.title);
    for msg in &conv.messages {
        println!("  {}: {}", msg.role, msg.content);
    }
}
```

## Creating Custom Extensions

### Step 1: Define Your Data Structures

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyAppConfig {
    pub version: String,
    pub settings: HashMap<String, String>,
}
```

### Step 2: Implement Extension Trait

```rust
use cxp_core::Extension;

pub struct MyAppExtension {
    config: MyAppConfig,
}

impl Extension for MyAppExtension {
    fn namespace(&self) -> &str {
        "myapp"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}
```

### Step 3: Implement Serialization

```rust
impl MyAppExtension {
    pub fn to_extension_data(&self) -> Result<HashMap<String, Vec<u8>>> {
        let mut data = HashMap::new();

        let config_data = rmp_serde::to_vec(&self.config)?;
        data.insert("config.msgpack".to_string(), config_data);

        Ok(data)
    }

    pub fn from_extension_data(data: HashMap<String, Vec<u8>>) -> Result<Self> {
        let config = if let Some(config_data) = data.get("config.msgpack") {
            rmp_serde::from_slice(config_data)?
        } else {
            MyAppConfig::default()
        };

        Ok(Self { config })
    }
}
```

## Best Practices

1. **Use Namespaces** - Choose unique namespace names to avoid conflicts
2. **Version Your Extensions** - Use semantic versioning for your extension versions
3. **Handle Missing Data** - Provide defaults when deserializing to handle partial data
4. **Keep Data Compact** - Use MessagePack for efficient binary serialization
5. **Document Your Schema** - Clearly document what data your extension stores

## Examples

See the `examples/` directory for complete working examples:

- `contextai_example.rs` - Using the ContextAI extension
- `extension_integration.rs` - Full integration with CXP files

Run with:
```bash
cargo run --example extension_integration --features contextai
```

## API Reference

### CxpBuilder Methods

- `add_extension<E: Extension>(&mut self, ext: &E, data: HashMap<String, Vec<u8>>) -> Result<&mut Self>`

### CxpReader Methods

- `list_extensions(&self) -> Vec<String>`
- `read_extension(&self, namespace: &str, key: &str) -> Result<Vec<u8>>`
- `get_extension_manifest(&self, namespace: &str) -> Option<&ExtensionManifest>`
- `list_extension_keys(&self, namespace: &str) -> Vec<String>`

### ExtensionManager

- `register<E: Extension>(&mut self, ext: E)`
- `write_data(&mut self, namespace: &str, key: &str, data: &[u8]) -> Result<()>`
- `read_data(&self, namespace: &str, key: &str) -> Result<Vec<u8>>`
- `list_extensions(&self) -> Vec<&str>`
- `list_data_keys(&self, namespace: &str) -> Vec<&str>`

## Feature Flags

- `contextai` - Enable the built-in ContextAI extension

```toml
[dependencies]
cxp-core = { version = "0.1.0", features = ["contextai"] }
```
