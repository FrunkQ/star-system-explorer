# Status: life from the energy it can get (track LE)

Owner thread: "Design life's spread and population from energy" (design only, 26 Nov 2026) for the note. Builder thread for the kernel packages LE1 to LE6a: "Build life's energy baseline" (Opus, opened 27 Nov 2026), which keeps the rows below current. LE-U1 and LE-U2 belong to the screen threads.
Design: `design/SSE4-life-from-energy-design.md`, **ACCEPTED by Alex 26 Nov 2026 23:50 and 23:54 UTC** (rulings in the note's section 0). Builders may take the 4.0 baseline packages below. The 4.1 bucket (note section 9A: LE6b, LE7 to LE11) is not for building in 4.0.

In one line: free energy captured, set against the cost of staying alive, decides whether a habitat lives, how its energy is shared among kinds of life by diet, and how fast life spreads, evolves and recovers.

**Golden lane.** (Found by LE1 and LE2, 27 Nov: the core contract's two `life_view` cases are fixtures with no habitats, and the life pack is not in its hashed stack, so LE1 and LE2 moved no core line. LE5 and LE6a may still.) `life_view` is in the core contract (`tests/contract/requests.json`, lines in `tests/golden/contract.jsonl`), so every kernel package below moves core lines and goes in the core lane, one at a time, in the coordinator's order. Each also moves its own `sse-life` test goldens.

**Not needed:** the gradient model wiring (`gradients/1` in the kernel's runs). LE1 puts the keep-alive test in `budget::finish`, which both models share, and every later package builds on that. Habitat light (`light_w_m2`) is already on the habitat profiles, so LE2's "arriving" per habitat needs nothing new. Only the world-level comparison with absorbed sunlight waits on the climate sunlight figures HV asked for, and shows as not published until they land.

| Package | Needs | Golden | State |
|---|---|---|---|
| LE1 keep-alive test in the shared step, starved and laboratory maintenance in one shared cited block, "alive, barely", biomass capped by power over maintenance | nothing | life goldens only (hashes; no core line moved) | built, PR with LE2; log `packages/LE1-LE2.md` |
| LE2 energy ledger on `life_view` (arriving, offered, captured, keep-alive, surplus, brought in at zero) | LE1 | none (new field on chart worlds only; the contract's life cases are fixtures) | built, PR with LE1; world "absorbed sunlight" not published until PR 678 lands |
| LE3 where the energy goes: power and share by diet per guild, rough numbers by size class | LE1 | life goldens only (hashes) | built, PR with LE4; log `packages/LE3-LE4.md` |
| LE4 speed read-out: fill time per habitat, one speed line per world | LE1 | none (new field on chart worlds) | built, PR with LE3 |
| LE5 broad-brush spread in the deep-time history (link rates, colonisation waiting times, refuges) | LE4 | core (dated history moves) | not started |
| LE6a recovery clocked in Earth-Myr of opportunity | LE4 | core (dated history moves) | not started |
| LE-U1 Energy flow life branch | LE2, the Energy flow view (PR 670, HV) | none | built (log `handover/packages/HV2b.md`) |
| LE-U2 Life screen card "Where the energy goes" | LE3, LE4 | none | not started |

## The catalogued Earth on a correctly tilted Earth (27 Nov 2026)

The chain, as found by this builder thread with the climate thread:

1. **Tilt bug (test harness, fixed by the climate thread, patch `open-sol-radians`).** `sol_tests::open_sol` wrote Earth's 23.44 degrees into a radians field, so the kernel's life tests ran an Earth tilted about 97 degrees, with polar seas near 305 K and an ocean up to 322 K. The app's own loaders were right. On that Earth, `life_tests::catalogued_earth_grows_land_life_and_its_own_oxygen` passed; on the correct Earth (mean 288 K, equator 303 K, poles about 250 K) it fails: no oxygen photosynthesis.
2. **Iron supply (a real fault, fixed: package L3-air, log `packages/L3-air.md`).** A chart world's habitats were built once from today's air, and vent iron reaches the ocean only if the ocean is anoxic. With today's 21% oxygen, Earth's early ocean got no ferrous iron for 4.5 Gyr. It read the oxygen life makes later. The Archean mats ran on vent hydrogen alone: 8e11 g C a year, against the fixture's 3e15. Now the history picks the ocean from its own air box each step (`Epoch::graph_while`). Mats reach 4.2e14, and on the correct Earth oxygen photosynthesis and complex cells arrive (4030 and 4415 Myr on seed "home").
3. **Held heat (open, to design with the heat-flow thread).** The history holds today's heat flow (47 TW) for 4.5 Gyr. Vent hydrogen and iron scale with heat, and the Archean ran about three times hotter. An experiment at three times the supply gave oxygen photosynthesis at 3375 Myr, complex cells at 3760, multicellular life at 4145 and large bodies at 4160, but still no land by today. The principled step: epochs carrying the world's own heat flow at each age, read from the heat-flow thread's `CoreStep` series (read only). It is to be written up as a package before building.
4. **Late origin on seed "home" (not a bug).** The origin hazard per habitat is as written (the sea floor had reached 1.76 of its keyed threshold 1.97 at 1205 Myr). This seed draws a late start at 1310 Myr, about a 3% tail (expected near 380 Myr). Life began in a polar land pool, which is legitimate.
5. **The lottery (with Alex).** Oxygen photosynthesis is calibrated as about 95% likely on a fixture Earth, so a real Earth can lose the draw. The recommendation, with the coordinator for Alex: condition a catalogued world's history on what is observed today (21% oxygen, land plants), for example by redrawing with the next keyed attempt until the end agrees, and saying so in the view. Nothing is built for it.

Until 3 and 5 are settled, the correct-tilt loader patch waits with this thread, unpushed. The Earth test cannot honestly pass on seed "home", and it is not loosened or skipped. LE5 and LE6a wait too, since they change the dated history.

The V3 sample Sol's Earth (1 GW, "alive, barely") is the same chain. Its history never evolved oxygen photosynthesis, and its colder climate gives about a sixth of the opportunity.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package.
