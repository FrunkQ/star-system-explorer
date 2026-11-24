---
name: sse4-exotic-build
description: Track XO build state (thread "Build exotic stars and black holes"): what is built, PR order, what waits on the golden lane
metadata:
  type: project
  modified: 2026-11-24T18:13:02.714Z
---

Track XO (exotic objects) builder state, 24 Nov 2026. Status file handover/status/exotic-objects.md is the truth; logs handover/packages/XO1.md, XO17.md, XO18.md.

- XO1 part 1: crates/sse-gravity bend table + tests/golden/gravity.json (PR 443).
- XO1 part 2: nine sse.gravity.* formula blocks, parked in handover/packages/XO1-part2/ because any new base block changes tests/golden/contract.jsonl (reach report). Waits for the golden lane (coordinator's queue), or for the kernel thread's split of contract.jsonl.
- XO17 Composite: band switch "Composite" with tick list, presets as data (ui-contract composite-presets.json), CompositeStack.svelte wraps any view; on system chart, galaxy chart, globe. PR follows 443.
- XO18 part 1: crates/sse-hazard, the Hazard map reading (heat, belts, air, climbing out; Park, Dive, safe bands, moons). PR follows XO17. Part 2 = op hazard_map + sse.belts.latitude_falloff (golden lane, together with XO1 part 2); part 3 = screen.
- Alex yes 24 Nov 18:24: XO19 pulsating stars and XO20 clocks (design section 15, PR 458). XO20 means XO18 must show the clock rate at Park and Dive, time lost on a pass, and the thrust to hover (part 2, with the gravity blocks).
- Known: dose along a field line uses the equatorial value (upper bound), so polar dives read too harsh until the latitude fall-off block lands. MF7 belt doses make Ganymede deadly behind 1 g/cm² (design expected harsh).

**Why:** the account move on 25 Nov; a new thread must pick up from the repo. **How to apply:** land PRs in order 443 → XO17 → XO18 part 1; ask the coordinator for the golden-lane turn for XO1 part 2 + XO18 part 2. See [[sse4-exotic-objects]].
