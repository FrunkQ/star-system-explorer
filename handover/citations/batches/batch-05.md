# Consensus batch 5: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-05.md` beside this file).

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

**mcdonough1995**: McDonough and Sun 1995 (already registered)
   What we rest on it:
   - /// Al 0.865 wt% (CI chondrite, McDonough and Sun 1995) × canonical 26Al/27Al
   - - Earth's composition and partitioning: McDonough and Sun 1995 (already registered); Rudnick and Gao 2014, doi:10.1016/B978-0-08-095975-7.00301-6
   - Earth today (KamLAND 2011: U and Th about 20 TW; mantle plus crust radiogenic total about 20 TW, McDonough and Sun 1995)

**domingos2006**: Domingos, Winter and Yokoyama 2006 eqs. 5 and 6
   DOI as cited: 10.1111/j.1365-2966.2006.11104.x
   What we rest on it:
   - //! - **Moon limit** (Domingos, Winter and Yokoyama 2006) for moons of
   - Domingos, Winter and Yokoyama 2006 eqs. 5 and 6
   - Domingos, Winter and Yokoyama 2006: from numerical runs, the widest stable moon orbit as a share of the planet's Hill radius, for moons going the planet's way and the other way

**rudolph2022**: Rudolph et al. 2022, doi:10.1029/2021GL094421
   DOI as cited: 10.1029/2021gl094421
   What we rest on it:
   - /// Rudolph et al. 2022: cracks cross Enceladus's shell where it is thinner
   - /// cracks reach, about 15 km (Rudolph et al. 2022). A thin, warm-based
   - | Eruption of water | ΔP ≥ (ρw − ρi) g H; without overpressure water stops about 10% of the shell below the surface. Cracks cross Enceladus's shell when it is under 15 km (Rudolph et al. 2022) | derived / FOUND |

**catling2017**: Catling, D. C. and Kasting, J. F. 2017, Atmospheric Evolution on Inhabited and Lifeless Worlds, Cambridge University Press, chapters 2 and 4. doi:10.1017/9781139020558"
   DOI as cited: 10.1017/9781139020558
   What we rest on it:
   - const CK: &str = "Catling, D. C. and Kasting, J. F. 2017, Atmospheric Evolution on Inhabited and Lifeless Worlds, Cambridge University Press, chapters 2 and 4. doi:10.1017/9781139020558";
   - // Earth's 288 K needs τ ≈ 0.83 in this model (Catling and Kasting 2017, ch. 2).
   - // Hydrogen at a 1000 K exobase 500 km up: λ ≈ 7 (Catling and Kasting 2017, ch. 5).

**doi:10.1016/0009-2541(94)00140-4**: 10.1016/0009-2541(94)00140-4
   DOI as cited: 10.1016/0009-2541(94)00140-4
   What we rest on it:
   - pub const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: the composition of the Earth (CI chondrite and pyrolite). doi:10.1016/0009-2541(94)00140-4";
   - pub const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: pyrolite, the silicate Earth's oxides. doi:10.1016/0009-2541(94)00140-4";
   - const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: the composition of the Earth (bulk silicate Earth U 20.3 ppb, Th 79.5 ppb, K 240 ppm). doi:10.1016/0009-2541(94)00140-4";

**eggleton1983**: Eggleton 1983, ApJ 268, 368 (lobe size)
   What we rest on it:
   - pub const REF_EGGLETON: &str = "Eggleton, P. P. 1983, ApJ 268, 368: Roche-lobe radius";
   - // Eggleton 1983: equal masses give a lobe of 0.379 of the separation.
   - pub const REF_EGGLETON: &str = "Eggleton, P. P. 1983, ApJ 268, 368: Roche lobe radius r_L/a = 0.49 q^(2/3) / (0.6 q^(2/3) + ln(1 + q^(1/3)))";

**mcdonough2003**: McDonough, W. F. 2003, Compositional model for the Earth's core, Treatise on Geochemistry vol. 2, 547, Table 4 (read from the GERM reservoir database's copy) doi:10.1016/B0-08-043751-6/02015-6 https://earthref.org/GERMRD/34/
   DOI as cited: 10.1016/b0-08-043751-6/02015-6
   What we rest on it:
   - pub const MD03: &str = "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: compositional model for the Earth's core (table 4). doi:10.1016/B0-08-043751-6/02015-6";
   - pub const MD03: &str = "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: bulk Earth 32 wt% iron, the core 85 wt% iron (tables 3 and 4). doi:10.1016/B0-08-043751-6/02015-6";
   - // (McDonough 2003, Treatise on Geochemistry 2, 547).

**blandhawthorn2016**: Bland-Hawthorn and Gerhard 2016, doi:10.1146/annurev-astro-081915-023441
   DOI as cited: 10.1146/annurev-astro-081915-023441
   What we rest on it:
   - /// Bland-Hawthorn and Gerhard 2016, doi:10.1146/annurev-astro-081915-023441).
   - // Discs of a few kpc in Milky Way haloes (the Milky Way's is 2.6 kpc, Bland-Hawthorn and Gerhard 2016).
   - /** Thin and thick disc scale heights in kpc (the Milky Way's, Bland-Hawthorn and Gerhard 2016). */

**chyba2001**: Chyba and Hand 2001 and serpentinisation supply
   What we rest on it:
   - // Chyba and Hand 2001: 1e10 to 1e14 g per year with oxidant delivery;
   - // Chyba and Hand 2001: 1e10 to 1e14 g per year with oxidant delivery; our
   - …y inside the same published ranges L1a is checked against (Field 1998, Canfield 2006, Chyba and Hand 2001). The listed model stays in the pack as `life_model: "listed/1"`; the new one is `"gradients/1"`. The default switches only when the contract passes, and…

**field1998**: Field et al. 1998
   What we rest on it:
   - // Field et al. 1998: about 105 Gt C per year (ocean 48.5, land 56.4).
   - // The same published ranges L1a is checked against (Field et al. 1998;
   - …ts, and the totals must stay inside the same published ranges L1a is checked against (Field 1998, Canfield 2006, Chyba and Hand 2001). The listed model stays in the pack as `life_model: "listed/1"`; the new one is `"gradients/1"`. The default switches only whe…

**frank2014**: Frank, E. A., Meyer, B. S. and Mojzsis, S. J. 2014, Icarus 243, 274
   What we rest on it:
   - const FMM14: &str = "Frank, E. A., Meyer, B. S. and Mojzsis, S. J. 2014, Icarus 243, 274: a radiogenic heating evolution model for cosmochemically Earth-like exoplanets https://ui.adsabs.harvard.edu/abs/2014Icar..243..274F";
   - // At formation it was several times higher (about 4 to 5 times; Frank et al. 2014, fig. 3).
   - Mars: widespread volcanism ends about 0.5 to 1 Gyr ago (Frank et al. 2014 calibration)

**hollands2026**: Hollands et al. 2026, doi:10.1093/rasti/rzag069
   DOI as cited: 10.1093/rasti/rzag069
   What we rest on it:
   - // Hollands et al. 2026 section 2.4 (new-physics demo design 2.4): a
   - …is saved as a pack anyone loads with no key. The test case is coloured white dwarfs (Hollands et al. 2026, doi:10.1093/rasti/rzag069), where a star's colour follows from what fell into it. Checked against main on 22 Nov, it is not possible yet: a white dwarf…
   - "source": { "text": "Kowalski 2010, as cited by Hollands et al. 2026", "doi": "(to be added from the citations check)", "where": "shift proportional to density" },

**koch1959**: Koch, H. W. and Motz, J. W. 1959, Bremsstrahlung cross-section formulas and related data, Rev. Mod. Phys. 31, 920
   DOI as cited: 10.1103/revmodphys.31.920
   What we rest on it:
   - /// bremsstrahlung in a material of atomic number `z` (Koch and Motz 1959).
   - const KOCH59: &str = "Koch, H. W. and Motz, J. W. 1959, Bremsstrahlung cross-section formulas and related data, Rev. Mod. Phys. 31, 920: the share of an electron's energy radiated as it stops in a material of atomic number Z, Y = k Z T / (1 + k Z T) with k abo…
   - Koch and Motz 1959, radiation yield

**laskar2017**: Laskar and Petit 2017 eqs. 28, 29 and 39
   DOI as cited: 10.1051/0004-6361/201630022
   What we rest on it:
   - //! - **Angular momentum deficit** (Laskar and Petit 2017; Petit, Laskar and
   - // Laskar and Petit 2017 section 5.4.1: the giants' deficit makes the
   - Laskar 1997; Laskar and Petit 2017: the angular momentum deficit, the spin a system of orbits lacks against flat circles. Planets trade it, but its total is kept, so it caps how stretched any orbit can become

**parker1958**: Parker, E. N. 1958, Dynamics of the interplanetary gas and magnetic fields, ApJ 128, 664
   DOI as cited: 10.1086/146579
   What we rest on it:
   - const PARKER58: &str = "Parker, E. N. 1958, Dynamics of the interplanetary gas and magnetic fields, ApJ 128, 664: the spiral interplanetary field. doi:10.1086/146579";
   - Parker 1958, the spiral: B_phi = -B_r Omega r / v in the ecliptic
   - // Parker's spiral at 1 au sits near 45 degrees (Parker 1958).

**petrenko1999**: Petrenko, V. F. and Whitworth, R. W. 1999, Physics of Ice, Oxford University Press (thermal conductivity of ice Ih, k = 651/T W/m). doi:10.1093/acprof:oso/9780198518945.001.0001"
   DOI as cited: 10.1093/acprof:oso/9780198518945.001.0001
   What we rest on it:
   - const PW: &str = "Petrenko, V. F. and Whitworth, R. W. 1999, Physics of Ice, Oxford University Press (thermal conductivity of ice Ih, k = 651/T W/m). doi:10.1093/acprof:oso/9780198518945.001.0001";
   - /// Ice's conductivity k = 651 / T W m⁻¹ K⁻¹ (Petrenko and Whitworth 1999).
   - Petrenko and Whitworth 1999

**porco2006**: Porco et al. 2006, Science 311, 1393. doi:10.1126/science.1123013
   DOI as cited: 10.1126/science.1123013
   What we rest on it:
   - // Grains stand tens to a few hundred km high (Porco et al. 2006: plume
   - | Tiger stripes and plumes | Enceladus | tidally timed opening of south polar rifts (Hurford et al. 2007; Porco et al. 2006) | gap (and fix 2) | M2 plus M3 |
   - | Porco et al. 2006, Science 311, 1393. doi:10.1126/science.1123013 | Enceladus south pole | found |

**unterborn2015**: Unterborn, C. T., Johnson, J. A. and Panero, W. R. 2015, ApJ 806, 139
   What we rest on it:
   - const UJP15: &str = "Unterborn, C. T., Johnson, J. A. and Panero, W. R. 2015, ApJ 806, 139: thorium abundances in solar twins and analogs, implications for the habitability of extrasolar planetary systems https://ui.adsabs.harvard.edu/abs/2015ApJ...806..139U";
   - /// (Unterborn et al. 2015); Earth-sized worlds make terawatts.
   - Earth around a thorium-rich star (Unterborn et al. 2015: twice the r-process, active far longer)

**yang2013**: Yang, J., Cowan, N. B. and Abbot, D. S. 2013, Stabilizing cloud feedback dramatically expands the habitable zone of tidally locked planets, ApJL 771, L45 doi:10.1088/2041-8205/771/2/L45
   DOI as cited: 10.1088/2041-8205/771/2/l45
   What we rest on it:
   - why: "Under the star the air always rises: a permanent storm and a thick cloud shield that reflects much of the light (Yang, Cowan and Abbot 2013).".into(),
   - …star, clearer night side, twin night-side gyres; the shield matters for habitability (Yang, Cowan and Abbot 2013, *Astrophysical Journal Letters* 771, L45 **[existence checked]**) |
   - | Yang, Cowan and Abbot 2013, ApJL 771, L45, doi:10.1088/2041-8205/771/2/L45 | cloud shield | pack |

**baron2018**: Bar-On et al. 2018
   What we rest on it:
   - /// (Earth's 550 Gt C; Bar-On et al. 2018).
   - // Bar-On et al. 2018: about 550 Gt C of biomass, mostly plants.
   - // Bar-On et al. 2018).

**behroozi2019**: Behroozi et al. 2019, MNRAS 488, 3143, z = 0 median
   What we rest on it:
   - // Behroozi et al. 2019 (UniverseMachine), z = 0 median M*/M_h, read off their figure 9.
   - /// (Behroozi et al. 2019, MNRAS 488, 3143, z = 0 median), around the Small
   - …lations at the present day and at two earlier epochs: stellar mass against halo mass (Behroozi et al. 2019), star-forming main sequence, mass-metallicity relation, cosmic star formation history summed over a volume (Madau and Dickinson 2014), fraction of quenc…

**burrows2001**: Burrows et al. 2001, Rev. Mod. Phys. 73, 719 (radii near 0.9 to 1 Jupiter radius)
   What we rest on it:
   - //! cooling and contracting on the same law (Burrows et al. 2001 eq. 3 is
   - /// Burrows et al. 2001 eq. 3 for brown dwarfs and giant planets, solar
   - // Burrows et al. 2001 (Rev. Mod. Phys. 73, 719) eq. 3: cooling

**canfield2006**: Canfield et al. 2006 (iron-based, 10 to 25 times lower) to Kharecha et
   What we rest on it:
   - // Canfield et al. 2006 (iron-based, 10 to 25 times lower) to Kharecha et
   - …totals must stay inside the same published ranges L1a is checked against (Field 1998, Canfield 2006, Chyba and Hand 2001). The listed model stays in the pack as `life_model: "listed/1"`; the new one is `"gradients/1"`. The default switches only when the contra…
   - …hem. Sources already cited by the built packs (Kiang 2007, Gillooly 2001, Field 1998, Canfield 2006, Chyba and Hand 2001, Hallsworth 2021) are not repeated.

**catling2005**: Catling, Glein, Zahnle and McKay 2005, "Why O2 is required by complex life on habitable planets and the concept of planetary 'oxygenation time'", Astrobiology. https://scilit.com/publications/c07ffa637132725776cc127d422d7196
   What we rest on it:
   - //! reducing sinks (Catling and Claire 2005; Catling, Zahnle and McKay 2001).
   - /// f_T = 2 f_CH4 (Catling et al. 2001; Catling and Claire 2005 eq. 4) is 1.34e16 mol H2
   - - oxygen is a **boost**: feeding levels uncapped, large bodies possible, innovation faster *[inferred: Catling et al. 2005 argue oxygen is the only plentiful oxidant energetic enough for large complex life]*;

**forget2017**: Forget et al. 2017, Icarus 287, 54, "A post-New Horizons global climate model of Pluto including the N2, CH4 and CO cycles", https://ui.adsabs.harvard.edu/abs/2017Icar..287...54F
   What we rest on it:
   - - Nothing needs a fluid simulation. Seasonal volatile transport in a real GCM (Forget et al. 2017) stays a swappable big block if ever wanted, like weather.
   - …µbar would be treated as nearly airless; nothing seasonal | exchange block with S17 (Forget et al. 2017; Bertrand and Forget 2016), haze from production; geyser plumes through W4 |
   - | W8 | Forget et al. 2017; Fray and Schmitt 2009 and Bertrand et al. 2024 (exogeology checked these via Consensus; re-check the coefficients) |

**gladman1996**: Gladman, B. et al. 1996, Icarus 122, 166, Synchronous locking of tidally evolving satellites doi:10.1006/icar.1996.0117
   DOI as cited: 10.1006/icar.1996.0117
   What we rest on it:
   - const GLADMAN: &str = "Gladman, B. et al. 1996, Icarus 122, 166: synchronous locking of tidally evolving satellites, eq. 9 https://ui.adsabs.harvard.edu/abs/1996Icar..122..166G";
   - /// Despinning time, Gyr (Gladman et al. 1996 eq. 9 form).
   - Gladman, B. et al. 1996, Icarus 122, 166, Synchronous locking of tidally evolving satellites

**globus2017**: Globus, A. and Hall, T. 2017, Space settlement population rotation tolerance, NSS Space Settlement Journal 2
   What we rest on it:
   - …t comfortable spin (up to about 4 turns a minute for anyone, 6 for trained residents; Globus and Hall 2017); lowest healthy gravity | spin rings and tumbling pairs (1 g at 2 turns a minute needs a 224 m arm, at 4 a 56 m arm; computed) | gravity plates |
   - | Globus and Hall 2017 | Globus, A. and Hall, T. 2017, NSS Space Settlement Journal, June 2017 | up to 4 rpm acceptable (56 m at 1 g); 6 rpm for residents with training; the older 1 to 2 rpm assumption | read |
   - Globus, A. and Hall, T. 2017, Space settlement population rotation tolerance, NSS Space Settlement Journal 2.

**grimm1993**: Grimm, R. E. and McSween, H. Y. 1993, Science 259, 653
   DOI as cited: 10.1126/science.259.5095.653
   What we rest on it:
   - const GM93: &str = "Grimm, R. E. and McSween, H. Y. 1993, Science 259, 653: heliocentric zoning of the asteroid belt by aluminum-26 heating. doi:10.1126/science.259.5095.653";
   - Iron-meteorite parent body, formed within 0.5 Myr: melted and dry (Grimm and McSween 1993)

**gunn1972**: Gunn and Gott 1972 (doi:10.1086/151605)
   DOI as cited: 10.1086/151605
   What we rest on it:
   - /// Linear collapse threshold for a top-hat (Gunn and Gott 1972).
   - //! its gas, 2πG Σ_tot Σ_gas at its disc scale (Gunn and Gott 1972,
   - …y's fixed 3 Gyr delay, so small ones stop fast and big ones keep going, as observed | Gunn and Gott 1972 (doi:10.1086/151605); Fillingham et al. 2015 (doi:10.1093/mnras/stv2058) |

**hubbell2004**: Hubbell, J. H. and Seltzer, S. M. 2004, Tables of X-ray mass attenuation coefficients and mass energy-absorption coefficients (NIST Standard Reference Database 126)
   What we rest on it:
   - const HUBBELL: &str = "Hubbell, J. H. and Seltzer, S. M. 2004, Tables of X-ray mass attenuation coefficients and mass energy-absorption coefficients (NIST Standard Reference Database 126): aluminium and liquid water. https://physics.nist.gov/PhysRefData/XrayMa…
   - Photon coefficients for belt shielding (MF7 part 2b-i): the shield is aluminium, the dose point liquid water as a stand-in for soft tissue. Copied from NIST Standard Reference Database 126 (Hubbell and Seltzer 2004), photon energies 0.05 to 20 MeV.
   - Hubbell and Seltzer 2004, aluminium (Z = 13)

**kaspi2015**: Kaspi, Y. and Showman, A. P. 2015, Atmospheric dynamics of terrestrial exoplanets over a wide range of orbital and atmospheric parameters, ApJ 804, 60 doi:10.1088/0004-637X/804/1/60
   DOI as cited: 10.1088/0004-637x/804/1/60
   What we rest on it:
   - **Step 1, the circulation regime**, from spin rate, size, air mass and heating (after Kaspi and Showman 2015, *Astrophysical Journal* 804, "Atmospheric dynamics of terrestrial exoplanets over a wide range of orbital and atmospheric parameters" **[existence che…
   - | Kaspi and Showman 2015, ApJ 804, 60, doi:10.1088/0004-637X/804/1/60 | trends across spin and air | pack |
   - …the Rhines arrest wavenumber k = (beta/2U)^1/2 (Rhines 1975); trends checked against Kaspi and Showman 2015. The wind scale comes from the thermal wind (W7b, strengths); the contrast is tuned to Earth (docs/fudges.md). The regime is a label read off the stren…

**kormendy2013**: Kormendy and Ho 2013, https://doi.org/10.1146/annurev-astro-082708-101811
   DOI as cited: 10.1146/annurev-astro-082708-101811
   What we rest on it:
   - // Black holes stay below about a percent of their galaxy (Kormendy and Ho 2013 give 0.1 to 0.5% of bulges).
   - /// track their galaxies, Kormendy and Ho 2013).
   - /// (Kormendy and Ho 2013; Reines and Volonteri 2015). This is how the

**krissansentotton2016**: Krissansen-Totton, Bergsman and Catling 2016, "On detecting biospheres from chemical thermodynamic disequilibrium in planetary atmospheres", Astrobiology. arXiv:1503.08249
   arXiv as cited: 1503.08249
   What we rest on it:
   - //! the measure of Krissansen-Totton, Bergsman and Catling (2016,
   - // Krissansen-Totton et al. 2016: 1.51 J/mol for the atmosphere alone
   - …s atmosphere and ocean and showed how it changed over Earth's history *[demonstrated: Krissansen-Totton, Bergsman and Catling 2016; Krissansen-Totton, Olson and Catling 2018]*. Because this design already computes every gradient, box I can read its "atmosphere…

**lane2010**: Lane and Martin 2010, "The energetics of genome complexity", Nature 467. doi:10.1038/nature09486
   DOI as cited: 10.1038/nature09486
   What we rest on it:
   - /// power gate on complex cells, Lane and Martin 2010).
   - // The power gate (Lane and Martin 2010): large genomes need
   - …The complex-cells rate, currently tuned to one data point, gets a physical gate from Lane and Martin (2010): large genomes need far more power per gene than a simple cell's membrane can supply *[plausible]*. That keeps the long microbial era as a result.

**li2018**: Li, L. et al. 2018, Less absorbed solar energy and more internal heat for Jupiter, Nature Communications 9, 3709. doi:10.1038/s41467-018-06107-2"
   DOI as cited: 10.1038/s41467-018-06107-2
   What we rest on it:
   - const LI18: &str = "Li, L. et al. 2018, Less absorbed solar energy and more internal heat for Jupiter, Nature Communications 9, 3709. doi:10.1038/s41467-018-06107-2";
   - Jupiter (Li et al. 2018: Bond albedo 0.503, internal heat 7.485 W/m², effective temperature 125.57 K)
   - Jupiter: Juno's 7.5 W/m² internal flux (Li et al. 2018) through a metallic-hydrogen dynamo at 0.85 R_J gives about 4 G at the equator (JRM09)

**lichtenberg2019**: Lichtenberg, T. et al. 2019, Nature Astronomy 3, 307
   DOI as cited: 10.1038/s41550-018-0688-5
   What we rest on it:
   - const L19: &str = "Lichtenberg, T. et al. 2019, Nature Astronomy 3, 307: a water budget dichotomy of rocky protoplanets from 26Al-heating. doi:10.1038/s41550-018-0688-5";
   - Same early body around a star with no Al-26: wet (Lichtenberg et al. 2019)

**lineweaver2004**: Lineweaver, Fenner and Gibson 2004
   What we rest on it:
   - /// Complex life needs this long since the rocky floor was passed (Lineweaver et al. 2004: 4 +- 1 Gyr).
   - // Complex life nowhere before the universe was 6 Gyr old (Lineweaver et al. 2004: the zone emerges about 8 Gyr ago).
   - …, and (for complex life) enough quiet time. This is the galactic habitable zone idea (Lineweaver, Fenner and Gibson 2004) made editable. On the galaxy chart it is a hatched green band that visibly starts near the centre and moves outward over billions of years…

**matteucci1989**: Matteucci and Francois 1989
   What we rest on it:
   - //! particles (Matteucci and Francois 1989; Chiappini et al. 1997, 2001).
   - …it into radial rings (default 24, log-spaced). Classic multi-zone chemical evolution (Matteucci and Francois 1989; Chiappini et al. 1997), with no particles.
   - Multi-zone chemical evolution (Matteucci and Francois 1989; Chiappini et al. 1997, 2001); Kennicutt 1998 star formation with a Martin and Kennicutt 2001 threshold; Ia rate Maoz and Mannucci 2012 and Maoz and Graur 2017; solar mass fractions Asplund et al. 2009…

**mcbride2009**: McBride et al. 2009 form
   What we rest on it:
   - //! - growth: M(z) = M0 (1+z)^alpha exp(beta z), the McBride et al. 2009 form,
   - //! - how massive then: the halo's own growth history (McBride et al. 2009 form,
   - - **Growth history**: M(z) = M0 · (1+z)^β · exp(−γ z) (McBride et al. 2009). β and γ are drawn with scatter correlated with environment (denser means earlier formation).

**ojakangas1989**: Ojakangas, G. W. and Stevenson, D. J. 1989, Icarus 81, 220 https://ui.adsabs.harvard.edu/abs/1989Icar...81..220O
   What we rest on it:
   - const OS: &str = "Ojakangas, G. W. and Stevenson, D. J. 1989, Icarus 81, 220: thermal state of an ice shell on Europa (conductive shell with conductivity falling as 1/T) https://ui.adsabs.harvard.edu/abs/1989Icar...81..220O";
   - // inside the 10 to 30 km range of conductive models (Ojakangas and Stevenson 1989).
   - Ojakangas, G. W. and Stevenson, D. J. 1989, Icarus 81, 220

**pierrehumbert2010**: Pierrehumbert, R. T. 2010, Principles of Planetary Climate, Cambridge University Press, sections 2.7 and 8.7. doi:10.1017/CBO9780511780783"
   DOI as cited: 10.1017/cbo9780511780783
   What we rest on it:
   - const PIERRE: &str = "Pierrehumbert, R. T. 2010, Principles of Planetary Climate, Cambridge University Press, sections 2.7 and 8.7. doi:10.1017/CBO9780511780783";
   - …he sea fraction, the ground's seasonal skin, thermal inertia I times sqrt(year / pi) (Pierrehumbert 2010, ch. 7), over the land, plus the air column c_p p / g. One ground inertia (1500 SI, between dry regolith near 200 and wet rock near 2500) on every world un…
   - …latent heat flux, a share of the sunlight reaching wet ground, falls back as liquid (Pierrehumbert 2010, ch. 9 energy budget). Sunlight reaching the ground falls with air column, fitted to Earth, Venus and Titan; cloud cover from latent flux fitted to Earth (…

**prsa2016**: Prsa et al. 2016, doi:10.3847/0004-6256/152/2/41
   DOI as cited: 10.3847/0004-6256/152/2/41
   What we rest on it:
   - - Constants: CODATA 2018, Tiesinga et al. 2021, doi:10.1103/RevModPhys.93.025010; IAU 2015 Resolution B3 nominal values, Prsa et al. 2016, doi:10.3847/0004-6256/152/2/41
   - IAU 2015 Resolution B3, nominal solar radius (Prša et al. 2016, AJ 152, 41)
   - Prša, A. et al. 2016, AJ 152, 41: nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3

**redmer2011**: Redmer et al. 2011, Icarus 211, 798. doi:10.1016/j.icarus.2010.08.008 (matter pack design)
   DOI as cited: 10.1016/j.icarus.2010.08.008
   What we rest on it:
   - …explains comfortably | Ness et al. 1986, 1989 [P20]; Stanley and Bloxham 2004 [P21]; Redmer et al. 2011 [P22]; Millot et al. 2019 [P23]; Soderlund and Stanley 2020 [P24]; Militzer 2024 [P25] |
   - - [P22] Redmer et al. 2011, Icarus 211, 798. doi:10.1016/j.icarus.2010.08.008 (matter pack design)
   - - Water's phases and the fields of Uranus and Neptune: Redmer, Mattsson, Nettelmann and French 2011, doi:10.1016/j.icarus.2010.08.008; the thin-shell explanation of their shape: Stanley and Bloxham 2004, doi:10.1038/nature02376

**sauro2020**: Sauro, F. et al. 2020, Earth-Science Reviews 209, 103288
   DOI as cited: 10.1016/j.earscirev.2020.103288
   What we rest on it:
   - //! passage is a little lower than it is wide (Sauro et al. 2020: Earth's
   - "Sauro, F. et al. 2020, Earth-Science Reviews 209, 103288: lava tubes on Earth, Moon and Mars; terrestrial tubes up to about 30 m wide. doi:10.1016/j.earscirev.2020.103288",
   - /// Typical and largest tube width on Earth, m (Sauro et al. 2020).

**showman2011**: Showman, A. P. and Polvani, L. M. 2011, Equatorial superrotation on tidally locked exoplanets, ApJ 738, 71 doi:10.1088/0004-637X/738/1/71
   DOI as cited: 10.1088/0004-637x/738/1/71
   What we rest on it:
   - why: "Twin standing gyres off the equator, set up by the day-night heating contrast on a locked world (Showman and Polvani 2011).".into(),
   - | Showman and Polvani 2011, ApJ 738, 71, doi:10.1088/0004-637X/738/1/71 | locked super-rotation | pack |
   - One face to the star: rising air and a permanent cloud shield under the star, clearer night side (Yang, Cowan and Abbot 2013; Showman and Polvani 2011).

**sossi2020**: Sossi et al. 2020's IW+0.5
   What we rest on it:
   - // its magma ocean's surface (Sossi et al. 2020); Mars more oxidised; a
   - // surface (Sossi et al. 2020), and gives off mostly water with carbon
   - temperature at which the magma ocean's surface last set its air, K (about 1700 to 2200 K; Sossi et al. 2020 used 2173 K)

**zurek1993**: Zurek, R. W. and Martin, L. J. 1993, Interannual variability of planet-encircling dust storms on Mars, JGR 98(E2), 3247-3259 doi:10.1029/92JE02936
   DOI as cited: 10.1029/92je02936
   What we rest on it:
   - why: "A planet-encircling dust storm in the dust season: dry loose ground, thin air and strong seasonal winds (Zurek and Martin 1993).".into(),
   - | Zurek and Martin 1993, JGR 98, 3247, doi:10.1029/92JE02936 | global dust storm years | pack |
   - Air so thin it heats and cools in days (radiative time under 10 days): thin ice cloud, frost, fog in basins and dust storms where there is dust (Zurek and Martin 1993).

**ballesteros2012**: Ballesteros 2012 (doi:10.1209/0295-5075/97/34008) puts a 10,000 K
   DOI as cited: 10.1209/0295-5075/97/34008
   What we rest on it:
   - // Ballesteros 2012 (doi:10.1209/0295-5075/97/34008) puts a 10,000 K
   - * Colour temperature from a B−V colour (Ballesteros 2012, EPL 97, 34008):
   - // Ballesteros 2012: the Sun's B−V of 0.65 is about 5,800 K; a B−V of 0 is about 10,000 K.

**barkana2001**: Barkana and Loeb 2001 eq. 26
   What we rest on it:
   - /// scales as (1+z)^-1.5 (Barkana and Loeb 2001 eq. 26).
   - // First stars: 3-sigma 1e6 solar-mass minihaloes near z = 20 to 30 (Tegmark et al. 1997; Barkana and Loeb 2001 fig. 6).
   - 3-sigma peaks of 1e6 solar-mass minihaloes (Tegmark et al. 1997; Barkana and Loeb 2001)

**bohren1987**: Bohren 1987, Am. J. Phys. 55, 524", url
   DOI as cited: 10.1119/1.15109
   What we rest on it:
   - // the layers by the adding method (Bohren 1987), as weather.ts does on the Body globe.
   - { what: "Reflectance of a scattering layer, two-stream, and layers combined by the adding method", cite: "Bohren 1987, Am. J. Phys. 55, 524", url: "https://doi.org/10.1119/1.15109" },
   - { what: "Reflectance of a non-absorbing scattering layer, two-stream: R = (1-g)τ / (2 + (1-g)τ); layers and ground combine by the adding method", cite: "Bohren 1987, Am. J. Phys. 55, 524", url: "https://doi.org/10.1119/1.15109" },
