# Status: Run desk and players' side

Owner thread: "Build Run desk playbook and players' side" (from 24 Nov 2026, taken over from the chart room). Covers: the players' page (packages/shell/src/Players.svelte, players.ts, documents/players.*.screen.json), the table's part of the Run desk (packages/shell/src/run/: TableStatus, SeatsPanel, ArrivalSlip), and from design the Playbook and Switchboard. Shared seams changed only with a note to the chart room: ui-contract seat.ts `Shown`, p2p host state (`shown`, `party`), `ws.sendLanding`. Tests owned: table.e2e, landing-table.e2e, party.e2e, players.e2e.

Only the owner thread edits this file. Earlier rows for this area (What the table is shown, adopted systems reach players, the table watches a landing, players see the party's ship, seat rows first cut) stay in `status/chart-room.md`.

| Package | State | Log |
|---|---|---|
| RD1 reveal on arrival and a landing sent too soon (landing run-through gap 6; landing-table race): when the party's ship comes to a world the table does not fully know, the table's line offers "Beowulf is at Mars. Tell the table about it?"; one press sends the ordinary `set_known` (audience everyone, landmark known, at the campaign's now) with Undo (`undo_known`) and "Not now". Logic in ui-contract arrival.ts (`arrivalOffer`, `arrivalReveal`). A "Send this view" pressed on Land before the Land screen's first landing arrived now goes again with the landing (it went as the plain system and the players never saw the landing). | done | packages/RD1.md |
| RD2 Playbook and Switchboard (Alex 24 Nov 15:28 to 15:39: no Next; presets fired at roles; a Switchboard "nerve centre" that replaces the Seats dialog, Table sees and seat rows; OBS out early, OBS in GM-side only) | design in thread "Design playbook and switchboard" (Fable); build after it is accepted | memory sse4-run-desk-beats |

## Next
- RD2 once the design lands. Small: the Run desk card at Curious leads with four facts and drops Mass for planets with a day and a year (gap 10): depth.ts puts mass fifth by design; ask Alex with the Switchboard questions rather than change it alone.
