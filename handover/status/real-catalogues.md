# Status: real catalogues, the real Milky Way and the Detail control (track RW)

Owner thread: "Design the real star catalogue import" (Fable, design only, 26 Nov 2026) for the note. No builder thread yet. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-real-catalogues-design.md`, **ACCEPTED by Alex 26 Nov 2026 18:08 UTC** ("yes to all otherwise - you have the drift"). Read its sections 1, 3, 6A and 12A first; 12A (his rulings) wins where it differs. Readable copy he comments on: https://claude.ai/artifact/UrxXiprDnjGmhqVLrrCgYW.

What it is, in one line: a catalogue (real or invented; stars, galaxies, planets, small bodies) is a list of measured entries stored biggest first, each of which claims the seeded slot that fits it and that slot's share of the budget; the real Milky Way is a universe pack made of such claims, starting at Earth today; every population is a ranked list, and one Detail control on every view cuts it by size without reading what is below the cut; the fully catalogued Solar System is the yardstick for generated systems.

It builds on, and does not replace: the night sky note's section 8 and stage Y8 (catalogues, pins and thinning), galaxy design 10, the grand zoom tie-up (GZ12, GZ13), the conserved chain (claims are pins; the tracked set; CC6 and CC8).

Nothing is built. RW is a code only, never shown in the product.

**Alex's steers, 26 Nov 17:53 UTC (rulings, note 12A):** reuse the existing backfill machinery for a probable system round a real star, on by default (V3's `infillSystem`; in SSE4, formation `from_star` with F9 pins); every sum this track adds is a block chain in the physics, not importer code. Gaia data may be added to any pack and any star system (17:58 UTC); the old "Gaia only in an optional pack" line is withdrawn, and a pack holding downloaded catalogue data may be used and given away but never sold (18:00 UTC; the shop refuses a price and says why). Each provider's actual credit text ships in the pack as plain readable words and is shown in full at the first import (18:01 UTC). Answers so far (18:03 UTC): question 1, two doors on the start sheet; question 3, about 9,000 stars ship in the app with about 100 shown by default, and everything bulkier is a free detail expansion pack on the Explorers store, like the Atomic Rockets pack.

**Order.** The system half (RW3 to RW7, RW9, RW10, RW16, RW17) is inside a system and can start now. The star half (RW11 to RW14) waits for GZ13 and CC6.

**Known before any build:** the cloud build machine cannot reach any catalogue host (JPL, the Minor Planet Center, CDS, the NASA Exoplanet Archive, ESA, Codeberg; tried 26 Nov), so catalogue files are built on Alex's PC. Every figure marked unverified in the note is from memory until RW1.

| Package | State | Owner (proposed) |
|---|---|---|
| RW0 the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| RW1 read every catalogue's terms, size and count at source; replace the unverified figures; a terms table in `docs/citations.md` | accepted, not started | citations |
| RW2 measure, merge nothing: real size lists and counts per stop, moving points a phone and AC-ProArt can draw, the cost of a pin and of a table row, a 50 pc tracked set; gives the set points and ceilings | accepted, not started | quality tiers, small bodies |
| RW3 the catalogue format `sse.catalogue/1` and its reader crate (size order, range index, frame, anchor, date, completeness, grades, terms; never invent) | **done, 26 Nov, PR 643**: crate `sse-catalogue`, op `read_catalogue`, contract area `catalogues`, CLI `sse catalogue pack\|show`. Format spec `design/SSE4-catalogue-format.md`; log `packages/RW3-catalogue-format.md` | thread "Build the catalogue format and reader" |
| RW3b the estimating flows as blocks in a pack (note 6.3): star mass, small-body size, albedo and mass, completeness, thinning, the round test | accepted, needs RW3 | data sheet, kernel |
| RW4 populations: the table read (a block of ranks as columns) for seeded and catalogued lists, joined with thinning; own golden area | accepted, needs RW3 | small bodies, kernel |
| RW5 the Detail control on `ViewControls` (stops as data, ceiling from the probe, per-device memory, the always-shown rule), system level first | accepted, needs RW2, RW4 | chart room, quality tiers |
| RW6 drawing a crowd: columns to the GPU, orbits in the shader, a press resolves one member; phone and text paths; retires fudge SB-sample | accepted, needs RW4, RW5 | small bodies, renderer |
| RW7 the Solar System catalogue: every moon, minor planet and comet in size order; base-sol gains full orbits at a date, belt masses by group, the Expected stop and notable flags | **in progress** (thread "Add every Solar System body"). RW7a and RW7b done, 27 Nov, PR 663: Saturn's 7 small moons and J2 in base-sol; `scripts/build-sol-catalogues.py` (fetch on AC-ProArt, build from the cache), first real run 1,590,628 small-body rows and 46 moons, packed 333 MB; measured sizes for the large TNOs from their papers (`scripts/sol-measured-sizes.json`). Logs `packages/RW7a-saturn-small-moons.md`, `packages/RW7b-sol-catalogue-build.md`. Left: the irregular moons' sizes, more TNO sizes, base-sol moon orbits at a date, belt masses by group | thread "Add every Solar System body" |
| RW8 detail expansion packs on the Explorers store (note 5.3A): pack kind, shelf, byte-range reads (test the store first, a bucket only if it fails), live-query cache, the offline edge, "get this pack" marks on the Detail control | accepted, needs Alex's answer to question 8 | kernel hand-offs or Explorers |
| RW9 filters as chips over population columns (group, makeup, orbit, hazard, known, visited, interesting) | accepted, needs RW5 | navigation |
| RW10 the census: op `census`, the "Against the Solar System" card, the thousand-seed report and the gap list | accepted, needs RW4, RW7 | proven core |
| RW11 claims in the star field: a pin from outside, best-fit slot in the count tree, thinning by completeness, the block overdraft rule, three tests. Core golden | accepted, needs GZ13, CC6, RW3 | kernel (core lane) |
| RW12 the real sky pack: home wish, measured galaxy claims, arms, the Sun's slot, the date, named neighbours, shipped bright and named stars; the start sheet's Solar System door opens it | accepted, needs GZ12, RW11, the Local Arm fix, RW1 | galaxy charts and histories, kernel |
| RW13 star sources: CNS5, doubles, known planets, names, the live route, reusing V3's importer through the converter module; known planets as formation pins (F9), backfill on by default and formed on a visit (note 6.2) | accepted, needs RW8, RW11 | converter |
| RW14 the import action: centre, radius, preview, apply; any anchor in any galaxy; a Traveller sector as a catalogue | accepted, needs RW11, RW13 | galaxy charts and histories, Traveller |
| RW15 Detail on the other levels (Local stars, Galaxies nearby, the web, a world's moons and rings); the night sky's star budgets move under it | accepted, needs RW5, GZ7 | each view's owner |
| RW16 small bodies as their own kind and the designer (note 6A.1): consistent lumpy shapes in every view, measured shapes on catalogued bodies, a Build desk designer over the `make_small_body` recipes | accepted; the designer can start now | small bodies, editing |
| RW17 measured motion (note 6A.2): a motion model on a catalogue entry with its span; planets and the Moon from published series as cited pack tables; eclipse checks (2017, 2024, 2026, one lunar, one ancient) | accepted, needs RW1, RW7 | kernel, data sheet |

## Next
- The coordinator opens RW1, RW2 and RW3, then RW7 and RW17; the designer half of RW16 can start at once.
