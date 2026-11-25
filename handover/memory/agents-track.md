---
name: agents-track
description: State of SSE4 track A (Orac, Minerva, Tars) built in thread "Build Orac, Minerva and Tars", updated 25 Nov 2026
metadata:
  type: project
  modified: 2026-11-19T16:12:42.306Z
---

Thread "Build Orac, Minerva and Tars" builds track A in packages/agents. Source of truth: handover/status/agents.md (rows A, ND4, ND5, ND9, SV7, AT11, AT16).

- Merged: A1-A4, A2 parts 5-7 (handbook guide:<topic>, sheet:* catalogue, substance target, askOrac, triggers), SV7 part 1 + ND4 (PR 480), AT11 + AT16 (PR 545, 25 Nov 09:25 UTC: Orac kit/edition/note set targets checked by the atlas loaders, kept in campaign.atlas; parts from partsOf(PLAYER_STACK + ATLAS_CONTROLS); Minerva Guide's note kept GM-only, tag guide-note).
- Next: SV7 part 2 after kernel SV5 (covers, precision told); ND5 one job as a set; ND9 rehearsal. Not yet: side-by-side preview for editions/kits; Atlas reading guide-note notes.
- Keys only in localStorage sse4.agents.key.<provider id>. Tars works with no key (tars.shelf.json; JSON strings must not contain raw double quotes).
- Ovid code relicensed by Alex under Apache-2.0 (NOTICE).
- Local checks pattern: worktree on the dependency branch, rebuild wasm (npm run build:client) and shell (scripts/build-shell.sh) whenever main moved, then ui:check, ui:test, tests/ui/{orac,agents,minerva,atlas}.e2e.mjs. A stale wasm gives false failures (reach.test.ts).
- Alex: Minerva jokes only as GM-accepted notes.

- Alex 24 Nov: showcase tests for Orac are the coloured white dwarf (ND) and "create Astrophage and Taumoeba" (Project Hail Mary): fictional substance + energy-gradient microbe dimming a star (flow change, climate cools planets) + predator-prey box; needs ND2/MP11, ND3, ND5, E life blocks. Alex 24 Nov ("go for it"): Astrophage and Taumoeba is the ND9 rehearsal subject. Use Weir's names (Astrophage, Taumoeba) and show Andy Weir the result, hoping for a mention; publish the pack on Explorers once he's seen it, ideally with his OK (like the 4X pack needing its author's agreement).

Related: [[traveller-pack-track]] [[sse4-new-physics-demo]]
