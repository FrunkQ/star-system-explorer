# Status: Atlas and editions

Owner: the thread "Build star atlas and editions" (24 Nov 2026). Covers: track AT (the Atlas view, its editions,
print, letterhead and ASCII art, margin notes, the interface kits). Owns `packages/views/atlas/`,
`packages/ui-contract/src/atlas/` and `packages/shell/src/atlas/`.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs
are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

**Design (accepted by Alex 24 Nov 2026 20:14 UTC, "yes to all"):** `handover/design/SSE4-atlas-and-reports-design.md`
(also `/mnt/project-files/sse4/SSE4-atlas-and-reports-design.md`). Read its sections 1, 13, 14A
(Alex's notes, which win over the rest) and 16. Mockups: project files `sse4/handover-screens/AT-screens/`.

**Firm rules:** an edition changes words, order, look, jokes and letterhead, never a fact. A gap is drawn from the
template and the audience, never from the value: the kernel writes a placeholder only where a value exists, so the
Atlas never reads a placeholder (a placeholder and an absent fact read the same). Figures go through
`ui-contract/src/figures.ts`; nothing in the Atlas formats a number itself. The SSE3 joke file is on Alex's PC only:
it is in the public SSE3 repo FrunkQ/star-system-generator (`src/lib/catalogue/guideNotes.ts`); never recreate his jokes from memory. The Weyland-Yutani logo stays out unless Alex says otherwise.

**On main:** AT1 to AT4 (with AT4b), AT6, AT7 (with AT7c), AT14 and PC7 merged in PR 481 on 25 Nov 2026 08:26 UTC. AT8 part 1 merged in PR 543 (25 Nov 09:22 UTC). Next here, in order: AT15, AT5 (waits on the kernel's `withheld`), AT8 part 2 (the grant's own source and day).

| Package | State | Log |
|---|---|---|
| AT1 the page model: blocks, the page builder (pure, over the audience's scene), contents and chapters, the fact catalogue, `sse.edition/1` with `extends`, the Survey atlas edition with templates for sector, system, star, world, moon, station | done | packages/AT1.md |
| AT2 the Atlas view on screen: page, contents, step, links, phone, both chart modes; screen document with a `rail` entry; "Open in Atlas" in the action list (navigation thread) | done; "Open in Atlas" asked of the navigation thread; Seen by one role waits on the plan's audiences reaching views | packages/AT2.md |
| AT3 editions as data: Survey atlas, The Guide, Company; `vocab` and voice labels, gap styles, plates, sources; `checkEdition` (checks 1 to 5) and `testRender` (check 6) as pure functions for Orac; the GM's Edition switch; the look from the shell | done; Guide and Company looks are interim until AT7 and AT6 | packages/AT3.md |
| AT4 print under the view: this page, chapter, whole atlas; A4 and Letter; Save ink; sheet furniture; page turn for paper editions (14A note 5, the curl through AT15a's `curlTurn`, `pageTurn` and `resolveDelivery`); Copy as text, Save page; the print view (every page a small sheet with a tick box, quick picks, sheet count) and black and white toner ink (14A note 7, AT4b) | done, with the paper curl and drag turn through AT15a (25 Nov) | packages/AT4.md |
| AT5 gaps and redaction, "Show as withheld" (kernel `withheld` precision) | not started; kernel thread for `withheld` | |
| AT6 letterhead, brand `accent` and `motto`, built-in SSE3 logos (six; not Weyland-Yutani), ASCII art; the Company edition | done; `accent` and `motto` reach `SeatBrand` with the Run desk thread's PB6 to PB8 PR (read as optional until then) | packages/AT6.md |
| AT7 margin notes and The Guide edition (gadget look, cover every time; 14A note 4) | done; all 131 of Alex's lines in the Guide, Douglas Adams lines included, and Megadodo on its cover (25 Nov, no fan pack); his five figure lines back as plain jokes (over 100%, 25 Nov); one-liners by use case for the top and bottom of screens (`pickBanners`, 25 Nov); 20 topics, one of a topic to a page, a note once in a book, "Mostly Harmless (updated!)" on anything called Earth (14A note 8, AT7c); campaign-kept "Not this one" and the GM's Guide's notes next; Alex reads the note set before it ships | packages/AT7.md |
| AT8 Why on the Atlas, "How we know" | part 1 done (25 Nov): figures open their Why (how we know, then the kernel chain on the GM seat); endnotes in the Survey atlas, Source lines in Company, none in the Guide; text, Markdown and HTML carry them. Next: the grant's own source and day when the kernel's grant carries them | packages/AT8.md |
| AT10 the GM's copy: pencil and ink on the page and in print | done (25 Nov): what the table has not been told is in pencil (dotted, quieter) on screen and in print, read off the table's own scene; a place the table does not know pencilled whole; GM notes (the record's `gm-notes`) in their section; text and HTML carry both | packages/AT10.md |
| AT14 `sse.kit/1`, its loader and checks, palettes with a contrast check, the four starters as documents | done for the document and checks; "Interface" on the role card and swatches are the Run desk thread's (AT13) | packages/AT14.md |
| AT15 wearing a kit: the four starters on screen | not started; needs AT13 (parts declare controls) and AT15a | |
| PC7 (postcards track, owned here) the quote set: `by` and `source` on a margin line, 21 scientists' and astronauts' quotes in the base app, 51 others in `store-packs/space-quotes-fan` | done | packages/PC7.md |
| AT9, AT13 (Run desk), AT11, AT16 (Orac), AT15a (look transitions thread) | other threads | |

Seams agreed by message: the look transitions thread ("Build look transitions for screens", AT15a) builds
`ui-contract/src/delivery.ts` and `chart-grammar/src/deliver.ts`; the Atlas page is
`<article class="atlas-page" data-atlas-page>` with each block a direct child marked `data-deliver-item` and prose
and figures marked `data-deliver-text`; this thread makes the call on each step once that lands.

Agreed with the Run desk thread (AT13, PR 485): once 485 is on main, the Atlas view module declares
`switchable: PartControl[]` (contents, step, print, links; no edition on a player's screen, 14A note 6) and reads them
with `controlOn` from `ui-contract/src/part-controls.ts`; `kit.ts` then takes part controls from those declarations
and its own `controlsOn` is dropped. Agreed with the Orac thread (AT11, AT16): accepted drafts live in the campaign
pack `campaign.atlas` (kind `presentation`) under `provides.other.atlas`, ids starting `mine.`, read by `atlasDocsOf`.

Settled 25 Nov 2026: no Weyland-Yutani logo in the built-in set ("can wait for the game pack"); paper editions curl their pages from the corner (design section 14A, notes 2 and 5).
