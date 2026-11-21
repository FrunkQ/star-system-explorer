# W7b: circulation as process strengths (21 Nov 2026)

Weather review section 8, package W7b, under the processes-not-classes rule. Reading notes:
`handover/packages/W-citations/W7b-reading.md`.

## What was built
- `crates/sse-climate/src/strengths.rs` (pack key `climate.strengths`): each circulation process computes its strength,
  0 to 1, from the physics, and the circulation reads the numbers, never a regime.
  - **Thermal wind and Ro_T** (Mitchell and Vallis 2010 eq. 3.3b): U = R T0 Δ_H/(2Ωa), capped at sqrt(R T0 Δ_H) for slow
    spins. The Rhines eddy wind is 0.4 U, so the cell count follows each world's forcing (Earth 11.9 m/s, Mars 11.2,
    Venus 87, a 6 h Earth 3.0). W7a's fixed 12 m/s is gone.
  - **Super-rotation** x²/(1+x²), x = Ro_T: Earth 0.032 gives 0.00, Mars 0.058 gives 0.00, Titan 17 and Venus 3600 give 1.00.
  - **Baroclinic growth** 0.31 f dU/dz / N against the radiative time c_p p/(g σ T³): Earth 0.98, Mars 0.61, Venus 0.80.
  - **Day-night contrast** (Komacek and Showman 2016 eq. 31, drag from Koll and Abbot 2016), times the share of a night
    the air has to cool in, 1 − exp(−day/2τ_rad): the same curve for a locked world and a slow rotator.
  - **Collapse**: the night side against the main gas's frost point (Clausius-Clapeyron), floored at Wordsworth 2015's
    thin radiator.
  - **Collisional air**: Knudsen number at the ground. Winds and fluid processes scale with it (the Moon gets none).
  - **Regime lengths**: deformation radius and Rhines length over the radius (Haqq-Misra et al. 2018).
- The circulation lays out cells from the Held and Hou edge and the Rhines count alone. The regime is a **label** read
  off strengths and facts by pack rules (airless: collisional < 0.5; thin-air: radiative time under 10 days;
  slow-rotator: super-rotation > 0.5). Every Solar System fixture keeps its label.
- **Spin and tilt carried through**: `sse_surface::Body` gains `spin_hours`, `obliquity_deg` and `spin_source`.
  Formation runs (Modern block) hand over F4's settled spin and tilt ("formation"). Bodies adopted from facts read
  `body.rotation_period` and `body.obliquity` / `body.axial_tilt`, marked "authored (V3)" when they come from a V3 save
  (ruling 5), and the climate's `assumed` list names the source. Surface fixtures carry NASA fact-sheet spins and tilts
  (Venus backwards). `day_hours` reads the body's spin too.
- **Tilt and eccentricity** (after Alex asked about heavily tilted worlds): the Hadley share of the moisture transport
  is centred on each season's moist-static-energy peak (the energy-flux equator) instead of the geographic equator,
  so the rain belt chases the overhead sun (a 98° world rains on its summer pole). Seasons now follow the real orbit:
  Kepler's equation gives the solar longitude and the sunlight factor (a/r)², with `Conditions.perihelion_lon_deg`
  placing perihelion in the seasons (Earth 283°, Mars 251° in the fixtures). With e = 0 the old seasons come back
  exactly. Sunlight was already right at any tilt (polar day and night), so at 54° the poles match the equator and
  beyond it they are warmer.
- **Tested ranges** (after Alex asked whether the model can make brand-new weather): `climate.strengths.tested`
  lists the range each paper covered (Mitchell and Vallis's Ro_T 0.02 to 10.5; Komacek and Showman is a giant-planet
  theory; Wordsworth's 0.01 to 10 bar and 0.2 to 3 Earth sunlights; Haqq-Misra's 1 to 100 day rotators). A world
  outside one gets an `outside` line ("an extrapolation") in its strengths and its "Why?". New combinations of known
  processes are expected; new mechanisms are not in the model.
- Example: `cargo run -p sse-climate --example strengths` prints every weather fixture's strengths and label.

## Tests (`crates/sse-climate/src/strengths_tests.rs`, kernel adopt test)
- Solar System labels unchanged; Venus super-rotates (> 0.9) and Earth does not (< 0.01); Ro_T in Mitchell and Vallis's
  ranges.
- A spin sweep of Earth's air from 1 h to its year, 5 % apart: no strength moves more than 0.05 between neighbours,
  and the Hadley edge and eddy wind change no more than 6 %. Locking at that spin changes nothing.
- A cold thin CO₂ locked world collapses; the temperate locked fixture does not; thicker air narrows the contrast.
- Pluto's 10 µbar is collisional (> 0.99); the Moon's air is not, and moves no wind.
- A 54° world has poles within 8 K of its equator; a 98° world has warmer poles, a wetter pole than equator, and each
  season's wettest band in the summer hemisphere; Earth's rain stays tropical. Mars's southern summers are over 10 K
  hotter than its northern ones; a circular orbit reproduces the old seasons exactly.
- Kernel: the imported V3 Earth's weather spins at V3's 23.9345 h and its survey climate says "authored (V3)"; a
  formation world's says "(formation)".

## Goldens changed (on purpose)
surface-cover, surface-wear, weather, weather-storms, weather-swap, weather-aerosols and the survey and weather lines of
contract. Reason: the Rhines wind now follows the thermal wind, surface fixtures carry their real spins and tilts, and
formation and V3 worlds carry theirs. Main (with S3b's sediment budget) was merged in first, so this is one bless for both.

## Known limits (docs/fudges.md, W7b table)
- The strength curve of super-rotation is ours (the paper gives only the onset).
- Lindzen and Farrell's 0.31 is confirmed only by a citing source.
- Komacek and Showman's ratio is a hot-Jupiter theory; on rocky worlds it gives small contrasts for Earth-like air and
  softer contrasts for thin CO₂ than Wordsworth's GCM. A thin CO₂ locked world at Earth's flux does not collapse here,
  although Wordsworth finds it does below about 0.14 bar. Koll and Abbot's rocky theory is the better next step (W7c).
- Formation spins are primordial (no tidal despinning yet).
- Surface bodies carry no longitude of perihelion yet, so a formation or V3 world's perihelion falls at its northern
  spring equinox (eccentricity itself is read). Precession and obliquity cycles come with W9's insolation cycles.
- The circulation layout (cell edges, storm tracks) is an annual one; only the rain belt moves with the seasons.
- Year phase still counts from the northern spring equinox (dates keep their seasons); the orbit's mean anomaly is set
  from the perihelion longitude.
- On a world with an almost flat moist static energy (Titan) the peak the rain belt follows can hop between bands from
  season to season. Titan's annual rain is now wettest toward the poles with a dry equator (as the lakes and dunes
  suggest); the Titan sky test now looks for methane rain anywhere rather than in one tile.
- Weather formations still gate on the label; W10 turns them into strengths.
- The strengths are not yet in the Workshop (W12) or the zonal temperature profile (day-night EBM, W7c).

## Next
W9: climate supply to geology (rain, snow, runoff per cell and epoch; shared saltation; insolation cycles).
