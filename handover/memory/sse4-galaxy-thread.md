---
name: sse4-galaxy-thread
description: Thread "Build galaxy charts and histories" (split from travel 24 Nov 2026): what it owns, boundaries agreed, state
metadata:
  type: project
  modified: 2026-11-24T17:36:59.417Z
---
Started 24 Nov 2026 17:15 at Alex's ask for more parallel builders. Status file handover/status/galaxy.md (owns U6, U8, V9 flat maps, GH8, V3 gaps 17 and 39). Travel thread keeps V8, voyages, sse-nav, voyage.ts/party.ts and renames its file to travel.md.
- Boundaries: "Build cosmic web view" owns foam-map/Web3D.svelte and renderer web-cube.ts (FoamMap.svelte and cluster map stay mine; small hunks with heads-up OK). Navigation ("Build object selector and menus") owns nav/ and chart-grammar Picker; they put summaryWords on picker system rows; never edit nav/. Renderer keeps GH9 and GalaxyScars.
- PR 431 (24 Nov): data-stellar-halo fix, system-summary.ts, your-systems.ts (readCampaignSystems loader), Your systems view (galaxy scale rail, screen `systems`), GH8 leadingArmWords (hidden until kernel GH3 leading_arm mark, golden lane, thread "Build galaxy arm fix and atmosphere advice"), U8 Tell it with Minerva.
- PR 431 MERGED 25 Nov 02:06 UTC (also routes, grid, home star cell, V9 Find a home, U8 first event starter, figures via sayNumber). Waiting: kernel `from_star` on run_formation/adopt for "Grow a system here" (golden lane, bundled with class_sample); kernel GH3 leading_arm mark for GH8 e2e; PR 487 pencil and areas kit to use on galaxy chart and Your systems.
- PR 515 MERGED 25 Nov 04:06 UTC: pencil and ink on Your systems (Build/Run desks), map layers from holdings_in (shows only when a pack declares an area layer; none ships yet).
- PR 560 MERGED 25 Nov 11:11 UTC: GH8 part 2, the leading-arm line reads the kernel's real marks (only strength >= 0.25, LEADING_ARM_SEEN, matching kernel seen_strength). from_star landed (PR 546). "Grow a system here" built on the branch (commit ccc2424, adds optional ViewContext.adopted and FormationRequest.from_star), no PR yet: waits on two kernel fixes asked of "Build kernel resolve and rules" 25 Nov 12:40 (systems_of lists only root+1 sys addresses, so grown deep systems are unlisted; run_formation/resolved_system with from_star OOM in wasm serializing core_history steps).
- Citation corrections listed for sse-cosmos and base-cosmology belong to the kernel thread, not here.
- Done: mount NamesHere (names thread, from packages/views/reference) as a "Names here ›" door on Your systems once it merges.
Related: [[sse4-second-ui-track]] [[sse4-galaxy-histories]]
- Grand zoom (25 Nov): this thread builds GZ5, GZ6, GZ7 (Stage B, Local Group) and the screen halves of GZ12/GZ13; design thread "Design the grand zoom" designs only. PR 581 = GZ5 part 1 (3D Galaxies nearby from sky_from_here galaxies, no stars). Grow commit ccc2424 reverted on the branch (b0fc2b8) to merge GZ5 alone; re-apply (cherry-pick ccc2424) after the kernel's from_star age + systems_of fix lands (~14:00 25 Nov).
