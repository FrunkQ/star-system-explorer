# Consensus batch 8: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-08.md` beside this file).

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

**judge2003**: Judge et al. 2003, as used by
   What we rest on it:
   - /// The Sun's X-ray fraction (Judge et al. 2003, as used by Wright et al. 2011).

**kattenhorn2009**: Kattenhorn and Hurford 2009, in Europa, 199
   What we rest on it:
   - …r example: "ridges 0.7: tidal stress 85 kPa against an ice strength of about 100 kPa (Kattenhorn and Hurford 2009)". Where no published relation exists, the curve is a fudge and goes in the fudge register with its reason.
   - tidal flexing cracks the shell and ridges grow either side (Kattenhorn and Hurford 2009, in Europa, 199; Europa's lineae trend about 45 degrees from east).

**katz1952**: Katz, L. and Penfold, A. S. 1952, Range-energy relations for electrons and the determination of beta-ray end-point energies by absorption, Rev. Mod. Phys. 24, 28
   DOI as cited: 10.1103/revmodphys.24.28
   What we rest on it:
   - const KATZ52: &str = "Katz, L. and Penfold, A. S. 1952, Range-energy relations for electrons and the determination of beta-ray end-point energies by absorption, Rev. Mod. Phys. 24, 28: R = 412 E^(1.265 - 0.0954 ln E) mg/cm² for 0.01 to 2.5 MeV, R = 530 E - 106…
   - /// (Katz and Penfold 1952; nearly the same in any light material).

**kervella2017**: Kervella et al. 2017, A&A 598, L7 (0.1542 solar radii, from the
   What we rest on it:
   - Kervella et al. 2017, A&A 598, L7 (0.1542 solar radii, from the Mann et al. 2015 relation; Teff 3050 K is not from this paper and still needs a source)
   - Kervella et al. 2017, A&A 597, A137 (0.8632 solar radii; Teff 5231 K as adopted there from Jofré et al. 2015)

**khare1984**: Khare, B. N., Sagan, C., Arakawa, E. T., Suits, F., Callcott, T. A. and Williams, M. W. 1984, Icarus 60, 127
   DOI as cited: 10.1016/0019-1035(84)90142-8
   What we rest on it:
   - // MP33: Khare et al. 1984 tholin; k falls from the ultraviolet to the
   - Khare, B. N., Sagan, C., Arakawa, E. T., Suits, F., Callcott, T. A. and Williams, M. W. 1984, Icarus 60, 127: optical constants of organic tholins produced in a simulated Titanian atmosphere, from soft X-ray to microwave frequencies; Table I, n and k = a 10^b…
   - 10.1016/0019-1035(84)90142-8

**kharecha2005**: Kharecha et al. 2005, about 1e12 to 1e13
   What we rest on it:
   - // With a hydrogen supply too (Kharecha et al. 2005, about 1e12 to 1e13
   - Stoichiometry: 4 Fe2+ per CO2 fixed (Widdel et al. 1993; Canfield et al. 2006); 2 H2 per CO2 (Kharecha et al. 2005); chemolithotroph and methanogen yields are order-of-magnitude growth yields (McCollom and Amend 2005)

**kivelson1995**: Kivelson and Russell 1995 [M2] |
   What we rest on it:
   - | Where the bubble ends, and its shape | settled | Pressure balance between the field and the wind; shape from fits to thousands of crossings | Shue et al. 1997 [M1]; Kivelson and Russell 1995 [M2] |
   - Earth (NSSDC fact sheet: Bond albedo 0.294, black-body temperature 254.0 K; magnetopause about 10 Earth radii, Kivelson and Russell 1995)

**koll2016**: Koll, D. D. B. and Abbot, D. S. 2016, Temperature structure and atmospheric circulation of dry tidally locked rocky exoplanets, ApJ 825, 99 https://arxiv.org/abs/1605.01066
   arXiv as cited: 1605.01066
   What we rest on it:
   - //! drag time τ_wave H / (C_D a) (Koll and Abbot 2016) and 1/f; weighted by how
   - Koll, D. D. B. and Abbot, D. S. 2016, Temperature structure and atmospheric circulation of dry tidally locked rocky exoplanets, ApJ 825, 99
   - Koll, D. D. B. and Abbot, D. S. 2016, Temperature structure and atmospheric circulation of dry tidally locked rocky exoplanets, ApJ 825, 99 https://arxiv.org/abs/1605.01066

**konopkova2016**: Konopkova et al. 2016 about 30
   What we rest on it:
   - param("k", dims::CONDUCTIVITY, 130.0, (10.0, 300.0), "thermal conductivity of the liquid alloy (Pozzo et al. 2012 give about 150; Konopkova et al. 2016 about 30; until MP21's iron rows)"),
   - thermal conductivity of the liquid alloy (Pozzo et al. 2012 give about 150; Konopkova et al. 2016 about 30; until MP21's iron rows)

**kozai1962**: Kozai 1962 section 4
   What we rest on it:
   - // up to e = 0.76 (Kozai 1962), a note, not a verdict of loss.
   - Kozai 1962 and Lidov 1962: a distant outer body tilted more than 39.2 degrees (below 140.8) trades the inner orbit's tilt for stretch, up to e = √(1 − (5/3)cos² i) from a round start. Kiseleva, Eggleton and Mikkola 1998: the time for one swing
   - Kozai 1962 section 4; Kiseleva, Eggleton and Mikkola 1998 eq. 1

**lay2008**: Lay et al. 2008, 5 to 15 TW
   What we rest on it:
   - Earth: 10 TW out of the core (Lay et al. 2008, 5 to 15 TW) gives a dipole moment near the measured 7.7e22 A m² (IGRF-13)

**leroy2008**: Leroy et al. 2008
   What we rest on it:
   - /// t_dep times (v_c / v_dep)^-k below v_dep (Leroy et al. 2008; Huang et al. 2012).
   - …. 2013; Dave, Finlator and Oppenheimer 2012); depletion times Tacconi et al. 2018 and Leroy et al. 2008; winds Muratov et al. 2015; fallback Henriques et al. 2013; hot haloes and radio-mode feedback Croton et al. 2006; ex-situ growth Rodriguez-Gomez et al. 201…

**lin1964**: Lin and Shu 1964
   What we rest on it:
   - //! and star formation are higher (Lin and Shu 1964; Roberts 1969)
   - …axy design section 7: rings interpolated in radius and time; log-spiral density wave (Lin and Shu 1964), pitch 12 degrees and pattern speed 25 km/s/kpc (rough Milky Way values); scale heights 100, 300 and 900 pc; sterilising distance 10 pc (Gehrels et al. 2003…

**lyons2014**: Lyons et al. 2014
   What we rest on it:
   - /// before 2.1 Gyr, 1% to 3.9 Gyr, 21% after (Lyons et al. 2014), counted only
   - …ago, Tice and Lowe 2004); oxygenic within about a billion years (before 2.4 Gyr ago, Lyons et al. 2014); complex cells a hard step once oxygen exists (about 1.8 Gyr ago, Knoll 2014); large animals once oxygen nears modern levels (Ediacaran, Sperling et al. 20…

**machado2009**: Machado, G. M., Oliveira, M. M. and Fernandes, L. A. F. (2009), "A physiologically-based model for simulation of
   What we rest on it:
   - // protanopia, deuteranopia and tritanopia, by the Machado, Oliveira and Fernandes (2009) model, and also as
   - // Machado, G. M., Oliveira, M. M. and Fernandes, L. A. F. (2009), "A physiologically-based model for simulation of
   - // told apart stay apart as people with protanopia, deuteranopia and tritanopia see them (Machado et al. 2009), and

**mamajek2009**: Mamajek 2009, AIP Conf. Proc. 1158, 3
   What we rest on it:
   - pub const REF_LIFETIME: &str = "Haisch, Lada and Lada 2001, ApJL 553, L153; Mamajek 2009, AIP Conf. Proc. 1158, 3: half of discs gone by about 3 Myr, e-folding about 2.5 Myr";
   - // past 6 (Haisch et al. 2001; Mamajek 2009).

**mangano2005**: Mangano, G. et al. 2005, Nucl. Phys. B 729, 221
   DOI as cited: 10.1016/j.nuclphysb.2005.09.041
   What we rest on it:
   - | Background temperature, neutrino count | 2.7255 K, 3.046 | real | Fixsen 2009; Mangano et al. 2005 |
   - Mangano, G. et al. 2005, Nucl. Phys. B 729, 221: relic neutrino decoupling including flavour oscillations (N_eff 3.046; Omega_nu h2 = sum of neutrino masses / 93.14 eV)
   - Mangano, G. et al. 2005, Nucl. Phys. B 729, 221: relic neutrino decoupling including flavour oscillations (N_eff 3.046; Omega_nu h2 = sum of neutrino masses / 93.14 eV) doi:10.1016/j.nuclphysb.2005.09.041

**manvi1986**: Manvi, R. and Fujita, T. 1986, SP-100 program users handbook, JPL D-2185 issue 2
   What we rest on it:
   - …1.2 to 1.4 W/kg); Mason, L. S., NASA (NTRS 20180005431): Kilopower-class 1 to 7 W/kg; Manvi and Fujita 1986, SP-100 users handbook (JPL D-2185): 100 kWe for about 3,000 kg (about 33 W/kg) | corrected: later designs 1 to 7 W/kg small, about 33 large, none flown…
   - Manvi, R. and Fujita, T. 1986, SP-100 program users handbook, JPL D-2185 issue 2: 100 kWe at end of life for about 3,000 kg (design; never flown). https://www.osti.gov/servlets/purl/10184673

**maoz2012**: Maoz and Mannucci 2012 and
   What we rest on it:
   - …1); Kennicutt 1998 star formation with a Martin and Kennicutt 2001 threshold; Ia rate Maoz and Mannucci 2012 and Maoz and Graur 2017; solar mass fractions Asplund et al. 2009; radial winds Belfiore et al. 2019; inward drift Spitoni and Matteucci 2011; yields a…
   - type Ia delay-time distribution proportional to 1/t from 40 Myr, 1.3e-3 per solar mass formed (Maoz and Mannucci 2012)

**martin2001**: Martin and Kennicutt 2001 threshold
   What we rest on it:
   - //! (Martin and Kennicutt 2001);
   - …and Francois 1989; Chiappini et al. 1997, 2001); Kennicutt 1998 star formation with a Martin and Kennicutt 2001 threshold; Ia rate Maoz and Mannucci 2012 and Maoz and Graur 2017; solar mass fractions Asplund et al. 2009; radial winds Belfiore et al. 2019; inwa…

**mckee2015**: McKee, Parravano and Hollenbach 2015 give 0.04 to 0.05
   What we rest on it:
   - // Midplane gas about 0.05 solar masses per pc^3 (McKee, Parravano and Hollenbach 2015 give 0.04 to 0.05).
   - …rsen 2009 section 6.3, total dispersion); midplane density 0.1 solar masses per pc^3 (McKee et al. 2015); cluster mass function M^-2 (Lada and Lada 2003)

**melott1994**: Melott, Pellman and Shandarin (1994, MNRAS 269, 626)
   What we rest on it:
   - /// once orbits cross. Melott, Pellman and Shandarin (1994, MNRAS 269, 626)
   - …lott and Shandarin 1993, MNRAS 260, 765), smoothed where the linear spread reaches 1 (Melott, Pellman and Shandarin 1994, MNRAS 269, 626)

**mitchell2016**: Mitchell, J. L. and Lora, J. M. 2016, The climate of Titan, Annu. Rev. Earth Planet. Sci. 44, 353-380 doi:10.1146/annurev-earth-060115-012428
   DOI as cited: 10.1146/annurev-earth-060115-012428
   What we rest on it:
   - | Mitchell and Lora 2016, Annu. Rev. Earth Planet. Sci. 44, 353, doi:10.1146/annurev-earth-060115-012428 | Titan climate | pack |
   - Titan's orange photochemical haze from methane under ultraviolet (Mitchell and Lora 2016); optical depth a stand-in (docs/fudges.md)
   - Mitchell, J. L. and Lora, J. M. 2016, The climate of Titan, Annu. Rev. Earth Planet. Sci. 44, 353-380

**owen2017**: Owen, J. E. and Wu, Y. 2017, ApJ 847, 29
   What we rest on it:
   - pub const REF_OW17: &str = "Owen, J. E. and Wu, Y. 2017, ApJ 847, 29: energy-limited photoevaporation carves the radius valley";
   - /// Photoevaporation efficiency (Owen and Wu 2017 use about 0.1).
   - // Owen and Wu 2017: a few-Earth-mass core with a few percent hydrogen is

**palme2014**: Palme and O'Neill 2014 (pyrolite 0.3.9 CH_PalmeONeill2014.csv, one copy)
   What we rest on it:
   - CI chondrite by mass; from Palme and O'Neill 2014 (pyrolite 0.3.9 CH_PalmeONeill2014.csv, one copy); neither copy of McDonough and Sun 1995 carries it

**park2016**: Park, R. S. et al. 2016, Nature 537, 515
   DOI as cited: 10.1038/nature18955
   What we rest on it:
   - /// metal through its rock (Park et al. 2016 find it only partly separated).
   - Park, R. S. et al. 2016, Nature 537, 515: a partially differentiated interior for (1) Ceres deduced from its gravity field and shape (mass 938.416e18 kg)
   - Park, R. S. et al. 2016, Nature 537, 515: a partially differentiated interior for (1) Ceres deduced from its gravity field and shape (mass 938.416e18 kg) doi:10.1038/nature18955

**parkin2018**: Parkin, K. L. G. 2018, The Breakthrough Starshot system model, Acta Astronautica 152, 370, arXiv:1805.01306 v3, Tables 1 and 2. https://arxiv.org/pdf/1805.01306
   arXiv as cited: 1805.01306
   What we rest on it:
   - - Licence: Apache-2.0. The figures come from US Government works and Parkin 2018.
   - Parkin, K. L. G. 2018, The Breakthrough Starshot system model, Acta Astronautica 152, 370, arXiv:1805.01306 v3, Tables 1 and 2. https://arxiv.org/pdf/1805.01306

**peng2008**: Peng, B. et al. 2008, Nature Nanotechnology 3, 626 (abstract)
   What we rest on it:
   - | nanotube single tubes against yarn | Peng, B. et al. 2008, Nature Nanotechnology 3, 626 (abstract): single tubes above 100 GPa; Bai, Y. et al. 2018, Nature Nanotechnology 13, 589 (abstract): centimetre bundles over 80 GPa; Taylor, L. W. et al. 2020, Carbon (…
   - Peng, B. et al. 2008, Nature Nanotechnology 3, 626: single carbon nanotube shells near 100 GPa (as summarised; not read)

**petit2017**: Petit, Laskar and Boué 2017 eqs. 89, 97, 100 and 101
   DOI as cited: 10.1051/0004-6361/201630022
   What we rest on it:
   - Petit, Laskar and Boué 2017 eq. 29: r = (K1(2/3) + 2 K0(2/3))/π
   - …the outer planet's circular momentum, is below the least that lets the orbits touch. Petit, Laskar and Boué 2017: very close pairs use instead the deficit at which first-order resonances overlap, zero where even circular orbits overlap
   - Laskar and Petit 2017 eqs. 28, 29 and 39; Petit, Laskar and Boué 2017 eqs. 89, 97, 100 and 101

**pike1977**: Pike 1977, transitions 15, 150 and 600 km at lunar gravity scaled as 1/g
   What we rest on it:
   - "Pike, R. J. 1977, in Impact and Explosion Cratering, 489: depth and rim height against diameter, simple and complex lunar craters. https://ui.adsabs.harvard.edu/abs/1977iecp.symp..489P",
   - /// Fresh crater geometry (Pike 1977 forms, transitions scaled as 1/g):
   - …-0.216 (Holsapple 1993, doi:10.1146/annurev.ea.21.050193.002001); depth and rim after Pike 1977, transitions 15, 150 and 600 km at lunar gravity scaled as 1/g (Melosh 1989); atmosphere cut-off 0.1 km x P^0.75 fitted to Earth and Venus (no Venus craters under a…

**piran2014**: Piran and Jimenez 2014 give about 0.5 to 1 per Gyr at the Sun now
   What we rest on it:
   - // Lethal gamma-ray bursts: Piran and Jimenez 2014 give about 0.5 to 1 per Gyr at the Sun now.
   - …distance 10 pc (Gehrels et al. 2003); lethal bursts within 2 kpc at low metallicity (Piran and Jimenez 2014; Graham and Fruchter 2013), burst yield tuned so the Sun now sees about 0.4 per Gyr; texture is keyed lognormal noise (sketch)

**policy2025**: Policy (2025) allows fan work for personal, non-commercial use and
   What we rest on it:
   - Why: Mongoose Publishing's Traveller Fair Use Policy (2025) allows fan work for personal, non-commercial use and
   - https://www.mongoosepublishing.com/pages/traveller-licensing (Fair Use Policy, 2025)

**pope1997**: Pope, R. M. and Fry, E. S. 1997, Applied Optics 36, 8710
   DOI as cited: 10.1364/ao.36.008710
   What we rest on it:
   - // MP32: pure water's absorption (Pope and Fry 1997; Hale and Querry 1973)
   - Pope, R. M. and Fry, E. S. 1997, Applied Optics 36, 8710: absorption spectrum (380-700 nm) of pure water. II. Integrating cavity measurements; table on a 2.5 nm grid in 1/cm (OMLC copy pope97.txt)
   - Pope, R. M. and Fry, E. S. 1997, Applied Optics 36, 8710: absorption spectrum (380-700 nm) of pure water. II. Integrating cavity measurements; table on a 2.5 nm grid in 1/cm (OMLC copy pope97.txt) doi:10.1364/AO.36.008710 https://omlc.org/spectra/water/data/po…

**read2014**: Read 2014, doi:10.1088/0954-3899/41/6/063101
   DOI as cited: 10.1088/0954-3899/41/6/063101
   What we rest on it:
   - // Local dark matter 0.3 to 0.5 GeV/cm³ (Read 2014 review).
   - Not on the sheet: annihilation and decay (nothing in the model reads them yet), and the local density near the Sun, which is a result to check (0.3 to 0.5 GeV/cm³; Read 2014, doi:10.1088/0954-3899/41/6/063101), not an input.

**reimers1975**: Reimers 1975
   What we rest on it:
   - /// Reimers 1975: dM/dt = -4e-13 eta L R / M solar masses per year (L, R,
   - Reimers 1975; eta near 0.5 fits the red giant branch mass loss implied by horizontal branch stars in globular clusters (McDonald and Zijlstra 2015 give 0.477)

**reines2015**: Reines and Volonteri 2015, https://doi.org/10.1088/0004-637X/813/2/82
   DOI as cited: 10.1088/0004-637x/813/2/82
   What we rest on it:
   - /// (Kormendy and Ho 2013; Reines and Volonteri 2015). This is how the
   - …rive with (Kormendy and Ho 2013, https://doi.org/10.1146/annurev-astro-082708-101811; Reines and Volonteri 2015, https://doi.org/10.1088/0004-637X/813/2/82); discs Mo, Mao and White 1998; bars Efstathiou, Lake and Negroponte 1982. Tuned against Behroozi et al.…

**renzini2015**: Renzini and Peng 2015 (the local ridge line)
   What we rest on it:
   - // Renzini and Peng 2015 (the local ridge line): log SFR = 0.76 log M* - 7.64.
   - …1998; bars Efstathiou, Lake and Negroponte 1982. Tuned against Behroozi et al. 2019, Renzini and Peng 2015, Andrews and Martini 2013 and Madau and Dickinson 2014 (package log G3)

**richardson2014**: Richardson, I. A., Leachman, J. W. and Lemmon, E. W. 2014, J. Phys. Chem. Ref. Data 43, 013103
   DOI as cited: 10.1063/1.4864752
   What we rest on it:
   - normal deuterium at its normal boiling point, from CoolProp's saturated-liquid fit to Richardson et al. 2014 (fit good to 0.17%)
   - Richardson, I. A., Leachman, J. W. and Lemmon, E. W. 2014, J. Phys. Chem. Ref. Data 43, 013103: fundamental equation of state for deuterium
   - Richardson, I. A., Leachman, J. W. and Lemmon, E. W. 2014, J. Phys. Chem. Ref. Data 43, 013103: fundamental equation of state for deuterium doi:10.1063/1.4864752

**roberts1969**: Roberts 1969
   What we rest on it:
   - //! and star formation are higher (Lin and Shu 1964; Roberts 1969)
   - * each crest (a share of the scale length), on its concave side, where the gas is shocked before the stars light (Roberts 1969). */

**rozwadowska2021**: Rozwadowska et al. 2021
   What we rest on it:
   - // Milky Way: core collapse 1.6 +- 0.5 per century (Rozwadowska et al. 2021), type Ia 0.54 +- 0.12 (Li et al. 2011).
   - // against 1.63 +- 0.46 (Rozwadowska et al. 2021).

**russell1916**: Russell, H. N. 1916, On the Albedo of the Planets and Their Satellites, ApJ 43, 173 (the Lambert sphere phase law). doi:10.1086/142244
   DOI as cited: 10.1086/142244
   What we rest on it:
   - //! geometric albedo (Russell 1916 for the phase law; the Sun's V magnitude
   - Russell, H. N. 1916, On the Albedo of the Planets and Their Satellites, ApJ 43, 173 (the Lambert sphere phase law). doi:10.1086/142244

**sansonetti2005**: Sansonetti, J. E. and Martin, W. C. 2005, J. Phys. Chem. Ref. Data 34, 1559
   DOI as cited: 10.1063/1.1800011
   What we rest on it:
   - NIST Handbook of Basic Atomic Spectroscopic Data (Sansonetti, J. E. and Martin, W. C. 2005, J. Phys. Chem. Ref. Data 34, 1559), potassium, strong lines of K I: wavelengths (air, A) and transition probabilities Aki; oscillator strengths here are worked out from…
   - NIST Handbook of Basic Atomic Spectroscopic Data (Sansonetti, J. E. and Martin, W. C. 2005, J. Phys. Chem. Ref. Data 34, 1559), sodium, strong lines of Na I: wavelengths (air, A) and transition probabilities Aki; oscillator strengths here are worked out from A…
   - 10.1063/1.1800011

**santarius1998**: Santarius, J. F. and Logan, B. G. 1998, Generic magnetic fusion rocket model, Journal of Propulsion and Power 14, 519. doi:10.2514/2.5309
   DOI as cited: 10.2514/2.5309
   What we rest on it:
   - Santarius, J. F. and Logan, B. G. 1998, Generic magnetic fusion rocket model, Journal of Propulsion and Power 14, 519. doi:10.2514/2.5309
   - Santarius, J. F. and Logan, B. G. 1998, Generic magnetic fusion rocket model, Journal of Propulsion and Power 14, 519 (abstract read; full text not read)

**scale1978**: Scale 1978, Unesco Technical Papers in Marine Science 37 (Joint Panel on Oceanographic Tables and Standards)
   What we rest on it:
   - Lewis, E. L. 1980, IEEE J. Oceanic Eng. OE-5, 3: the Practical Salinity Scale 1978 and its antecedents, eq. 8: r_T = C(35,T,0)/C(35,15,0) = c0 + c1 T + c2 T^2 + c3 T^3 + c4 T^4, T in deg C (c0..c4 = 6.766097e-1, 2.00564e-2, 1.104259e-4, -6.9698e-7, 1.0031e-9)
   - Lewis, E. L. 1980, IEEE J. Oceanic Eng. OE-5, 3: the Practical Salinity Scale 1978 and its antecedents, eq. 8: r_T = C(35,T,0)/C(35,15,0) = c0 + c1 T + c2 T^2 + c3 T^3 + c4 T^4, T in deg C (c0..c4 = 6.766097e-1, 2.00564e-2, 1.104259e-4, -6.9698e-7, 1.0031e-9)…
   - UNESCO 1981, Background papers and supporting data on the Practical Salinity Scale 1978, Unesco Technical Papers in Marine Science 37 (Joint Panel on Oceanographic Tables and Standards): practical salinity S = 0.0080 - 0.1692 K^0.5 + 25.3851 K + 14.0941 K^1.5…

**schenk2004**: Schenk et al. 2004, in Jupiter, ch. 18
   DOI as cited: 10.1016/s0019-1035(03)00048-4
   What we rest on it:
   - "Zahnle, K. et al. 2003, Icarus 163, 263: cratering rates in the outer Solar System; Europa's surface about 60 Myr old (Schenk et al. 2004, in Jupiter, ch. 18). doi:10.1016/S0019-1035(03)00048-4",
   - …world loses most of its relief in about 100 Myr; renewal times: Europa about 60 Myr (Schenk et al. 2004, in Jupiter: The Planet, Satellites and Magnetosphere, ch. 18, from the cratering rates of Zahnle et al. 2003, doi:10.1016/S0019-1035(03)00048-4), Venus ab…

**sedov1959**: Sedov 1959
   What we rest on it:
   - const SEDOV: &str = "Sedov, L. I. 1959, Similarity and Dimensional Methods in Mechanics, Academic Press; Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159 (ξ = 1.1517 for γ = 5/3). doi:10.1098/rspa.1950.004…
   - Taylor 1950; Sedov 1959

**setzmann1991**: Setzmann, U. and Wagner, W. 1991, J. Phys. Chem. Ref. Data 20, 1061
   DOI as cited: 10.1063/1.555898
   What we rest on it:
   - Setzmann, U. and Wagner, W. 1991, J. Phys. Chem. Ref. Data 20, 1061: a new equation of state and tables of thermodynamic properties for methane (values read from the NIST WebBook fluid tables, which use it: saturated liquid 422.36 kg/m3 at the normal boiling p…
   - Setzmann, U. and Wagner, W. 1991, J. Phys. Chem. Ref. Data 20, 1061: a new equation of state and tables of thermodynamic properties for methane from the melting line to 625 K at pressures up to 1000 MPa; read through CoolProp 8.1.0 (handover/packages/MP32-data…
   - 10.1063/1.555898

**shao2000**: Shao and Lu 2000 (A_N 0.111, cohesion 2.9e-4 N/m from
   What we rest on it:
   - /// Shao and Lu 2000 (Kok et al. 2012 eq. 2.8): u*ft = A_N sqrt(...), and the cohesion γ, N/m.
   - Fluid threshold of Shao and Lu 2000 (A_N 0.111, cohesion 2.9e-4 N/m from Kok and Renno 2006) as given by Kok et al. 2012; impact threshold 0.82 of it on Earth and about Earth's value in thin air (Kok 2010); law of the wall with z0 = D/30 (Kok et al. 2012); Wei…

**span1999**: Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779
   DOI as cited: 10.1063/1.556037
   What we rest on it:
   - Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779: a new equation of state for argon (critical point 150.687 K, 4.863 MPa; normal boiling point 87.303 K)
   - Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779: a new equation of state for argon (critical point 150.687 K, 4.863 MPa; normal boiling point 87.303 K) doi:10.1063/1.556037
   - Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem. Ref. Data 28, 779: a new equation of state for argon from the melting line to 700 K at pressures up to 1000 MPa; read through CoolProp 8.1.0 (handover/packages/MP32-data/sat-liquids.csv)

**spitoni2011**: Spitoni and Matteucci 2011
   What we rest on it:
   - /// Slow inward gas drift, km/s (Lacey and Fall 1985; Spitoni and Matteucci 2011
   - …r mass fractions Asplund et al. 2009; radial winds Belfiore et al. 2019; inward drift Spitoni and Matteucci 2011; yields are IMF-integrated net yields in solar units, tuned (package log G3)
