# Kestrel

**Vector search at SIMD speed. One Rust engine, phone to cloud.**

Kestrel is an AI-native, vector-first, multimodal database that runs on any
device: an embedded library, a single-node server, and a distributed
lakehouse service share one Rust codebase, one segment format, and one
query planner.

> Status: pre-alpha. Design docs and the first kernel benchmarks are landing
> now. Every performance claim in this repo is produced by an open benchmark
> harness you can re-run — or it does not ship.

## Documents

- [Architecture & design](docs/architecture.md) — system design in Mermaid:
  tiers, hybrid query flow, embed-on-ingest, distributed I/O, kernel strategy.
- [Launch plan](docs/launch-plan.md) — how this project goes open-source and
  earns its speed claims.

## License

Apache-2.0. See [LICENSE](LICENSE).
