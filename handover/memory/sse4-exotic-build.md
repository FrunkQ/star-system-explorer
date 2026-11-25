---
name: sse4-exotic-build
description: Track XO build state (thread "Build exotic stars and black holes"): what is built, PR order, the golden chunk's area
metadata:
  type: project
  modified: 2026-11-25T09:03:01.561Z
---

Track XO (exotic objects) builder state, updated 25 Nov 2026 09:10 UTC. Status file handover/status/exotic-objects.md is the truth; logs handover/packages/XO1.md, XO17.md, XO18.md.

- MERGED 25 Nov 09:05 UTC, PR 443 (squash 73acc980): XO1 part 1 (crates/sse-gravity bend table, tests/golden/gravity.json), XO17 Composite on every view, XO18 part 1 (crates/sse-hazard reading).
- Golden chunk (next PR, same branch claude/project-thread-ri0k6m restarted from main): XO1 part 2 (ten sse.gravity.* blocks), sse.belts.latitude_falloff, op hazard_map, the Hazard map screen (packages/views/hazard), XO18 part 3 (phone) and part 4 = XO20 clocks at Park and Dive, time lost on a pass, thrust to hover. Built on local branch xo-golden3 in worktree /home/user/sse-golden; patches also in project files sse4/handover-screens/XO-screens/xo-golden-chunk/.
- Golden lane after the split (PR 536): the hazard cases live in their own area tests/contract/areas/exotic.json, golden tests/golden/areas/exotic.jsonl; core contract.jsonl untouched, frozen fixture pack not re-copied; ten appended cannot_follow lines in tests/golden/contract-reach.txt. Told the coordinator it is area-only, so it merges in parallel when green.
- Alex yes 24 Nov 18:24: XO19 pulsating stars and XO20 clocks (design section 15, PR 458).
- Known: MF7 belt doses make Ganymede deadly behind 1 g/cm² (design expected harsh).
- Local trap: main checkout's node_modules once became a symlink to itself (25 Nov 08:00); fix with rm node_modules && npm ci.

**Why:** the account move on 25 Nov; a new thread must pick up from the repo. **How to apply:** if the golden chunk PR is not open, rebuild from the patches onto main, re-bless (sse-atoms, export_bindings, kernel contract) and run the full local set first. See [[sse4-exotic-objects]], [[sse4-light-track-local-ci]].
