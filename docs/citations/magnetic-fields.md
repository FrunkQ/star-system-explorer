# Citations: magnetic fields (track MF)

References in section 14 of the magnetic fields design (`handover/design/SSE4-magnetic-fields-design.md`,
accepted 22 Nov 2026). They were written from memory, and each row here says where the DOI was seen against the
paper's authors, year, journal, volume and page (MF0, 22 Nov 2026). "verified (metadata)" means a fetched page
(the publisher, NASA ADS, PubMed, Crossref, an institutional repository, or an arXiv page printing the journal
DOI) showed this DOI for this work. The numbers each block takes from a paper are checked when that block is
built (MF1 onwards), and get their own rows then.

Two corrections, both metadata:
- P2 (also in `crates/sse-atoms/src/magnetic.rs`, `CA06`): the DOI is 10.1111/j.1365-246X.2006.03009.x, not
  02972.x.
- G1: the year is 2016 (A&A Rev. 24, 4), not 2015.

Where the design shortens a title, the note gives the title as printed. Rows marked "secondary" were seen only on a
page that quotes the paper (Wikipedia's reference list, or an aggregator), with the metadata checked on ADS.
108 rows, none unverified.

### Planets and moons

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| P1 | Stevenson 2003, Planetary magnetic fields, EPSL 208, 1. doi:10.1016/S0012-821X(02)01126-3 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2003E%26PSL.208....1S |  |
| P2 | Christensen and Aubert 2006, GJI 166, 97. doi:10.1111/j.1365-246X.2006.02972.x (in the code) | corrected (metadata) | https://www.maths.gla.ac.uk/~rs/res/B/DynamoScaling/2006ChristensenAubert.pdf | doi:10.1111/j.1365-246X.2006.03009.x (not 02972.x); title "Scaling properties of convection-driven dynamos in rotating spherical shells and application to planetary magnetic fields" |
| P3 | Christensen, Holzwarth and Reiners 2009, Nature 457, 167. doi:10.1038/nature07626 (in the code) | verified (metadata) | https://ideas.repec.org/a/nat/nature/v457y2009i7226d10.1038_nature07626.html |  |
| P4 | Christensen 2010, Space Sci. Rev. 152, 565. doi:10.1007/s11214-009-9553-2 (in the code) | verified (metadata) | https://link.springer.com/doi/10.1007/s11214-009-9553-2 |  |
| P5 | Olson and Christensen 2006, Dipole moment scaling for convection-driven planetary dynamos, EPSL 250, 561. doi:10.1016/j.epsl.2006.08.008 (b) | verified (metadata) | https://www.maths.gla.ac.uk/~rs/res/B/DynamoScaling/2006OlsonChristensen.pdf |  |
| P6 | Driscoll and Bercovici 2014, On the thermal and magnetic histories of Earth and Venus, PEPI 236, 36. doi:10.1016/j.pepi.2014.08.004 (b) | verified (metadata) | https://api.crossref.org/works/10.1016/j.pepi.2014.08.004 | title adds ": Influences of melting, radioactivity, and conductivity" |
| P7 | Acuña et al. 1999, Science 284, 790. doi:10.1126/science.284.5415.790 | verified (metadata) | https://zenodo.org/record/1231157 |  |
| P8 | Lillis et al. 2013, Time history of the Martian dynamo from crater magnetic field analysis, JGR Planets 118, 1488. doi:10.1002/jgre.20105 | verified (metadata) | https://iro.uiowa.edu |  |
| P9 | Mittelholz et al. 2020, Timing of the martian dynamo, Science Advances 6, eaba0513. doi:10.1126/sciadv.aba0513 (b) | verified (metadata) | https://experts.umn.edu/en/publications/timing-of-the-martian-dynamo-new-constraints-for-a-core-field-45-/ | full title "Timing of the martian dynamo: New constraints for a core field 4.5 and 3.7 Ga ago" |
| P10 | Pozzo et al. 2012, Thermal and electrical conductivity of iron at Earth's core conditions, Nature 485, 355. doi:10.1038/nature11031 | verified (metadata) | https://ideas.repec.org/a/nat/nature/v485y2012i7398d10.1038_nature11031.html |  |
| P11 | Labrosse 2015, Thermal evolution of the core with a high thermal conductivity, PEPI 247, 36. doi:10.1016/j.pepi.2015.02.002 (b) | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2015PEPI..247...36L |  |
| P12 | O'Rourke and Stevenson 2016, Powering Earth's dynamo with magnesium precipitation from the core, Nature 529, 387. doi:10.1038/nature16495 | verified (metadata) | https://ideas.repec.org/a/nat/nature/v529y2016i7586d10.1038_nature16495.html |  |
| P13 | Stixrude et al. 2020, A silicate dynamo in the early Earth, Nature Communications 11, 935. doi:10.1038/s41467-020-14773-4 | verified (metadata) | https://ideas.repec.org/a/nat/natcom/v11y2020i1d10.1038_s41467-020-14773-4.html |  |
| P14 | Tarduno et al. 2010, Geodynamo, solar wind, and magnetopause 3.4 to 3.45 billion years ago, Science 327, 1238. doi:10.1126/science.1183445 | verified (metadata) | https://pure.uj.ac.za/en/publications/geodynamo-solar-wind-and-magnetopause-34-to-345-billion-years-ago/ |  |
| P15 | Christensen 2006, A deep dynamo generating Mercury's magnetic field, Nature 444, 1056. doi:10.1038/nature05342 | verified (metadata) | https://ideas.repec.org |  |
| P16 | Anderson et al. 2011, The global magnetic field of Mercury from MESSENGER orbital observations, Science 333, 1859. doi:10.1126/science.1211001 | verified (metadata) | https://maths.gla.ac.uk | copy of paper |
| P17 | Dougherty et al. 2018, Saturn's magnetic field revealed by the Cassini Grand Finale, Science 362, eaat5434. doi:10.1126/science.aat5434 | verified (metadata) | https://pubmed | 30287636 |
| P18 | Connerney et al. 2018, GRL 45, 2590. doi:10.1002/2018GL077312 (in the code) | verified (metadata) | https://ntrs.nasa.gov/citations/20180004823 |  |
| P19 | Moore et al. 2018, A complex dynamo inferred from the hemispheric dichotomy of Jupiter's magnetic field, Nature 561, 76. doi:10.1038/s41586-018-0468-5 | verified (metadata) | https://ideas.repec.org |  |
| P20 | Ness et al. 1986, Science 233, 85. doi:10.1126/science.233.4759.85; Ness et al. 1989, Science 246, 1473. doi:10.1126/science.246.4936.1473 | verified (metadata) | https://ADS | titles "Magnetic Fields at Uranus", "Magnetic Fields at Neptune" 1986Sci...233...85N; zenodo 1230994 |
| P21 | Stanley and Bloxham 2004, Nature 428, 151. doi:10.1038/nature02376 (matter pack design) | verified (metadata) | https://ideas.repec.org |  |
| P22 | Redmer et al. 2011, Icarus 211, 798. doi:10.1016/j.icarus.2010.08.008 (matter pack design) | verified (metadata) | https://copy | of paper (cfa.harvard.edu) |
| P23 | Millot et al. 2019, Nature 569, 251. doi:10.1038/s41586-019-1114-6 (matter pack design) | verified (abstract read in MP18) | https://www.nature.com/articles/s41586-019-1114-6 |  |
| P24 | Soderlund and Stanley 2020, The underexplored frontier of ice giant dynamos, Phil. Trans. R. Soc. A 378, 20190479. doi:10.1098/rsta.2019.0479 (b) | verified (metadata) | https://eld.bl.uk |  |
| P25 | Militzer 2024, Phase separation of planetary ices explains nondipolar magnetic fields of Uranus and Neptune, PNAS 121, e2403981121. doi:10.1073/pnas.2403981121 (a) | verified (metadata) | https://nature-index; | cmap.rochester.edu |
| P26 | Kivelson et al. 1996, Discovery of Ganymede's magnetic field, Nature 384, 537. doi:10.1038/384537a0 | verified (metadata) | https://nature.com | abs/384537a0 |
| P27 | Hauck, Aurnou and Dombard 2006, Sulfur's impact on core evolution and magnetic field generation on Ganymede, JGR 111, E09008. doi:10.1029/2005JE002557 | verified (metadata) | https://escholarship.org/uc/item/14x4s6kp |  |
| P28 | Kivelson et al. 2000, Galileo magnetometer measurements: a stronger case for a subsurface ocean at Europa, Science 289, 1340. doi:10.1126/science.289.5483.1340 | verified (metadata) | https://scilit.com |  |
| P29 | Zimmer, Khurana and Kivelson 2000, Subsurface oceans on Europa and Callisto, Icarus 147, 329. doi:10.1006/icar.2000.6456 | verified (metadata) | https://ntrs.nasa.gov/citations/20020046790 | title "Subsurface Oceans on Europa and Callisto: Constraints from Galileo Magnetometer Observations"; first page not seen |
| P30 | Tikoo et al. 2017, A two-billion-year history for the lunar dynamo, Science Advances 3, e1700207. doi:10.1126/sciadv.1700207 | verified (metadata) | https://dspace.mit.edu/handle/1721.1/113646?show=full |  |
| P31 | Weiss et al. 2010, Paleomagnetic records of meteorites and early planetesimal differentiation, Space Sci. Rev. 152, 341. doi:10.1007/s11214-009-9580-z | verified (metadata) | https://api.crossref.org/works/10.1007/s11214-009-9580-z |  |
| P32 | Bryson et al. 2015, Long-lived magnetism from solidification-driven convection on the pallasite parent body, Nature 517, 472. doi:10.1038/nature14114 | verified (metadata) | https://www.nature.com/articles/nature14114 |  |
| P33 | Glatzmaier and Roberts 1995, Nature 377, 203. doi:10.1038/377203a0 | verified (metadata) | https://link.springer.com/article/10.1038/377203a0 | title: A three-dimensional self-consistent computer simulation of a geomagnetic field reversal |
| P34 | Pineda and Villadsen 2023, Coherent radio bursts from known M-dwarf planet-host YZ Ceti, Nature Astronomy 7, 569. doi:10.1038/s41550-023-01914-0 (b) | verified (metadata) | https://www.nature.com/articles/s41550-023-01914-0 |  |
| P35 | Cauley et al. 2019, Magnetic field strengths of hot Jupiters from signals of star-planet interactions, Nature Astronomy 3, 1128. doi:10.1038/s41550-019-0840-x | verified (metadata) | https://arxiv.org/abs/1907.09068 |  |
| P36 | Yadav and Thorngren 2017, Estimating the magnetic field strength in hot Jupiters, ApJL 849, L12. doi:10.3847/2041-8213/aa93fd | verified (metadata) | https://astroexplorer.org |  |

### Stars

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| S1 | Noyes et al. 1984, ApJ 279, 763. doi:10.1086/161945 (in the code) | verified (metadata) | https://api.crossref.org/works/10.1086/161945 |  |
| S2 | Wright et al. 2011, ApJ 743, 48. doi:10.1088/0004-637X/743/1/48 (in the code) | verified (metadata) | https://uhra.herts.ac.uk/handle/2299/9502 |  |
| S3 | Reiners et al. 2022, Magnetism, rotation, and nonthermal emission in cool stars, A&A 662, A41. doi:10.1051/0004-6361/202243251 (a) | verified (metadata) | https://www.aanda.org/articles/aa/abs/2022/06/aa43251-22/aa43251-22.html | title ends ": Average magnetic field measurements in 292 M dwarfs" |
| S4 | Kochukhov 2021, Magnetic fields of M dwarfs, A&A Rev. 29, 1. doi:10.1007/s00159-020-00130-3 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2021A&ARv..29....1K |  |
| S5 | Wright and Drake 2016, Solar-type dynamo behaviour in fully convective stars without a tachocline, Nature 535, 526. doi:10.1038/nature18638 | verified (metadata) | https://ideas.repec.org/a/nat/nature/v535y2016i7613d10.1038_nature18638.html |  |
| S6 | Charbonneau 2020, Dynamo models of the solar cycle, Living Rev. Solar Phys. 17, 4. doi:10.1007/s41116-020-00025-6 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2020LRSP...17....4C/abstract |  |
| S7 | Brun and Browning 2017, Magnetism, dynamo action and the solar-stellar connection, Living Rev. Solar Phys. 14, 4. doi:10.1007/s41116-017-0007-8 | verified (metadata) | https://link.springer.com/10.1007/s41116-017-0007-8 |  |
| S8 | Böhm-Vitense 2007, Chromospheric activity in G and K main-sequence stars, and what it tells us about stellar dynamos, ApJ 657, 486. doi:10.1086/510482 | verified (metadata) | https://api.crossref.org/works/10.1086/510482 |  |
| S9 | Pevtsov et al. 2003, The relationship between X-ray radiance and magnetic flux, ApJ 598, 1387. doi:10.1086/378944 | verified (metadata) | https://iopscience.iop.org/article/10.1086/378944/ |  |
| S10 | Klimchuk 2006, On solving the coronal heating problem, Solar Physics 234, 41. doi:10.1007/s11207-006-0055-z | verified (metadata) | https://arxiv.org/abs/astro-ph/0511841 |  |
| S11 | Shibata and Magara 2011, Solar flares: magnetohydrodynamic processes, Living Rev. Solar Phys. 8, 6. doi:10.12942/lrsp-2011-6 | verified (metadata) | https://link.springer.com/article/10.12942/lrsp-2011-6 |  |
| S12 | Benz and Güdel 2010, Physical processes in magnetically driven flares on the Sun, stars, and young stellar objects, ARA&A 48, 241. doi:10.1146/annurev-astro-082708-101757 | verified (metadata) | https://api.crossref.org/works/10.1146/annurev-astro-082708-101757 |  |
| S13 | Aschwanden et al. 2016, 25 years of self-organized criticality: solar and astrophysics, Space Sci. Rev. 198, 47. doi:10.1007/s11214-014-0054-6 | verified (metadata) | https://link.springer.com/doi/10.1007/s11214-014-0054-6 |  |
| S14 | Shibata et al. 2013, Can superflares occur on our Sun?, PASJ 65, 49. doi:10.1093/pasj/65.3.49 | verified (metadata) | https://api.crossref.org/works/10.1093/pasj/65.3.49 |  |
| S15 | Alvarado-Gómez et al. 2018, Suppression of coronal mass ejections in active stars by an overlying large-scale stellar magnetic field, ApJ 862, 93. doi:10.3847/1538-4357/aacb7f | verified (metadata) | https://api.crossref.org/works/10.3847/1538-4357/aacb7f | title: "Suppression of Coronal Mass Ejections in Active Stars by an Overlying Large-scale Magnetic Field: A Numerical Study" |
| S16 | Weber and Davis 1967, The angular momentum of the solar wind, ApJ 148, 217. doi:10.1086/149138 | verified (metadata) | https://dx.doi.org/10.1086%2F149138 |  |
| S17 | Matt et al. 2015, The mass-dependence of angular momentum evolution in Sun-like stars, ApJL 799, L23. doi:10.1088/2041-8205/799/2/L23 | verified (metadata) | https://api.crossref.org/works/10.1088/2041-8205/799/2/L23 |  |
| S18 | van Saders et al. 2016, Weakened magnetic braking as the origin of anomalously rapid rotation in old field stars, Nature 529, 181. doi:10.1038/nature16168 (a) | verified (metadata) | https://www.nature.com/articles/nature16168 |  |
| S19 | Wood et al. 2021, New observational constraints on the winds of M dwarf stars, ApJ 915, 37. doi:10.3847/1538-4357/abfda5 | verified (metadata) | https://api.crossref.org/works/10.3847/1538-4357/abfda5 |  |
| S20 | Vidotto et al. 2014, Stellar magnetism: empirical trends with age and rotation, MNRAS 441, 2361. doi:10.1093/mnras/stu728 (b) | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2014MNRAS.441.2361V/abstract |  |
| S21 | Donati and Landstreet 2009, Magnetic fields of nondegenerate stars, ARA&A 47, 333. doi:10.1146/annurev-astro-082708-101833 | verified (metadata) | https://arxiv.org/abs/0904.1938 |  |
| S22 | Wade et al. 2016, The MiMeS survey of magnetism in massive stars, MNRAS 456, 2. doi:10.1093/mnras/stv2568 | verified (metadata) | https://dare.uva.nl/id/a9952bea-d55b-4d14-8906-92b5d975f140 |  |
| S23 | Braithwaite and Spruit 2004, A fossil origin for the magnetic field in A stars and white dwarfs, Nature 431, 819. doi:10.1038/nature02934 | verified (metadata) | https://ideas.repec.org/a/nat/nature/v431y2004i7010d10.1038_nature02934.html |  |
| S24 | Schneider et al. 2019, Stellar mergers as the origin of magnetic massive stars, Nature 574, 211. doi:10.1038/s41586-019-1621-5 | verified (metadata) | https://ideas.repec.org/a/nat/nature/v574y2019i7777d10.1038_s41586-019-1621-5.html |  |

### Discs, jets, formation

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| D1 | Koenigl 1991, Disk accretion onto magnetic T Tauri stars, ApJ 370, L39. doi:10.1086/185972 | verified (metadata) | https://api.crossref.org/works/10.1086/185972 |  |
| D2 | Hartmann, Herczeg and Calvet 2016, Accretion onto pre-main-sequence stars, ARA&A 54, 135. doi:10.1146/annurev-astro-081915-023347 | verified (metadata) | https://www2.mpia-hd.mpg.de/~beuther/hartmann2016.pdf |  |
| D3 | Balbus and Hawley 1991, ApJ 376, 214. doi:10.1086/170270 | verified (metadata) | bibbase record | title: A Powerful Local Shear Instability in Weakly Magnetized Disks. I. Linear Analysis |
| D4 | Bai and Stone 2013, Wind-driven accretion in protoplanetary disks I, ApJ 769, 76. doi:10.1088/0004-637X/769/1/76 | verified (metadata) | https://www.arxiv.org/abs/1301.0318 |  |
| D5 | Lesur et al. 2023, Hydro-, magnetohydro-, and dust-gas dynamics of protoplanetary disks, Protostars and Planets VII. https://arxiv.org/abs/2203.09821 (a) | verified (metadata) | https://arxiv.org/abs/2203.09821 | arXiv only |
| D6 | Fu et al. 2014, Solar nebula magnetic fields recorded in the Semarkona meteorite, Science 346, 1089. doi:10.1126/science.1258022 | verified (metadata) | https://orbit.dtu.dk/en/publications/solar-nebula-magnetic-fields-recorded-in-the-semarkona-meteorite/ |  |
| D7 | Weiss, Bai and Fu 2021, History of the solar nebula from meteorite paleomagnetism, Science Advances 7, eaba5967. doi:10.1126/sciadv.aba5967 | verified (metadata) | https://dspace.mit.edu/handle/1721.1/133941 |  |
| D8 | Blandford and Payne 1982, MNRAS 199, 883. doi:10.1093/mnras/199.4.883 | verified (metadata) | https://authors.library.caltech.edu/95664 |  |
| D9 | Blandford and Znajek 1977, MNRAS 179, 433. doi:10.1093/mnras/179.3.433 | verified (metadata) | https://authors.library.caltech.edu/records/2dtm0-88f56 |  |
| D10 | Event Horizon Telescope Collaboration 2021, First M87 results VII: polarization of the ring, ApJL 910, L12. doi:10.3847/2041-8213/abe71d | verified (metadata) | https://repository.arizona.edu/handle/10150/660524 |  |

### Magnetospheres, air, radiation, aurorae

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| M1 | Shue et al. 1997, A new functional form to study the solar wind control of the magnetopause size and shape, JGR 102, 9497. doi:10.1029/97JA00196 (a) | verified (metadata) | https://scilit.com/publications/23ac0703abaf5af692379043c96676b0 |  |
| M2 | Kivelson and Russell (eds) 1995, Introduction to Space Physics, Cambridge University Press (in the code) | textbook | book, no DOI |  |
| M3 | Gunell et al. 2018, Why an intrinsic magnetic field does not protect a planet against atmospheric escape, A&A 614, L3. doi:10.1051/0004-6361/201832934 (a) | verified (metadata) | https://www.aanda.org/articles/aa/abs/2018/06/aa32934-18/aa32934-18.html |  |
| M4 | Ramstad and Barabash 2021, Do intrinsic magnetic fields protect planetary atmospheres from stellar winds?, Space Sci. Rev. 217, 36. doi:10.1007/s11214-021-00791-1 (a) | verified (metadata) | https://link.springer.com/content/pdf/10.1007/s11214-021-00791-1.pdf |  |
| M5 | Jakosky et al. 2018, Loss of the Martian atmosphere to space, Icarus 315, 146. doi:10.1016/j.icarus.2018.05.030 | verified (metadata) | https://api.crossref.org/works/10.1016/j.icarus.2018.05.030 |  |
| M6 | Egan et al. 2019, Planetary magnetic field control of ion escape from weakly magnetized planets, MNRAS 488, 2108. doi:10.1093/mnras/stz1819 | verified (metadata) | https://api.crossref.org/works/10.1093/mnras/stz1819 |  |
| M7 | Smart and Shea 2005, A review of geomagnetic cutoff rigidities for earth-orbiting spacecraft, Adv. Space Res. 36, 2012. doi:10.1016/j.asr.2004.09.015 | verified (metadata) | https://api.crossref.org/works/10.1016/j.asr.2004.09.015 |  |
| M8 | Atri, Hariharan and Grießmeier 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910. doi:10.1089/ast.2013.1052 | verified (metadata) | https://api.crossref.org/works/10.1089/ast.2013.1052 |  |
| M9 | Grießmeier et al. 2005, Cosmic ray impact on extrasolar Earth-like planets in close-in habitable zones, Astrobiology 5, 587. doi:10.1089/ast.2005.5.587 | verified (metadata) | https://api.crossref.org/works/10.1089/ast.2005.5.587 |  |
| M10 | Kennel and Petschek 1966, Limit on stably trapped particle fluxes, JGR 71, 1. doi:10.1029/JZ071i001p00001 | verified (metadata) | https://rbspgway.jhuapl.edu/bibcite/reference/917 |  |
| M11 | Mauk and Fox 2010, Electron radiation belts of the solar system, JGR 115, A12220. doi:10.1029/2010JA015660 (a) | verified (metadata) | https://earthref.org/ERR/131271/ |  |
| M12 | Li and Hudson 2019, Earth's Van Allen radiation belts: from discovery to the Van Allen Probes era, JGR Space Physics 124, 8319. doi:10.1029/2018JA025940 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2019JGRA..124.8319L | title "Earth's Van Allen Radiation Belts: From Discovery to the Van Allen Probes Era" |
| M13 | Newell et al. 2007, A nearly universal solar wind-magnetosphere coupling function, JGR 112, A01206. doi:10.1029/2006JA012015 | verified (metadata) | https://earthref.org/ERR/98173/ |  |
| M14 | Feldstein and Starkov 1967, Dynamics of auroral belt and polar geomagnetic disturbances, Planet. Space Sci. 15, 209. doi:10.1016/0032-0633(67)90190-0 | verified (metadata) | https://scilit.com/publications/088570f73199ccf38ab2d9d1c394644b |  |
| M15 | Bertaux et al. 2005, Discovery of an aurora on Mars, Nature 435, 790. doi:10.1038/nature03603 | verified (metadata) | https://www.nature.com/articles/nature03603 |  |
| M16 | Zarka 2007, Planet. Space Sci. 55, 598. doi:10.1016/j.pss.2006.05.045 (in the code) | verified (metadata) | https://sciencedirect.com/science/article/abs/pii/S003206330600256X |  |
| M17 | Goldreich and Lynden-Bell 1969, Io, a Jovian unipolar inductor, ApJ 156, 59. doi:10.1086/149947 | verified (metadata) | https://scispace.com/papers/io-a-jovian-unipolar-inductor-cxadaby87q |  |
| M18 | Bagenal and Delamere 2011, Flow of mass and energy in the magnetospheres of Jupiter and Saturn, JGR 116, A05209. doi:10.1029/2010JA016294 | verified (metadata) | https://experts.colorado.edu/display/pubid_10097 |  |
| M19 | Hood and Schubert 1980, Lunar magnetic anomalies and surface optical properties, Science 208, 49. doi:10.1126/science.208.4439.49 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/1980Sci...208...49H/abstract |  |

### Remnants

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| R1 | Ferrario, de Martino and Gänsicke 2015, Magnetic white dwarfs, Space Sci. Rev. 191, 111. doi:10.1007/s11214-015-0152-0 | verified (metadata) | https://researchportalplus.anu.edu.au/en/publications/magnetic-white-dwarfs-2/ |  |
| R2 | Bagnulo and Landstreet 2021, New insight into the magnetism of degenerate stars from the analysis of a volume-limited sample of white dwarfs, MNRAS 507, 5902. doi:10.1093/mnras/stab2046 | verified (metadata) | https://arxiv.org/abs/2106.11109 |  |
| R3 | Isern et al. 2017, A common origin of magnetism from planets to white dwarfs, ApJL 836, L28. doi:10.3847/2041-8213/aa5eae (b) | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2017ApJ...836L..28I |  |
| R4 | Goldreich and Julian 1969, Pulsar electrodynamics, ApJ 157, 869. doi:10.1086/150119 | verified (metadata) | https://synapsesocial.com/papers/W4230786011 | secondary page |
| R5 | Spitkovsky 2006, Time-dependent force-free pulsar magnetospheres, ApJL 648, L51. doi:10.1086/507518 | verified (metadata) | https://www.doi.org/10.1086/507518 |  |
| R6 | Philippov and Kramer 2022, Pulsar magnetospheres and their radiation, ARA&A 60, 495. doi:10.1146/annurev-astro-052920-112338 | verified (metadata) | https://annualreviews.org/content/journals/10.1146/annurev-astro-052920-112338 |  |
| R7 | Duncan and Thompson 1992, Formation of very strongly magnetized neutron stars, ApJ 392, L9. doi:10.1086/186413 | verified (metadata) | https://en.wikipedia.org/wiki/Magnetar | secondary; ADS metadata matches (reference list) |
| R8 | Kaspi and Beloborodov 2017, Magnetars, ARA&A 55, 261. doi:10.1146/annurev-astro-081915-023329 | verified (metadata) | https://arxiv.org/abs/1703.00068 |  |
| R9 | Viganò et al. 2013, Unifying the observational diversity of isolated neutron stars via magneto-thermal evolution models, MNRAS 434, 123. doi:10.1093/mnras/stt1008 (b) | verified (metadata) | https://researchconnect.stonybrook.edu |  |
| R10 | Raynaud et al. 2020, Magnetar formation through a convective dynamo in protoneutron stars, Science Advances 6, eaay2732. doi:10.1126/sciadv.aay2732 | verified (metadata) | https://arxiv.org/abs/2003.06662 | (journal DOI printed) |
| R11 | CHIME/FRB Collaboration 2020, A bright millisecond-duration radio burst from a Galactic magnetar, Nature 587, 54. doi:10.1038/s41586-020-2863-y | verified (metadata) | https://www.nature.com/articles/s41586-020-2863-y |  |
| R12 | Hurley et al. 2005, An exceptionally bright flare from SGR 1806-20, Nature 434, 1098. doi:10.1038/nature03519 | verified (metadata) | https://www.nature.com/nature/journal/v434/n7037/full/nature03519.html |  |

### Galaxies

| Id | Reference (as in the design) | Status | Seen at | Note |
|---|---|---|---|---|
| G1 | Beck 2015, Magnetic fields in spiral galaxies, A&A Rev. 24, 4. doi:10.1007/s00159-015-0084-4 | corrected (metadata) | https://link.springer.com/article/10.1007/s00159-015-0084-4 | year 2016 (A&ARv 24:4); DOI right |
| G2 | Brandenburg and Subramanian 2005, Astrophysical magnetic fields and nonlinear dynamo theory, Physics Reports 417, 1. doi:10.1016/j.physrep.2005.06.005 | verified (metadata) | https://arxiv.org/abs/astro-ph/0405052 | (journal DOI printed) |
| G3 | Jansson and Farrar 2012, A new model of the Galactic magnetic field, ApJ 757, 14. doi:10.1088/0004-637X/757/1/14 | verified (metadata) | https://ui.adsabs.harvard.edu/abs/2012ApJ...757...14J |  |
| G4 | Crutcher 2012, Magnetic fields in molecular clouds, ARA&A 50, 29. doi:10.1146/annurev-astro-081811-125514 | verified (metadata) | https://www.doi.org/10.1146/ANNUREV-ASTRO-081811-125514 |  |
| G5 | Subramanian 2016, The origin, evolution and signatures of primordial magnetic fields, Rep. Prog. Phys. 79, 076901. doi:10.1088/0034-4885/79/7/076901 | verified (metadata) | https://synapsesocial.com/papers/W1704886924 | secondary page |
| G6 | Neronov and Vovk 2010, Evidence for strong extragalactic magnetic fields from Fermi observations of TeV blazars, Science 328, 73. doi:10.1126/science.1184192 | verified (metadata) | https://arxiv.org/abs/1006.3504 | (journal DOI printed) |
| G7 | Potgieter 2013, Solar modulation of cosmic rays, Living Rev. Solar Phys. 10, 3. doi:10.12942/lrsp-2013-3 | verified (metadata) | https://link.springer.com/article/10.12942/lrsp-2013-3 |  |