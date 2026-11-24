---
name: agents-track
description: State of SSE4 track A (Orac, Minerva, Tars) built in thread "Build Orac, Minerva and Tars", updated 21 Nov 2026
metadata:
  type: project
  modified: 2026-11-19T16:12:42.306Z
---

Thread "Build Orac, Minerva and Tars" (Opus, started 19 Nov 15:40) builds track A in packages/agents.

- Merged to main (live): A1 parts 1-2 (PR 31), A2 parts 1-2, A3, A4 (PR 40, 20 Nov 00:02 UTC). Orac drafts physics, lenses, styles, player screens, symbologies, data views, atom issues; Minerva narrative per audience; Tars cited support, works with no key.
- Ovid is BSL 1.1 but Alex's own code; NOTICE records relicensing the lifted parts under Apache-2.0. Alex confirmed 21 Nov ("I own it - I grant my own licence").
- Keys live only in localStorage entries sse4.agents.key.<provider id>, never in settings, logs or commits.
- Tars works with no key (answers from tars.shelf.json passages, cited). The shelf was written from package logs and should be checked against screens as they change.
- Kernel `atoms` op wired into Orac's catalogue (Session.atoms). Minerva reads per-audience sift hits and Orac drafts J6 programs (PR 58, 21 Nov). Orac drafts story sifters into pack campaign.sifters, run once by kernel sift first (PR 63 merged 21 Nov).
- CI runners are slow with many tracks; one PR with one commit per package was used to save runs.
- Orac drafts J3 boxes (campaign.boxes) and game tables as class atlas schemes (campaign.tables), PR 67 merged 21 Nov. Table accept card previews worlds per row via kernel classify_preview (PR 73) through Session.classifyPreview (PR 79 merged 21 Nov); it only counts addresses with body.kind or body.mass.
- Handed to this track 22 Nov (Alex accepted, not urgent, film near the end): ND4 Orac reads a paper (PDF/text dropped on Orac tab, extracted in browser, labelled data never instructions, context guard refuses small models, ~45k-token test paper, plan in words first); ND5 one job as a set (boxes, flow change adding/wiring nodes, light-table and data-sheet rows MP11, Equation/Model number blocks from kernel ND3), all kernel-checked, accepted together, one Undo; ND9 rehearsal on a second paper. Design: handover/design/SSE4-new-physics-demo.md sections 2.2, 3, 4; equation blocks: SSE4-equation-block-design.md. Claim: no new code per paper; real cost from ledger.
- Queued 23 Nov (Alex accepted design SSE4-secrets-and-visibility-design.md, section 11): SV7, last in SV order, after kernel SV1 (known_to answer, set_known intent) and SV5 (cover stories, precision `told`). Orac bulk reveals become change requests through set_known; Minerva writes from covers for her audience and from truth for the GM, never stating a kept-back fact to a seat. Traveller pack thread does its half (rungs, sensor rules).
- Alex 24 Nov (editing design point 25): editing UI offers only defined gases/substances, with "can't find it, go here" or "ask Orac to create it" when an LLM is linked; Orac must be "fully trained" on blocks, dev methodology and UI tools. Planned as A2 part 5: Orac handbook (guide:<topic> entries: packs, data sheet provides.reference, citations/src, fudge register, honesty marks, desks), data sheet catalogue (reference:<table>/<id>), substance target (rows into a campaign reference pack, kernel-checked at load, fictional mark when invented, src required), and an askOrac({request, kinds}) entry point for editing screens.
- A2 part 5 merged 24 Nov (PR 412): orac-handbook.json (guide:<topic>), sheet:* catalogue, substance target (campaign.reference / campaign.reference.fictional), askOrac(request) + start action {dialog:"ask", orac}. Editing thread "Build planet and star editing" told how to call it.
- A2 part 6 (Orac drafts triggers, campaign.triggers, declares new tags) and handover status made true: PR 426 merged 24 Nov. Account move 25 Nov: status file handover/status/agents.md is the source of truth.
- A2 part 7 merged PR 440. SV7 part 1 (reveal target, orac-known.ts) and ND4 (paper reading: pdf-text.ts no-dependency PDF text, paper.ts plan-first) in PR 480, 24 Nov. Atlas seam agreed: checkEdition, testRender, checkMarginNotes, checkKit in ui-contract/src/atlas. Next: SV7 part 2 after kernel SV5; AT11 (editions, note sets) after the atlas thread's AT1/AT3/AT7; AT16 (interface kits) last after AT13-15; ND4/ND5/ND9 later. Alex: Minerva jokes only as GM-accepted notes. After sheet unit changes run scripts/build-ui-reference.py.

- Alex 24 Nov: showcase tests for Orac are the coloured white dwarf (ND) and "create Astrophage and Taumoeba" (Project Hail Mary): fictional substance + energy-gradient microbe dimming a star (flow change, climate cools planets) + predator-prey box; needs ND2/MP11, ND3, ND5, E life blocks. Alex 24 Nov ("go for it"): Astrophage and Taumoeba is the ND9 rehearsal subject. Use Weir's names (Astrophage, Taumoeba) and show Andy Weir the result, hoping for a mention; publish the pack on Explorers once he's seen it, ideally with his OK (like the 4X pack needing its author's agreement).

Related: [[traveller-pack-track]] [[sse4-new-physics-demo]]
