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
| Master data sheet (track MP) | [citations/matter-reference.md](citations/matter-reference.md) | 17 | 0 | none yet (sources for the sheet) |
| Light, reference objects and design files | [citations/light-reference.md](citations/light-reference.md) | 108 | 18 | planet disc brightness factor, Sizes tab star radii and temperatures |

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
| fixtures | NASA Planetary Fact Sheets, https://nssdc.gsfc.nasa.gov/planetary/factsheet/ | planet values | unverified (typed, not fetched) | |
