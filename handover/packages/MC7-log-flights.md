# MC7: the ship's log reads its flights

Track MC. Owner thread "Build ship manifest and crew roster". Alex, 24 Nov 17:54: "The log record all flights as well as cargo transfers and refits?" Transfers were done in MC5 and refits in MC6. This package adds flights, read from the flight log the travel track writes (V8, PR 452: `record.flight`, ops `log_flight` and `flight_log`, `session.flightLog`).

## What it does

- `flightLines(log, yr, place)` (ui-contract constructs.ts) turns a construct's flight log events into Log lines (`ShipLogEntry.what` "flight"):
  - A voyage is one line on arrival: "Flew Earth to Mars (108 days)", or hours under two days. A departure with no arrival yet reads "Left Mars, under way".
  - Launches, burns ("Burn of 3.6 km/s, 12.5 t of propellant"), flybys, entries, landings, splits, joins, refuels, loads and unloads each get a line on their own date.
  - An entry marked planned or estimated, rather than recorded, says so beside its reason.
- `LOG_RANK` is the one ordering of same-moment lines, shared by `shipLog` and the card.
- The Ships and stations card's Log reads `session.flightLog({construct})` on the GM's machine (players' seats have no flight log read) and merges the flight lines with the record, aboard and refit lines.
- The voyage's fuel draw-down (travel: `setKept` on `FUEL_FIELDS`, reason "voyage to <place>") already showed as kept-value lines.
- `tests/ui/manifest.e2e.mjs` joins CI's ui job (one test line; the coordinator relayed Alex's 24 Nov 20:42 "test locally, push in bigger chunks", which allows adding test lines).

## Checks

- `tests/ui/manifest.test.ts`: a voyage, a burn with its estimate, a flyby, a refuel and an unfinished departure, out of order in, in order out.
- `tests/ui/party.e2e.mjs`: after Beowulf flies Earth to Mars, its Log on Ships and stations reads "Flew Earth to Mars (… days) · planned" and the "voyage to Mars" draw-down. Screens: project files `sse4/handover-screens/MC7-screens/party-ship-log.png`.
- `tests/ui/manifest.e2e.mjs`: passed 3 runs of 3 locally before going into CI.
- No golden change.

## Known

- The card's "at Earth" still names where the construct was made. Where the ship is now is the travel track's (`ws.party.at`, later the flight log's `where_at`).
- Fuel lines are dated at departure (when the voyage is flown at the table) and the flight line at arrival.
