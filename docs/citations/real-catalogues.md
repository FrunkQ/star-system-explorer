# Real catalogues (track RW): terms, counts, versions and credit wording

RW1. Every catalogue named in `handover/design/SSE4-real-catalogues-design.md` (sections 5.3, 5.3A, 6.1, 6A.2 and
6A.3) was read at its source on **26 Nov 2026**, partly through a cloud page reader that quotes in short pieces, and
partly through the built-in browser on Alex's PC (AC-ProArt), which read whole pages and the live APIs. Status words
are as in [the register](../citations.md). "Verbatim" means the wording below was copied from the page and is the
text a pack's terms part (design section 3.1) carries. Where a credit could only be read in pieces, or not at all,
the row says so: the reader refuses a catalogue with no credit text, so those gaps block shipping that catalogue.

Alex's rules that these rows serve (design 5.3, 12A): credits ship in packs as plain readable text and are shown in
full at the first import; a pack holding downloaded catalogue data may be used and given away but never sold, and the
store refuses a price; anything a survey has reported with an orbit is catalogued, not hypothetical.

## 1. What this changes in the design

1. **Most star data is non-commercial, so the never-sold rule is the right rule, and it covers the app too.**
   ESA's archive terms put Gaia **and Hipparcos** under CC BY-NC 3.0 IGO, and they define commercial use as "any use
   or application that directly or indirectly generates a financial gain". The design's "shipped in the app ... from
   sources with open terms: the Hipparcos and Yale Bright Star catalogues" is wrong on both counts: Hipparcos from
   ESA's archive is non-commercial, and the Yale Bright Star Catalogue's 5th edition has no clear licence at all
   (row S5). The app is free and Apache-2.0, so shipping non-commercial data in it is fine, but the in-app stars must
   carry the ESA credit, and nothing paid (a priced pack, a paid tier) may include them. Question for the coordinator,
   and for Alex if the coordinator thinks it changes a goal: does the store's 20% take on *other* packs make the app a
   "financial gain" in ESA's sense? My reading is no (the catalogue data is never in a priced item), but it is
   ESA's word to give, through data.licences@esa.int, not ours.
2. **The Yale Bright Star Catalogue should not ship.** Its holder says the copyright may still be Yale's and it is
   provided "solely for the purposes of scientific research". The bright and named stars in the app should come from
   Hipparcos (ESA credit) and the IAU names (CC BY), with the BSC's HR numbers only as identifiers.
3. **The Minor Planet Center's data files may be redistributed with the source named**; its circulars and web pages
   may not go into any product. So RW7 builds from the data files (MPCORB, the comet elements) or from JPL's
   small-body database, never from MPECs. The older, stricter MPC catalogue policy is withdrawn (404 on the live site).
4. **JPL's small-body database is the cleanest source** for RW7: NASA science data is CC0, and JPL asks only for a
   citation. Its live API gives the counts in section 4.
5. **ShareAlike data must stay in its own file.** AT-HYG and OpenNGC are CC BY-SA 4.0. A pack that holds their rows
   keeps those rows in a separately licensed data file with the credit and link (the naming track's Wiktionary rule).
   AT-HYG also contains Gaia DR3 values, so it is non-commercial as well.
6. **The Gaia archive was offline on 26 Nov "due to preparations for DR4"**, so DR4 is close. Catalogue versions are
   pinned (handoff ruling): RW13 pins DR3 and moving to DR4 is a deliberate step with a "what changed" list.
7. **Two moon theories have no readable terms.** IMCCE, which publishes the Galilean (L1) and Saturn (TASS) theories,
   merged into the Laboratoire Temps Espace on 1 Jan 2025 and its terms page is gone. RW17 asks LTE before bundling
   them, and uses mean orbits with their turning rates meanwhile (the design's first step).

## 2. Stars and galaxies

| Id | Catalogue, version | Count read | Licence and commercial use | Credit to ship (verbatim unless marked) | Status, and how read |
|---|---|---|---|---|---|
| S1 | Gaia DR3 (ESA). DR4 in preparation | not read (archive offline) | CC BY-NC 3.0 IGO: "Gaia data are distributed under the CC BY-NC 3.0 IGO license." (cosmos.esa.int/web/gaia-users/license). ESA terms clause 2: "Prior to any commercial use by the User of any Data or Data Product, including any use or application that directly or indirectly generates a financial gain, a detailed request for authorisation/licence shall be made by the User by sending email to data.licences@esa.int." Clause 3: "The User shall not use Data or Data Products to state or imply the endorsement by ESA or by any ESA employee of a commercial product, service, or activity, or used in any misleading manner." | Credit line, from ESA's table (verbatim): "Credit: ESA, Gaia DPAC". DR3 page: "The Gaia data are open and free to use, provided credit is given to 'ESA/Gaia/DPAC'." The DR3 acknowledgement sentence was read only in pieces: "This work has made use of data from the European Space Agency (ESA) mission Gaia" ... "processed by the Gaia Data Processing and Analysis Consortium (DPAC" ... "Funding for the DPAC has been provided by national institutions" ... "the institutions participating in the Gaia Multilateral Agreement" (links https://www.cosmos.esa.int/gaia and https://www.cosmos.esa.int/web/gaia/dpac/consortium). Cite Gaia Collaboration 2016 (mission) and 2023 (DR3 summary). | verified (licence, terms and credit line, browser, ESA terms page); the long acknowledgement is **partial**: re-read it whole when the archive is back |
| S2 | Gaia Catalogue of Nearby Stars (GCNS), from EDR3 (Smart et al. 2021, A&A 649, A6, doi 10.1051/0004-6361/202039498) | **331,312 objects** within 100 pc, "at least 92% of stars within 100 pc at spectral type M9" (GAVO table page) | "Gaia Catalogue of Nearby Stars © 2023 by Richard Smart is licensed under CC BY-NC 4.0." (GAVO). Gaia data inside, so S1's terms too | Smart et al. 2021 citation, plus S1's credit line | verified (count and licence, cloud reader) |
| S3 | Fifth Catalogue of Nearby Stars (CNS5), Golovin et al. 2023, A&A 670, A19; VizieR J/A+A/670/A19, doi 10.26093/cds/vizier.36700019 | **5,931 objects** within 25 pc: 5,230 stars (4,946 main sequence, 20 red giants, 264 white dwarfs) and 701 brown dwarfs (abstract) | No data licence found (the ReadMe could not be read: CDS refused both the cloud and the browser on 26 Nov). Built on Gaia EDR3, so treated as S1's terms | Golovin et al. 2023 citation; the VizieR line (V1); S1's credit line | verified (count); licence **unverified** (re-read the ReadMe when CDS answers) |
| S4 | Hipparcos, the New Reduction (van Leeuwen 2007, A&A 474, 653, doi 10.1051/0004-6361:20078357; VizieR I/311) | **117,955 stars** (VizieR ReadMe) | ESA archive data: CC BY-NC 3.0 IGO and the clauses in S1 | "Credit: ESA" (ESA's credit table, verbatim); the ReadMe's acknowledgement names Floor van Leeuwen, Institute of Astronomy, Cambridge | verified (browser for the terms; cloud reader for the ReadMe) |
| S5 | Yale Bright Star Catalogue, 5th revised edition, preliminary (Hoffleit and Warren 1991; VizieR V/50) | brighter than V 6.5; the record count was not read (CDS unreachable) | **Unclear, do not ship.** HEASARC, verbatim: "The HEASARC provides this catalog as a resource to the astronomical community solely for the purposes of scientific research. The HEASARC does not own the copyright and, therefore, does not control the licensing for commercial usage, nor can we clarify its terms of use beyond its usage in scientific research or provide legal advice on intellectual property rights." and "It is possible that Yale University Observatory and/or the authors listed in the References section still retain the copyright and intellectual property rights on this 5th edition." | (none: not to be shipped) | verified (terms, browser); count **unverified** |
| S6 | IAU Catalog of Star Names (IAU-CSN), WGSN | the file has **451 names** (header "Last updated 2022-04-04"); the IAU added **59** more on 12 Feb 2026 (40 from 2025, 19 from 2024). No single current total was found; about 510 is a sum, not a reading | CC BY. File header, verbatim: "All IAU-produced products (Images, Videos, Texts) are released under Creative Commons Attribution (i.e. free to use in all perpetuity, world-wide, as long as the source is mentioned)." The WGSN also says it "does not sanction the selling or buying of star names" | File header, verbatim: "For use of the contents of the data of this file, we encourage users to cite the official IAU version of the data at: https://www.iau.org/public/themes/naming_stars/". That IAU address returned 404 on 26 Nov (the IAU site has moved); credit "IAU Working Group on Star Names (WGSN)" with the file's address until the new one is found | verified (browser, the file itself). The naming area's row dates the file 1 Jan 2021 (the WGSN page's date); the file's own header says 2022-04-04 |
| S7 | Washington Double Star Catalog (WDS), USNO | **149,104 systems** from 1,899,327 means, "as of Wed Oct 28 09:53:03 AM EDT 2026" (crf.usno.navy.mil/wds) | US federal work; no licence named. The FAQ says the catalogue is updated about monthly | Verbatim: "This research has made use of the Washington Double Star Catalog maintained at the U.S. Naval Observatory." | verified (cloud reader) |
| S8 | NASA Exoplanet Archive, Planetary Systems table (doi 10.26133/NEA12); paper Christiansen et al. 2025, PSJ, doi 10.3847/PSJ/ade3c2 | **6,473 planets in 4,865 systems** (TAP count of `pscomppars`, browser, 26 Nov) | No licence stated; NASA science data (C1) | Verbatim: "This research has made use of the NASA Exoplanet Archive, which is operated by the California Institute of Technology, under contract with the National Aeronautics and Space Administration under the Exoplanet Exploration Program." | verified (browser for the count; cloud reader for the wording, in two adjacent pieces) |
| S9 | SIMBAD (CDS), the live route | live | No licence stated | Verbatim: "This research has made use of the SIMBAD database, operated at CDS, Strasbourg, France." Cite Wenger et al. 2000, A&AS 143, 9 | verified (cloud reader) |
| V1 | VizieR (CDS), for any catalogue fetched through it | n/a | VizieR's "rules of usage" page could not be read (CDS 404 on 26 Nov) | Verbatim (line breaks joined): "This research has made use of the VizieR catalogue access tool, CDS, Strasbourg, France (DOI : 10.26093/cds/vizier). The original description of the VizieR service was published in 2000, A&AS 143, 23" | credit verified; rules **unverified** |
| S10 | AT-HYG v4.0 (astronexus, Codeberg) | "over 2.5 million stars"; "Nearly 99% of stars in AT-HYG have distance data from Gaia DR3" | Author's licence CC BY-SA 4.0; the Gaia values inside also carry S1's non-commercial terms | "This work is licensed under a Creative Commons Attribution-ShareAlike 4.0 International License." plus S1's credit line | verified (browser) |
| G1 | Updated Nearby Galaxy Catalog (Karachentsev, Makarov and Kaisina 2013, AJ 145, 101, doi 10.1088/0004-6256/145/4/101) | **869 galaxies** within 11 Mpc or V_LG below 600 km/s (abstract) | HEASARC's copy is listed on data.gov as a US government work (https://www.usa.gov/government-works); the authors' own terms are not stated | Cite the paper; "This is a service provided by NASA HEASARC" for the HEASARC copy | verified (cloud reader) |
| G2 | OpenNGC (Mattia Verga) | count not read (the data folder is closed to readers) | "OpenNGC is released under CC-BY-SA-4.0 license" | Verbatim: "Credits: Mattia Verga <https://github.com/mattiaverga/OpenNGC>". Its README also lists NED, HyperLEDA, SIMBAD and HEASARC acknowledgements | licence verified; count **unverified** |

## 3. The Solar System

| Id | Source, version | Count read | Licence and commercial use | Credit to ship (verbatim unless marked) | Status, and how read |
|---|---|---|---|---|---|
| C1 | NASA science data licence (science.data.nasa.gov/about/license) | n/a | NASA-led mission data with no restrictive notice "are licensed as Creative Commons Zero"; "There are no restrictions on the usage of these data." | Citing the source is encouraged, not required | verified (cloud reader) |
| P1 | JPL Small-Body Database (SBDB), Query API version 1.0 | see section 4 | NASA/JPL data; only a citation is asked | Verbatim form from ssd.jpl.nasa.gov/about: "NASA/JPL Solar System Dynamics. Data retrieved on (Downloaded Year, Month, Date). (Title of the Page). https://ssd.jpl.nasa.gov" (filled in by the build script with the date and page) | verified (browser, live API) |
| P2 | Minor Planet Center data files (MPCORB, comet elements) | 895,910 numbered minor planets; 1,604,068 orbits in its database (MPC annex status, 26 Nov 18:00 GMT) | Live Website Use Policy, verbatim: "Data files are copyrighted entities and may be redistributed as long as the source for the data is clearly specified." But: "Inclusion of circulars or MPC web pages in products of any description (including, but not limited to CDs and magazines), whether or not a charge is made for the product, is strictly prohibited unless express written permission has been obtained in advance." The older Catalogue Distribution Policy ("personal (scientific) use only") is 404 on the live site and survives only on an old mirror, so it is withdrawn | Verbatim, under "To cite the Minor Planet Center, please use the following reference:": "Data from the Minor Planet Center (MPC) is made freely available to the public. NASA's Planetary Defense Coordination Office (PDCO) supports the MPC's data services and operations through grant 80NSSC22M0024, administered through a University of Maryland-Smithsonian Astrophysical Observatory (SAO) subaward (106075-Z6415201). The NASA award and the Tamkin Foundation both contribute funding for the MPC's computing equipment." The SBN replica terms add that "any modifications or derived data products must be explicitly and clearly marked as such if they are redistributed." | verified (browser, live policy page) |
| P3 | JPL planetary satellites (ssd.jpl.nasa.gov/sats) | **460** IAU-recognised moons (Pluto's included, Earth's not): Mars 2, Jupiter 115, Saturn 293, Uranus 29, Neptune 16, Pluto 5. The page says "These data were last updated 2023-May-23." though it cites 2026 designations; Johnston's archive lists 468 (Jul 2026, secondary) | NASA/JPL data (C1, P1) | P1's citation form | verified (browser) |
| P4 | NSSDCA Planetary Fact Sheets (Dr. David R. Williams, curator) | planets page last updated 8 Sep 2026 | US government work; no licence stated. The notes say, verbatim: "Note that the values listed on the factsheets are not "official" values" | "NASA Space Science Data Coordinated Archive, Planetary Fact Sheets, D. R. Williams" (my wording: the site gives no credit line) | verified (cloud reader) |
| P5 | GCAT, General Catalog of Artificial Space Objects, Release 1.8.9 (2026 Nov 6), data update 2026 Nov 26 | not read | "GCAT is released under the Creative Commons CC-BY licence. You are free to use the data in GCAT as long as you include a citation to it." (c) 2020-2026 Jonathan McDowell | Verbatim short line: "data from GCAT (J. McDowell, planet4589.org/space/gcat)"; full: "McDowell, Jonathan C., 2020. General Catalog of Artificial Space Objects, Release 1.8.9 , https://planet4589.org/space/gcat" | verified (cloud reader); matches what `sol-today` carries |
| M1 | VSOP87 (Bretagnon and Francou 1988, A&A 202, 309; VizieR VI/81, files "BDL-9502") | six versions; the planets, the Earth-Moon barycentre, the Sun (version E) | No terms in the file | Cite the paper | verified (cloud reader). Accuracy, verbatim in part: about 1 arcsecond for Mercury, Venus, the Earth-Moon barycentre and Mars over 4000 years either side of J2000, and "The same precision is ensured for Jupiter and Saturn over 2000 years and for Uranus and Neptune over 6000 years before and after J2000." Fitted to DE200 |
| M2 | ELP/MPP02 (Chapront and Francou 2003, A&A 404, 735, doi 10.1051/0004-6361:20030529) | n/a | The data files' terms were not read (they sit on an Observatoire de Paris FTP host). A Python port on PyPI shows two licences (EUPL 1.2 and GPL-3.0); do not use that code | Cite the paper | accuracy verified from the abstract: over [-3000, +3000] fitted to DE406, "less than 3 arcsec in longitude and latitude and 2 km in distance"; over two millennia the residuals are below 0.03 arcsec and 30 m. Terms **unverified** |
| M3 | IMCCE L1.1, Galilean moons (Lainey, Duriez and Vienne 2007, A&A 456, 783) | n/a | The README says "THE USE OF THESE DATA IS SUBJECTED TO CONDITIONS" and points to an IMCCE page that is now 404: IMCCE merged with SYRTE into the Laboratoire Temps Espace on 1 Jan 2025 | Cite the paper | accuracy verified, verbatim: "few hundreds of kilometers for the period [1970:2010]". Terms **unverified**: ask LTE |
| M4 | IMCCE TASS 1.7, Saturn's first eight moons (Vienne and Duriez) | n/a | as M3 | Cite the theory | **unverified** (README unreadable) |

## 4. Small-body counts, read live on 26 Nov 2026 about 18:30 UTC

From the JPL SBDB Query API (`info=count` and class or group filters), through the browser on Alex's PC.

| What | Count |
|---|---|
| Asteroids, numbered / unnumbered / all | 895,910 / 693,932 / **1,589,842** |
| Comets, numbered / unnumbered / all | 610 / 3,479 / **4,089** |
| Main belt (MBA class) | 1,404,751 |
| Outer main belt (OMB class) | 51,473 |
| Jupiter trojans | **16,684** |
| Near-Earth objects / of them potentially hazardous | **43,392** / 2,554 |
| Centaurs | 1,114 |
| Trans-Neptunian objects | **8,423** |
| With a measured diameter | 139,688 (8.8% of asteroids: the design's "about one in ten" holds) |
| Measured diameter above 100 / 30 / 10 / 1 km | 240 / 1,472 / 10,183 / 138,765 |
| Brighter than H 8.1 / 10.7 / 13.1 / 15.7 / 18.1 | 5,932 / 9,574 / 17,317 / 160,090 / 991,017 |

The H cuts are where a body of albedo 0.10 is 100, 30, 10, 3 and 1 km across (D = 1329 km / sqrt(albedo) x
10^(-H/5)); the albedo is my assumption, so these are estimated sizes, graded as such, and RW2 replaces them with its
own per-group albedos. Johnston's archive (secondary, July 2026) gives 1,557,477 minor planets, 41,956 near-Earth
asteroids and 6,326 TNOs and centaurs, consistent with the above four months earlier.

## 5. Newly found large distant bodies (design 6A.3)

37 small bodies are brighter than H 4 (about 600 km and up at moderate albedo): Vesta, Ceres and 35 beyond Neptune
(JPL SBDB, 26 Nov). All have orbits and MPC designations, so all are catalogued, graded "estimated" for size.

| Body | H (JPL) | Orbit (JPL) | Found | Size |
|---|---|---|---|---|
| 2026 ED138 | 2.85 (the paper gives 3.19 ± 0.04 in V) | a 42.95 au, e 0.142, i 24.5°; Neptune's 12:7 resonance in 46% of clones (arXiv 2610.38693) | Rubin, found 13 Oct 2026 in data from 4 Mar to 11 Jul 2026; announced by the MPC on 27 Oct (MPEC 2026-U85); precovered to 1937 | NOIRLab, 27 Oct: "the team estimates that its size is between 600 and 1500 kilometers"; the paper gives 612 to 1530 km for albedos 0.25 to 0.04. Unmeasured |
| 2026 EE138 | 3.95 | a 71.13 au, e 0.512, i 11.2° | Rubin, 2026 | about 500 to 1,000 km (Sky and Telescope, secondary) |
| 2017 OF201 | 3.49 | a 823.6 au, e 0.945, q 44.9 au | archival DECam images, announced May 2025 (MPEC 2025-K47) | about 700 km (560 to 850), assumed albedo |
| 2021 DR15, 2018 VG18, 2015 RR245, 2014 UZ224 | 3.61, 3.94, 3.81, 3.48 | in the SBDB | found 2014 to 2021 | unmeasured |

Rubin's totals: over 11,000 new asteroids including about 380 TNOs submitted in April 2026 (NOIRLab); the ED138
paper reports a Rubin search that "found 2,440 previously unknown TNOs", and estimates "about 3 more objects like
them, and probably more, remain undiscovered away from the Galactic plane". The LSST itself began in June 2026
(NOIRLab). The IAU still recognises five dwarf planets (Pluto, Ceres, Eris, Haumea, Makemake), per NOIRLab.

So the small-body packs must be rebuilt on a schedule (design 6A.3): the TNO count above will move by thousands a
year while the LSST runs.

## 6. Still open

- The full Gaia DR3 acknowledgement in one piece (the archive was offline); re-read when DR4 is out.
- CDS: the CNS5 and BSC ReadMes and VizieR's rules of usage (CDS refused the browser and returned 404 on 26 Nov).
- The current total of IAU star names, and the IAU's new address for the names page.
- The terms of ELP/MPP02's data files, IMCCE L1 and TASS (ask the Observatoire de Paris and LTE).
- The 10 pc sample's size (design 5.3's "about 340 systems") was not read at source: CNS5's ReadMe has it, and CDS was
  unreachable. It stays marked unverified.
- ESA's reading of "financial gain" for a free app whose store takes a share of other, priced packs (section 1.1).
