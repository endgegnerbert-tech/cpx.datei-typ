# CXP - Universal AI Context Format

---

## 📊 AKTUELLER FORTSCHRITT (Stand: 26. Dezember 2025)

```
Phase 1: Core Library      ████████████████████ 100% ✅
Phase 2: Embeddings/Search ████████████████░░░░  80% 🔄
Phase 3: Extension System  ░░░░░░░░░░░░░░░░░░░░   0% ⏳
Phase 4: Multi-Platform    ░░░░░░░░░░░░░░░░░░░░   0% ⏳
Phase 5: ContextAI         ░░░░░░░░░░░░░░░░░░░░   0% ⏳

Gesamt:                    ████████░░░░░░░░░░░░  36%
```

### ✅ Was funktioniert JETZT:
- `cxp build /path output.cxp` - CXP-Dateien erstellen
- `cxp info file.cxp` - Statistiken anzeigen
- `cxp list file.cxp` - Dateien auflisten
- `cxp extract file.cxp` - Dateien extrahieren
- FastCDC Chunking mit Deduplication
- Zstandard Kompression (85% kleiner als JSON!)
- Binary Embeddings (32x kleiner als float32)
- Int8 Embeddings für Rescoring
- HNSW Index mit Hamming Distance
- **15/15 Integration Tests bestanden**

### ⏳ Was fehlt noch:
- Query Engine (`cxp query file.cxp "search term"`)
- Embeddings in CXP-Datei speichern
- Extension System für ContextAI
- WASM Build für Browser
- Node.js/Python Bindings
- SQLite Migration Tool

### 📁 Implementierte Module (cxp-core/src/):
```
lib.rs              ✅ Public API, Feature Flags
format.rs           ✅ CxpBuilder, CxpReader (ZIP Container)
chunker.rs          ✅ FastCDC Content-Defined Chunking
dedup.rs            ✅ SHA256 Deduplication
compress.rs         ✅ Zstandard Compression
manifest.rs         ✅ Manifest mit Stats & FileTypes
error.rs            ✅ CxpError Enum (12 Varianten)
embeddings.rs       ✅ ONNX Runtime Engine (ort 2.0.0-rc.10)
embeddings_tract.rs ✅ Tract Engine für WASM (tract-onnx 0.22)
index.rs            ✅ HNSW Index (usearch 2.15)
```

---

## Vision
Ein **offenes, universelles Datenformat** für KI-Anwendungen:
- **$0 Kosten** - Komplett lokal, keine API-Calls
- **Überall lauffähig** - Rust, WASM, Node.js, Python
- **Ersetzt SQLite** - Eine Datei statt Datenbank
- **Multi-KI Ready** - Claude, GPT, Gemini, Llama...
- **Zukunftssicher** - Erweiterbar für neue Use Cases
- **Open Standard** - Jeder kann es nutzen/implementieren

## Die große Idee
CXP wird das **"PDF für KI"** - ein universelles Format das:
1. Jede KI lesen kann
2. Semantische Suche built-in hat
3. Komplette App-States speichern kann
4. Portabel und offline funktioniert
5. Ein offener Standard werden kann

---

## Use Cases (Heute & Zukunft)

### 1. ContextAI (Erste Implementation)
```
ContextAI App - SQLite wird komplett ersetzt durch CXP!

Vorher (SQLite):           Nachher (CXP):
├── 7 Tabellen             ├── 1 Datei: context.cxp
├── Keyword-Suche          ├── Semantische Suche
├── App-gebunden           ├── Portabel
└── Nicht teilbar          └── Einfach kopieren/teilen
```

### 2. Personal Knowledge Base
```
Alle deine Dokumente, Notizen, Code in einer .cxp Datei
→ Frag jede KI Fragen über DEINE Daten
→ Lokal, privat, keine Cloud
```

### 3. Projekt-Kontext für Entwickler
```
my-project.cxp
→ Enthält komplette Codebase mit Embeddings
→ Cursor, Windsurf, Claude Code können es laden
→ "Versteh mein Projekt" in einer Datei
```

### 4. Team Knowledge Sharing
```
team-knowledge.cxp
→ Team teilt Wissen in einer Datei
→ Neue Mitarbeiter: Datei laden → KI kennt alles
→ Kein Onboarding-Chaos mehr
```

### 5. Zukunft: Universal AI Data Layer
```
Alle Apps speichern in .cxp
→ Deine Daten gehören DIR
→ Wechsel zwischen KIs/Apps ohne Datenverlust
→ Interoperabilität zwischen Tools
```

---

## Multi-Platform Architektur

CXP läuft **überall**:

```
cxp/
├── cxp-core/              # Rust Core Library
│   ├── src/
│   │   ├── lib.rs         # Public API
│   │   ├── format.rs      # CXP Read/Write
│   │   ├── chunker.rs     # FastCDC
│   │   ├── embeddings.rs  # ONNX Runtime
│   │   ├── index.rs       # HNSW Search
│   │   ├── quantize.rs    # Binary/Int8
│   │   └── extensions.rs  # Namespace System
│   └── Cargo.toml
│
├── cxp-wasm/              # WebAssembly Build
│   └── (Browser, Deno, Cloudflare Workers)
│
├── cxp-node/              # Node.js Bindings (napi-rs)
│   └── (npm package: @cxp/core)
│
├── cxp-python/            # Python Bindings (PyO3)
│   └── (pip package: cxp)
│
├── cxp-cli/               # Standalone CLI
│   └── cxp build, cxp query, cxp export
│
└── schemas/               # FlatBuffers Schemas
    ├── manifest.fbs
    ├── embeddings.fbs
    └── extensions/
        └── contextai.fbs
```

### Platform Support Matrix

| Platform | Runtime | Use Case |
|----------|---------|----------|
| **Tauri/Desktop** | Rust Native | ContextAI App |
| **Browser** | WASM | Web Apps, PWAs |
| **Node.js** | napi-rs | CLI Tools, Servers |
| **Python** | PyO3 | Data Science, ML |
| **Deno** | WASM | Edge Functions |
| **Mobile** | Rust FFI | iOS/Android Apps |

---

## CXP als SQLite-Ersatz für ContextAI

### Aktuelles SQLite Schema (wird ersetzt):
```sql
files, conversations, chat_messages, context_log,
user_habits, habit_history, watched_folders,
browser_history, custom_dictionary
```

### Neues CXP Format:
```
context.cxp (ZIP Container)
├── core/                      # Standard CXP
│   ├── manifest.fbs           # Metadata, Version, Stats
│   ├── embeddings/
│   │   ├── binary.bin         # Binary Embeddings (48B/vec)
│   │   ├── int8.bin           # Int8 für Rescoring
│   │   └── index.hnsw         # HNSW Index
│   ├── chunks/
│   │   └── *.zst              # Zstandard komprimiert
│   └── file_map.msgpack       # Datei → Chunks
│
└── contextai/                 # ContextAI Extension
    ├── conversations/
    │   ├── index.msgpack      # Conversation List
    │   └── conv_*.msgpack     # Individual Conversations
    ├── habits.msgpack         # User Preferences
    ├── dictionary.msgpack     # Custom Terms
    ├── watched_folders.msgpack
    └── settings.msgpack       # App Settings
```

### Migration Path:
```
1. CXP Library implementieren
2. ContextAI: SQLite → CXP Adapter
3. Migration Tool: SQLite → CXP Export
4. SQLite Code entfernen
5. Nur noch CXP
```

---

## Neue Erkenntnisse aus der Recherche

### 1. Embedding-Modelle (2025 State-of-the-Art)

| Modell | Size | Dims | Besonderheit |
|--------|------|------|--------------|
| **EmbeddingGemma** | 308M / ~200MB RAM | 768 (MRL: 512/256/128) | Best-in-class für On-Device, int4 quantized |
| **all-MiniLM-L6-v2** | 22M / ~90MB | 384 | Bewährt, schnell |
| **BGE-small** | 33M | 384 | Multilingual |

**Breakthrough:** EmbeddingGemma mit Matryoshka (MRL) erlaubt flexible Dimensionen + int4 Quantisierung = **32x kleinere Vektoren**!

### 2. Binary Embeddings (Game-Changer!)

```
float32 (384 dims) = 1.5 KB pro Vector
int8 (384 dims)    = 384 Bytes (4x kleiner)
binary (384 dims)  = 48 Bytes (32x kleiner!)
```

**Strategie:**
1. Binary Search (48 Bytes) für Vorfilterung
2. int8 Rescoring für Top-100
3. Optional: Reranking für Top-10

**Ergebnis:** 95% Qualität bei 32x weniger Speicher!

### 3. WebGPU Embeddings im Browser

- **Transformers.js v3** mit WebGPU: **64x schneller** als WASM!
- Läuft direkt im Browser, keine Server-Kosten
- 70% Browser-Support (Chrome, Edge, Firefox)
- Fallback auf WASM für ältere Browser

### 4. HNSW im Browser (WASM)

- **EdgeVec** (Rust/WASM): 148KB Bundle, sub-ms Search bei 100k Vectors
- **hnswlib-wasm**: Browser HNSW mit IndexedDB Persistenz
- **USearch**: Cross-platform, SIMD-optimiert

### 5. Bessere Serialisierung

| Format | vs JSON | Zero-Copy | Use-Case |
|--------|---------|-----------|----------|
| **FlatBuffers** | 80% kleiner | JA | Manifest, schneller Zugriff |
| **MessagePack** | 70% kleiner | Nein | Flexible Daten |
| **Protobuf** | 80% kleiner | Nein | Embeddings |

**Neu:** FlatBuffers für Manifest = Zero-Copy Zugriff ohne Parsing!

### 6. Semantic Chunking (2025)

- **FastCDC** mit Gear Hash: O(log N) Chunking
- **HOPE Metric:** Semantische Unabhängigkeit optimieren
- **Hashless CDC:** Noch schneller, keine Rolling Hashes

### 7. LMCompress (Neural Compression)

- Halbiert JPEG-XL, FLAC, H.264
- Text: 1/3 der zpaq-Größe
- **Aber:** Zu compute-intensiv für $0-Ziel

---

## CXP 2.0 Architektur (Optimiert)

```
contextpack.cxp (ZIP Container)
├── manifest.fbs          # FlatBuffers (Zero-Copy, 5-15KB)
├── embeddings/
│   ├── binary.bin        # Binary Embeddings (48B/vector) - Primary
│   ├── int8.bin          # Int8 für Rescoring (384B/vector) - Optional
│   └── index.hnsw        # HNSW Index (WASM-kompatibel)
├── chunks/
│   └── *.zst             # Zstandard komprimiert
├── file_map.msgpack      # Datei → Chunks
├── keywords.fst          # FST statt Trie (kleiner, schneller)
└── meta.cbor             # Zusätzliche Metadaten
```

### Größenvergleich

```
Original CXP (Spec):     170MB für 500MB Input
CXP 2.0 (Binary Emb):    ~50MB für 500MB Input (70% kleiner!)
```

---

## Zero-Cost Pipeline

### Phase 1: Chunking ($0, lokal)
```
Input Files → FastCDC (Gear Hash) → SHA256 Dedup → Chunks
```
- Rust native
- 100% lokal, keine Dependencies außer Crypto

### Phase 2: Embeddings ($0, lokal)
```
Chunks → ONNX Runtime → Binary Quantization
```
- **Desktop:** ONNX Runtime (native)
- **Browser:** WebGPU oder WASM
- **Model:** EmbeddingGemma (200MB, On-Demand)

### Phase 3: Index ($0, lokal)
```
Binary Embeddings → HNSW Build → .hnsw File
```
- usearch (Rust native)
- WASM-kompatibel für Browser

### Phase 4: Search ($0, lokal)
```
Query → Embed → Binary HNSW Search → Int8 Rescore → Top-K
```
- Alles lokal!
- Keine Server, keine API-Kosten

---

## Neue Features (Ideen zum Nachdenken)

### 1. Progressive Loading
```
Lade zuerst: manifest.fbs (5KB) + binary.bin Header
Dann on-demand: Chunks nur wenn gebraucht
```
→ Instant Start, selbst für GB-große CXP-Dateien

### 2. Streaming Embeddings
```
Während User tippt → Query Embedding berechnen
Binary Search startet sofort → Latenz maskiert
```

### 3. Differential Updates (Delta-CXP)
```
Original: data.cxp (50MB)
Update:   delta.cxp (500KB) - nur geänderte Chunks
Merge:    Lazy, on-demand
```

### 4. Peer-to-Peer Sharing
```
CXP ist eine Datei → Torrent, IPFS, lokales Netzwerk
Kein Server nötig für Sharing
```

### 5. Multi-Modal ohne API
```
Images: CLIP ONNX (lokal)
Audio:  Whisper ONNX (lokal) → Text → Embedding
PDF:    pdf.js → Text → Embedding
```
Alles $0!

### 6. Hybrid Intelligence
```
Lokal:    Binary Search + Int8 Rescore (95% accuracy)
Optional: Claude für Top-3 Ergebnisse ($0.01/query)
```
→ 95% der Queries komplett kostenlos

---

## Umwelt-Impact

| Ansatz | CO2/Query (geschätzt) |
|--------|----------------------|
| GPT-4 API | ~4.5g CO2 |
| Claude API | ~2-3g CO2 |
| **CXP (lokal)** | ~0.01g CO2 |

**Faktor 200-400x weniger CO2** durch lokale Verarbeitung!

---

## Technische Entscheidungen

### Embedding Model
**Gewählt:** `EmbeddingGemma` (Google, 2025)
- 308M Parameter, ~200MB
- 768 dims (MRL: flexible 512/256/128)
- int4 quantized out-of-the-box
- Best-in-class für On-Device
- Multilingual (100+ Sprachen)
- On-Demand Download (nicht bundled)

### Quantization
**Gewählt:** Binary + Int8 Hybrid
- Binary für HNSW Index (schnell, klein)
- Int8 für Rescoring (bessere Präzision)
- ~3% Qualitätsverlust, 32x Speicherersparnis

### Serialisierung
**Gewählt:**
- Manifest: FlatBuffers (Zero-Copy)
- File Map: MessagePack (flexibel)
- Embeddings: Raw Binary (effizient)

### Search
**Gewählt:** HNSW via usearch
- Sub-ms bei 100k Vectors
- Rust native + WASM Support

---

## Implementation Roadmap

### Phase 1: CXP Core Library ✅ KOMPLETT
- [x] Rust Workspace Setup (cxp-core, cxp-cli)
- [x] MessagePack für Manifest (statt FlatBuffers - einfacher)
- [x] FastCDC Chunking (Gear Hash)
- [x] SHA256 Deduplication
- [x] Zstandard Compression
- [x] ZIP Container Read/Write
- [x] CLI: `cxp build`, `cxp info`, `cxp list`, `cxp extract`

### Phase 2: Embeddings & Search 🔄 80% FERTIG
- [x] ONNX Runtime Integration (`ort = "2.0.0-rc.10"`)
- [x] WASM-Alternative mit tract-onnx (`tract-onnx = "0.22"`)
- [x] Model Support (all-MiniLM-L6-v2, EmbeddingGemma)
- [x] Binary Quantization (float32 → binary, 32x kleiner!)
- [x] Int8 Quantization für Rescoring (4x kleiner)
- [x] HNSW Index Build (`usearch = "2.15"`)
- [x] Hamming Distance für Binary Embeddings
- [x] **15/15 Integration Tests bestanden**
- [ ] Query Engine: `cxp query file.cxp "search term"`
- [ ] Embeddings in CXP-Datei integrieren

### Phase 3: Extension System ⏳
- [ ] Namespace System für Extensions
- [ ] ContextAI Extension Schema
- [ ] Conversations Storage
- [ ] Habits/Dictionary/Settings Storage
- [ ] CLI: `cxp ext add contextai`

### Phase 4: Multi-Platform ⏳
- [ ] WASM Build (wasm-pack) - tract-onnx Grundlage vorhanden!
- [ ] Node.js Bindings (napi-rs)
- [ ] Python Bindings (PyO3)
- [ ] npm/pip Package Publishing

### Phase 5: ContextAI Integration ⏳
- [ ] SQLite → CXP Migration Tool
- [ ] Tauri Commands für CXP
- [ ] Frontend anpassen
- [ ] SQLite Code entfernen
- [ ] Testing & Bug Fixes

---

### Aktuelle Crate Versionen (Getestet & Funktionierend)

```toml
[workspace]
members = ["cxp-core", "cxp-cli"]

[workspace.dependencies]
# Core
fastcdc = "3.1"
zstd = "0.13"
sha2 = "0.10"
zip = "2.2"
rayon = "1.10"

# Serialization
flatbuffers = "24.12"
rmp-serde = "1.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"

# File System
walkdir = "2.5"

# Misc
chrono = "0.4"
uuid = { version = "1.11", features = ["v4"] }
hex = "0.4"

# Embeddings (optional) - NATIVE (schnell)
ort = { version = "2.0.0-rc.10", optional = true }
ndarray = { version = "0.16", optional = true }
tokenizers = { version = "0.21", optional = true }
num_cpus = { version = "1.16", optional = true }

# Embeddings (optional) - WASM KOMPATIBEL (portabel)
tract-onnx = { version = "0.22", optional = true }

# Search (optional)
usearch = { version = "2.15", optional = true }

[features]
default = []
embeddings = ["ort", "ndarray", "tokenizers", "num_cpus"]
embeddings-wasm = ["tract-onnx", "ndarray", "tokenizers"]
search = ["usearch"]
```

---

## Entschieden

1. **Vision:** CXP als universelles KI-Datenformat (Potenzial für Standard)
2. **Model:** EmbeddingGemma (200MB, On-Demand Download)
3. **Plattform:** Multi-Platform (Rust + WASM + Node + Python)
4. **Manifest:** FlatBuffers (Zero-Copy)
5. **ContextAI:** SQLite wird komplett durch CXP ersetzt
6. **Extension System:** Namespaces für App-spezifische Daten

---

## Lizenz & Schutz Strategie

### Phase 1: Build in Private (JETZT)
```
├── Private GitHub Repository
├── Kein öffentlicher Code
├── Fokus auf Bauen, nicht Marketing
└── Niemand sieht was du machst
```

### Phase 2: Teaser & Hype (Wenn MVP fertig)
```
├── Twitter/X: "Building something new..."
├── Screenshots & Demo Videos
├── Waitlist aufbauen
├── KEIN Code zeigen
└── Interesse wecken
```

### Phase 3: Launch (App Release)
```
├── ContextAI App = Closed Source
├── CXP Format Spec = Noch nicht veröffentlichen
├── User können App nutzen
└── Format bleibt "Black Box" fürs Erste
```

### Phase 4: Open Standard (Optional, später)
```
Wenn du bereit bist:
├── CXP Spec unter AGPL-3.0 veröffentlichen
├── Commercial License für Firmen anbieten
├── Community aufbauen
└── Standard-Adoption anstreben

ODER proprietär bleiben - du entscheidest später
```

### Warum dieser Ansatz gut ist:
```
✓ Maximaler Schutz während du baust
✓ Kein Stress wegen Konkurrenz
✓ Flexibilität für später
✓ Hype aufbauen ohne Code zu zeigen
✓ Du behältst alle Optionen
```

## ContextAI App Pfad
`/Users/einarjaeger/Documents/GitHub/context Ai App`

## Projekt-Struktur (dieser Ordner)
```
/Users/einarjaeger/Documents/GitHub/cpx.datei typ/
├── cxp-core/          # Rust Core Library
├── cxp-cli/           # CLI Tool
├── cxp-wasm/          # WASM Build
├── cxp-node/          # Node.js Bindings
├── cxp-python/        # Python Bindings
├── schemas/           # FlatBuffers Schemas
├── docs/              # Spezifikation & Docs
│   └── SPEC.md        # Offizielle CXP Spezifikation
├── examples/          # Beispiele
├── CXP-2.0-PLAN.md    # Dieser Plan
└── cpx.newdatatyp.md  # Original Spec
```

---

## Quellen

### Embedding Models
- [EmbeddingGemma - Google](https://developers.googleblog.com/en/introducing-embeddinggemma/)
- [Transformers.js v3 WebGPU](https://huggingface.co/blog/transformersjs-v3)
- [FastEmbed - Qdrant](https://github.com/qdrant/fastembed)

### Quantization
- [Binary & Scalar Embedding Quantization - HuggingFace](https://huggingface.co/blog/embedding-quantization)
- [Matryoshka Embeddings - Vespa](https://blog.vespa.ai/combining-matryoshka-with-binary-quantization-using-embedder/)
- [Voyage AI Quantization](https://blog.voyageai.com/2025/05/20/voyage-3-5/)

### Chunking
- [FastCDC Paper - USENIX](https://www.usenix.org/conference/atc16/technical-sessions/presentation/xia)
- [Semantic Chunking 2025](https://www.emergentmind.com/topics/content-defined-chunking-cdc)

### Search
- [LSH Guide - Pinecone](https://www.pinecone.io/learn/series/faiss/locality-sensitive-hashing/)
- [USearch - HNSW](https://github.com/unum-cloud/USearch)
- [EdgeVec - Browser Vector Search](https://news.ycombinator.com/item?id=46249896)
- [hnswlib-wasm](https://github.com/ShravanSunder/hnswlib-wasm)

### Serialization
- [FlatBuffers Benchmarks](https://flatbuffers.dev/benchmarks/)
- [Binary Format Comparison](https://www.cloudthat.com/resources/blog/optimizing-api-performance-with-protocol-buffers-flatbuffers-messagepack-and-cbor)

### Compression
- [LMCompress - Nature](https://www.nature.com/articles/s42256-025-01033-7)
- [WebGPU Embedding Benchmark](https://huggingface.co/posts/Xenova/906785325455792)

---

*Erstellt: 2025-12-26*
*Status: Ready for Implementation*
