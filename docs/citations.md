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
| Area | File | Checked | Wrong | Physics changed |
|---|---|---|---|---|
| Stars | [citations/stars.md](citations/stars.md) | in progress | | |
| Orbits and formation | [citations/orbits-formation.md](citations/orbits-formation.md) | in progress | | |
| Interiors and heat | [citations/interiors-heat.md](citations/interiors-heat.md) | in progress | | |
| Surfaces | [citations/surfaces.md](citations/surfaces.md) | in progress | | |
| Climate and life | [citations/climate-life.md](citations/climate-life.md) | in progress | | |
| Galaxy and cosmology | [citations/galaxy-cosmology.md](citations/galaxy-cosmology.md) | in progress | | |
| Light, reference objects and design files | [citations/light-reference.md](citations/light-reference.md) | in progress | | |

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
| fixtures | NASA Planetary Fact Sheets, https://nssdc.gsfc.nasa.gov/planetary/factsheet/ | planet values | unverified (typed, not fetched) | |
