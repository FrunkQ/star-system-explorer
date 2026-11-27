# L3-heat: a chart world's history reads its own heat at each age

Track LE (life on the chart's worlds, L3). Built 27 Nov 2026 by "Build life's energy baseline", on the coordinator's go for the plan (`packages/L3-heat-plan.md`), with the heat-flow thread's answer on the door. It follows L3-air and is point 3 of the chain in `status/life-energy.md`.

## The fault

A chart world's deep-time history held today's heat flow for the world's whole life. Vent hydrogen, vent iron in an anoxic ocean, the vents themselves and the crust's warmth all scale with that heat. The Archean Earth lost about three times today's heat, so its early chemistry was starved about threefold.

## The fix

- `WorldReading` gains `heat_history: Vec<HeatSample { at_myr, heat_flow_w_m2 }>`. The kernel fills it in one read function, `life.rs::heat_lost_by_age`: the interior's own thermal history (`env.core`, `CoreStep::q_surface_tw` over 4πR², by `t_gyr`). The heat-flow thread chose this door. When HF4b.1 publishes `heat_lost` (W/m² by age), and when the time-scales step times land, they swap in there and nowhere else.
- `world_life::heat_epochs` (bare form `epochs_by_heat`) makes one history epoch per real change: a new epoch where the heat has moved by more than `heat_epoch_step` (0.2, a share) since the last. There are at most `heat_epoch_cap` (8) of them; the step doubles until they fit. Each epoch takes the heat at its own start and never reads a later age.
- `world_life::life` builds each epoch's habitats (and its anoxic-ocean variant, L3-air) at that epoch's heat. The ids are `<name>.heat@<myr>`. With no history (fixtures, worlds without a thermal history) there is one `today-held` epoch as before.
- Climate, light, air and the star stay today's. Only what the interior supplies moves.
- Two new pack numbers, both bookkeeping, not physics: `heat_epoch_step` and `heat_epoch_cap` in `packs/base-life/pack.json`, with sources.

## Results

The catalogued Earth now has 6 heat epochs (0, 1240, 1840, 2500, 3260, 4200 Myr; 0.23 down to 0.073 W/m²). On the correct-tilt loader (patch `open-sol-radians`), seed "home", every kernel life test passes, land included. Archean production is 1.9e15 g C a year. Oxygen photosynthesis comes at 2105 Myr, complex cells at 2300, multicellular life at 2490, large bodies at 2495 and land at 2920.

## What it found, not fixed here

1. **Two doors for "now".** Today's heat (`heat_flow_w_m2`, the present-day biosphere) is read from the envelope's regime timeline: radiogenic plus tidal, 0.039 W/m² for Earth. The thermal history gives 0.073 W/m² at 4200 Myr (the real Earth loses about 0.087). So the history's last epoch and today's biosphere use different heats. Recommendation: read "now" from the same door once the heat-flow thread publishes `heat_lost`. This package leaves "now" alone, as planned.
2. **Mars never had vents.** Its thermal history gives 0.017 to 0.021 W/m² from the start, below the vents' threshold (0.03), so the planned check "Mars loses its vents, dated" cannot honestly be met. Published early-Mars estimates are higher (unverified until the paper search returns). This goes to the heat-flow thread. The check is replaced by a direct one of the mechanism (below), not loosened.
3. **The air oxygenates too fast (air box, L2).** After oxygen photosynthesis arrives the air reaches 14% oxygen within about 50 Myr. There is no long low-oxygen middle age (the "boring billion"), so large bodies and land come about 2 Gyr early. The volcanic reductant sink (`3e12 × exp((ref - t)/2000)` mol a year) is the place to look. It is reported, not tuned.

## Checks

- `sse-life` `history_heat_tests` (5): no history holds today's heat; each epoch takes the heat at its own start; small changes merge; the cap widens the step; samples after today are not read.
- `sse-kernel` `life_tests::earths_young_interior_runs_hotter`: the catalogued Earth's heat 3.5 Gyr ago is 2 to 5 times its own history's today, read from the thermal history, not typed. It went red with the kernel's history emptied.
- `life_tests::a_dying_interior_changes_the_history_from_its_date`: an Earth whose heat drops below the vents' threshold at 2500 Myr has the same history before that date and a different one after. It went red with the history held at today's heat.

## Goldens

None moved: the core contract's life cases are fixtures without a heat history.
