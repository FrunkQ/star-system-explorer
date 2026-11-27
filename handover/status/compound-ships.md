# Status: compound ships that separate and rejoin (track CM)

Design thread: "Design compound ships that separate and rejoin" (Fable). Design: `design/SSE4-compound-ships-design.md`, **ACCEPTED by Alex 27 Nov 2026 00:43 UTC ("Otherwise yes to all")**, with his rulings in its section 14A winning (question 1 amended: ships, stations and land structures are all constructs; only one with a drive moves itself; the builder shows blueprint shadows behind). Part A of the design is the audit of the Sol today spaceflight pack.

Only the owner thread edits this file. Packages are in design section 13; owners as its question 12. Build threads update their rows in the same PR as the package. CM1, CM2 and CM4 rows were added after the fact by the design thread, from their package logs, at the coordinator's request.

| Package | State | Log |
|---|---|---|
| CM0 make Sol today installable | done 27 Nov, PR 683: Sol today installs through the Open door like any pack | packages/CM0.md |
| CM1 the mount: `mount_construct` and `release_construct`, five places, bonds, lines, ports and bays; `docked.to` read as a latch on the hull | done 27 Nov, PR 682 | packages/CM1-mounts.md |
| CM2 the flight log joined up: `mounted` pieces, position from the compound's root, capture checked for reach and speed with a rendezvous offered, release seeding the carried log (including from a course) with the push | done 27 Nov, PR 682 | packages/CM2-flight-log-join-up.md |
| CM3 budgets across mounts | not started | |
| CM4 the absolute conservation check (found and fixed a dock loop in RS4); propellant terms arrive with CM3 | done 27 Nov, PR 690 | packages/CM4-conservation-check.md |
| CM5 to CM18 | accepted, not started (design section 13) | |
