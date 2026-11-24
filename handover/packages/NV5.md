# NV5: A description and notes for the campaign and each system (V3 gap 42)

Status: done 24 Nov 2026 (thread "Build object selector and menus"). Accepted by Alex on 24 Nov (second V3 audit, item 42).

## What it does
- **Notes** beside the place's name on the place line opens a card from it. At a system or a world, it is that system's; above the system (galaxy, galaxies nearby, cosmic web), it is the whole campaign's. On a phone the line keeps its room for the name, and Notes is a row of the picker instead.
- Two boxes, edited in place: **Description** (for everyone) and **GM notes** (for the GM alone, drawn with a dashed edge). A change is kept when the box loses focus, on Ctrl+Enter, on Escape or a press outside.
- **Kept in the campaign record**, as every other note is (kernel `add_note`, record design section 2), at the place's own address: the system's address, or the universe's root (`u:<id>`) for the campaign. A change supersedes the last note and clearing withdraws it, so the record keeps the history. Tags `description` (the same tag Describe uses on a body, gap 23) and `gm-notes`. Notes never change the universe's state hash, so this is open on every desk.

## Where
- `ui-contract/src/place-notes.ts`: `PLACE_NOTE`, `placeNoteOf`, `placeNoteEdit`, `campaignAddress`.
- `shell/src/nav/PlaceNotes.svelte` (the card); `shell/src/nav/PlaceLine.svelte` (the button, and the picker row on a phone). App passes `session` and `systemAddress` to PlaceLine (two lines).

## Checks
- `packages/ui-contract/test/place-notes.test.ts`: edits only on change; withdraw; on the real kernel, writing, changing and clearing a system's notes and the campaign's, with the state hash unchanged.
- `tests/ui/place-notes.e2e.mjs` (in `ui:e2e`): write and reopen at Kestrel, the campaign's own notes at the galaxy, Escape keeps, Night chart, phone row and card, no overflow, state hash unchanged.

Screenshots: project files `sse4/handover-screens/NV5-screens/`.

## Not yet
- Players seeing a system's description: notes reach seats through record disclosures, which the replicas do not carry yet (see G23-describe.md, Next).
- A body's own description and notes on its card (gap 23 part 2, Traveller track).
