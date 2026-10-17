# Contributing to Kestrel

Thanks for looking. Kestrel is pre-alpha and built kernel-first; the fastest
way to help is small, well-tested changes to the crates that exist.

## Ground rules

1. **Correctness before speed.** Every optimized kernel ships in the same PR
   as (a) a scalar Rust reference implementation and (b) a differential test
   proving the optimized path matches it. No exceptions.
2. **Benchmarks or it did not happen.** Performance claims in PRs must
   include `cargo bench` output (before/after) with hardware noted. A kernel
   that does not beat its scalar reference in the harness gets removed, not
   kept "for later."
3. **`unsafe` stays in the kernel crate.** All `unsafe`/SIMD code lives in
   `kestrel-kernels` behind safe wrappers with documented invariants. Other
   crates are `#![forbid(unsafe_code)]`.
4. **No fake numbers.** Docs and READMEs only carry figures the in-repo
   harness reproduces.

## Workflow

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo bench -p kestrel-kernels   # when touching kernels
```

- Open an issue before large changes (new index types, format changes).
- Keep PRs focused: one kernel, one feature, one fix.
- Commit messages: imperative mood, say what and why.

## Architecture pointers

- [docs/architecture.md](docs/architecture.md) — system design and roadmap.
- [docs/kernels.md](docs/kernels.md) — kernel design, dispatch, measured results.
- [docs/launch-plan.md](docs/launch-plan.md) — launch sequence and claim policy.
