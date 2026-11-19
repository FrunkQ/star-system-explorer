# Fudge register

Alex, 19 Nov 2026: "the physics is WAY more interesting than faking it. But understand a browser has limited
capability. Fall on physics rather than simplification where practical. If we decide to "fudge" it is a known and
documented fudge with a damn fine reason to exist."

A fudge is any deliberate simplification, stand-in, tuned constant or placeholder that is not the published
physics. Each one is listed here, in the PR that adds it, by the thread that adds it.

Columns: **where** (file and item), **what it does instead of the physics**, **why** (the damn fine reason:
browser cost, missing data, not built yet, tuned to match an observation), **real physics that would replace it**,
**status** (stand-in until package X, permanent with reason, to review).

Add entries under your area. Keep one row per fudge.

## Stars

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Orbits and formation

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Interiors and heat

### Fudge register

| Where | Instead of the physics | Why (as the code/log says) | Real physics to replace it | Status |
|---|---|---|---|---|
| crates/sse-atoms/src/heat.rs:220, `sse.heat.active_geology_lifetime` param `q_crit` = 0.019 W/m² | A fixed radiogenic heat-flow threshold for "geology stops" | "a calibration, not a law": half of Earth's present radiogenic flux, tuned to put Mars's widespread volcanism ending about 1 Gyr ago and the Moon's mare volcanism in its first 2 Gyr | Parameterised mantle convection (Nu–Ra scaling, stagnant-lid versus mobile-lid regime) with secular cooling: melt production when T_mantle exceeds the solidus at lid depth (e.g. Foley and Smye 2018; O'Rourke and Korenaga 2012) | documented, editable param |
| heat.rs:153, :226 (`radiogenic_heating`, lifetime) | Radiogenic heat only: no primordial (accretion and core) heat, no tides, no lid style | Stated in the science text | Add a secular-cooling term (Earth loses about as much again; KamLAND 2011) and tidal heating input (`tides.heating_power` exists) to the heat-flow sum | documented |
| heat.rs:71 (`radiogenic_heating`) | Every body's budget is BSE today wound back by the **Sun's** age 4.567 Gyr, then scaled by `rel`; body age is time since formation | Implicit: rel is "relative to the Sun's birth cloud" | Fine if `rel` is defined at the star's birth. Document that the body is assumed to form at its star's birth (Myr offsets ignored) | undocumented, harmless |
| heat.rs:37-52 isotope shares | Atom fractions used as mass fractions | Oversight | Mass fractions (fix P1/P2) | to fix |
| heat.rs:176 (`sse.heat.mantle_mass`) | All U, Th, K in silicate; core gets none; crust not separated | Lithophile partitioning (MS95) | Fine to first order; K in the core (up to a few hundred ppm, debated) and crust/mantle split for heat-flow profiles | documented |
| heat.rs:265-267 (`al26_planetesimal_heating` params e_dry_start 6e5, e_dry 1.8e6 J/kg, dry_residual 0.1) | Tuned energy thresholds and a log-linear water-loss ramp | Params with short descriptions, no source | Derive from c_p ≈ 800–1000 J/kg/K × (T_dehydration ≈ 600–900 K − T_0) for the start and (T_solidus ≈ 1400 K − T_0) + latent heat ≈ 4e5 J/kg for melting; residual water from melt solubility (e.g. Lichtenberg 2019 methods) | tuned, uncited |
| heat.rs:272 (`al26_planetesimal_heating`) | Adiabatic upper bound: all Al-26 energy stays in the body | "for a body larger than a few tens of kilometres" | Conductive cooling with radius: τ_diff = R²/κ against τ_Al = 1.03 Myr (a radius input and a loss factor 1 − f(R²/κτ)) | documented |
| heat.rs:115 Al-26 initial | Single canonical 26Al/27Al for all bodies, scaled by `al26_rel` | Canonical CAI value | Fine; note the debate over heterogeneity (lower in chondrule precursors) | documented in text |
| crates/sse-atoms/src/planet.rs:21-26 (`radius_from_mass`) | Chen and Kipping median relation, no scatter, no composition, no irradiation or age | "a forecast, not a composition model" | Composition-aware chain: rocky (Zeng 2016) + water mantle + H/He envelope (Lopez and Fortney 2014) for sub-giants; giant evolution tables (Fortney 2007 / Baraffe) for giants | documented |
| planet.rs:126-129 (`rocky_radius`) R⊕ unit | Uses equatorial R⊕ 6378.1 km in place of Zeng's 6371.0 km | Oversight | Fix P4 | to fix |
| crates/sse-formation/src/early.rs:149-151 (`rocky_radius`) | Zeng 2016 used outside 1–8 M⊕ | "a labelled extrapolation" | For M < 1 M⊕ use Seager et al. 2007 / Zeng 2016 table models; the 1/3.7 exponent overestimates compression in small bodies | documented |
| early.rs:212 and crates/sse-place/src/model.rs:197 | Water mantle as radius × (1 + 0.5 × water fraction) | "Zeng et al. 2019: 50 percent water, about 25 percent larger" | Zeng et al. 2016/2019 tabulated rock-water curves or a two-layer EOS; the linear factor is a fit to one point | documented-ish, reference incomplete |
| early.rs:207 | FeO oxidation: a water-rich body loses up to half its core, `ox = min(water/(water+rockmetal), 0.5)` | "first order ... (sketch)" | Redox equilibrium Fe + H2O → FeO + H2 with a water budget limit | sketch |
| early.rs:179 (`GIANT_GAS_FRACTION` 0.5), crates/sse-formation/src/violent.rs:260 (giants 11.2 R⊕) | Fixed giant radius for encounters; gas fraction cut for "giant" | "11.2 for giants (as F6)" | `radius_from_mass` (CK17) or giant cooling tracks | stand-in |
| early.rs:156-160, model.rs:198-202 (Lopez and Fortney envelope) | Passes core or solid mass where LF14 uses total mass; comment says "solar metallicity, enhanced opacity" with age exponent −0.18 | Not explained | Re-read LF14 eq. 3: total mass, and pick one opacity case (−0.11 solar, −0.18 enhanced) | inconsistent, to check |
| crates/sse-place/src/lib.rs:344 | Shoreline escape velocity with a fixed CMF 0.33 and Zeng radius at the upper mass prior | Not explained (quick bound) | Use the drawn body's CMF and the `rocky_radius` atom | stand-in |
| crates/sse-atoms/src/layers.rs:35 (brine `osmotic_coefficient` 0.93) | One constant osmotic coefficient for every salinity and temperature | "non-ideality of the brine"; good to about 10 percent below 20 percent salt | Pitzer model for NaCl–H2O (φ(m, T)), or tabulated freezing points (CRC), ending at the 252 K eutectic | documented |
| layers.rs:67 (ice shell) | Conduction only, k = 651/T, no convection, no porosity, no salt or clathrate | "No convection: thick shells convect" | Stagnant-lid convection threshold (Ra_crit) and Nu scaling for thick shells (e.g. McKinnon 1999), porosity-reduced k near the surface | documented |
| layers.rs:116 (hydrostatic) | Constant g and ρ | "fine for oceans and crusts" | Integrate dP/dr = −ρ g(r) with g(r) from enclosed mass (Adams–Williamson with an EOS) | documented |
| crates/sse-import-v3/src/lib.rs:509-510 | Missing V3 ocean depth taken as Earth's mean 3688 m | "a missing depth is taken as Earth's mean" | Fine as a default; cite (Charette and Smith 2010) and flag the value as assumed in evidence | uncited default |

### Candidates to move into flows

| Where | What it computes | Blocks it would need | View that would show it |
|---|---|---|---|
| crates/sse-atoms/src/heat.rs:61-78 `radiogenic_heating` (one atom, four isotopes inside) | Sum over 4 isotopes of c·share·e^{λ·4.567}·rel·h·e^{−λt} | `sse.heat.isotope_heating` (one isotope: concentration, share, H, half-life, rel, age; cited per isotope) ×4 + a `sum` block; the budget params become pack data per isotope | Stacked-area chart of W/kg against time (0 to 15 Gyr) per isotope, with "now" marked: shows U-235/K-40 early and Th late |
| heat.rs:213-251 `active_geology_lifetime` (bisection inside an atom, recomputing the whole heat chain) | Time when the radiogenic flux falls to q_crit | Reuse the planet-heat chain (heating → power → flow) as a function of age, plus a generic "time where a curve crosses a threshold" solver block (fixed 80 bisections, deterministic) | Heat flow against age curve with the q_crit line and the crossing marked, and Earth, Mars and Moon overlaid |
| heat.rs:253-292 `al26_planetesimal_heating` (energy and water loss in one atom) | Remaining Al-26 energy after formation time; water kept | Split: `sse.heat.al26_energy` (H0 from Al wt%, ratio, W/kg: each a cited param) → `sse.planetesimal.devolatilisation` (energy to water kept, with thresholds derived from c_p·ΔT + latent heat blocks) | Energy against formation time (log) with dehydration and melting bands; slider for `al26_rel` |
| crates/sse-place/src/model.rs:186-208 `radius_re` and crates/sse-formation/src/early.rs:149-160, :212, violent.rs:258-272 (three Rust copies of the radius chain) | Radius from mass and make-up: CK17 for giants, Zeng rock × water factor + LF14 envelope | Existing `sse.interior.rocky_radius`; new `sse.interior.water_mantle_inflation`, `sse.envelope.lopez_fortney_thickness`, and a `select` block (gas fraction ≥ 0.5 → `sse.planet.radius_from_mass`). One flow `planet.radius` used by place and formation | Mass-radius diagram (log M against R) with Zeng iron/rock/water curves, CK17 median line and the body as a point, with its layer stack (core, mantle, water, envelope) to scale |
| crates/sse-formation/src/early.rs:162-245 (XUV fluence and energy-limited envelope loss) | Envelope survives if absorbed XUV exceeds binding energy | `sse.star.xuv_fraction` (Jackson 2012 saturation and decay), `sse.integrate.log_time` (fixed 200-step), `sse.envelope.binding_energy`, `sse.envelope.energy_limited_loss` (efficiency param, Owen and Wu 2017) | XUV luminosity against time with the cumulative absorbed energy against the binding energy line |
| crates/sse-world/src/lib.rs:574-630 `physics_notes` (Rust glue chaining brine freezing → ice lid heat flow) | Is each brine liquid; heat flow needed to hold the ice lid over it | A `layer.ocean-lid` flow: salinity → `sse.ocean.brine_freezing_point` → base temperature → `sse.heat.ice_shell_heat_flow` (+ `sse.interior.hydrostatic_pressure` for the ocean floor pressure), evaluated per stack pair | The Strata ladder with each shell annotated (freezing point, liquid or frozen, W/m² through the lid), plus a temperature-depth profile through lid and ocean |
| crates/sse-place/src/lib.rs:339-350 (shoreline escape-velocity bound with fixed CMF 0.33) | Escape velocity of the heaviest allowed rocky world | `sse.interior.rocky_radius` → `sse.planet.escape_velocity` (both exist) | Escape velocity against mass with the shoreline cut |
| crates/sse-import-v3/src/lib.rs:501-580 `hydrosphere` | Surface water in Earth oceans | `sse.ocean.surface_water_volume` (coverage × 4πR² × depth) + a normalise-by-Earth block | Globe coverage with an "Earth oceans" gauge |

## Surfaces

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Climate, weather and life

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Galaxy and cosmology

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Light, rendering and sound

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Interface and player views

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

## Candidates to move into flows

Logic written in Rust that could be an editable, cited flow of small blocks (Alex, 19 Nov: "try and do everything in
the physics chain blocks... other than Big Blocks. And have a UI element that makes sense to visualise").

| Where | What it computes | Why it is not a flow yet | Blocks it would need | View that would show it |
|---|---|---|---|---|
