# Status: Finding your way (navigation)

Owner thread: "Build object selector and menus". Covers: NV, the second V3 audit's items 37 (selector part 2, after the chart room's first cut in PR 399), 38, 41 and 42.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| NV1 a menu on every object (item 38): one action list, desk-aware, greyed with reasons, phone sheet | done | packages/NV1.md |
| NV2 selector part 2 (item 37): find by #tag, search across the campaign, menu on its rows (part 1); campaign search wired, picking goes there (part 2); every "which one?" opens it (part 3, next) | parts 1 and 2 done | packages/NV2.md |
| NV3 the menu everywhere: System 3D pins and canvas (part 1); Build tools and More read the one action list, ships and stations (part 2, next) | part 1 done | packages/NV3.md |
| NV4 find by tag and light it up (item 41) on the chart; the map waits on the galaxy charts thread | done | packages/NV4.md |
| NV5 description and notes for the campaign and each system (item 42) | done | packages/NV5.md |

## Where it stands (24 Nov 2026)
- Open PRs: none. PR 422 (NV5, NV2 part 2, NV4, NV3 part 1) merged 24 Nov 18:24 UTC.
- **Next package: NV3 part 2** (part 1 is done: the menu in System 3D; More and Build's tool strip read their words and keys from `OBJECT_ACTIONS`). (b) Ships and stations dispatch `sse:object-menu` `{address, x, y}` (see `shell/src/object-menu.ts`) so they get the menu too. (c) Move the object-menu block (`watchObjectMenu`, `objMenuItems`, `runAction`) out of `shell/src/App.svelte` into `shell/src/nav/`. Checks: `tests/ui/object-menu.e2e.mjs` grows a ship and a 3D case.
- System rows in the picker: the galaxy charts thread ("Build galaxy charts and histories") adds `ui-contract/src/system-summary.ts` (`systemSummary`, `summaryWords`; PR 431) and then `your-systems.ts` `readCampaignSystems(session)`. When they land, give the `sys:` rows in `nav/PlaceLine.svelte` `caption = summaryWords(systemSummary(subjects))` and switch the `seen` cache to `readCampaignSystems` for systems not yet opened (keep `seen` for the campaign's own system, which reads back with no bodies). Agreed with that thread: they do not edit `nav/`.
- After NV3: NV2 part 3 (every "which one?" opens the Picker; touches the voyages and Build threads' dialogs, so agree it with them), then lighting up tagged systems on the galaxy map (needs the map's systems to carry `data-address`).
- Alex's rulings for this track (24 Nov, second V3 audit, "yes to all"): one selector on the place line that replaces Below and doubles as Find; one menu on every object from one action list, desk-aware; find by tag and light it up; a description and GM notes for the campaign and each system. A press on a picker row goes there in one step (no separate ›).

## AT15a screen delivery (from 24 Nov 20:19)
| Package | State | Log |
|---|---|---|
| AT15a part 1: deliveries data, player (three scales), looks carry them, players' screen powers on, arrives and powers off | done (PR below) | packages/AT15a.md |
| AT15a part 2: Turn on the Atlas page (atlas thread wires it), Leave, drag page turn, look card row, Orac | next | packages/AT15a.md |
