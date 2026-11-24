# F-redox: how oxidised a young mantle is, and what its magma ocean breathes out (proven core thread, 24 Nov)

The kernel thread asked for this (atmospheres ruling, Alex 23 Nov 22:27: "mantle redox state sets the outgassed mix"). Stage 8 (`sse-formation/src/early.rs`) already outgassed water and carbon. Now the mantle's oxygen fugacity decides which gases they leave as.

## Blocks

`sse.interior.redox_iw` (interior.json). It reads the core mass fraction f, free carbon c, and the molar masses of Fe, FeO and C (from the sheet, on ports).
- Iron the core did not take stays in the mantle as FeO, at (bulk iron − core iron · f). Bulk iron is 0.32 and core iron 0.85 (McDonough 2003). Free carbon smelts FeO back to metal, mole for mole.
- The FeO mole fraction among the silicate's oxides (pyrolite mean molar mass 0.0506 kg/mol, McDonough and Sun 1995) is held at no less than 1e-3. It gives ΔIW = 2 log10(γ x_FeO / x_Fe) at core formation, with γ = 1 and x_Fe = 0.79 (Wade and Wood 2005).
- A deep magma ocean then oxidises itself, as FeO disproportionates under pressure (Armstrong et al. 2019). One shift of 2.7 puts Earth at Sossi et al. 2020's IW+0.5.
- Water's part is already in f, because early.rs lets water oxidise metal before the core forms.
- Outputs are `interior.redox_iw` (the magma ocean's surface, clamped to −8 to +6) and `interior.redox_iw_core`.
- Earth comes out at IW−2.2 at core formation and IW+0.46 at the surface. Mars (f = 0.24) comes out at +1.2, and a Mercury-like core (f = 0.7) at −3.1.

`sse.atmosphere.outgassed_mix` (atmosphere.json). It reads ΔIW, the hydrogen (as H2, mol), the carbon (mol) and the pressure.
- fO2 is the iron-wüstite buffer (Frost 1991, −27489/T + 6.702) times 10^ΔIW.
- H2O/H2 = K1 fO2^½ and CO2/CO = K2 fO2^½. K1 and K2 are fitted as A/T − B to NIST-JANAF from 1000 to 2000 K.
- CH4 comes from CO + 3 H2 = CH4 + H2O, as one step from the methane-free mix, capped by the CO and H2 present.
- T = 2000 K is a parameter.
- Outputs are the mole fractions of H2, H2O, CO, CO2 and CH4. They sum to 1, or are all 0 when nothing was given off.

Both blocks have matching sums in `sse_atoms::redox`, held to them by `redox_blocks_match_the_stage_8_sums`. The atom count goes up by 2, to 114.

## Stage 8

`early_life` works out the free carbon (refractory carbon not taken into the core, over rock and metal), the redox, and the mix.
- The hydrogen is the outgassed water's moles (none on a desiccated world).
- The carbon is the outgassed CO2-equivalent's moles.
- The pressure is the outgassed mass's weight, counted as water and CO2.
- New hand-off facts: `interior.redox_iw`, `early.outgas_h2`, `early.outgas_h2o`, `early.outgas_co`, `early.outgas_co2` and `early.outgas_ch4`.
- `WorldStart` gains `redox_iw` and `outgas`.
- Existing facts are unchanged. `early.co2_partial_pressure` stays the total carbon as CO2, as the kernel thread asked, and `early.surface_water` stays the outgassed water.
- A world built from facts (`sse-surface` `start_of`) gets its redox from its core alone. Its outgas is all zeros, since no formation ledger says what it gave off.

Earth's stage-8 world gives off about 51% H2O, 39% H2, 8% CO, 2% CO2 and 0.04% CH4 by moles at 2000 K, with ΔIW about +0.15. Its free carbon and a slightly larger core put it a little below the block's Earth example. A desiccated Venus gives off only CO and CO2.

## Checks

- `sse-atoms`: `redox_blocks_match_the_stage_8_sums` holds the blocks and sums together. It checks Earth's two values, Mars above Earth, Mercury below, carbon reducing, the fractions summing to 1, and oxidised air going to H2O and CO2 while reduced air goes to H2 and CO.
- `sse-formation`: `the_mantle_sets_what_the_magma_ocean_breathes_out` checks Earth and dry Venus.

## Goldens changed on purpose

- `tests/golden/formation-early.json` and the pipeline golden: every world gains `redox_iw`, `outgas` and the six facts.
- The `base-physics` pack and its fixture gain the two blocks and their kinds.
- Contract stack hashes follow the pack.

## Fudge register

- One self-oxidation shift for every world. Armstrong et al. 2019 say it grows with magma ocean depth.
- One outgassing temperature.
- Methane comes from a single step.
- Pressure counts the gas as water and CO2.
- No Fe3+ chemistry beyond the shift, and no later cooling, escape or photochemistry of the air.
