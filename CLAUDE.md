# Working in this repo

- Build state: `handover/START-HERE.md` (read it first) is the index; each track's packages, done and next, are in `handover/status/<track>.md`. Only a track's owner thread edits its status file.
- Order of joining up, and the lanes for changes that touch many blocks: `handover/design/SSE4-coming-together.md`.
- Standing rules: `handover/design/SSE4-build-plan.md` sections 1 and 2. Authority: `handover/design/SSE4-design-handoff.md`.
- Interface style for every screen ("dynamic minimalism", motion and popups): `handover/interface-style.md`.
- Top-level layout (place line, instrument rail, Workshop, desks, depth, start sheet, tours): `handover/design/SSE4-top-level-ui-and-onboarding.md`. A new GM screen gives itself a `rail` entry in its screen document; never add a tab.
- Commits authored as `FrunkQ <frunk@frunk.net>`. One work package at a time; its checks pass before the next starts.
- Deterministic Rust code: no wall clock, no HashMap/HashSet, no std transcendental maths (use `sse_core::math`), no fused multiply-add. Clippy enforces this (`clippy.toml`).
- Before committing: `scripts/check.sh` (fmt, clippy -D warnings, all tests).
- Golden hashes (`tests/golden/`) change only on purpose, with the reason in the package log.
