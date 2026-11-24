# MP32: sea liquids (part 1, liquid water's optics)

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

## Waiting (part 2)

- **Density, viscosity and surface tension of every sea liquid along saturation.** Water, methane, ethane, ammonia, nitrogen, helium-4 and propane are all waiting.
  - The NIST WebBook fluid pages hold all of them, but the cloud cannot read them. curl is blocked, and the web reader gets only the input form or "[binary data]".
  - Mulero et al. 2012 (JPCRD 41, 043105) gives the surface tension fits, but no readable copy was found.
  - An open-source reference implementation of the same correlations (CoolProp) would give every value at once, with each correlation's paper cited, but installing it here was refused.
- **Liquid methane and ethane absorption.** No readable table was found:
  - Ramaprasad et al. 1978 (Icarus 35, 400) gives band strengths for ten liquid methane bands from 0.62 to 1.94 µm, but has no free text.
  - Grundy et al. 2002 (Icarus 155, 486) includes a liquid methane spectrum at 93 K; its data files were not reachable.
  - Adams et al. 2012 (GRL 39, L04309) gives reflectances only.
- **Ethane substance row.** The sheet has no `c2h6` substance yet. It comes with part 2.
- **Liquid methane refractive index.** Khare et al. (about 1989, NTRS 19910007878) gives n of about 1.28 at 110 K in an OCR-degraded table. It is not used until a clean copy is read.
