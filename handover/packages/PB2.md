# PB2: the Switchboard screen

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 15, row PB2.

## What
- `packages/shell/src/run/Switchboard.svelte` (view module `switchboardView` in `run/screens.run.ts`, screen doc `documents/switchboard.gm.json`, rail group "Table" new in `RAIL_GROUPS`, entries at web, cluster, galaxy, system and world).
- Header: on-air chip or "Go on air", Undo and Redo, Hold all / Release all (key H; Ctrl+Z undoes the board while the Switchboard is open).
- Live card: what the GM has in front now, with Send (to Everyone) and a source dot: drag it onto a role or screen, or press it and then press the destination.
- Everyone row with Lift. A section per role: Back, With Everyone, Hold or Release and the hold card. Under each role its screens by name; each screen has a grip to move it to another role or swap with another screen, "Back with <role>", and Hold.
- Lines drawn from Live to every destination: solid when live, dashed when held, faint under a layer; press a line (or its lozenge) to hold or release.
- Phone width: one column, no lines.
- `run/table-port.ts`: the Workspace's `table` port (board, roles, players, link, act, undo, redo, start, links) so the Switchboard reads and acts without reaching into App.
- `run/TableStatus.svelte`: the status line's table half is now one chip "On air · N screens · k held · <what the table sees>" that opens the Switchboard. "Send this view", the players' link, the arrival slip, requests and "More seats" stay.

## Shared edits (told the chart room)
- `ui-contract/screen.ts` RAIL_GROUPS gains "table"; `view.ts` Workspace gains `table?`.
- `App.svelte`: spreads `RUN_SCREENS`, sets `ws.table`, TableStatus gets `onswitchboard`. `registry.ts` spreads `RUN_VIEWS`.

## Checks
- `npm run ui:check`, `npm run ui:test` (513 pass).
- New `tests/ui/switchboard.e2e.mjs` (in `ui:e2e` after landing-table): two screens join by name, Send, hold a line, Hold all and release, a layer and Lift, move a screen, undo, phone width. `table.e2e` now holds the table from the Switchboard; `landing-table.e2e` ends on Hold all.
- Screenshots: project files `sse4/handover-screens/PB2-screens/`.

## Later
- The chip belongs in the top bar (chart room); PB2b Bookmarks sits beside Undo there.
- Seats dialog goes when PB7's role card lands.
