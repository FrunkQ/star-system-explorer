---
name: sse4-highways
description: SSE4 "space highways" (V3 routes between stars): one record kind "link", plan HW1-HW4, Alex's 23 Nov rulings and open questions
metadata:
  type: project
---

Alex 23 Nov 19:21-19:23: SSE3 had "space highways" (links between stars, e.g. a stargate route); SSE4 must have them and build on them. A link's ends follow the same FTL start/end conditions as options (clear of a well, L point, gate, anywhere) and may also sit on, over or in a planet (surface site, orbit, inside), reached by real climb/descent.

Plan (galaxy and travel thread, posted 23 Nov): one mechanism, a **link**: two ends (any address, each with FTL Place conditions), name, kind label, dates, optional own FtlRule (time, cost, who may use), MP16 counted values ("2 jumps", Traveller thread owns MP16), audience (who knows).
- HW1 record kind (kernel thread with travel thread); V3 importer maps `v3.routes` (now kept raw as v3.routes kept value).
- HW2 voyage finder legs along links; FTL condition "only along links".
- HW3 drawn under HUD on Voyages, Your systems, sector map, galaxy chart; gate marks on system chart.
- HW4 Build desk authoring.
Later: territory grows along links; 4.1 economy prices trade.

Alex 23 Nov 19:25 answered: yes to 1 (ends anywhere incl. on/over/in a world), 2 (two-way default, one-way option), 4 (rule-less link = charted lane, ship's own drive, finder may prefer). 3: links can be secret, but secrecy shows through ONE improved general visibility control used across the app (V3 had a small visibility icon by each fact; make it more intuitive, in far more places, subtle). Store who-knows on the link; use the record's existing GM-only / known-to-players meanwhile; the control is a separate design.
Related: [[sse4-second-ui-track]] [[sse4-example-packs]]
Secrets design accepted 23 Nov 19:42 (track SV, handover/design/SSE4-secrets-and-visibility-design.md, Alex "a work of genius"): pencil (GM only, thin dotted) and ink (known) on charts, a three-state eye. This thread owns its half of SV3: pencil and ink on links (HW3), voyages, sector and galaxy charts, using the renderer's shared draw style; waits on SV1 (kernel known_to/set_known) and SV2 (chart room eye). Link "who knows" feeds known_to; the voyage finder must use only links the audience knows.
