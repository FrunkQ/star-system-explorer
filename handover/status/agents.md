# Status: Orac, Minerva and Tars

Owner thread: "Build Orac, Minerva and Tars". Covers: A, ND4, ND5, ND9, SV7, AT11, AT16.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| A1 agents shell, bring your own key | done (parts 1 and 2): providers, keys on the device only and apart from the settings, ledger and cap, loaded-context guard, reconciled with Ovid's source (agents track); Ovid relicensing line in NOTICE confirmed by Alex 21 Nov | packages/A1.md |
| A2 Orac: change requests from a sentence, Ovid's draft, check and fix loop; every draft checked by SSE's own validators before it can be accepted | parts 1 to 7 done (agents track): physics (kernel-checked, filed on the Physics screen), lenses, atom issue drafts, styles, player screens, symbologies, data views, programs (J6), story sifters, boxes (J3), game tables as class schemes (with a per-row preview), data sheet substances (fictional pack for invented matter), all kernel-checked; Orac's handbook and the data sheet in its catalogue; `askOrac` entry point for the editing screens (part 5, PR 412, merged 24 Nov); triggers with their tag declarations (part 6, 24 Nov); object templates so a new program gets a body (part 7, 24 Nov). **Next for this row:** none planned; track A's next package is SV7 below | packages/A2.md |
| A3 Minerva: narrative from one audience's scene, every number checked against her facts, secrets never sent | done (agents track): scenes and the kernel's per-audience sifter hits | packages/A3.md |
| A4 Tars: support from SSE's help passages and the kernel's why?, every answer cites its source, humour setting | done (agents track); works with no key, from the passages | packages/A4.md |
| ND4 Orac reads a paper (PDF or text on the Orac tab, plan in words first) | done (24 Nov, agents track): PDF text read in the browser with no library (`pdf-text.ts`), the paper sent as labelled data, a checked plan in words before any draft, a too-long paper refused before sending | packages/ND4.md |
| ND5 one job as a set (boxes, flow node changes, light-table and data sheet rows, Equation and Model number blocks; accepted together, one Undo) | not started; needs MP11, the kernel's ND3 and the chart room's flow editor. Substance rows exist (A2 part 5) | |
| ND9 rehearsal on a second subject | not started; subject chosen by Alex 24 Nov: Astrophage and Taumoeba (Project Hail Mary), using Andy Weir's names; a shared pack waits until he has seen it. Needs ND2 (light through gas), ND3, ND5 and the life blocks (E) | |
| SV7 Orac's bulk reveals as change requests; Minerva writes from covers for the audience and from the truth for the GM, never stating a kept-back fact to a seat | part 1 done (24 Nov): Orac's `reveal` target, checked against the kernel's `known_to`, sent as the eye's `set_known` (one undo for a list). **Exact next package, SV7 part 2:** waits for the kernel's SV5 (precision `told`); Minerva from covers for a seat and from the truth for the GM, a check that no `withheld` value reaches a seat's text, and a cover story target. The Traveller pack thread does the pack half | packages/SV7.md |
| AT11 Orac drafts and edits Atlas editions and margin note sets as change requests (the loader's checks and test render, side-by-side preview); Minerva drafts a Guide's note as an authored note the GM accepts, never live on a player's screen (Alex, 24 Nov) | not started; waits for the atlas thread's AT1 and AT3 (`sse.edition/1` and its loader) and AT7 (`sse.margin-notes/1`). Design `design/SSE4-atlas-and-reports-design.md` sections 6.4, 12 and 12.8 | |
| AT16 Orac makes interfaces: kit, edition and note set targets; catalogue entries for parts, controls, palettes, filters and shaders; side-by-side preview; the `guide:interfaces` handbook entry | not started; comes last in the starters order (after AT13 controls, AT14 `sse.kit/1` and its loader, AT15 the four starters). Check: "Hauler but amber" gives a kit that passes every check before the GM sees it. Design section 11A.7 | |

## Decisions of Alex's held in this thread (written down 24 Nov for the account move)

- **Showcases for Orac.** The coloured white dwarf paper film (ND, Hollands et al. 2026, `design/SSE4-new-physics-demo.md`) is "the real test". The second is "create Astrophage and Taumoeba" (Project Hail Mary), the ND9 rehearsal subject. Expected shape: Astrophage as a fictional data sheet substance (stores energy as mass, absorbs light, emits its infrared line: ND2 and MP11 rows), as energy-gradient life (track E), dimming its star through a flow change (ND5) so the existing climate flows cool every planet; Taumoeba as a predator-prey box (ND3 maths). The centre shot is opening the pack to show plain, named, cited blocks, then nudging one.
- **Names.** Use Andy Weir's names (Astrophage, Taumoeba) and show him the result, hoping for a mention. A pack published on Explorers waits until he has seen it, ideally with his OK (the same rule as the 4X pack and its author).
- **Orac must know SSE4.** Alex, 24 Nov (editing design point 25): editing screens offer only defined gases and substances, with "can't find it, go here" or "ask Orac to create it" when a model is linked; Orac must be trained on the blocks, the development method and the UI tools. Done as A2 part 5; keep `packages/agents/src/orac-handbook.json` true as those change.
- **Ovid licence.** Alex owns Ovid and granted the relicensing of the lifted parts to Apache-2.0 (21 Nov); the line is in `NOTICE`.
- **Minerva and jokes.** Minerva drafts a Guide joke only as a note the GM accepts, never generated live on a player's screen (atlas design 6.4, accepted 24 Nov).
- **Merging.** Threads merge their own PRs when checks pass (Alex, 19 Nov); CI workflow edits need his own yes.

## Who else depends on this track

- "Build planet and star editing": calls `askOrac(request)` (App.svelte) from Add a gas and New substance, shown only when `linked(loadAgents(localStorage), "building")`; told 24 Nov.
- "Build kernel resolve and rules": `classify_preview` (PR 73) serves the table preview; SV1 (used by the reveal target) and SV5 come from there.
- "Build star atlas and editions": owns the edition, margin note and kit documents and their loaders; AT11 and AT16 here call those loaders as their checks.
