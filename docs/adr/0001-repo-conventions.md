# ADR 0001: repository conventions for V4

Status: accepted (18 Nov 2026), Alex may change.

- V4 lives in its own repository (FrunkQ/star-system-explorer), so it uses the root, not a `v4/` folder.
- Work happens on a working branch and lands one work package per commit series; once `main` exists each package arrives as a pull request.
- Build handover state (design copies, progress, per-package logs) lives in `handover/`. Nothing outside it may depend on it, so it can be deleted. `CLAUDE.md` has a one-line pointer to it, to delete with it.
- Crates follow handoff section 15. Addition: `sse-pack` (pack manifests and stacks, K5), because the layout names `/packs` content but no crate for the stack logic.
