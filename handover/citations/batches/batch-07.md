# Consensus batch 7: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-07.md` beside this file).

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

**amend2001**: Amend, J. P. and Shock, E. L. 2001, FEMS Microbiology Reviews 25, 175
   DOI as cited: 10.1111/j.1574-6976.2001.tb00576.x
   What we rest on it:
   - const AMEND: &str = "Amend, J. P. and Shock, E. L. 2001, FEMS Microbiology Reviews 25, 175: energetics of overall metabolic reactions of thermophilic and hyperthermophilic Archaea and Bacteria. doi:10.1111/j.1574-6976.2001.tb00576.x";
   - …es; beyond about 150 °C the heat-capacity terms of the full revised HKF model matter (Amend and Shock 2001 tabulate those).",
   - …es; beyond about 150 °C the heat-capacity terms of the full revised HKF model matter (Amend and Shock 2001 tabulate those).

**appleton1996**: Appleton and Struck-Marcell 1996
   What we rest on it:
   - …es, the culprit a little way off on the axis | 100 to 500 Myr (Lynds and Toomre 1976; Appleton and Struck-Marcell 1996) |
   - * and its middle is emptied by up to `hole` (Lynds and Toomre 1976; Appleton and Struck-Marcell 1996). */

**armstrong2019**: Armstrong, K., Frost, D. J., McCammon, C. A., Rubie, D. C. and Ballaran, T. B. 2019, Science 365, 903
   DOI as cited: 10.1126/science.aax8376
   What we rest on it:
   - pub const ARM19: &str = "Armstrong, K., Frost, D. J., McCammon, C. A., Rubie, D. C. and Ballaran, T. B. 2019, Science 365, 903: a deep magma ocean oxidises itself as FeO turns to Fe2O3 and metal under pressure. doi:10.1126/science.aax8376";
   - how far a deep magma ocean oxidises itself above its core-formation value, log10 units (Armstrong et al. 2019; 2.7 puts Earth at Sossi et al. 2020's IW+0.5)
   - …deep magma ocean then oxidises itself, ferrous iron turning to ferric under pressure (Armstrong et al. 2019), which lifts the surface value; one shift for every world (fudge register). Earth comes out near IW-2.2 at core formation and IW+0.5 at its magma ocean…

**atmosphere1976**: Atmosphere 1976, NOAA/NASA/USAF (sea-level dry-air composition by volume) https://ntrs.nasa.gov/citations/19770009539
   What we rest on it:
   - /// that a recipe leaves out (US Standard Atmosphere 1976 dry air lists
   - U.S. Standard Atmosphere 1976, NOAA/NASA/USAF (sea-level dry-air composition by volume)
   - U.S. Standard Atmosphere 1976, NOAA/NASA/USAF (sea-level dry-air composition by volume) https://ntrs.nasa.gov/citations/19770009539

**bailerjones2018**: Bailer-Jones et al. 2018 find about 20 within 1 pc per Myr
   What we rest on it:
   - // Passages within 0.1 pc: a few per Myr at most (Bailer-Jones et al. 2018 find about 20 within 1 pc per Myr).
   - // Stars passing within 1 pc: 19.7 per Myr (Bailer-Jones et al. 2018);

**barstow2005**: Barstow, M. A. et al. 2005, MNRAS 362, 1134 (80.4 ± 4.8 km/s)"
   What we rest on it:
   - pub const R_GR: &str = "Gravitational redshift of light leaving a static spherical mass: 1 + z = (1 − 2GM / r c²)^(-½) (Schwarzschild); tested on Sirius B by Barstow, M. A. et al. 2005, MNRAS 362, 1134 (80.4 ± 4.8 km/s)";
   - /// about 75 km/s; Barstow et al. 2005 measured 80.4 ± 4.8 km/s.

**belfiore2019**: Belfiore et al. 2019
   What we rest on it:
   - /// observed gradients need, Belfiore et al. 2019).
   - …2012 and Maoz and Graur 2017; solar mass fractions Asplund et al. 2009; radial winds Belfiore et al. 2019; inward drift Spitoni and Matteucci 2011; yields are IMF-integrated net yields in solar units, tuned (package log G3)

**bell2001**: Bell and de Jong 2001, ApJ 550, 212
   What we rest on it:
   - * Bell and de Jong 2001). */
   - /** Near-infrared (2.2 µm) light per solar mass, L☉/M☉ (mass-to-light near 1 in K: Bell and de Jong 2001), and of
   - "Bell and de Jong 2001, ApJ 550, 212: mass-to-light ratios",

**bell2014**: Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498
   DOI as cited: 10.1021/ie4033999
   What we rest on it:
   - Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498: pure and pseudo-pure fluid thermophysical property evaluation and the open-source thermophysical property library CoolProp (version 8.1.0 used, scratch only, to evaluate…
   - Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498: CoolProp, pure and pseudo-pure fluid property evaluation (MIT licence; its saturated-liquid density fits to the equations of state, dev/fluids/*.json)
   - Bell, I. H., Wronski, J., Quoilin, S. and Lemort, V. 2014, Ind. Eng. Chem. Res. 53, 2498: CoolProp, pure and pseudo-pure fluid property evaluation (MIT licence; its saturated-liquid density fits to the equations of state, dev/fluids/*.json) doi:10.1021/ie40339…

**benz2017**: Benz, A. O. (2017). Flare Observations
   DOI as cited: 10.1007/s41116-016-0004-3
   What we rest on it:
   - * time `decay_s` (the classic impulsive-gradual light curve, Benz 2017; the kernel gives the two times). Zero before
   - { text: "Benz, A. O. (2017). Flare Observations. Living Reviews in Solar Physics 14, 2 (the impulsive rise and gradual fall; white-light flare kernels near 10,000 K).", doi: "10.1007/s41116-016-0004-3" },

**berner1997**: Berner 1997's 4
   What we rest on it:
   - /// Land life multiplies silicate weathering by this (Berner 1997).
   - /// Weathering factor at today's Earth (Berner 1997's 4) and its cap.

**bouche2010**: Bouche et al. 2010
   What we rest on it:
   - A small system of rate equations per galaxy, stepped on a fixed grid (default 10 Myr, about 1,400 steps for the whole of history; cost is trivial). This is the "bathtub" or "regulator" approach (Bouché et al. 2010; Lilly et al. 2013; Davé et al. 2012).
   - Regulator model (Bouche et al. 2010; Lilly et al. 2013; Dave, Finlator and Oppenheimer 2012); depletion times Tacconi et al. 2018 and Leroy et al. 2008; winds Muratov et al. 2015; fallback Henriques et al. 2013; hot haloes and radio-mode feedback Croton et al.…

**buffett1996**: Buffett, B. A., Huppert, H. E., Lister, J. R. and Woods, A. W. 1996, J. Geophys. Res. 101, 7989 doi:10.1029/95JB03539
   DOI as cited: 10.1029/95jb03539
   What we rest on it:
   - const BR95: &str = "Buffett, B. A., Huppert, H. E., Lister, J. R. and Woods, A. W. 1996, Journal of Geophysical Research 101, 7989: on the thermal evolution of the Earth's core (light elements released at the inner core boundary as a buoyancy source). doi:10.1…
   - Buffett, B. A., Huppert, H. E., Lister, J. R. and Woods, A. W. 1996, J. Geophys. Res. 101, 7989
   - Buffett, B. A., Huppert, H. E., Lister, J. R. and Woods, A. W. 1996, J. Geophys. Res. 101, 7989 doi:10.1029/95JB03539

**calov2005**: Calov and Greve 2005, A semi-analytical solution for the positive degree-day model with stochastic temperature variations, Journal of Glaciology 51, 173 doi:10.3189/172756505781829601
   DOI as cited: 10.3189/172756505781829601
   What we rest on it:
   - //! Calov and Greve 2005), so a season's snow share is
   - //! (Calov and Greve 2005), melting snow at `ddf_snow_mm` and bare ice at
   - /// freezing) with scatter `s` (Calov and Greve 2005).

**canup2006**: Canup, R. M. and Ward, W. R. 2006, Nature 441, 834
   What we rest on it:
   - //! giant's circumplanetary disc (Canup and Ward 2006), and the large moons
   - pub const REF_CW06: &str = "Canup, R. M. and Ward, W. R. 2006, Nature 441, 834: regular satellite systems hold about 1e-4 of their planet's mass";
   - /// Regular satellites' share of a giant (Canup and Ward 2006).

**chambers1996**: Chambers, Wetherill and Boss 1996, and
   What we rest on it:
   - Gladman 1993: two planets on near-circular orbits can never meet when more than 2√3 mutual Hill radii apart. Chambers, Wetherill and Boss 1996, and Pu and Wu 2015: more than two planets need about 10 to last billions of years
   - Gladman 1993: two planets on near-circular orbits can never meet when more than 2√3 mutual Hill radii apart. Chambers, Wetherill and Boss 1996, and Pu and Wu 2015: more than two planets need about 10 to last billions of years doi:10.1006/icar.1993.1169
   - …two planets (Gladman 1993); multi-planet systems go unstable on times growing with Δ (Chambers et al. 1996), and Δ ≈ 10 survives Gyr (Pu and Wu 2015). The closest gap uses the inner apoapsis and the outer periapsis.

**chandrasekhar1960**: Chandrasekhar 1960 (thin layer, single scattering)
   What we rest on it:
   - | **Chandrasekhar 1960** (thin layer, single scattering) | Textbook | Rings lit from the front and seen from the dark side | lit side yes (`ringLight`); dark side to check | Extend |
   - * `n` the ring's normal). A layer of many particles scattering once (Chandrasekhar 1960; Cuzzi et al. 1984 for

**collaboration2025**: Collaboration 2025, Phys. Rev. D 112, 083535, doi:10.1103/3dzh-d8f5
   DOI as cited: 10.1103/3dzh-d8f5
   What we rest on it:
   - - The standard picture keeps passing new tests: counts of 16,000 galaxy clusters with lensing and clustering agree with the cosmic background's parameters (DES Collaboration 2025, Phys. Rev. D 112, 083535, doi:10.1103/3dzh-d8f5).
   - 10.1103/3dzh-d8f5
   - DES Collaboration (Abbott, T. M. C. et al.) 2025, Phys. Rev. D 112, 083535: Dark Energy Survey year 3, cosmological constraints from cluster abundances, weak lensing and galaxy clustering doi:10.1103/3dzh-d8f5

**croton2006**: Croton et al. 2006
   What we rest on it:
   - //! (radio-mode feedback, Croton et al. 2006), smoothly, with no threshold.
   - /// so a big black hole in a big halo shuts off the gas supply (Croton et al. 2006).
   - …ratov et al. 2015; fallback Henriques et al. 2013; hot haloes and radio-mode feedback Croton et al. 2006; ex-situ growth Rodriguez-Gomez et al. 2016; ex-situ black holes at 1e-3 of the stars they arrive with (Kormendy and Ho 2013, https://doi.org/10.1146/annur…

**curtis2014**: Curtis, H. D. 2014, Orbital Mechanics for Engineering Students, 3rd ed., Elsevier, algorithms 3.4 and 5.2. doi:10.1016/C2011-0-69685-1
   DOI as cited: 10.1016/c2011-0-69685-1
   What we rest on it:
   - //! References: Curtis 2014, Orbital Mechanics for Engineering Students, 3rd
   - Curtis, H. D. 2014, Orbital Mechanics for Engineering Students, 3rd ed., Elsevier, algorithms 3.4 and 5.2. doi:10.1016/C2011-0-69685-1

**demircan1991**: Demircan, O. and Kahraman, G. 1991, Ap&SS 181, 313
   What we rest on it:
   - const DK91: &str = "Demircan, O. and Kahraman, G. 1991, Ap&SS 181, 313: stellar mass-luminosity and mass-radius relations https://ui.adsabs.harvard.edu/abs/1991Ap%26SS.181..313D";
   - // Eker et al. 2018 six-piece MLR and Demircan and Kahraman 1991 radii,

**doi:10.1016/b978-0-08-095975-7.00201-1**: 10.1016/b978-0-08-095975-7.00201-1
   DOI as cited: 10.1016/b978-0-08-095975-7.00201-1
   What we rest on it:
   - pub const PO14: &str = "Palme, H. and O'Neill, H. St. C. 2014, Treatise on Geochemistry (2nd ed.) 3, 1: cosmochemical estimates of mantle composition (CI H and O). doi:10.1016/B978-0-08-095975-7.00201-1";
   - 10.1016/B978-0-08-095975-7.00201-1
   - Palme, H. and O'Neill, H. St. C. 2014, Cosmochemical estimates of mantle composition, Treatise on Geochemistry 2nd edition vol. 3, 1: CI chondrite hydrogen and oxygen (read from the pyrolite package's copy) doi:10.1016/B978-0-08-095975-7.00201-1

**doi:10.1038/ngeo1205**: 10.1038/ngeo1205
   DOI as cited: 10.1038/ngeo1205
   What we rest on it:
   - const KAMLAND11: &str = "KamLAND Collaboration (Gando, A. et al.) 2011, Nature Geoscience 4, 647: partial radiogenic heat model for Earth revealed by geoneutrino measurements. doi:10.1038/ngeo1205";
   - 10.1038/ngeo1205
   - KamLAND Collaboration (Gando, A. et al.) 2011, Nature Geoscience 4, 647 doi:10.1038/ngeo1205

**doi:10.1088/1475-7516/2025/02/021**: 10.1088/1475-7516/2025/02/021
   DOI as cited: 10.1088/1475-7516/2025/02/021
   What we rest on it:
   - - The two pressure rows w0 and wa in the expansion rate, with the growth factor found by stepping its small differential equation when w is not −1 (cheap, and it then follows any change). Evolving dark energy is an open question: the DESI survey's 2024 and 202…
   - 10.1088/1475-7516/2025/02/021
   - DESI Collaboration (Adame, A. G. et al.) 2025, JCAP 02, 021: DESI 2024 VI, cosmological constraints from baryon acoustic oscillations doi:10.1088/1475-7516/2025/02/021

**doi:10.1103/physrevd.110.030001**: 10.1103/physrevd.110.030001
   DOI as cited: 10.1103/physrevd.110.030001
   What we rest on it:
   - const PDG24: &str = "Navas, S. et al. (Particle Data Group) 2024, Review of Particle Physics: Cosmic rays, Phys. Rev. D 110, 030001: primary nucleon intensity falls as about E^−2.7 above a few GeV (integral index about 1.7). doi:10.1103/PhysRevD.110.030001";
   - 10.1103/PhysRevD.110.030001
   - Navas, S. et al. (Particle Data Group) 2024, Phys. Rev. D 110, 030001 doi:10.1103/PhysRevD.110.030001

**efstathiou1982**: Efstathiou, Lake and Negroponte 1982. Tuned against
   What we rest on it:
   - // Efstathiou, Lake and Negroponte 1982: v_max / sqrt(G M_d / R_d) below about 1.1 is bar-unstable.
   - …2015, https://doi.org/10.1088/0004-637X/813/2/82); discs Mo, Mao and White 1998; bars Efstathiou, Lake and Negroponte 1982. Tuned against Behroozi et al. 2019, Renzini and Peng 2015, Andrews and Martini 2013 and Madau and Dickinson 2014 (package log G3)

**eiband1959**: Eiband, A. M. 1959, Human tolerance to rapidly applied accelerations
   What we rest on it:
   - //! - Brief crew tolerance from Eiband 1959 (NASA Memo 5-19-59E), read roughly
   - rough reading of the eyeballs-in (+Gx) voluntary tolerance curve: Eiband, A. M. 1959, Human tolerance to rapidly applied accelerations: a summary of the literature, NASA Memo 5-19-59E

**elkinstanton2008**: Elkins-Tanton, L. T. 2008, EPSL 271, 181
   What we rest on it:
   - pub const REF_ELKINS: &str = "Elkins-Tanton, L. T. 2008, EPSL 271, 181: magma ocean outgassing; most water and CO2 go to the early atmosphere, a fraction stays in the mantle";
   - /// Fraction of water and CO2 that stays in a solidifying mantle (Elkins-Tanton 2008: a few to tens of percent).
   - /// left (Elkins-Tanton 2008's retained share), rock the rest.

**elser2011**: Elser, S. et al. 2011, Icarus 214, 357
   What we rest on it:
   - pub const REF_CANUP04: &str = "Canup, R. M. 2004, Icarus 168, 433; Elser, S. et al. 2011, Icarus 214, 357: moon-forming impacts are oblique (30 to 60 degrees), impactor 5 to 50 percent of the total; a few to tens of percent of rocky planets get one";
   - /// impactor leaves a large moon (1: the window alone decides; Elser et al. 2011's rate follows).
   - // Moon-forming impacts: a few to tens of percent of rocky planets (Elser et al. 2011).

**fabrycky2014**: Fabrycky et al. 2014, ApJ 790, 146
   What we rest on it:
   - // Kepler period ratios: median about 1.9 to 2.2, few below 1.3 (Fabrycky et al. 2014,
   - // Near first-order resonances: a modest excess, about 10 to 15 percent (Fabrycky 2014).
   - /// scale of about 1.5 degrees (Fabrycky et al. 2014, ApJ 790, 146).

**fixsen2009**: Fixsen, D. J. 2009, ApJ 707, 916
   DOI as cited: 10.1088/0004-637x/707/2/916
   What we rest on it:
   - | Background temperature, neutrino count | 2.7255 K, 3.046 | real | Fixsen 2009; Mangano et al. 2005 |
   - Fixsen, D. J. 2009, ApJ 707, 916: the temperature of the cosmic microwave background (2.72548 +- 0.00057 K)
   - Fixsen, D. J. 2009, ApJ 707, 916: the temperature of the cosmic microwave background (2.72548 +- 0.00057 K) doi:10.1088/0004-637X/707/2/916

**frost1991**: Frost, B. R. 1991, Reviews in Mineralogy 25, 1
   DOI as cited: 10.1515/9781501508684-004
   What we rest on it:
   - pub const FRO91: &str = "Frost, B. R. 1991, Reviews in Mineralogy 25, 1: introduction to oxygen fugacity and its petrologic importance; iron-wüstite log fO2 = -27489/T + 6.702 (table 1). doi:10.1515/9781501508684-004";
   - iron-wüstite buffer, log fO2 = -A/T + B: A in K (Frost 1991)
   - iron-wüstite buffer: B (Frost 1991)

**fulton2017**: Fulton et al. 2017 see
   What we rest on it:
   - // about 2 Earth radii and stripped cores below 1.6 (Fulton et al. 2017).
   - // 1 to 5 percent hydrogen spans the sub-Neptune peak Fulton et al. 2017 see
   - // The radius gap (Fulton et al. 2017): a super-Earth peak below about

**gaillard2022**: Gaillard, F. et al. 2022, EPSL 577, 117255 doi:10.1016/j.epsl.2021.117255
   DOI as cited: 10.1016/j.epsl.2021.117255
   What we rest on it:
   - pub const GAI22: &str = "Gaillard, F., Bernadou, F., Roskosz, M. et al. 2022, EPSL 577, 117255: redox controls during magma ocean degassing. doi:10.1016/j.epsl.2021.117255";
   - Gaillard, F. et al. 2022, EPSL 577, 117255
   - Gaillard, F. et al. 2022, EPSL 577, 117255 doi:10.1016/j.epsl.2021.117255

**genovali2014**: Genovali et al. 2014
   What we rest on it:
   - // Cepheids give -0.06 (Genovali et al. 2014), open clusters -0.05 (Netopil et al. 2016).
   - ISM [Fe/H] rises as log10 of 1 - exp(-t/tau); present-day gradient -0.06 dex per kpc (Genovali et al. 2014)

**gillooly2001**: Gillooly et al. 2001
   What we rest on it:
   - …re given only where I saw them. Sources already cited by the built packs (Kiang 2007, Gillooly 2001, Field 1998, Canfield 2006, Chyba and Hand 2001, Hallsworth 2021) are not repeated.
   - 0.65 eV, the metabolic theory's activation energy for generation rate (Gillooly et al. 2001)

**godfrey1988**: Godfrey, D. A. 1988, A hexagonal feature around Saturn's north pole, Icarus 76, 335-356 doi:10.1016/0019-1035(88)90075-9
   DOI as cited: 10.1016/0019-1035(88)90075-9
   What we rest on it:
   - …t (the hexagon)**: Saturn's six-sided jet near 78° north has persisted since Voyager (Godfrey 1988, "A hexagonal feature around Saturn's north pole", *Icarus* **[title checked; volume and page to confirm]**). Laboratory and numerical work shows a strong, narro…
   - …975, docs/fudges.md). Saturn: 78.6 degrees, against the hexagon's about 78 degrees N (Godfrey 1988).
   - Rule fitted to Saturn's hexagon at about 78 degrees N (Godfrey 1988). A strong, narrow circumpolar jet settles into a stable meander whose number of sides is set by the deformation radius against the jet (laboratory wavenumber 6: Barbosa Aguiar et al. 2010; nu…

**gough1981**: Gough, D. O. 1981, Solar Physics 74, 21
   DOI as cited: 10.1007/bf00151270
   What we rest on it:
   - "Gough, D. O. 1981, Solar Physics 74, 21: solar interior structure and luminosity variations. doi:10.1007/BF00151270";
   - // Gough 1981: the zero-age Sun had about 70 percent of today's light.

**graham2013**: Graham and Fruchter 2013 find few above a third solar
   What we rest on it:
   - /// 1 / (1 + (Z / Z_c)^2) (Graham and Fruchter 2013 find few above a third solar).
   - …et al. 2003); lethal bursts within 2 kpc at low metallicity (Piran and Jimenez 2014; Graham and Fruchter 2013), burst yield tuned so the Sun now sees about 0.4 per Gyr; texture is keyed lognormal noise (sketch)

**gwin1959**: Gwin, R. and Mee, W. T. 1959, Critical assemblies of uranium metal, Y-1248, Union Carbide Nuclear Company, Oak Ridge Y-12 Plant, Table I
   What we rest on it:
   - Gwin, R. and Mee, W. T. 1959, Critical assemblies of uranium metal, Y-1248, Union Carbide Nuclear Company, Oak Ridge Y-12 Plant, Table I: bare sphere of 93 to 94% U-235 metal (about 18.5 g/cm3), critical at 53.9 kg. https://www.osti.gov/servlets/purl/4246875
   - Gwin, R. and Mee, W. T. 1959, Critical assemblies of uranium metal, Y-1248, Table I: a bare sphere of 93 to 94% U-235 metal is critical at 53.9 kg. https://www.osti.gov/servlets/purl/4246875

**haisch2001**: Haisch, Lada and Lada 2001, ApJL 553, L153
   What we rest on it:
   - pub const REF_LIFETIME: &str = "Haisch, Lada and Lada 2001, ApJL 553, L153; Mamajek 2009, AIP Conf. Proc. 1158, 3: half of discs gone by about 3 Myr, e-folding about 2.5 Myr";
   - // past 6 (Haisch et al. 2001; Mamajek 2009).

**hale1973**: Hale, G. M. and Querry, M. R. 1973, Applied Optics 12, 555
   DOI as cited: 10.1364/ao.12.000555
   What we rest on it:
   - // MP32: pure water's absorption (Pope and Fry 1997; Hale and Querry 1973)
   - Hale, G. M. and Querry, M. R. 1973, Applied Optics 12, 555: optical constants of water in the 200 nm to 200 micron wavelength region; absorption in 1/cm (OMLC copy hale73.txt)
   - Hale, G. M. and Querry, M. R. 1973, Applied Optics 12, 555: optical constants of water in the 200 nm to 200 micron wavelength region; absorption in 1/cm (OMLC copy hale73.txt) doi:10.1364/AO.12.000555 https://omlc.org/spectra/water/data/hale73.txt

**hansen2006**: Hansen et al. 2006, Science 311
   What we rest on it:
   - /// plume carries about 200 to 300 kg/s (Hansen et al. 2006, Science 311,
   - // Hansen et al. 2006: about 200 kg/s; within a factor of three.

**haruyama2009**: Haruyama, J. et al. 2009, GRL 36, L21206
   DOI as cited: 10.1029/2009gl040635
   What we rest on it:
   - "Haruyama, J. et al. 2009, GRL 36, L21206: a possible lunar lava tube skylight in the Marius Hills, about 65 m across. doi:10.1029/2009GL040635",
   - …ubes on Earth, Moon and Mars); Blair et al. 2017 (Icarus, lunar lava tube stability); Haruyama et al. 2009 (GRL, Marius Hills skylight); Keszthelyi 1995 (JGR, lava tube thermal budget); Palmer 1991 (GSA Bulletin, limestone caves); Cornet et al. 2015 (JGR Plane…

**hassler2014**: Hassler et al. 2014, doi:10.1126/science.1244797
   DOI as cited: 10.1126/science.1244797
   What we rest on it:
   - /// (Hassler et al. 2014, doi:10.1126/science.1244797). Moon: 1.37 mSv a day
   - …net below, attenuated with a 44 g/cm2 length so Mars's 16 g/cm2 gives its 230 mSv/yr (Hassler et al. 2014), plus 2.4 mSv/yr from the ground (UNSCEAR 2008)

**henriques2013**: Henriques et al. 2013
   What we rest on it:
   - /// Ejected gas falls back on t = gamma (1e10 / M_h) (Henriques et al. 2013),
   - …times Tacconi et al. 2018 and Leroy et al. 2008; winds Muratov et al. 2015; fallback Henriques et al. 2013; hot haloes and radio-mode feedback Croton et al. 2006; ex-situ growth Rodriguez-Gomez et al. 2016; ex-situ black holes at 1e-3 of the stars they arrive…

**hirschmann2012**: Hirschmann, M. M. 2012, EPSL 341, 48
   DOI as cited: 10.1016/j.epsl.2012.06.015
   What we rest on it:
   - pub const HIR12: &str = "Hirschmann, M. M. 2012, EPSL 341, 48: magma ocean influence on early atmosphere mass and composition. doi:10.1016/j.epsl.2012.06.015";
   - …he air's later cooling, escape and photochemistry are not here (Gaillard et al. 2022, Hirschmann 2012).

**horn1981**: Horn, B. K. P. (1981). Hill shading and the reflectance map
   What we rest on it:
   - text: "Horn, B. K. P. (1981). Hill shading and the reflectance map. Proceedings of the IEEE 69(1), 14-47.",
   - // a face, and the surface normal tilted by the real slope between neighbouring texels (Horn 1981's idea, on the

**irsic2017**: Iršič et al. 2017, doi:10.1103/PhysRevD.96.023522
   DOI as cited: 10.1103/physrevd.96.023522
   What we rest on it:
   - …52.x). Fewer ultra-faint dwarfs, later first stars | heavier than about 5 to 6.5 keV (Iršič et al. 2017, doi:10.1103/PhysRevD.96.023522; Nadler et al. 2021, doi:10.1103/PhysRevLett.126.091101) |
   - Iršič, V. et al. 2017, Phys. Rev. D 96, 023522: new constraints on the free-streaming of warm dark matter from Lyman-alpha forest data
   - Iršič, V. et al. 2017, Phys. Rev. D 96, 023522: new constraints on the free-streaming of warm dark matter from Lyman-alpha forest data doi:10.1103/PhysRevD.96.023522

**jewitt2007**: Jewitt, D. and Haghighipour, N. 2007, ARA&A 45, 261
   What we rest on it:
   - pub const REF_JEWITT07: &str = "Jewitt, D. and Haghighipour, N. 2007, ARA&A 45, 261: irregular moons orbit at 0.1 to 0.5 Hill radii, eccentric, many retrograde";
   - …giant impacts (F4), and scatters small captured ones far out, half of them backwards (Jewitt and Haghighipour 2007). So a *generated* system has its moons from the physics. The only thing missing is small: **Add here** can ask formation "what forms at this orb…
