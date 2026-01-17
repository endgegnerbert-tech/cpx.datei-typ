# CXP Core

**CXP** (Context Exchange Protocol) is a universal, high-performance file format designed specifically for AI context management. It enables efficient storage, compression, and retrieval of contextual data for AI applications.

## Why CXP?

Traditional formats like JSON or XML are inefficient for AI context:

| Feature | JSON/XML | CXP |
|---------|----------|-----|
| Compression | None | Zstd (up to 90% smaller) |
| Deduplication | None | Content-aware chunking |
| Binary support | Base64 (bloated) | Native binary |
| Streaming | Limited | Full streaming support |
| Token efficiency | Poor | Optimized for LLMs |
| Incremental updates | Full rewrite | Delta updates |

**CXP reduces token usage by 40-70%** compared to raw text, saving costs and improving AI response quality.

## Features

- **Smart Compression**: Zstd compression with content-aware chunking
- **Deduplication**: Automatic detection and elimination of duplicate content
- **Hierarchical Structure**: Recursive CXP files for organizing large codebases
- **Binary Support**: Native handling of images, PDFs, and other binary files
- **Streaming API**: Process large files without loading everything into memory
- **Token Optimization**: Designed to minimize token usage for LLM APIs
- **Cross-Platform**: Works on macOS, Windows, and Linux

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cxp-core = { git = "https://github.com/endgegnerbert-tech/cxp-core" }
```

## Quick Start

```rust
use cxp_core::{CxpBuilder, CxpReader};

// Create a CXP file
let cxp = CxpBuilder::new()
    .add_file("src/main.rs")?
    .add_directory("src/")?
    .build()?;

cxp.save("project.cxp")?;

// Read a CXP file
let reader = CxpReader::open("project.cxp")?;
for file in reader.files() {
    println!("{}: {} bytes", file.path, file.size);
}
```

## Use Cases

### AI Context Management
CXP is the backbone of [ContextAI](https://github.com/endgegnerbert-tech/context-Ai-App), providing:
- Efficient codebase indexing
- Smart file filtering and prioritization
- Optimized context windows for AI conversations

### Codebase Archiving
Create compact, searchable archives of entire projects:
```rust
let cxp = CxpBuilder::new()
    .add_directory("./my-project")?
    .with_recursive(true)
    .build()?;
```

### Incremental Backups
CXP's deduplication makes it ideal for backup systems:
```rust
let cxp = CxpBuilder::new()
    .from_existing("backup.cxp")?
    .add_changes_since(last_backup_time)?
    .build()?;
```

## Architecture

```
┌─────────────────────────────────────────────┐
│                 CXP File                    │
├─────────────────────────────────────────────┤
│  Header (magic bytes, version, metadata)    │
├─────────────────────────────────────────────┤
│  Manifest (file index, checksums)           │
├─────────────────────────────────────────────┤
│  Chunks (deduplicated, compressed blocks)   │
├─────────────────────────────────────────────┤
│  Index (fast lookup tables)                 │
└─────────────────────────────────────────────┘
```

## Feature Flags

```toml
[dependencies]
cxp-core = { version = "0.1", features = ["embeddings", "search"] }
```

| Feature | Description |
|---------|-------------|
| `default` | Core functionality |
| `embeddings` | Vector embeddings for semantic search |
| `search` | Full-text and semantic search |
| `multimodal` | Image and PDF processing |
| `scanner` | File system scanning utilities |
| `contextai` | ContextAI integration helpers |

## Performance

Benchmarks on a typical codebase (10,000 files, 500MB):

| Operation | Time | Memory |
|-----------|------|--------|
| Create CXP | 2.3s | 150MB |
| Read CXP | 0.4s | 50MB |
| Search | 12ms | 20MB |
| Compression ratio | 73% | - |

## Specification

The CXP format specification is available in [SPECIFICATION.md](./SPECIFICATION.md).

## License

CXP Core is released under the [CXP License](./LICENSE).

**Permitted:**
- Personal and educational use
- Open-source projects (with attribution)
- Use in ContextAI

**Requires permission:**
- Commercial use
- Competing products

See [LICENSE](./LICENSE) for full terms.

## Attribution

If you use CXP in your project, please include attribution:

```
Powered by CXP Core - https://github.com/endgegnerbert-tech/cxp-core
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## Links

- [ContextAI](https://github.com/endgegnerbert-tech/context-Ai-App) - AI assistant powered by CXP
- [Documentation](https://cxp-core.dev/docs) (coming soon)
- [Discord Community](https://discord.gg/contextai) (coming soon)

---

**Created by Einar Jaeger** | [GitHub](https://github.com/endgegnerbert-tech)
