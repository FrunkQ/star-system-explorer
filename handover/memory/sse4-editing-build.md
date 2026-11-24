---
name: sse4-editing-build
description: Track ED build state (24 Nov 2026): owner thread, kernel ask order, units engine location and waiting sheet units
metadata:
  type: project
---
- Owner thread for ED0 to ED13 and ED0b: "Build planet and star editing" (status file handover/status/editing.md). Design: [[sse4-editing-design]].
- Kernel order agreed 24 Nov with "Build kernel resolve and rules": K13 pairs (BC1 first, then host checks, rehost, withdraw choices, preview_tree, Alpha Cen and Kepler-16), then K1 sweep, K2, K3, K17, K15, K16, K4, K12, K5, K7/K8, K9, K10/K11/K14.
- Units engine (ED1a part 1, PR 410): ui-contract/src/figures.ts (say, readFigure) over figures-said.json; no factors typed, all from the sheet's Units table via unit_rows in the generated copy.
- Sheet units all landed (PR 419, 24 Nov); WAITING list emptied in PR 427. After any sheet units change, rerun scripts/build-ui-reference.py (it writes unit_rows).
- Small bodies for Add (ED4/ED6): thread "Build asteroids and rings up close" (track SB) adds kernel op `make_small_body` {seed, kind asteroid|comet-nucleus|contact-binary|binary-pair, diameter_km or mass_kg, a_au, around?} in SB2 (golden lane), and recipes in packs/base-small-bodies/pack.json provides.other.small_bodies.recipes {id,label,kernel_kind,inputs[{id,default,unit (sheet unit id)}],citation}; ids small-body, comet, contact-binary, binary-asteroid.
- ED1a part 2 (24 Nov, in progress): card facts said by sayFact/readFact (figures.ts) with fact_about in figures-said.json; a world's surface reads °C first; pressing a unit steps the reading, kept in ws.unit_choices and the campaign save.
- Orac hook for ED8 "Can't find it?" (agents thread, PR 412, 24 Nov): shell `askOrac(request)` in App.svelte, or action `{"dialog":"ask","orac":"<request>"}`; show only when `linked(loadAgents(localStorage), "building")` (packages/agents/src/agents.ts), else link the data sheet editor. Wording "Add the gas <name> to the data sheet for <world>". Details handover/packages/A2.md part 5.
- Kernel BC1/K13 PR (pending golden lane, 24 Nov 17:01): resolved_system {branch} (pause previews: branch, branch_set, resolved_system with branch, discard); orbits_at sys_m, centre_m; bad_host on write; rehost {address,to,keep tidy|motion,at_s}; withdraw others leave (default, Alex 16:41) | take | hand; preview_tree {system,at_s,edits}; packs/base-starters (Alpha Cen, Kepler-16, base-sol shape).
