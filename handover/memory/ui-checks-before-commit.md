---
name: ui-checks-before-commit
description: scripts/check.sh runs neither svelte-check nor the UI unit tests; UI PRs also need npm run ui:check and npm run ui:test or CI's ui job fails
metadata:
  type: project
  modified: 2026-11-21T16:42:05.071Z
---

`scripts/check.sh` (fmt, clippy, Rust tests) does not run `npm run ui:check` (svelte-check --fail-on-warnings) or `npm run ui:test` (node --test over ui-contract, renderer, skins, chart-grammar, agents, tests/ui/*.test.ts), but CI's "ui" job runs both. PR 66 failed on a readonly array passed to a Svelte prop; PR 78 failed `tests/ui/symbol-free.test.ts` (views may not write `.shape === "..."`: use a ui-contract helper such as `isShell`).

**Why:** both passed check.sh locally and still failed CI (21 Nov).
**How to apply:** before committing UI changes, run check.sh, `npm run ui:check`, `npm run ui:test`, and the e2e suite (`npm run ui:e2e` after `scripts/build-shell.sh`; run `scripts/build-wasm.sh` first after merging kernel changes, or the shell reports unknown atoms).
