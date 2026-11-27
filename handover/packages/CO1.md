# CO1: Trojans for any world

Track CO (`handover/status/coorbital.md`). Thread "Add co-orbital asteroids as general physics", 26 Nov 2026.

## Why
Alex sent Anton Petrov's video "The Vera Rubin Observatory Just Solved a Decades Old Neptune Mystery" and its sources, asking that it be taken in as physics, not a Solar System special. Read at source (through the built-in browser on Alex's PC; the cloud is refused by every one of these hosts):
- Schwamb et al. 2026, ApJL 1010, L36 (iopscience, open access, full text): two new gigayear-stable L5 Neptune Trojans from Rubin's science validation survey, doubling the known four; Neptune's Trojans are estimated at 4 to 10 times Jupiter's (citing Sheppard and Trujillo, Gladman, Alexandersen, Lin); long-lived ones are primordial.
- Lykawka and Horner 2010, MNRAS 405, 1375 (OUP behind a bot check; arXiv:1003.2137 abstract read): all four giants capture Trojans from the disc during migration (probabilities quoted in the blocks); most captured are unstable and decay.
- Markwardt et al. 2023 (arXiv:2307.10542, abstract): Neptune Trojan colours (the "Trojan colour conundrum" resolved by sampling bias); colours are not used here.
- de la Fuente Marcos et al. 2024, A&A 683, L14 (aanda refused; arXiv:2403.04061 abstract): Mars's Trojans, 13 at L5 (mostly the Eureka family, possibly primordial) and 2 at L4; 2023 FW14 a temporary capture lasting millions of years.
- NSF news, 7 Mar 2022: 2020 XL5, Earth's second Trojan, temporary (several millennia).
- Emery et al. 2024, Space Science Reviews 220, 28 (open access, section 1 read): Jupiter's Trojans stable over the Solar System's age; capture in place as Jupiter grew (Marzari and Scholl 1998) does not explain their inclinations; Nice-model migration empties and refills the Lagrange regions.
- The video (transcript): the same points, plus Saturn's 2019 UO14 (temporary) and the claim that long-lived swarms need a planet much lighter than its star but heavier than its perturbing neighbours.

## What the kernel did before
Only the giant that stirred a belt caught Trojans, a fixed 7e-7 of the stirred mass (`violent::TROJAN_CAPTURE`, Nesvorny et al. 2013 for Jupiter), placed in a fixed band of plus or minus 2% of its orbit. No stability limit, no decay, nothing for other giants or rocky worlds, so a Neptune-like outer giant got Jupiter's rate, about 100 times too low against Lykawka and Horner.

## What landed
- `packs/base-physics/formulas/coorbital.json` (builder `scripts/build-coorbital-blocks.py`, `--check` in `scripts/check.sh`): `sse.coorbital.trojan_zone`, `capture_share`, `long_lived_share`, `neighbour_disturbs`, each with source, explain texts, model numbers and worked examples. Registered in `sse-atoms/src/formulas.rs`.
- `crates/sse-formation/src/coorbital.rs`: finds the inputs (adjacent worlds, nearest resonance in lowest terms, giants between a world and a belt) and runs the blocks. Masses enter as ratios only, so it reads nothing from the sheet.
- `violent.rs` `stir`: every world whose points hold catches its share; `TROJAN_CAPTURE` and `REF_TROJANS` removed. At the end of `upheave`, one swarm per world, thinned to its long-lived share; the rest goes to `solid_ejected`; a `TrojansThinned` event names the disturbing neighbour and resonance.
- `leftovers.rs`: the swarm spans the world's tadpole width instead of plus or minus 2%.
- `docs/fudges.md`: CO-capture, CO-keep, CO-neighbour, CO-temporary.

## Checks
- `coorbital_tests.rs` (6): every planet of ours can hold Trojans and a 0.1-mass companion cannot, with the limit at 27 mu (1 - mu) = 1; Jupiter's width 0.05; the neighbour rule keeps exactly Earth, Mars, Jupiter and Neptune and names Saturn's 5:2 with Jupiter and Uranus's 2:1 with Neptune; the four giants' catches sit in Lykawka and Horner's ranges (Saturn's widened to 1.5e-5); Jupiter keeps 6e-7 to 8e-7 and Neptune / Jupiter is between 4 and 10 (an absolute assertion, not only a ratio).
- Seen red: with the builder's near set to 0.01 and c to 60, `neighbours_decide_who_keeps_them` and `neptune_keeps_several_times_jupiters_swarm` both fail (Jupiter keeps 1.5e-7); restored, green.
- Existing formation tests pass: budgets close for 300 systems, resolved members are the same on every ask, reservoirs populate.
- `leftovers_tests::coorbital_census` (ignored, run by hand): across 300 Sun-like systems 38 giants keep a swarm and none lose all of one; 225 rocky worlds catch some and 32 keep any; 60 swarms lost to neighbours.
- Goldens re-blessed on purpose: `tests/golden/formation-upheaval.json` (event counts and digests of the two systems with a stirring) and `formation-leftovers.json` (cloud order and Trojan masses and spans). Reason: Trojans are now caught by every world and thinned. Core golden `tests/golden/contract.jsonl` unchanged; `tests/golden/contract-reach.txt` gains the four new blocks' "cannot follow" lines (the reach report lists every block).

## Not done
- No temporary Trojans (CO2), no swarms born in place (CO3), no survival against age (CO4), real catalogue entries (CO5).
- The neighbour rule is judged, not derived (fudge CO-neighbour). Secular resonances and non-adjacent worlds are ignored.
- Every figure in the blocks was read at source today except the replacement-physics pointers in the fudge rows marked unverified.
- The blocks run through the block door `sse_physics::run` (ER2), so an edit to their settings reaches the Trojans; no side door was added.
