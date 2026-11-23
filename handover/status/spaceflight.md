# Status: Real spaceflight pack

Owner thread: "Build real spaceflight history pack". Covers: RS (design: `design/SSE4-real-spaceflight-feasibility.md`, accepted 23 Nov 2026).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

The pack is "Sol today" on Explorers, not built into SSE4; the onboarding tour (chart room thread) uses it to teach loading a pack from the store. Parts use the one power model of `design/SSE4-atomic-rockets-plan.md` section 11.3 and the instrument shape of its section 12.

| Package | State | Log |
|---|---|---|
| RS1 catalogue import (GCAT, Wikidata, NSSDCA) to pack data; licence wording confirmed; counts firmed up | part 1 done 23 Nov: `scripts/spaceflight/import_gcat.py` writes `store-packs/sol-today/catalogue.json` (7,223 orbital launches, 472 payloads that left Earth orbit, 469 landings and impacts, 770 human spaceflight vehicle records; 816 KB, 139 KB gzipped) plus an authored `stations.json` (16 station cores). GCAT is CC BY 4.0, credited. Wikidata and NSSDCA cross-checks next | packages/RS1.md |
| RS2 parts catalogue, additive on SSE3's construct shape; the twelve craft's parts with cited figures and the future-tech test figures | waiting on RS1 | |
| RS3 flight logs for the twelve; Horizons tracks as the yardstick; big-burns fudge measured | part 1: test logs for Voyager 2 and Apollo 11 in the travel thread's shape (`tests/fixtures/flightlog/`); Voyager's flybys read from Horizons | |
| RS4 flight log on every construct; split and join at dated events | travel thread ("Build galaxy charts and travel screens") | |
| RS5 ascent from the ground | travel thread | |
| RS6 shape hints and finishes for real hardware | renderer thread | |
| RS7 missions on the ruler; flown and SSE4-says lines; the tour step | chart room thread | |
| RS8 flight logs for all ~300 departed craft, stations and crewed flights | after RS3 | |
