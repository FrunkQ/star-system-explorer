---
name: sse4-party-ship
description: Landing run-through seam: the party's ship (ws.party) carried from Ships and stations to Voyages to Land; who owns which part
metadata:
  type: project
  modified: 2026-11-23T22:55:33.339Z
---
Landing run-through (Alex 23 Nov 22:16, target next week): the campaign's own ship carries voyage → arrival → landing.

- `ws.party = {ship, name, nav, at, voyage?}` (ui-contract party.ts), saved with the campaign; set by "Make this the party's ship" in Ships and stations (travel thread, PR 339).
- plan_descent takes `lander: {ship}` (ship landed on its own numbers).
- Renderer owns Land: reads ws.party, players replay descent from `LandingShow` (no telemetry stream).
- Chart room owns players' transport: adds a `party` host-state key beside `shown`; "Fly there" never moves players' screens, only the GM's "Send this view".
- Where the ship is lives on the workspace for now; it moves to the construct's flight log in the record later.
See [[sse4-coming-together]].
