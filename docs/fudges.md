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

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|

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

## Game-system packs

Codes that a game reads from physics. Where a code's band had to be drawn on the physics, the line is a choice,
listed here. Imported codes always win over these (they are kept as authored and marked where physics disagrees).

| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `scripts/build-traveller-pack.py`, rule `traveller.prep.oxygen` | Air "carries oxygen" when O2 is at least 10% of the surface pressure | Traveller's breathable bands are named, not defined by partial pressure | A breathing model per species (O2 partial pressure against altitude and acclimatisation), shared with the habitability atlas | permanent for the code; the habitability scheme (C2) carries the real need |
| `traveller.prep.tainted` | Tainted when CO2 > 0.005 bar, O2 > 0.5 bar, SO2 ≥ 1e-5 bar, or an authored `atmosphere.taint` | The published taints are a story list (pollutants, biologicals, radioactivity...) | Toxicity thresholds per gas from a gas table (V3's tags port in X3d gas mixes) | to review in X3d |
| `traveller.prep.corrosive`, atmosphere C | Corrosive when SO2 ≥ 1 mbar or the main gas is SO2, H2S, Cl2, F2, HCl or H2SO4; insidious is corrosive at 10 bar or more | No physical definition exists for "insidious" | Materials attack rates from composition, pressure and temperature | permanent for the code |
| Atmosphere E and F | Never produced by physics; authored with `atmosphere.traveller_kind` | "Low" and "unusual" describe places and times, not a mean state | Regional and time-varying air from the weather track (W) | stand-in until W |
| `traveller.count.gas-giant` (PBG G digit) | A gas giant is anything 2.5 Earth radii or larger and below 13 Jupiter masses | The games count ice giants and puffy sub-Neptunes alike | The physical atlas's giant classes once the one-body object settles mass and envelope | to review after the one-body unification |
| Trade codes Fr, Co, Ho, Tu, Tr | Mean surface temperature bands (below 223 K, 223 to 263 K, 303 to 353 K) | T5 places these by orbit against the habitable zone, which SSE4 does not keep as a fact | Climate classes from the weather track | to review with W |
| Size F, population F | Size F also holds anything over 23,200 km across; population F anything over 10^15 | The code has no symbol beyond F | None needed: the reading keeps the true value | permanent |
| Missing facts | Absent pressure, liquid cover, population, government, law and tech read as 0, starport as X | A profile always has every digit | None: each reading lists the facts that stood in (`defaulted`) | permanent |
| `crates/sse-codes/src/plan.rs`, `target_temperature`, `optical_depth` | A coded world is placed for 288 K if it has surface liquid (220 K under next to no air, 300 K if dry), through the grey-atmosphere atom with infrared optical depth 0.83 x pressure in bar, at albedo 0.3 | Codes carry no temperature; the orbit must come from somewhere | The climate flows' own greenhouse and albedo for the pinned air, solved for the orbit (C3 reverse placement through base-physics flows) | to review: wire to C3 placement |
| `plan.rs`, `EXOSPHERE_K`, `JEANS_KEEP` | Air retention runs the `sse.atmosphere.jeans_parameter` atom at the surface radius and a fixed 1000 K exosphere, keeping air when it is 30 or more | Exosphere temperature needs the upper-atmosphere energy balance, which no flow computes | Exosphere temperature from stellar XUV and composition; Jeans plus hydrodynamic escape | to review |
| `plan.rs`, `rocky_mass`, `star` | A coded world's mass is M = R^3.7 (Earth-like rock); stars other than class V are grown as dwarfs of the same spectral type, white dwarfs as 0.6 solar-mass dwarfs | Codes give size only; the formation pipeline grows stars from birth and no age is chosen yet to reach a giant or remnant | Composition from the system's formation run; an age solve that evolves the star to its written class | stand-in until the one-body star work |
| `plan.rs`, `giants_in` | A grown world counts as a gas giant from 10 Earth masses, whatever its envelope | The game counts ice giants and big sub-Neptunes alike | As the gas giant reading rule above | to review with it |
| `mix.rs`, `gas_mixes` | A coded world's air is a named preset: mid-range fractions scaled to one, at the pressure of the code's own class (not the preset's range) | No flow yet derives composition from outgassing and escape; the game names air by code | Per-gas partial pressures from an atmosphere flow (outgassing, escape, photochemistry) | to review |
| rules `traveller.need.*` | What to wear outside is read from the atmosphere digit, not from partial pressures and toxicity limits | The game's gear follows its codes | Physiological limits per gas (oxygen 0.16 to 0.5 bar, carbon dioxide and toxic-gas limits) as rules over partial pressures | to review |
