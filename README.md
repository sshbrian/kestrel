# Kestrel

**Vector search at SIMD speed. One Rust engine, phone to cloud.**

[![CI](https://github.com/sshbrian/kestrel/actions/workflows/ci.yml/badge.svg)](https://github.com/sshbrian/kestrel/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Kestrel is an AI-native, vector-first, multimodal database that runs on any
device. One Rust codebase ships as an **embedded library** (phone, edge,
laptop, browser via WASM), a **single-node server**, and a **distributed
lakehouse service** over object storage — same segment format, same query
planner, same semantics at every tier.

> **Status: pre-alpha.** The design is public; the engine is being built
> kernel-first. Every performance number in this repo comes from an open
> benchmark harness in-tree that you can re-run. No harness output, no claim.

## Try it

```sh
git clone https://github.com/sshbrian/kestrel
cd kestrel
cargo run --release -p kestrel-core --example demo   # exact search, timed, recall-checked
cargo bench -p kestrel-kernels                        # the kernel numbers below, on your CPU
cargo test --workspace --release                      # differential tests: SIMD vs scalar
```

### Library example

Ten lines, copy-pasteable — save as `crates/kestrel-core/examples/quick.rs`,
then `cargo run --release -p kestrel-core --example quick`:

```rust
use kestrel_core::{FlatIndex, Metric};
fn main() {
    let mut idx = FlatIndex::new(4, Metric::Cosine); // vocab: fast, exact, search, vector
    idx.insert(&[0.0, 1.0, 1.0, 1.0]); // "Exact search scores every vector against the query."
    idx.insert(&[0.0, 0.0, 1.0, 1.0]); // "Graph search walks a neighbor instead of scanning every vector."
    let hits = idx.search(&[1.0, 1.0, 1.0, 1.0], 2); // "Fast exact search over a vector index."
    for (id, score) in hits {
        println!("{id}: {score:.3}");
    }
}
```

The vectors are bag-of-words counts over the vocabulary `fast, exact,
search, vector`: entry `i` counts how often word `i` occurs in the sentence.
Expected output:

```
0: 0.866
1: 0.707
```

Full version with a 12-sentence dataset:
`cargo run --release -p kestrel-core --example sentences`
(dataset: `crates/kestrel-core/examples/data/sentences.txt`).
Expected output:

```
Query: Fast exact search over a vector index.
Top 3 (Cosine similarity, bag-of-words counts):
  0.800  A vector database stores an index for fast similarity search.
  0.671  Exact search scores every vector against the query.
  0.548  Exact search is the ground truth for approximate graph search.
```

The encoding is a toy: raw word counts, not neural embeddings. Kestrel ships
no embedding model; you supply `f32` vectors.

## What this is NOT yet

Pre-alpha scope, stated plainly. None of the following ships today; the
roadmap tracks what is planned as issues.

- **No approximate index.** Search is exact brute-force (`FlatIndex` over the
  SIMD kernels). HNSW is roadmap issue #1, not shipped.
- **No quantization or compression.** No product quantization (PQ, #2);
  every vector is a full `f32` array in memory.
- **No persistence.** No on-disk segment format (#3); the index is
  in-memory only. Nothing is saved or loaded.
- **No server, no distributed tiers, no object-store backend.** No
  single-node server, no lakehouse service, no Iceberg interop.
- **No hybrid search.** No BM25, sparse, metadata-filtered, or graph-hop
  query planning — dense exact search only.
- **No embeddings.** No embedding model, no tokenizer, no embed-on-ingest.
  You supply `f32` vectors.
- **No governance or agent surface.** No lineage, no row/ABAC policy inside
  retrieval, no MCP interface, no contradiction detection.
- **No WASM build.** The browser target is vision, not a shipped artifact.
- **No crates.io release.** `kestrel-core` and `kestrel-kernels` have no
  published versions on crates.io. The bare `kestrel` name on crates.io is
  an unrelated 2020 actuarial crate (v0.0.1), not this project. Publishing
  is a `needs-human` decision (charter §9), not decided by this checklist.
  The `v0.1.0` git tag marks the first engine slice in this repo, not a
  published package.

## Why

AI apps today stitch together a vector store, a keyword index, a metadata
database, a data lake, a graph store, and a separate embedding pipeline.
Every seam costs latency, consistency, and governance — and none of it runs
the same engine on a phone and across a petabyte of cloud data.

Kestrel's answer:

- **Hybrid search as one primitive** — dense ANN + sparse + BM25 + metadata
  filters + graph hops in a single cost-based plan.
- **Speed where it matters** — hand-tuned SIMD distance/quantization kernels
  (AVX2, AVX-512, NEON, SVE) with runtime dispatch, each differential-tested
  against a scalar Rust reference. Everything else stays safe Rust.
- **Multimodal embed-on-ingest** — text, image, audio, video stored next to
  chunks and embeddings, re-embedded in the background on model change.
- **Open lakehouse storage** — Iceberg-style tables on object stores:
  snapshots, time travel, stateless compute.
- **Governance inside retrieval** — lineage on every embedding; row/ABAC
  policy enforced *inside* ANN traversal.
- **Agent-native** — MCP tool calls, governed write-back, contradiction
  detection.

Full design: [docs/architecture.md](docs/architecture.md).

## Benchmarks

Speed is the brand; benchmarks are the proof. Methodology:

- Open harness, one command (`cargo bench`), fixed hardware profile recorded
  with every result.
- Recall@k reported next to every throughput number.
- Losses published alongside wins; competitors at vendor-recommended settings.
- Each SIMD kernel must beat its scalar reference in the harness or it is
  deleted.

First measured slice (Ryzen 7 9800X3D, AVX2+FMA vs scalar reference):
distance kernels run **6.2x–14.2x faster** than plain Rust across
128–1536 dimensions. Full table, hardware profile, and how to re-run:
[benchmarks/README.md](benchmarks/README.md). Kernel design notes:
[docs/kernels.md](docs/kernels.md).

## Roadmap

Near-term engineering order (tracked as GitHub issues):

1. Distance kernels: scalar reference → AVX2/NEON dispatch → differential tests.
2. Exact top-k search slice + CLI demo + recall/QPS harness.
3. Segment format v0, HNSW + PQ indexes.
4. Hybrid planner (dense + BM25 + metadata), embed-on-ingest, single-node server.
5. Object-store backend, distributed preview, Iceberg interop, MCP interface.

How this project launches and earns its claims:
[docs/launch-plan.md](docs/launch-plan.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Kernel work has one hard rule: a
scalar reference implementation and a differential test land in the same PR
as any optimized path.

## License

Apache-2.0. See [LICENSE](LICENSE).
