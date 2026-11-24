---
name: sse4-repo-tidy
description: SSE4 repo size tidy-ups Alex approved 23 Nov (screenshots out, split big files, drop old Rust copies)
metadata:
  type: project
  modified: 2026-11-23T22:49:22.669Z
---
Measured 23 Nov 2026: main ~250k lines code+tests (34k on 18 Nov), ~290 PRs merged; Rust 152k, UI 58k, packs 92k. Alex said yes (22:46) to three tidy-ups:
1. Screenshots never committed. They live in project files at sse4/handover-screens/<PKG>-screens/ (PR 338 moved 241 PNGs, 73 MB; gitignore blocks them). History not rewritten (needs Alex's yes).
2. Split crates/sse-kernel/src/lib.rs (5.9k lines) and packages/shell/src/App.svelte (2.7k) ONLY after the run-through work lands (kernel and chart room threads edit them).
3. Remove crates/sse-atoms/src/test/ (2.4k lines of pre-formula Rust kept as bit-for-bit oracles) once golden checks are trusted.
**How to apply:** 2 and 3 are queued; check with the owning threads before starting.
