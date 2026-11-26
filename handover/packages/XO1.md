# XO1: gravity blocks and the bend table

Track XO (exotic objects), design `handover/design/SSE4-exotic-objects-design.md` (accepted 24 Nov 2026), sections 3.2, 8 and 11. Built 24 Nov 2026 by the thread "Build exotic stars and black holes".

## Two parts

Adding any block to the base registry changes `tests/golden/contract.jsonl` (the constants reach report lists every block), so XO1 is split:

- **Part 1 (this pull request): the bend table.** No existing golden changes; one new golden file.
- **Part 2 (landed with XO18 part 2, in the golden lane): the nine gravity blocks** below, in `packs/base-physics/formulas/gravity.json`, their tests in `crates/sse-gravity/src/blocks_tests.rs`; sixteen kinds in `sse-atoms` `KIND_DESCRIPTIONS`; first-wave count 122 to 131 (132 with `sse.belts.latitude_falloff`). Golden: `tests/golden/contract.jsonl` changes on purpose: the base-physics stack hash (new blocks) and the constants reach report, which lists every block.

## What was done

- **Nine gravity blocks (part 2)** as formula blocks in `packs/base-physics/formulas/gravity.json` (family `sse.gravity`), loaded with the other base formulas (`crates/sse-atoms/src/formulas.rs`), each with a worked example run on load:
  `gravitational_radius` (GM/c²), `horizon` (by spin), `photon_orbit` (by signed spin), `last_stable_orbit` (Bardeen, Press and Teukolsky 1972, eq. 2.21, signed spin: below zero means orbiting against the spin), `shadow_angle` (Synge 1966, static viewer at any distance outside 3 r_g), `weak_bend` (4 r_g / b), `einstein_angle`, `clock_rate` (hovering and on a circular orbit), `tide` (2GM/r³). G and c come through `constants/G` and `constants/c`, so an edited sheet reaches them. Sixteen new port kinds are described in `sse-atoms` `KIND_DESCRIPTIONS`; the first-wave atom count went from 122 to 131.
- **The bend table**, new crate `crates/sse-gravity` (`bend.rs`): 256 impact parameters (128 captured, 128 escaping, both crowding towards √27) by 256 angles swept (0 to 3π), each cell u = r_g / r, plus per row: captured, end angle, turn, closest approach. Worked out with RK4 on `+ − * /` only (32 steps a column), the turning point by bisection. About 2 million steps; under 50 ms natively. `BendTable::hash()` over the layout and every number's bits; `u_at(b, φ)` reads it back; `flat()` gives the float-texture layout (per row: b, captured, end angle, turn, then 256 cells).
- **Same bits everywhere**: the golden scenario `tests/golden/gravity.json` (new file; no existing golden changed) holds the hash and seven sample rows, and runs in the browser harness through `sse-wasm`'s engine list (`"gravity"`), compared in `tests/harness/run-node.mjs` like every engine.

## Checks (design section 11)

Rows marked (2) run in part 2.

| Row | Result |
|---|---|
| Sun's limb (2) | 1.751″ (`starlight_grazing_the_sun_is_bent_by_1_75_arcseconds`) |
| Bend table | captured exactly below √27; sweep matches an independent quadrature to 10⁻⁶; turn matches the four-term weak-field series (Keeton and Petters 2005) to 2 × 10⁻⁵ at b = 64. The design's "4/b to four figures" holds only far outside the table (at b = 64 the first term alone is 5% low, which is why the next term is stated in `weak_bend`'s words); hash pinned in the golden |
| M87's centre (2) | shadow 39.7 µas across (measured ring 42 ± 3) |
| Milky Way's centre (2) | shadow 51.9 µas |
| Last stable orbit (2) | 6, 1.237 (a = 0.998), 9 (a = −1) r_g; horizon ≤ photon orbit ≤ last stable orbit at every spin tested |
| Clock rate (2) | 0.8165 hovering at 3 Schwarzschild radii |
| Tide (2) | 10 Suns at 1,000 km: thousands of g over two metres; 4 million Suns at its horizon: under 0.01 g |

`cargo test -p sse-gravity` (6 tests in part 1, 11 with part 2), `scripts/check.sh`.

## Citations and fudges

`docs/citations/exotic-objects.md` (nine rows: metadata of BPT 1972, Synge 1966 and Keeton and Petters 2005 checked through ADS and arXiv search results; the EHT figures are still from memory and marked unverified). `docs/fudges.md` section "Exotic objects": no-spin table, far viewer, table edges, no-spin shadow, and why the table is Rust.

## Integration points

- **XO2** (op `read_light_paths`) returns the table's hash and the sizes from these blocks; the renderer fetches the table once (`flat()`), keyed by the hash. Adding the op changes `contract.jsonl` (its op list), so XO2 goes through the golden lane.
- **XO4** (renderer) reads `flat()` into one float texture; `row_of_b` and `phi_of_col` are the index formulas a shader repeats.
- **XO18** (approach diagram) reads `clock_rate`, `tide` and `last_stable_orbit` for compact bodies.
