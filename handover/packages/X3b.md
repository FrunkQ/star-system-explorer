# X3b part 1: a whole system from a coded world

Status: done (19 Nov 2026), Traveller pack track. Design: handoff 7A part 2; build plan X3b. Builds on X3a (`sse-codes`) and F9 (formation pins).

## What was built (`crates/sse-codes/src/plan.rs`, general: the sheet's `system` hints name the segments)
- `plan(codes, sheet, lines, stars, extra, seed)`: the code lines become pins (X3a `pins`), the plainest world that reads back to the same codes (`typical_world`), and from it:
  - the main world's **formation pin**: mass from radius (rocky, M = R^3.7), orbit where it reaches its target temperature around the primary (luminosity from the mass; grey-atmosphere surface temperature as the base-physics atom `sse.climate.grey_surface_temperature` computes it, solved for the equilibrium temperature), with the placement written out in words;
  - the stars from stellar data (`star`: "G2 V", "M3 V", "D", "BD"; dwarf mass table, others noted);
  - the counts wanted (gas giants, belts) from the PBG segments;
  - **edits**: every coded fact for the main world, plus `provenance.generator = traveller-generated` and `provenance.code` (the line as given). Codes win: values are never moved.
  - **objections** (physics checks, `objections`): air a world too small to keep (the base-physics atom `sse.atmosphere.jeans_parameter` below 30 at a 1000 K exosphere), liquid on a surface below water's triple-point pressure. The facts they touch are written `impossible: true` with the reason, which the kernel's `set_authored` already records as authored-impossible.
- `grow(plan, system, candidates, tries)`: F9 `pins::solve` around the main-world pin; candidate seeds (`<seed>#counts-k`) are tried until the giant and belt counts match, else the closest is kept. Whatever was asked but not met is listed in `unmet` in plain words.
- `sound(codes, sheet, layout, line)`: "make this physically sound": the smallest single-symbol change that clears every objection (for `X266000-0`, size 2 with standard air, it proposes a bigger world), or the line itself if nothing objects. This is what an Orac change request will offer.
- Pack: the sheet gained `system: {giants, belts, tag, giant_min_mass_earth}`.

## Checks (`cargo test -p sse-codes`, 16 tests in all)
- Stellar data read to masses; Earth's orbit for the Sun from albedo 0.3.
- `A867975-9 814` around a G2 V: placed at about 1 au, about 1 Earth mass, no marks, tagged, and its facts read back to the same codes and population multiplier.
- `X266000-0`: one objection (Jeans parameter), pressure kept at 1 bar and marked; reads back unchanged. `X507000-0`: liquid under vacuum marked. `X766000-0`: none.
- `sound` changes exactly one symbol and clears the marks; a sound line comes back as it is.
- Growing: the main world sits on its pin, the same plan grows the same system twice, and counts are met or said plainly. Golden `tests/golden/traveller-codes.json` gained `grown` (and the pack hash moved because the sheet gained its system hints: on purpose).

## Fudges
`docs/fudges.md`, "Game-system packs" (placement temperature and greenhouse, exosphere temperature, rocky mass, star classes, giant count).

## Limits and next
- Not yet behind a kernel op: the UI will call `run_formation` with the plan's pin and star pins, `adopt`, then `set_authored` the edits on the main world. A one-shot op is part of X3a part 2 / X3c's import work.
- Only the main world is pinned; moons, a main world that is a satellite (Sa), and placing by C3 reverse placement through the climate flows are next.
- Seeded subsector rolls (random UWPs by keyed draws) are not done yet.
- Belts come from formation's leftovers; the count search over 12 candidates may still miss them (said in `unmet`).

## Golden re-bless (19 Nov, late)

`tests/golden/traveller-codes.json`, `grown` block only: after merging main
(PR 37, citations for orbits and formation), the pinned formation run for
the Earth-like test world now succeeds on attempt 7 with 2 giants and
2 belts (was attempt 1, 3 giants, 0 belts). Every UWP line is unchanged.
The cause is upstream formation constants, not this track.
