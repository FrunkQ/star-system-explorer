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
- ED1a part 2 (24 Nov, in progress): card facts said by sayFact/readFact (figures.ts) with fact_about in figures-said.json; a world's surface reads °C first; pressing a unit steps the reading, kept in ws.unit_choices and the campaign save.
- Orac hook for ED8 "Can't find it?" (agents thread, PR 412, 24 Nov): shell `askOrac(request)` in App.svelte, or action `{"dialog":"ask","orac":"<request>"}`; show only when `linked(loadAgents(localStorage), "building")` (packages/agents/src/agents.ts), else link the data sheet editor. Wording "Add the gas <name> to the data sheet for <world>". Details handover/packages/A2.md part 5.
- Kernel BC1/K13 (merged PR 468): resolved_system {branch}; rehost; withdraw others leave (default) | take | hand; preview_tree; packs/base-starters.
- Figures ledger (tests/ui/figures-ledger.json, PR 479 and 490, 24 Nov): only a file going UP fails; a row above the code is a diagnostic. When parallel merges push a file up, its owner may raise the row with WRITE_LEDGER=1 and the reason in the PR. Use sayNumber(x, sig, {decimals}) for plain numbers.
- ED12 part 1 and ED7 part 1 (PR 498, 25 Nov): `box` in handles.json (parts, chips mirror/flip/lock/resonance, results), Box.svelte, Ring.svelte, ui-contract box.ts, Session.orbitStability. Handles move on any what-if; "Try changing it" on Explore opens "EXAMPLE · change anything". BC1 merged (PR 468, 25 Nov 04:55): the row now reads resolved_system {branch} (PR 526); orbits_at and orbit_stability have no branch, so the box hides year and verdict on a what-if.
- SB kernel op MERGED (PR 459, 25 Nov), shape changed: make_small_body {system, address, kind asteroid|comet-nucleus|contact-binary|binary-pair, diameter_km or mass_kg, a_au, e?}; made from the system's nearest belt or cloud; answer small_body_is {body, made_from}; icy answers carry activity (coma, tails). Recipes inputs [{id, default, unit km|au, kind}]. Input to ED4/ED6.
- 25 Nov 07:00 to 08:00: PR 539 (ED5 Reach it via scratch branch `reachIt`/`resolveWith` in ui-contract/src/reach.ts; ED1 3b held answer on pause). Then ED5 2a (`reach` in handles.json), ED4 1 (ws.adding ghost), ED8 1 (AirBox, commitFields: one set_values, one Undo), ED13 1 (Sheet.svelte). Authored air mix does NOT change the climate yet (K7). Titan has no pressure in base-sol, so no Titan dot on the Air strip.
- 25 Nov to 13:30: PRs 542-574 merged (sheet, Could be, ledger 202, sweep ED2 p1). PR 584 (ED9 Makeup box, class marks) open. Gotcha: kernel ops that edit a scratch branch (sweep, expects) must pass at_s = campaign now, else a main-line edit at now (any Undo) hides the swept value. Class regions need all five makeup.* parts set.
- 25 Nov early: PRs 518, 521, 525, 526 (moon wall, moon recipes, phone dock, what-if figures). make_small_body is a READ: adding one needs a node kind the resolve accepts.
