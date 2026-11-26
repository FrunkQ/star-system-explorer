# SB4b: comet ion tails and warm rocky members (kernel fixes)

Raised by the coordinator from SB3b (PR 613), which drew comets in system True and saw two faults in `sse-small` comet.rs without changing them. Alex's standing rule: fix the cause, never tune.

## Fault 1: ion tails of 55 to 81 AU
**Cause.** The ion tail was `solar wind speed x the dominant gas's photo lifetime`, with that lifetime growing as r^2. The lifetime of a neutral molecule says how long it lasts before sunlight breaks or ionises it, so it sets where in the coma ions are born (the coma's own scale length is `outflow x lifetime`). It is not how long an ion travels down the tail. The old ion tail was therefore just the coma stretched by the ratio of wind speed to outflow speed (about 470 at 1 AU, growing as r^0.5), so it grew as r^2.5 outward while the comet faded: 0.06 AU at 0.5 AU, 185 AU on the small-bodies golden case.

**Fix.** Both tails are now what the star has carried off over one tail age (pack `comet.tail_age_days`, 10 days, the dust tail's existing stand-in): the dust by radiation pressure (unchanged), the ions by the solar wind, `solar_wind_km_s x tail age` = 3.5e8 km, about 2.3 AU, at every distance. Ions are picked up slower than the free wind and accelerate (the flow is mass-loaded near the nucleus), so this is an upper bound. Real ion tails run to about 1e8 km and the longest seen reach a few AU (recalled, unverified: paper search is out until 1 Dec 2026). The drawing already fades a tail by production, so a faint far comet shows faintly.

## Fault 2: rocky members at 0.1 AU shown as big comets
**Cause.** Activity was gated on bulk ice (`makeup.ice > 0`), so an S or Q type member with a trace of ice in its makeup sublimated from its whole active fraction at 0.1 AU, where the flux is enormous (seed "cold harbour": up to 4e7 kg/s). The member's resolve already has the rule that such a surface lost its ice over the system's age (pack `ice.survives_below_k`, 150 K, read by the type label), but the comet code did not read it.

**Fix.** `sse_small::keeps_surface_ice`: a member wakes only if its blackbody temperature, at the distance it spends its time, is below `ice.survives_below_k`, the same rule its label reads; a GM-made comet nucleus (`made-comet`) is a fresh arrival and keeps its ice. `activity` now takes the whole `Settings` to read it. Fudge row SB-comet-surface-ice.

## Checks
- `cargo test -p sse-small comet`: 6 pass, two new. `an_ion_tail_reaches_a_few_au_at_most_and_does_not_grow_outward` (0.1 to 5 AU at 0.5 to 12 AU, never longer further out) and `a_warm_member_with_ice_in_its_bulk_has_no_activity` (880 K belt member with 5% ice: no activity; same member at 40 K: active; made nucleus at 300 K: active). Both were run with the old rules put back and failed (ion tail 0.055 AU at 0.5 AU and growing; warm member active).
- `scripts/check.sh` green (the citations warning is the existing 28 papers, none added here).
- Golden: only `tests/golden/areas/small-bodies.jsonl` moves, re-blessed; no core line moves. Two changes: `cloud:0/obj:40` (debris belt, 97 K) keeps its activity and its ion tail goes from 2.8e10 km (185 AU) to 3.5e8 km; the GM-made contact binary `small:gm-1` at 1.5 AU (282 K) is no longer active, which now agrees with its own label ("C, carbon-rich, its ice long since turned to clays"). Only kind `made-comet` counts as a fresh arrival; a made asteroid or pair is made of the old belt it sits in.

## Seen, not changed
- The coma is still `outflow x lifetime`, which grows as r^1.5: a CO2-driven comet at 7 AU has a coma scale of several million km. That is the right Haser scale length but larger than the visible coma; the drawing fades it by production. A visible-coma rule is part of the Haser model the fudge row points to.
- Main-belt comets (ice dug up by impacts) do not wake under the surface-ice rule. Recorded in the fudge row.
