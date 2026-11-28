# Status: compound ships that separate and rejoin (track CM)

Design thread: "Design compound ships that separate and rejoin" (Fable). Design: `design/SSE4-compound-ships-design.md`, **ACCEPTED by Alex 27 Nov 2026 00:43 UTC ("Otherwise yes to all")**, with his rulings in its section 14A winning (question 1 amended: ships, stations and land structures are all constructs; only one with a drive moves itself; the builder shows blueprint shadows behind). Part A of the design is the audit of the Sol today spaceflight pack.

Only the owner thread edits this file. Packages are in design section 13; owners as its question 12. Build threads update their rows in the same PR as the package. CM1, CM2 and CM4 rows were added after the fact by the design thread, from their package logs, at the coordinator's request.

| Package | State | Log |
|---|---|---|
| CM0 make Sol today installable | done 27 Nov, PR 683: Sol today installs through the Open door like any pack | packages/CM0.md |
| CM1 the mount: `mount_construct` and `release_construct`, five places, bonds, lines, ports and bays; `docked.to` read as a latch on the hull | done 27 Nov, PR 682 | packages/CM1-mounts.md |
| CM2 the flight log joined up: `mounted` pieces, position from the compound's root, capture checked for reach and speed with a rendezvous offered, release seeding the carried log (including from a course) with the push | done 27 Nov, PR 690 | packages/CM2-flight-log-join-up.md |
| CM3 budgets across mounts | not started | |
| CM4 the absolute conservation check (found and fixed a dock loop in RS4); propellant terms arrive with CM3 | done 27 Nov, PR 690 | packages/CM4-conservation-check.md |
| CM5, CM6 | accepted, not started (design section 13) | |
| CM7 the designer places each construct by its mount; bays, ports, turned panels | done 27 Nov, PR 686 | packages/CM7-placement.md |
| CM8 to CM12 | accepted, not started; CM8 (renderer owner, the builder thread) waits on CM6 | |
| CM13 the palette: part 1 the one part format and 61 base parts, PR 697 (done 27 Nov); part 2 the tank wall, split surface, dish gain and link rate blocks, PR 704 (core lane). Eras of spaceflight, not decades (ruling 14A.8): `palette.eras` in base-parts | part 1 done; part 2 in the lane | packages/CM13-palette.md, packages/CM13-part2-blocks.md |
| CM14a first builder slice (ruling 14A.7): catalogue-first start, Build it and Build to a plan on one canvas, the Build desk pictures are the builder, Style and Remix gone, catalogue ships converted to parts on open | done, PR 720 | packages/CM14a-builder.md |
| CM14b the builder to the mock-up: full-screen parts list, 3D parts over fixed blueprint walls, glowing clip points, drag to clip, Parts from year slider, Selected / Build tree / Paint, shopping list; every ship picture drawn from parts, old designer gone | done, PR 746 | packages/CM14b-builder-mockup.md |
| CM14c parts 1 and 2: parts drawn as what they are (domed tanks, ribbed habitats, bells, lattice trusses, solar wings, dishes), the name on the hull, parts list by use; six example ships assembled from parts, "needs part masses" in place of invented numbers | done, PR 752 | packages/CM14c-builder-look.md |
| CM14c part 3: Cutaway; real part masses (tank walls from the wall block through flow sse.base.ship-tank-wall, cited engine masses, one cited mass row per kind), each part says where its mass came from | in review, builder thread | packages/CM14c-builder-look.md |
| CM14c later: masses for shields, legs, dishes, bays, trusses, the RL10A-3 and tech drives; Sensor views; "Build it for me" | next, builder thread | |
| CM14 the full builder (compounds, staging, sequence) | waits on CM8 | |
| CM15 to CM18 | accepted, not started (design section 13); CM15 reads eras, not decades | |
