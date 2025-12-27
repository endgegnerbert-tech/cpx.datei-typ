# CXP Embeddings Storage - Nutzungsanleitung

## Überblick

Das CXP-Format wurde um optionale Embeddings-Storage erweitert. Diese Funktionalität ist feature-gated und funktioniert auch OHNE Embeddings.

## Neue Komponenten

### 1. `semantic.rs` - Embeddings Storage Module

Neue Datei: `cxp-core/src/semantic.rs`

**Features:**
- `EmbeddingStore` struct für Binary + Int8 Embeddings
- Kompakte Serialisierung/Deserialisierung
- Feature-gated: `#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]`

**Funktionen:**
- `serialize_binary_embeddings(embeddings: &[BinaryEmbedding]) -> Result<Vec<u8>>`
- `deserialize_binary_embeddings(data: &[u8]) -> Result<Vec<BinaryEmbedding>>`
- `serialize_int8_embeddings(embeddings: &[Int8Embedding]) -> Result<Vec<u8>>`
- `deserialize_int8_embeddings(data: &[u8]) -> Result<Vec<Int8Embedding>>`

### 2. Erweiterte `CxpBuilder` (format.rs)

**Bestehende Methode:**
```rust
#[cfg(all(feature = "embeddings", feature = "search"))]
pub fn with_embeddings<P: AsRef<Path>>(
    &mut self,
    model_path: P,
    model: EmbeddingModel,
) -> Result<&mut Self>
```

**Speicherung:**
Die Embeddings werden automatisch im `build()` gespeichert, wenn sie aktiviert wurden:
- `embeddings/binary.bin` - Binary quantized embeddings
- `embeddings/int8.bin` - Int8 quantized embeddings (für Rescoring)
- `embeddings/index.hnsw` - HNSW Index für schnelle Suche

### 3. Erweiterte `CxpReader` (format.rs)

**Neue Methoden:**

```rust
// Prüfen ob Embeddings vorhanden sind
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm", feature = "search"))]
pub fn has_embeddings(&self) -> bool

// Embeddings ohne Index laden
#[cfg(any(feature = "embeddings", feature = "embeddings-wasm"))]
pub fn get_embedding_store(&self) -> Result<EmbeddingStore>

// Embeddings UND Index laden (für Suche)
#[cfg(all(feature = "embeddings", feature = "search"))]
pub fn load_embeddings(&mut self) -> Result<()>
```

## Verwendung

### Embeddings erstellen und speichern

```rust
use cxp_core::{CxpBuilder, EmbeddingModel};

let mut builder = CxpBuilder::new("./source");

builder
    .scan()?
    .with_embeddings("./models/all-MiniLM-L6-v2", EmbeddingModel::MiniLM)?
    .process()?
    .build("output.cxp")?;
```

### Embeddings laden (ohne Suche)

```rust
use cxp_core::CxpReader;

let reader = CxpReader::open("output.cxp")?;

if reader.has_embeddings() {
    let store = reader.get_embedding_store()?;
    println!("Loaded {} embeddings", store.len());
    println!("Dimensions: {}", store.dimensions);

    // Zugriff auf einzelne Embeddings
    if let Some(binary) = store.get_binary(0) {
        println!("Binary embedding size: {} bytes", binary.size_bytes());
    }

    if let Some(int8) = store.get_int8(0) {
        println!("Int8 embedding size: {} bytes", int8.size_bytes());
    }
}
```

### Embeddings laden (mit Suche)

```rust
use cxp_core::CxpReader;

let mut reader = CxpReader::open("output.cxp")?;

if reader.has_embeddings() {
    // Lädt Embeddings UND HNSW Index in den Speicher
    reader.load_embeddings()?;

    // Jetzt kann semantische Suche durchgeführt werden
    let query = vec![0.1, 0.2, 0.3, ...]; // 384 Dimensionen für MiniLM
    let results = reader.search_semantic(&query, 10)?;

    for result in results {
        println!("Chunk {} - Score: {}", result.id, result.distance);
    }
}
```

## CXP Archive Struktur (mit Embeddings)

```
file.cxp (ZIP)
├── manifest.msgpack         # Metadaten (inkl. embedding_model, embedding_dim)
├── file_map.msgpack         # File -> Chunk References
├── chunks/
│   ├── 0001.zst             # Komprimierte Chunks
│   ├── 0002.zst
│   └── ...
├── embeddings/              # Nur wenn with_embeddings() verwendet
│   ├── binary.bin           # Binary quantized (32x kleiner)
│   ├── int8.bin             # Int8 quantized (4x kleiner, für Rescoring)
│   └── index.hnsw           # HNSW Index für schnelle Suche
└── extensions/              # Optional
    └── ...
```

## Binary Format

### Binary Embeddings (`binary.bin`)

```
Header:
- u32: Anzahl der Embeddings
- u32: Dimensionen

Für jedes Embedding:
- bytes: packed bits (dimensions / 8 bytes)
```

### Int8 Embeddings (`int8.bin`)

```
Header:
- u32: Anzahl der Embeddings
- u32: Dimensionen

Für jedes Embedding:
- f32: Scale factor
- i8 * dimensions: Quantisierte Werte
```

## Feature Flags

- **Ohne Features:** CXP funktioniert normal ohne Embeddings
- **`embeddings`:** Native ONNX-basierte Embeddings + Storage
- **`embeddings-wasm`:** WASM-basierte Embeddings (via Tract) + Storage
- **`search`:** HNSW Index für schnelle semantische Suche
- **`embeddings + search`:** Vollständige Embeddings + Suche Integration

## Wichtige Hinweise

1. **Optional:** CXP funktioniert auch OHNE Embeddings - die Extension ist vollständig optional
2. **Feature-gated:** Alle Embeddings-Funktionen sind hinter Feature-Flags
3. **Speicher-effizient:** Binary (32x) und Int8 (4x) Quantisierung
4. **Zwei-Stufen-Suche:** Binary für schnelle Initial-Suche, Int8 für präzises Rescoring
5. **Manifest-Integration:** `embedding_model` und `embedding_dim` werden im Manifest gespeichert
6. **Extension-Markierung:** `"embeddings"` wird in `manifest.extensions` hinzugefügt

## Tests

Alle Serialisierungs/Deserialisierungs-Funktionen haben Unit-Tests in `semantic.rs`:
- Binary Roundtrip
- Int8 Roundtrip
- Dimension Mismatch Detection
- Empty Embeddings Handling
- Size Calculations
