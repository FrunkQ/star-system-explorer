# MP30: electrical conductivity for the magnetic field ledger

The proven core asked for these values for MF2. The field ledger reads the magnetic diffusivity η = 1/(μ0 σ) from the sheet through Look up, so σ itself must be on the sheet; the ledger never types a number. The values are built by `scripts/conductivity_rows.py`, which `build-base-reference.py` calls after the thermal rows.

## Rows

| Row | Fields | Source |
|---|---|---|
| `earth-core-liquid` (new mixture) | `sigma_elec` 1.11e6 S/m at the core-mantle boundary (4186 K); `sigma_elec_icb` 1.25e6 S/m at the inner-core boundary (5700 K) | Pozzo et al. 2012, Table 1: Fe with 10 % Si and 8 % O, density jump 0.6 g/cm³ |
| `iron` | `sigma_elec_liquid` 1.36e6 S/m for pure liquid iron at the core-mantle boundary (1.56e6 at the inner-core boundary is in the note) | Pozzo et al. 2012, Table 1, the values in parentheses |
| `giant-envelope` (new mixture) | `sigma_elec_profile`: a table of rows `[r_rj, density, t_k, p_pa, sigma_elec]` from 0.0923 to 0.930 Jupiter radii (4180 GPa down to 23 GPa); `metallic_from`: conduction becomes metal-like (σ ≥ 2e4 S/m) from 0.5 Mbar, about 0.9 Jupiter radii | French et al. 2012, Tables 1 and 2 and section 6.2, Jupiter model J11-8a |
| `seawater` (new mixture) | `salinity_practical` 35; `sigma_elec` 4.2914 S/m at 15 °C; `sigma_elec_25c` 5.31 S/m; `sigma_elec_t_fit` (PSS-78 r_T); `salinity_fit` (PSS-78 S(K15), valid 2 to 42) | Sea-Bird AN14 (Culkin and Smith's 42.914 mmho/cm), UNESCO TPMS 37, Lewis 1980 eq. 8 |

### Notes on reading the rows

- **The ledger walks the giant profile.** The ledger finds the top of the dynamo by walking down `sigma_elec_profile` until σ passes its threshold.
- **The giant profile is Jupiter's H-He mixture, not pure hydrogen.** It is held on a mixture row, as the magnetic fields design section 10 item 1 asks for mixtures.
- **The 25 °C seawater value is derived, not printed.** It is C(35,15,0) multiplied by Lewis's r_T(25 °C) = 1.23654.
- **Seawater is quoted at the sea surface.** PSS-78's zero pressure means the sea surface, so the values carry `at_place` "sea surface" rather than an absolute pressure.
- **Pozzo's core pressures are missing.** The core-boundary pressures are not printed in the text that was read, so the core rows name the place (`at_place`) instead of giving `at_pa`.

## Not done

- **Ionic water (`h2o.ionic_fluid`) now reads 1.5e4 S/m** (MP34 batch, 24 Nov), from Millot et al. 2018 (Nature Physics, OSTI copy): about 150 S/cm on reverberating shocks at 70 to 180 GPa below about 5000 K, the states closest to an ice giant's interior. The note on the row also gives the single-shock Hugoniot plateau (about 30 S/cm above 30 GPa and 2000 K) and Yakushev 1998 (2000 S/m at 75 GPa). No Rust reader takes this field yet, so no golden moved. Earlier misses: Knudson et al. 2012 gives only reflectivities; the Redmer and Knudson slides show values only on plots; Redmer et al. 2011 was not reached.
- **Salinity dependence away from 15 °C needs PSS-78's full eq. 9.** Its constant k was not legible in the OCR, so only the r_T fit at salinity 35 and the 15 °C definition are on the row.

All values were read on 24 Nov 2026 from fetched text and are unverified.
