# PB3b: who steers a screen

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 5.3 (Alex: "a bit suck it and see"; this is the first version to try).

## What
- `ui-contract/src/board.ts`: `Board.steer` (Read only or Theirs by destination; absent is Theirs), `steerOf(b, role, screen?)` (screen, else role, else Everyone), `setSteer` (a role's heading clears its screens' own settings; a screen keeps its own only when it differs; making a driven destination Theirs lets go), `Board.driving`, `drive(b, dest | null)` (driving makes it Read only), `driveTo(b, live)` (the driven line, or Everyone's top layer, takes the live view). `boardWire` adds `steer: "read"` to read-only destinations.
- `seat.ts`: `Shown.steer?: "read"`. Players' page: read-only main is `inert`, a small "The game master is steering this screen" mark, no ask bar; sound stays theirs.
- Share: `follow(live)` moves the driven destination without a history step. App: while driving (and not on the Switchboard), every change of `here()` follows; `here()` now carries `focus` (the GM's selection), so players' screens select it.
- Switchboard: a Theirs / Read only switch on Everyone, each role heading and each screen; Drive on a read-only one (opens the chart), Let go; the driven line in accent; "You are driving <name>" in the header and on the on-air chip.

## Checks
- board.test.ts: two new tests (steer settings and the wire; driving and letting go). `ui:check`, `ui:test` (547).
- New `tests/ui/steer.e2e.mjs` (in `ui:e2e`): Read only makes the screen inert; Drive opens the chart and the screen follows Vault then Tarn; Let go leaves it; Theirs gives it back; a role heading sets its screens. switchboard, table, landing-table, players, playbook, bookmarks, party, p2p and known pass.
- Screenshots: project files `sse4/handover-screens/PB3b-screens/`.

## Later
- "Showing …" words on the Switchboard name the place, not the selected body.
- Theirs keeps the player inside what was sent, as the players' page already has no place line; a layout's own controls come with PB13.
- A preset saying which state it wants (read only for a landing) is not built.
