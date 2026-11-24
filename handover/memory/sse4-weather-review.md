---
name: sse4-weather-review
description: 21 Nov weather review (accepted) and the redo W7a-W12: findings, package order, W7a state (PR 84), volatile split agreed with exogeology
metadata:
  type: project
  modified: 2026-11-21T17:13:52.736Z
---
Thread "Review weather against new geology" (21 Nov 2026). Report: /mnt/project-files/sse4/SSE4-weather-review.md, ACCEPTED by Alex 21 Nov (section 9 recommendations are rulings), synced to handover/design and the build plan (PR 74). Build thread "Build shared climate and weather" (Opus) owns W7a-W12, one PR each. Alex: V3 default data is authored over time and may be wrong, so V3 values carry "authored (V3)" and disagreements with our physics are flagged, not silently overridden. Section 10 lists the papers each package must read before registering.

Original finding: four disconnected climates (wear, cover, weather rule block, zonal EBM); spin always positive; Rhines wind fixed 12 m/s; about 25 regime gates.

W7a (21 Nov, PR 84): crate sse-climate + packs/base-climate is now the ONE climate (seasonal EBM with heat capacity, Siler 2018 moisture, Held-Hou/Rhines circulation, Glance/Orbit keyed results and Cache). Wear, cover and both weather blocks read it; spin signed (Venus -5832.5 h); kernel `weather` op takes system+body and matches survey_surface's climate key. Agreed with "Build SSE4 proven core": W7a owns climate at D0/D1, they read it. Limits for later packages: locked worlds zonal, spin/tilt defaults 10 h/20° when Body lacks them (carry from formation and V3 in W7b), 12 m/s wind, no CO2 frost (W8), moon year from its own orbit.

Order after W7a: W7b circulation as strengths, W9 supply to geology, W8 moving ices with S17a, W7c land and sea, W10 storms/aerosols/giants as strengths, W11 levels and cost, W12 Workshop blocks.

Volatile split agreed with exogeology thread: S17 owns ground ice; shared `sse.volatile.species` in base physics; weather owns `sse.volatile.exchange`; shared `sse.wind.saltation`; haze shared with `sse.tholin.production`. Strength blocks named `sse.weather.<process>`.

Related: [[weather-track]], [[sse4-exogeology-review]], [[sse4-processes-not-classes]]
