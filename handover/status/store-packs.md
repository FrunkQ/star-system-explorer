# Status: store packs (track SP)

Owner thread: "Design store packs" (Fable, design only, 27 Nov 2026) for the note. Explorers builder: thread "Build store packs on Explorers" (SP14 to SP17, Explorers halves).
Design: `design/SSE4-store-packs-design.md`, **ACCEPTED by Alex 27 Nov 2026 12:02 to 12:05 UTC**; his rulings are in the note's section 13A (question 1 changed: kind is a label read from the contents, nothing refused). Builders may take the packages in order.

What it is, in one line: one way to make, find, stack, update and publish a pack. Kind becomes a label worked out from the contents; the shelf is read from the contents; one `about` block; prerequisites fetched from Explorers on one "this needs" sheet and checked by fingerprint; one layering rule for data as for flows; campaigns pin exact versions and updates are offered, never applied unasked; Make a pack and Publish from the Packs screen; V3 items marked and opened through the importer.

It builds on, and does not replace: paid packs (PP, PK), compound ships' parts packs (CM16 to CM18), detail expansion packs (RW8), the Open door, and Explorers D-98 and D-99.

Only the owner thread edits this file. Build threads update their rows in the same PR as the package.

| Package | State | Log |
|---|---|---|
| SP1 kind as a label worked out from contents | accepted, not started | |
| SP2 one layering rule for data | accepted, not started | |
| SP3 the `about` block, `suggests`, `changes` | accepted, not started | |
| SP4 resolve route and the "this needs" sheet | accepted, not started | |
| SP5 library and pins; updates | accepted, not started | |
| SP6 Packs screen in three parts | accepted, not started | |
| SP7 Make a pack, with "Add to my ship pack" | accepted, not started | |
| SP8 Connect and Publish from the app | accepted, not started | |
| SP9 `scripts/pack.mjs` and the CI check | accepted, not started | |
| SP10 V3 marks and "Open in SSE4" | accepted, not started | |
| SP11 shared campaigns carry pins, not packs | accepted, not started | |
| SP12 the V3 upgrade pass, on Alex's go, once V4 is settled | accepted, not started |
| SP13 the sale scanner on Explorers (priced packs only) | accepted, not started |
| SP14 badges on the store, from the shared table | Explorers half done (0.72.0, starsystemx-creator-hub PR 16, D-103): `$lib/itemBadges.ts` table, on shelf cards, item pages, V3 cards and `/api/v4` + `/api/maps` as `badges`. Real data and Free only wait on SP3 (`downloaded_data`); prices on PP. App side with SP6 | thread "Build store packs on Explorers" |
| SP15 edit listing (owner edits title, summary, fan-setting, cover) | Explorers half done except cover (0.70.0, PR 14, D-101): any maker edits title, summary, fan setting; take off the shelf (`withdrawn`, still served) and back. Needs migration 0045 run by Alex for withdraw. V4 items have no cover yet. App's card button with SP8 | thread "Build store packs on Explorers" |
| SP16 admin functions cover V4 items; backup first | done (0.68.2 PR 12 backup with V4 file copies; 0.69.0 PR 13 every admin page, D-100). Finding for Alex: account deletion cascades to its v4_items | thread "Build store packs on Explorers" |
| SP17 V3 starmaps and systems as shelves | done (0.71.0, PR 15, D-102): two V3 shelves on /shelves, one search, shared contains pills, Browse leads from them | thread "Build store packs on Explorers" |
