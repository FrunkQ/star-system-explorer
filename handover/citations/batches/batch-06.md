# Consensus batch 6: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-06.md` beside this file).

---

You are checking the references of a science-education program. For each numbered paper below, use Consensus
(search by title words, authors and year, or by DOI) to find the paper and read its abstract or text. For each:

1. Does the paper exist as cited? Give the full citation you found: authors, year, title, journal, volume, pages, DOI.
2. What is its key formula, number or finding that bears on the claim?
3. Does it say what we claim? Answer with exactly one status word:
   - `confirmed`: it exists as cited and supports the claim;
   - `wrong citation`: the paper exists but a detail is wrong (year, journal, DOI, author, equation number); say what;
   - `says something else`: the paper does not support the claim, or gives a different number or formula; say what it says;
   - `not found`: Consensus cannot find it (say what you searched).
   If you could only read the abstract and the claim is a detail inside the paper, say so in the note and use
   `confirmed` only when the abstract itself supports it; otherwise use `not found` with the note "abstract only".

Do not guess. Keep each note under 60 words.

Reply with one line per paper, in this exact form, and nothing between the lines:

`key | status | full citation found | key formula or figure | note`

Papers:

**dupont2017**: DuPont 2017, Kevlar aramid fiber technical guide, Table II-1
   What we rest on it:
   - …, aramid, pbo-fibre, carbon-nanotube (tensile_strength and density). Aramid read from DuPont 2017 Table II-1: Kevlar 49 yarn 3.0 GPa (2.1 MJ/kg), resin strand 3.6 GPa (2.5); the sheet holds the yarn. PBO (Zylon HM) 5.8 GPa / 1,560 = 3.7 MJ/kg, read from the Fi…
   - DuPont 2017, Kevlar aramid fiber technical guide, Table II-1: typical properties of Kevlar 29 and 49 yarns (Kevlar 49 straight-test yarn 3,000 MPa, resin-impregnated strand 3,600 MPa, density 1.44 g/cm3; typical values, not specifications)
   - DuPont 2017, Kevlar aramid fiber technical guide, Table II-1: typical properties of Kevlar 29 and 49 yarns (Kevlar 49 straight-test yarn 3,000 MPa, resin-impregnated strand 3,600 MPa, density 1.44 g/cm3; typical values, not specifications) https://www.r-g.de/w…

**bond2005**: Bond, Woodward and Midgley 2005, "The global distribution of ecosystems in a world without fire", New Phytologist. https://pesquisa.bvsalud.org/portal/resource/pt/mdl-15720663
   What we rest on it:
   - //! twice the area it does (Bond, Woodward and Midgley 2005), so the
   - /// (annual burning in mesic savannah, Bond et al. 2005; fudge F33).
   - …the world's savannah and grassland exists because it burns *[demonstrated in a model: Bond, Woodward and Midgley 2005]*. So Earth's map cannot be right unless section 7's fire is feeding the disturbance term here. The oxygen fix and the biomes are one piece of…

**bond2017**: Bond et al. 2017, ApJ 840, 70 (0.008098 solar radii, Teff 25369 K)
   What we rest on it:
   - Bond et al. 2017, ApJ 840, 70 (0.008098 solar radii, Teff 25369 K)
   - Sirius A, 2.06 M☉, 0.24 Gyr (Liebert et al. 2005; Bond et al. 2017: L 25.4 L☉, R 1.71 R☉). The radius relation is for a typical-age star; young Sirius is 14 percent smaller, a known limit of the first wave

**bullock2001**: Bullock et al. 2001
   What we rest on it:
   - //! - spin: lognormal, median 0.035, sigma ln = 0.5 (Bullock et al. 2001);
   - - **Spin**: lognormal, median 0.035, width sigma(ln lambda) = 0.5 (Bullock et al. 2001; natural log, about 0.22 dex). Sets disc size later.
   - …(paper I, MNRAS 450, 1514, doi:10.1093/mnras/stv689, eqs. 14, 15 and 20 to 23); spin Bullock et al. 2001; major mergers Fakhouri, Ma and Boylan-Kolchin 2010; formation scatter and environment shift are rough (sketch)

**chapman1931**: Chapman, S. and Ferraro, V. C. A. 1931, A new theory of magnetic storms, Terrestrial Magnetism and Atmospheric Electricity 36, 77 (pressure balance of a compressed dipole against the solar wind). doi:10.1029/TE036i002p00077
   DOI as cited: 10.1029/te036i002p00077
   What we rest on it:
   - /// B = μ₀M/4πr³ (Chapman and Ferraro 1931; Earth about 10 radii); belts need
   - * The wind's squeeze on the field (Chapman and Ferraro 1931): the currents on the bubble's sunward face act like an
   - "The wind's squeeze as an image dipole: Chapman, S. and Ferraro, V. C. A. 1931, A new theory of magnetic storms, Terr. Magn. Atmos. Electr. 36, 77, doi:10.1029/TE036i002p00077; Mead, G. D. 1964, Deformation of the geomagnetic field by the solar wind, J. Geophy…

**chiappini1997**: Chiappini et al. 1997, 2001
   What we rest on it:
   - //! particles (Matteucci and Francois 1989; Chiappini et al. 1997, 2001).
   - …24, log-spaced). Classic multi-zone chemical evolution (Matteucci and Francois 1989; Chiappini et al. 1997), with no particles.
   - Multi-zone chemical evolution (Matteucci and Francois 1989; Chiappini et al. 1997, 2001); Kennicutt 1998 star formation with a Martin and Kennicutt 2001 threshold; Ia rate Maoz and Mannucci 2012 and Maoz and Graur 2017; solar mass fractions Asplund et al. 2009…

**chiavassa2010**: Chiavassa, A. et al. (2010). Radiative hydrodynamics simulations of red supergiant stars II. A&A 515, A12 (Betelgeuse
   DOI as cited: 10.1051/0004-6361/200913907
   What we rest on it:
   - /// 2002; Chiavassa et al. 2010); none above the granulation boundary (Gray
   - { text: "Chiavassa, A. et al. (2010). Radiative hydrodynamics simulations of red supergiant stars II. A&A 515, A12 (Betelgeuse: a few giant convective cells and an uneven limb).", doi: "10.1051/0004-6361/200913907" },
   - { text: "Chiavassa, A. et al. (2010). Radiative hydrodynamics simulations of red supergiant stars II. A&A 515, A12 (Betelgeuse: a few giant convective cells).", doi: "10.1051/0004-6361/200913907" },

**dave2012**: Dave, Finlator and Oppenheimer 2012
   What we rest on it:
   - //! al. 2010; Lilly et al. 2013; Dave et al. 2012).
   - //! falls in at all (Dave, Finlator and Oppenheimer 2012), which lets small
   - /// Preventive feedback: inflow times 1 / (1 + (v_c / v_prev)^-k) (Dave, Finlator and Oppenheimer 2012).

**doi:10.0000/example**: 10.0000/example
   DOI as cited: 10.0000/example
   What we rest on it:
   - {"id": "fit", "text": "A made-up fit", "doi": "10.0000/example"}
   - 10.0000/example
   - A made-up fit doi:10.0000/example

**doi:10.1175/1520-0469(1975)032**: 10.1175/1520-0469(1975)032
   DOI as cited: 10.1175/1520-0469(1975)032
   What we rest on it:
   - | North 1975, J. Atmos. Sci. 32, 2033, doi:10.1175/1520-0469(1975)032<2033:TOEBCM>2.0.CO;2 | energy balance core | pack |
   - 10.1175/1520-0469(1975)032<2033:TOEBCM>2.0.CO;2
   - North, G. R. 1975, Theory of energy-balance climate models, J. Atmos. Sci. 32, 2033-2043 doi:10.1175/1520-0469(1975)032<2033:TOEBCM>2.0.CO;2

**dole1970**: Dole, S. H. 1970, Icarus 13, 494
   DOI as cited: 10.1016/0019-1035(70)90036-9
   What we rest on it:
   - /// The corrected rule: Dole 1970 and Fogg 1985 as published (the three
   - /// Dole 1970's dust density at the centre for the Sun, and Fogg 1985's scaling with the star.
   - "Dole, S. H. 1970, Icarus 13, 494: computer simulation of the formation of planetary systems";

**duncan1987**: Duncan, Quinn and Tremaine 1987, AJ 94, 1330 (Oort cloud, leftovers.rs)
   What we rest on it:
   - pub const REF_OORT: &str = "Duncan, M., Quinn, T. and Tremaine, S. 1987, AJ 94, 1330: the Oort cloud spans about 3,000 to 100,000 AU, isotropic, thermal eccentricities";
   - …Oort cloud as isotropic (`Tilt::Isotropic` in `sse-formation/src/leftovers.rs`, from Duncan, Quinn and Tremaine 1987) and the scattered disc as a thick disc. The chart room draws every cloud as a flat band, and the grown-system preview leaves the Oort cloud o…
   - Already in the repo: Duncan, Quinn and Tremaine 1987, AJ 94, 1330 (Oort cloud, `leftovers.rs`); Henry et al. 2018 (RECONS, G5 checks); Kroupa 2001 (IMF, G5).

**eddington1926**: Eddington, A. S. (1926). The Internal Constitution of the Stars
   What we rest on it:
   - | `sse.feed.eddington_rate` | feeding rate at which light balances gravity | Eddington 1926; textbook | settled |
   - | `sse.star.pulse_period` | period from mean density; overtone ratios | Ritter 1879; Eddington 1926; Cox 1980 | settled |
   - { text: "Eddington, A. S. (1926). The Internal Constitution of the Stars. Cambridge University Press (grey atmosphere limb darkening, I(mu)/I(1) = (2 + 3 mu) / 5).", url: "https://archive.org/details/internalconstitu0000eddi" },

**eker2018**: Eker, Z. et al. 2018, MNRAS 479, 5491
   DOI as cited: 10.1093/mnras/sty1834
   What we rest on it:
   - const EKER: &str = "Eker, Z. et al. 2018, MNRAS 479, 5491: interrelated main-sequence mass-luminosity, mass-radius and mass-effective temperature relations (six-piece MLR, table 4). doi:10.1093/mnras/sty1834";
   - /// Eker et al. 2018 six-piece mass-luminosity relation, (upper mass bound in
   - // Formula reproduction (Eker et al. 2018 table 4) and the Sun.

**fischer2005**: Fischer and Valenti 2005
   What we rest on it:
   - /// Above this, migrating giant planets dominate (Fischer and Valenti 2005; Lineweaver 2001).
   - …metallicity; giant planet frequency rises steeply with [Fe/H] and with stellar mass (Fischer and Valenti 2005); C/O and Mg/Si set rock chemistry; r-process and K-40 set radiogenic heat; Al-26 sets early water loss; cluster ultraviolet truncates outer discs; c…
   - …Gibson 2004) as editable criteria: [Fe/H] -0.5 to +0.3 (giant-planet dominance above, Fischer and Valenti 2005), at most 4 sterilising supernovae and 3 lethal bursts per Gyr (Earth has lived through several), 4 Gyr of quiet for complex life

**french2012**: French et al. 2012, doi:10.1088/0067-0049/202/1/5
   DOI as cited: 10.1088/0067-0049/202/1/5
   What we rest on it:
   - // 2012); Jupiter's deep fluid rises past 1e6 with depth (French 2012);
   - - Hydrogen and helium at depth: Chabrier, Mazevet and Soubiran 2019, doi:10.3847/1538-4357/aaf99f; conductivity along Jupiter's interior, French et al. 2012, doi:10.1088/0067-0049/202/1/5
   - French, M. et al. 2012, ApJS 202, 5: ab initio simulations for material properties along the Jupiter adiabat (model J11-8a), Table 2 (linear transport properties) with Table 1 (thermodynamic properties); section 6.2: metal-like conduction (sigma >= 2e4 S/m) fr…

**gehrels2003**: Gehrels et al. 2003, scaled from 8 pc
   What we rest on it:
   - // Supernovae within 10 pc: about 1 to 3 per Gyr (Gehrels et al. 2003, scaled from 8 pc).
   - …ugh Milky Way values); scale heights 100, 300 and 900 pc; sterilising distance 10 pc (Gehrels et al. 2003); lethal bursts within 2 kpc at low metallicity (Piran and Jimenez 2014; Graham and Fruchter 2013), burst yield tuned so the Sun now sees about 0.4 per Gy…
   - a supernova within 8 pc can strip an ozone layer (Gehrels et al. 2003)

**hahn2007**: Hahn et al. 2007 use 0
   What we rest on it:
   - //! a threshold (the T-web, Hahn et al. 2007).
   - /// T-web eigenvalue threshold (Hahn et al. 2007 use 0), used when
   - …on). Count above a threshold gives void, sheet, filament or knot (the "T-web" method, Hahn et al. 2007).

**halliday2013**: Halliday, D., Resnick, R. and Walker, J. 2013, Fundamentals of Physics, 10th edn, Wiley, chapter 7. https://www.wiley.com/en-us/Fundamentals+of+Physics%2C+10th+Edition-p-9781118230718
   What we rest on it:
   - Computed (SSE4): energy is half the mass times the closing speed squared, spread over the projectile's face; the textbook kinetic energy. Halliday, D., Resnick, R. and Walker, J. 2013, Fundamentals of Physics, 10th edn, Wiley, chapter 7.
   - …losing speed squared, spread over the projectile's face; the textbook kinetic energy. Halliday, D., Resnick, R. and Walker, J. 2013, Fundamentals of Physics, 10th edn, Wiley, chapter 7. https://www.wiley.com/en-us/Fundamentals+of+Physics%2C+10th+Edition-p-9781…
   - Computed: kinetic energy (Halliday, Resnick and Walker 2013, chapter 7)

**henry2018**: Henry et al. 2018 (RECONS, G5 checks)
   What we rest on it:
   - // RECONS 10 pc census (Henry et al. 2018): 317 systems; ours from G4's 0.099 per pc^3.
   - // About 75% of nearby stars are M dwarfs (Henry et al. 2018); a Kroupa IMF
   - Already in the repo: Duncan, Quinn and Tremaine 1987, AJ 94, 1330 (Oort cloud, `leftovers.rs`); Henry et al. 2018 (RECONS, G5 checks); Kroupa 2001 (IMF, G5).

**hoehler2004**: Hoehler 2004, "Biological energy requirements as quantitative boundary conditions for life in the subsurface", Geobiology. https://www.scilit.net/publications/9add82f897d687ce5fbcbf93c6764e8d
   What we rest on it:
   - …J per mole *[observed in syntrophic microbes: Schink 1997; lower values are argued in Hoehler 2004]*. The number is recalled, not re-read; check it at E0.
   - - Hoehler 2004, "Biological energy requirements as quantitative boundary conditions for life in the subsurface", Geobiology. https://www.scilit.net/publications/9add82f897d687ce5fbcbf93c6764e8d
   - Schink, B. 1997, Microbiology and Molecular Biology Reviews 61, 262: energetics of syntrophic cooperation in methanogenic degradation (about -20 kJ per mol per turnover). https://doi.org/10.1128/mmbr.61.2.262-280.1997 (recalled; Hoehler 2004 argues lower)

**hoehler2013**: Hoehler and Jørgensen 2013, "Microbial life under extreme energy limitation", Nature Reviews Microbiology. doi:10.1038/nrmicro2939
   DOI as cited: 10.1038/nrmicro2939
   What we rest on it:
   - …ower does not mean no life; it means very little life, living very slowly *[observed: Hoehler and Jørgensen 2013; Bradley et al. 2020]*.
   - - Hoehler and Jørgensen 2013, "Microbial life under extreme energy limitation", Nature Reviews Microbiology. doi:10.1038/nrmicro2939
   - …t about 56 Gt C per year); deep subsurface cells turn over in centuries to millennia (Hoehler and Jorgensen 2013)

**hogg1999**: Hogg, D. W. 1999, Distance measures in cosmology, arXiv:astro-ph/9905116
   arXiv as cited: astro-ph/9905116
   What we rest on it:
   - /// c / H0 times the integral of da / (a² E(a)) from a to 1 (Hogg 1999,
   - //! O7 shift 1/(1+z), with the luminosity distance (1+z) D_M, Hogg 1999) and
   - pub const R_HOGG99: &str = "Hogg, D. W. 1999, Distance measures in cosmology, arXiv:astro-ph/9905116: flux from luminosity with the luminosity distance d_L = (1 + z) D_M";

**holmberg2009**: Holmberg, Nordstrom and Andersen 2009 section 6.3, total dispersion
   arXiv as cited: 0811.3982
   What we rest on it:
   - // survey: mean -0.14 (Nordstrom et al. 2004; Holmberg et al. 2009) to
   - …ons Raghavan et al. 2010 (log a about 1.7 +- 1.5 au); dispersions rising as age^0.40 (Holmberg, Nordstrom and Andersen 2009 section 6.3, total dispersion); midplane density 0.1 solar masses per pc^3 (McKee et al. 2015); cluster mass function M^-2 (Lada and Lad…
   - velocity dispersion proportional to age^0.40, about 50 km/s at 10 Gyr (Holmberg, Nordstrom and Andersen 2009 section 6.3: exponents 0.39 U, 0.40 V, 0.53 W, 0.40 total; arXiv 0811.3982)

**holsapple1993**: Holsapple, K. A. 1993, Annual Review of Earth and Planetary Sciences 21, 333
   DOI as cited: 10.1146/annurev.ea.21.050193.002001
   What we rest on it:
   - "Holsapple, K. A. 1993, Annual Review of Earth and Planetary Sciences 21, 333: gravity-regime crater scaling, D proportional to g^-0.22 for rock. doi:10.1146/annurev.ea.21.050193.002001",
   - - **Size**: S4's crater scaling (Holsapple 1993) already takes impactor density and speed. So a dense iron fragment and a fluffy comet of the same mass make different craters.
   - …-1.83 equilibrium; documented fudge (docs/fudges.md); crater size scales as g^-0.216 (Holsapple 1993, doi:10.1146/annurev.ea.21.050193.002001); depth and rim after Pike 1977, transitions 15, 150 and 600 km at lunar gravity scaled as 1/g (Melosh 1989); atmosphe…

**izidoro2017**: Izidoro, A. et al. 2017, MNRAS 470, 1750
   What we rest on it:
   - pub const REF_IZIDORO17: &str = "Izidoro, A. et al. 2017, MNRAS 470, 1750; Izidoro et al. 2021, A&A 650, A152: migration piles super-Earths into resonant chains at the disc's inner edge";
   - pub const REF_IZIDORO: &str = "Izidoro, A. et al. 2017, MNRAS 470, 1750; 2021, A&A 650, A152: about 90 to 95 percent of resonant chains break after the gas leaves, mostly within 100 Myr";
   - // Chains: nearly all break (Izidoro et al. 2017, 2021).

**kalirai2008**: Kalirai et al. 2008 for white
   What we rest on it:
   - /// Initial-final mass relation, white dwarf = wd_a x M + wd_b (Kalirai et al. 2008).
   - //! giant phase, remnants by initial mass: Kalirai et al. 2008 for white
   - /// White dwarf mass = wd_a x M + wd_b (Kalirai et al. 2008).

**kasten1989**: Kasten, F., Young, A. T. (1989). Revised optical air mass tables and approximation formula
   DOI as cited: 10.1364/ao.28.004735
   What we rest on it:
   - /** Kasten and Young (1989) relative air mass for a cosine of the zenith angle (cos > -0.08; else a large number). */
   - // the star through the air (Kasten and Young 1989 air mass for the sun's
   - // Kasten and Young (1989): air mass = 1 / (cos z + 0.50572 (96.07995 - z)^-1.6364), z in degrees.

**kennicutt1998**: Kennicutt 1998, ApJ 498, 541
   What we rest on it:
   - //! (Kennicutt 1998), dropping steeply below a threshold density
   - // star formation the gas allows (Kennicutt 1998), dust in the gas dims the visible light, and each band reads the
   - /** Star formation from gas: Σ_SFR = a Σ_gas^n, M☉/yr/kpc² from M☉/pc² (Kennicutt 1998, ApJ 498, 541). */

**kita2013**: Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383
   DOI as cited: 10.1111/maps.12141
   What we rest on it:
   - const KITA13: &str = "Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383: 26Al-26Mg isotope systematics of the first solids in the early solar system (canonical 26Al/27Al about 5.2e-5). doi:10.1111/maps.12141";
   - /// 5.25e-5 (Kita et al. 2013) × 0.3583 W per kg of Al-26 (Ruedas 2017, table 2).
   - Kita, N. T. et al. 2013, Meteoritics and Planetary Science 48, 1383: 26Al-26Mg isotope systematics of the first solids in the early solar system

**kivelson1998**: Kivelson, M. G. et al. 1998, J. Geophys. Res. 103, 19963 doi:10.1029/98JE00227
   DOI as cited: 10.1029/98je00227
   What we rest on it:
   - const KIV98: &str = "Kivelson, M. G. et al. 1998, Ganymede's magnetosphere: magnetometer overview, J. Geophys. Res. 103, 19963: a moon's own field stands off its host's field and plasma about two moon radii out. doi:10.1029/98JE00227";
   - /// two Ganymede radii out (Kivelson et al. 1998); Titan at 20.3 Saturn radii
   - Kivelson, M. G. et al. 1998, J. Geophys. Res. 103, 19963

**kok2012**: Kok, Parteli, Michaels and Karam 2012, The physics of wind-blown sand and dust, Reports on Progress in Physics 75, 106901 doi:10.1088/0034-4885/75/10/106901
   DOI as cited: 10.1088/0034-4885/75/10/106901
   What we rest on it:
   - //! Lu 2000 as given by Kok et al. 2012 (eq. 2.8):
   - //! U = (u*/κ) ln(z/z0), with z0 = D/30 (Kok et al. 2012).
   - /// Shao and Lu 2000 (Kok et al. 2012 eq. 2.8): u*ft = A_N sqrt(...), and the cohesion γ, N/m.

**liebert2005**: Liebert, Bergeron and Holberg 2005
   What we rest on it:
   - // (Liebert, Bergeron and Holberg 2005); within a factor of two.
   - Sirius A, 2.06 M☉, 0.24 Gyr (Liebert et al. 2005; Bond et al. 2017: L 25.4 L☉, R 1.71 R☉). The radius relation is for a typical-age star; young Sirius is 14 percent smaller, a known limit of the first wave

**lilly2013**: Lilly et al. 2013
   What we rest on it:
   - //! al. 2010; Lilly et al. 2013; Dave et al. 2012).
   - A small system of rate equations per galaxy, stepped on a fixed grid (default 10 Myr, about 1,400 steps for the whole of history; cost is trivial). This is the "bathtub" or "regulator" approach (Bouché et al. 2010; Lilly et al. 2013; Davé et al. 2012).
   - Regulator model (Bouche et al. 2010; Lilly et al. 2013; Dave, Finlator and Oppenheimer 2012); depletion times Tacconi et al. 2018 and Leroy et al. 2008; winds Muratov et al. 2015; fallback Henriques et al. 2013; hot haloes and radio-mode feedback Croton et al.…

**lopez2014**: Lopez, E. D. and Fortney, J. J. 2014, ApJ 792, 1, eq. 3
   What we rest on it:
   - "Lopez, E. D. and Fortney, J. J. 2014, ApJ 792, 1, eq. 3: radius of a hydrogen envelope";
   - /// Lopez and Fortney 2014 envelope thickness, Earth radii (solar metallicity,
   - // Lopez and Fortney 2014 eq. 3 on a 5 Earth-mass core at 10 S_earth and 5 Gyr:

**millot2018**: Millot et al. 2018, doi:10.1038/s41567-017-0017-4, and
   DOI as cited: 10.1038/s41567-017-0017-4
   What we rest on it:
   - …zzoni et al. 1999, doi:10.1126/science.283.5398.44 (which also treats ammonia); seen, Millot et al. 2018, doi:10.1038/s41567-017-0017-4, and Millot et al. 2019, doi:10.1038/s41586-019-1114-6; its conductivity, French, Mattsson and Redmer 2010, doi:10.1103/Phys…
   - 3250 K at 100 GPa is Cheng et al.'s; 5000 K at 190 GPa is recalled from Millot et al. 2018 and must be checked; beyond 190 GPa extrapolated
   - Millot, M. et al. 2018, Nature Physics 14, 297: experimental evidence for superionic water ice using shock compression. Not read; its melting point near 5000 K at 190 GPa is recalled

**nishita1993**: Nishita, T., Sirai, T., Tadamura, K., Nakamae, E. (1993). Display of the Earth taking into account atmospheric scattering. SIGGRAPH 93, 175-182.", doi
   DOI as cited: 10.1145/166117.166140
   What we rest on it:
   - { text: "Nishita, T., Sirai, T., Tadamura, K., Nakamae, E. (1993). Display of the Earth taking into account atmospheric scattering. SIGGRAPH 93, 175-182.", doi: "10.1145/166117.166140" },
   - // Nishita et al. (1993). Per pixel it marches the view ray from the shell to

**pearl1991**: Pearl, J. C. and Conrath, B. J. 1991, The albedo, effective temperature, and energy balance of Neptune, as determined from Voyager data, J. Geophys. Res. 96, 18921 https://ui.adsabs.harvard.edu/abs/1991JGR....9618921P"
   What we rest on it:
   - const PC91: &str = "Pearl, J. C. and Conrath, B. J. 1991, The albedo, effective temperature, and energy balance of Neptune, as determined from Voyager data, J. Geophys. Res. 96, 18921 https://ui.adsabs.harvard.edu/abs/1991JGR....9618921P";
   - …he effective temperature Voyager and Cassini measured for the giants (Neptune 59.3 K, Pearl and Conrath 1991). The same block gives a star its effective temperature when the internal flux is its own luminosity over its surface.",
   - Neptune (Pearl and Conrath 1991: Bond albedo 0.290, internal heat 0.433 W/m², effective temperature 59.3 ± 0.8 K)

**prentice1992**: Prentice et al. 1992, "A global biome model based on plant physiology and dominance, soil properties and climate", Journal of Biogeography. https://hal-amu.archives-ouvertes.fr/hal-01788308
   What we rest on it:
   - //! The steps follow BIOME (Prentice et al. 1992), the first physiology-based
   - /// (after Prentice et al. 1992's woodland and forest limits).
   - …for a long time (Holdridge 1947), and the first physiology-based global model, BIOME (Prentice et al. 1992), is the direct comparison. Target, to be confirmed against a named land-cover dataset at build: the right broad type over at least 70 percent of land ar…

**roden2011**: Roden and Jin 2011, "Thermodynamics of microbial growth coupled to metabolism of glucose, ethanol, short-chain organic acids, and hydrogen", Applied and Environmental Microbiology. https://pmc.ncbi.nlm.nih.gov/articles/PMC3067288 (page blocked
   What we rest on it:
   - …tep with the energy the reaction releases, across very different microbes *[observed: Roden and Jin 2011; I confirmed the paper but could not read its regression, so the slope is to be taken from the paper at E1]*. The yield is therefore one number per chemist…
   - - Roden and Jin 2011, "Thermodynamics of microbial growth coupled to metabolism of glucose, ethanol, short-chain organic acids, and hydrogen", Applied and Environmental Microbiology. https://pmc.ncbi.nlm.nih.gov/articles/PMC3067288 (page blocked; regression no…
   - …ut 0.1 C per H2 aerobically, 0.01 C per H2 for methanogens; McCollom and Amend 2005). Roden and Jin 2011 find yield rises in step with energy, which this assumes; their slope is to be read (fudge register)

**rodriguezgomez2016**: Rodriguez-Gomez et al. 2016
   What we rest on it:
   - /// galaxies are mostly built this way (Rodriguez-Gomez et al. 2016).
   - …ass and metals against biggest progenitor; share of stars not born here against mass (Rodriguez-Gomez et al. 2016, used in G3); a Gaia-Sausage-like history gives a thick disc and two chemical families; G3 goldens re-blessed on purpose |
   - …es et al. 2013; hot haloes and radio-mode feedback Croton et al. 2006; ex-situ growth Rodriguez-Gomez et al. 2016; ex-situ black holes at 1e-3 of the stars they arrive with (Kormendy and Ho 2013, https://doi.org/10.1146/annurev-astro-082708-101811; Reines and…

**schink1997**: Schink 1997, "Energetics of syntrophic cooperation in methanogenic degradation", Microbiology and Molecular Biology Reviews. https://pmc.ncbi.nlm.nih.gov/articles/PMC232610
   DOI as cited: 10.1128/mmbr.61.2.262-280.1997
   What we rest on it:
   - …hinery can run on it. Default about 20 kJ per mole *[observed in syntrophic microbes: Schink 1997; lower values are argued in Hoehler 2004]*. The number is recalled, not re-read; check it at E0.
   - - Schink 1997, "Energetics of syntrophic cooperation in methanogenic degradation", Microbiology and Molecular Biology Reviews. https://pmc.ncbi.nlm.nih.gov/articles/PMC232610
   - Schink, B. 1997, Microbiology and Molecular Biology Reviews 61, 262: energetics of syntrophic cooperation in methanogenic degradation (about -20 kJ per mol per turnover). https://doi.org/10.1128/mmbr.61.2.262-280.1997 (recalled; Hoehler 2004 argues lower)

**strous1999**: Strous et al. 1999, "Missing lithotroph identified as new planctomycete", Nature 400. doi:10.1038/22749
   DOI as cited: 10.1038/22749
   What we rest on it:
   - …he ammonium one was found in the 1990s and identified in 1999 *[observed: Broda 1977; Strous et al. 1999]*. On Earth, almost every redox pair that clears the thresholds has something living on it. That is the empirical licence for generating metabolisms from t…
   - - Strous et al. 1999, "Missing lithotroph identified as new planctomycete", Nature 400. doi:10.1038/22749
   - Strous, M. et al. 1999, Nature 400, 446: missing lithotroph identified as new planctomycete (anammox, -357 kJ per mol). https://doi.org/10.1038/22749

**thorngren2016**: Thorngren, D. P., Fortney, J. J., Murray-Clay, R. A. and Lopez, E. D. 2016, ApJ 831, 64
   DOI as cited: 10.3847/0004-637x/831/1/64
   What we rest on it:
   - pub const TF16: &str = "Thorngren, D. P., Fortney, J. J., Murray-Clay, R. A. and Lopez, E. D. 2016, ApJ 831, 64: the mass-metallicity relation for giant planets (Z_planet / Z_star = 9.7 (M / M_Jup)^-0.45). doi:10.3847/0004-637X/831/1/64";
   - /// (Thorngren et al. 2016): Jupiter about 14% heavy, Neptune far more.
   - how many times its star's metal share a Jupiter-mass giant holds (Thorngren et al. 2016: 9.7 plus or minus 1.3)

**wade2005**: Wade, J. and Wood, B. J. 2005, EPSL 236, 78 doi:10.1016/j.epsl.2005.05.017
   DOI as cited: 10.1016/j.epsl.2005.05.017
   What we rest on it:
   - pub const WW05: &str = "Wade, J. and Wood, B. J. 2005, EPSL 236, 78: core formation and the oxidation state of the Earth; ΔIW = 2 log10(x_FeO / x_Fe) between silicate and metal. doi:10.1016/j.epsl.2005.05.017";
   - // Earth: about IW-2.2 at core formation (Wade and Wood 2005), IW+0.5 at
   - activity coefficient of FeO in the melt over that of iron in the metal (1: ideal, as Wade and Wood 2005)

**williams2005**: Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing '2001
   What we rest on it:
   - // Discovery II's two settings are one 4,830 MW jet (Williams 2005, Table 2).
   - Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing '2001: A Space Odyssey': piloted spherical torus nuclear fusion propulsion, NASA/TM-2005-213559 (AIAA 2001-3805): D-He3, specific impulse 35,435 s (Jupiter mission), jet power…
   - Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing '2001: A Space Odyssey': piloted spherical torus nuclear fusion propulsion, NASA/TM-2005-213559 (AIAA-2001-3805), Tables 1 and 2. https://ntrs.nasa.gov/api/citations/200501609…

**wright2004**: Wright et al. 2004, "The worldwide leaf economics spectrum", Nature 428. doi:10.1038/nature02403
   DOI as cited: 10.1038/nature02403
   What we rest on it:
   - def("tissue_economics", "nutrients", 0.25, 0.1, "0 slow and tough, 1 fast and cheap (Wright et al. 2004): fast where nutrients are rich"),
   - | Fast-cheap versus slow-tough tissue | The "leaf economics" axis: every land plant on Earth sits somewhere on one line from flimsy fast-growing to durable slow-growing *[observed: Wright et al. 2004]* |
   - - Wright et al. 2004, "The worldwide leaf economics spectrum", Nature 428. doi:10.1038/nature02403

**adams2003**: Adams, R. B. et al. 2003, Crewed mission to Callisto using advanced plasma propulsion systems, AIAA 2003-4527 (NASA HOPE study), Tables 1 to 8. https://ntrs.nasa.gov/api/citations/20030065984/downloads/20030065984_Update.pdf
   What we rest on it:
   - Adams, R. B. et al. 2003, Conceptual design of in-space vehicles for human exploration of the outer planets, AIAA 2003-4527 (NTRS 20030065984): D-He3 magnetised target fusion, specific impulse 77,000 s
   - Adams, R. B. et al. 2003, Crewed mission to Callisto using advanced plasma propulsion systems, AIAA 2003-4527 (NASA HOPE study), Tables 1 to 8. https://ntrs.nasa.gov/api/citations/20030065984/downloads/20030065984_Update.pdf

**aguiar2010**: Aguiar, A. C. et al. 2010, A laboratory model of Saturn's North Polar Hexagon, Icarus 206, 755-763 doi:10.1016/j.icarus.2009.10.022
   DOI as cited: 10.1016/j.icarus.2009.10.022
   What we rest on it:
   - …ides depends on the jet's width and strength relative to its latitude circle: Barbosa Aguiar et al. 2010, *Icarus* 206, 755, "A laboratory model of Saturn's North Polar Hexagon" **[existence checked]**; Morales-Juberías et al., "Meandering shallow atmospheric…
   - …es is set by the deformation radius against the jet (laboratory wavenumber 6: Barbosa Aguiar et al. 2010; numerical 3 to 8 sides with jet width, strength and curvature: Morales-Juberias et al. 2015). Here: a polygon only on a coherent circumpolar jet (single p…
   - Barbosa Aguiar, A. C. et al. 2010, A laboratory model of Saturn's North Polar Hexagon, Icarus 206, 755-763

**alken2021**: Alken, P. et al. 2021, Earth, Planets and Space 73, 49
   DOI as cited: 10.1186/s40623-020-01288-x
   What we rest on it:
   - const IGRF: &str = "Alken, P. et al. 2021, Earth, Planets and Space 73, 49: the International Geomagnetic Reference Field, 13th generation (Earth's dipole moment about 7.7e22 A m²). doi:10.1186/s40623-020-01288-x";
   - Alken, P. et al. 2021, International Geomagnetic Reference Field: the thirteenth generation, Earth Planets Space 73, 49: g10 negative (the dipole points south). doi:10.1186/s40623-020-01288-x
