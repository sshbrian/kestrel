# How Kestrel's distance kernels work

Vector search spends its life in a handful of tiny loops: inner product,
squared L2, cosine, Hamming. Kestrel's position is simple — those loops get
hand-tuned SIMD, everything else stays safe idiomatic Rust, and **benchmarks
decide what ships**. This page documents the design, the safety story, and
the measured results. Reproduce every number with:

```sh
cargo bench -p kestrel-kernels
cargo run --release -p kestrel-core --example demo
```

## Layout

All `unsafe` in the workspace lives in one crate, `kestrel-kernels`:

| File | Contents |
|---|---|
| `src/scalar.rs` | Plain Rust reference kernels. The correctness ground truth. |
| `src/simd_x86.rs` | AVX2+FMA kernels (`std::arch::x86_64` intrinsics). |
| `src/simd_arm.rs` | NEON kernels (`std::arch::aarch64` intrinsics). |
| `src/lib.rs` | Safe public API + runtime CPU dispatch. |

`kestrel-core` (and every future crate) is `#![forbid(unsafe_code)]` and
only sees the safe wrappers.

## Dispatch

Each public function checks the CPU once per call
(`is_x86_feature_detected!("avx2") && "fma"`; the std detection result is
cached, so this is an atomic load, not a CPUID) and routes to the best
compiled kernel, falling back to scalar. On aarch64, NEON is baseline for
the targets Kestrel supports, so no runtime check is needed.
`kestrel_kernels::active_backend()` reports which path a machine takes, and
the demo prints it — a benchmark number should never be ambiguous about
which code produced it.

Inside each kernel: four independent vector accumulators (to hide FMA
latency), unaligned loads (no alignment contract on callers), a scalar tail
for dimensions that are not a multiple of the vector width.

## Correctness: differential testing

Every optimized kernel is tested against its scalar reference across 22
dimensions (0, 1, primes, powers of two ±1, 384/768/1536) with a
deterministic PRNG, plus known-value and zero-vector cases.

One honest subtlety: SIMD sums lanes in a different order than a scalar
loop, so `f32` results differ in the last bits. The tests therefore use a
tight relative tolerance (`1e-4`) for float kernels — bit-for-bit equality
is not a property floating-point reductions can promise without serializing
the sum (which would defeat the purpose). Integer kernels (Hamming) are
exact and tested with `assert_eq`.

CI runs the full test suite in release mode on x86_64 (SIMD paths active)
and compile-checks the NEON path with
`cargo check --target aarch64-unknown-linux-gnu`. A scalar-only run is one
env var away for any machine without the features.

## Measured results

Hardware: AMD Ryzen 7 9800X3D, Windows, rustc 1.99.1, thin LTO. Mean
ns/call, Criterion, 50 samples. Full methodology in
[`benchmarks/README.md`](../benchmarks/README.md).

| Kernel | Dim | Scalar | AVX2+FMA | Speedup |
|---|---|---:|---:|---:|
| dot | 128 | 32.71 | 5.24 | 6.2x |
| dot | 768 | 282.59 | 26.69 | 10.6x |
| dot | 1536 | 577.34 | 40.75 | 14.2x |
| l2_sq | 128 | 35.09 | 5.22 | 6.7x |
| l2_sq | 768 | 282.24 | 29.65 | 9.5x |
| l2_sq | 1536 | 583.36 | 49.23 | 11.9x |

Why the gap widens with dimension: the scalar loop is bottlenecked by the
dependent-add latency chain (and strict FP semantics stop the compiler from
reassociating it), while the SIMD kernels run 32 independent lanes of FMA.
The interesting future comparisons are not scalar-vs-SIMD — they are
Kestrel-vs-other-engines at equal recall. Those land with the ANN indexes,
under the same rule: harness output or it does not ship.

End-to-end, the exact-search demo (`kestrel-core`, 10,000 × 128 vectors,
single thread) measures **1,882 QPS at p50 523 µs with recall@10 = 1.00**
against scalar ground truth. Most of that latency is now the full sort of
10k scores — the scan stopped being the bottleneck, which is the point of
the exercise. (Partial top-k selection is the obvious next optimization;
it is deliberately not in v0.1.0 so result ordering stays trivially
deterministic.)

## Rules for future kernels

1. Scalar reference and differential test land in the same PR.
2. Intrinsics first; hand-written assembly only if profiling shows a
   lasting gap over intrinsics (none has been needed yet).
3. If a kernel does not beat scalar in the harness, it is deleted.
4. AVX-512 and SVE paths join AVX2/NEON behind the same dispatch, gated on
   the same tests. Every path gets re-justified as compilers improve;
   SIMD is a replaceable optimization, never load-bearing architecture.
