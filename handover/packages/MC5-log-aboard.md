# MC5: the ship's log reads what is aboard; fuel as kept values

Track MC. Owner thread "Build ship manifest and crew roster". From the MC1 open list ("the ship's log should include aboard moves") and Alex's fuel ask (MC4).

## What it does

- `aboardLog(place, entries, yr, placeName)` (ui-contract manifest.ts): from everyone and everything that was ever aboard a place, the lines "X came aboard (from Y)", "X left for Y" (cargo with its amount), and "X is in cold sleep / missing / dead / awake" for state changes made while aboard. Read from the entries' own dated `aboard.on` and `aboard.state` histories, with writer and reason, so the log cannot drift from what happened. A state change made elsewhere is not this ship's.
- The Ships and stations card's Log (Constructs.svelte `loadLog`) reads the entries at the universe root, keeps those ever aboard this construct, and merges their lines with its own (`ShipLogEntry.what` gains "aboard").
- Fuel levels move from construct facts (MC4) to kept values in the base manifest pack: `fuel.propellant_t` and `fuel.jump_t` (number, unit t, writers gm, tier scanned). Each set, top-up (reason "topped up") or, later, voyage draw-down is a dated log line with its change: "Jump fuel: 19.838 (+14.838) · topped up". `MANIFEST_WORDS` names them in the log. MC4 had not reached anyone else yet, so nothing else read the old fields.

## Checks

- `tests/ui/manifest.test.ts`: the log from three entries (came, left, state while aboard, a state change elsewhere left out).
- `tests/ui/manifest.e2e.mjs` (by hand): Kittiwake's log has the arrivals, the cargo leaving for Brass Lark, Rhee's cold sleep and the jump fuel top-up.
- Pack change: two kept kinds added to `sse.base-manifest`; no golden change.
