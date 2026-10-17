# Kestrel — Architecture & Design

Kestrel is an AI-native, vector-first, multimodal database. One Rust codebase
ships in three forms — an **embedded library** (phone, edge device, laptop,
browser via WASM), a **single-node server**, and a **distributed lakehouse
service** with compute separated from object storage. The data format and
query semantics are the same in all three.

**One-line thesis:** one Rust engine, one segment format, one query planner —
identical semantics embedded on a device, on a single server, and distributed
over object storage. Only the storage backend, scheduler, and coordinator change.

Design trade-offs and launch sequencing live in [launch-plan.md](launch-plan.md).

## 1. System architecture

```mermaid
flowchart TB
    subgraph IF["Interfaces"]
        SQL["SQL+"]
        SDK["SDKs: Python / TS / Rust / Swift / Kotlin"]
        API["REST / gRPC / Arrow Flight"]
        MCP["MCP tool-call server - typed, permission-scoped"]
        NL["Natural language to query - plan shown for review"]
    end
    subgraph CORE["kestrel-core - one Rust codebase"]
        PLAN["Query planner and optimizer<br/>hybrid fusion - filter strategy - graph hops"]
        EXEC["Vectorized execution engine - Arrow"]
        subgraph IDX["Index and kernel layer"]
            ANN["ANN: HNSW/graph - IVF-PQ - DiskANN-style"]
            LEX["Sparse / learned-sparse - BM25"]
            BMP["Bitmaps - metadata filters"]
            KERN["SIMD/asm kernels: AVX2 - AVX-512 - NEON - SVE<br/>scalar Rust reference always exists"]
        end
        GOV["Governance in the retrieval path<br/>RBAC / ABAC inside ANN - lineage - audit"]
        STORE["Storage abstraction - trait-based"]
    end
    subgraph BACK["Storage backends - same segment format"]
        LFS["Local FS / mmap"]
        MOB["Mobile sandbox / IndexedDB - WASM"]
        OBJ["Object store S3 / GCS / Azure + WAL + NVMe cache"]
    end
    IF --> PLAN
    PLAN --> EXEC
    EXEC --> IDX
    GOV -. "enforced inside traversal" .-> ANN
    IDX --> STORE
    STORE --> BACK
```

## 2. Deployment tiers — same core, three forms

```mermaid
flowchart LR
    subgraph E["kestrel-core - Embedded"]
        E1["In-process library"]
        E2["Phone - edge - laptop - browser WASM"]
        E3["Local files / mmap - on-device embedding"]
    end
    subgraph S["kestrel-server - Single node"]
        S1["Same core + network layer"]
        S2["Multi-tenant catalog"]
        S3["Background compaction and re-embedding"]
    end
    subgraph D["kestrel-cloud - Distributed"]
        D1["Stateless query and ingest nodes"]
        D2["Per-partition WAL to immutable segments"]
        D3["Metadata catalog - snapshots - fan-out and merge"]
    end
    E -- "sync subsets bidirectionally<br/>offline-first - data residency" --> D
    E -- "same segment format" --> S
    S -- "same planner and indexes" --> D
```

## 3. Hybrid query — one primitive, one plan

Dense + sparse + keyword + metadata + graph retrieval is a single primitive,
not five systems stitched together.

```mermaid
sequenceDiagram
    autonumber
    participant C as Client / Agent
    participant P as Planner
    participant I as Indexes
    participant S as Segments
    C->>P: Hybrid query - vector, text, filters, optional graph hops
    P->>P: Choose filter strategy - pre-filter / filtered traversal / post-filter
    P->>P: Apply RBAC/ABAC policy - unauthorized vectors excluded up front
    P->>I: Fan out - dense ANN, sparse, BM25, bitmap filters, graph expansion
    I->>S: Scan segments - partition pruning, predicate push-down
    S-->>I: Candidate sets per signal
    I-->>P: Per-signal ranked results
    P->>P: Fuse scores - RRF / weighted / learned re-rank, top-k
    P-->>C: Results + lineage - source version, chunker, model, parameters
```

## 4. Embed-on-ingest pipeline

```mermaid
flowchart LR
    IN["Ingest: text - image - audio - video - PDF - JSON"]
    EXT["Extract text and chunk"]
    EMB["Embed per declared column model<br/>local ONNX/GGUF or remote endpoint"]
    PROMO["Hot-field promotion<br/>frequently queried JSON fields to typed columns + indexes"]
    SEG["Write segment: object + text + chunks + embeddings<br/>+ lineage record"]
    RE["Model changed? Background re-embed<br/>old and new versions both tracked"]
    IN --> EXT --> EMB --> SEG
    IN --> PROMO --> SEG
    SEG -.-> RE -.-> SEG
```

## 5. Distributed tier — writes, compaction, reads

```mermaid
flowchart TB
    subgraph W["Write path"]
        ING["Ingest nodes - stateless"]
        WAL["Per-partition log-structured WAL<br/>on object storage"]
        COMP["Compaction to immutable segments<br/>each segment carries its own index"]
        ING --> WAL --> COMP
    end
    CAT["Metadata catalog<br/>snapshots - time travel - schema evolution<br/>Iceberg/Delta-style open table format"]
    subgraph R["Read path"]
        QN["Query nodes - stateless, scale independently"]
        CACHE["Local NVMe cache<br/>hot segments and index pages"]
        MERGE["Planner merges fan-out results"]
        QN --> CACHE --> MERGE
    end
    COMP --> CAT
    CAT --> QN
    EXT["Existing Iceberg lakes<br/>queried and indexed in place"] -.-> CAT
```

## 6. Kernel strategy — SIMD only where it pays

The hottest loops (distance kernels, quantized scans, decompression, bitmap
filtering) are where compilers often under-vectorize. Everything else —
planner, I/O, networking, cache-miss-bound traversal — stays in safe,
idiomatic Rust.

```mermaid
flowchart TD
    LOOP["Hot loop candidate<br/>distance - PQ scan - decompression - bitmap - top-k"]
    Q1{"Portable Rust / intrinsics<br/>fast enough on profile?"}
    ASM["Hand-written asm in one audited crate<br/>documented invariants - small kernel set"]
    DISP["Runtime CPU dispatch<br/>AVX2 / AVX-512 / NEON / SVE / scalar reference"]
    TEST["Differential tests + fuzzing vs scalar reference<br/>ISA CI - benchmark regression gate on every PR"]
    KEEP{"Periodic review:<br/>still beats compiler?"}
    DROP["Retire kernel - SIMD is a replaceable optimization"]
    LOOP --> Q1
    Q1 -- "yes" --> DISP
    Q1 -- "no, lasting gap" --> ASM --> DISP
    DISP --> TEST --> KEEP
    KEEP -- "no" --> DROP --> LOOP
    KEEP -- "yes" --> DISP
```

Rules: intrinsics first, hand asm only where profiling shows a lasting gain;
a scalar Rust reference always exists; every kernel is differential-tested
against it; all `unsafe` code is isolated in one audited crate. Benchmarks
decide what ships — see [kernels.md](kernels.md).

## 7. Agent-native write-back and contradiction detection

```mermaid
flowchart LR
    A["Agent via MCP tool call<br/>typed, permission-scoped"]
    F["New fact written as versioned record<br/>with provenance"]
    N["Find semantically close existing facts"]
    C{"Contradiction detected?"}
    OK["Committed - queryable with lineage"]
    FLAG["Flagged for resolution<br/>not silently merged"]
    A --> F --> N --> C
    C -- "no" --> OK
    C -- "yes" --> FLAG
```

## 8. Roadmap (engineering sequence)

```mermaid
gantt
    title Kestrel engineering roadmap
    dateFormat YYYY-MM
    axisFormat %Y-%m
    section Core engine
    Segment format v0, HNSW+PQ, AVX2/NEON kernels, alpha    :2026-10, 3M
    Hybrid planner, embed-on-ingest, server, WASM/mobile    :2027-01, 3M
    Hot-field promotion, lineage, RBAC, AVX-512/SVE - beta  :2027-04, 3M
    section Scale-out
    Object store, distributed preview, Iceberg read, MCP    :2027-07, 3M
    Cloud beta, ABAC in ANN, graph hops, edge-cloud sync    :2027-10, 6M
    section GA
    Core 1.0, Iceberg write, BYOC, SOC 2 process            :2028-04, 6M
```

Near-term work is tracked as GitHub issues. Benchmark methodology:
reproducible harness or it did not happen — losses published alongside wins,
competitors at vendor-recommended settings, recall reported with every
throughput number.

