---
name: sse4-matter-audit-ui-prs
description: Run the typed-copy matter audit locally before every PR, UI-only ones too; CI's rust job runs it on main and a slip turns main red
metadata:
  type: feedback
  modified: 2026-11-24T15:25:40.434Z
---
On 24 Nov 2026, PR 396 typed `30 * 86400` in renderer rings.ts. CI skipped the rust job for that UI-only PR. The rust job runs `scripts/audit-matter.py --closed` on main and on every later PR, so main went red and other threads' PRs (403, 404, 406) failed on it. Three threads raced to fix one line.

**Why:** the audit catches typed constants in TS as well as Rust, but CI runs it only in the rust job, which is skipped when no Rust changes.
**How to apply:** before opening any PR, run `python3 scripts/audit-matter.py --closed`, even for UI-only work. Read constants with `constant("day")` and the like from ui-contract. If a slip does land, tell the coordinator once and agree on one line, so the parallel fixes merge cleanly.
