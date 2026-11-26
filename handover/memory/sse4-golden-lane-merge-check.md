---
name: sse4-golden-lane-merge-check
description: Coordinator ruling 25 Nov 03:17 UTC: before merging any PR, check for golden or kernel-output changes and wait for a lane slot if so
metadata:
  type: feedback
  modified: 2026-11-25T03:18:07.592Z
---
Before merging any SSE4 PR, check whether it touches tests/golden/ (contract.jsonl or any per-area golden file) or changes kernel output the golden captures. If it does, don't merge: tell the coordinator and wait for a lane slot. Otherwise merge when green as usual. This holds until the coordinator lifts it.

**Why:** PR 459 merged appended golden lines out of turn. Together with PR 437, it pushed the kernel's green PR 468 back into conflict and cost an 80-minute CI run. Even append-only golden changes conflict with other golden PRs, because they all append at the end of the same file.

**How to apply:** run `git diff origin/main --stat -- tests/golden` before every merge, and treat any line there as a lane request.

Related: [[sse4-small-bodies]], [[sse4-light-track-local-ci]]
