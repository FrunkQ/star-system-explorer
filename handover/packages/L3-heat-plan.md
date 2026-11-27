# L3-heat (plan, not built): a chart world's history reads its own heat at each age

Track LE (life on the chart's worlds, L3). This is a write-up for the coordinator before building, as ruled 27 Nov 2026. It follows L3-air (`packages/L3-air.md`) and is point 3 of the chain in `status/life-energy.md`.

## The gap

A chart world's deep-time history holds today's heat flow for its whole life (`WorldReading::heat_flow_w_m2`, one number: `sse-kernel/src/life.rs` reads the surface envelope's heat at "now" only). Everything life draws from the interior scales with that heat: vent hydrogen (`vent_h2_mol_yr_per_w`), vent iron in an anoxic ocean (`anoxic_fe2_mol_yr_per_w`), the vents' own presence (`vents_above_w_m2`), the deep crust's warmth and the air box's volcanic reductants. The Archean Earth ran about three times hotter, and the pack's own source for the iron rate already says so ("about 1e15 mol a year in the Archean over about three times today's heat flow"). So the young Earth's chemistry is starved about threefold. An experiment at three times the supply moved oxygen photosynthesis from 4030 to 3375 Myr on seed "home".

## The source (read only)

The heat history is already computed. The surface envelope's `timeline` (`sse-surface`, `RegimeSpan { regime, from_gyr, to_gyr, heat_flow_w_m2: [start, end] }`, radiogenic plus tidal) is what `life.rs` interpolates for "now". The heat-flow thread's `CoreStep` series (`sse-surface/src/core.rs`, `q_surface_tw` by `t_gyr`) is the fuller thermal history. Which of the two is the door is the heat-flow thread's call. This package reads one and changes nothing of either.

## The step

1. `WorldReading` gains `heat_history: Vec<(at_myr, heat_flow_w_m2)>`, sampled by the kernel at the envelope's span edges (or the `CoreStep` series) from `habitable_from_myr` to now. When it is absent, the world's history is as today (fixtures, old callers).
2. `world_life::life` builds one epoch per sample, `today-held` becoming `<name>.heat@<myr>`. Each takes its own stack at that age's heat (vents on or off, hydrogen and iron supply, crust warmth) and its own anoxic variant (L3-air). The air box's volcanic reductants follow the same samples.
3. Climate, light and the star stay today's: this step moves only what the interior supplies. The young Sun and the early climate are the time pipeline's and the climate history's, later.
4. There are no new numbers, and the epochs are capped (about eight; merged where the heat changes by less than 20%) to keep the history's budget cache small.

## Size and risk

Medium: about 150 lines in `world_life.rs` and `life.rs`, plus tests. The core contract's life cases are fixtures, so no core line moves. Chart-world life tests and area goldens with chart life may move, and are re-blessed only for this cause.

## Checks it would carry

- The catalogued Earth's vent supply 3.5 Gyr ago is about three times today's, read from the envelope, not typed.
- A world whose interior has died (Mars) loses its vents in the history when its heat falls below `vents_above_w_m2`, and its mats go with them, dated.
- Fixtures unchanged (no `heat_history`); deterministic.

## What it will not do alone

On seed "home" the experiment still gave no land by today, because the origin draws a late start (a 3% tail). That is point 5, conditioning on what is observed, which is with Alex.
