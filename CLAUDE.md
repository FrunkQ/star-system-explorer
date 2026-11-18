# Working in this repo

- Build state, what is done and what is next: `handover/START-HERE.md` (read it first).
- Standing rules: `handover/design/SSE4-build-plan.md` sections 1 and 2. Authority: `handover/design/SSE4-design-handoff.md`.
- Commits authored as `FrunkQ <frunk@frunk.net>`. One work package at a time; its checks pass before the next starts.
- Deterministic Rust code: no wall clock, no HashMap/HashSet, no std transcendental maths (use `sse_core::math`), no fused multiply-add. Clippy enforces this (`clippy.toml`).
- Before committing: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Golden hashes (`tests/golden/`) change only on purpose, with the reason in the package log.
