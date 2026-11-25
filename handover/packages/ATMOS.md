# ATMOS: advice on a world's air (`atmosphere_advice`)

Alex's ruling, 23 Nov 2026 22:27: mantle redox sets the outgassed mix; a "the science expects" line beside
hand-set air; a "doesn't go together" note as advice, never a refusal; a "Suggest an atmosphere" button; UV
photochemistry later. Editing design section 11A (Air) and F7. Built 24 Nov 2026 by "Build galaxy arm fix and
atmosphere advice" (kernel queue item 5). Fudge F-ATMOS; citations in `docs/citations/climate-life.md`, section ATMOS.

## The op (`crates/sse-kernel/src/atmos.rs`, protocol `AtmosphereAdvice` / `AtmosphereAdvised`)
Request `{op: "atmosphere_advice", body, at_s?, facts?}`: the body's facts at `at_s`, with `facts` laid over (a
draft mix from the Air box). Read-only; nothing is written or refused. Reply `advice`:
- `redox_iw`, `redox_from`: formation's `interior.redox_iw` (`formation`), else the `sse.interior.redox_iw` block on
  the core mass fraction alone (`core`), else `unknown`.
- `expected`: the `sse.atmosphere.outgassed_mix` block's H2, H2O, CO, CO2, CH4 at that redox, from formation's
  outgassed water and CO2 (`early.surface_water`, `early.co2_partial_pressure`) when there are, else the bulk
  silicate Earth's C/H. `pressure_pa`: the weight of that water and CO2 (formation only; else null).
- `set`, `set_pressure_pa`: the hand-set air (`atmosphere.composition.<gas>`, `atmosphere.surface_pressure` or
  `body.surface_pressure`).
- `differs`: the reduced share of hydrogen (H2 of H2 + H2O) and of carbon (CO of CO + CO2) against the mantle's,
  outside a factor of 10 ("more oxidised than the mantle's gas ... something has oxidised it since"); and each gas
  the rock does not give off (nitrogen is said to be not counted by the block yet).
- `clashes`: E8's available Gibbs energy of the hand-set air (`sse_life::disequilibrium`, the stack's
  `life_chemistry`), credited per reaction: `{pair, sink, j_mol, time_s, why}`. Known lifetimes in Earth's air:
  methane with oxygen 9.1 yr, hydrogen 2 yr, carbon monoxide about two months; others "slowly without life or a
  catalyst". Gases with no formation energy on the stack (argon, neon) are left out, and a note says so.
- `escapes`: hydrogen's lifetime at the diffusion limit (2.5e13 f per cm² per s on Earth, scaled by gravity, over
  the air's column from its own mean molar mass): about 27,000 years for Earth.
- `suggest`: one `set_authored_batch` for "Suggest an atmosphere": the rock's gases, less water where there are
  seas and hydrogen that escapes in under 100 Myr, beside the hand-set gases the block does not cover, dropping any
  of those that clash with the rest (oxygen does); shares sum to one; the pressure when formation gives it (CO2
  alone when the seas hold the water).
- `says` (one line), `notes`, `blocks` (`{block, inputs, results}` for each block run).

## Also
- The kernel keeps the stack's `life_chemistry` (E0) on load (`Observe.chemistry`); absent, the base life pack's.
- V3 import (`crates/sse-import-v3`, `hydrosphere()`): a V3 save's `atmosphere.composition` becomes
  `atmosphere.composition.<gas>` authored facts (lower case; a list summing past 1.5 is read as percent), so the
  advice reads an imported air as hand-set. V3's own atmosphere-reaction rules, if it has any, are **unchecked**:
  V3's source is only on Alex's PC.
- Contract case in the climate area (`tests/contract/areas/climate.json`, golden `tests/golden/areas/climate.jsonl`; Earth-like air on `u:atmos/sys:0/planet:9`). It moved there from the core when the golden split closed `requests.json` (GS2).

## Checks (`crates/sse-kernel/src/atmos_tests.rs`)
- Earth as catalogued: methane with oxygen is the clash (about 9 years); oxygen does not come from the rock; the
  carbon is far more oxidised than the mantle's gas; hydrogen leaves in 3e3 to 3e5 years; the suggestion keeps
  nitrogen and argon, drops oxygen, leaves water to the seas and hydrogen to space, and sums to one; argon is left
  out of the chemistry with a note; the blocks are listed in order.
- A big core (cmf 0.6) leaves a more reduced mantle than a small one (0.1), and its gas has over ten times the
  H2/H2O and CO/CO2.
- A formation run's redox and outgassed water and CO2 win, and give a pressure of 100 to 1,000 bar (as steam).
- Nothing known about the mantle: nothing expected; N2 with CO2 has no clash.

## Not yet (later packages)
- Nitrogen, noble gases and sulphur in the outgassing block; the air through time (escape, weathering, the
  carbonate cycle, photochemistry under the world's own star). "Kept by life" chips (F7) when the life sockets fill.
- A pack override for `AdviceParams` (defaults only today).
- The Air box itself is the editing thread's (ED); it calls this op.
