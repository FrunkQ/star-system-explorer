---
name: sse4-real-spaceflight
description: Track RS (proposed 23 Nov 2026): real spaceflight pack, parts-built real craft, flight log on every construct; Alex's rulings and open questions
metadata:
  type: project
  modified: 2026-11-23T20:18:50.296Z
---
Thread "Build real spaceflight history pack". Feasibility note: /mnt/project-files/sse4/SSE4-real-spaceflight-feasibility.md (ACCEPTED 23 Nov 22:01, all 12 questions yes).

Alex's words, 23 Nov 20:14 to 20:17:
- Cover spaceflight until now: all stations, all craft that left Earth; our own simple 3D models and textures; test our systems from Shuttle entry to reusable craft; transfers to other planets; maybe a store pack with every mission on the timeline.
- Every real craft is made from its real engines, fuels and other parts, sensors included (no lumps).
- SSE3 constructs are the starting and import point; everything additive (SSE3 shape: construct node with engines [{engine_id, quantity}], fuel_tanks, sensors, systems.power_plants, engineDefinitions, fuelDefinitions, sensorDefinitions, flight_log, scheduled_journeys, attachedTo).
- SSE3 flight logs let you rewind and the ship is where it was; SSE4 needs that; it makes the historical pack work.
- RULING: simplify and approximate: big burns only, stable orbits between, no station-keeping or small corrections (log as a fudge with its size).
- It is SSE3's construct model extended and re-engineered for SSE4, done now because 4.0 and later needs (future tech, 4.1, 4.2) are clear; design for later, build only 4.0.

Findings 23 Nov: SSE4 main has plan_voyage legs with depart_s but no stored dated flight log on constructs; the V3 importer keeps flight_log/scheduled_journeys raw and unread. Flight log must be built in travel code (sse-nav) with kernel thread. Flybys must be log events too, not only burns.

Proposed first slice (also the future-tech improvement 6 test set, FT thread told): Saturn V/Apollo 11, Shuttle STS-1, Falcon 9 landing, Voyager 2, Cassini-Huygens, New Horizons, Dawn, Curiosity, JWST, ISS, with Juno for New Horizons plus V-2 and SNAP-10A (FT thread asked, 12 in all). Pack is "Sol today" on Explorers, NOT built in; the onboarding tour uses it to teach loading a pack from the store (Alex 22:01).

Related: [[sse4-future-tech]], [[sse4-ship-design]], [[sse4-example-packs]], [[sse4-atomic-rockets]]

Build state 24 Nov 04:05 UTC: merged PRs 331, 340, 363, 364, 367. craft.json has 13 craft (second batch: Cassini, SNAP-10A, ISS radiators, STS-1 OMS burns, Falcon 9, V-2). Still to read: Shuttle hardware, ISS power and life support, entry heating, engine masses. Cloud proxy allows only planet4589, ssd.jpl, nssdc, wikidata; other NASA pages via the web-fetch agent. PR 373 rebuilt parts.json after the sheet gained pban-ap-al and htpb-ap-al; sheet_missing is empty. Next: RS3 flight logs.

RS3 part 1 (24 Nov 14:40 UTC, PR 395): scripts/spaceflight/build_logs.py writes 30 logs to store-packs/sol-today/logs/ (Apollo 11 with every stage, Falcon 9 flight 20 with B1019 landing, STS-1, Cassini-Huygens from Horizons, Voyager 2); test crates/sse-nav/tests/sol_today_logs.rs. Big-burns fudge measured on Cassini cruise: at most 1.4% of distance. Horizons has Apollo 11's S-IVB (-399110). Next: Dawn, Juno, Curiosity, JWST, ISS, V-2, SNAP-10A logs; read Apollo 11 and STS-1 mission reports.

24 Nov 17:10 UTC: PR 405 merged (RS3 part 2, 45 showcase logs). PR 415 (RS8 part 1: 462 GCAT logs in logs/departed/) in CI. Local commit 6050ffe (horizons.py helper, rebuild steps, rulings in handover/status/spaceflight.md) to push as its own PR after 415. Rebuild recipe lives in that status file.
24 Nov 20:45 UTC: PRs 415, 429, 446 merged (462 departed + 770 Earth-orbit logs; handover recipe in handover/status/spaceflight.md). PR 482 open (wording fix: GCAT has no later phases for crewed flights; next package RS8 part 3 takes undocking/landing from mission timelines).
