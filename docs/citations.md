# Citations register

Every reference, published relation and quoted constant in SSE4, with what was checked against the source.
Alex's rule: every block is real, explained physics with a link to its paper (a DOI or URL on every reference,
shown in "Why?"), and a published relation is checked against its source before it is relied on.

## Status words
- **verified**: the DOI or URL resolves to the work named, and the number or relation matches the work
  (units, validity range, equation or table) as used in the code.
- **corrected**: something was wrong and has been fixed. The row says what: *metadata* (wrong DOI, year, page,
  author, equation number; physics unchanged) or *physics* (a misremembered relation or constant; values changed,
  goldens re-baselined with the reason in the package log).
- **unverified**: the source could not be reached or read (paywall, book not online). Not guessed; left as written.
- **textbook**: a standard law (Kepler, Stefan-Boltzmann, Newton) with a textbook pointer; the law itself is checked,
  the page or equation number may be unverified.

## How to add one
A paper cited in a pack or the code must also be on the run-through's master list (`handover/citations/`):
run `python3 scripts/citations.py --add` and commit it (`scripts/check.sh` warns when one is missing).
Add a row to the area file below in the same PR that adds the reference. Columns: where (file or atom id),
reference as written, what is relied on, status, what was checked.

## Areas
| Area | File | Checked (rows) | Wrong (fixed) | Physics changed |
|---|---|---|---|---|
| Stars | [citations/stars.md](citations/stars.md) | 65 | 5 | multiplicity by primary mass, triple share, main-sequence L and R from the fitted atoms, low-mass q_crit |
| Orbits and formation | [citations/orbits-formation.md](citations/orbits-formation.md) | 101 | 9 | giant-star tide factor (was inverted), pebble isolation mass diffusion term |
| Interiors and heat | [citations/interiors-heat.md](citations/interiors-heat.md) | 32 | 8 | K-40 and U isotope shares, Al-26 decay energy, Zeng radius unit |
| Surfaces | [citations/surfaces.md](citations/surfaces.md) | 72 | 5 | none (wording and DOIs only) |
| Climate and life | [citations/climate-life.md](citations/climate-life.md) | 79 | 9 | methane lifetime in oxic air, hydrogen escape coefficient |
| Galaxy and cosmology | [citations/galaxy-cosmology.md](citations/galaxy-cosmology.md) | 90 | 11 | merger mass ratios, disc heating exponent, Hawking lifetime |
| Master data sheet (track MP) | [citations/matter-reference.md](citations/matter-reference.md) | 34 | 0 | none yet (sources for the sheet) |
| Galaxy histories design (track GH, GH0) | [citations/galaxy-histories.md](citations/galaxy-histories.md) | 113 (metadata) | 1 (Bekki and Chiba DOI) | none (design stage) |
| Naming (track NM, NM0) | [citations/naming.md](citations/naming.md) | 11 | 2 metadata (WGSN list has no origin column; Hessman 2010 is a preprint), 1 replaced (English place-name elements are copyrighted) | none (design stage) |
| Magnetic fields (track MF) | [citations/magnetic-fields.md](citations/magnetic-fields.md) | 108 | 2 | none yet (design references; DOIs checked in MF0) |
| Light, reference objects and design files | [citations/light-reference.md](citations/light-reference.md) | 108 | 18 | planet disc brightness factor, Sizes tab star radii and temperatures |
| Real spaceflight (track RS) | [citations/spaceflight.md](citations/spaceflight.md) | 3 of 5 | 0 | none (test data for the flight log) |
| Exotic objects (track XO) | [citations/exotic-objects.md](citations/exotic-objects.md) | 12 | 1 | none (XO1: gravity blocks and the bend table; XO18: hazard map lines and the Sun check) |

Counts are table rows in each area file after the first pass (19 Nov 2026). About a third of rows are still
unverified, mostly because sources could not be fetched in that pass; they say so and nothing was guessed.

## Weather (track W)

Checked by two research passes on 19 Nov (raw results `handover/packages/W-citations/`); every reference in
`packs/base-weather` carries a DOI or URL and the pack loader refuses one without.

| Where | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| circulation.rs | Held and Hou 1980, J. Atmos. Sci. 37, 515, doi 10.1175/1520-0469(1980)037<0515:NASCIA>2.0.CO;2 | φ_H = (5/3 g H Δh / (Ωa)²)^½ | verified (form via two citing sources) | small-angle form |
| circulation.rs | Rhines 1975, J. Fluid Mech. 69, 417, doi 10.1017/S0022112075001504 | k_β = (β/2U)^½ | corrected (physics: design wrote π sqrt(U/β)) | abstract |
| circulation.rs | Kaspi and Showman 2015, ApJ 804, 60, doi 10.1088/0004-637X/804/1/60 | trends of jets and Hadley width with spin | verified | abstract |
| formations.rs | Emanuel 2003, Annu. Rev. Earth Planet. Sci. 31, 75, doi 10.1146/annurev.earth.31.100901.141259 | genesis over sea above 26 °C, at least 5° from the equator | corrected (physics: design wrote 26.5 °C) | quoted text |
| formations.rs | Eady 1949, Tellus 1(3), 33-52, doi 10.1111/j.2153-3490.1949.tb01265.x | baroclinic waves; 3.9 N H / f | corrected (metadata); the 3.9 unverified | metadata |
| formations.rs | Price and Rind 1992, JGR 97, 9919, doi 10.1029/92JD00719 | F = 3.44e-5 H^4.9 land, 6.4e-4 H^1.73 sea | verified via a citing paper | |
| formations.rs | Yang, Cowan and Abbot 2013, ApJL 771, L45; Showman and Polvani 2011, ApJ 738, 71; Zurek and Martin 1993, JGR 98, 3247; Roe 2005, Annu. Rev. 33, 645; Klein and Hartmann 1993, J. Climate 6, 1587 | qualitative rules | verified (existence and metadata) | slope of Klein and Hartmann unverified |
| giants.rs | Brueshaber, Sayanagi and Dowling 2019, Icarus 323, 46, doi 10.1016/j.icarus.2019.02.001; O'Neill, Emanuel and Flierl 2015, Nat. Geosci. 8, 523, doi 10.1038/ngeo2459 | Bu decides polar regime; a/L_d ≤ 30 single, ≥ 40 many | verified via citing works | Brueshaber and Sayanagi 2021 values used, DOI still to add |
| giants.rs | Adriani et al. 2018, Nature 555, 216; Godfrey 1988, Icarus 76, 335; Barbosa Aguiar et al. 2010, Icarus 206, 755; Morales-Juberías et al. 2015, ApJL 806, L18; Sánchez-Lavega et al. 2011, Nature 475, 71; Smith et al. 1989, Science 246, 1422; Vasavada and Showman 2005, Rep. Prog. Phys. 68, 1935 | ring counts, hexagon, wavenumber dependence, storm families | verified (metadata); some numbers from memory flagged in W3.md | |
| aerosols.rs | Mastin et al. 2009, JVGR 186, 10, doi 10.1016/j.jvolgeores.2009.01.008 | H = 2.00 V^0.241 | verified | figure caption |
| aerosols.rs | Morton, Taylor and Turner 1956, Proc. R. Soc. A 234, 1; McEwen and Soderblom 1983, Icarus 55, 191; Robock 2000, Rev. Geophys. 38, 191 | H ∝ MER^¼ N^-¾; Io vent speeds 0.5 and 1 km/s; sulphate e-folding about 1 year | verified (Robock and McEwen from abstracts; MTT via a citing review) | |
| settings (substances) | NIST Chemistry WebBook phase-change pages for water, CO₂, CH₄, NH₃, N₂ | triple points, latent heats | verified (read through a summarising fetch; spot-check welcome) | |
| zonal.rs | North 1975, J. Atmos. Sci. 32, 2033, doi 10.1175/1520-0469(1975)032<2033:TOEBCM>2.0.CO;2; Budyko 1969 (URL) | diffusive energy balance | verified | |
| sse-climate ebm.rs (W7a) | North and Coakley 1979, J. Atmos. Sci. 36, 1189 (URL; DOI not yet checked) | seasonal EBM with heat capacity C dT/dt, mixed layer about 50 m | unverified (abstract page only; DOI to add) | |
| sse-climate ebm.rs (W7c) | North, Mengel and Short 1983, J. Geophys. Res. 88 (C11), 6576, "Simple energy balance model resolving the seasons and the continents: application to the astronomical theory of the ice ages" (NTRS 19830061623; DOI not yet checked) | land and sea with their own heat capacities in a seasonal diffusive energy balance | unverified (record found by search; not read; no number taken from it) | the two-column coupling is our derivation from D |
| tides.json `sse.tides.ocean_range`, `sse.tides.period` (W-sea) | Murray and Dermott 1999, Solar System Dynamics, ch. 4, doi 10.1017/CBO9781139174817 | equilibrium tide H = 3/2 (m/M)(R/d)³ R from the degree-2 potential over surface gravity; semidiurnal period at twice the synodic frequency | verified (against the textbook result: Earth from the Moon 0.535 m, the Sun 0.46 of it, 12.42 h; tests) | γ, the diminishing factor, is a param left at 1 |
| ocean.json `sse.ocean.wind_waves` (W-sea) | US Army Corps of Engineers 2002, Coastal Engineering Manual EM 1110-2-1100 Part II ch. 2 (URL) | g Hm0/u*² = 4.13e-2 (gX/u*²)^½, g Tp/u* = 0.651 (gX/u*²)^⅓ (eq. II-2-36); fully developed 211.5 and 239.8 (eq. II-2-37); C_D = 0.001 (1.1 + 0.035 U10) | partly verified (the chapter's text read through a fetch that lost the superscripts; the coefficients match, the exponents are the well-known ones and give the manual's worked figures, 2.75 m at 10 m/s over 300 km) | a page-image check of the exponents is welcome |
| sse-climate ebm.rs (W7a) | North, Cahalan and Coakley 1981, Rev. Geophys. 19, 91 (URL to the PDF) | the seasonal and annual EBM forms, heat capacity of land and sea | unverified (URL listed; the form used is the standard seasonal EBM, not checked line by line against the PDF) | |
| sse-climate ebm.rs (W7a) | Siler, Roe and Armour 2018, J. Climate 31, 7481, doi 10.1175/JCLI-D-18-0081.1 | Hadley share w = exp(−x²/σ²), σ = 0.3; F_q = −F_HC L q / (h_T − h), h_T = 1.06 h(equator); eddy moisture diffusing down-gradient | verified (paper read) | σ's scaling to other worlds is ours (docs/fudges.md) |
| sse-climate ebm.rs (W7a) | Pierrehumbert 2010, Principles of Planetary Climate, doi 10.1017/CBO9780511780783 | ground seasonal skin I sqrt(P/π); energy-limited rain | textbook | chapter numbers unverified |
| sse-climate strengths.rs (W7b) | Mitchell and Vallis 2010, JGR 115, E12008 (arXiv 1008.1996; DOI 10.1029/2010JE003587 to check) | Ro_T = R T0 Δ_H/(2Ωa)², eq. 3.3b; super-rotation for Ro_T above about 1 | verified (full text via arXiv) | no strength curve in the paper |
| sse-climate strengths.rs (W7b) | Lindzen and Farrell 1980, J. Atmos. Sci. 37, 1648, doi 10.1175/1520-0469(1980)037<1648:ASARFT>2.0.CO;2 | σ = 0.31 f dU/dz / N | unverified (paper is a scan; factor from NCAR NCL, a citing source) | |
| sse-climate strengths.rs (W7b) | Komacek and Showman 2016, ApJ 821, 16 (arXiv 1601.00069) | eq. 31 drag and Coriolis branches; τ_wave = L/(N H) | verified (full text via arXiv; drag branch re-derived) | |
| sse-climate strengths.rs (W7b) | Koll and Abbot 2016, ApJ 825, 99 (arXiv 1605.01066) | τ_rad = c_p p/(g σ T³); τ_wave/τ_drag = C_D a/H | unverified (fetch rebuilt some equations) | |
| sse-climate strengths.rs (W7b) | Haqq-Misra et al. 2018, ApJ 852, 67 (arXiv 1710.00435) | eqs. 8 and 9, deformation and Rhines lengths over the radius | verified (full text via arXiv; the abstract swaps the Rhines regime, body followed) | |
| sse-climate strengths.rs (W7b) | Wordsworth 2015, ApJ 806, 180 (arXiv 1412.5575) | eq. 27 thin-radiator night temperature, κ = 5e-5 m²/kg | verified (full text; 70.4 K at 0.01 bar reproduced) | the frost-point comparison is ours |
| sse-climate strengths.rs (W7b) | Knudsen number λ = k T/(√2 π d² p) against the scale height | collisional edge | textbook (Chapman and Cowling) | no DOI-bearing source checked |
| sse-climate water.rs (W9) | Jennings et al. 2018, Nature Communications 9, 1148, doi 10.1038/s41467-018-03629-7 | 50 % rain-snow threshold averaging 1.0 °C (−0.4 to 2.4 °C at 95 % of 6883 stations); 0.8 °C lower per 10 % humidity | verified (article text; supplementary tables not read) | |
| sse-climate water.rs (W9) | Calov and Greve 2005, J. Glaciol. 51, 173, doi 10.3189/172756505781829601 | positive degree days with a normal temperature scatter σ; σ = 5 °C in its Greenland example | partly verified (equation rendered as an image; the closed form used is the standard expectation of max(T + ε, 0)) | |
| sse-climate water.rs (W9) | Braithwaite 1995, J. Glaciol. 41, 153, doi 10.3189/S0022143000017846 | degree-day factors about 8 mm d⁻¹ °C⁻¹ on ice; snow 36 to 44 % of ice | verified (abstract and summary of the factors section) | 3.2 for snow is our pick in that range |
| sse-climate water.rs (W9) | Zhang et al. 2004, Water Resour. Res. 40, W02502, doi 10.1029/2003WR002710; Fu 1981 (Sci. Atmos. Sin. 1, 23) | Fu's curve E/P = 1 + Φ − (1 + Φ^ω)^(1/ω), ω = 2.6 matches Budyko | verified through Greve et al. 2016, HESS 20, 2195 | |
| sse-climate water.rs (W9) | Kottek et al. 2006, Meteorol. Z. 15, 259, doi 10.1127/0941-2948/2006/0130 | Köppen-Geiger Table 1 (A Tmin ≥ 18 °C, C −3 to 18, D ≤ −3, E Tmax < 10, EF < 0; B by P < 10 Pth, BW ≤ 5 Pth; Pth eq. 2.1) | verified (full paper) | |
| sse-climate saltation.rs (W9) | Kok, Parteli, Michaels and Karam 2012, Rep. Prog. Phys. 75, 106901, doi 10.1088/0034-4885/75/10/106901 (arXiv 1201.4353) | Shao and Lu fluid threshold eq. 2.8, A_N = 0.111, γ 1.65e-4 to 5e-4 N/m (Kok and Renno 2.9e-4); law of the wall, κ 0.40, z0 = D/30; 10 minute averaging | partly verified (text truncated; eq. 2.8 parameters verbatim, form as summarised); Iversen and White 1982 not used because its coefficients could not be read | the review named Iversen and White; Shao and Lu is the form Kok et al. give whole |
| sse-climate saltation.rs (W9) | Kok 2010, PRL 104, 074502, doi 10.1103/PhysRevLett.104.074502 (arXiv 1002.1346) | impact/fluid threshold 0.82 on Earth, an order smaller on Mars; Weibull P(U > U_r) with k = 2; hysteresis | verified (full text) | |
| sse-climate saltation.rs (W9) | Telfer et al. 2018, Science 360, 992, doi 10.1126/science.aao2975 | Pluto: 1 Pa, about 45 K, g 0.62, methane ice 494 kg/m³, grains 210 to 310 µm, winds 1 to 10 m/s, lofting by subliming nitrogen; initiation needs winds orders of magnitude above Pluto's | verified (main text of the accepted manuscript; supplement not read) | |
| sse-climate saltation.rs (W9) | Lancaster 1988, J. Arid Environ. 14, 233 (read through Lancaster and Helm 2000, ESPL 25, 197, and Delgado-Fernandez et al.) | dune mobility M = W/(P/PE); < 50 inactive, > 200 fully active | partly verified (secondary sources; the original not read) | |
| sse-climate forcing.rs (W9 part 2) | Zeebe 2022 (arXiv 2207.04006) | eq. 1 ṡ = α(n·s)(s×n), eq. 2 cos ε = n·s, eqs. 3 to 5 α = K(κ+β), K = (3/2)E_d GM/(ω a³), κ = (1−e²)^(−3/2), β = g_L (a/R)³ m_L/M; Table 1 Earth Ψ0 = 50.384815″/yr, E_d = 0.00327381 | verified (text, first two thirds) | the journal reference not checked |
| sse-climate settings.rs, ebm.rs (W8 part 1, MP3) | Fray and Schmitt 2009, Planet. Space Sci. 57, 2053, doi 10.1016/j.pss.2009.09.011, via Lisse et al. 2021 SOM Table 2 | ln P[bar] = A0 + Σ A_i/T^i for N2 (α, β), CO (α, β), CO2 (two ranges), CH4, NH3, Ar; water by Feistel and Wagner 2007 | verified (coefficients read from the SOM; each fit passes through its own triple point) | lower ends of the valid ranges from memory |
| sse-climate exchange.rs (W8 part 2) | Hess, S. L. et al. 1980, Geophys. Res. Lett. 7, 197: the annual cycle of pressure on Mars measured by Viking Landers 1 and 2 | Mars's pressure swings about 25 to 30 % with two minima, the deeper in southern winter (the review's test) | not read here (figure from the weather review) | from memory |
| sse-climate exchange.rs (W8 part 2) | Bertrand, T. and Forget, F. 2016, Nature 540, 86: observed glacier and volatile distribution on Pluto from atmosphere-topography processes | N2 ice collects at low latitudes on Pluto, where the year-averaged sunlight is least | not read here (named in the exogeology gaps design) | from memory |
| sse-climate exchange.rs (W8 part 2) | Bertrand et al. 2024 (named in the weather review) | the same volatile model with only the orbit changed gives Pluto low-latitude ice and Triton polar caps | not read here | reference to find |
| sse-climate forcing.rs (W9 part 2) | Huybers 2006, Science 313, 508, doi 10.1126/science.1125249 | summer energy J = Σ over days with insolation ≥ 275 W/m², at 65° N; early Pleistocene cycles at 40 kyr follow obliquity | verified (paper; SOM not read) | DOI from memory, to check |
| sse-kernel ice_ages.rs (W9 part 2) | Laplace-Lagrange secular theory (Murray and Dermott 1999, sections 7.2 to 7.4), via T1's `sse-time::secular` | e, ϖ, I, Ω through time | as T1 | |
| fixtures | NASA Planetary Fact Sheets, https://nssdc.gsfc.nasa.gov/planetary/factsheet/ | planet values | unverified (typed, not fetched) | |

## Ship manifest and crew (track MC)

| Where | Reference | What is relied on | Status | Checked |
|---|---|---|---|---|
| `packs/base-manifest` `life_support.rates.human` (MC1) | Anderson, M. S., Ewert, M. K. and Keener, J. F. 2018, Life Support Baseline Values and Assumptions Document, NASA/TP-2015-218570/REV1, https://ntrs.nasa.gov/citations/20180001338 | a nominal crew member uses 0.82 kg oxygen, about 2.5 kg drinking and food-preparation water and about 1.8 kg packaged food a day | unverified | figures from memory of the document's tables; the document was found but its tables were not read here |

## Small bodies (track SB)

| Where | Reference | What is relied on | Status | Checked |
|---|---|---|---|---|
| `packs/base-small-bodies` `pairs.yorp_contact` 0.11 (SB-benner08) | Benner, L. A. M. et al. 2008, Arecibo and Goldstone radar imaging of contact binary asteroids, Binary Asteroid Dynamics workshop, Paris (abstract) | about 11% (14 of 132) of near-Earth asteroids larger than 200 m imaged by radar are contact binaries | verified | abstract read 24 Nov 2026 (it also says binaries and contact binaries together exceed 25% above 200 m) |
| `pairs.yorp_separated` 0.16 (SB-margot02) | Margot, J. L. et al. 2002, Science 296, 1445 | about 16% of near-Earth asteroids larger than 200 m may be binary systems | verified | abstract and text read 24 Nov 2026 |
| `collisions.t0_rock_myr` 20, `t0_metal_myr` 1400 (SB-farinella98) | Farinella, P., Vokrouhlicky, D. and Hartmann, W. K. 1998, Icarus 132, 378, eqs. 14 and 15 | disruption lifetime about 2.0e7 yr (R / 1 m)^0.5 for stony bodies and 1.4e9 yr for iron ones | partly verified | read from a text extraction of the PDF; the strength term inside the bracket was unclear, recheck |
| `collisions.gravity_transition_km` 0.3 | Benz, W. and Asphaug, E. 1999, Icarus 142, 5 | the disruption energy Q*_D is lowest near 0.3 km; larger bodies are held by gravity | unverified | from memory |
| `collisions.main_belt_over_1km` 1.2e6 | Tedesco, E. F. and Desert, F.-X. 2002, AJ 123, 2070 | about 1.2 million main-belt asteroids larger than 1 km | unverified | from memory |
| `yorp.flat_below_hours` 2.2, spin barrier | Pravec, P. and Harris, A. W. 2000, Icarus 148, 12; Pravec, P. et al. 2008, Icarus 197, 497 | rubble piles over about 200 m do not spin faster than about 2.2 h; small asteroids' spin rates are nearly flat in frequency | unverified | from memory; the barrier itself is computed as sqrt(3 pi / (G rho)) |
| `pairs.cold_contact` 0.15, `cold_separated` 0.3 | Noll, K. S. et al. 2008 (The Solar System Beyond Neptune, 345); Thirouin, A. and Sheppard, S. S. 2019, AJ 157, 228 | about 30% of cold classical Kuiper belt objects are resolved binaries; perhaps 10 to 25% are contact binaries | unverified | from memory |
| `figure.cold_lobe_c_over_a` 0.45 | Spencer, J. R. et al. 2020, Science 367, eaay3999; Keane, J. T. et al. 2022, JGR Planets 127 | Arrokoth's lobes are flattened, the larger about 20.6 x 19.9 x 9.4 km | unverified | from memory |
| figure.rs Maclaurin and Jacobi constants | Chandrasekhar, S. 1969, Ellipsoidal Figures of Equilibrium | bifurcation at Omega^2/(pi G rho) = 0.37423 (e = 0.81267), Maclaurin maximum 0.44933 (e = 0.92996), Jacobi end b/a 0.432, c/a 0.345 | verified by test | the Maclaurin relation is computed and checked against these values in `sse-small` tests |
| `figure.round_km_rock` 600, `round_km_ice` 400 | Lineweaver, C. H. and Norman, M. 2010, arXiv 1004.1091 | rocky bodies round above about 600 km, icy ones above about 400 km | unverified | from memory |
| `ice.survives_below_k` 150 | Schorghofer, N. 2008, ApJ 682, 697 | exposed water ice lasts the age of the Solar System only below about 145 to 150 K | unverified | from memory |
| `weathering.t_myr_at_1au` 1.0 | Vernazza, P. et al. 2009, Nature 458, 993 | space weathering darkens a fresh stony surface in about a million years | unverified | from memory |
| `types.albedo` | Mainzer, A. et al. 2011, ApJ 741, 90 (NEOWISE); DeMeo, F. E. and Carry, B. 2013, Icarus 226, 723 | typical geometric albedos by type | unverified | from memory |
| type mix check | DeMeo, F. E. and Carry, B. 2014, Nature 505, 629 | the main belt mixes types, with S inner and C outer by mass | unverified | used as a check, not an input |
| `figure.top_shape_share_of_barrier` 0.85 | Watanabe, S. et al. 2019, Science 364, 268 (Ryugu); Scheeres, D. J. et al. 2019, Nature Astronomy 3, 352 (Bennu) | the ridges formed near 3.5 to 3.6 h at about 1,190 kg/m3, where the cohesionless barrier is 3.03 h: about 0.85 of the barrier rate | verified (Consensus check, visual polish citation check section 14) | the 0.85 share is derived from those two numbers |
| top shapes from spin-up or reaccumulation | Walsh, K. J., Richardson, D. C. and Michel, P. 2008, Nature 454, 188; Michel, P. et al. 2020, Nature Communications 11, 2655; Ostro, S. J. et al. 2006, Science 314, 1276 (1999 KW4, about 2.8 h) | slow spin-up sheds mass from the equator; top shapes can also form by reaccumulation | verified (abstracts, citation check section 14) | |
| VP-R3 edge waves | Tiscareno, M. S. 2013, Planetary Rings, Planets, Stars and Stellar Systems vol. 3, arXiv:1112.3305 (crediting Cuzzi and Scargle 1985, ApJ 292, 276; Showalter et al. 1986, Icarus 66, 297) | edge-wave wavelength 3 pi x the moon's distance from the gap edge | verified (primary, citation check section 7) | |
| VP-R3 resonant edges | French, R. G. et al. 2023, Icarus 405, 115678; El Moutamid, M. et al. 2016, Icarus 279, 125 | the B ring's Mimas 2:1 edge distortion varies 4 to 71 km over 5.36 years; the A ring's Janus 7:6 pattern (12.8 km) only while Janus is on its inner leg | verified with corrections (citation check section 7) | |
| VP-R3 density waves | Tiscareno, M. S. et al. 2007, Icarus 189, 14, eq. 5 | wavenumber k(r) = D_L (r - r_L) / (2 pi G sigma0 r_L) | verified (primary, citation check section 7) | |
| VP-R3 resonant torque | Tajeddine, R., Nicholson, P. D., Longaretti, P.-Y., El Moutamid, M. and Burns, J. A. 2017, "What confines the rings of Saturn?", arXiv:1710.08462 | eq. 5 first-order torque T = 8.6 m^2 mu^2 Sigma n^2 a^4 (from Goldreich and Tremaine 1982, rough for low m); eq. 6 viscous torque 3 pi nu Sigma n a^2; viscosities 1 to 3e-3 m^2/s in the outer A and the B ring; Mimas 2:1 holds the B ring with more than ten times its viscous torque | verified (primary, 24 Nov 2026; journal not checked) | |
| SB4 sublimation balance | Cowan, J. J. and A'Hearn, M. F. 1979, Moon and Planets 21, 155 | free sublimation: absorbed sunlight = emission + flux x latent heat; subsolar and isothermal (fast rotator) limits | recalled, not read | |
| SB4 vapour curves | Fray, N. and Schmitt, B. 2009, Planetary and Space Science 57, 2053 (on the sheet as fray2009) | sublimation pressures of water, CO2 and CO ices, read from the sheet; latent heat from their slope | on the sheet | |
| SB4 67P check | Hansen, K. C. et al. 2016, MNRAS 462, S491 | 67P's water production near perihelion of order 1e28 molecules a second (the test's range is 1e27 to 1e29) | recalled, not read | |
| VP-R2 unlit side | Scharringhausen, B. R. and Nicholson, P. D. 2013, Icarus 226, 1275; Dollfus, A. and Brunier, S. 1982, Icarus 52, 516; Miller, K. E. et al. 2024, Space Sci. Rev. 220 | transmitted brightness peaks at intermediate optical depth (which ring is brightest depends on the Sun's and viewer's elevations); typical optical depths C 0.05 to 0.10, B 0.9 to over 5, Cassini Division 0.10 to 0.20, A about 0.5 | verified with caveat (citation check section 8) | |
| VP-R2 opposition surge | Hapke, B. 1986, Icarus 67, 264; French, R. G. et al. 2007, PASP 119, 623 | shadow-hiding surge B(g) = B0 / (1 + tan(g/2) / h); Saturn's rings' surge half-width about 0.1 degrees | checked, formula second-hand (citation check section 4) | |
| VP-R2 forward scatter | visual polish citation check (PR 464), correction to section 5.9 | ring and dust grains read a phase table per particle kind from particle size and the sheet's refractive index (ask K-e of the light thread), not the Jendersie and d'Eon fit, which holds only for water droplets of 5 to 50 micrometres | ruling 24 Nov | |
| SB4 comets | Womack, M., Sarid, G. and Wierzchos, K. 2017, PASP 129, 031001 (printed title "CO and Other Volatiles in Distantly Active Comets"); Kelley, M. S. P. et al. 2013, Icarus 225, 475; Borsovszky, J. et al. 2021, PNAS 118, e2113315118 | activity turns on near 3 au for water, 12 au for CO2, about 50 au for CO; C2 lives about 2 days at 1 au, so the coma is green and the tail is not | verified (citation check section 13) | do not present the Haser C2 scale length as lifetime x outflow speed |
