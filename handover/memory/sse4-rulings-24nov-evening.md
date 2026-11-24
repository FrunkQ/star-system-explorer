---
name: sse4-rulings-24nov-evening
description: Alex's rulings of 24 Nov 2026 afternoon and evening (CI add-only test lines, frozen physics golden, golden order, composite band, hazard map, seas, CoolProp, call me)
metadata:
  type: project
  modified: 2026-11-24T18:08:58.293Z
---
Recorded by the handover thread from the coordinator's relays, 24 Nov 2026. Also in handover/START-HERE.md.

- **CI test lines (Alex 18:07 UTC, "yes - if it makes your job easier - do it!")**: builders may ADD lines to the CI checks file (`.github/workflows/ci.yml`) so new browser tests run automatically. Adding only: never removing, skipping or loosening a check. Every other CI workflow or merge-check edit still needs Alex's own yes.
- **Frozen physics golden (Alex 18:07 UTC)**: the shared contract golden may run on a frozen copy of the base-physics pack, with the live pack's loading tested separately. Kernel thread builds it after the barycentre PR.
- **Golden lane**: Alex agreed to split the golden by area (kernel thread: op list line 1 and unknown-op line 41 out of contract.jsonl first, then per-area files; no CI edit needed). Until then: one golden-changing PR merges at a time, in order of readiness, each merged with main, regenerated and green; tell the kernel thread when one opens and when it merges.
- **Data sheet UI copy**: after any change to the data sheet or `scripts/build-ui-reference.py`, merge main, run `python3 scripts/build-ui-reference.py`, check with `--check`, before merging (a stale copy turned main red 24 Nov, fix PR 445).
- **Composite band**: last entry of the band switch on every view; opens a drop-down to mix several bands at once.
- **Hazard map** (was "approach diagram"): for any hazardous body, a diagram of dose against distance, the energy to escape from there, with Park and Dive marks and the closest a given ship can go before it is too late; a GM figure that can be shown to players (XO design PR 433).
- **Seas**: three pieces in order (colour by the liquid's absorption, waves by liquid properties, whitecaps); test case a rogue planet in a void with a helium sea.
- **Pulsar glitches** in, as a small cited block `sse.pulsar.glitch`.
- **CoolProp** may be used in scratch work only, and only with Alex's own yes each time.
- **"Call me if there is a stop"**: if work is blocked on Alex, ask him rather than wait silently.
- XO is a package code only, never shown in the product (Alex queried the name).

Related: [[sse4-exotic-objects]], [[sse4-coming-together]], [[sse4-bands-table]]
- **Consensus for research (Alex 18:47)**: the Consensus research connector is available for deep research. On the new account Claude will have it directly (an account transfer), so use it yourself rather than writing briefs for Alex to pass on.
