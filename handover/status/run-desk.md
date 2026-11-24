# Status: Run desk and players' side

Owner thread: "Build Run desk playbook and players' side" (from 24 Nov 2026, taken over from the chart room). Covers: the players' page (packages/shell/src/Players.svelte, players.ts, documents/players.*.screen.json), the table's part of the Run desk (packages/shell/src/run/: TableStatus, SeatsPanel, ArrivalSlip), and from design the Playbook and Switchboard. Shared seams changed only with a note to the chart room: ui-contract seat.ts `Shown`, p2p host state (`shown`, `party`), `ws.sendLanding`. Tests owned: table.e2e, landing-table.e2e, party.e2e, players.e2e.

Only the owner thread edits this file. Earlier rows for this area (What the table is shown, adopted systems reach players, the table watches a landing, players see the party's ship, seat rows first cut) stay in `status/chart-room.md`.

| Package | State | Log |
|---|---|---|
| RD1 reveal on arrival and a landing sent too soon (landing run-through gap 6; landing-table race): when the party's ship comes to a world the table does not fully know, the table's line offers "Beowulf is at Mars. Tell the table about it?"; one press sends the ordinary `set_known` (audience everyone, landmark known, at the campaign's now) with Undo (`undo_known`) and "Not now". Logic in ui-contract arrival.ts (`arrivalOffer`, `arrivalReveal`). A "Send this view" pressed on Land before the Land screen's first landing arrived now goes again with the landing (it went as the plain system and the players never saw the landing). | done | packages/RD1.md |
| PB1 the board underneath (design SSE4-playbook-and-switchboard-design.md, accepted 24 Nov): ui-contract board.ts holds who sees what as plain data (Everyone as a layer over the floor, a line per role and per screen, holds with a card, Hold all that remembers, redirects, signal levels) with undo and redo (`BoardHistory`); `boardWire` turns it into the old `Shown` per role and per screen, so older players still work. Players' screens say hello with a name and a role (`?name=`, id kept per tab); a held screen shows its card ("Stand by", plain words or a joke). Sealed seats get an empty scene while held. | done | packages/PB1.md |
| PB2 the Switchboard screen (rail entry under Table, every scale): the Live card and Send, roles and screens as a list with lines, press a line to hold and again to release, Hold all (H), Lift, Back, Undo and Redo (Ctrl+Z), drag or press-then-press a source onto a role or screen, move and swap screens by their grip. The status line's table half is now one on-air chip that opens it; Send this view stays. Seats dialog stays behind "More seats" until PB7. | done | packages/PB2.md |
| PB2b Bookmarks beside Undo (needs the chart room's top bar) | next | |
| PB3 the Playbook; PB3b who steers (Theirs, Read only); PB4 to PB14 per design section 15 | queued | |

## Next
- PB2b, then PB3 and on in design order. The on-air chip moves to the chart room's top bar when it has room. Small: the Run desk card at Curious leads with four facts and drops Mass for planets with a day and a year (gap 10): depth.ts puts mass fifth by design; ask Alex with the Switchboard questions rather than change it alone.
