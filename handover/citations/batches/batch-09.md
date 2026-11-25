# Consensus batch 9: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-09.md` beside this file).

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

**sudarsky2000**: Sudarsky, D., Burrows, A. and Pinto, P. 2000, Albedo and reflection spectra of extrasolar giant planets, ApJ 538, 885 doi:10.1086/309160
   DOI as cited: 10.1086/309160
   What we rest on it:
   - …oud chemistry by 1-bar temperature, adapted from the effective-temperature classes of Sudarsky et al. 2000 (ApJ 538, 885. doi:10.1086/309160: ammonia below about 150 K Teff, water, clear, alkali 900 to 1500 K, silicate above); the thresholds here are on the 1-…
   - Cloud classes by temperature after Sudarsky et al. 2000 (as the S6 cloud deck); zones carry thick high cloud, belts thin high cloud that shows the layers below; covers and optical depths are stand-ins (docs/fudges.md).
   - Sudarsky, D., Burrows, A. and Pinto, P. 2000, Albedo and reflection spectra of extrasolar giant planets, ApJ 538, 885

**summers2009**: Summers, D., Tang, R. and Thorne, R. M. 2009, Limit on stably trapped particle fluxes in planetary magnetospheres, J. Geophys. Res. 114, A10210
   DOI as cited: 10.1029/2009ja014428
   What we rest on it:
   - //! The growth rate is Summers et al. (2009)'s relativistic form of Xiao et
   - const SUMMERS09: &str = "Summers, D., Tang, R. and Thorne, R. M. 2009, Limit on stably trapped particle fluxes in planetary magnetospheres, J. Geophys. Res. 114, A10210: the relativistic growth rate used for the limit. doi:10.1029/2009JA014428";
   - science: "Relativistic whistler growth (Summers et al. 2009) along each cyclotron resonance curve for the spectrum C E^-1 sin^(2S)α, solved for the C at which waves gain e^3 in one pass of length D R_p. The lowest C over 0.1 to 1 MeV is the cap.",

**takai2008**: Takai et al. 2008
   What we rest on it:
   - /// Life's temperature limits, K (about -20 C to 122 C: Takai et al. 2008
   - Lower: about 253 K, the limit of cell division in brines (Clarke et al. 2013); upper: 395 K, Methanopyrus kandleri strain 116 (Takai et al. 2008)

**tegmark1997**: Tegmark et al. 1997
   What we rest on it:
   - // First stars: 3-sigma 1e6 solar-mass minihaloes near z = 20 to 30 (Tegmark et al. 1997; Barkana and Loeb 2001 fig. 6).
   - 3-sigma peaks of 1e6 solar-mass minihaloes (Tegmark et al. 1997; Barkana and Loeb 2001)

**treumann2006**: Treumann, R. A. 2006, The electron-cyclotron maser for astrophysical application, Astron. Astrophys. Rev. 13, 229
   DOI as cited: 10.1007/s00159-006-0001-y
   What we rest on it:
   - const CYCLOTRON: &str = "Treumann, R. A. 2006, The electron-cyclotron maser for astrophysical application, Astron. Astrophys. Rev. 13, 229: auroral radio is emitted near the local electron gyrofrequency f = eB/(2π m_e). doi:10.1007/s00159-006-0001-y";
   - Treumann, R. A. 2006, Astron. Astrophys. Rev. 13, 229
   - Treumann, R. A. 2006, Astron. Astrophys. Rev. 13, 229 doi:10.1007/s00159-006-0001-y

**vallado2013**: Vallado, D. A. 2013, Fundamentals of Astrodynamics and Applications, 4th ed., Microcosm Press, algorithm 8. https://celestrak.org/software/vallado-sw.php
   DOI as cited: 10.1016/c2011-0-69685-1
   What we rest on it:
   - //! doi:10.1016/C2011-0-69685-1; Vallado 2013, Fundamentals of Astrodynamics
   - Vallado, D. A. 2013, Fundamentals of Astrodynamics and Applications, 4th ed., Microcosm Press, algorithm 8. https://celestrak.org/software/vallado-sw.php

**wagner2002**: Wagner, W. and Pruss, A. 2002, J. Phys. Chem. Ref. Data 31, 387
   DOI as cited: 10.1063/1.1461829
   What we rest on it:
   - - Water, the working equation of state: IAPWS-95, Wagner and Pruss 2002, doi:10.1063/1.1461829; melting and sublimation curves of ice Ih, III, V, VI, VII: Wagner, Riethmann, Feistel and Harvey 2011, doi:10.1063/1.3657937
   - Wagner, W. and Pruss, A. 2002, J. Phys. Chem. Ref. Data 31, 387: the IAPWS Formulation 1995 for the thermodynamic properties of ordinary water substance; read through CoolProp 8.1.0 (handover/packages/MP32-data/sat-liquids.csv)
   - Wagner, W. and Pruss, A. 2002, J. Phys. Chem. Ref. Data 31, 387: the IAPWS Formulation 1995 for the thermodynamic properties of ordinary water substance; read through CoolProp 8.1.0 (handover/packages/MP32-data/sat-liquids.csv) doi:10.1063/1.1461829

**willmer2018**: Willmer, C. N. A. 2018, The Absolute Magnitude of the Sun in Several Filters, ApJS 236, 47 (V = -26.76). doi:10.3847/1538-4365/aabfdf
   DOI as cited: 10.3847/1538-4365/aabfdf
   What we rest on it:
   - //! from Willmer 2018). The star's brightness scales from its luminosity, so
   - Willmer, C. N. A. 2018, The Absolute Magnitude of the Sun in Several Filters, ApJS 236, 47 (V = -26.76). doi:10.3847/1538-4365/aabfdf

**yoshida1990**: Yoshida 1990, Phys. Lett. A 150, 262, section 2
   What we rest on it:
   - //! symplectic property, Yoshida 1990, Phys. Lett. A 150, 262, section 2).
   - //! - Fourth order by Yoshida's triple-jump composition (Yoshida 1990, eq. 4.12):
   - // Fourth order: errors fall 16 times when the step halves (Yoshida 1990);

**zahnle2003**: Zahnle et al. 2003, doi:10.1016/S0019-1035(03)00048-4
   DOI as cited: 10.1016/s0019-1035(03)00048-4
   What we rest on it:
   - "Zahnle, K. et al. 2003, Icarus 163, 263: cratering rates in the outer Solar System; Europa's surface about 60 Myr old (Schenk et al. 2004, in Jupiter, ch. 18). doi:10.1016/S0019-1035(03)00048-4",
   - …upiter: The Planet, Satellites and Magnetosphere, ch. 18, from the cratering rates of Zahnle et al. 2003, doi:10.1016/S0019-1035(03)00048-4), Venus about 500 Myr, Titan about 500 Myr, Io under 1 Myr; volcanism thresholds, plains rates and shield growth are rul…

**zeng2019**: Zeng et al. 2019
   What we rest on it:
   - // Solid radius: rock and metal by Zeng, a water mantle adds about half its mass fraction (Zeng et al. 2019: 50 percent water, about 25 percent larger).
   - // Zeng et al. 2019: half water by mass is about a quarter larger.

**aarseth2001**: Aarseth (2001) criterion, the inner pair against the other side as its
   What we rest on it:
   - //! Aarseth (2001) criterion, the inner pair against the other side as its

**abbot2016**: Abbot 2016
   What we rest on it:
   - //! Abbot 2016): strength σ τ_rad / (1 + σ τ_rad).

**ackeret1946**: Ackeret, J. 1946, Zur Theorie der Raketen, Helvetica Physica Acta 19, 103 (the relativistic rocket equation)
   What we rest on it:
   - // Drive physics is the relativistic rocket (Ackeret 1946), which is the classical one at chemical speeds:
   - rocket: "Ackeret, J. 1946, Zur Theorie der Raketen, Helvetica Physica Acta 19, 103 (the relativistic rocket equation)",

**adibekyan2015**: Adibekyan et al. 2015
   What we rest on it:
   - solar Mg/Si 1.2 by number (Asplund et al. 2009); stellar spread from Adibekyan et al. 2015

**airy1835**: Airy, G. B. 1835, On the diffraction of an object-glass with circular aperture, Trans
   What we rest on it:
   - Airy, G. B. 1835, On the diffraction of an object-glass with circular aperture, Trans. Cambridge Phil. Soc. 5, 283: the first dark ring at 1.22 wavelengths over the aperture

**anderson1979**: Anderson, F. A. and West, W. R. 1979, Alternate propellant program phase I final report, JPL Publication 79-29 (NASA-CR-158815)
   What we rest on it:
   - Anderson, F. A. and West, W. R. 1979, Alternate propellant program phase I final report, JPL Publication 79-29 (NASA-CR-158815): TP-H1148 (Shuttle SRB) 69.60% ammonium perchlorate, 16.00% aluminium, 14.00% PBAN binder, 0.40% iron oxide, cured with an epoxy (Ta…

**anderson2016**: Anderson, J. D. 2016, Introduction to Flight, 8th ed., McGraw-Hill, section 6.8
   What we rest on it:
   - lift equal to weight at the stall: v = sqrt(2 m g / (rho C_L,max S)) (Anderson, J. D. 2016, Introduction to Flight, 8th ed., McGraw-Hill, section 6.8)

**anderson2018**: Anderson, M. S., Ewert, M. K. and Keener, J. F. 2018, Life Support Baseline Values and Assumptions Document, NASA/TP-2015-218570/REV1. https://ntrs.nasa.gov/citations/20180001338
   What we rest on it:
   - Anderson, M. S., Ewert, M. K. and Keener, J. F. 2018, Life Support Baseline Values and Assumptions Document, NASA/TP-2015-218570/REV1. https://ntrs.nasa.gov/citations/20180001338

**antares1971**: Antares (1971-U01, NASA), launched 1971 Feb  6. Its phases from GCAT
   What we rest on it:
   - Antares (1971-U01, NASA), launched 1971 Feb 6. Its phases from GCAT: Luna. Orbits keep GCAT's shape and inclination; their orientation and the place along them are estimated.

**anzellini2013**: Anzellini, S. et al. 2013, Science 340, 464
   DOI as cited: 10.1126/science.1233514
   What we rest on it:
   - "Anzellini, S. et al. 2013, Science 340, 464: melting of iron at Earth's inner core boundary (Simon-Glatzel fit). doi:10.1126/science.1233514",
   - /// Melting point of iron, K, at `p_gpa` (Anzellini et al. 2013, Simon-Glatzel).

**arp1998**: Arp, V. D., McCarty, R. D. and Friend, D. G. 1998, NIST Technical Note 1334 (revised)
   What we rest on it:
   - Arp, V. D., McCarty, R. D. and Friend, D. G. 1998, NIST Technical Note 1334 (revised): thermophysical properties of helium-4 from 0.8 to 1500 K with pressures to 2000 MPa; read through CoolProp 8.1.0 (handover/packages/MP32-data/sat-liquids.csv)

**artymowicz1994**: Artymowicz, P. and Lubow, S. H. 1994, ApJ 421, 651
   What we rest on it:
   - pub const REF_AL94: &str = "Artymowicz, P. and Lubow, S. H. 1994, ApJ 421, 651: circumstellar discs truncated near a third of the binary separation, circumbinary inner edge near 2 to 3 separations";

**aschwanden2005**: Aschwanden, M. J. (2005). Physics of the Solar Corona
   DOI as cited: 10.1007/3-540-30766-4
   What we rest on it:
   - { text: "Aschwanden, M. J. (2005). Physics of the Solar Corona. Springer (hydrostatic scale height kT/(mu m_H g); optically thin emission measure).", doi: "10.1007/3-540-30766-4" },

**asphaug2010**: Asphaug, E. 2010, Chemie der Erde 70, 199
   What we rest on it:
   - pub const REF_ASPHAUG10: &str = "Asphaug, E. 2010, Chemie der Erde 70, 199: grazing impacts above the critical impact parameter are hit-and-run; the runner loses mantle";

**aston1951**: Aston, Finke et al. 1951
   What we rest on it:
   - NIST Chemistry WebBook, SRD 69: methylhydrazine (Ttriple 220.79 K, Aston, Finke et al. 1951; liquid enthalpy of formation 54.14 kJ/mol, Cole and Gilbert 1951)
   - NIST Chemistry WebBook, SRD 69: methylhydrazine (Ttriple 220.79 K, Aston, Finke et al. 1951; liquid enthalpy of formation 54.14 kJ/mol, Cole and Gilbert 1951) https://webbook.nist.gov/cgi/cbook.cgi?ID=C60344&Units=SI&Mask=7

**atomic1964**: Atomic 1964, Nuclear pulse vehicle study, condensed summary report, GA-4891, contract NAS 8-11053 for NASA Marshall (OCR scan). https://www.classe.cornell.edu/~seb/celestia/orion/files/19770085619_1977085619.pdf
   What we rest on it:
   - General Atomic 1964, Nuclear pulse vehicle study, condensed summary report, GA-4891, contract NAS 8-11053 for NASA Marshall (OCR scan). https://www.classe.cornell.edu/~seb/celestia/orion/files/19770085619_1977085619.pdf

**bagenal2020**: Bagenal, F. and Dols, V. 2020, The space environment of Io and Europa, J. Geophys. Res. Space Physics 125, e2019JA027485
   DOI as cited: 10.1029/2019ja027485
   What we rest on it:
   - Bagenal, F. and Dols, V. 2020, The space environment of Io and Europa, J. Geophys. Res. Space Physics 125, e2019JA027485: Io supplies about a tonne of plasma a second. doi:10.1029/2019JA027485

**ball2019**: Ball, J. 2019, Maximizing specific energy by breeding deuterium, arXiv:1908.00834, Table 1
   arXiv as cited: 1908.00834
   What we rest on it:
   - Ball, J. 2019, Maximizing specific energy by breeding deuterium, arXiv:1908.00834, Table 1: D-He3 3.7 MeV per nucleon

**barnes2014**: Barnes et al. 2014, Computers and Geosciences 62, 117. doi:10.1016/j.cageo.2013.04.024
   DOI as cited: 10.1016/j.cageo.2013.04.024
   What we rest on it:
   - …180, 170. doi:10.1016/j.geomorph.2012.10.008); depression filling by priority-flood (Barnes et al. 2014, Computers and Geosciences 62, 117. doi:10.1016/j.cageo.2013.04.024); Hack's law L ~ A^0.6 (Hack 1957) as the drainage check; diffusion is an effective coa…

**barrios2019**: Barrios, A., Hamel, S., Swift, D. C. and Eggert, J. H. 2019, Nature 569, 251
   DOI as cited: 10.1038/s41586-019-1114-6
   What we rest on it:
   - Millot, M., Coppari, F., Rygg, J. R., Correa Barrios, A., Hamel, S., Swift, D. C. and Eggert, J. H. 2019, Nature 569, 251: nanosecond X-ray diffraction of shock-compressed superionic water ice (fcc superionic ice XVIII at 100 to 400 GPa and 2000 to 3000 K; sim…

**barth1965**: Barth, V. D. and McIntire, H. O. 1965, Tungsten powder metallurgy, NASA SP-5035, Battelle Memorial Institute, Table I
   What we rest on it:
   - Barth, V. D. and McIntire, H. O. 1965, Tungsten powder metallurgy, NASA SP-5035, Battelle Memorial Institute, Table I: melting points W 6126 F, Re 5720 F, Ta 5428 F, Nb 4474 F
   - Barth, V. D. and McIntire, H. O. 1965, Tungsten powder metallurgy, NASA SP-5035, Battelle Memorial Institute, Table I: melting points W 6126 F, Re 5720 F, Ta 5428 F, Nb 4474 F https://apps.dtic.mil/sti/pdfs/ADA399829.pdf

**bartlett2009**: Bartlett 2009
   What we rest on it:
   - // 1.086e8 Pa measured (Bartlett 2009); compression is inside the 3 percent.

**beals1988**: Beals, K. A. et al. 1988, Project Longshot
   What we rest on it:
   - Beals, K. A. et al. 1988, Project Longshot: an unmanned probe to Alpha Centauri, US Naval Academy, NASA CR-184718 (OCR scan). https://ntrs.nasa.gov/api/citations/19890007533/downloads/19890007533.pdf

**beniteznelson2000**: Benitez-Nelson 2000
   What we rest on it:
   - Ocean: each new phosphorus atom is reused about 1000 times before burial (marine P residence about 20 kyr against a turnover of weeks to decades; Benitez-Nelson 2000); land: about 46 times (Cleveland et al. 2013)

**benson2005**: Benson 2005 ranges
   What we rest on it:
   - /// Orbit sketch: circularity and pericentre in virial radii (Benson 2005 ranges).

**berdyugina2005**: Berdyugina, S. V. (2005). Starspots
   DOI as cited: 10.12942/lrsp-2005-8
   What we rest on it:
   - { text: "Berdyugina, S. V. (2005). Starspots: a key to the stellar dynamo. Living Reviews in Solar Physics 2, 8 (spot temperature contrasts; polar spots on active stars).", doi: "10.12942/lrsp-2005-8" },
   - /** Spot umbra this much cooler than the photosphere (Berdyugina 2005: about 1500 to 2000 K for Sun-like stars). Stand-in. */
   - * (Berdyugina 2005). Places and sizes are drawn from the star's address and

**bertucci2008**: Bertucci et al. 2008
   What we rest on it:
   - /// crosses Saturn's edge as the wind gusts (Bertucci et al. 2008,

**bessell1990**: Bessell 1990 UBVRI
   What we rest on it:
   - Photometric bands for magnitudes: effective wavelength and AB minus Vega offset from Blanton and Roweis 2007 Table 1 (Bessell 1990 UBVRI, Cohen et al. 2003 JHKs). A star's magnitude is its flux density at the effective wavelength.

**beuthe2019**: Beuthe 2019, Icarus 332, 66
   DOI as cited: 10.1002/2016gl068634
   What we rest on it:
   - //! response (Beuthe 2019; Čadek et al. 2016 find Enceladus's south polar
   - /// 43, 5653, doi:10.1002/2016GL068634; Beuthe 2019, Icarus 332, 66,

**billig1967**: Billig 1967
   What we rest on it:
   - * nose radii (Billig 1967). The shock layer's glow is a display choice (fudge register "ship pass glow"). */

**binney2008**: Binney and Tremaine 2008, eq. 4.103
   What we rest on it:
   - // (Binney and Tremaine 2008, eq. 4.103): σ = v_c / √2.

**bisterzo2014**: Bisterzo, S. et al. 2014, ApJ 787, 10
   DOI as cited: 10.1088/0004-637x/787/1/10
   What we rest on it:
   - Bisterzo, S. et al. 2014, ApJ 787, 10: Galactic chemical evolution and solar s-process abundances, dependence on the 13C-pocket structure (s-process percentages per isotope, Kr to Bi)
   - Bisterzo, S. et al. 2014, ApJ 787, 10: Galactic chemical evolution and solar s-process abundances, dependence on the 13C-pocket structure (s-process percentages per isotope, Kr to Bi) doi:10.1088/0004-637X/787/1/10
   - 10.1088/0004-637X/787/1/10

**bitsch2018**: Bitsch, B. et al. 2018, A&A 612, A30
   arXiv as cited: 1801.02341
   What we rest on it:
   - pub const REF_BITSCH18: &str = "Bitsch, B. et al. 2018, A&A 612, A30: pebble isolation mass 25 Earth masses (h/0.05)^3 (0.34 (log 0.001 / log alpha)^4 + 0.66) (eqs. 10-11) plus the diffusion term alpha/(2 St)/lambda (eq. 26); the (M/M_sun) scaling is an extrap…
   - /// Pebble isolation mass, Earth masses (Bitsch et al. 2018 eq. 26: eqs. 10-11's

**blackie1887**: Blackie, C. 1887, Geographical Etymology, 3rd ed. (Project Gutenberg #69018)
   What we rest on it:
   - …ead British astronomers and explorers. The meanings are our own short glosses, citing Blackie 1887. First pass, corrections welcome.
   - Blackie, C. 1887, Geographical Etymology, 3rd ed. (Project Gutenberg #69018)

**blanton2003**: Blanton et al. 2003, ApJ 594, 186
   What we rest on it:
   - * light at about 5,000 K, the colour of a typical galaxy today (g − r near 0.6, Blanton et al. 2003, ApJ 594, 186;

**bloecker1995**: Bloecker 1995 tracks reach 1-2 × 10⁵ K
   What we rest on it:
   - /// white dwarf track are about 10⁵ K; Bloecker 1995 tracks reach 1-2 × 10⁵ K).

**bohlin1978**: Bohlin, Savage and Drake 1978, ApJ 224, 132
   What we rest on it:
   - /** Dust in the gas: A_V per M☉/pc² of gas at the Sun's metallicity (N_H/A_V = 1.9×10²¹ cm⁻², Bohlin et al. 1978;
   - "Bohlin, Savage and Drake 1978, ApJ 224, 132: gas to dust",

**bolton1980**: Bolton 1980
   What we rest on it:
   - /// Saturation vapour pressure of water, Pa (Bolton 1980).
   - 611.2 * m::exp(17.67 * c / (c + 243.5)) // not-sheet: the Magnus fit's coefficients (Bolton 1980), not the triple point

**bonanno2002**: Bonanno, Schlattl and Paternò 2002, A&A 390, 1115
   What we rest on it:
   - /// The Sun's age, 4.57 Gyr (Bonanno, Schlattl and Paternò 2002, A&A 390, 1115,
