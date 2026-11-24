---
name: ci-runs-once-per-pr
description: SSE4 CI (ci.yml) changes need Alex's own yes; the 19 Nov runner-queue problem and the pending speed-up decision
metadata:
  type: feedback
  modified: 2026-11-19T16:54:06.221Z
---
Changes to `.github/workflows/ci.yml` (the checks that guard merges) need Alex's own approval, even though main is otherwise ours to merge. On 19 Nov 2026 the kernel thread bundled a ci.yml speed-up into PR 28 and the coordinator had it reverted (commit 14330f6), because the same change was already in front of Alex as a decision card (full speed-up, safe-only, or leave as is) from the "Verify physics citations" thread.

**Why:** CI guards merges; editing it alongside feature work bypasses Alex's say on the gate.
**How to apply:** never put CI edits in feature PRs; route them as their own decision. Background: about 15:59 UTC on 19 Nov no Actions runs started at all (maybe an account limit only Alex can see); every PR-branch push ran ci.yml twice (push plus pull_request). Cancelling your own stale queued runs is fine. Related: [[sse4-build-conventions]].
