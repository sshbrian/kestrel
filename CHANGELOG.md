# Changelog

All notable changes to Kestrel are documented here. This file tracks the
pre-launch checklist and what lands; it is not release notes for a published
release.

## [Unreleased]

### Pre-launch checklist (charter §9, October 2026)

- [x] README 10-line library example + real-sentences example dataset (#5)
- [x] README "What this is NOT yet" section (#6)
- [ ] macOS CI job (#7)
- [x] CHANGELOG.md (#8 — this file)

## Baseline

- Baseline commit: `71897df` (2026-10-17, "Add bot operations charter:
  human + Muse + Grok roles, handoffs, guardrails").
- Repo state at the baseline: exact top-k search (`FlatIndex`) over SIMD
  distance kernels, runnable demo, Criterion kernel benchmarks with the
  results table in `benchmarks/README.md`, README quickstart, bot
  operations charter. Earlier tag `v0.1.0` points at `a2fd529`
  ("First engine slice: exact top-k search (FlatIndex) over SIMD kernels,
  runnable demo").
