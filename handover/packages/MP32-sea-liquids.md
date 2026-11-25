# MP32: sea liquids (liquid water's optics; every sea liquid's density, viscosity and surface tension)

The climate thread needs data on each sea liquid for sea colour and for waves:

- liquid absorption spectra;
- viscosity;
- surface tension;
- density.

It asked for water, methane, ethane, ammonia, nitrogen, helium and the like. The rows are built by `scripts/liquid_rows.py`, which `build-base-reference.py` calls after the conductivity rows.

## Part 1 rows (on substance `h2o`, `optical` block)

| Field | What | Source |
|---|---|---|
| `absorption_liquid` | table `lambda_nm` and `alpha` (m⁻¹), 380 to 2000 nm; `src_from_nm` says which source covers which range | Pope and Fry 1997 (380 to 700 nm, integrating cavity), Hale and Querry 1973 (750 to 2000 nm) |
| `refractive_index_liquid` | table `lambda_nm`, real `n` and imaginary `k`, 500 to 2000 nm | Segelstein 1981 |

Light falls to 1/e over 1/alpha metres:

| Wavelength | Distance |
|---|---|
| 420 nm (absorption least) | about 22 m |
| 700 nm | about 1.6 m |
| 2 µm | 0.14 mm |

The builder checks that Segelstein's k agrees with Hale and Querry's alpha through alpha = 4πk/λ at 1 µm.

The values were read from the Oregon Medical Laser Center's copies of the authors' tables and are unverified.

### Scattering (added 24 Nov for W-sea2)

`optical.scattering_liquid` on `h2o`: `{kind table, lambda_nm [450, 550], b {v [0.0035, 0.0015], unit m-1}, exponent -4.3, backscatter_share 0.5, seawater_factor 1.3}`, all from Morel and Prieur 1977 (Limnol. Oceanogr. 22, 709), as read by the climate and weather thread from a scan; unverified. The 650 nm entry of Table 1 was unreadable, so readers use the exponent beyond 550 nm. The sea's deep colour is R = 0.33 b_b / (a + b_b) with `absorption_liquid`.

## Part 2 rows (on each substance's `phases` block)

`saturated_liquid` is a table of the liquid along its boiling curve. It runs from just above the triple point to 0.95 of the critical temperature, in nine rows.

| Column | Unit |
|---|---|
| `t_k` | K |
| `p_pa` | Pa (the vapour pressure) |
| `density` | kg m⁻³ |
| `viscosity` | Pa s |
| `surface_tension` | N m⁻¹ |

`src_by_column` names the paper behind each column:

| Substance | Equation of state (p, density) | Viscosity | Surface tension |
|---|---|---|---|
| `h2o` | Wagner and Pruss 2002 (IAPWS-95) | Huber et al. 2009 | Mulero et al. 2012 |
| `ch4` | Setzmann and Wagner 1991 | Sotiriadou et al. 2025 | Mulero et al. 2012 |
| `c2h6` (new row) | Buecker and Wagner 2006 | Friend et al. 1991 | Mulero et al. 2012 |
| `nh3` | Gao et al. 2023 | Monogenidou et al. 2018 | Mulero et al. 2012 |
| `n2` | Span et al. 2000 | Huber et al. 2024 | Mulero et al. 2012 |
| `he` | Ortiz-Vega et al. 2023 | Arp et al. 1998 | Mulero et al. 2012 |
| `co2` | Span and Wagner 1996 | Laesecke and Muzny 2017 | Mulero et al. 2012 |
| `ar` | Tegeler et al. 1999 | Lemmon and Jacobsen 2004 | Mulero et al. 2012 |
| `c3h8` (new row) | Lemmon et al. 2009 | Vogel et al. 1998 | Mulero et al. 2012 |

### How the values were read

The values were evaluated with CoolProp 8.1.0, the open-source implementation of the same reference correlations NIST's WebBook uses. NIST's own pages could not be read from the cloud.

- The CoolProp install was approved by Alex on 24 Nov and lived in scratch only. Nothing of the library is committed or shipped.
- `MP32-data/make_sat_liquids.py` wrote `MP32-data/sat-liquids.csv`. The builder reads only that CSV.
- Every source row says it was read through CoolProp, so the citation check can compare the numbers against each paper's own tables.

### New substance rows

Ethane and propane are new substance rows, with:

- formula;
- triple and critical points;
- liquid density just above the triple point;
- the saturated-liquid table.

They have no vapour-pressure fit yet, so the climate does not treat them as volatiles. Adding Fray and Schmitt 2009's ethane fit, a condensed colour and the latent heat makes ethane a Titan volatile, and that may change golden hashes.

### Helium (added 24 Nov for the helium-sea test)

On `he.phases`: `lambda_point_k` 2.1768 K at `lambda_point_pa` 5039.33 Pa, `critical_point_k` 5.1953 K and `critical_point_pa` 228322.8 Pa, read from CoolProp 8.1.0 (the Ortiz-Vega et al. 2023 equation of state, NIST IR 8474, whose lower limit is the lambda line). Helium has no triple point at its own vapour pressure, so it has no `triple_point_k`; its `saturated_liquid` p_pa column is its vapour curve, and the row's note says so. Climate's volatile list does not pick helium up, so no golden moved.

## Still waiting

- **Liquid methane and ethane absorption.** No readable table was found:
  - Ramaprasad et al. 1978 (Icarus 35, 400) gives band strengths for ten liquid methane bands from 0.62 to 1.94 µm, but has no free text.
  - Grundy et al. 2002 (Icarus 155, 486) includes a liquid methane spectrum at 93 K. It is on SSHADE as SPECTRUM_BS_20130114_001 (experiment DOI 10.26302/SSHADE/EXPERIMENT_BS_20130114_001): absorption coefficient in cm-1 from 2380 to 14350 cm-1, which is 0.70 to 4.2 µm, so it has no visible part. Its export zip disallows robots, so download it from a local machine.
  - Adams et al. 2012 (GRL 39, L04309) gives reflectances only.
- **Ethane as a volatile.** Ethane needs its vapour-pressure fit, colour and latent heat, which go through the golden lane.
- **Liquid methane refractive index.** Khare et al. (about 1989, NTRS 19910007878) gives n of about 1.28 at 110 K in an OCR-degraded table. It is not used until a clean copy is read.
