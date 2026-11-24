# Status: Atlas and editions

Owner: the thread "Build star atlas and editions" (24 Nov 2026). Covers: track AT (the Atlas view, its editions,
print, letterhead and ASCII art, margin notes, the interface kits). Owns `packages/views/atlas/`,
`packages/ui-contract/src/atlas/` and `packages/shell/src/atlas/`.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs
are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

**Design (accepted by Alex 24 Nov 2026 20:14 UTC, "yes to all"):** `handover/design/SSE4-atlas-and-reports-design.md`
(arrives with PR 478; also `/mnt/project-files/sse4/SSE4-atlas-and-reports-design.md`). Read its sections 1, 13, 14A
(Alex's notes, which win over the rest) and 16. Mockups: project files `sse4/handover-screens/AT-screens/`.

**Firm rules:** an edition changes words, order, look, jokes and letterhead, never a fact. A gap is drawn from the
template and the audience, never from the value: the kernel writes a placeholder only where a value exists, so the
Atlas never reads a placeholder (a placeholder and an absent fact read the same). Figures go through
`ui-contract/src/figures.ts`; nothing in the Atlas formats a number itself. The SSE3 joke file is on Alex's PC only:
ask him for it; never recreate his jokes from memory. The Weyland-Yutani logo stays out unless Alex says otherwise.

| Package | State | Log |
|---|---|---|
| AT1 the page model: blocks, the page builder (pure, over the audience's scene), contents and chapters, the fact catalogue, `sse.edition/1` with `extends`, the Survey atlas edition with templates for sector, system, star, world, moon, station | done | packages/AT1.md |
| AT2 the Atlas view on screen: page, contents, step, links, phone, both chart modes; screen document with a `rail` entry; "Open in Atlas" in the action list (navigation thread) | done; "Open in Atlas" asked of the navigation thread; Seen by one role waits on the plan's audiences reaching views | packages/AT2.md |
| AT3 editions as data: look, voice labels, gap styles, plates, sources; loader checks 1 to 5; the Edition switch (GM only, design 14A note 6). Exports `checkEdition(doc, docs) -> string[]` and a DOM-free `testRender` for Orac (agreed with "Build Orac, Minerva and Tars", who start AT11 when it lands) | next | |
| AT4 print under the view: this page, chapter, whole atlas; A4 and Letter; Save ink; sheet furniture; page turn for paper editions (14A note 5); Copy as text, Save page | not started | |
| AT5 gaps and redaction, "Show as withheld" (kernel `withheld` precision) | not started; kernel thread for `withheld` | |
| AT6 letterhead, brand `accent` and `motto`, built-in SSE3 logos (six; not Weyland-Yutani), ASCII art; the Company edition | not started; Run desk thread for the brand fields | |
| AT7 margin notes and The Guide edition (gadget look, cover every time; 14A note 4) | not started; needs Alex's joke file (`Mostly_harmless.txt`) | |
| AT8 Why on the Atlas, "How we know" | not started | |
| AT10 the GM's copy: pencil and ink on the page and in print | not started | |
| AT14, AT15 interface kits and the four starters | not started | |
| AT9, AT13 (Run desk), AT11, AT16 (Orac), AT15a (look transitions thread) | other threads | |

Seams agreed by message: the look transitions thread ("Build look transitions for screens", AT15a) builds
`ui-contract/src/delivery.ts` and `chart-grammar/src/deliver.ts`; the Atlas page is
`<article class="atlas-page" data-atlas-page>` with each block a direct child marked `data-deliver-item` and prose
and figures marked `data-deliver-text`; this thread makes the call on each step once that lands.
