# Consensus batch 2: 50 papers (2026-11-25)

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-02.md` beside this file).

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

**tijhuis1993**: Tijhuis, L., van Loosdrecht, M. C. M. and Heijnen, J. J. 1993, Biotechnology and Bioengineering 42, 509
   DOI as cited: 10.1002/bit.260420509
   What we rest on it:
   - /// temperature, Tijhuis et al. 1993).
   - Tijhuis, L., van Loosdrecht, M. C. M. and Heijnen, J. J. 1993, Biotechnology and Bioengineering 42, 509: a thermodynamically based correlation for maintenance Gibbs energy requirements in aerobic and anaerobic chemotrophic growth (4.5 kJ per C-mol per hour at…

**urry1995**: Urry and Padovani 1995 (https://doi.org/10.1086/133630)
   DOI as cited: 10.1086/133630
   What we rest on it:
   - //! (unified model, Urry and Padovani 1995): `seen_from`.
   - …nified model Antonucci 1993 (https://doi.org/10.1146/annurev.aa.31.090193.002353) and Urry and Padovani 1995 (https://doi.org/10.1086/133630); radio-loud share Best et al. 2005 section 3.3 fit, about 1.25% at 1e8 Msun rising as M^1.6, ceiling near 30% (https:/…

**yu2000**: Yu et al. 2000 (Science 287, 637, doi:10.1126/science.287.5453.637), citation confirmed, abstract not read
   DOI as cited: 10.1126/science.287.5453.637
   What we rest on it:
   - …HM) 5.8 GPa / 1,560 = 3.7 MJ/kg, read from the Fiber-Line datasheet. Nanotube 47 from Yu et al. 2000 (Science 287, 637, doi:10.1126/science.287.5453.637), citation confirmed, abstract not read. Steel still textbook from memory, marked so on the sheet |
   - 10.1126/science.287.5453.637
   - Yu, M.-F., Lourie, O., Dyer, M. J., Moloni, K., Kelly, T. F. and Ruoff, R. S. 2000, Science 287, 637: strength and breaking mechanism of multiwalled carbon nanotubes under tensile load (outer shells broke at 11 to 63 GPa) doi:10.1126/science.287.5453.637

**antonucci1993**: Antonucci 1993 (https://doi.org/10.1146/annurev.aa.31.090193.002353) and
   DOI as cited: 10.1146/annurev.aa.31.090193.002353
   What we rest on it:
   - …2014, https://doi.org/10.1146/annurev-astro-082812-141003, to verify); unified model Antonucci 1993 (https://doi.org/10.1146/annurev.aa.31.090193.002353) and Urry and Padovani 1995 (https://doi.org/10.1086/133630); radio-loud share Best et al. 2005 section 3.…

**bock2006**: Bock, E. and Wagner, M. 2006, in The Prokaryotes, vol. 2, 457
   DOI as cited: 10.1007/0-387-30742-7_16
   What we rest on it:
   - Bock, E. and Wagner, M. 2006, in The Prokaryotes, vol. 2, 457: oxidation of inorganic nitrogen compounds as an energy source (about -275 kJ per mol for ammonium to nitrite at pH 7). https://doi.org/10.1007/0-387-30742-7_16 (recalled, to verify)

**bruneton2008**: Bruneton and Neyret 2008
   What we rest on it:
   - | **Bruneton and Neyret 2008**; `precomputed_atmospheric_scattering` | BSD-3-Clause **[from memory]** | Precomputed scattering, the classic | no | Reference; Hillaire suits many planets better |

**cameron2014**: Cameron, 2014
   What we rest on it:
   - | **Gaseous Giganticus** (S. Cameron, 2014) | Code licence not stated on its page; its parent project is GPL **[from memory]** | Gas giant pictures by moving particles through curl noise on a cube map; offline, on the processor (checked) | no | Idea only |

**kelly1999**: Kelly, D. P. 1999, Archives of Microbiology 171, 219
   DOI as cited: 10.1007/s002030050703
   What we rest on it:
   - Kelly, D. P. 1999, Archives of Microbiology 171, 219: thermodynamic aspects of energy conservation by chemolithotrophic sulfur bacteria (about -798 kJ per mol for sulfide to sulfate). https://doi.org/10.1007/s002030050703 (recalled, to verify)

**mikkelsen2022**: Mikkelsen 2022, "Practical real-time hex-tiling
   What we rest on it:
   - | **Heitz and Neyret 2018** (histogram-preserving blending); **Mikkelsen 2022**, "Practical real-time hex-tiling" | Papers; hex-tiling sample is MIT **[from memory]** | A small texture covering a planet with no visible repeat | no | Implement. Needed for groun…

**soltan1982**: Soltan 1982, MNRAS 200, 115, https://doi.org/10.1093/mnras/200.1.115
   DOI as cited: 10.1093/mnras/200.1.115
   What we rest on it:
   - …, https://doi.org/10.1086/147973); radiative efficiency 0.1 from the Soltan argument (Soltan 1982, MNRAS 200, 115, https://doi.org/10.1093/mnras/200.1.115; Yu and Tremaine 2002, https://doi.org/10.1046/j.1365-8711.2002.05532.x); thin disc peak temperature Shak…

**springel2008**: Springel et al. 2008
   What we rest on it:
   - …those already checked in the Y4 log (Giocoli et al. 2008, Boylan-Kolchin et al. 2008, Springel et al. 2008, Wetzel et al. 2013). **Every other reference and number in this document is written from memory** and must be checked at source by the citations thread…

**volonteri2010**: Volonteri 2010 (https://doi.org/10.1007/s00159-010-0029-x), direct collapse in atomic-cooling haloes
   DOI as cited: 10.1007/s00159-010-0029-x
   What we rest on it:
   - …as M^1.6, ceiling near 30% (https://doi.org/10.1111/j.1365-2966.2005.09192.x); seeds Volonteri 2010 (https://doi.org/10.1007/s00159-010-0029-x), direct collapse in atomic-cooling haloes Bromm and Loeb 2003 (https://doi.org/10.1086/377529); tidal disruption ra…

**wagman1982**: Wagman et al. 1982, J. Phys. Chem. Ref. Data 11, suppl. 2
   What we rest on it:
   - …://doi.org/10.1128/br.41.1.100-180.1977 (enthalpies of formation from the NBS tables, Wagman et al. 1982, J. Phys. Chem. Ref. Data 11, suppl. 2; values recalled, to verify: package log E0)

**wetzel2013**: Wetzel et al. 2013
   What we rest on it:
   - …in the Y4 log (Giocoli et al. 2008, Boylan-Kolchin et al. 2008, Springel et al. 2008, Wetzel et al. 2013). **Every other reference and number in this document is written from memory** and must be checked at source by the citations thread in GH0 before a block…

**wright2011**: Wright, N. J., Drake, J. J., Mamajek, E. E. and Henry, G. W. 2011, The stellar-activity-rotation relationship and the evolution of stellar dynamos, ApJ 743, 48
   DOI as cited: 10.1088/0004-637x/743/1/48
   arXiv as cited: 1109.4634
   What we rest on it:
   - /// Wright et al. 2011 parameters.
   - /// The Sun's X-ray fraction (Judge et al. 2003, as used by Wright et al. 2011).
   - /// Convective turnover time, s, from mass in kg (Wright et al. 2011 eq. 11).

**thauer1977**: Thauer, Jungermann and Decker 1977, "Energy conservation in chemotrophic anaerobic bacteria", Bacteriological Reviews. https://pmc.ncbi.nlm.nih.gov/articles/PMC413997
   DOI as cited: 10.1128/br.41.1.100-180.1977
   What we rest on it:
   - const THAUER: &str = "Thauer, R. K., Jungermann, K. and Decker, K. 1977, Bacteriological Reviews 41, 100: energy conservation in chemotrophic anaerobic bacteria (free energies of formation, appendix). doi:10.1128/br.41.1.100-180.1977";
   - // sulfate, then carbon dioxide (Thauer et al. 1977's tower).
   - …ntrations: ΔG = ΔG° + RT ln Q. This is the microbiologist's "redox tower" *[observed: Thauer et al. 1977]*, computed instead of memorised.

**vlachos2010**: Vlachos, A. (2010). Water flow in Portal 2. SIGGRAPH 2010 Advances in Real-Time Rendering (two-phase flow maps
   What we rest on it:
   - | **Perlin and Neyret 2001**, "Flow noise"; **Vlachos 2010**, "Water flow in Portal 2" (flow maps) | Papers and talk | Texture that flows along a wind field for two extra texture reads | no | Implement. The Basic form of the giants and clouds |
   - …es; Shadertoy's default licence; the simplex patent's expiry; Perlin and Neyret 2001; Vlachos 2010; Heitz and Neyret 2018; Tessendorf 2001; and the sub-claims the check file lists under "Still unverified" (Hapke's formula in a primary text, the original wind-p…
   - // Flow: two samples carried along `along`, half a cycle apart, cross-faded (Vlachos 2010).

**blandford1977**: Blandford and Znajek 1977, MNRAS 179, 433. doi:10.1093/mnras/179.3.433
   DOI as cited: 10.1093/mnras/179.3.433
   What we rest on it:
   - - Blandford and Znajek 1977; Blandford and Payne 1982 (jets; already cited in the light and MF designs).
   - | Jets | argued | Launched by field threading a disc or a spinning black hole. The Event Horizon Telescope has imaged ordered field at M87's ring | Blandford and Payne 1982 [D8]; Blandford and Znajek 1977 [D9]; EHT 2021 [D10] |
   - - [D9] Blandford and Znajek 1977, MNRAS 179, 433. doi:10.1093/mnras/179.3.433

**blandford1982**: Blandford and Payne 1982, MNRAS 199, 883. doi:10.1093/mnras/199.4.883
   DOI as cited: 10.1093/mnras/199.4.883
   What we rest on it:
   - - Blandford and Znajek 1977; Blandford and Payne 1982 (jets; already cited in the light and MF designs).
   - | Jets | argued | Launched by field threading a disc or a spinning black hole. The Event Horizon Telescope has imaged ordered field at M87's ring | Blandford and Payne 1982 [D8]; Blandford and Znajek 1977 [D9]; EHT 2021 [D10] |
   - - [D8] Blandford and Payne 1982, MNRAS 199, 883. doi:10.1093/mnras/199.4.883

**wolszczan1992**: Wolszczan, A. and Frail, D. A. 1992, A planetary system around the millisecond pulsar PSR1257+12, Nature 355, 145. doi:10.1038/355145a0"
   DOI as cited: 10.1038/355145a0
   What we rest on it:
   - const WF92: &str = "Wolszczan, A. and Frail, D. A. 1992, A planetary system around the millisecond pulsar PSR1257+12, Nature 355, 145. doi:10.1038/355145a0";
   - /// Pulsar planets (Wolszczan and Frail 1992): a few Earth masses round a
   - - Wolszczan and Frail 1992, Nature 355, 145 (planets round a pulsar).

**jiang2021**: Jiang et al. 2021, doi:10.1093/mnras/staa4034
   DOI as cited: 10.1093/mnras/staa4034
   What we rest on it:
   - //! radius on its own dynamical time (the SatGen method, Jiang et al. 2021,
   - …y for all of history. This is the method of the published satellite generator SatGen (Jiang et al. 2021, doi:10.1093/mnras/staa4034), written fresh from the paper. It is not a particle simulation: the bodies do not pull on each other, and there are tens to tho…
   - **Citations.** Read at source on 22 Nov: Jiang et al. 2021, van Dokkum et al. 2022, Kruijssen et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026…

**kruijssen2020**: Kruijssen et al. 2020, doi:10.1093/mnras/staa2452
   DOI as cited: 10.1093/mnras/staa2452
   What we rest on it:
   - …usters tell apart from another's. It is also how the real family tree was worked out (Kruijssen et al. 2020, doi:10.1093/mnras/staa2452; Aguado-Agelet et al. 2026, arXiv:2608.27611).
   - | about 11 or more | Kraken (also Heracles), near the centre; perhaps the largest by mass ratio | 2 to 5 × 10⁸ | argued: may be born here (Kruijssen et al. 2020; Horta et al. 2021, doi:10.1093/mnras/staa2987) |
   - **Citations.** Read at source on 22 Nov: Jiang et al. 2021, van Dokkum et al. 2022, Kruijssen et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026…

**oprea2026**: Oprea et al. 2026, MNRAS (doi:10.1093/mnras/stag319), RAS summary
   DOI as cited: 10.1093/mnras/stag319
   What we rest on it:
   - | Oprea et al. 2026, MNRAS (doi:10.1093/mnras/stag319), RAS summary | The hot gas halo is up to 12% warmer in the south. The Large Magellanic Cloud pulls the disc towards it at about 40 km/s, squeezing the gas on that side. The difference grew in the last 100…
   - | Hot gas | a massive companion | one side squeezed and warmer (Oprea et al. 2026) |
   - …ough ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026 (through the RAS summary), Belokurov's article, and those already checked in the Y4 log (Giocoli et al. 2008, Boylan-Kolchin et al. 2008, Springel et al. 20…

**sestito2026**: Sestito et al. 2026, MNRAS 548, stag563 (doi:10.1093/mnras/stag563)
   DOI as cited: 10.1093/mnras/stag563
   What we rest on it:
   - | Sestito et al. 2026, MNRAS 548, stag563 (doi:10.1093/mnras/stag563) | 20 very metal-poor stars within 2 kpc of the Sun, all orbiting in the plane, 11 forwards and 9 backwards, with dwarf-galaxy chemistry. Read as one small galaxy swallowed very early, tentat…
   - | very early | Loki, in the plane | small | tentative (Sestito et al. 2026) |
   - …nlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026 (through the RAS summary), Belokurov's article, and those already checked in the Y4 log (Giocoli et al. 2008, Boylan-Kolchin et al. 200…

**xiang2022**: Xiang and Rix 2022, Nature (doi:10.1038/s41586-022-04496-5), MPG summary
   DOI as cited: 10.1038/s41586-022-04496-5
   What we rest on it:
   - | Xiang and Rix 2022, Nature (doi:10.1038/s41586-022-04496-5), MPG summary | 250,000 dated stars. The thick disc began forming 13 Gyr ago, 0.8 Gyr after the Big Bang. A burst of star formation 11 Gyr ago goes with the Gaia-Enceladus merger. The thin disc from…
   - | 13 to 12.5 | first, disordered stars; then the disc spins up and the thick disc forms | (born here) | well supported (Belokurov and Kravtsov 2022, doi:10.1093/mnras/stac1267; Xiang and Rix 2022) |
   - …n et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026 (through the RAS summary), Belokurov's article, and those already checked in the Y4 log (Gi…

**dokkum2022**: van Dokkum et al. 2022, doi:10.1038/s41586-022-04665-6
   DOI as cited: 10.1038/s41586-022-04665-6
   What we rest on it:
   - …l. 2020, doi:10.3847/1538-4357/aba434; Lee et al. 2021, doi:10.3847/2041-8213/ac16e0; van Dokkum et al. 2022, doi:10.1038/s41586-022-04665-6). Rare. **Argued.**
   - **Citations.** Read at source on 22 Nov: Jiang et al. 2021, van Dokkum et al. 2022, Kruijssen et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026…

**donlon2024**: Donlon et al. 2024, doi:10.1093/mnras/stae1264
   DOI as cited: 10.1093/mnras/stae1264
   What we rest on it:
   - …i:10.1038/s41586-018-0625-x). Date argued: one group puts the debris under 3 Gyr old (Donlon et al. 2024, doi:10.1093/mnras/stae1264) |
   - …t al. 2021, van Dokkum et al. 2022, Kruijssen et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026 (through the RAS summary), Belokurov's article,…

**seager2003**: Seager, S. and Mallén-Ornelas, G. 2003, A unique solution of planet and star parameters from an extrasolar planet transit light curve, ApJ 585, 1038
   DOI as cited: 10.1086/346105
   What we rest on it:
   - const SM03: &str = "Seager, S. and Mallén-Ornelas, G. 2003, A unique solution of planet and star parameters from an extrasolar planet transit light curve, ApJ 585, 1038: depth (Rp/R*)², duration, impact parameter. doi:10.1086/346105";
   - Sources **[from memory, verify]**: Seager and Mallén-Ornelas (2003) for transit geometry and what it measures; Winn (2010), "Transits and Occultations", in *Exoplanets* (ed. S. Seager; arXiv 1001.2010); Lovis and Fischer (2010) for the wobble method; Perryman,…

**winn2010**: Winn, J. N. 2010, Transits and occultations, in Exoplanets (ed. S. Seager), University of Arizona Press
   arXiv as cited: 1001.2010
   What we rest on it:
   - const WINN10: &str = "Winn, J. N. 2010, Transits and occultations, in Exoplanets (ed. S. Seager), University of Arizona Press: transit probability R*/a, total duration. https://arxiv.org/abs/1001.2010";
   - …erify]**: Seager and Mallén-Ornelas (2003) for transit geometry and what it measures; Winn (2010), "Transits and Occultations", in *Exoplanets* (ed. S. Seager; arXiv 1001.2010); Lovis and Fischer (2010) for the wobble method; Perryman, *The Exoplanet Handbook*…

**davenport2016**: Davenport 2016
   What we rest on it:
   - …n-down with age; Kepler-mission flare statistics (Maehara et al. 2012 on superflares; Davenport 2016); Blandford and Znajek (1977) and Blandford and Payne (1982) for jet launching; Lammer et al. and Owen and Wu (2013, 2017) for atmospheric loss under stellar X…

**gaudi2012**: Gaudi (2012) for microlensing
   What we rest on it:
   - …wobble method; Perryman, *The Exoplanet Handbook*, for all methods and their limits; Gaudi (2012) for microlensing; Wolszczan and Frail (1992) for pulsar planets.

**geoscience2015**: Geoscience 2015
   What we rest on it:
   - …rueshaber, Sayanagi and Dowling, Icarus 2019, and O'Neill, Emanuel and Flierl, Nature Geoscience 2015; from memory, verify titles and thresholds before use]**

**maehara2012**: Maehara et al. 2012 on superflares
   What we rest on it:
   - …sby number; Skumanich (1972) for spin-down with age; Kepler-mission flare statistics (Maehara et al. 2012 on superflares; Davenport 2016); Blandford and Znajek (1977) and Blandford and Payne (1982) for jet launching; Lammer et al. and Owen and Wu (2013, 2017)…

**owen2013**: Owen and Wu (2013, 2017) for atmospheric loss under stellar X-ray and ultraviolet light
   What we rest on it:
   - …and Znajek (1977) and Blandford and Payne (1982) for jet launching; Lammer et al. and Owen and Wu (2013, 2017) for atmospheric loss under stellar X-ray and ultraviolet light.

**osterbrock2006**: Osterbrock, D. E. and Ferland, G. J. 2006, Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd edn, University Science Books https://ui.adsabs.harvard.edu/abs/2006agna.book.....O
   What we rest on it:
   - const OF06: &str = "Osterbrock, D. E. and Ferland, G. J. 2006, Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd edn, University Science Books: chapter 2 (Strömgren sphere; case B recombination coefficient 2.59e-13 cm³/s at 10⁴ K) and chapter 4 (…
   - /// over the photon-energy ratio = 0.45 (Osterbrock and Ferland 2006, tables 4.1, 4.2).
   - /// a sphere about 3.1 pc across in radius (Osterbrock and Ferland 2006, sec 2.3).

**peck1972**: Peck, E. R. and Reeder, K. 1972, Dispersion of air, J. Opt. Soc. Am. 62, 958 doi:10.1364/JOSA.62.000958
   DOI as cited: 10.1364/josa.62.000958
   What we rest on it:
   - /// Dry-air refractivity n − 1 at `lambda_m` (Peck and Reeder 1972), at N_S_288.
   - const PR72: &str = "Peck, E. R. and Reeder, K. 1972, Dispersion of air, J. Opt. Soc. Am. 62, 958: (n−1)×10⁸ = 8060.51 + 2480990/(132.274 − λ⁻²) + 17455.7/(39.32957 − λ⁻²), λ in µm, 288.15 K, 101325 Pa. doi:10.1364/JOSA.62.000958";
   - Peck and Reeder 1972 eq. 2 (wavelength in micrometres)

**hurley2000**: Hurley, Pols and Tout 2000, MNRAS 315, 543, doi:10.1046/j.1365-8711.2000.03426.x
   DOI as cited: 10.1046/j.1365-8711.2000.03426.x
   What we rest on it:
   - How::Calculated { source: "Hurley, Pols and Tout 2000, MNRAS 315, 543; Tout et al. 1996, MNRAS 281, 257".into() },
   - block: "Hurley 2000 fits".into(),
   - //! Coefficient tables of Hurley, Pols and Tout 2000 (MNRAS 315, 543), appendix,

**thompson1987**: Thompson, P., Cox, D. E. and Hastings, J. B. 1987, J. Appl. Cryst. 20, 79 doi:10.1107/S0021889887087090
   DOI as cited: 10.1107/s0021889887087090
   What we rest on it:
   - Thompson, Cox and Hastings 1987 eqs. 3 and 4
   - Thompson, P., Cox, D. E. and Hastings, J. B. 1987, J. Appl. Cryst. 20, 79
   - Thompson, P., Cox, D. E. and Hastings, J. B. 1987, J. Appl. Cryst. 20, 79 doi:10.1107/S0021889887087090

**oberg2011**: Oberg, K. I., Murray-Clay, R. and Bergin, E. A. 2011, ApJL 743, L16
   DOI as cited: 10.1088/2041-8205/743/1/l16
   What we rest on it:
   - pub const REF_OBERG: &str = "Oberg, K. I., Murray-Clay, R. and Bergin, E. A. 2011, ApJL 743, L16: H2O 135 K, CO2 47 K, CO 20 K";
   - /// water loses the oxygen that extra carbon locks into CO (Oberg 2011 figure 2,
   - split of protosolar Z by carrier (Asplund 2009, Oberg 2011)

**shue1998**: Shue, J.-H. et al. 1998, Magnetopause location under extreme solar wind conditions, J. Geophys. Res. 103, 17691, doi:10.1029/98JA01103
   DOI as cited: 10.1029/98ja01103
   What we rest on it:
   - /// Shue et al. 1998 flaring exponent, from the wind's ram pressure (Pa) and
   - const SHUE98: &str = "Shue, J.-H. et al. 1998, Magnetopause location under extreme solar wind conditions, J. Geophys. Res. 103, 17691: r = r₀ (2 / (1 + cos θ))^α, α = (0.58 − 0.007 B_z)(1 + 0.024 ln P_dyn), B_z in nT, P_dyn in nPa. doi:10.1029/98JA01103";
   - /// (Shue et al. 1998), Mercury about 1.5 (Anderson et al. 2011, doi:10.1126/science.1211001).

**bucholtz1995**: Bucholtz, A. 1995, Rayleigh-scattering calculations for the terrestrial atmosphere, Applied Optics 34, 2765. doi:10.1364/AO.34.002765"
   DOI as cited: 10.1364/ao.34.002765
   What we rest on it:
   - const BUCH95: &str = "Bucholtz, A. 1995, Rayleigh-scattering calculations for the terrestrial atmosphere, Applied Optics 34, 2765. doi:10.1364/AO.34.002765";
   - /// Bodhaine et al. 1999 (4.5105e-31 m², τ = 0.0971) and Bucholtz 1995.
   - /// Earth from its gases against Bucholtz 1995 (4.84, 11.49 and 28.6 per

**nordlund2009**: Nordlund, Å., Stein, R. F. and Asplund, M. 2009, Solar surface convection, Living Rev. Solar Phys. 6, 2 doi:10.12942/lrsp-2009-2
   DOI as cited: 10.12942/lrsp-2009-2
   What we rest on it:
   - /// Surface cells against the Sun's granules (Nordlund et al. 2009: 1 to 2
   - const NSA09: &str = "Nordlund, Å., Stein, R. F. and Asplund, M. 2009, Solar surface convection, Living Rev. Solar Phys. 6, 2: granules about 1 to 2 Mm across living minutes, with upflows of a few km/s near the surface. doi:10.12942/lrsp-2009-2";
   - param("v_sun", D::VELOCITY, 2000.0, (500.0, 5000.0), "the Sun's near-surface convective speed, m/s (a few km/s; Nordlund et al. 2009)"),

**rybicki1979**: Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics doi:10.1002/9783527618170
   DOI as cited: 10.1002/9783527618170
   What we rest on it:
   - const RL79: &str = "Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 1: photon numbers from Planck's law. doi:10.1002/9783527618170";
   - pub const R_PLANCK: &str = "Planck's law and the Stefan-Boltzmann law: Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 1";
   - pub const R_RL79_SHIFT: &str = "Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 4: Doppler factor, relativistic beaming and the transformation of specific intensity (I_ν / ν³ invariant)";

**blair2017**: Blair, D. M. et al. 2017, Icarus 282, 47
   DOI as cited: 10.1016/j.icarus.2016.10.008
   What we rest on it:
   - //! passes the widest roof that holds itself up (Blair et al. 2017). The
   - "Blair, D. M. et al. 2017, Icarus 282, 47: the structural stability of lunar lava tubes; a roof spans wider as gravity falls. doi:10.1016/j.icarus.2016.10.008",
   - …1998). Gravity {g:.2} m/s² lets a roof span about {w:.0} m ({ratio:.1} times Earth's; Blair et al. 2017). {kept_pc:.0}% of the roof still stands: {imp}{wear}.",

**cardelli1989**: Cardelli, J. A., Clayton, G. C. and Mathis, J. S. 1989, The relationship between infrared, optical, and ultraviolet extinction, ApJ 345, 245 https://doi.org/10.1086/167900
   DOI as cited: 10.1086/167900
   What we rest on it:
   - const CCM89: &str = "Cardelli, J. A., Clayton, G. C. and Mathis, J. S. 1989, The relationship between infrared, optical, and ultraviolet extinction, ApJ 345, 245: A_λ/A_V = a(x) + b(x)/R_V, x = 1/λ in µm⁻¹, valid 0.3 to 10 µm⁻¹; R_V = 3.1 for the diffuse inter…
   - pub const R_CCM89: &str = "Cardelli, J. A., Clayton, G. C. and Mathis, J. S. 1989, The relationship between infrared, optical, and ultraviolet extinction, ApJ 345, 245. doi:10.1086/167900";
   - …ARA&A 41, 241. doi:10.1146/annurev.astro.41.011802.094840); the optical shape follows Cardelli, Clayton and Mathis 1989, ApJ 345, 245. doi:10.1086/167900";

**raghavan2010**: Raghavan, D. et al. 2010, ApJS 190, 1
   arXiv as cited: 1007.0414
   What we rest on it:
   - /// (Raghavan et al. 2010), mass ratio uniform from q_min to 1.
   - // Raghavan et al. 2010: triples 8 and higher orders 3 of 44 percent of
   - block: "Duchene and Kraus 2013, Raghavan 2010".into(),

**rudnick2003**: Rudnick, R. L. and Gao, S. 2003, Composition of the continental crust, Treatise on Geochemistry vol. 3, 1, Table 11, bulk crust (read from the pyrolite package's copy, checked against the 2014 edition) doi:10.1016/B0-08-043751-6/03016-4
   DOI as cited: 10.1016/b0-08-043751-6/03016-4
   What we rest on it:
   - pub const RG03: &str = "Rudnick, R. L. and Gao, S. 2003, Treatise on Geochemistry 3, 1: composition of the continental crust. doi:10.1016/B0-08-043751-6/03016-4";
   - Rudnick, R. L. and Gao, S. 2003, Treatise on Geochemistry 3, 1
   - Rudnick, R. L. and Gao, S. 2003, Treatise on Geochemistry 3, 1 doi:10.1016/B0-08-043751-6/03016-4

**zeitlin2013**: Zeitlin, C. et al. 2013, Measurements of energetic particle radiation in transit to Mars on the Mars Science Laboratory, Science 340, 1080. doi:10.1126/science.1235989
   DOI as cited: 10.1126/science.1235989
   What we rest on it:
   - const ZEIT13: &str = "Zeitlin, C. et al. 2013, Measurements of energetic particle radiation in transit to Mars on the Mars Science Laboratory, Science 340, 1080: 1.84 mSv per day in interplanetary space near 1 AU. doi:10.1126/science.1235989";
   - /// (MSL in cruise near 1 AU, Zeitlin et al. 2013).
   - //! shielding (Zeitlin et al. 2013, Science 340, 1080, doi:10.1126/science.1235989:

**adriani2018**: Adriani et al. 2018, *Nature* 555, "Clusters of cyclones encircling Jupiter's poles", doi:10.1038/nature25491 [existence checked]
   DOI as cited: 10.1038/nature25491
   What we rest on it:
   - "The ring holds {ring_count} cyclones, drawn from {:?} (Juno saw 8 around Jupiter's north pole and 5 around its south: Adriani et al. 2018).",
   - …g**: Juno found eight cyclones around Jupiter's north pole and five around its south (Adriani et al. 2018, *Nature* 555, "Clusters of cyclones encircling Jupiter's poles", doi:10.1038/nature25491 **[existence checked]**). SSE4 draws the count from the regime,…
   - | Adriani et al. 2018, Nature 555, 216, doi:10.1038/nature25491 | Jupiter's cyclone rings | pack |

**coles1993**: Coles, Melott and Shandarin 1993, MNRAS 260, 765
   What we rest on it:
   - /// Truncated Zel'dovich approximation (Coles, Melott and Shandarin 1993,
   - //! parcels (Coles, Melott and Shandarin 1993) and lands on the filaments and
   - …rown linearly; each parcel of matter moved by the truncated Zel'dovich approximation (Coles, Melott and Shandarin 1993) and given the room that step leaves it; voids, sheets, filaments and knots by the T-web, its threshold set so voids fill the pack's share of…
