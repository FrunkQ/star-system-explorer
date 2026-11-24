# RD1: reveal on arrival, and a landing sent too soon

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Landing run-through gap 6 (`/mnt/project-files/sse4/plans/landing-run-through-gaps.md`).

## What
- When the party's ship moves to a world (Voyages' "Fly … there" sets `ws.party.at`), the table's part of the status line shows a slip: "Beowulf is at Mars. Tell the table about it?" with **Tell the table** and **Not now**. Nothing is revealed by itself (guide, don't decide).
- Tell the table sends one `set_known` (address the world, audience `everyone`, landmark `known`, scope `this`, at the campaign's now, exact `at_s`). The slip then says "The table knows Mars now." with Undo (`undo_known` by the event id).
- No offer when the ship has not moved, when it is at a whole system, when the table already knows the world at its top rung, or on opening a campaign with the ship already there.
- Fix: "Send this view" pressed on the Land screen before the Land view's first `sendLanding` went out as the plain system, so players never got the landing (landing-table.e2e failed on main after 399/402 slowed the first publish). App's `here()` now notes a landing is awaited, and the first `sendLanding` sends the view again with it.

## Files
- `packages/ui-contract/src/arrival.ts` (`arrivalOffer`, `arrivalReveal`), test `packages/ui-contract/test/arrival.test.ts`.
- `packages/shell/src/run/ArrivalSlip.svelte`, mounted in `run/TableStatus.svelte` (new props `party`, `now_yr`).
- `packages/shell/src/App.svelte`: passes `party` and `now_yr`; `here()` and `ws.sendLanding` (landing awaited).

## Checks
- `npm run ui:check`, `npm run ui:test` (500 pass).
- `tests/ui/party.e2e.mjs` (the offer at Mars, Tell the table, Undo shown), `landing-table.e2e.mjs`, `table.e2e.mjs`, `players.e2e.mjs`.
- Screenshots: project files `sse4/handover-screens/RD1-screens/`.

## Later
- Sensor-driven reveals (RK14, kernel) will write the same grant; the slip can then name what the sensors found.
- The slip goes into the Switchboard (RD2) when that replaces the table line.
