# PB1: the board underneath

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design: `handover/design/SSE4-playbook-and-switchboard-design.md` sections 4 to 6 and 11 (data shapes).

## What
- `packages/ui-contract/src/board.ts`: a `Board` is plain data. `lines.everyone` is the floor; `over` is Everyone sent as a layer (Lift takes it off). Each role (`role:<seat>`) and screen (`screen:<id>`) may have its own line with `before` (Back goes to it) and `held` (a `HoldCard`: blank, plain, joke or own words). `all` is Hold all, which remembers each line's hold and puts them back on release. `redirects` send a role's lines elsewhere; `signal` is per destination for the later Looks dial.
- `resolve(board, role, screen?)` gives what one screen sees. Holds win in order all, screen, role, everyone; then layer, screen line, role line, floor.
- `boardWire` turns the board into `Shown` per role and per screen. A hold goes out as `{off: true, card}`, so older players show a blank screen.
- `BoardHistory`: undo and redo of whole boards.
- Wire (p2p.ts): hello carries `screen {id, name, role}`; host state carries `screens`; the GM host reports new screens (`onScreen`). Sealed seats: `SealedHost.setShown` builds an empty scene while held, and replicas read `held`.
- Shell share (`share.svelte.ts`) keeps the board and history and pushes through `boardWire`. `show()` keeps its old meaning (reset to the floor).
- Players page: a screen keeps its id per tab (`sse4.screen.<room>`), takes its name from `?name=`, and shows a held card as "Stand by" with the card's words. The ask bar hides while held so the place name doesn't leak.

## Checks
- `packages/ui-contract/test/board.test.ts` (13), p2p.test.ts (screen hello, per-screen shown), seats.test.ts (sealed hold and release).
