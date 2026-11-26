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

## MP34 part 1 rows

| Id | What | Source |
|---|---|---|
| mixture `troilite` (FeS, the "sulphide" the giant-bands thread asked for) | n,k from 250 to 774 nm at 300 K on two crystal axes: `optical.refractive_index` is E parallel to x, `optical.refractive_index_ez` is E parallel to z | Kranhold et al. 2026, A&A, DOI 10.1051/0004-6361/202659441, the Jena optical constants database files |
| `optical.n_solid` on substances `ch4` (1.32) and `n2` (1.23) | Real index of the ice in the visible; k is still to come (visible k of these ices is tiny and sample-dependent) | CH4: Protopapa et al. 2015 n0 at 15800 cm-1 (633 nm), from Trotta 1996. N2: Quirico, SSHADE experiment DOI 10.26302/SSHADE/EXPERIMENT_BS_20120925_001 |
| `optical.refractive_index_solid` on substance `h2o` (ice Ih) | n,k at 16 wavelengths from 250 to 2000 nm, 266 K; k below 400 nm is an upper limit | Warren and Brandt 2008, JGR 113, D14220, printable table IOPtable_2008.pdf |
| mixtures `basalt`, `limonite` (mostly goethite), `magnetite` | n,k on the files' own grid: basalt and limonite 260 to 2000 nm, magnetite 310 to 1230 nm | Egan and Hilgeman 1979 (basalt, limonite); Schlegel et al. 1979, J. Phys. C 12, 1157 (magnetite); all read from the Waterloo NPSG files |
| mixture `hematite` (Fe2O3, the ferric oxide the plumes thread asked for) | n,k at 13 wavelengths from 250 to 1000 nm; orientation not stated | Sokolik and Toon 1999, JGR 104(D8), 9423, read from the Waterloo NPSG files riHm.txt and ecHm.txt (1 nm steps, 212 to 1001 nm) |
| mixtures `jsc-mars-1`, `mms-2`, `mgs-1` (Mars dust and soil simulants) | k at 15 wavelengths from 250 to 2000 nm; n is held at 1.5 (assumed, as in the paper) | Martikainen et al. 2023, ApJS 268, 47, Table 3 (irregular-grain shape GRS1). The note on each row also gives the airborne Martian dust of Wolff et al. 2009, read through Dabrowska et al. 2015 Table 1: k about 0.01 at 488 nm and 0.001 to 0.002 at 647 nm, darker than the simulants. |

## Ground reflectance (25 Nov, for True's ground colour)

Alex saw a brown Moon: base-surface typed a beige "regolith" colour. The sheet now holds measured reflectance in a new shape, `optical.reflectance = {kind table, lambda_nm, r, quantity "reflectance factor", i_deg 30, e_deg 0, maturity, maturity_index?, src, check}`. Readers interpolate linearly and hold the ends. The detail ladder thread samples it at 450, 550 and 650 nm, and draws R_fresh × factor^(maturity/255) from the proven core's space-weathering block.

| Id | Sample | Maturity (Is/FeO) | Source |
|---|---|---|---|
| `lunar-soil-mature-highland` | Apollo 16 soil 62231, 415 to 1000 nm | mature (the paper's word) | Pieters 1999, Clementine calibration standard, Table 2 |
| `lunar-soil-mature-mare` | Apollo 11 soil 10084, 400 to 1000 nm | 88 | LSCC spectra at RELAB (Taylor et al. 2001); Is/FeO via Wu et al. 2024 Table 1 |
| `lunar-soil-submature-mare` | Apollo 17 soil 71501 | 44 | same |
| `lunar-soil-immature-mare` | Apollo 12 soil 12030 | 20 | same |

Still to come: an immature highland soil (the RELAB LSCC page also lists Apollo 16 files 61221 and 67461; their maturity was not found), and a reflectance row for each other cover material. The base-surface pack names which row each cover material reads.

## Waiting, same shape

| Wanted | By | Lead | Blocker |
|---|---|---|---|
| `h2so4` (Venus cloud droplets, 75 % and 84.5 % by weight) | giant bands | Palmer and Williams 1975, Appl. Opt. 14, 208, Table I (measured at 300 K) | The text layer of the HITRAN PDF loses the row alignment. HITRAN's ASCII copy is `PalmerandWilliams.dat`, beside `Aerosol_readme_2008.pdf` in HITRAN2012/Aerosols/ascii/. The site disallows automated fetching, so read it from a local machine. |
| `nh4sh` (Jupiter's and Saturn's NH4SH cloud) | giant bands | Sromovsky et al. 2017 treat the main cloud as non-absorbing, with n near 1.4 for NH3 and near 1.8 for NH4SH (Sato et al. 2013: 1.85). | No measured n,k found yet. Howett et al. 2007 and Ferraro et al. 1980 are not yet read. |
| `jupiter-chromophore` (red-brown chromophore) | giant bands | Carlson et al. 2016, Icarus 274, 106; Sromovsky et al. 2017, Icarus 291, 232, Fig. 6B (n = 1.4 assumed) | k is printed only in a figure. |
| H2S ice, silicate, alkali haze, iron (sulphide is troilite, above) | giant bands | (not yet searched) | |
| SO2 frost; S8 and the red S3/S4 allotropes; N2 ice and CH4 ice in the visible; CO2 frost (Hansen 2005); a space-weathering darkening rule (Hapke 2001, npFe0) | plumes | Lab sources Trumbo et al. 2022 (PSJ, HST/STIS Io map) uses, read from the summary of its reference list and still to be checked: SO2 frost, Hapke et al. 1981 (Icarus 47, 361, about 77 K); alpha-S8, Gradie et al. 1980 (Icarus 44, 373); beta-S8 and quenched melt, Moses and Nash 1991 (Icarus 89, 277); S4 and red sulphur glass, Meyer et al. 1972 (J. Mol. Spectrosc. 42, 335; ACS Adv. Chem. 110, 53); Na2S, Nash and Nelson 1979 (Nature 280, 763); S2O, Hapke and Graham 1989 (Icarus 79, 47). Also Grundy and Schmitt 1998 (CH4), Grundy 1993 (N2). | Queued as MP34. Where only reflectance is published, use a `{lambda_nm, reflectance, grain size}` table. |
| Mature lunar regolith reflectance and the npFe0 darkening rule (hematite, basalt, limonite and magnetite are on the sheet, above) | plumes thread, to replace the typed `cover.colours` in packs/base-surface | Morris et al. 2000 and Bell et al. 2000 (Pathfinder spectra); Christensen et al. 2000 (hematite); RELAB spectra; Hapke 2001 (npFe0) | Queued as MP34. The plumes thread's order of use: tholin, ferric oxide, SO2 and sulphur, N2 and CH4 ice, then the rest. |

## MP36: sulphur and Venus cloud acid (26 Nov 2026)

Read in Claude's built-in browser on the owner's PC, because the cloud's network refuses these hosts. Every number below was copied from a data file that was opened, not from memory or a figure. All rows are `unverified` until the citation check reads them.

| Id | What | Source |
|---|---|---|
| mixture `sulfur` (the kernel's stain id) | `optical.reflectance`, absolute reflectance of reagent sulphur powder, 25 printed rows from 301 to 2496 nm. Dark below 400 nm (about 0.04), the yellow edge from 410 to 500 nm, about 0.83 from 650 to 800 nm. | USGS splib05a, Sulfur GDS94 Reagent (Clark et al. 2003, OFR 03-395), file `ASCII/M/sulfur_gds94.6449.asc`. Its XRD matches JCPDS 8-247, which is orthorhombic alpha S8. The file gives no grain size or viewing geometry. |
| mixtures `sulfuric-acid-75`, `sulfuric-acid-84p5` | `optical.refractive_index` n,k at 300 K from 702 to 2500 nm, every row with a measured k (72 and 71 rows). `optical.n_visible` holds the real index only, from 360 to 702 nm, because the paper prints no k there. | Palmer and Williams 1975, Appl. Opt. 14, 208. The ARIA `.ri` files and HITRAN's `palmer_williams_h2so4.dat` were both read, and they agree. |

### Still unsourced (searched 26 Nov)

- **SO2 frost.** Hapke et al. 1981 (Icarus 47, 361) is the visible lab spectrum that Trumbo et al. 2022 use. Its abstract says only that reflectance has "a sharp drop at 0.32 μm". The full text sits behind Elsevier. The SSHADE SO2 data are mid-infrared only: Trotta and Schmitt 1996 cover 400 to 2700 cm-1, and Nash and Betts 1995 cover 2.3 to 23 µm. The USGS splib05a has no SO2 frost. Next places to look: splib07a on ScienceBase (it blocks automated browsers behind a bot check) or RELAB.
- **Beta-S8, quenched red sulphur, S4.** Moses and Nash 1991 and Meyer et al. 1972 are paywalled. Trumbo 2022 gives no data location for them.
- **`jupiter-chromophore`.** Carlson et al. 2016 (INAF open copy) and Sromovsky et al. 2017 (arXiv 1706.02779) show k only in figures (Carlson Fig. 3A, Sromovsky Fig. 6B). n = 1.4 is assumed in both. Neither paper names a data file.
- **`nh4sh`.** ARIA and HITRAN have no NH4SH and no ammonia ice. The only n values found so far are model assumptions (Sato 2013: 1.85).

## MP37: water ice ground, fine and coarse (26 Nov 2026)

Asked by the Europa and Titan package (PR 628) and earlier by IG1: `ground_optics` took ice's colour from water's authored `condensed_colour_rgb` (0.95 at 550 nm). Read in Claude's built-in browser on the owner's PC. Every number was copied from a data file that was opened. Rows are `unverified` until the citation check reads the paper.

| Id | What | Source |
|---|---|---|
| mixture `water-ice-fine` | `optical.reflectance`, reflectance factor (i 0, e 30) of pure spherical ice grains of about 70 µm at 100 K, 450 to 1000 nm every 50 nm. 0.982 at 550 nm. | Stephan et al. 2021, Minerals 11, 1328 (doi 10.3390/min11121328, CC BY). Data: SSHADE/CSS EXPERIMENT_OP_20201223_001, spectrum SPECTRUM_OP_20201223_301, read from `/plot/SPECTRUM_OP_20201223_301/1.micron.json.gz`. SHINE gonio radiometer, Spectralon calibration, vacuum, behind sapphire windows. |
| mixture `water-ice-coarse` | The same for irregular grains of about 680 µm crushed from a slowly frozen slab. 0.814 at 550 nm. | Same experiment, spectrum SPECTRUM_OP_20201223_304. |

The kernel's `ground_optics` (sse-surface `ice_ground.rs`) now reads `water-ice-fine` at 550 nm through the sheet's Look up, so ice ground goes from 0.95 to 0.982. Grain size is not modelled, so open ice is taken as fine fresh grains; the coarse row is on the sheet for a later grain-growth block. Fudge register row "sse-surface ice_ground.rs `ground_optics` (IG1)" updated.

What this means for Europa: measured ice is brighter than the authored 0.95, not darker, so the ice colour does not explain Europa's tiles coming out above its measured geometric albedo of 0.68. Only coarse grains (0.81) would bring it down, and nothing yet says Europa's ice is coarse. The rest of the gap is the non-ice matter, the reflectance factor being used as an albedo, and the grain size.

The renderer's `ground-rows.json` still draws ice from `h2o`'s n,k (Warren and Brandt 2008, as a Hapke grain layer). Pointing it at `water-ice-fine` is the renderer thread's choice.

### Not sourced, searched 26 Nov

- **USGS splib05a H2O-Ice GDS136** (frost grown on a 77 K cold finger, `ASCII/L/h2o_ice_gds136.9212.asc`): every row below 859 nm is printed as deleted, so it gives no visible colour. Its other ice files are melting snow. splib07a is on ScienceBase behind a bot check, and the old crustal.usgs.gov speclab pages redirect to usgs.gov.
- **400 nm.** Stephan 2021's files start at 450 nm, and the 450 nm point is noisy (in the 70 µm sample it ranges from 0.94 to 1.05 across the eight temperatures). The table holds its first value below 450 nm.
- **Few-micron frost.** The Bern SCITEAS spectra of 4.5 µm and 67 µm spherical ice (SSHADE EXPERIMENT_OP_20171130_001) are valid only from 700 nm, and their reflectance factor goes above 1 at their geometry, so they were not used.
- **The JPL ECOSTRESS library** (frost and snow samples) did not load from the owner's PC.
