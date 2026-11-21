## A. Constants of nature, conventions and reference objects

One row per quantity: the canonical value, how many places type it (outside tests / in tests), the distinct values written, and where. A crate's own `consts` module counts as a typed place.

| Quantity | Canonical | Unit | Source | Typed (code / tests) | Distinct values written | Places (code) |
|---|---|---|---|---|---|---|
| gravitational constant (`G`) | 6.6743e-11 | m3 kg-1 s-2 | CODATA 2018 | 12 / 0 | 6.6743e-11 | `crates/sse-codes/src/plan.rs:21`, `crates/sse-core/src/quantity.rs:199`, `crates/sse-cosmos/src/nucleus.rs:45`, `crates/sse-formation/src/disc.rs:33`, `crates/sse-formation/src/early.rs:20`, `crates/sse-formation/src/growth.rs:30` and 6 more |
| speed of light (`c`) | 299792458 | m/s | CODATA 2018 (exact) | 7 / 5 | 299792458 | `crates/sse-atoms/src/nebula.rs:23`, `crates/sse-core/src/quantity.rs:200`, `crates/sse-cosmos/src/nucleus.rs:44`, `crates/sse-life/src/budget.rs:173`, `crates/sse-nav/src/sky.rs:110`, `packages/ui-contract/src/light.ts:25` and 1 more |
| Planck constant (`h`) | 6.62607015e-34 | J s | CODATA 2018 (exact) | 4 / 3 | 6.62607015e-34 | `crates/sse-atoms/src/nebula.rs:23`, `crates/sse-atoms/src/nebula.rs:44`, `crates/sse-life/src/budget.rs:172`, `packages/ui-contract/src/light.ts:24` |
| Boltzmann constant (`k_B`) | 1.380649e-23 | J/K | CODATA 2018 (exact) | 9 / 2 | 1.380649e-23 | `crates/sse-atoms/src/air.rs:17`, `crates/sse-atoms/src/common.rs:10`, `crates/sse-atoms/src/convection.rs:25`, `crates/sse-atoms/src/nebula.rs:44`, `crates/sse-formation/src/disc.rs:35`, `crates/sse-formation/src/growth.rs:31` and 3 more |
| Stefan-Boltzmann constant (`sigma_SB`) | 5.670374419e-08 | W m-2 K-4 | CODATA 2018 (derived) | 6 / 1 | 5.670374e-08, 5.670374419e-08 | `crates/sse-climate/src/conditions.rs:109`, `crates/sse-core/src/quantity.rs:211`, `crates/sse-cosmos/src/nucleus.rs:46`, `crates/sse-formation/src/disc.rs:34`, `crates/sse-life/src/budget.rs:175`, `packages/views/body/src/StarDisc.svelte:21` |
| Avogadro constant (`N_A`) | 6.02214076e+23 | 1/mol | CODATA 2018 (exact) | 0 / 1 | 6.02214076e+23 | tests only |
| molar gas constant (`R_gas`) | 8.314462618 | J mol-1 K-1 | CODATA 2018 (derived) | 10 / 1 | 8.314, 8.314462618 | `crates/sse-atoms/src/chem.rs:11`, `crates/sse-climate/src/conditions.rs:14`, `crates/sse-kernel/src/descent.rs:14`, `crates/sse-nav/src/scenario.rs:104`, `crates/sse-place/src/model.rs:313`, `crates/sse-surface/src/cover.rs:56` and 4 more |
| elementary charge (and the electronvolt in J) (`e`) | 1.602176634e-19 | C | CODATA 2018 (exact) | 2 / 0 | 1.602176634e-19 | `crates/sse-atoms/src/light.rs:26`, `crates/sse-atoms/src/nebula.rs:35` |
| electron mass (`m_e`) | 9.109383702e-31 | kg | CODATA 2018 | 1 / 0 | 9.109383702e-31 | `crates/sse-atoms/src/light.rs:26` |
| proton mass (`m_p`) | 1.672621924e-27 | kg | CODATA 2018 | 1 / 0 | 1.672621924e-27 | `crates/sse-atoms/src/light.rs:28` |
| atomic mass constant (`m_u`) | 1.660539067e-27 | kg | CODATA 2018 | 4 / 0 | 1.660539067e-27 | `crates/sse-atoms/src/air.rs:19`, `crates/sse-atoms/src/common.rs:12`, `crates/sse-atoms/src/convection.rs:26`, `packages/renderer/src/atmosphere.ts:73` |
| Wien displacement constant (`b_wien`) | 0.002897771955 | m K | CODATA 2018 | 1 / 0 | 0.002897771955 | `crates/sse-light/src/observe.rs:15` |
| vacuum permeability (`mu0`) | 1.256637062e-06 | N/A2 | CODATA 2018 | 2 / 0 | 1.256637062e-06 | `crates/sse-atoms/src/light.rs:24`, `crates/sse-atoms/src/magnetic.rs:27` |
| astronomical unit (`au`) | 1.495978707e+11 | m | IAU 2012 B2 (exact) | 48 / 17 | 1.4959787e+11, 1.495978707e+11, 1.49598e+11, 1.496e+11 | `crates/sse-core/src/quantity.rs:201`, `crates/sse-formation/src/disc.rs:37`, `crates/sse-formation/src/early.rs:21`, `crates/sse-formation/src/growth.rs:34`, `crates/sse-formation/src/leftovers.rs:54`, `crates/sse-import-v3/src/reconstruct.rs:16` and 42 more |
| parsec (`pc`) | 3.085677581e+16 | m | IAU 2015 B2 | 0 / 5 | 3.085677581e+16, 3.0857e+16 | tests only |
| Julian year (`yr_julian`) | 31557600 | s | IAU | 44 / 6 | 31557600 | `crates/sse-chronicle/src/event.rs:28`, `crates/sse-climate/src/climate.rs:119`, `crates/sse-cosmos/src/cosmology.rs:19`, `crates/sse-cosmos/src/nucleus.rs:47`, `crates/sse-formation/src/disc.rs:39`, `crates/sse-formation/src/growth.rs:36` and 38 more |
| sidereal year (`yr_sidereal`) | 31558149.8 | s | IERS | 0 / 1 | 31558149 | tests only |
| day (`day`) | 86400 | s | SI | 30 / 21 | 86400 | `crates/sse-climate/src/ebm.rs:245`, `crates/sse-climate/src/ebm.rs:287`, `crates/sse-core/src/quantity.rs:202`, `crates/sse-formation/src/pipeline.rs:30`, `crates/sse-import-v3/src/lib.rs:382`, `crates/sse-import-v3/src/reconstruct.rs:17` and 24 more |
| standard atmosphere (`atm`) | 101325 | Pa | SI (exact) | 5 / 3 | 101325 | `crates/sse-climate/src/conditions.rs:16`, `crates/sse-place/src/model.rs:313`, `crates/sse-surface/src/cover.rs:578`, `packages/shell/src/documents/vocab.guide.json:65`, `packages/shell/src/documents/vocab.plain.json:17` |
| standard gravity (`g0`) | 9.80665 | m/s2 | CGPM 1901 (exact) | 37 / 11 | 9.80665, 9.81 | `crates/sse-climate/src/conditions.rs:16`, `crates/sse-life/src/biosphere.rs:135`, `crates/sse-life/src/scenario.rs:316`, `crates/sse-life/src/scenario.rs:321`, `crates/sse-nav/src/lib.rs:33`, `crates/sse-nav/src/scenario.rs:130` and 31 more |
| 0 degrees Celsius (`T0C`) | 273.15 | K | SI (exact) | 12 / 4 | 273.15 | `crates/sse-atoms/src/layers.rs:37`, `crates/sse-core/src/quantity.rs:271`, `crates/sse-place/src/model.rs:314`, `crates/sse-place/src/model.rs:316`, `crates/sse-place/src/model.rs:323`, `crates/sse-surface/src/climate.rs:68` and 6 more |
| solar mass parameter (`GM_sun`) | 1.3271244e+20 | m3/s2 | IAU 2015 B3 | 3 / 1 | 1.3271244e+20, 1.3271244e+20 | `crates/sse-core/src/quantity.rs:204`, `crates/sse-time/src/epoch.rs:23`, `packages/shell/src/documents/sample-sol.v3.json:84` |
| solar mass (`M_sun`) | 1.9884099e+30 | kg | IAU 2015 B3 GM over CODATA 2018 G | 14 / 12 | 1.98840987e+30, 1.98841e+30, 1.98847e+30, 1.98892e+30, 1.989e+30 | `crates/sse-cosmos/src/nucleus.rs:43`, `crates/sse-cosmos/src/patch.rs:36`, `crates/sse-formation/src/accrete.rs:55`, `crates/sse-formation/src/disc.rs:38`, `crates/sse-formation/src/growth.rs:35`, `crates/sse-formation/src/leftovers.rs:55` and 8 more |
| solar radius (nominal) (`R_sun`) | 695700000 | m | IAU 2015 B3 | 10 / 16 | 695700000 | `crates/sse-core/src/quantity.rs:207`, `crates/sse-time/src/epoch.rs:22`, `crates/sse-time/src/epoch.rs:352`, `packages/shell/src/documents/derivations.base.json:37`, `packages/shell/src/documents/reference-objects.json:26`, `packages/shell/src/documents/sample-sol.v3.json:58` and 4 more |
| solar luminosity (nominal) (`L_sun`) | 3.828e+26 | W | IAU 2015 B3 | 8 / 4 | 3.828e+26 | `crates/sse-core/src/quantity.rs:208`, `crates/sse-formation/src/early.rs:22`, `crates/sse-time/src/epoch.rs:354`, `packages/shell/src/documents/derivations.base.json:29`, `packages/shell/src/documents/vocab.guide.json:26`, `packages/shell/src/documents/vocab.plain.json:11` and 2 more |
| solar effective temperature (nominal) (`T_sun`) | 5772 | K | IAU 2015 B3 | 11 / 29 | 5770, 5772 | `crates/sse-atoms/src/convection.rs:23`, `crates/sse-cosmos/src/stars.rs:274`, `crates/sse-formation/src/lifeline.rs:13`, `crates/sse-kernel/src/adopt.rs:564`, `crates/sse-life/src/scenario.rs:326`, `crates/sse-life/src/scenario.rs:435` and 5 more |
| solar constant at 1 au (nominal) (`S_sun`) | 1361 | W/m2 | IAU 2015 B3 | 6 / 3 | 1361 | `crates/sse-formation/src/early.rs:309`, `crates/sse-nav/src/hazards.rs:281`, `crates/sse-surface/src/climate.rs:58`, `packs/base-light/pack.json:858`, `packs/base-physics/flows/planet-climate.json:38`, `packs/base-physics/pack.json:674` |
| Earth mass parameter (`GM_earth`) | 3.986004e+14 | m3/s2 | IAU 2015 B3 | 3 / 0 | 3.986004e+14, 3.986004418e+14 | `crates/sse-core/src/quantity.rs:205`, `crates/sse-time/src/epoch.rs:23`, `packages/shell/src/documents/sample-sol.v3.json:150` |
| Earth mass (`M_earth`) | 5.9722e+24 | kg | IAU 2015 B3 (GM/G) | 45 / 8 | 5.97e+24, 5.972e+24, 5.9722e+24 | `crates/sse-codes/src/plan.rs:22`, `crates/sse-formation/src/early.rs:17`, `crates/sse-formation/src/violent.rs:53`, `crates/sse-import-v3/src/reconstruct.rs:15`, `crates/sse-kernel/src/adopt.rs:37`, `crates/sse-nav/src/lib.rs:32` and 39 more |
| Earth equatorial radius (`R_earth_eq`) | 6378100 | m | IAU 2015 B3 | 2 / 1 | 6378100, 6378137 | `crates/sse-core/src/quantity.rs:209`, `packs/base-light/pack.json:870` |
| Earth mean radius (`R_earth_mean`) | 6371000 | m | IUGG (mean radius) | 48 / 23 | 6371000 | `crates/sse-atoms/src/planet.rs:13`, `crates/sse-codes/src/plan.rs:23`, `crates/sse-formation/src/early.rs:18`, `crates/sse-formation/src/leftovers.rs:53`, `crates/sse-formation/src/violent.rs:52`, `crates/sse-nav/src/scenario.rs:129` and 42 more |
| Jupiter mass parameter (`GM_jup`) | 1.2668653e+17 | m3/s2 | IAU 2015 B3 | 1 / 0 | 1.2668653e+17 | `crates/sse-core/src/quantity.rs:206` |
| Jupiter mass (`M_jup`) | 1.89813e+27 | kg | IAU 2015 B3 (GM/G) | 6 / 1 | 1.8979016e+27, 1.89819e+27 | `packs/base-atlas/pack.json:2079`, `packs/base-atlas/pack.json:2161`, `packs/base-atlas/pack.json:2313`, `packs/base-atlas/pack.json:2876`, `packs/base-atlas/pack.json:7556`, `packs/base-atlas/pack.json:9092` |
| Jupiter equatorial radius (`R_jup_eq`) | 71492000 | m | IAU 2015 B3 | 2 / 2 | 71492000 | `crates/sse-core/src/quantity.rs:210`, `packs/base-light/pack.json:911` |
| Jupiter mean radius (`R_jup_mean`) | 69911000 | m | NSSDCA | 2 / 5 | 69911000 | `packages/shell/src/documents/reference-objects.json:22`, `packages/ui-contract/src/compare.ts:12` |
| Moon mass (`M_moon`) | 7.346e+22 | kg | NSSDCA Moon fact sheet (DE440 GM over G gives 7.3458e22) | 9 / 2 | 7.34556e+22, 7.346e+22 | `packages/shell/src/documents/sample-sol.v3.json:145`, `packs/base-atlas/pack.json:1071`, `packs/base-atlas/pack.json:3483`, `packs/base-atlas/pack.json:4889`, `packs/base-atlas/pack.json:6807`, `packs/base-atlas/pack.json:8077` and 3 more |
| Moon mean radius (`R_moon`) | 1737400 | m | NSSDCA | 7 / 9 | 1737400 | `crates/sse-nav/src/scenario.rs:156`, `packages/shell/src/documents/reference-objects.json:14`, `packages/shell/src/documents/sample-sol.v3.json:146`, `packs/base-physics/flows/planet-bulk.json:49`, `packs/base-physics/flows/planet-heat.json:70`, `packs/base-physics/pack.json:576` and 1 more |
| Moon semi-major axis (`a_moon`) | 384400000 | m | NSSDCA | 6 / 2 | 384400000, 384466527.7 | `packs/base-atlas/pack.json:1075`, `packs/base-atlas/pack.json:3487`, `packs/base-atlas/pack.json:4893`, `packs/base-atlas/pack.json:6811`, `packs/base-atlas/pack.json:8081`, `packs/base-atlas/pack.json:9802` |
| Sun-to-Earth mass ratio (`Msun_Mearth`) | 332946.0487 | 1 | IAU 2015 B3 GM ratio | 5 / 1 | 332775.64, 332946 | `crates/sse-formation/src/accrete.rs:54`, `crates/sse-formation/src/early.rs:19`, `crates/sse-formation/src/growth.rs:37`, `crates/sse-formation/src/pins.rs:218`, `packages/ui-contract/src/sound.ts:199` |
| Sun-to-Jupiter mass ratio (`Msun_Mjup`) | 1047.348644 | 1 | IAU 2015 B3 GM ratio | 1 / 0 | 1048 | `crates/sse-cosmos/src/epochs.rs:75` |
| astronomical unit in solar radii (`au_Rsun`) | 215.032 | 1 | IAU au over nominal R_sun | 3 / 0 | 215, 215.032 | `crates/sse-formation/src/history.rs:218`, `packages/ui-contract/src/compare.ts:104`, `packages/ui-contract/src/compare.ts:106` |
| Chandrasekhar mass (`M_ch`) | 1.44 | M_sun | textbook (mu_e = 2) | 1 / 1 | 1.44 | `crates/sse-formation/src/lifeline.rs:15` |
| water triple point (`T3_h2o`) | 273.16 | K | NIST WebBook | 1 / 0 | 273.16 | `packs/base-climate/pack.json:164` |
| water triple-point pressure (`P3_h2o`) | 611.657 | Pa | NIST WebBook | 5 / 0 | 611, 611.657, 612 | `crates/sse-place/src/model.rs:314`, `crates/sse-surface/src/cover.rs:1022`, `packages/shell/src/documents/kestrel.scene.json:116`, `packs/base-climate/pack.json:165`, `packs/base-life/pack.json:314` |
| water molar mass (`M_h2o`) | 0.0180153 | kg/mol | IUPAC 2021 | 2 / 0 | 0.018015, 0.01801528 | `packs/base-climate/pack.json:167`, `packs/base-light/pack.json:574` |
| carbon dioxide molar mass (`M_co2`) | 0.0440095 | kg/mol | IUPAC 2021 | 2 / 0 | 0.0440095, 0.04401 | `packs/base-climate/pack.json:182`, `packs/base-light/pack.json:526` |
| nitrogen molar mass (`M_n2`) | 0.0280134 | kg/mol | IUPAC 2021 | 2 / 0 | 0.0280134, 0.028014 | `packs/base-climate/pack.json:227`, `packs/base-light/pack.json:490` |
| carbon monoxide molar mass (`M_co`) | 0.0280101 | kg/mol | IUPAC 2021 | 1 / 0 | 0.0280101 | `packs/base-light/pack.json:610` |
| oxygen molar mass (`M_o2`) | 0.0319988 | kg/mol | IUPAC 2021 | 1 / 0 | 0.0319988 | `packs/base-light/pack.json:502` |
| methane molar mass (`M_ch4`) | 0.0160425 | kg/mol | IUPAC 2021 | 2 / 0 | 0.0160425, 0.016043 | `packs/base-climate/pack.json:197`, `packs/base-light/pack.json:562` |
| ammonia molar mass (`M_nh3`) | 0.0170305 | kg/mol | IUPAC 2021 | 2 / 0 | 0.0170305, 0.017031 | `packs/base-climate/pack.json:212`, `packs/base-light/pack.json:586` |
| argon molar mass (`M_ar`) | 0.039948 | kg/mol | IUPAC 2021 | 1 / 0 | 0.039948 | `packs/base-light/pack.json:514` |
| dry air molar mass (`M_air`) | 0.0289644 | kg/mol | US Standard Atmosphere 1976 | 2 / 4 | 0.0289595, 0.02896, 0.02897 | `packs/base-light/pack.json:478`, `packs/base-weather/pack.json:503` |
| carbon dioxide triple point (`T3_co2`) | 216.58 | K | NIST WebBook | 1 / 0 | 216.58 | `packs/base-climate/pack.json:179` |
| methane triple point (`T3_ch4`) | 90.694 | K | NIST WebBook | 1 / 0 | 90.67 | `packs/base-climate/pack.json:194` |
| nitrogen triple point (`T3_n2`) | 63.151 | K | NIST WebBook | 1 / 0 | 63.14 | `packs/base-climate/pack.json:224` |
| dry air heat capacity (`cp_air`) | 1004 | J kg-1 K-1 | textbook | 1 / 0 | 1004 | `packs/base-surface/pack.json:942` |

## B. Disagreements (one quantity, more than one value)

A value is **rounded** when it is the canonical value at the precision it is written with, and **differs** when it is not (a different definition, an older measurement or a slip).

| Quantity | Canonical | Values written (in the catalogue unit) | Where each value is |
|---|---|---|---|
| Stefan-Boltzmann constant (`sigma_SB`) | 5.670374419e-08 | 5.670374e-08, 5.670374419e-08 | 5.670374e-08 (rounded): `crates/sse-activity/src/tests.rs:242`; 5.670374419e-08 (rounded): `crates/sse-climate/src/conditions.rs:109`, `crates/sse-core/src/quantity.rs:211`, `crates/sse-cosmos/src/nucleus.rs:46`, `crates/sse-formation/src/disc.rs:34` +2 |
| molar gas constant (`R_gas`) | 8.314462618 | 8.314, 8.314462618 | 8.314 (rounded): `crates/sse-place/src/model.rs:313`, `crates/sse-surface/src/regimes.rs:743`; 8.314462618 (rounded): `crates/sse-atoms/src/chem.rs:11`, `crates/sse-atoms/src/tests.rs:728`, `crates/sse-climate/src/conditions.rs:14`, `crates/sse-kernel/src/descent.rs:14` +5 |
| astronomical unit (`au`) | 1.495978707e+11 | 1.4959787e+11, 1.495978707e+11, 1.49598e+11, 1.496e+11 | 1.4959787e+11 (rounded): `packs/traveller/pack.json:8610`, `packs/traveller/pack.json:8992`, `packs/traveller/pack.json:9017`, `packs/traveller/pack.json:9043`; 1.495978707e+11 (rounded): `crates/sse-core/src/quantity.rs:201`, `crates/sse-core/src/quantity.rs:325`, `crates/sse-formation/src/disc.rs:37`, `crates/sse-formation/src/early.rs:21` +51; 1.49598e+11 (rounded): `crates/sse-atoms/src/tests.rs:216`, `crates/sse-atoms/src/tests.rs:217`, `packs/base-physics/flows/orbit-basics.json:42`, `packs/base-physics/pack.json:360`; 1.496e+11 (rounded): `crates/sse-kernel/src/tests.rs:2509`, `crates/sse-light/src/tests.rs:394` |
| parsec (`pc`) | 3.085677581e+16 | 3.085677581e+16, 3.0857e+16 | 3.085677581e+16 (rounded): `crates/sse-atoms/src/tests.rs:631`, `crates/sse-light/src/tests.rs:609`, `crates/sse-sense/src/tests.rs:5`, `crates/sse-survey/src/tests.rs:4`; 3.0857e+16 (rounded): `crates/sse-atoms/src/tests.rs:619` |
| standard gravity (`g0`) | 9.80665 | 9.80665, 9.81 | 9.80665 (rounded): `crates/sse-atoms/src/tests.rs:363`, `crates/sse-atoms/src/tests.rs:368`, `crates/sse-atoms/src/tests.rs:696`, `crates/sse-climate/src/conditions.rs:16` +30; 9.81 (rounded): `crates/sse-atoms/src/tests.rs:447`, `crates/sse-life/src/biosphere.rs:135`, `crates/sse-life/src/scenario.rs:316`, `crates/sse-life/src/scenario.rs:321` +10 |
| solar mass (`M_sun`) | 1.9884099e+30 | 1.98840987e+30, 1.98841e+30, 1.98847e+30, 1.98892e+30, 1.989e+30 | 1.98840987e+30 (differs): `crates/sse-core/src/quantity.rs:328`; 1.98841e+30 (rounded): `crates/sse-atoms/src/tests.rs:632`, `crates/sse-cosmos/src/patch.rs:36`, `crates/sse-light/src/tests.rs:23`, `crates/sse-light/src/tests.rs:580` +2; 1.98847e+30 (differs): `crates/sse-cosmos/src/nucleus.rs:43`, `crates/sse-cosmos/src/nucleus_tests.rs:115`, `crates/sse-formation/src/disc.rs:38`, `crates/sse-formation/src/growth.rs:35` +12; 1.98892e+30 (differs): `crates/sse-surface/src/lib.rs:45`; 1.989e+30 (rounded): `crates/sse-formation/src/accrete.rs:55`, `crates/sse-kernel/src/tests.rs:3010` |
| solar effective temperature (nominal) (`T_sun`) | 5772 | 5770, 5772 | 5770 (rounded): `packages/ui-contract/test/light.test.ts:104`; 5772 (rounded): `crates/sse-activity/src/tests.rs:13`, `crates/sse-atoms/src/convection.rs:23`, `crates/sse-atoms/src/tests.rs:173`, `crates/sse-cosmos/src/stars.rs:274` +31 |
| Earth mass parameter (`GM_earth`) | 3.986004e+14 | 3.986004e+14, 3.986004418e+14 | 3.986004e+14 (rounded): `crates/sse-core/src/quantity.rs:205`, `crates/sse-time/src/epoch.rs:23`; 3.986004418e+14 (differs): `packages/shell/src/documents/sample-sol.v3.json:150` |
| Earth mass (`M_earth`) | 5.9722e+24 | 5.97e+24, 5.972e+24, 5.9722e+24 | 5.97e+24 (rounded): `crates/sse-core/src/cbor.rs:641`, `crates/sse-kernel/src/tests.rs:3007`; 5.972e+24 (rounded): `crates/sse-kernel/src/orbits.rs:454`, `crates/sse-kernel/src/orbits.rs:506`, `crates/sse-surface/src/impacts.rs:38`, `packs/base-atlas/pack.json:1017` +24; 5.9722e+24 (rounded): `crates/sse-atoms/src/tests.rs:287`, `crates/sse-codes/src/plan.rs:22`, `crates/sse-formation/src/early.rs:17`, `crates/sse-formation/src/violent.rs:53` +19 |
| Earth equatorial radius (`R_earth_eq`) | 6378100 | 6378100, 6378137 | 6378100 (rounded): `crates/sse-core/src/quantity.rs:209`; 6378137 (differs): `crates/sse-light/src/tests.rs:355`, `packs/base-light/pack.json:870` |
| Jupiter mass (`M_jup`) | 1.89813e+27 | 1.8979016e+27, 1.89819e+27 | 1.8979016e+27 (differs): `packs/base-atlas/pack.json:2079`, `packs/base-atlas/pack.json:2161`, `packs/base-atlas/pack.json:2313`, `packs/base-atlas/pack.json:2876` +2; 1.89819e+27 (differs): `crates/sse-survey/src/tests.rs:7` |
| Moon mass (`M_moon`) | 7.346e+22 | 7.34556e+22, 7.346e+22 | 7.34556e+22 (differs): `packs/base-atlas/pack.json:1071`, `packs/base-atlas/pack.json:3483`, `packs/base-atlas/pack.json:4889`, `packs/base-atlas/pack.json:6807` +2; 7.346e+22 (rounded): `crates/sse-atoms/src/tests.rs:289`, `crates/sse-atoms/src/tests.rs:402`, `packages/shell/src/documents/sample-sol.v3.json:145`, `packs/base-physics/flows/planet-bulk.json:49` +1 |
| Moon semi-major axis (`a_moon`) | 384400000 | 384400000, 384466527.7 | 384400000 (rounded): `crates/sse-atoms/src/tests.rs:401`, `packages/ui-contract/test/compare.test.ts:99`; 384466527.7 (differs): `packs/base-atlas/pack.json:1075`, `packs/base-atlas/pack.json:3487`, `packs/base-atlas/pack.json:4893`, `packs/base-atlas/pack.json:6811` +2 |
| Sun-to-Earth mass ratio (`Msun_Mearth`) | 332946.0487 | 332775.64, 332946 | 332775.64 (differs): `crates/sse-formation/src/accrete.rs:54`; 332946 (rounded): `crates/sse-formation/src/early.rs:19`, `crates/sse-formation/src/growth.rs:37`, `crates/sse-formation/src/pins.rs:218`, `crates/sse-formation/src/violent_tests.rs:180` +1 |
| astronomical unit in solar radii (`au_Rsun`) | 215.032 | 215, 215.032 | 215 (rounded): `packages/ui-contract/src/compare.ts:104`, `packages/ui-contract/src/compare.ts:106`; 215.032 (rounded): `crates/sse-formation/src/history.rs:218` |
| water triple-point pressure (`P3_h2o`) | 611.657 | 611, 611.657, 612 | 611 (rounded): `packs/base-climate/pack.json:165`; 611.657 (rounded): `crates/sse-place/src/model.rs:314`, `crates/sse-surface/src/cover.rs:1022`, `packs/base-life/pack.json:314`; 612 (rounded): `packages/shell/src/documents/kestrel.scene.json:116` |
| water molar mass (`M_h2o`) | 0.0180153 | 0.018015, 0.01801528 | 0.018015 (rounded): `packs/base-climate/pack.json:167`; 0.01801528 (differs): `packs/base-light/pack.json:574` |
| carbon dioxide molar mass (`M_co2`) | 0.0440095 | 0.0440095, 0.04401 | 0.0440095 (rounded): `packs/base-light/pack.json:526`; 0.04401 (rounded): `packs/base-climate/pack.json:182` |
| nitrogen molar mass (`M_n2`) | 0.0280134 | 0.0280134, 0.028014 | 0.0280134 (rounded): `packs/base-light/pack.json:490`; 0.028014 (rounded): `packs/base-climate/pack.json:227` |
| methane molar mass (`M_ch4`) | 0.0160425 | 0.0160425, 0.016043 | 0.0160425 (rounded): `packs/base-light/pack.json:562`; 0.016043 (rounded): `packs/base-climate/pack.json:197` |
| ammonia molar mass (`M_nh3`) | 0.0170305 | 0.0170305, 0.017031 | 0.0170305 (rounded): `packs/base-light/pack.json:586`; 0.017031 (rounded): `packs/base-climate/pack.json:212` |
| dry air molar mass (`M_air`) | 0.0289644 | 0.0289595, 0.02896, 0.02897 | 0.0289595 (differs): `crates/sse-atoms/src/tests.rs:693`, `crates/sse-atoms/src/tests.rs:696`, `packs/base-light/pack.json:478`; 0.02896 (rounded): `packages/renderer/test/atmosphere.test.ts:42`; 0.02897 (rounded): `crates/sse-atoms/src/tests.rs:363`, `packs/base-weather/pack.json:503` |

## C. Matter properties in packs

Every pack number under a key that names a matter property (molar mass, density, phase points, latent heat, optics, half-life, thermochemistry).

| Pack file | JSON path | Value |
|---|---|---|
| `packs/base-atlas/pack.json` | `/provides/other/classes/small-body/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/small-body/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/iron-rich/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/iron-rich/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/Kepler-10 b/values/body.bulk_density` | 5658.904406 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/55 Cancri e/values/body.bulk_density` | 6638.702407 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/TRAPPIST-1 b/values/body.bulk_density` | 5376.916112 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/rocky/examples/TRAPPIST-1 b/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Ganymede/values/body.bulk_density` | 1940 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Ganymede/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/water-rich/examples/Kepler-138 d/values/body.bulk_density` | 3363.219517 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/sub-neptune/examples/GJ 1214 b/values/body.bulk_density` | 2189.964777 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/sub-neptune/examples/K2-18 b/values/body.bulk_density` | 2676.430901 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/sub-neptune/examples/K2-18 b/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/ice-giant/examples/Uranus/values/body.bulk_density` | 1270 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/ice-giant/examples/Uranus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/ice-giant/examples/Neptune/values/body.bulk_density` | 1640 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/ice-giant/examples/Neptune/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/ice-giant/examples/GJ 436 b/values/body.bulk_density` | 1680.547803 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ammonia-clouds/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ammonia-clouds/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ammonia-clouds/examples/Saturn/values/body.bulk_density` | 687 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ammonia-clouds/examples/Saturn/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/water-clouds/examples/Kepler-16 b/values/body.bulk_density` | 966.6989304 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/water-clouds/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/water-clouds/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/cloudless/examples/WASP-107 b/values/body.bulk_density` | 143.7746877 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/alkali-clouds/examples/HD 189733 b/values/body.bulk_density` | 974.5500659 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/silicate-clouds/examples/HD 209458 b/values/body.bulk_density` | 326.7178584 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ultra-hot/examples/WASP-12 b/values/body.bulk_density` | 266.6966583 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/gas-giant/ultra-hot/examples/KELT-9 b/values/body.bulk_density` | 530.8454722 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/brown-dwarf/examples/Gliese 229 B/values/body.bulk_density` | 87108.20854 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tidally-locked/examples/TRAPPIST-1 b/values/body.bulk_density` | 5376.916112 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tidally-locked/examples/TRAPPIST-1 b/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tidally-locked/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tidally-locked/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/eccentric/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/eccentric/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/eccentric/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/eccentric/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Saturn/values/body.bulk_density` | 687 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Saturn/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Uranus/values/body.bulk_density` | 1270 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Uranus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Neptune/values/body.bulk_density` | 1640 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ringed/examples/Neptune/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/volcanic/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/volcanic/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/volcanic/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/volcanic/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/tide-heated/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/unshielded/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/young/examples/illustrative: Earth at 50 million years/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/young/examples/illustrative: Earth at 50 million years/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ancient/examples/illustrative: an Earth-like world 11 billion years old/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/ancient/examples/illustrative: an Earth-like world 11 billion years old/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/captured/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/captured/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/living/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/living/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/inhabited/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/inhabited/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/terraformed/examples/illustrative: a terraformed Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/terraformed/examples/illustrative: a terraformed Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/puffed/examples/HD 209458 b/values/body.bulk_density` | 326.7178584 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/puffed/examples/WASP-12 b/values/body.bulk_density` | 266.6966583 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/physical/puffed/examples/WASP-107 b/values/body.bulk_density` | 143.7746877 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/TRAPPIST-1 b/values/body.bulk_density` | 5376.916112 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/airless/examples/TRAPPIST-1 b/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/molten/examples/Kepler-10 b/values/body.bulk_density` | 5658.904406 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/molten/examples/55 Cancri e/values/body.bulk_density` | 6638.702407 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hot-greenhouse/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hot-greenhouse/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hot-desert/examples/illustrative: a dry Earth at 0.7 au/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hot-desert/examples/illustrative: a dry Earth at 0.7 au/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-desert/examples/illustrative: Earth with its oceans gone/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-desert/examples/illustrative: Earth with its oceans gone/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-ocean/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-ocean/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-ocean/examples/illustrative: an ocean world/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/temperate-ocean/examples/illustrative: an ocean world/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ocean-covered/examples/illustrative: an ocean world/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ocean-covered/examples/illustrative: an ocean world/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/cold-desert/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/cold-desert/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-open-equator/examples/illustrative: a slushball Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-open-equator/examples/illustrative: a slushball Earth/values/surface.ice_fraction` | 0.8 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/frozen/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/frozen/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/frozen/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/frozen/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Ganymede/values/body.bulk_density` | 1940 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Ganymede/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/ice-shell-ocean/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hydrocarbon-lakes/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hydrocarbon-lakes/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hycean/examples/K2-18 b/values/body.bulk_density` | 2676.430901 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/surface/hycean/examples/K2-18 b/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Ganymede/values/body.bulk_density` | 1940 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Ganymede/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/none/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen-oxygen/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen-oxygen/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/nitrogen/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/carbon-dioxide/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/carbon-dioxide/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/carbon-dioxide/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/carbon-dioxide/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Saturn/values/body.bulk_density` | 687 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Saturn/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Uranus/values/body.bulk_density` | 1270 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Uranus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Neptune/values/body.bulk_density` | 1640 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/hydrogen-helium/examples/Neptune/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/sulphurous/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/sulphurous/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/steam/examples/illustrative: a steam world/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/steam/examples/illustrative: a steam world/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/methane/examples/illustrative: a methane-sky moon/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/methane/examples/illustrative: a methane-sky moon/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Ganymede/values/body.bulk_density` | 1940 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Ganymede/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/trace/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/very-thin/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/very-thin/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/thin/examples/illustrative: a high-plateau world with thin oxygen/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/thin/examples/illustrative: a high-plateau world with thin oxygen/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/standard/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/standard/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/standard/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/standard/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/dense/examples/illustrative: Earth at 2 bar/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/dense/examples/illustrative: Earth at 2 bar/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/very-dense/examples/illustrative: Earth at 5 bar/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/very-dense/examples/illustrative: Earth at 5 bar/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Jupiter/values/body.bulk_density` | 1330 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Jupiter/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Saturn/values/body.bulk_density` | 687 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Saturn/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Uranus/values/body.bulk_density` | 1270 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Uranus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Neptune/values/body.bulk_density` | 1640 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/crushing/examples/Neptune/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/tainted/examples/illustrative: Earth with 2% carbon dioxide/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/air/tainted/examples/illustrative: Earth with 2% carbon dioxide/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Venus/values/body.bulk_density` | 5240 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Venus/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Io/values/body.bulk_density` | 3530 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Io/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Europa/values/body.bulk_density` | 3010 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Europa/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Ganymede/values/body.bulk_density` | 1940 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Ganymede/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Enceladus/values/body.bulk_density` | 1610 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/sealed-habitat/examples/Enceladus/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Mercury/values/body.bulk_density` | 5430 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Mercury/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Moon/values/body.bulk_density` | 3340 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Moon/values/surface.ice_fraction` | 0 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Mars/values/body.bulk_density` | 3930 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Mars/values/surface.ice_fraction` | 0.02 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Ceres/values/surface.ice_fraction` | 0.1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Callisto/values/body.bulk_density` | 1830 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Callisto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Titan/values/body.bulk_density` | 1880 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Titan/values/surface.ice_fraction` | 0.98 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Triton/values/body.bulk_density` | 2060 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Triton/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Pluto/values/body.bulk_density` | 1850 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/pressure-suit/examples/Pluto/values/surface.ice_fraction` | 1 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/breathing-apparatus/examples/illustrative: a high-plateau world with thin oxygen/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/breathing-apparatus/examples/illustrative: a high-plateau world with thin oxygen/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/filter-mask/examples/illustrative: Earth with 2% carbon dioxide/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/filter-mask/examples/illustrative: Earth with 2% carbon dioxide/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/shirt-sleeve/examples/Earth/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/shirt-sleeve/examples/Earth/values/surface.ice_fraction` | 0.03 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/shirt-sleeve/examples/illustrative: Earth with 2% carbon dioxide/values/body.bulk_density` | 5510 |
| `packs/base-atlas/pack.json` | `/provides/other/classes/habitable/shirt-sleeve/examples/illustrative: Earth with 2% carbon dioxide/values/surface.ice_fraction` | 0.03 |
| `packs/base-climate/pack.json` | `/provides/other/climate/circulation/delta_h` | 0.3333 |
| `packs/base-climate/pack.json` | `/provides/other/climate/cloud/latent_scale_w_m2` | 25 |
| `packs/base-climate/pack.json` | `/provides/other/climate/ice/t_ice_k` | 263.15 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/water/condensed_kg_m3` | 1000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/water/latent_j_kg` | 2468000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/water/molar_kg` | 0.018015 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/water/triple_k` | 273.16 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/water/triple_pa` | 611 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/carbon dioxide/condensed_kg_m3` | 1560 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/carbon dioxide/latent_j_kg` | 573000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/carbon dioxide/molar_kg` | 0.04401 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/carbon dioxide/triple_k` | 216.58 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/carbon dioxide/triple_pa` | 518500 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/methane/condensed_kg_m3` | 450 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/methane/latent_j_kg` | 531000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/methane/molar_kg` | 0.016043 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/methane/triple_k` | 90.67 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/methane/triple_pa` | 11690 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/ammonia/condensed_kg_m3` | 820 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/ammonia/latent_j_kg` | 1832000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/ammonia/molar_kg` | 0.017031 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/ammonia/triple_k` | 194.95 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/ammonia/triple_pa` | 6060 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/nitrogen/condensed_kg_m3` | 800 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/nitrogen/latent_j_kg` | 200000 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/nitrogen/molar_kg` | 0.028014 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/nitrogen/triple_k` | 63.14 |
| `packs/base-climate/pack.json` | `/provides/other/climate/substances/nitrogen/triple_pa` | 12520 |
| `packs/base-climate/pack.json` | `/provides/other/climate/supply/latent_share` | 0.5 |
| `packs/base-cosmology/pack.json` | `/provides/other/cosmology/rings/k40_half_life_gyr` | 1.248 |
| `packs/base-cosmology/pack.json` | `/provides/other/cosmology/stars/midplane_density` | 0.1 |
| `packs/base-cosmology/pack.json` | `/provides/other/cosmology/stars/triple_share` | 0.25 |
| `packs/base-cosmos/pack.json` | `/provides/other/patch_model/populations/thin/density_pc3` | 0.0712 |
| `packs/base-cosmos/pack.json` | `/provides/other/patch_model/populations/thick/density_pc3` | 0.00855 |
| `packs/base-cosmos/pack.json` | `/provides/other/patch_model/populations/halo/density_pc3` | 0.000356 |
| `packs/base-life/pack.json` | `/provides/other/life_feedback/decay_methane_anoxic` | 0.5 |
| `packs/base-life/pack.json` | `/provides/other/life_feedback/decay_methane_oxic` | 0.0025 |
| `packs/base-life/pack.json` | `/provides/other/life_signatures/decay_methane_anoxic` | 0.5 |
| `packs/base-life/pack.json` | `/provides/other/life_signatures/decay_methane_oxic` | 0.005 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/0/strength` | 0.05, 0.15, 0.3, 0.6, 1, 0.6, 0.1 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/1/strength` | 0.05, 0.15, 0.3, 0.6, 1, 0.6, 0.1 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/2/strength` | 0.05, 0.15, 0.3, 0.6, 1, 0.6, 0.1 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/3/strength` | 0, 0, 0, 0, 0, 0.1, 1 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/4/strength` | 0, 0, 0, 0, 0.1, 1, 0.5 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/5/strength` | 0.8, 1, 1, 0.7, 0.3, 0.05, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/6/strength` | 0.8, 1, 1, 0.7, 0.3, 0.05, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/7/strength` | 0.5, 0.9, 0.8, 0.3, 0, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/8/strength` | 0.8, 0.9, 0.7, 0.4, 0.1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/9/strength` | 0.7, 0.9, 0.7, 0.3, 0.05, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/10/strength` | 1, 0.8, 0.5, 0.2, 0.05, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/absorption/lines/11/strength` | 1, 0.2, 0, 0, 0, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/air/king` | 1.0469, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/air/molar_mass_g` | 28.9595 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/air/refractivity_589` | 0.0002926 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/n2/king` | 1.034, 0.000317, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/n2/molar_mass_g` | 28.0134 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/n2/refractivity_589` | 0.000298 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/o2/king` | 1.096, 0.001385, 0.0001448 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/o2/molar_mass_g` | 31.9988 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/o2/refractivity_589` | 0.000271 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ar/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ar/molar_mass_g` | 39.948 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ar/refractivity_589` | 0.000281 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co2/king` | 1.15, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co2/molar_mass_g` | 44.0095 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co2/refractivity_589` | 0.000449 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2/molar_mass_g` | 2.01588 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2/refractivity_589` | 0.000132 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/he/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/he/molar_mass_g` | 4.002602 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/he/refractivity_589` | 3.5e-05 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ch4/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ch4/molar_mass_g` | 16.0425 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ch4/refractivity_589` | 0.000444 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2o/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2o/molar_mass_g` | 18.01528 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/h2o/refractivity_589` | 0.000256 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/nh3/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/nh3/molar_mass_g` | 17.0305 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/nh3/refractivity_589` | 0.000376 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/so2/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/so2/molar_mass_g` | 64.066 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/so2/refractivity_589` | 0.000686 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co/molar_mass_g` | 28.0101 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/co/refractivity_589` | 0.000338 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ne/king` | 1, 0, 0 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ne/molar_mass_g` | 20.1797 |
| `packs/base-light/pack.json` | `/provides/other/light/gases/list/ne/refractivity_589` | 6.7e-05 |
| `packs/base-nav/pack.json` | `/provides/other/nav/descent/top_density` | 1e-09 |
| `packs/base-nav/pack.json` | `/provides/other/nav/parking_radii` | 1.1 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/H2-He/cp_j_kg_k` | 12000 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/H2-He/molar_kg` | 0.0023 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/CO2/cp_j_kg_k` | 1100 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/CO2/molar_kg` | 0.044 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/N2-CH4/cp_j_kg_k` | 1040 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/N2-CH4/molar_kg` | 0.028 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/N2-O2/cp_j_kg_k` | 1004 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/N2-O2/molar_kg` | 0.029 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/CO2 (thin)/cp_j_kg_k` | 750 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/gases/CO2 (thin)/molar_kg` | 0.044 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/ice_albedo` | 0.6 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/ice_dry_k` | 10 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/land_ice_k` | 266 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/sea_ice_k` | 271 |
| `packs/base-surface/pack.json` | `/provides/other/surface/cover/shadow_ice_max_bar` | 0.001 |
| `packs/base-surface/pack.json` | `/provides/other/surface/features/ice_min_km2` | 100000 |
| `packs/base-surface/pack.json` | `/provides/other/surface/impacts/impactor_density` | 2500 |
| `packs/base-surface/pack.json` | `/provides/other/surface/regime_features/clouds/ice_giant_mass_earth` | 50 |
| `packs/base-surface/pack.json` | `/provides/other/surface/regime_features/clouds/mean_molar_kg` | 0.0023 |
| `packs/base-surface/pack.json` | `/provides/other/surface/regime_features/clouds/wind_ice_m_s` | 400 |
| `packs/base-surface/pack.json` | `/provides/other/surface/wear/ice_k` | 263 |
| `packs/base-weather/pack.json` | `/provides/other/weather_aerosols/earth/density_kg_m3` | 1.225 |
| `packs/base-weather/pack.json` | `/provides/other/weather_aerosols/earth/molar_kg` | 0.02897 |
| `packs/base-weather/pack.json` | `/provides/other/weather_aerosols/impact/density_kg_m3` | 3000 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/bands/equator_ice` | -1 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/bands/equatorial_bands_ice` | 99 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/bands/ice_giant_mass_earth` | 50 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/bands/wind_ice_m_s` | 400 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/dark_spots/strength_u` | 0.2, 0.4 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/outbreaks/strength_m_s` | 100 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/ovals/strength_u` | 0.6, 1 |
| `packs/base-weather/pack.json` | `/provides/other/weather_giants/strings/strength_u` | 0.3, 0.6 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/c-type/examples/Bennu/values/body.bulk_density` | 1190 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/c-type/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/s-type/examples/Eros/values/body.bulk_density` | 2670 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/s-type/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/m-type/examples/16 Psyche/values/body.bulk_density` | 3900 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/m-type/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/comet/examples/67P/Churyumov-Gerasimenko/values/body.bulk_density` | 530 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/comet/examples/Ceres/values/body.bulk_density` | 2160 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/rubble-pile/examples/Bennu/values/body.bulk_density` | 1190 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/asteroid/rubble-pile/examples/Eros/values/body.bulk_density` | 2670 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/planet/eyeball/examples/GJ 876 b/values/body.bulk_density` | 1200 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/planet/hot-eyeball/examples/GJ 876 b/values/body.bulk_density` | 1200 |
| `packs/sse3-classic/pack.json` | `/provides/other/classes/planet/cold-eyeball/examples/GJ 876 b/values/body.bulk_density` | 1200 |

## D. Matter-property constants in Rust

| File | Line | Name | Value as written |
|---|---|---|---|
| `crates/sse-activity/src/lib.rs` | 28 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-activity/src/lib.rs` | 29 | `KEV_M` | `1.239_84e-9` |
| `crates/sse-atoms/src/activity.rs` | 29 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-atoms/src/air.rs` | 17 | `K_B` | `1.380_649e-23` |
| `crates/sse-atoms/src/air.rs` | 19 | `AMU` | `1.660_539_066_60e-27` |
| `crates/sse-atoms/src/air.rs` | 23 | `N_LOSCHMIDT` | `2.686_780_111e25` |
| `crates/sse-atoms/src/chem.rs` | 11 | `R_GAS` | `8.314_462_618` |
| `crates/sse-atoms/src/common.rs` | 10 | `BOLTZMANN` | `1.380_649e-23` |
| `crates/sse-atoms/src/common.rs` | 12 | `AMU` | `1.660_539_066_60e-27` |
| `crates/sse-atoms/src/convection.rs` | 23 | `T_SUN` | `5772.0` |
| `crates/sse-atoms/src/convection.rs` | 24 | `G_SUN` | `GM_SUN / (R_SUN * R_SUN)` |
| `crates/sse-atoms/src/convection.rs` | 25 | `K_B` | `1.380_649e-23` |
| `crates/sse-atoms/src/convection.rs` | 26 | `AMU` | `1.660_539_066_60e-27` |
| `crates/sse-atoms/src/heat.rs` | 119 | `AL26_H0` | `0.00865 * 5.25e-5 * 0.3583` |
| `crates/sse-atoms/src/heat.rs` | 121 | `AL26_TAU_MYR` | `0.717 / LN2` |
| `crates/sse-atoms/src/layers.rs` | 14 | `NACL_MOLAR_MASS` | `0.058_44` |
| `crates/sse-atoms/src/layers.rs` | 16 | `WATER_KF` | `1.86` |
| `crates/sse-atoms/src/layers.rs` | 18 | `ICE_K` | `651.0` |
| `crates/sse-atoms/src/light.rs` | 24 | `MU0` | `1.256_637_062_12e-6` |
| `crates/sse-atoms/src/light.rs` | 26 | `E_OVER_ME` | `1.602_176_634e-19 / 9.109_383_701_5e-31` |
| `crates/sse-atoms/src/light.rs` | 28 | `M_PROTON` | `1.672_621_923_69e-27` |
| `crates/sse-atoms/src/light.rs` | 34 | `OMEGA_SUN` | `2.0 * m::PI / (25.38 * DAY)` |
| `crates/sse-atoms/src/magnetic.rs` | 27 | `MU0` | `1.256_637_062_12e-6` |
| `crates/sse-atoms/src/nebula.rs` | 23 | `H_ALPHA_J` | `6.626_070_15e-34 * 299_792_458.0 / 656.28e-9` |
| `crates/sse-atoms/src/nebula.rs` | 28 | `M_H` | `1.673_5e-27` |
| `crates/sse-atoms/src/nebula.rs` | 29 | `MU_H` | `1.4` |
| `crates/sse-atoms/src/nebula.rs` | 32 | `PC` | `648_000.0 / m::PI * AU` |
| `crates/sse-atoms/src/nebula.rs` | 33 | `YR` | `JULIAN_YEAR` |
| `crates/sse-atoms/src/nebula.rs` | 35 | `E_ION_H` | `13.598 * 1.602_176_634e-19` |
| `crates/sse-atoms/src/planet.rs` | 9 | `M_EARTH` | `GM_EARTH / G` |
| `crates/sse-atoms/src/planet.rs` | 10 | `M_JUPITER` | `GM_JUPITER / G` |
| `crates/sse-atoms/src/planet.rs` | 11 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-atoms/src/planet.rs` | 13 | `R_EARTH_ZENG` | `6.371e6` |
| `crates/sse-atoms/src/star.rs` | 14 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-atoms/src/tests.rs` | 11 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-atoms/src/tests.rs` | 12 | `M_EARTH` | `GM_EARTH / G` |
| `crates/sse-atoms/src/tests.rs` | 13 | `M_JUP` | `GM_JUPITER / G` |
| `crates/sse-climate/src/climate.rs` | 119 | `YEAR_S` | `3.15576e7` |
| `crates/sse-climate/src/conditions.rs` | 14 | `R_GAS` | `8.314462618` |
| `crates/sse-climate/src/conditions.rs` | 16 | `EARTH_COLUMN_KG_M2` | `101_325.0 / 9.80665` |
| `crates/sse-climate/src/ebm.rs` | 33 | `EARTH_OMEGA` | `2.0 * m::PI / (23.934 * 3600.0)` |
| `crates/sse-codes/src/plan.rs` | 21 | `G` | `6.674_30e-11` |
| `crates/sse-codes/src/plan.rs` | 22 | `M_EARTH` | `5.972_2e24` |
| `crates/sse-codes/src/plan.rs` | 23 | `R_EARTH` | `6.371_0e6` |
| `crates/sse-codes/src/plan.rs` | 149 | `EXOSPHERE_K` | `1000.0` |
| `crates/sse-core/src/quantity.rs` | 199 | `G` | `6.674_30e-11` |
| `crates/sse-core/src/quantity.rs` | 200 | `C` | `299_792_458.0` |
| `crates/sse-core/src/quantity.rs` | 201 | `AU` | `149_597_870_700.0` |
| `crates/sse-core/src/quantity.rs` | 202 | `DAY` | `86_400.0` |
| `crates/sse-core/src/quantity.rs` | 203 | `JULIAN_YEAR` | `365.25 * DAY` |
| `crates/sse-core/src/quantity.rs` | 204 | `GM_SUN` | `1.327_124_4e20` |
| `crates/sse-core/src/quantity.rs` | 205 | `GM_EARTH` | `3.986_004e14` |
| `crates/sse-core/src/quantity.rs` | 206 | `GM_JUPITER` | `1.266_865_3e17` |
| `crates/sse-core/src/quantity.rs` | 207 | `R_SUN` | `6.957e8` |
| `crates/sse-core/src/quantity.rs` | 208 | `L_SUN` | `3.828e26` |
| `crates/sse-core/src/quantity.rs` | 209 | `R_EARTH` | `6.378_1e6` |
| `crates/sse-core/src/quantity.rs` | 210 | `R_JUPITER` | `7.149_2e7` |
| `crates/sse-core/src/quantity.rs` | 211 | `STEFAN_BOLTZMANN` | `5.670_374_419e-8` |
| `crates/sse-cosmos/src/cosmology.rs` | 19 | `YEAR_S` | `31_557_600.0` |
| `crates/sse-cosmos/src/cosmology.rs` | 21 | `MPC_M` | `3.085_677_581_491_367e22` |
| `crates/sse-cosmos/src/cosmology.rs` | 23 | `RHO_CRIT_H2` | `2.775_366e11` |
| `crates/sse-cosmos/src/events.rs` | 39 | `C_PC_MYR` | `306_601.39` |
| `crates/sse-cosmos/src/events.rs` | 40 | `KMS_PC_MYR` | `1.022_712` |
| `crates/sse-cosmos/src/galaxy.rs` | 36 | `G_KPC` | `4.300_917e-6` |
| `crates/sse-cosmos/src/nucleus.rs` | 43 | `M_SUN_KG` | `1.988_47e30` |
| `crates/sse-cosmos/src/nucleus.rs` | 44 | `C_M_S` | `299_792_458.0` |
| `crates/sse-cosmos/src/nucleus.rs` | 45 | `G_SI` | `6.674_30e-11` |
| `crates/sse-cosmos/src/nucleus.rs` | 46 | `SIGMA_SB` | `5.670_374_419e-8` |
| `crates/sse-cosmos/src/nucleus.rs` | 47 | `YEAR_S` | `31_557_600.0` |
| `crates/sse-cosmos/src/nucleus.rs` | 49 | `L_EDD_PER_MSUN` | `1.257e31` |
| `crates/sse-cosmos/src/origin.rs` | 103 | `KMS_KPC_GYR` | `1.022_712` |
| `crates/sse-cosmos/src/patch.rs` | 35 | `GYR_S` | `3.15576e16` |
| `crates/sse-cosmos/src/patch.rs` | 36 | `MSUN_KG` | `1.988_41e30` |
| `crates/sse-cosmos/src/patch.rs` | 40 | `KMS_PC_MYR` | `1.022_712` |
| `crates/sse-cosmos/src/stars.rs` | 220 | `KMS_PC_MYR` | `1.022_712` |
| `crates/sse-cosmos/src/stars.rs` | 221 | `G_PC3` | `4.498_502e-3` |
| `crates/sse-flow/src/toy.rs` | 68 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-formation/src/accrete.rs` | 54 | `SOLAR_MASS_IN_EARTH` | `332_775.64` |
| `crates/sse-formation/src/accrete.rs` | 55 | `SOLAR_MASS_KG` | `1.989e30` |
| `crates/sse-formation/src/body.rs` | 28 | `R_JUPITER_RSUN` | `0.1028` |
| `crates/sse-formation/src/body.rs` | 140 | `MJ_MSUN` | `9.547_9e-4` |
| `crates/sse-formation/src/disc.rs` | 33 | `G` | `6.674_30e-11` |
| `crates/sse-formation/src/disc.rs` | 34 | `SIGMA_SB` | `5.670_374_419e-8` |
| `crates/sse-formation/src/disc.rs` | 35 | `K_B` | `1.380_649e-23` |
| `crates/sse-formation/src/disc.rs` | 36 | `M_H` | `1.673_5e-27` |
| `crates/sse-formation/src/disc.rs` | 37 | `AU_M` | `1.495_978_707e11` |
| `crates/sse-formation/src/disc.rs` | 38 | `MSUN_KG` | `1.988_47e30` |
| `crates/sse-formation/src/disc.rs` | 39 | `YR_S` | `3.155_76e7` |
| `crates/sse-formation/src/disc.rs` | 40 | `MYR_S` | `3.155_76e13` |
| `crates/sse-formation/src/early.rs` | 17 | `M_EARTH_KG` | `5.972_2e24` |
| `crates/sse-formation/src/early.rs` | 18 | `R_EARTH_M` | `6.371_0e6` |
| `crates/sse-formation/src/early.rs` | 19 | `MSUN_IN_EARTH` | `332_946.0` |
| `crates/sse-formation/src/early.rs` | 20 | `G` | `6.674_30e-11` |
| `crates/sse-formation/src/early.rs` | 21 | `AU_M` | `1.495_978_707e11` |
| `crates/sse-formation/src/early.rs` | 22 | `L_SUN_W` | `3.828e26` |
| `crates/sse-formation/src/early.rs` | 23 | `MYR_S` | `3.155_76e13` |
| `crates/sse-formation/src/early.rs` | 25 | `EARTH_OCEAN_KG` | `1.4e21` |
| `crates/sse-formation/src/growth.rs` | 30 | `G` | `6.674_30e-11` |
| `crates/sse-formation/src/growth.rs` | 31 | `K_B` | `1.380_649e-23` |
| `crates/sse-formation/src/growth.rs` | 32 | `M_H` | `1.673_5e-27` |
| `crates/sse-formation/src/growth.rs` | 34 | `AU_M` | `1.495_978_707e11` |
| `crates/sse-formation/src/growth.rs` | 35 | `MSUN_KG` | `1.988_47e30` |
| `crates/sse-formation/src/growth.rs` | 36 | `YR_S` | `3.155_76e7` |
| `crates/sse-formation/src/growth.rs` | 37 | `MSUN_IN_EARTH` | `332_946.0` |
| `crates/sse-formation/src/history.rs` | 218 | `AU_IN_RSUN` | `215.032` |
| `crates/sse-formation/src/leftovers.rs` | 53 | `R_EARTH_KM` | `6_371.0` |
| `crates/sse-formation/src/leftovers.rs` | 54 | `AU_KM` | `1.495_978_707e8` |
| `crates/sse-formation/src/leftovers.rs` | 55 | `MSUN_KG` | `1.988_47e30` |
| `crates/sse-formation/src/lifeline.rs` | 13 | `TEFF_SUN` | `5772.0` |
| `crates/sse-formation/src/lifeline.rs` | 15 | `M_CH` | `1.44` |
| `crates/sse-formation/src/pipeline.rs` | 30 | `GYR_S` | `1e9 * 365.25 * 86_400.0` |
| `crates/sse-formation/src/violent.rs` | 51 | `R_EARTH_AU` | `4.258_75e-5` |
| `crates/sse-formation/src/violent.rs` | 52 | `R_EARTH_M` | `6.371e6` |
| `crates/sse-formation/src/violent.rs` | 53 | `M_EARTH_KG` | `5.972_2e24` |
| `crates/sse-formation/src/violent.rs` | 54 | `G` | `6.674_30e-11` |
| `crates/sse-formation/src/violent.rs` | 56 | `V_ESC_EARTH` | `11.186` |
| `crates/sse-formation/src/violent.rs` | 57 | `V_KEP_1AU` | `29.784_7` |
| `crates/sse-import-v3/src/lib.rs` | 527 | `EARTH_OCEANS_M3` | `1.335e18` |
| `crates/sse-import-v3/src/lib.rs` | 528 | `EARTH_MEAN_DEPTH_M` | `3688.0` |
| `crates/sse-import-v3/src/reconstruct.rs` | 14 | `M_SUN_KG` | `1.988_47e30` |
| `crates/sse-import-v3/src/reconstruct.rs` | 15 | `M_EARTH_KG` | `5.972_2e24` |
| `crates/sse-import-v3/src/reconstruct.rs` | 16 | `AU_M` | `1.495_978_707e11` |
| `crates/sse-import-v3/src/reconstruct.rs` | 17 | `GYR_S` | `1e9 * 365.25 * 86_400.0` |
| `crates/sse-kernel/src/adopt.rs` | 28 | `M_SUN_KG` | `1.988_47e30` |
| `crates/sse-kernel/src/adopt.rs` | 37 | `M_EARTH_KG` | `5.972_2e24` |
| `crates/sse-kernel/src/clock.rs` | 29 | `JULIAN_YEAR_S` | `31_557_600.0` |
| `crates/sse-kernel/src/descent.rs` | 14 | `R_GAS` | `8.314_462_618` |
| `crates/sse-kernel/src/orbits.rs` | 28 | `G` | `6.674_30e-11` |
| `crates/sse-life/src/biosphere.rs` | 135 | `EARTH_G` | `9.81` |
| `crates/sse-life/src/biosphere.rs` | 136 | `EARTH_AIR` | `1.225` |
| `crates/sse-life/src/budget.rs` | 172 | `H` | `6.626_070_15e-34` |
| `crates/sse-life/src/budget.rs` | 173 | `C` | `299_792_458.0` |
| `crates/sse-life/src/budget.rs` | 174 | `K_B` | `1.380_649e-23` |
| `crates/sse-life/src/budget.rs` | 175 | `SIGMA` | `5.670_374_419e-8` |
| `crates/sse-life/src/budget.rs` | 176 | `WIEN_NM_K` | `2.897_771_955e6` |
| `crates/sse-life/src/budget.rs` | 177 | `YEAR_S` | `3.155_76e7` |
| `crates/sse-life/src/feedback.rs` | 25 | `G_PER_MOL_C` | `12.011` |
| `crates/sse-life/src/gradients.rs` | 29 | `YEAR_S` | `3.155_76e7` |
| `crates/sse-life/src/gradients.rs` | 30 | `R_GAS` | `sse_atoms::chem::R_GAS` |
| `crates/sse-light/src/lib.rs` | 35 | `H` | `6.626_070_15e-34` |
| `crates/sse-light/src/lib.rs` | 37 | `C` | `299_792_458.0` |
| `crates/sse-light/src/lib.rs` | 39 | `K_B` | `1.380_649e-23` |
| `crates/sse-light/src/lib.rs` | 41 | `SIGMA` | `sse_core::quantity::consts::STEFAN_BOLTZMANN` |
| `crates/sse-light/src/lib.rs` | 43 | `G` | `sse_core::quantity::consts::G` |
| `crates/sse-light/src/observe.rs` | 15 | `WIEN_B` | `2.897_771_955e-3` |
| `crates/sse-light/src/tests.rs` | 23 | `MSUN` | `1.988_41e30` |
| `crates/sse-light/src/tests.rs` | 24 | `YEAR` | `sse_core::quantity::consts::JULIAN_YEAR` |
| `crates/sse-light/src/tests.rs` | 609 | `PC` | `3.085_677_581e16` |
| `crates/sse-nav/src/lib.rs` | 31 | `M_SUN_KG` | `1.988_47e30` |
| `crates/sse-nav/src/lib.rs` | 32 | `M_EARTH_KG` | `5.972_2e24` |
| `crates/sse-nav/src/lib.rs` | 33 | `G0` | `9.806_65` |
| `crates/sse-nav/src/sky.rs` | 110 | `C_M_S` | `299_792_458.0` |
| `crates/sse-nebula/src/lib.rs` | 26 | `YEAR_S` | `sse_core::quantity::consts::JULIAN_YEAR` |
| `crates/sse-nebula/src/lib.rs` | 27 | `PC_M` | `648_000.0 / m::PI * sse_core::quantity::consts::AU` |
| `crates/sse-nebula/src/lib.rs` | 28 | `MSUN_KG` | `1.988_41e30` |
| `crates/sse-nebula/src/tests.rs` | 7 | `RSUN` | `6.957e8` |
| `crates/sse-place/src/model.rs` | 21 | `M_EARTH` | `GM_EARTH / G` |
| `crates/sse-place/src/model.rs` | 22 | `M_SUN` | `GM_SUN / G` |
| `crates/sse-place/src/model.rs` | 23 | `GYR_S` | `3.15576e16` |
| `crates/sse-place/src/model.rs` | 24 | `I_EARTH` | `L_SUN / (4.0 * m::PI * AU * AU)` |
| `crates/sse-sense/src/tests.rs` | 4 | `AU` | `1.495_978_707e11` |
| `crates/sse-sense/src/tests.rs` | 5 | `PC` | `3.085_677_581e16` |
| `crates/sse-sift/src/sifter.rs` | 119 | `YEAR` | `31_557_600.0` |
| `crates/sse-surface/src/climate.rs` | 14 | `R_EARTH_M` | `6.371e6` |
| `crates/sse-surface/src/climate.rs` | 15 | `G` | `6.6743e-11` |
| `crates/sse-surface/src/climate.rs` | 16 | `M_EARTH_KG` | `5.9722e24` |
| `crates/sse-surface/src/cover.rs` | 56 | `R_GAS` | `8.314462618` |
| `crates/sse-surface/src/impacts.rs` | 37 | `R_EARTH_KM` | `6371.0` |
| `crates/sse-surface/src/impacts.rs` | 38 | `M_EARTH_KG` | `5.972e24` |
| `crates/sse-surface/src/lib.rs` | 43 | `M_EARTH_KG` | `5.9722e24` |
| `crates/sse-surface/src/lib.rs` | 44 | `R_EARTH_M` | `6.371e6` |
| `crates/sse-surface/src/lib.rs` | 45 | `M_SUN_KG` | `1.98892e30` |
| `crates/sse-surface/src/lib.rs` | 46 | `AU_M` | `1.495978707e11` |
| `crates/sse-surface/src/lib.rs` | 47 | `G` | `6.6743e-11` |
| `crates/sse-surface/src/lib.rs` | 48 | `GYR_S` | `3.15576e16` |
| `crates/sse-surface/src/lib.rs` | 477 | `SIGMA_ROCK_PA` | `1.54e8` |
| `crates/sse-surface/src/lib.rs` | 478 | `SIGMA_ICE_PA` | `5.6e6` |
| `crates/sse-surface/src/lib.rs` | 479 | `RHO_ROCK` | `3500.0` |
| `crates/sse-surface/src/lib.rs` | 480 | `RHO_ICE` | `930.0` |
| `crates/sse-surface/src/lib.rs` | 504 | `T_MELT_ICE_K` | `273.15` |
| `crates/sse-surface/src/lib.rs` | 506 | `K_ICE_651` | `651.0` |
| `crates/sse-survey/src/lib.rs` | 23 | `G` | `sse_core::quantity::consts::G` |
| `crates/sse-survey/src/lib.rs` | 24 | `C` | `sse_core::quantity::consts::C` |
| `crates/sse-survey/src/lib.rs` | 25 | `HOUR` | `3600.0` |
| `crates/sse-survey/src/lib.rs` | 26 | `DAY` | `86_400.0` |
| `crates/sse-survey/src/lib.rs` | 27 | `YEAR` | `sse_core::quantity::consts::JULIAN_YEAR` |
| `crates/sse-survey/src/lib.rs` | 29 | `PHOTON_550` | `6.626_070_15e-34 * C / 550e-9` |
| `crates/sse-survey/src/tests.rs` | 3 | `AU` | `sse_core::quantity::consts::AU` |
| `crates/sse-survey/src/tests.rs` | 4 | `PC` | `3.085_677_581e16` |
| `crates/sse-survey/src/tests.rs` | 5 | `MSUN` | `1.988_41e30` |
| `crates/sse-survey/src/tests.rs` | 6 | `MEARTH` | `5.9722e24` |
| `crates/sse-survey/src/tests.rs` | 7 | `MJUP` | `1.898_19e27` |
| `crates/sse-time/src/binary.rs` | 24 | `RSUN_AU` | `0.004_650_47` |
| `crates/sse-time/src/epoch.rs` | 22 | `AU_RSUN` | `149_597_870_700.0 / 6.957e8` |
| `crates/sse-time/src/epoch.rs` | 23 | `ME_MSUN` | `3.986_004e14 / 1.327_124_4e20` |
| `crates/sse-time/src/kepler.rs` | 8 | `G` | `4.0 * m::PI * m::PI` |
| `crates/sse-time/src/tests.rs` | 10 | `MJ` | `9.547_919e-4` |
| `crates/sse-weather/src/aerosols.rs` | 31 | `R_GAS` | `8.314462618` |
| `crates/sse-weather/src/aerosols.rs` | 32 | `DAY_S` | `86_400.0` |
| `crates/sse-weather/src/aerosols.rs` | 33 | `YEAR_DAYS` | `365.25` |
| `crates/sse-weather/src/aerosols.rs` | 35 | `M_SO2` | `0.064066` |
| `crates/sse-weather/src/circulation.rs` | 17 | `R_GAS` | `8.314462618` |
| `crates/sse-weather/src/giants.rs` | 32 | `G` | `6.6743e-11` |
| `crates/sse-weather/src/giants.rs` | 33 | `M_EARTH_KG` | `5.9722e24` |
| `crates/sse-weather/src/giants.rs` | 34 | `DAY_S` | `86_400.0` |
| `crates/sse-weather/src/giants.rs` | 35 | `YEAR_D` | `365.25` |
