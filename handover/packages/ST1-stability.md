# ST1: orbital stability on the card (Alex, 23 Nov 22:03, "yes to all six")

Alex asked that every check be an inspectable physics block, "so you can see the working". So every check here is a formula block in `packs/base-physics/formulas/stability.json`, written by `scripts/build-stability-blocks.py`. Each block carries its source, its fitted numbers as Model numbers, and worked examples that run on load. The kernel only chooses which bodies to compare and then runs the blocks.

## Blocks
| Block | What it says | Source |
|---|---|---|
| `sse.stability.hill_spacing` | The spacing in mutual Hill radii, and the closest gap (inner apoapsis to outer periapsis). Verdicts: crossing (3), under 2√3 (2), under the lasting spacing of 10 (1), lasts (0) | Gladman 1993; Chambers et al. 1996; Pu and Wu 2015 |
| `sse.stability.amd_term` | One orbit's angular momentum deficit, Λ(1 − √(1−e²) cos i) | Laskar 1997; Laskar and Petit 2017 eq. 2 |
| `sse.stability.amd_collision_step` | One Newton step towards the collision root of eq. 35. The kernel runs it 16 times from e = 0, which reaches 1e-15 over α 0.01 to 0.99 and γ 1e-4 to 1e4 (checked against bisection) | Laskar and Petit 2017 eqs. 35 to 37 |
| `sse.stability.amd_critical` | The relative AMD C/Λ′, the collision critical value, and the critical value where resonances overlap (used for 1 − α < (4/3)(2rε)^¼ + √(2rε)/4; zero where even circular orbits overlap). Verdict 1 is "has the stretch to collide in time" | Laskar and Petit 2017 eqs. 28, 29, 39; Petit, Laskar and Boué 2017 eqs. 29, 89, 97, 100, 101 |
| `sse.stability.moon_limit` | The widest lasting moon orbit, prograde or retrograde | Domingos, Winter and Yokoyama 2006 |
| `sse.stability.kozai` | The mutual inclination, e_max = √(1 − (5/3)cos² i), and the swing time | Kozai 1962; Lidov 1962; Kiseleva, Eggleton and Mikkola 1998 |
| `sse.stability.binary_limit_one_star` | The widest S-type orbit | Holman and Wiegert 1999 eq. 3 |
| `sse.stability.binary_limit_both_stars` | The nearest P-type orbit | Holman and Wiegert 1999 eq. 4 |

The g of Petit et al. 2017 is 3⁴(1−α)⁵/(2⁹rε) − 32rε/(9(1−α)²). It is checked against the paper's own limits: g = 0 gives 1 − α = 1.46ε^{2/7} (their eq. 77), and the eccentric limit gives 1.38ε^{1/5}c^{1/10} (eq. 79).

## Op
`orbit_stability {system, bodies?, add?, at_s?}` (Light). It reads the universe's facts at `at_s` (all of them when absent), or reads inline `bodies`. `add` is laid over those facts, and then only the checks that concern the added bodies come back: those naming an added body, and the deficit check of any family it joined. This is the card's verdict on add.

The reply is `orbit_stability_is {verdict: lasts|watch|unstable, checks: [{check, block, around, bodies, level, says, inputs, results}], unchecked: [{check, bodies, missing}]}`. A body that lacks what a check needs is named under `unchecked`, never guessed.

What gets compared:
- Hill and AMD: neighbour pairs among the non-star bodies round each host.
- Moon limit: for a world that circles a star.
- Kozai: against every outer body at least as heavy, and against companion stars. It is a note (level 1), never a loss.
- Holman and Wiegert: for stars circling stars. Planets inside the companion's orbit are S-type, those outside are P-type, and the companion's own planets are S-type against the primary.

## Checks (`sse-kernel/src/stability_tests.rs`)
- The Solar System: every Hill spacing is over 2√3. The inner pairs are AMD-unstable and the outer pairs AMD-stable, which gives the verdict "watch", as Laskar and Petit 2017 (section 5.4.1) say. The inner four alone are AMD-stable, the "hierarchical AMD-stable" case.
- Two Earths 0.02 au apart are unstable; orbits stretched until they cross get level 3; neighbours 12 mutual Hill radii apart last.
- The Moon stays. A prograde moon at 0.6 R_H is lost; a retrograde one there stays.
- Alpha Centauri A: a planet at 1 au lasts, the limit is about 2.8 au, and 5 au is lost. Kepler-16 b lasts round both stars. A companion 300 au out, tilted 60°, gives e_max 0.764 (a note).
- Adding a body returns only the checks it touches, and a body with no mass is listed as unchecked.

`sse-atoms`: the first-wave atom count goes from 94 to 102, and 13 new port kinds are described. `packs/base-physics/pack.json` is re-blessed for the new kinds.

## Golden (on purpose)
`tests/golden/contract.jsonl` changes in five places:
- line 1: the op list gains `orbit_stability`;
- line 41: the unknown-op message names it;
- line 318: the reach list for G gains the stability blocks that read G (`amd_term`, `amd_critical`);
- lines 45 to 50: the stack reports after load, because `tests/fixtures/packs/base-physics.json` (a copy of the real pack) gains the 13 new port kinds;
- line 355: the new `orbit_stability` case (the Earth, the Moon and Mars, with an Earth added at 1.03 au).

No G goldens change.

## Next
- Resonance flags (mean motion ratios near small whole numbers) as a block.
- A direct integration on request (about 1 Myr) through `sse-time`'s n-body.
- Moons against their star's Kozai pull, once moon inclinations name their reference plane.
