# Citations: master data sheet (track MP)

Sources the master data sheet (`packs/base-reference`, design `handover/design/SSE4-matter-pack-design.md`
section 11) will cite. The design's DOIs were written from memory; each row says what was read to check it
(21 Nov 2026, MP2). A sheet row's `src` names the id in the first column.

"verified (metadata)" here means the DOI resolves to the work named: either a page that states the DOI was read,
or a search result showed the DOI's own landing page under that title. Numbers are checked row by row as the
sheet is filled, and each sheet row records that check.

## A. Works the sheet cites

| Id | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| codata2018 | Tiesinga, E., Mohr, P. J., Newell, D. B. and Taylor, B. N. 2021, Rev. Mod. Phys. 93, 025010: CODATA recommended values of the fundamental physical constants, 2018. doi:10.1103/RevModPhys.93.025010 | every constant of nature | verified (metadata) | DOI landing page title at link.aps.org/doi/10.1103/RevModPhys.93.025010; NIST PDF at physics.nist.gov/cuu/pdf/RevModPhys.93.025010.pdf |
| iau2015b3 | Prša, A. et al. 2016, AJ 152, 41: Nominal values for selected solar and planetary quantities, IAU 2015 Resolution B3. doi:10.3847/0004-6256/152/2/41 | nominal Sun, Earth and Jupiter values and GM | verified (metadata) | ADS 2016AJ....152...41P; arXiv 1605.09788 |
| iupac2021 | Prohaska, T. et al. 2022, Pure Appl. Chem. 94, 573: Standard atomic weights of the elements 2021 (IUPAC Technical Report). doi:10.1515/pac-2019-0603 | atomic weights of all elements; molar masses worked out from formulas | verified (metadata) | De Gruyter landing page for doi:10.1515/pac-2019-0603; USGS record 70236140 |
| lodders2021 | Lodders, K. 2021, Space Sci. Rev. 217, 44: Relative atomic solar system abundances, mass fractions, and atomic masses of the elements and their isotopes. doi:10.1007/s11214-021-00825-8 | element and isotope abundances | verified (metadata) | Springer landing page link.springer.com/article/10.1007/s11214-021-00825-8 |
| lodders2003 | Lodders, K. 2003, ApJ 591, 1220: Solar system abundances and condensation temperatures of the elements. doi:10.1086/375492 | 50% condensation temperatures | verified (metadata) | ADS 2003ApJ...591.1220L; doi.org/10.1086/375492 |
| wood2019 | Wood, B. J., Smythe, D. J. and Harrison, T. 2019 (authors not yet read from the page), Am. Mineral. 104, 844: The condensation temperatures of the elements: a reappraisal. doi:10.2138/am-2019-6852CCBY | updated condensation temperatures | verified (metadata) | De Gruyter landing page for the DOI; MSA open-access PDF AM104P0844 |
| kobayashi2020 | Kobayashi, C., Karakas, A. I. and Lugaro, M. 2020, ApJ 900, 179: The origin of elements from carbon to uranium. doi:10.3847/1538-4357/abae65 | which process makes each element (galaxy rings' groups) | verified (metadata) | IOP landing page for the DOI; ADS 2020ApJ...900..179K |
| rudnick2014 | Rudnick, R. L. and Gao, S. 2014, Composition of the continental crust, chapter 3.01 in Treatise on Geochemistry, 2nd edn, vol. 4, pp. 1-51, Elsevier. doi:10.1016/B978-0-08-095975-7.00301-6 | crust composition (granite and crust mixtures) | unverified (DOI) | author PDF (geol.umd.edu/~rudnick/PDF/Rudnick_Gao_Treatise.pdf) shows chapter 3.01 by Rudnick and Gao, which fits the DOI's "00301" suffix; the DOI itself was not seen on any page read |
| fray2009 | Fray, N. and Schmitt, B. 2009, Planet. Space Sci. 57, 2053-2080: Sublimation of ices of astrophysical interest, a bibliographic review. doi:10.1016/j.pss.2009.09.011 | vapour-pressure fits of the ices (MP3, W8): N2 (two), CO (two), CO2 (two), CH4, NH3, Ar, and water by Feistel and Wagner 2007 | verified (coefficients) | coefficients read from Lisse et al. 2021 SOM Table 2 (lisse2021); the paper itself is behind a bot check on HAL. Pressure unit (bar) and which fit covers which range worked out by passing each through its own triple point (all within 3 %); the lower ends of the ranges are from memory |
| hp2011 | Holland, T. J. B. and Powell, R. 2011, J. Metamorph. Geol. 29, 333-383: An improved and extended internally consistent thermodynamic dataset for phases of petrological interest. doi:10.1111/j.1525-1314.2010.00923.x | mineral formation energies, densities, heat capacities | verified (metadata) | running footer of the article PDF (pages.uoregon.edu mirror) |
| hitran2020 | Gordon, I. E. et al. 2022, JQSRT 277, 107949: The HITRAN2020 molecular spectroscopic database. doi:10.1016/j.jqsrt.2021.107949 | gas absorption bands | verified (metadata) | ODU Digital Commons record chemistry_fac_pubs/219 |
| adams2008 | Adams, F. C. 2008, JCAP 08, 010: Stars in other universes, stellar structure with different fundamental constants. doi:10.1088/1475-7516/2008/08/010 | which blocks follow a changed constant (design section 7A) | verified (metadata) | arXiv 0807.3697 abstract page, which gives the DOI |
| nist-webbook | NIST Chemistry WebBook, SRD 69. https://webbook.nist.gov/chemistry/ | gas phase points, formation enthalpies | textbook | standard reference database; per-substance pages checked when each row is filled |

## A2. Further sources the first-wave sheet cites

| Id | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| lisse2021 | Lisse, C. M. et al. 2021, Icarus 356, 114072: On the origin and thermal stability of Arrokoth's and Pluto's ices; Supplementary Online Material. https://ntrs.nasa.gov/api/citations/20210019502/downloads/Supplementary%20Online%20Material%2020210019502.pdf | SOM Table 2 reproduces Fray and Schmitt 2009's fits | verified (table read) | SOM PDF read 21 Nov 2026; volume and article number from memory |
| si2019 | SI Brochure, 9th edition 2019, BIPM. https://www.bipm.org/en/publications/si-brochure | the seven exact defining constants | verified (values) | CODATA 2018 Table I read verbatim: c, h, e, k, N_A exact values |
| nubase2020 | Kondev, F. G. et al. 2021, Chinese Phys. C 45, 030001: the NUBASE2020 evaluation. doi:10.1088/1674-1137/abddae | half-lives beyond the heat producers | verified (metadata); values unverified | CPC landing page for the DOI in search results |
| iau2012b2 | IAU 2012 Resolution B2 (au = 149 597 870 700 m). https://www.iau.org/static/resolutions/IAU2012_English.pdf | astronomical unit | unverified | URL from memory; the value matches every code copy (MP0) |
| iau2015b2 | IAU 2015 Resolution B2 (parsec = 648000/π au). https://www.iau.org/static/resolutions/IAU2015_English.pdf | parsec | unverified | as stated in the Prša 2016 summary read; resolution not read |
| oberg2011 | Öberg, Murray-Clay and Bergin 2011, ApJL 743, L16. doi:10.1088/2041-8205/743/1/L16 | ice lines 135, 47, 20 K | unverified | DOI from memory; already in orbits-formation register as "from memory" |
| peck-reeder1972 | Peck, E. R. and Reeder, K. 1972, JOSA 62, 958: dispersion of air. doi:10.1364/JOSA.62.000958 | dry-air refractivity 2.926e-4 | unverified | DOI from memory |
| bodhaine1999 | Bodhaine, B. A. et al. 1999, J. Atmos. Oceanic Technol. 16, 1854. doi:10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2 | King factors for N2, O2, CO2, air | verified | carried from the light register |
| atom-isotropic | King factor 1 for a spherically symmetric atom (Ar, He, Ne) | King factor | textbook | physics: an isotropic polarisability gives no depolarisation |
| thauer1977 | Thauer, Jungermann and Decker 1977, Bacteriol. Rev. 41, 100. https://pmc.ncbi.nlm.nih.gov/articles/PMC413997 | free energies and enthalpies of formation | verified (metadata) | PMC record in search results; values carried from base-life |
| ussa1976 | U.S. Standard Atmosphere 1976. https://ntrs.nasa.gov/citations/19770009539 | dry-air composition | verified (metadata); recipe unverified | NTRS record in search results |
| lineweaver2010 | Lineweaver, C. H. and Norman, M. 2010: the potato radius, a lower minimum size for dwarf planets. https://arxiv.org/abs/1004.1091 | rock and ice yield strengths | verified (metadata) | arXiv PDF 1004.1091 in search results under that title |
| nssdca-earth, -moon, -jupiter | NASA NSSDCA planetary fact sheets | mean radii, Moon mass and distance | textbook | standard fact sheets; values match the MP0 canonical column |
| cgpm1901, cgpm1954 | CGPM resolutions on standard gravity and the standard atmosphere | g0, atm | textbook | exact by definition |
| fudge-register | docs/fudges.md | authored stand-ins | authored | each authored value says why in the fudge register |

## A3. Where each element was made (origin shares and stories)

| Id | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| kobayashi2020 | Kobayashi, Karakas and Lugaro 2020, ApJ 900, 179. doi:10.3847/1538-4357/abae65 | group statements for C, N, F (about half AGB) and the iron peak (about half Type Ia); the r-process shortfall | verified (text) | arXiv 2008.04660 introduction and abstract read verbatim |
| johnson-origins | Johnson, J. A., Origin of the Elements in the Solar System (CC BY-SA 4.0). https://www.astronomy.ohio-state.edu/johnson.3064/nucleo/index.html | the six-source legend | verified (page) | page and the SDSS blog post (2017-01-09) read; shares exist only as image areas, no data |
| bisterzo2014 | Bisterzo, S. et al. 2014, ApJ 787, 10. doi:10.1088/0004-637X/787/1/10 | heavy-element s-process shares | unverified (values) | ar5iv read to the introduction (weak s about 10% of Sr; LEPP about 8% Sr, 18% Y and Zr); the per-element table was not reached; VizieR J/ApJ/787/10 holds it |
| kasen2017 | Kasen, D. et al. 2017, Nature 551, 80. doi:10.1038/nature24453 | a neutron-star merger making heavy elements, seen once | verified (metadata) | DOI in the RePEc record in search results |
| abbott2017 | Abbott, B. P. et al. 2017, PRL 119, 161101. doi:10.1103/PhysRevLett.119.161101 | GW170817 | verified (metadata) | ADS 2017PhRvL.119p1101A |
| fields2020 | Fields, Olive, Yeh and Young 2020, JCAP 03, 010. https://arxiv.org/abs/1912.01132 | primordial helium; the lithium problem | verified (metadata) | arXiv record under that title |

## A4. Water's phases (MP18)

| Id | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| iapws2011-meltsub | IAPWS R14-08(2011); Wagner, Riethmann, Feistel and Harvey 2011, J. Phys. Chem. Ref. Data 40, 043103. doi:10.1063/1.3657937 | melting curves of Ih, III, V, VI and VII; sublimation of Ih; the four liquid triple points | verified | release text read (iapws.org MeltSub download); all six check values reproduce |
| iapws1992-sat | IAPWS SR1-86(1992); Wagner and Pruss 1993, J. Phys. Chem. Ref. Data 22, 783. doi:10.1063/1.555926 | boiling curve, critical point, saturated liquid density at 273.16 K | verified | NIST reprint read; check values at 273.16 K and 373.1243 K reproduce (a5 = -15.9618719) |
| frank2004 | Frank, Fei and Hu 2004, Geochim. Cosmochim. Acta 68, 2781 | ice VII melting above 585 K (Simon fit, 3 to 60 GPa) | verified (fit) | paper read (cfa.harvard.edu copy); Table 2 points reproduce; DOI not printed in that copy |
| lin2005 | Lin et al. 2005, Geophys. Res. Lett. 32, L11306. doi:10.1029/2005GL022499 | break in the melting curve near 35 GPa; ice VII to X at 60 GPa, 300 K (quoting Goncharov 1999) | verified | paper read |
| journaux2020 | Journaux et al. 2020, JGR Planets 125, e2019JE006176. doi:10.1029/2019JE006176 | compression of ices III, V, VI (Table 3) | verified (values) | arXiv 1907.09598 read; DOI from the journal citation (not printed in the arXiv copy) |
| chaplin-lsbu | Chaplin, Water Structure and Science (LSBU): phase diagram and ice phases pages | solid-solid triple points; densities, ordering of every ice | verified (secondary) | pages read; each triple point names a primary paper there, not yet read one by one |
| wikipedia-ice-phases | Wikipedia, Phases of ice | cross-check only | verified (secondary) | read 22 Nov 2026 |
| salzmann2019 | Salzmann 2019, J. Chem. Phys. 150, 060901. doi:10.1063/1.5085163 | ices XVI, XVII; amorphous ices; hydrogen order | verified | UCL copy read; it holds no triple-point table |
| millot2019 | Millot et al. 2019, Nature 569, 251. doi:10.1038/s41586-019-1114-6 | fcc superionic ice XVIII at 100-400 GPa, 2000-3000 K; ionic conductivity above 100 S/cm | verified (abstract) | abstract read; body paywalled |
| millot2018 | Millot et al. 2018, Nature Physics 14, 297. doi:10.1038/s41567-017-0017-4 | superionic melting near 5000 K at 190 GPa | unverified | not read; the point is recalled |
| cheng2021 | Cheng, Bethkenhagen, Pickard and Hamel 2021, arXiv:2103.09035 (Nature Physics 17, 1228) | bcc below about 75 GPa; onset near 2000 K; melting 3250 K at 100 GPa | verified (text) | arXiv read; boundaries only in figures; journal DOI not printed |
| haldemann2020 | Haldemann, Alibert, Mordasini and Benz 2020, A&A 643, A105 (AQUA) | ice VI-VII fit as a cross-check | verified (text) | arXiv 2009.10098 read; its melting fits came through garbled and are not used |
| redmer2011 | Redmer, Mattsson, Nettelmann and French 2011, Icarus 211, 798. doi:10.1016/j.icarus.2010.08.008 | ionic-fluid edge; ionic water conductivity | unverified | not read; recalled |
| mattsson2006 | Mattsson and Desjarlais 2006, PRL 97, 017801. doi:10.1103/PhysRevLett.97.017801 | ionic-fluid edge | unverified | not read; recalled |
| recalled | (none) | pure water's conductivity, 5.5e-6 S/m at 25 C | unverified | textbook figure, to read from a source |

## A5. The Cosmos group (DM1)

Galaxy histories design section 3.2 (`handover/design/SSE4-galaxy-histories-design.md`). Checked 22 Nov 2026:
metadata from api.crossref.org/works/<doi> (first author, year, journal, volume, page and title all as cited);
Planck's numbers read from the journal's own Table 2 page.

| Id | Reference | Relied on | Status | Checked |
|---|---|---|---|---|
| planck2018 | Planck Collaboration 2020, A&A 641, A6: Planck 2018 results VI, cosmological parameters. doi:10.1051/0004-6361/201833910 | H0, Omega_b, Omega_c, Omega_m, sigma8, n_s, Omega_Lambda (check) | verified (numbers) | aanda.org full_html/2020/09/aa33910-18/T2.html, last column (TT,TE,EE+lowE+lensing+BAO): Omega_b h2 0.02242, Omega_c h2 0.11933, n_s 0.9665, H0 67.66, Omega_Lambda 0.6889, Omega_m 0.3111, Omega_m h2 0.14240, sigma8 0.8102. One 0.06 eV neutrino (section 2.1, arXiv text). **Correction to the design:** its Omega_c 0.2621 with Omega_c h2 0.1200 is the lensing-only column; the column the cosmology has always used gives 0.2607 |
| fixsen2009 | Fixsen, D. J. 2009, ApJ 707, 916. doi:10.1088/0004-637X/707/2/916 | T_cmb 2.7255 K | verified (metadata) | Crossref; the value is from memory |
| mangano2005 | Mangano, G. et al. 2005, Nucl. Phys. B 729, 221. doi:10.1016/j.nuclphysb.2005.09.041 | N_eff 3.046; Omega_nu h2 = sum of masses / 93.14 eV | verified (metadata); numbers unverified | Crossref. Planck's abstract quotes 3.046 as the Standard Model prediction |
| chevallier2001 | Chevallier, M. and Polarski, D. 2001, Int. J. Mod. Phys. D 10, 213. doi:10.1142/S0218271801000822 | the w0, wa form | verified (metadata) | Crossref |
| linder2003 | Linder, E. V. 2003, Phys. Rev. Lett. 90, 091301. doi:10.1103/PhysRevLett.90.091301 | the w0, wa form | verified (metadata) | Crossref |
| desi2024 | DESI Collaboration (Adame, A. G. et al.) 2025, JCAP 02, 021: DESI 2024 VI. doi:10.1088/1475-7516/2025/02/021 | evolving dark energy is argued | verified (metadata) | Crossref |
| schneider2012 | Schneider, A. et al. 2012, MNRAS 424, 684. doi:10.1111/j.1365-2966.2012.21252.x | half-mode mass (warm dark matter) | verified (metadata) | Crossref |
| irsic2017 | Iršič, V. et al. 2017, Phys. Rev. D 96, 023522. doi:10.1103/PhysRevD.96.023522 | warm dark matter limit | verified (metadata) | Crossref; the 5 to 6.5 keV figures are from the design, unverified |
| nadler2021 | Nadler, E. O. et al. 2021, Phys. Rev. Lett. 126, 091101. doi:10.1103/PhysRevLett.126.091101 | warm dark matter limit | verified (metadata) | Crossref |
| rocha2013 | Rocha, M. et al. 2013, MNRAS 430, 81. doi:10.1093/mnras/sts514 | self-interaction gives a core | verified (metadata) | Crossref |
| harvey2015 | Harvey, D. et al. 2015, Science 347, 1462. doi:10.1126/science.1261381 | cluster limit on self-interaction | verified (metadata) | Crossref |
| randall2008 | Randall, S. W. et al. 2008, ApJ. doi:10.1086/587859 | cluster limit on self-interaction | verified | Crossref: ApJ 679, 1173 (2008), matches |
| kaplinghat2016 | Kaplinghat, M., Tulin, S. and Yu, H.-B. 2016, Phys. Rev. Lett. 116, 041302. doi:10.1103/PhysRevLett.116.041302 | velocity-dependent self-interaction | verified (metadata) | Crossref |
| hui2017 | Hui, L. et al. 2017, Phys. Rev. D 95, 043541. doi:10.1103/PhysRevD.95.043541 | fuzzy dark matter | verified (metadata) | Crossref |
| clowe2006 | Clowe, D. et al. 2006, ApJ 648, L109. doi:10.1086/508162 | the particle route: mass apart from gas | verified (metadata) | Crossref |
| boran2018 | Boran, S. et al. 2018, Phys. Rev. D 97, 041501. doi:10.1103/PhysRevD.97.041501 | the particle route: GW170817 | verified (metadata) | Crossref |
| des2025 | DES Collaboration (Abbott, T. M. C. et al.) 2025, Phys. Rev. D 112, 083535. doi:10.1103/3dzh-d8f5 | the standard picture passes new tests | verified (metadata) | Crossref; the "16,000 clusters" figure is from the design, unverified |
| vijaykumar2021 | Vijaykumar, A., Kapadia, S. J. and Ajith, P. 2021, Phys. Rev. Lett. 126, 141104. doi:10.1103/PhysRevLett.126.141104 | G has not changed with time | verified (metadata) | Crossref (title: binary neutron stars) |
| sakstein2026 | Sakstein, J. and Desmond, H. 2026, arXiv:2610.04048 | red dwarfs rule out recent steps in G | verified | arXiv abstract page: steps excluded at 3.1 and 4.0 sigma |
| mcgaugh2016 | McGaugh, S. S., Lelli, F. and Schombert, J. M. 2016, Phys. Rev. Lett. 117, 201101. doi:10.1103/PhysRevLett.117.201101 | the check on haloes (radial acceleration relation) | verified (metadata) | Crossref |

## B. The four "from memory" gas refractivities

`packs/base-light/pack.json` gases, note: "Ammonia, sulphur dioxide, carbon monoxide and neon: refractivity
from Kaye and Laby at 589 nm (from memory, verify)". The pack uses 0 °C and 1 atm. The Particle Data Group's
gas pages take their indices from Kaye and Laby (dry gas, 0 °C, 1 atm, 589.3 nm; PDG explain_gas.html, 2023).

| Gas | Pack value (n-1) | Source read | Status |
|---|---|---|---|
| ammonia | 3.76e-4 | PDG "Ammonia (NH3)", index of refraction (STP, Na D) 376 ×10⁻⁶ (pdg.lbl.gov/2010/AtomicNuclearProperties/HTML_PAGES/108.html) | verified |
| neon | 6.7e-5 | PDG "Neon gas (Ne)", 67.1 ×10⁻⁶ (pdg.lbl.gov/2010/AtomicNuclearProperties/HTML_PAGES/010.html) | verified; the sheet holds 6.71e-5 |
| carbon monoxide | 3.38e-4 | no page with a value reached (PDG's CO page not found; Wikipedia's data page has none) | unverified |
| sulphur dioxide | 6.86e-4 | no page with a value reached; Cuthbertson and Cuthbertson's paper on gaseous NO, SO2 and SO3 refractivity found by title only | unverified |

Cross-check: PDG's carbon dioxide page gives 449 ×10⁻⁶, the same as the light pack's carbon dioxide row
(4.49e-4), so the pack and PDG use the same conditions.
