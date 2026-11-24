# MP28: filling MP27's gaps (boiling, sublimation, steel, alloys, rhenium)

This package follows MP27 and fills in what it left out. It is built in `scripts/thermal_rows.py`, using the same fields that RK12's burn-through reads.

## What was added

- **Tungsten.** Values: `boiling_point_k` 6157 K, `heat_to_boil`, and `latent_vaporisation` 4.58 MJ/kg.
  - The JANAF table W-005 for W(g) stops at 6000 K, where its log Kf is still −0.186. Above 3680 K the reference state is liquid tungsten, so log Kf there is the log of the vapour pressure in bar, and delta-f H is the heat of vaporisation.
  - I fitted log Kf = a + b/T through the 5900 K and 6000 K rows and took Tb where it reaches 0. That is an extrapolation of 157 K past the end of the table, and the note on the row says so.
  - Above 6000 K, the liquid enthalpy uses W-001's constant liquid Cp of 35.564 J/K/mol.
  - Older handbook values put tungsten's boiling point near 5800 to 5900 K, about 300 K lower than this.
- **Graphite.** Values: `sublimation_point_k` 4080 K, `heat_to_sublime`, and `latent_sublimation` 22.8 MJ/kg.
  - The sublimation point is where the total pressure of the C1 to C5 vapour over graphite reaches 1 bar. The tables are C-003, C-113, C-138, C-143 and C-148; I interpolated log p between the 4000 K and 4100 K rows.
  - The vapour at that point is about 74 % C3 by pressure.
  - `latent_sublimation` is the heat per mole of carbon to make that equilibrium vapour mix (274 kJ/mol). It replaces MP27's value for sublimation to single atoms at 298 K (717 kJ/mol), which is kept in the row's note.
  - `heat_to_sublime` is graphite's H − H(298) at Ts, from C-002.
- **`stainless-304l` (new row).** All six burn-through fields, plus solidus 1670 K, liquidus 1730 K and density 7894 kg/m³.
  - Source: Kim 1975, ANL-75-55. The report's heats are in cal/g, and the builder converts them using the thermochemical calorie (4.184 J, exact).
  - The solid enthalpy fit plus the 64.0 cal/g heat of fusion matches the liquid fit at 1700 K, and the builder checks this.
  - The ATI type 304 data sheet is given as a second source for the melting range and density.
- **`ti-6al-4v` (new row).** Melting point 1943 K, and `heat_capacity_hot` 826 J/kg K at 1850 K. Source: Cezairliyan, McClure and Taylor 1977. No room-temperature heats were sourced.
- **`al-6061` (new row).** `heat_capacity` at 300 K, from the top end of NIST's 4 to 300 K cryogenic fit (5 % fit error). No melting or latent heats were sourced, so the pure `aluminium` row stands in for those.
- **Rhenium.** `latent_sublimation` at 298 K from Plante and Szwarc 1966 (185.9 kcal/mol, third law). Rhenium is not in JANAF.

The test `hull_skins_carry_the_burn_through_heats` now also covers tungsten, stainless-304l, and graphite's sublimation point.

## Still missing

- Heats of combustion for the two solid propellants. These searches found none:
  - JPL TM 33-649 (1973), read in full.
  - JPL 79-29 and Thiokol TWR-13009, read in part.
- Ti-6Al-4V and Al 6061 latent heats. NASA's 6061 Materials Data Handbook (Sessler and Weiss) and the TPRC eight-alloy report were found only as tech briefs.
- Rhenium's melting and boiling heats.
- Room-temperature heat capacity for Ti-6Al-4V. The NIST cryogenic page has no Cp fit for it.
