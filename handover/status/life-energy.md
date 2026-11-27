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
| LE-U1 Energy flow life branch | LE2, the Energy flow view (PR 670, HV) | none | not started |
| LE-U2 Life screen card "Where the energy goes" | LE3, LE4 | none | not started |

Only the owner thread edits this file. One row per package: update it in the same pull request as the package.
