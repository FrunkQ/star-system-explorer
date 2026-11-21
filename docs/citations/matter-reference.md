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
| fray2009 | Fray, N. and Schmitt, B. 2009, Planet. Space Sci. 57, 2053-2080: Sublimation of ices of astrophysical interest, a bibliographic review. doi:10.1016/j.pss.2009.09.011 | vapour-pressure fits and triple points of ices | verified (metadata) | SSHADE record PUBLI_Fray_2009 |
| hp2011 | Holland, T. J. B. and Powell, R. 2011, J. Metamorph. Geol. 29, 333-383: An improved and extended internally consistent thermodynamic dataset for phases of petrological interest. doi:10.1111/j.1525-1314.2010.00923.x | mineral formation energies, densities, heat capacities | verified (metadata) | running footer of the article PDF (pages.uoregon.edu mirror) |
| hitran2020 | Gordon, I. E. et al. 2022, JQSRT 277, 107949: The HITRAN2020 molecular spectroscopic database. doi:10.1016/j.jqsrt.2021.107949 | gas absorption bands | verified (metadata) | ODU Digital Commons record chemistry_fac_pubs/219 |
| adams2008 | Adams, F. C. 2008, JCAP 08, 010: Stars in other universes, stellar structure with different fundamental constants. doi:10.1088/1475-7516/2008/08/010 | which blocks follow a changed constant (design section 7A) | verified (metadata) | arXiv 0807.3697 abstract page, which gives the DOI |
| nist-webbook | NIST Chemistry WebBook, SRD 69. https://webbook.nist.gov/chemistry/ | gas phase points, formation enthalpies | textbook | standard reference database; per-substance pages checked when each row is filled |

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
