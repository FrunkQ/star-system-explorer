# Status: life from the energy it can get (track LE)

Owner thread: "Design life's spread and population from energy" (design only, 26 Nov 2026). No builder thread yet. Until one exists, each package below belongs to the thread that takes it, and that thread records it in its own status file too.
Design: `design/SSE4-life-from-energy-design.md`, **ACCEPTED by Alex 26 Nov 2026 23:50 and 23:54 UTC** (rulings in the note's section 0). Builders may take the 4.0 baseline packages below. The 4.1 bucket (note section 9A: LE6b, LE7 to LE11) is not for building in 4.0.

In one line: free energy captured, set against the cost of staying alive, decides whether a habitat lives, how its energy is shared among kinds of life by diet, and how fast life spreads, evolves and recovers.

**Golden lane.** `life_view` is in the core contract (`tests/contract/requests.json`, lines in `tests/golden/contract.jsonl`), so every kernel package below moves core lines and goes in the core lane, one at a time, in the coordinator's order. Each also moves its own `sse-life` test goldens.

**Not needed:** the gradient model wiring (`gradients/1` in the kernel's runs). LE1 puts the keep-alive test in `budget::finish`, which both models share, and every later package builds on that. Habitat light (`light_w_m2`) is already on the habitat profiles, so LE2's "arriving" per habitat needs nothing new. Only the world-level comparison with absorbed sunlight waits on the climate sunlight figures HV asked for, and shows as not published until they land.

| Package | Needs | Golden | State |
|---|---|---|---|
| LE1 keep-alive test in the shared step, starved and laboratory maintenance in one shared cited block, "alive, barely", biomass capped by power over maintenance | nothing | core (values move) | not started |
| LE2 energy ledger on `life_view` (arriving, offered, captured, keep-alive, surplus, brought in at zero) | LE1 | core (new fields) | not started |
| LE3 where the energy goes: power and share by diet per guild, rough numbers by size class | LE1 | core (new fields) | not started |
| LE4 speed read-out: fill time per habitat, one speed line per world | LE1 | core (new fields) | not started |
| LE5 broad-brush spread in the deep-time history (link rates, colonisation waiting times, refuges) | LE4 | core (dated history moves) | not started |
| LE6a recovery clocked in Earth-Myr of opportunity | LE4 | core (dated history moves) | not started |
| LE-U1 Energy flow life branch | LE2, the Energy flow view (PR 670, HV) | none | not started |
| LE-U2 Life screen card "Where the energy goes" | LE3, LE4 | none | not started |

Only the owner thread edits this file. One row per package: update it in the same pull request as the package.
