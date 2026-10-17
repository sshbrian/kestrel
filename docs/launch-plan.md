# Kestrel — Open-Source, Resume-Grade, Speed-First Launch Plan

## Goal

Turn the private `sshbrian/kestrel` docs repo into a **public open-source
project** that (1) reads as a credible startup-grade repo on a resume or
GitHub profile, and (2) is positioned on **raw speed** — "the fastest vector
search that runs anywhere" — as a claim *earned by reproducible benchmarks*,
not asserted by marketing copy. Source of truth for product facts:
`business-plan.md` (kept private; never published).

## Success Criteria

- Repo is public; **no commit in any branch ever contained `business-plan.md`**
  or its funding/pricing content.
- Apache-2.0 `LICENSE` present (per business plan §6); GitHub detects it.
- README leads with a one-line pitch, a runnable demo, and a benchmark table
  where **every number is produced by an open, one-command harness** in-repo
  (business plan §12 discipline: no claim ships until the harness reproduces it).
- At least one real speed artifact exists before any "fastest" language:
  SIMD distance kernels (AVX2/NEON) benchmarked against the scalar Rust
  reference, with differential tests proving identical results (§3.3).
- A tagged `v0.1.0` release a reviewer can clone, build, and run in minutes.
- Resume-ready: repo pinnable, with measurable bullets (QPS, recall@k, p99)
  that trace to benchmark output, plus a short technical write-up.

## Approach

**Speed is the brand; benchmarks are the proof.** The failure mode of
"fastest ever" repos is claims without data — reviewers and the ANN community
punish that instantly. So the launch is sequenced proof-first:

1. **Scrub, then flip public.** The repo currently has exactly one commit
   (`4c91a44`) containing the business plan. Because it is still private and
   single-commit, replace history with a fresh initial commit that never
   contained it — cheaper and safer than filtering later. The public copy of
   `docs/architecture.md` keeps only the technical diagrams (§1–7, roadmap);
   the pricing/funding sections (§8, §10 funding text) are removed, since
   they leak the business plan.
2. **Polish like a startup, not a homework repo.** Hero README (pitch, demo,
   benchmark snapshot, architecture diagram, roadmap), repo description and
   topics, `CONTRIBUTING.md`, issue templates, CI badge. Everything a
   reviewer checks in the first 30 seconds.
3. **Kernel-first engineering.** Before any database features: the hot loops
   the business plan already names (§3.3) — distance kernels on f32/int8/
   binary, then PQ scan — scalar reference first, `std::arch` intrinsics
   second, hand asm only if profiling shows a lasting gap. This is the
   smallest artifact that makes "fast" measurable, and it matches the
   M0–M3 roadmap (§9).
4. **Benchmark harness from day one.** Criterion micro-benchmarks per kernel,
   plus an ann-benchmarks-style recall@k-vs-QPS run (§12) once exact search
   exists. Publish losses as well as wins; competitors (from §4/§12: Qdrant,
   LanceDB, pgvector, SQLite-vec) only at vendor-recommended settings.
5. **Demo, release, write-up.** A tiny in-memory exact-search CLI over the
   kernels, an example dataset, a terminal recording/GIF, `v0.1.0`, and one
   "how the kernels work" post in `docs/`. This trio (code + numbers +
   writing) is what converts to resume value.

Explicit non-goals for this launch: distributed/cloud tiers, pricing,
fundraising material, graph/agent features (later roadmap phases, §9–§10).

## Steps

### Phase 0 — Scrub and go public (gate: nothing private leaks)

1. Preserve `business-plan.md` outside the repo (private local copy; it is
   already only in this repo and your machine).
2. Create scrubbed public `docs/architecture.md` (technical sections only)
   and update `README.md` to drop business-plan links.
3. Fresh history: new initial commit with `README.md`, `LICENSE`,
   `docs/architecture.md`, `docs/launch-plan.md`; force-push to replace
   `main` on the still-private remote; delete the old commit from reachability.
4. Add Apache-2.0 `LICENSE` (canonical text via `gh api licenses/apache-2.0`).
5. Flip visibility: `gh repo edit sshbrian/kestrel --visibility public`
   (requires the `--accept-visibility-change-consequences` flag).

### Phase 1 — Startup-grade repo polish

6. Rewrite `README.md`: tagline ("Vector search at SIMD speed. One Rust
   engine, phone to cloud."), status banner (pre-alpha, honest), demo and
   benchmark sections (empty until Phase 2 numbers exist — labeled as
   targets, never fake results), architecture diagram, roadmap link.
7. `gh repo edit`: description + topics (`rust`, `vector-database`, `simd`,
   `ann`, `embeddings`, `rag`). Add `CONTRIBUTING.md`, issue templates,
   `.gitignore` for Rust (`/target`).
8. GitHub Actions CI: `cargo test` + `cargo clippy` on push (badge in README).

### Phase 2 — Speed proof v0 (the resume centerpiece)

9. Cargo workspace: `kestrel-kernels` crate (scalar reference kernels:
   L2/IP/cosine f32, Hamming/popcount binary) — Rust 1.99.1 already installed.
10. Criterion benchmarks per kernel; commit baseline results to
    `benchmarks/README.md` with hardware profile.
11. AVX2 + NEON intrinsics behind runtime CPU dispatch; differential tests
    vs scalar on random + adversarial inputs; CI runs the scalar path so
    correctness never depends on the runner's CPU.
12. Publish the kernel table in the README: scalar vs intrinsics speedup per
    kernel, only from harness output. Any kernel that doesn't beat scalar
    gets dropped, per the yearly-review rule (§3.3) applied early.

### Phase 3 — First runnable engine slice

13. `kestrel-core`: in-memory exact (brute-force) top-k search over the
    kernels, load/query CLI, small example dataset, recall@k-vs-QPS harness
    run vs a brute-force NumPy/SQLite-vec baseline where practical.
14. Terminal demo recording, `v0.1.0` release with notes, seeded roadmap
    issues (HNSW, PQ, segment format v0 — §9 M0–M3) labeled for contributors.

### Phase 4 — Launch and resume packaging

15. `docs/kernels.md` write-up: design, dispatch, testing, measured results.
16. Pin the repo on your profile; resume bullets drafted **only** from
    measured artifacts (e.g., "AVX2 distance kernels: Nx vs scalar at
    recall@10 = R, harness in-repo" — N/R filled from Phase 2 output).
17. Optional announcement posts (dev blog / Show HN / r/rust) — only after
    Phase 3, when there is something runnable to show.

## Validation Plan

- **Leak check (highest risk):** before flipping public,
  `git log --all --oneline -- business-plan.md` returns nothing, and a grep
  across all commits for the business plan's funding and pricing figures
  (pattern list kept in a local, uncommitted checklist — never in this repo)
  returns nothing; then
  `gh repo view sshbrian/kestrel --json visibility` = `PUBLIC` and the same
  checks re-run against the remote.
- **License:** `gh repo view --json licenseInfo` reports Apache-2.0.
- **Correctness:** `cargo test` — every SIMD kernel equals scalar reference.
- **Speed claims:** `cargo bench` output is the *only* source for README
  numbers; benchmark doc records CPU, dataset, and command to reproduce.
- **Docs render:** Mermaid blocks re-parsed with the existing validation
  script (11/11 OK last run); README links clicked through on github.com.
- **Fresh-clone test:** clone to a temp dir, `cargo run --example demo`
  (or CLI equivalent) succeeds with no setup beyond Rust.

## Risks / Open Questions

- **"Fastest ever" backlash:** unqualified superlatives invite debunking.
  Mitigation: superlatives only attached to a specific harness result
  ("fastest in our harness on X, reproduce it: `cargo bench`"), losses
  published (§12). This is also what makes it resume-credible.
- **History rewrite:** `4c91a44` is discarded. Acceptable: one commit,
  still private, no collaborators. Rollback: keep a local `git bundle` of
  the original commit before rewriting.
- **Scope creep:** the full business plan is a 24-month roadmap (§10 risk #1).
  This launch covers only kernels + exact search; everything else stays
  roadmap issues.
- **Name collision:** "Kestrel" is a placeholder (business plan p.1);
  crates.io/GitHub org availability unchecked — check before Phase 4 posts.
- Open questions: None blocking. Tagline wording and logo are taste calls
  you can veto at Phase 1 without rework.
