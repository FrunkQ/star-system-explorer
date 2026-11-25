# Status: Real spaceflight pack

Owner thread: "Build real spaceflight history pack". Covers: RS (design: `design/SSE4-real-spaceflight-feasibility.md`, accepted 23 Nov 2026).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

The pack is "Sol today" on Explorers, not built into SSE4; the onboarding tour (chart room thread) uses it to teach loading a pack from the store. Parts use the one power model of `design/SSE4-atomic-rockets-plan.md` section 11.3 and the instrument shape of its section 12.

| Package | State | Log |
|---|---|---|
| RS1 catalogue import (GCAT, Wikidata, NSSDCA) to pack data; licence wording confirmed; counts firmed up | part 1 done 23 Nov: `scripts/spaceflight/import_gcat.py` writes `store-packs/sol-today/catalogue.json` (7,223 orbital launches, 472 payloads that left Earth orbit, 469 landings and impacts, 770 human spaceflight vehicle records; 816 KB, 139 KB gzipped) plus an authored `stations.json` (16 station cores). GCAT is CC BY 4.0, credited. part 2 done 23 Nov: Wikidata cross-check (336 of 472 matched, ids added; 1 date and 67 mass disagreements listed in packages/RS1-wikidata.md). NSSDCA read per craft in RS2 | packages/RS1.md |
| RS2 parts catalogue, additive on SSE3's construct shape; the twelve craft's parts with cited figures and the future-tech test figures | part 1 done 24 Nov: `scripts/spaceflight/build_parts.py` writes `store-packs/sol-today/parts.json` from GCAT (5 launch vehicles, 13 stages, 17 engines in the RK 11.3 field names, exhaust speed from the sheet's g0) with every missing figure listed under `to_read`. Part 2 first batch done 24 Nov: `craft.json`, 7 craft from NSSDCA with quoted figures (power sources, Dawn's NSTAR at 55.5% jet efficiency, LM, CM shield, JWST). Second batch done 24 Nov: Cassini, SNAP-10A, ISS radiators, STS-1 burns, Falcon 9, V-2 (13 craft). Still to read: Shuttle hardware, ISS power and life support, entry heating, engine masses (list in the log) | packages/RS2.md |
| RS3 flight logs for the twelve; Horizons tracks as the yardstick; big-burns fudge measured | part 1 done 24 Nov: `scripts/spaceflight/build_logs.py` writes 30 logs to `store-packs/sol-today/logs/` (Saturn V/Apollo 11 with every stage, Eagle and its descent stage; Falcon 9 flight 20 with booster B1019 landing and 11 satellites; STS-1 with SRBs and tank; Cassini-Huygens from Horizons; Voyager 2), checked by `crates/sse-nav/tests/sol_today_logs.rs`. Fudge measured on Cassini's cruise: at most 1.4% of distance. Part 2 done 24 Nov: Juno, Dawn, Curiosity (with its four cast-off parts), JWST, ISS and SNAP-10A from Horizons, the V-2 from GCAT (45 logs); low-orbit drift measured (720 km a day after a sample). Next: read the Apollo 11 and STS-1 reports for flown figures | packages/RS3.md |
| RS4 flight log on every construct; split and join at dated events | travel thread ("Build galaxy charts and travel screens") | |
| RS5 ascent from the ground | travel thread | |
| RS6 shape hints and finishes for real hardware | renderer thread | |
| RS7 missions on the ruler; flown and SSE4-says lines; the tour step | chart room thread | |
| RS8 flight logs for all ~300 departed craft, stations and crewed flights | part 1 done 24 Nov: `scripts/spaceflight/build_departed_logs.py` writes 462 logs to `store-packs/sol-today/logs/departed/` from GCAT's phase tables, one for every departed payload besides the showcase ones (PR 415, merged; 108 end on the ground at landercat's sites, 133 gone, 221 still flying or docked; 118 heliocentric legs solved to meet their planet). Measured against Horizons in cruise: 0.4 to 5% of distance from the Sun, MESSENGER 23%. Part 2 done 24 Nov: 770 Earth-orbit logs in `logs/earth-orbit/` (station cores and crewed vehicles, GCAT satcat first phase). Part 3 done 24 Nov: 255 docked flights land on Wikidata's landing day (programme landing zones), Earth inclinations in the showcase logs turned to the ecliptic. Part 4 done 24 Nov: 58 releases as splits on the carriers' logs (GCAT REL phases). Part 5 done 24 Nov: GCAT's end of mission (psatcat TDate) lands or burns up what docks (443 landed, 296 gone, 10 still docked, all modules or flights docked today); programme recovery areas for capsules with no site, 4 stand-ins left. Next package: RS8 part 6, each flight's own landing or splashdown point and the undocking hour from press kits, then moons and small bodies in the NAV fixture (see the package log) | packages/RS8.md |
| RS9 everyone who has been to space, and where each was on any date (Alex, 25 Nov 01:00: "a crew roster of every who was in space on that day") | part 1 done 25 Nov: `scripts/spaceflight/build_crews.py` writes `store-packs/sol-today/crews.json` from McDowell's astro and segments lists: 792 travellers (Ham and Enos the chimpanzees included), 3,314 dated legs, 3,114 aboard a craft or station with a log here (5 fall outside their craft's log dates); 19 who died in flight; a step count of people in space (peak 20); peak heights for 133 short-flight legs (74 below the 100 km edge) from McDowell's suborbital table. Part 2 done 25 Nov: `in-memory.json`, 17 losses (6 on flights, 8 preparing to fly, 3 in ground work), each with two or more sources (McDowell, Wikidata, the Wikipedia list), deaths on the people, Challenger's legs marked as never reaching space and left out of the count, the pack calendar, clean names. Next package: RS9 part 3, the design thread's asks (track SH, `handover/design/SSE4-spaceflight-history-design.md`): memorials.json (real memorials only: objects left, named features, named small bodies with JPL orbits, remains carried), the four ground losses waiting for a second source, Wikidata ids on everyone, animals, and the legs as `aboard.on` changes for the manifest (with the manifest thread) | packages/RS9.md |

## How to rebuild the pack (no chat history needed)

1. GCAT tables: download from https://planet4589.org/space/gcat/tsv/ into one folder outside the repo, flat:
   launch/launch.tsv, cat/deepcat.tsv, cat/hcocat.tsv, cat/lprcat.tsv, cat/landercat.tsv, cat/psatcat.tsv, and
   for the parts the vehicle, stage, engine and site tables build_parts.py names in its header.
2. `python3 -I scripts/spaceflight/import_gcat.py <folder> store-packs/sol-today` (catalogue), then
   `python3 -I scripts/spaceflight/build_parts.py <folder> store-packs/sol-today` (parts).
3. `python3 -I scripts/spaceflight/build_logs.py` (the showcase logs, from the horizons_*.json files here and the
   sources it names), then `python3 -I scripts/spaceflight/build_departed_logs.py <folder>` (GCAT logs; run it
   after build_logs.py, which it reads to avoid clashing ids).
4. New Horizons figures: `scripts/spaceflight/horizons.py` (elements, vectors, lookup; burn delta-v method in its
   header). `craft.json` is authored by hand from the sources in docs/citations/spaceflight.md.
5. Crews: download https://planet4589.org/space/astro/tsv/astro.tsv and segments.tsv into another folder, then
   `python3 -I scripts/spaceflight/build_crews.py <that folder>`, with https://planet4589.org/space/astro/web/suborb.html saved there as suborb.html (after the logs, which it resolves craft against).
6. Check: `cargo test -p sse-nav --test sol_today_logs`, then `scripts/check.sh`.

## Alex's rulings that shape this track (23 to 24 Nov 2026)

- The design (`design/SSE4-real-spaceflight-feasibility.md`) was accepted in full ("yes to all"), question 10 as
  amended: the pack is on Explorers, not built in, and the onboarding tour uses it to teach loading a pack.
- "And turns this into a historical records tool!": the logs are the record of what flew, so every figure is cited,
  every estimate says why, and the flown line and the SSE4-says line can be compared (RS7).
- Big burns and flybys only, stable orbits between (fudge register, Real spaceflight).
- Every craft is built from its real parts, sensors included; SSE3 constructs are the import point, so the shapes
  are additive; power is a general input an engine draws (Atomic Rockets plan 11.3).
