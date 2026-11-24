---
name: sse4-ship-views-locked
description: Alex's rule, 24 Nov 2026: play desks never change a ship or its look; ShipPlans has a `locked` prop for play screens
metadata:
  type: feedback
  modified: 2026-11-24T14:10:44.114Z
---
Alex, 24 Nov 2026 (relayed by the manifest thread, PR 391): play (the Explore and Run desks) changes what is aboard a ship, never the ship or its look. A ship's style and remix live on the construct as `construct.design` and are edited only on the Build desk. `ShipPlans.svelte` gained an opt-in `locked` prop (default false) that hides Style and Remix but keeps turning and framing.

**Why:** a ship's look is design, set on the Build desk, not something play changes.
**How to apply:** pass `locked` to any ship view put on a play screen.

Open point seen 24 Nov: the Traveller courier (`traveller.courier`) designs as 582 m long and 4.3 km across, because of 1.2 TW of drive waste heat. That is honest physics, but worth checking before players see it.

Related: [[sse4-atomic-rockets]]
