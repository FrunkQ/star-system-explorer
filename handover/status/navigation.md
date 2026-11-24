# Status: Finding your way (navigation)

Owner thread: "Build object selector and menus". Covers: NV, the second V3 audit's items 37 (selector part 2, after the chart room's first cut in PR 399), 38, 41 and 42.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| NV1 a menu on every object (item 38): one action list, desk-aware, greyed with reasons, phone sheet | done | packages/NV1.md |
| NV2 selector part 2 (item 37): find by #tag, search across the campaign, menu on its rows (part 1); campaign search wired, picking goes there (part 2); every "which one?" opens it (part 3, next) | parts 1 and 2 done | packages/NV2.md |
| NV3 Build tools and More read the one action list; menu on ships, stations and System 3D | queued | |
| NV4 find by tag and light it up (item 41) on the chart; the map waits on the galaxy charts thread | done | packages/NV4.md |
| NV5 description and notes for the campaign and each system (item 42) | done | packages/NV5.md |

## Where it stands (24 Nov 2026)
- Open: PR 422 (NV5, NV2 part 2, NV4). If it is still open when you pick this up, rerun its checks and merge when green; nothing in it changes goldens.
- **Next package: NV3.** (a) Build's tool strip and the More menu offer the same actions as the object menu by reading `OBJECT_ACTIONS` (`ui-contract/src/actions.ts`) instead of their own lists. (b) Ships, stations and the System 3D view dispatch `sse:object-menu` `{address, x, y}` (see `shell/src/object-menu.ts`) so they get the menu too. (c) Move the object-menu block (`watchObjectMenu`, `objMenuItems`, `runAction`) out of `shell/src/App.svelte` into `shell/src/nav/`. Checks: `tests/ui/object-menu.e2e.mjs` grows a ship and a 3D case.
- After NV3: NV2 part 3 (every "which one?" opens the Picker; touches the voyages and Build threads' dialogs, so agree it with them), then lighting up tagged systems on the galaxy map (needs the map's systems to carry `data-address`).
- Alex's rulings for this track (24 Nov, second V3 audit, "yes to all"): one selector on the place line that replaces Below and doubles as Find; one menu on every object from one action list, desk-aware; find by tag and light it up; a description and GM notes for the campaign and each system. A press on a picker row goes there in one step (no separate ›).
