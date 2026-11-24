# MP33: optical constants of condensates (part 1, tholin)

The giant-bands thread asked for these, and the plumes thread needs them too. Both want colours from the sheet, not typed into the renderer.

## Shape

Every row carries the same shape, read through Look up:

`optical.refractive_index = {kind: "table", lambda_nm, n, k, src, check, note}`

The absorption coefficient is alpha = 4πk/λ.

The same shape is already on water as `optical.refractive_index_liquid` (MP32).

## Part 1 rows

| Id | What | Source |
|---|---|---|
| mixture `titan-tholin` | n and k, 264 to 2422 nm, 24 printed rows. k falls from about 0.18 in the ultraviolet to about 4e-4 near 1.8 µm; n is 1.62 to 1.72. | Khare et al. 1984, Icarus 60, 127, Table I (0.9 N2 / 0.1 CH4 discharge at 0.2 mbar, room temperature) |

Khare's Table I was read from the text layer of the HITRAN aerosol archive's PDF.

- That text layer drops the minus signs of the k exponents. The paper's own text (k near 4e-4 at 1.5 µm, a few hundredths in the visible) fixes them as negative.
- The column order was inferred as: wavelength, k mantissa, k exponent, n. One leading column, running 1.57 to 2.15, could not be identified and is not used.
- The rows are unverified until checked against the page image (page 133).

## Waiting, same shape

| Wanted | By | Lead | Blocker |
|---|---|---|---|
| `h2so4` (Venus cloud droplets, 75 % and 84.5 % by weight) | giant bands | Palmer and Williams 1975, Appl. Opt. 14, 208, Table I (measured at 300 K) | The text layer of the HITRAN PDF loses the row alignment. HITRAN's ASCII copy is `PalmerandWilliams.dat`, beside `Aerosol_readme_2008.pdf` in HITRAN2012/Aerosols/ascii/. The site disallows automated fetching, so read it from a local machine. |
| `nh4sh` (Jupiter's and Saturn's NH4SH cloud) | giant bands | Sromovsky et al. 2017 treat the main cloud as non-absorbing, with n near 1.4 for NH3 and near 1.8 for NH4SH (Sato et al. 2013: 1.85). | No measured n,k found yet. Howett et al. 2007 and Ferraro et al. 1980 are not yet read. |
| `jupiter-chromophore` (red-brown chromophore) | giant bands | Carlson et al. 2016, Icarus 274, 106; Sromovsky et al. 2017, Icarus 291, 232, Fig. 6B (n = 1.4 assumed) | k is printed only in a figure. |
| H2S ice, silicate, sulphide, alkali haze, iron | giant bands | (not yet searched) | |
| SO2 frost; S8 and the red S3/S4 allotropes; N2 ice and CH4 ice in the visible; CO2 frost (Hansen 2005); a space-weathering darkening rule (Hapke 2001, npFe0) | plumes | Lab sources Trumbo et al. 2022 (PSJ, HST/STIS Io map) uses, read from the summary of its reference list and still to be checked: SO2 frost, Hapke et al. 1981 (Icarus 47, 361, about 77 K); alpha-S8, Gradie et al. 1980 (Icarus 44, 373); beta-S8 and quenched melt, Moses and Nash 1991 (Icarus 89, 277); S4 and red sulphur glass, Meyer et al. 1972 (J. Mol. Spectrosc. 42, 335; ACS Adv. Chem. 110, 53); Na2S, Nash and Nelson 1979 (Nature 280, 763); S2O, Hapke and Graham 1989 (Icarus 79, 47). Also Grundy and Schmitt 1998 (CH4), Grundy 1993 (N2). | Queued as MP34. Where only reflectance is published, use a `{lambda_nm, reflectance, grain size}` table. |
| Mars bright dust (nanophase ferric oxide or palagonite); crystalline hematite; fresh basalt and mature lunar regolith reflectance | plumes thread, to replace the typed `cover.colours` in packs/base-surface | Morris et al. 2000 and Bell et al. 2000 (Pathfinder spectra); Christensen et al. 2000 (hematite); RELAB spectra; Hapke 2001 (npFe0) | Queued as MP34. The plumes thread's order of use: tholin, ferric oxide, SO2 and sulphur, N2 and CH4 ice, then the rest. |
