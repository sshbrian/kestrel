# Benchmarks

Every number in this repo comes from the Criterion harness in
[`crates/kestrel-kernels/benches/kernels.rs`](../crates/kestrel-kernels/benches/kernels.rs).
Reproduce the table below with:

```sh
cargo bench -p kestrel-kernels
```

## Hardware profile (baseline run)

- CPU: AMD Ryzen 7 9800X3D (AVX2 + FMA path active; `active_backend() = "avx2+fma"`)
- OS / toolchain: Windows, rustc 1.99.1, release profile with thin LTO
- Data: deterministic xorshift64-generated `f32` vectors in [-1, 1), in-cache
- Criterion: 50 samples, 3s measurement; figures are mean ns per call

The scalar column is the plain Rust reference loop the SIMD paths are
differential-tested against. Rust does not auto-vectorize it (the
accumulator dependency chain plus strict FP semantics keep it scalar),
which is exactly why it is the honest baseline.

## Distance kernels — mean ns/call (lower is better)

| Kernel | Dim | Scalar | AVX2+FMA | Speedup |
|---|---|---:|---:|---:|
| dot | 128 | 32.71 | 5.24 | 6.2x |
| dot | 384 | 125.05 | 10.80 | 11.6x |
| dot | 768 | 282.59 | 26.69 | 10.6x |
| dot | 1536 | 577.34 | 40.75 | 14.2x |
| l2_sq | 128 | 35.09 | 5.22 | 6.7x |
| l2_sq | 384 | 131.91 | 11.64 | 11.3x |
| l2_sq | 768 | 282.24 | 29.65 | 9.5x |
| l2_sq | 1536 | 583.36 | 49.23 | 11.9x |
| hamming (popcnt) | 1024 bits | 7.83 | — | same path |
| hamming (popcnt) | 6144 bits | 43.04 | — | same path |

Hamming has no separate SIMD path yet: `u64::count_ones` compiles to
POPCNT, and it stays that way until profiling shows a reason to change.

## Rules

- A kernel that does not beat its scalar reference here gets deleted.
- New results replace this table in the same PR as the kernel change,
  with the hardware profile updated.
- Cross-engine comparisons (recall@k vs QPS) land with `kestrel-core`
  and follow the same policy: vendor-recommended settings, losses published.
