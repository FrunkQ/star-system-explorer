# Citations: light, reference objects and design files

**Applied (cite/light-reference PR):** fixes 1.1 to 1.9 (Sizes tab: Alpha Centauri B radius 600500 km and Teff 5231 K, Sirius A Teff 9900 K, Sirius B Teff 25370 K, Ceres and Titan sources, per-planet fact sheets), 2.1 (reflected light (2/3)·A·(R*/a)², was a /4 whole-sphere absorbed-flux factor), 2.2, 3.1 (Planck reference 2.6239e13) and the design-file fixes 4.1 to 4.8 and 4.10 to 4.13 in both the repo and project copies (4.13 in the project copy only; the repo copy of the nebulae design is stale). 4.9 (Lammer volume and page) waits for a read. Rows below record the state found before the fixes.


Checked 19 Nov 2026 by a verification agent. Repo: `/home/user/star-system-explorer` (no file edited). Paths below are repo-relative; `RO` = `packages/shell/src/documents/reference-objects.json`; `NL` = `handover/design/SSE4-nebulae-and-light-design.md` (authoritative copy `/mnt/project-files/sse4/SSE4-nebulae-and-light-design.md`, which is longer: its line numbers are given as "auth:N"). Sound, weather, handoff, classification and record-logic design files are identical in both places.

**Limits of this pass, stated honestly.** The shared web-fetch budget (100/hour for all agents) ran out part way, and then the per-turn search budget (200) ran out. Rows marked *unverified* were not reached; nothing in them is guessed. Where only a search result title or ADS bibcode was seen (not the page body), the row says "existence (title/bibcode)" and claims nothing about numbers.

Status key: **verified** (source read, number matches) / **corrected-metadata** (value fine, citation details wrong) / **corrected-physics** (value wrong, or not what the cited source says) / **unverified** / **textbook**.

## A. Sizes tab reference objects (`RO`)

| Where | Reference as written (short) | Relied on (number as used) | Status | What was checked (URL read) |
|---|---|---|---|---|
| RO:12 `ceres` | Park et al. 2019, Nature Astronomy 3, 223; doi 10.1038/s41550-018-0645-y | radius_km 469.7 | corrected-metadata | The value is right, but the paper is wrong. Park et al. 2019 *Icarus* 319, 812–827, "High-resolution shape model of Ceres from stereophotoclinometry using Dawn Imaging Data", doi 10.1016/j.icarus.2018.10.024: "volumetric mean radius of 469.72 km" (read: https://faculty.epss.ucla.edu/~ctrussell/russell_bib.html/papers/1-s2.0-S0019103518302835-main.pdf). The cited DOI did not resolve through doi.org (client error), and searching it found no Ceres radius paper. |
| RO:13 `europa` | NSSDCA Jovian satellite fact sheet | 1560.8 km | verified | https://nssdc.gsfc.nasa.gov/planetary/factsheet/joviansatfact.html: radius 1560.8 km. JPL SSD also gives 1560.80 ± 0.30 (https://ssd.jpl.nasa.gov/sats/phys_par). |
| RO:14 `moon` | NSSDCA Moon fact sheet | 1737.4 km | verified | https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html: volumetric mean radius 1737.4 km. |
| RO:15 `titan` | NSSDCA Saturnian satellite fact sheet | 2574.7 km | corrected-metadata | The NSSDCA sheet gives "2,575" km (https://nssdc.gsfc.nasa.gov/planetary/factsheet/saturniansatfact.html). 2574.7 matches JPL SSD's 2574.76 ± 0.02 km (cited there to Archinal et al. 2018, Celest. Mech. 130:22; https://ssd.jpl.nasa.gov/sats/phys_par), truncated rather than rounded. Cite JPL SSD and use 2574.8. |
| RO:16 `mercury` | NSSDCA planetary fact sheet (index URL) | 2439.7 km | corrected-metadata | The value is verified on https://nssdc.gsfc.nasa.gov/planetary/factsheet/mercuryfact.html (volumetric mean radius 2439.7). The cited index page gives only the equatorial diameter (4879 km), not this number. |
| RO:17 `mars` | same | 3389.5 km | corrected-metadata | The value is verified on .../marsfact.html (3389.5). The index page gives diameter 6792 km (equatorial). |
| RO:18 `earth` | same (volumetric mean radius) | 6371.0 km | corrected-metadata | The value is verified on .../earthfact.html (6371.000). The index page gives diameter 12,756 km. |
| RO:19 `k2-18b` | Benneke et al. 2019, ApJL 887, L14; doi 10.3847/2041-8213/ab59dc | 16628 km (2.61 × 6371) | verified | The DOI page https://iopscience.iop.org/article/10.3847/2041-8213/ab59dc/meta: Table 2 gives 2.610 ± 0.087 R⊕. ADS https://ui.adsabs.harvard.edu/abs/2019ApJ...887L..14B/abstract: "Water Vapor and Clouds on the Habitable-zone Sub-Neptune Exoplanet K2-18b", 2019. 2.610 × 6371.0 = 16628.3. |
| RO:20 `neptune` | NSSDCA planetary fact sheet (index) | 24622 km | corrected-metadata | The value is verified on .../neptunefact.html ("Volumetric mean radius (km) 24,622"). |
| RO:21 `saturn` | same | 58232 km | corrected-metadata | The value is verified on .../saturnfact.html (58,232). |
| RO:22 `jupiter` | same | 69911 km | corrected-metadata | The value is verified on .../jupiterfact.html (69,911). |
| RO:23 `brown-dwarf` (typical) | Burrows et al. 2001, Rev. Mod. Phys. 73, 719; doi 10.1103/RevModPhys.73.719 ("radii near 0.9 to 1 Jupiter radius") | 66000 km (0.944 R_J, mean) | unverified (value); citation exists | ADS https://ui.adsabs.harvard.edu/abs/2001RvMP...73..719B/abstract: "The theory of brown dwarfs and extrasolar giant planets", RvMP 73, 719, 2001 (arXiv astro-ph/0103383). The abstract does not state radii, and the body could not be read (fetch budget). The DOI did not resolve through doi.org from here (client error). Search found link.aps.org/doi/10.1103/RevModPhys.73.719, so the DOI looks right. |
| RO:24 `proxima` | Kervella et al. 2017, A&A 598, L7 (0.154 R☉); doi 10.1051/0004-6361/201629930 | 107300 km; teff 3050 K | radius verified; teff unverified (not in cited paper) | https://www.aanda.org/articles/aa/full_html/2017/02/aa29930-16/T1.html: R = 0.1542 ± 0.0045 R☉ (from the Mann et al. 2015 relation; the interferometric value is 0.141 ± 0.007). 0.1542 × 695700 = 107277 → 107300 ✓. The paper gives no Teff, so 3050 K needs its own source. |
| RO:25 `alpha-cen-b` | Kervella et al. 2017, A&A 597, A137 (0.859 R☉); doi 10.1051/0004-6361/201629505 | 597700 km; teff 5180 K | **corrected-physics** | https://www.aanda.org/articles/aa/full_html/2017/01/aa29505-16/aa29505-16.html, abstract: "RB = 0.8632 ± 0.0037 R⊙". 0.8632 × 695700 = 600528 km, not 597700. Teff is not derived there; Sect. 4.1 adopts "Teff [B] = 5231 ± 20 K" (Jofré et al. 2015, Heiter et al. 2015), not 5180. DOI, title and authors (Kervella, Bigot, Gallenne, Thévenin) are right. |
| RO:26 `sun` | IAU 2015 B3 (Prša et al. 2016, AJ 152, 41); doi 10.3847/0004-6256/152/2/41 | 695700 km; 5772 K | verified | https://knowledge.lancashire.ac.uk/id/eprint/16681/1/Pr%C5%A1a_2016_AJ_152_41.pdf, Table 1: R☉N = 6.957e8 m, T☉N = 5772 K; "The Astronomical Journal, 152:41 (7pp), 2016 August". |
| RO:27 `procyon-a` | Kervella et al. 2004, A&A 413, 251 (2.05 R☉); doi 10.1051/0004-6361:20031527 | 1425000 km; 6530 K | verified | https://www.aanda.org/articles/aa/abs/2004/01/aa4000/aa4000.html: "The diameter and evolutionary state of Procyon A…", A&A 413, 251–256 (2004), linear diameter 2.048 ± 0.025 D☉ (so R = 2.048 R☉). 2.048 × 695700 = 1424794. Teff: https://ar5iv.labs.arxiv.org/html/astro-ph/0309148, Sect. 2: "we adopt Teff=6530±50". |
| RO:28 `sirius-a` | Kervella et al. 2003, A&A 408, 681 (1.71 R☉); doi 10.1051/0004-6361:20030994 | 1190000 km; teff 9940 K | radius verified; **teff corrected-physics** (minor) | https://www.aanda.org/articles/aa/abs/2003/35/aa3846/aa3846.html: "The interferometric diameter and internal structure of Sirius A", A&A 408, 681–688 (2003), 1.711 ± 0.013 D☉ → 1190343 km ✓. https://ar5iv.labs.arxiv.org/html/astro-ph/0306604, Sect. 2.4: "we adopt an effective temperature of Teff = 9900 ± 200 K". 9940 is within the error but is not the cited paper's number. |
| RO:29 `arcturus` | Ramírez and Allende Prieto 2011, ApJ 743, 135 (25.4 R☉); doi 10.1088/0004-637X/743/2/135 | 17700000 km; 4290 K | verified | https://ui.adsabs.harvard.edu/abs/2011ApJ...743..135R: "Fundamental Parameters and Chemical Composition of Arcturus"; abstract "R = 25.4 ± 0.2 R⊙", "Teff = 4286 ± 30 K". 25.4 × 695700 = 17670780 → 17700000 (rounded to 3 s.f.). 4290 is 4286 rounded. |
| RO:30 `rigel` | Moravveji et al. 2012, ApJ 747, 108 (78.9 R☉); doi 10.1088/0004-637X/747/2/108 | 54900000 km; 12100 K | verified | https://www.arxiv.org/abs/1201.0843 (journal ref 2012ApJ...747..108M, DOI as cited). PDF https://www.arxiv.org/pdf/1201.0843, Table 1: "R/R⊙ 78.9 ± 7.4, This work"; Teff 12100 ± 150 K adopted from Przybilla et al. (2010). 78.9 × 695700 = 54890730 ✓. |
| RO:31 `betelgeuse` | Joyce et al. 2020, ApJ 902, 63 (764 +116/−62 R☉); doi 10.3847/1538-4357/abb8db | 531500000 km; 3600 K | verified | https://arxiv.org/abs/2006.09837v5 (DOI as cited), abstract: "764^{+116}_{-62} R⊙". PDF https://uhra.herts.ac.uk/id/eprint/8636/1/Joyce_2020_ApJ_902_63.pdf: "ApJ, 902:63 (25pp), 2020 October 13"; Teff 3600 ± 25 K adopted from Levesque & Massey (2020), widened to ±200. 764 × 695700 = 531514800 ✓. |
| RO:32 `neutron-star` (typical) | Riley et al. 2019, ApJL 887, L21 (12.7 km); doi 10.3847/2041-8213/ab481c | 12.7 km | verified | https://dspace.mit.edu/handle/1721.1/125899?show=full: "A NICER View of PSR J0030+0451: Millisecond Pulsar Parameter Estimation", ApJL 887, L21, DOI as cited; equatorial radius ≈ 12.71 (+1.14/−1.19) km, mass ≈ 1.34 M☉. This is one pulsar standing in for "typical" (see fudges). |
| RO:33 `sirius-b` | Bond et al. 2017, ApJ 840, 70 (0.0081 R☉); doi 10.3847/1538-4357/aa6af8 | 5630 km; 25000 K | radius verified; teff rounded (see fixes) | https://ar5iv.arxiv.org/html/1703.10625, Sect. VIII: R = 0.008098 ± 0.000046 R☉, Teff = 25,369 ± 46 K. 0.008098 × 695700 = 5634 → 5630 ✓. The DOI search matched the title "The Sirius System and Its Astrophysical Puzzles: Hubble Space Telescope and Ground-based Astrometry". The journal line "ApJ 840, 70" was not read on a publisher page. |
| `packages/ui-contract/src/compare.ts:11-13` | R_EARTH_KM 6371.0, R_JUPITER_KM 69911, R_SUN_KM 695700 | size words "× Earth/Jupiter/Sun" | verified | NSSDCA mean radii (earthfact, jupiterfact) and the IAU nominal solar radius, as above. These are *mean* radii, not the IAU nominal equatorial R⊕ = 6378.1 km or R_J = 71492 km (Prša 2016 Table 1). That is consistent with the objects, but worth saying in the UI. |
| `handover/packages/V14.md:53` | "The planets are NASA fact-sheet values" | – | verified | As the rows above show (but the URL should name the per-planet sheets). |

## B. Light (O1, `packages/ui-contract/src/light.ts`, `packages/shell/src/documents/light.recipes.json`, test)

| Where | Reference as written (short) | Relied on (number as used) | Status | What was checked |
|---|---|---|---|---|
| light.ts:23-26 | "CODATA 2018, exact in SI since 2019" | h 6.62607015e-34, c 299792458, k 1.380649e-23 | textbook | These are the exact SI defining constants. The numbers match. |
| light.ts:35-45 `BANDS` | design 2.1 | band edges 380/750 nm, 5/25/350 µm, 1 cm | textbook (convention) | These are conventional boundaries with no single source; they are fine as declared conventions. |
| light.ts:53-59 `LINES` | "rest wavelengths in air" | 500.7, 486.1, 656.3, 658.4, 671.6 nm | textbook | They match the standard air wavelengths ([O III] 500.68, Hβ 486.13, Hα 656.28, [N II] 658.35, [S II] 671.64; e.g. Osterbrock & Ferland). Note that [S II] is a doublet (671.6 + 673.1); see fudges. This was not re-read online this session. |
| light.ts:61 `NARROW_M` | "as amateur and Hubble narrowband filters roughly are" | 3 nm FWHM | unverified | It is a stated approximation with no source. Typical amateur filters are 3–7 nm. |
| light.ts:79-84 `planck` | – | 2hc²/λ⁵/(e^x−1) | textbook | The formula is correct. Checked numerically: B(500 nm, 5772 K) = 2.624e13 W sr⁻¹ m⁻³. |
| test/light.test.ts:33-34 | "about 2.6e13 … near 500 nm" | asserts 2.64e13 within 2 % | corrected-physics (test reference only) | The computed value is 2.6239e13, so the reference is 0.6 % off. The test passes only because of the 2 % slack (see fixes). |
| light.ts:116-127 `cmf`; recipes json:7 | Wyman, Sloan and Shirley (2013), JCGT 2(2), http://jcgt.org/published/0002/02/01/ | the multi-lobe fit coefficients | existence verified; coefficients unverified | Search: "Simple Analytic Approximations to the CIE XYZ Color Matching Functions" (https://research.nvidia.com/labs/rtr/publication/wyman2013simple/). The coefficient table could not be fetched (budget), so "2(2)" and the coefficients are not confirmed. O1-ui.md:49 already flags this. |
| light.ts:145-148 `linearSrgb` | IEC 61966-2-1, D65 | 3×3 XYZ→sRGB matrix; 12.92 / 1.055 / 2.4 transfer | textbook | It matches the published sRGB matrix and transfer function. The standard itself is paywalled and was not read. |
| light.ts:252-253 `DRAPER_K` | Draper 1847 | 798 K | textbook / ref unverified | 977 °F = 525 °C = 798 K is the commonly quoted Draper point. Draper, J. W. (1847), "On the production of light by heat", *Phil. Mag.* 30, 345 was not reached (search budget). |
| light.ts:255, 287 `DAYLIGHT` | "a 6504 K source (about D65)" | 6504 K blackbody as camera white | textbook + fudge | D65's correlated colour temperature is about 6504 K, but D65 is not a Planckian spectrum (see fudges). |
| light.ts:323-324 | – | R☉ 695 700 000 m; AU 149 597 870 700 m | verified / textbook | The solar radius is the IAU 2015 B3 nominal value (Prša 2016). The AU is exact by IAU 2012 Resolution B2 (textbook). |
| light.ts:362-363 | "A lit hemisphere's mean radiance: albedo × (R*/a)² / 4 of the star's" | reflected weight A·(R*/a)²/4 | **corrected-physics** | Derivation: incident flux πB(R*/a)². A Lambert surface has radiance A·B·(R*/a)²·cos i. The mean over the lit hemisphere's surface is A·B·(R*/a)²/2. The disc-mean seen at full phase is (2/3)·A·B·(R*/a)², with geometric albedo p = 2A/3, and `albedo` here is `planet.bond_albedo` (crates/sse-atoms/src/climate.rs:41). The /4 is the whole-sphere (day plus night) average of *absorbed* flux, not a radiance anyone sees. Reflected light is therefore under-weighted against thermal emission by 2× to 2.7×. |
| light.ts:407-420 `CLASS_K`, `teffFromSpectral` | none ("Rough", "Stubbed") | O 41000 … L 2100 K | unverified (no source) | It is labelled as stubbed. A citable table would be Pecaut & Mamajek 2013, ApJS 208, 9 and its online "modern mean dwarf" table (from memory, not checked this session). See fudges. |
| recipes json:23 (`sho` why) | "the palette the Hubble team used for the Pillars of Creation (1995)" | S II red, Hα green, O III blue | verified (existence) | Search: Hester et al. 1996, AJ 111, 2349, "Hubble Space Telescope WFPC2 Imaging of M16: Photoevaporation and Emerging Young Stellar Objects" (https://www.ipac.caltech.edu/publication/1996AJ....111.2349H; DOI 10.1086/117968 per https://oa.mg/work/10.1086/117968). The image was released in 1995. The channel order was not re-read. |
| recipes json:44 (`mid-ir` why) | Wien's law, ~10 µm for a few hundred K | – | textbook | b = 2.898e-3 m K; 290 K → 10 µm ✓. |
| recipes json:62 (`radio` why) | Rayleigh–Jeans, brightness ∝ T | – | textbook | ✓ |
| test/light.test.ts:20-26 | Mitchell Charity's blackbody table, http://www.vendian.org/mncharity/dir3/blackbody/ | 3000 K #ffb46b, 6500 K #fff9fd, 10000 K #ccdbff | unverified | Not reached (fetch then search budget). O1-ui.md:49 flags it. |
| `handover/packages/O1-ui.md:10,12,28-29,49` | Wyman 2013 fit; Draper 798 K; Charity table; Planck to 2 % | as above | as above | Same items as the light.ts rows. |

## C. Sound (`packages/ui-contract/src/sound.ts`, `handover/packages/N1.md`)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| sound.ts:85 | – | SEC_PER_YEAR 31 557 600 (Julian year) with P = √(a³/M) | textbook (small fudge) | Kepler's law in AU and M☉ gives the Gaussian sidereal year (365.2569 d), not the Julian year (365.25 d): 7 ppm. This is inaudible but worth a comment. |
| sound.ts:197 | – | M☉/M⊕ = 332 946 | textbook | It matches the IAU 2015 value of about 332 946.05 (memory). |
| sound.ts:219 | – | M⊕ = 5.9722e24 kg | textbook | NSSDCA earthfact (the mass was not re-read this session). |
| sound.ts:109 | "just intonation" | 1, 9/8, 5/4, 4/3, 3/2, 5/3, 15/8, 2 | textbook | This is the standard 5-limit just major scale ✓. |
| sound.ts:153-154, N1.md:11 | SYSTEM Sounds approach (Tamayo 2017) | orbital frequency raised by octaves | verified (concept); see design row | – |

## D. Design files

### D1. `handover/design/SSE4-nebulae-and-light-design.md` (also `/mnt/project-files/sse4/`, same line numbers for lines 147 and 195-201 in the repo, and auth:147, auth:237, auth:249-255)

| Where | Reference as written | Relied on | Status | What was checked |
|---|---|---|---|---|
| NL:147 | Noyes et al. (1984) | activity vs rotation, Rossby number | unverified | Two searches did not surface the record (expected: Noyes et al. 1984, ApJ 279, 763, "Rotation, convection, and magnetic activity in lower main-sequence stars": memory, not confirmed). |
| NL:147 | Wright et al. (2011) | activity–rotation | verified (existence) | https://www.arxiv.org/abs/1109.4634: "The stellar activity-rotation relationship and the evolution of stellar dynamos". The ApJ 743, 48 details were not read. |
| NL:147 | Skumanich (1972) | spin-down ∝ t^-1/2 | unverified | The search surfaced only secondary pages (Wikipedia "Gyrochronology", the astrobites review), not the paper record. |
| NL:147 | Maehara et al. 2012 superflares | flare statistics | verified (existence) | Search hits for "Superflares on solar-type stars" (Kyoto Univ. release, Nature news piece https://www.nature.com/articles/nature.2012.10653.pdf). Nature 485, 478 was not read. |
| NL:147 | Davenport 2016 | Kepler flare catalogue | verified (existence) | "The Kepler Catalog of Stellar Flares", VizieR J/ApJ/829/23 (https://dc.g-vo.org/rr/q/lp/custom/CDS.VizieR/J/ApJ/829/23), arXiv 1607.03494. |
| NL:147 | Blandford and Znajek (1977) | jet launching | verified (bibcode) | ADS 1977MNRAS.179..433B, "Electromagnetic extraction of energy from Kerr black holes". |
| NL:147 | Blandford and Payne (1982) | jet launching | verified (bibcode) | ADS 1982MNRAS.199..883B, "Hydromagnetic flows from accretion discs and the production of radio jets" (inspirehep). |
| NL:147 | "Lammer et al." (no year) | atmospheric loss | corrected-metadata (no year) | Lammer et al. 2003, ApJ 598, L121, "Atmospheric Loss of Exoplanets Resulting from Stellar X-Ray and Extreme-Ultraviolet Heating" exists (https://scilit.com/publications/af925b58005d51b3b8d5e0910581c500). Add the year. |
| NL:147 | Owen and Wu (2013, 2017) | evaporation | verified (bibcode) | ADS 2013ApJ...775..105O "Kepler Planets: A Tale of Evaporation"; ADS 2017ApJ...847...29O "The Evaporation Valley in the Kepler Planets". |
| NL:195 / auth:249 | Strömgren (1939) | ionised-region radius | verified (bibcode) | ADS 1939ApJ....89..526S. |
| NL:195 | Osterbrock and Ferland, *AGN²* | line emission | verified | 2nd ed., University Science Books 2006, ISBN 9781891389344 (https://uscibooks.aip.org/?p=3764). Add the edition and year. |
| NL:196 / auth:250 | Ferland et al., Cloudy release papers | line-strength grid | verified (existence) | "The 2017 Release of Cloudy", RMxAA 53, 385 (https://astronomia.unam.mx/journals/rmxaa/article/view/2017rmxaa..53..385f; arXiv 1705.10877). |
| NL:196 | Allen et al. (2008) MAPPINGS | shock library | verified | "The MAPPINGS III Library of Fast Radiative Shock Models", ApJS (arXiv 0805.0204; DOI 10.1086/589652 from the IOP record). |
| NL:197 / auth:251 | Kwok, Purton and Fitzgerald (1978) | interacting winds | verified (bibcode) | ADS 1978ApJ...219L.125K. |
| NL:197 | Balick (1987) | PN shape sequence | unverified | Two searches did not surface the paper record (expected 1987AJ.....94..671B: memory). |
| NL:197 | Balick and Frank (2002) | shaping review | verified | ARA&A 40, 439 (https://people.astro.ruhr-uni-bochum.de/kweis/vorlesung/araa2002_40_439.pdf). |
| NL:198 / auth:252 | Sedov and Taylor | blast wave | textbook | Taylor 1950, *Proc. R. Soc. A* 201, 159 (copy at https://www3.nd.edu/~powers/ame.60636/taylor1950a.pdf, by title in search); Sedov, *Similarity and Dimensional Methods in Mechanics* (1959). Not read. |
| NL:198 | Cioffi, McKee and Bertschinger (1988) | radiative SNR phases | verified (existence) | "Dynamics of radiative supernova remnants", DOI 10.1086/166834 (bib-bvb record). |
| NL:198 | Weaver et al. (1977) | wind bubbles | verified (bibcode) | ADS 1977ApJ...218..377W, "Interstellar bubbles. II - Structure and evolution". |
| NL:199 / auth:253 | Toomre and Toomre (1972) | tails and bridges | verified (bibcode) | ADS 1972ApJ...178..623T, "Galactic Bridges and Tails". |
| NL:199 | Lynds and Toomre (1976) | ring galaxies | verified (bibcode) | ADS 1976ApJ...209..382L. |
| NL:199 | Gunn and Gott (1972) | ram-pressure stripping | unverified | The search returned OSTI/scilit records without visible titles. Expected: ApJ 176, 1 (memory). |
| NL:200 / auth:254 | Rybicki and Lightman | Doppler, beaming, synchrotron | textbook | The search budget ran out before this could be checked. Standard text (Wiley 1979). |
| NL:201 / auth:255 | Hubble palette, Eagle Nebula (1995) | SHO mapping | verified (existence) | Hester et al. 1996 AJ 111, 2349 (as in section B). |
| auth:237 only | Seager and Mallén-Ornelas (2003) | transit geometry | verified (existence) | "A unique solution of planet and star parameters from an extrasolar planet transit light curve" (scilit; repositorio UC). ApJ 585, 1038 was not read. |
| auth:237 | Winn (2010), "Exoplanet transits and occultations" | transits | corrected-metadata | The actual title is "Transits and Occultations" (https://arxiv.org/abs/1001.2010), a chapter in *Exoplanets*, ed. S. Seager (Univ. Arizona Press, 2010). |
| auth:237 | Lovis and Fischer (2010) | radial velocity | unverified | The search did not surface the chapter record. |
| auth:237 | Perryman, *The Exoplanet Handbook* | all methods | unverified | The search budget ran out. |
| auth:237 | Gaudi (2012) | microlensing | verified (bibcode) | ADS 2012ARA&A..50..411G, "Microlensing Surveys for Exoplanets". |
| auth:237 | Wolszczan and Frail (1992) | pulsar planets | verified | https://www.nature.com/articles/355145a0 (Nature 355, 145). |

### D2. `handover/design/SSE4-sound-design.md` (identical in `/mnt/project-files/sse4/`)

| Where | Reference as written | Relied on | Status | What was checked |
|---|---|---|---|---|
| :62 | Leighton & Petculescu, "The sounds of music and voices in space", parts 1 and 2, Acoustics Today 5(3), July 2009 | voices and instruments on other worlds | corrected-metadata | https://eprints.soton.ac.uk/71546: "The sound of music and voices in space part 1: theory", Acoustics Today 5(3), 17–26, 2009. The title says "sound", not "sounds". Part 2 is eprints 71545 (title not read). |
| :63 | Maurice et al., "In situ recording of Mars soundscape", Nature 605, 653 (2022) | sound speed ~240 m/s, two speeds either side of ~240 Hz | verified (metadata) | https://authors.library.caltech.edu/records/yymhf-wyt75 ("Nature, Vol 605, 26 May 2022"); DOI 10.1038/s41586-022-04679-0 (RePEc). The quoted numbers were not re-read. Add the DOI. |
| :64 | Petculescu & Lueptow, "Atmospheric acoustics of Titan, Mars, Venus, and Earth", Icarus 186 (2007) | absorption and sound-speed models | unverified | Two searches found the ASA press-room pages by Petculescu but not the Icarus record. |
| :65 | Bass & Chambers, "Absorption of sound in the Martian atmosphere", JASA 109 (2001) | Mars absorption | unverified | The search did not surface the record. |
| :97 | Tamayo et al. (2017), ApJL, TRAPPIST-1 chain ("exact title to confirm") | period-to-pitch model | corrected-metadata (title supplied) | ADS 2017ApJ...840L..19T, "Convergent Migration Renders TRAPPIST-1 Long-lived"; arXiv 1704.02957. |
| :98 | Zanella et al., Nature Astronomy (2022), arXiv 2206.13536 | sonification review | verified | https://www.nature.com/articles/s41550-022-01848-z.pdf (DOI 10.1038/s41550-022-01848-z). Add the DOI. |
| :99 | Hermann, Hunt, Neuhoff (eds), *The Sonification Handbook* (2011) | mapping practice | verified | Logos Verlag Berlin, ISBN 978-3-8325-2819-5 (https://www.logos-verlag.de/cgi-bin/buch/isbn/2819; Bielefeld record https://pub.uni-bielefeld.de/record/2442349). |
| :101 | SYSTEM Sounds video (YouTube) | intended mapping | unverified | Not viewable from here, as the design already says. |

### D3. `handover/design/SSE4-weather-design.md` (identical in `/mnt/project-files/sse4/`; citations only)

| Where | Reference as written | Status | What was checked |
|---|---|---|---|
| :19 | Kaspi and Showman 2015, ApJ 804, title as given | verified (existence) | arXiv 1407.6349, same title; author PDF https://www.weizmann.ac.il/EPS/kaspi/sites/EPS.kaspi/files/publications/Kaspi-Showman-2015.pdf. Article number 60 was not read. |
| :26 | Yang, Cowan and Abbot 2013, ApJL 771, L45 | verified (bibcode) | ADS 2013ApJ...771L..45Y, "Stabilizing Cloud Feedback Dramatically Expands the Habitable Zone of Tidally Locked Planets". |
| :50 | Brueshaber, Sayanagi and Dowling, Icarus 2019 | unverified | Two searches did not surface it. |
| :50 | O'Neill, Emanuel and Flierl, Nature Geoscience 2015 | verified (existence) | MIT DSpace https://dspace.mit.edu/handle/1721.1/100773, "Polar vortex formation in giant-planet atmospheres due to moist convection". The journal line was not read. |
| :51 | Adriani et al. 2018, Nature 555, "Clusters of cyclones encircling Jupiter's poles" | verified | RePEc nature v555 i7695, DOI 10.1038/nature25491. The 8 + 1 north and 5 + 1 south claim is the known result (not re-read). |
| :52 | Godfrey 1988, Icarus | verified (title) | "A hexagonal feature around Saturn's north pole" (https://scilit.com/publications/de1064028530b69cc738b3133fd6639f). Volume 76 and page 335 were not read. |
| :52 | Barbosa Aguiar et al. 2010, Icarus 206, 755 | verified (bibcode) | ADS 2010Icar..206..755B. |
| :52 | Morales-Juberías et al., ApJL ("year to confirm") | corrected-metadata (year supplied) | ADS 2015ApJ...806L..18M: 2015, ApJL 806, L18. |

### D4. Other design and package files in scope

| Where | Finding | Status |
|---|---|---|
| `SSE4-design-handoff.md` | No literature citations or sourced physics constants found (grep for authors, years, DOIs and "verify"). | – (nothing to check) |
| `SSE4-classification-design.md` | Cites only internal sources (Alex's brief, SSE 3 docs, "current planetary science"). No literature citation. | – |
| `SSE4-record-logic-programs-design.md` | No literature citations. | – |
| `handover/packages/V12.md`, `V6.md` | No physics citations (symbology and 3D/globe). V12:72 describes the reference-link mechanism only. | – |
| `crates/sse-observe` | No light or observation physics with sources (disclosure and views only). | – |
| `packages/renderer`, `packages/sound/src`, `packages/views/*`, `chart-grammar/References.svelte` | No physics constants with sources. The sound engine has only synthesis settings. | – |
| `packages/shell/src/documents/*.json` other than RO and light.recipes | No references (derivations.base.json maps flows to facts). | – |

## Detail ladder (VP6, `packages/renderer/src/ladder-noise.ts` `NOISE_REFERENCES`)

Checked 24 Nov 2026 through Crossref and the URLs named.

| Where | Reference as written (short) | Relied on | Status | What was checked |
|---|---|---|---|---|
| `ladder-noise.ts` `simplex`, `permute` | McEwan, Sheets, Richardson and Gustavson 2012, J. Graphics Tools 16(2) 85-94; doi 10.1080/2151237X.2012.649621 | branch-free simplex corner ranking; a permutation polynomial mod 289 as a hash | corrected-metadata (author order; the hash's exact form unverified) | Crossref: title, volume, pages and DOI match; authors are McEwan, Sheets, Richardson, Gustavson. No abstract; whether the paper gives (34x² + x) mod 289 was not read. The code uses (34x² + 10x) mod 289 and its exactness in float32 is checked by the GPU test. |
| `simplex` | Gustavson 2005, "Simplex noise demystified" (weber.itn.liu.se) | the 3D simplex lattice: skew 1/3, unskew 1/6, radial kernels | unverified | Page refused automated fetch (robots). The method is reproduced and tested in `ladder.test.ts` (gradient, periodicity). |
| `simplex` (turning gradients) | Perlin and Neyret 2001, "Flow noise", SIGGRAPH Technical Sketches (www-evasion.imag.fr/Publications/2001/PN01/) | gradients turned over time give swirling detail | unverified (existence via a citing paper) | Page refused automated fetch. Bridson et al. 2007 cites it with this URL ("SIGGRAPH Technical Sketches and Applications 187"). |
| `curl` | Bridson, Hourihan and Nordenstam 2007, ACM TOG 26(3) 46; doi 10.1145/1276377.1276435 | the curl of a noise potential has no divergence | verified (metadata); the property is tested | Crossref: title, volume, issue, article 46 match; Crossref spells the second author "Houriham". |
| `cell` | Worley 1996, SIGGRAPH 96 291-294; doi 10.1145/237170.237267 | F1 and F2 distances to feature points | verified (metadata) | Crossref: title, author, pages match. |
| `ladder.ts` `FLOW_REACH`, `ladder-node.ts` flow | Vlachos 2010, "Water flow in Portal 2", SIGGRAPH Advances in Real-Time Rendering | two-phase flow maps | verified (existence) | advances.realtimerendering.com/s2010: the syllabus lists the talk by Alex Vlachos (Valve). |
| `ladder.ts` `EDDY_EXPONENT` | Kolmogorov 1941 (inertial range, velocity ∝ size^(1/3)) | eddy speed by size | textbook | Standard result; used as a stand-in (fudge register). |

## Counts

| Section | Rows | verified | corrected-metadata | corrected-physics | unverified | textbook |
|---|---|---|---|---|---|---|
| A. Reference objects (+ compare.ts, V14) | 24 | 12 | 8 | 2 | 2 | 0 |
| B. Light | 17 | 2 | 0 | 2 | 4 | 9 |
| C. Sound code | 5 | 1 | 0 | 0 | 0 | 4 |
| D1. Nebulae and light design | 30 | 20 | 2 | 0 | 6 | 2 |
| D2. Sound design | 8 | 3 | 2 | 0 | 3 | 0 |
| D3. Weather design (citations) | 8 | 6 | 1 | 0 | 1 | 0 |
| **Total** | **92** | **44** | **13** | **4** | **16** | **15** |

How rows are counted: Proxima is counted unverified (its radius is verified, but its teff is not in the cited paper). Sirius A is counted corrected-physics for teff only. Sirius B is verified (its teff 25000 is a rounding of 25369; an optional fix is listed). "Verified (existence/bibcode)" rows confirm that the work exists as named; they do not confirm any number, and the design files quote none from them.

## ND2 part 2b: the limb law (added 24 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.light.limb_temperature` | Rybicki, G. B. and Lightman, A. P. 1979, *Radiative Processes in Astrophysics*, Wiley, §1.8. doi:10.1002/9783527618170 | grey atmosphere T⁴ = ¾Teff⁴(τ + ⅔) and Eddington-Barbier (light at μ leaves τ ≈ μ) | textbook | Standard result; the test recovers Eddington's (2 + 3μ)/5 law. |
| `sse.light.reflected_limb` | Minnaert, M. 1941, ApJ 93, 403. doi:10.1086/144279 | b = μ0^k μ^(k−1); k = 1 is Lambert | textbook | Law as stated in the paper's title and abstract; k = 1 reduces to Lambert in the test. |

## O3b: outflows from stars (added 24 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.outflow.giant_mass_loss` | Schröder, K.-P. and Cuntz, M. 2005, ApJ 630, L73. doi:10.1086/491579 | Ṁ = 8e-14 L R / M (T/4000 K)^3.5 (1 + g☉/(4300 g)) M☉/yr | textbook | Equation 4 as widely quoted; not re-fetched 24 Nov. |
| `sse.outflow.dust_shell` | Höfner, S. and Olofsson, H. 2018, A&ARv 26, 1. doi:10.1007/s00159-017-0106-5 | R_c = (R/2)(T/T_c)^((4+p)/2), condensation near 1000 to 1500 K, winds 5 to 30 km/s | textbook | Review; values as quoted. |
| `sse.outflow.young_jet` | Hartigan, P., Edwards, S. and Ghandour, L. 1995, ApJ 452, 736. doi:10.1086/176343; Bally, J. 2016, ARA&A 54, 491. doi:10.1146/annurev-astro-081915-023341 | jets carry about 0.1 of the accretion; speeds near the escape speed at launch, a few hundred km/s; knots from episodic accretion | textbook | As quoted in the reviews. |

## VP-L4: rainbows and halos (added 24 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.light.rainbow` | Nussenzveig, H. M. 1977, The theory of the rainbow, Scientific American 236 (4), 116. doi:10.1038/scientificamerican0477-116 | Descartes' ray: cos² i = (n² − 1)/(k(k + 2)), deviation kπ + 2i − 2(k + 1)r | textbook | Reproduces water's 42° and 51°. |
| `sse.light.halo` | Tape, W. 1994, Atmospheric Halos, AGU Antarctic Research Series 64. doi:10.1029/AR064 | prism minimum deviation D = 2 asin(n sin(A/2)) − A | textbook | Reproduces the 22° and 46° halos for ice. |


## MF9 part 2b: how aurorae shine (added 24 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.glow.electron_range` | Rees, M. H. 1989, Physics and Chemistry of the Upper Atmosphere, Cambridge University Press, section 3.3. doi:10.1017/CBO9780511573118 | electron range in air R = 4.30e-7 + 5.36e-6 E^1.67 g cm⁻², E in keV | from memory | The fit as widely quoted; not re-fetched 24 Nov. Earth's green line lands at 100 to 115 km with it. |
| `sse.glow.wind_power` | Akasofu, S.-I. 1981, Energy coupling between the solar wind and the magnetosphere, Space Sci. Rev. 28, 121. doi:10.1007/BF00218810 | a few per cent of the wind's power on the bubble's cross-section gets in | textbook | The 1 % that comes down as aurora is a stand-in (F59); Earth then gives about 37 GW a hemisphere, inside NOAA's 5 to 100 GW. |
| `sse.glow.spin_power` | Hill, T. W. 2001, The Jovian auroral oval, J. Geophys. Res. 106, 8101. doi:10.1029/2000JA000302 | power to spin up plasma loaded at Ṁ to the corotation limit, of order 10¹⁴ W for Jupiter | textbook | Jupiter with Io's tonne a second gives 1.2e14 W. |
| `sse.glow.line_survival` | Kalogerakis, K. S. et al. 2009, Ann. Geophys. 27, 2183. doi:10.5194/angeo-27-2183-2009; Kawamura, S. et al. 2020, Earth Planets Space 72. doi:10.1186/s40623-020-01229-8 | radiative lifetime against collisional quenching; green 0.7 s, red about 110 s | checked (visual polish citation check, section 15) | Heights: green 110 km, red 203 km, N2+ 110 km for 8 keV electrons in the US Standard Atmosphere 1976. |
| base-light `aurora` on `oi-6300` | Streit, G. E. et al. 1976, J. Chem. Phys. 65, 4761. doi:10.1063/1.432930 | O(¹D) quenched by N2 at about 2e-11 cm³ s⁻¹ | from memory | Not re-fetched. |
| base-light `aurora` yields (Earth) | Rees, M. H. and Luckey, D. 1974, J. Geophys. Res. 79, 5181. doi:10.1029/JA079i034p05181 | about 1 kR of 557.7 nm and 200 R of 427.8 nm per erg cm⁻² s⁻¹ | from memory | Yields divided by the parent's share near 110 km; others authored (F59). |
| base-light `aurora` on `ha`, `hb` | Dyudina, U. A. et al. 2016, Saturn's aurora observed by the Cassini camera at visible wavelengths, Icarus 263, 32. doi:10.1016/j.icarus.2015.05.022 | a strong H-alpha line, pink low down | checked (visual polish citation check, section 15) | Jupiter's mix comes out pink (#FF4D9A). |
| base-light `aurora` on `lya` | Ingersoll, A. P. et al. 1998, Icarus 135, 251. doi:10.1006/icar.1998.5971 | giants' aurorae are hydrogen lines and bands | checked (visual polish citation check, section 15) | Yield authored. |
| base-light `aurora` on `oi-5577` (CO2) | Knutsen, E. W. et al. 2025, Detection of visible-wavelength aurora on Mars, Sci. Adv. 11, eads1563. doi:10.1126/sciadv.ads1563 | green 557.7 nm on Mars, 101 ± 14 R | checked (visual polish citation check, section 15) | No measured yield; the CO2 yield is authored. |
| Earth's upper air (sol-field.json) | US Standard Atmosphere 1976, NOAA-S/T 76-1562. https://ntrs.nasa.gov/citations/19770009539 | number density and mean molecular weight, 90 to 500 km | from memory | Values as tabulated; not re-fetched. |
| `AuroraSettings` | Evans, D. S. 1974, J. Geophys. Res. 79, 2853. doi:10.1029/JA079i019p02853; Gustin, J. et al. 2004, Icarus 171, 336. doi:10.1016/j.icarus.2004.04.020 | auroral arc electrons of a few to about ten keV; Jupiter's main oval tens of keV | from memory | 8 keV and 50 keV are chosen within those ranges (F59). |

## K-e: phase tables per kind of particle (added 25 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.light.mie_scattering`, `sse_atoms::mie` | Bohren, C. F. and Huffman, D. R. 1983, Absorption and Scattering of Light by Small Particles, Wiley. doi:10.1002/9783527618156 | the Mie series: equations 4.53 (aₙ, bₙ), 4.61 and 4.62 (efficiencies, asymmetry), 4.74 (S₁, S₂); appendix A's method (downward logarithmic derivative) | textbook | Written fresh, not copied. Reproduces the book's own example (n 1.55, x 5.213: Q_ext = Q_sca = 3.1054), the Rayleigh limit at x = 0.01, and Q_ext → 2 at x = 2000. |
| `sse_atoms::mie` term count | Wiscombe, W. J. 1980, Improved Mie scattering algorithms, Applied Optics 19, 1505. doi:10.1364/AO.19.001505 | x + 4 x^⅓ + 2 terms | textbook | As quoted. |

## O4 part 1: forbidden lines in glowing gas (added 25 Nov 2026)

| Where | Reference | Relied on | Status | What was checked |
|---|---|---|---|---|
| `sse.nebular.collisional_excitation` | Osterbrock, D. E. and Ferland, G. J. 2006, Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd edn, chapter 3. https://uscibooks.aip.org/books/astrophysics-of-gaseous-nebulae-and-active-galactic-nuclei-second-edition/ ; Storey, P. J., N-level atoms (notes), equations 6, 11, 15, 19. https://pages.astro.umd.edu/~jpha/N-level.pdf | q₂₁ = β Υ/(g₂ √T), q₁₂ by detailed balance, β = h²/((2π mₑ)^3/2 √k) | textbook | β worked from the sheet's h, k and mₑ comes to 8.629e-12 m³ s⁻¹ K^½, the notes' 8.629e-6 in cgs. |
| base-light `oiii`, `oiii-4959` `nebular` | Storey, P. J., Sochi, T. and Badnell, N. R. 2014, Collision strengths for nebular [O III] optical and infrared lines, MNRAS. https://arxiv.org/abs/1311.6517 | table 5 (1D2 at 20273.27 cm⁻¹; 3P1, 3P2 at 113.18, 306.17); table 6 at log T = 4.0 (Υ 0.24917, 0.75062, 1.26198 to 1D2) | read (table values quoted by a fetch of the arXiv text) | Summed Υ 2.26177 over the 3P term. The 5007:4959 split is taken as 3:1 (the notes say "about three"); Storey and Zeippen 2000 (MNRAS 312, 813) could not be reached from the container: to check (F61). |
| base-light `sii`, `sii-6731` `nebular` | A&A 694, A133 (2025), table B.5, a fit to Tayal, S. S. and Zatsarinny, O. 2010, ApJS 188, 32. doi:10.1051/0004-6361/202451865 ; doi:10.1088/0067-0049/188/1/32 | Υ at 10⁴ K 3.83 (2D5/2) and 2.56 (2D3/2) from the printed fits; A 2.019e-4 and 6.84e-4 s⁻¹ | read (table quoted by a fetch) | The pair's ratio runs 1.49 (thin) to 0.48 (dense), against the textbook 1.45 to 0.44. |
| `sse.nebular.photoionisation_cross_section`, base-light `sii` `balance.fit` | Verner, D. A., Ferland, G. J., Korista, K. T. and Yakovlev, D. G. 1996, ApJ 465, 487, equation 1, table 1 row S II. doi:10.1086/177435 | E₀ 8.787 eV, σ₀ 313.6 Mb, y_a 3.442, P 12.81, y_w 0.7354, y₀ 2.782, y₁ 0.1788, valid to 184.6 eV | read (arXiv astro-ph/9601009, quoted by a fetch) | σ just above threshold about 5 Mb. |
| `sse.nebular.ion_balance`, base-light `sii` `balance.recombination_m3_s` | Badnell, N. R. et al. 2015, Bootstrapping dielectronic recombination from second-row elements and the Orion Nebula. https://arxiv.org/abs/1503.05905 | table 6, S2+ to S+ at 10⁴ K: dielectronic 2.94e-12 plus radiative 1.54e-12 cm³ s⁻¹ (level average); the sum is ours | read (quoted by a fetch) | As quoted. |
| `sse.nebular.ion_zone_share`, base-light `photon_takers` | Osterbrock and Ferland 2006, chapter 2 (section 2.4, helium's zones); NIST Atomic Spectra Database ionisation energies (H 13.598434, He 24.587389, S 10.36001, S+ 23.33788, O+ 35.12112, O2+ 54.93554 eV). https://physics.nist.gov/PhysRefData/ASD/ionEnergy.html | photon counting ion by ion, the photons above 24.6 eV taken by helium | textbook; energies from memory of the NIST table, to check | A 40,000 K star's glow gives [O III] 5007/Hβ 2.0 (Orion about 3). |
