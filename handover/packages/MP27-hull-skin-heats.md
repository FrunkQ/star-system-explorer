# MP27: heat, melt and boil values for hull skins

Track RK asked for these for package RK12, effect at range ("what the target can take", Atomic Rockets plan section 13). The burn-through block needs the energy to heat, melt and boil a skin of density ρ and thickness t:

E = ρ·t·(heat_to_melt + latent_fusion + heat_to_boil + latent_vaporisation)

## What is on the sheet

The fields are per kilogram, on mixture rows. They are built by `scripts/thermal_rows.py`, which is hooked into `build-base-reference.py` after the RK1 part 3 materials.

| Field | Meaning |
|---|---|
| `heat_capacity` | Cp at 298.15 K (J kg-1 K-1, `at_k`) |
| `heat_to_melt` | H(solid at Tm) − H(298.15 K). This is the integral of c dT over the solid, including any solid phase changes. Prefer it over c·ΔT, because Cp roughly doubles by the melting point for refractory metals. |
| `latent_fusion` | The step in H at Tm (`at_k` = Tm). |
| `heat_to_boil` | H(liquid at Tb) − H(liquid at Tm). |
| `boiling_point_k` | The liquid-to-ideal-gas row, where fugacity is 1 bar. |
| `latent_vaporisation` | The step in H at Tb (`at_k` = Tb). |
| `latent_sublimation` | Graphite only: graphite to carbon atoms at 298.15 K. |

**Rows covered:**
- New rows: `aluminium`, `titanium`, `iron`, `graphite`. Each has a density from NIST SRD 126 Table 1.
- Rows updated: `niobium`, `molybdenum`, `tantalum`, `tungsten` (no boiling values yet), `lead`.
- `inconel-718` gets heat_capacity only (Special Metals SMC-045 Table 2: 0.104 Btu/lb °F, printed as 435 J/kg °C).
- `rhenium` gets a density.

**Where the values come from:**
- **NIST-JANAF, 4th ed. (Chase 1998).** The tables used are Al-001, Ti-001, Fe-003, Nb-001, Mo-001, Ta-001, W-001, Pb-001, C-002 (graphite) and C-003 (C(g)). The molar values are typed as printed, in kJ/mol and J/K/mol. The builder converts them per kilogram using the IUPAC atomic weights in the sheet's elements table.
- **Latent heats.** JANAF prints no ΔH column. Each latent heat is the difference between the H − H(298.15) values printed in the two rows at a transition.
- **Melting points.** The Lundberg melting points stay on the refractory rows. JANAF's fusion temperatures sit on `latent_fusion.at_k`, and the builder checks the two agree within 30 K.
- **Checking.** All values were read on 24 Nov 2026 from fetched text, so they are marked unverified.

## Checks

The builder asserts that latent heats are positive, that Tb is above Tm, and that L_v is above L_f. The test is `hull_skins_carry_the_burn_through_heats` (sse-reference). It checks that every skin has all six fields, and that aluminium's L_f is about 397 kJ/kg.

## Gaps

- **Tungsten:** W-001 has no boiling point or liquid-to-gas row.
- **Graphite:**
  - No sublimation temperature is sourced yet. At 1 bar graphite sublimes near 3900 K, but that is not read from a source.
  - `latent_sublimation` is to single atoms. Real carbon vapour is mostly C3, so it is an upper bound.
- **Rhenium:** not in JANAF, so it has no heats.
- **Steels and other alloys:** latent heats were not found. Inconel 718's data sheet gives none.
- **Hull alloys:** Al 2xxx/6xxx/7xxx and Ti-6Al-4V would need their own data sheets. The pure metals stand in for them.
