# Status: Real spaceflight pack

Owner thread: "Build real spaceflight history pack". Covers: RS (design: `design/SSE4-real-spaceflight-feasibility.md`, accepted 23 Nov 2026).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

The pack is "Sol today" on Explorers, not built into SSE4; the onboarding tour (chart room thread) uses it to teach loading a pack from the store. Parts use the one power model of `design/SSE4-atomic-rockets-plan.md` section 11.3 and the instrument shape of its section 12.

| Package | State | Log |
|---|---|---|
| RS1 catalogue import (GCAT, Wikidata, NSSDCA) to pack data; licence wording confirmed; counts firmed up | part 1 done 23 Nov: `scripts/spaceflight/import_gcat.py` writes `store-packs/sol-today/catalogue.json` (7,223 orbital launches, 472 payloads that left Earth orbit, 469 landings and impacts, 770 human spaceflight vehicle records; 816 KB, 139 KB gzipped) plus an authored `stations.json` (16 station cores). GCAT is CC BY 4.0, credited. part 2 done 23 Nov: Wikidata cross-check (336 of 472 matched, ids added; 1 date and 67 mass disagreements listed in packages/RS1-wikidata.md). NSSDCA read per craft in RS2 | packages/RS1.md |
| RS2 parts catalogue, additive on SSE3's construct shape; the twelve craft's parts with cited figures and the future-tech test figures | part 1 done 24 Nov: `scripts/spaceflight/build_parts.py` writes `store-packs/sol-today/parts.json` from GCAT (5 launch vehicles, 13 stages, 17 engines in the RK 11.3 field names, exhaust speed from the sheet's g0) with every missing figure listed under `to_read`. Part 2 first batch done 24 Nov: `craft.json`, 7 craft from NSSDCA with quoted figures (power sources, Dawn's NSTAR at 55.5% jet efficiency, LM, CM shield, JWST). Second batch done 24 Nov: Cassini, SNAP-10A, ISS radiators, STS-1 burns, Falcon 9, V-2 (13 craft). Still to read: Shuttle hardware, ISS power and life support, entry heating, engine masses (list in the log) | packages/RS2.md |
| RS3 flight logs for the twelve; Horizons tracks as the yardstick; big-burns fudge measured | part 1: test logs for Voyager 2 and Apollo 11 in the travel thread's shape (`tests/fixtures/flightlog/`); Voyager's flybys read from Horizons | |
| RS4 flight log on every construct; split and join at dated events | travel thread ("Build galaxy charts and travel screens") | |
| RS5 ascent from the ground | travel thread | |
| RS6 shape hints and finishes for real hardware | renderer thread | |
| RS7 missions on the ruler; flown and SSE4-says lines; the tour step | chart room thread | |
| RS8 flight logs for all ~300 departed craft, stations and crewed flights | after RS3 | |
