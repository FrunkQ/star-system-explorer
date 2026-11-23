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
| Real spaceflight (track RS) | [citations/spaceflight.md](citations/spaceflight.md) | 1 of 5 | 0 | none (test data for the flight log) |

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
