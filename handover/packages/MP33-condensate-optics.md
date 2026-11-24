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
| `h2so4` (Venus cloud droplets, 75 % and 84.5 % by weight) | giant bands | Palmer and Williams 1975, Appl. Opt. 14, 208, Table I (measured at 300 K) | The text layer of the HITRAN PDF loses the row alignment. Read the page image, or the HITRAN aerosol data files. |
| `nh4sh` (Jupiter's and Saturn's NH4SH cloud) | giant bands | Sromovsky et al. 2017 treat the main cloud as non-absorbing, with n near 1.4 for NH3 and near 1.8 for NH4SH (Sato et al. 2013: 1.85). | No measured n,k found yet. Howett et al. 2007 and Ferraro et al. 1980 are not yet read. |
| `jupiter-chromophore` (red-brown chromophore) | giant bands | Carlson et al. 2016, Icarus 274, 106; Sromovsky et al. 2017, Icarus 291, 232, Fig. 6B (n = 1.4 assumed) | k is printed only in a figure. |
| H2S ice, silicate, sulphide, alkali haze, iron | giant bands | (not yet searched) | |
| SO2 frost; S8 and the red S3/S4 allotropes; N2 ice and CH4 ice in the visible; CO2 frost (Hansen 2005); a space-weathering darkening rule (Hapke 2001, npFe0) | plumes | Nash et al. or Hapke 1981 (SO2); Spencer et al. 2000 and Carlson et al. 2007 (sulphur); Grundy and Schmitt 1998 (CH4); Grundy 1993 (N2) | Queued as MP34. Where only reflectance is published, use a `{lambda_nm, reflectance, grain size}` table. |
