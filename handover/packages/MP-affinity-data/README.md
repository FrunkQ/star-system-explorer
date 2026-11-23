# Element affinity data: sources and how complete each table is

Collected 2026-11-23. The session's network policy blocked direct downloads from publishers, ADS, arXiv, earthref.org and university hosts (CONNECT 403). The only ways to read anything were (a) the WebFetch tool, which reads only the first ~75-85k characters of a PDF and won't reproduce long passages word for word, and (b) direct downloads from GitHub raw and PyPI. Because of this, **no table below was transcribed from the original publication's own page.** Each one comes from a machine-readable reproduction. Where a check against the original or a second copy was possible, it is listed. Treat every value as "reproduction, checked against the copies named", not "transcribed from the paper".

Scripts: `../affscripts/build.py` builds all three CSVs from the downloaded copies in `../dl/`.

---

## 1. lodders2003_tc50.csv (83 rows, H through U; no Tc or Pm, and no elements above Bi except Th and U)

- **Citation:** Lodders, K. 2003, "Solar System Abundances and Condensation Temperatures of the Elements", ApJ 591, 1220-1247, Table 8.
- **DOI:** 10.1086/375492
- **What was actually read:**
  - https://raw.githubusercontent.com/nhmc/Barak/master/barak/data/abundances/CondensationTemperatures.txt (Barak package). This is the only copy with the phase columns: initial condensate, its Tc, 50% Tc, and major phase or host. Its header wrongly says "Table 2"; the content matches Table 8.
  - https://raw.githubusercontent.com/adrn/engulfment/master/data/Lodders2003.yaml (50% Tc only, labelled "Lodders 2003 (Table 8)").
  - https://raw.githubusercontent.com/astro-seanwhy/ExoInt/HEAD/data/atomwttc_new.txt (ExoInt, Wang et al.; 50% Tc only, "Lodders 2003 (Table 8)").
  - Spot check: https://www.es.tohoku.ac.jp/JP/private/class/archives/McDonough_Lecture/lodders2003solar_cosmochem-table.pdf. This is a periodic-table chart of the Lodders (2003) Tc values used in McDonough's lecture, read with WebFetch. The values for H-Kr, Rb-Te, Cs, Ba, La and Hf-Bi agree with the CSV. (The tool's alignment shifted by one cell at Tc(43), but the sequence matched.) The lanthanide and actinide cells could not be aligned.
  - Tried and failed: iopscience PDF (blocked); https://www.ucolick.org/~woosley/ay220-19/papers/lodders03.pdf (readable, but the text extraction stops in section 2.3.5, before Table 8).
- **Status:** read PARTIALLY from the original. The full table comes from reproductions. tc50_k agrees exactly across all three independent copies for all 81 elements that have a value. Zero mismatches.
- **Columns:** `host_phase_as_written` is the "major phase or host" column as given in the Barak copy. LaTeX markup was converted, e.g. `CH_{4}\cdot 7H_{2}O` became `CH4·7H2O`. `note` holds the initial condensate and its Tc. A phase shown in braces `{...}` follows the source and means the element dissolves into the host phase rather than condensing as a pure phase.
- **Unsure:**
  - **H and He:** the Barak copy has no 50% Tc for them. It gives initial-condensation Tc values of 182 K (H2O ice) and 3 K (He ice). ExoInt puts 182 and 3 in its 50%-Tc column. tc50_k is left blank and the conflict is noted.
  - **Mg and Si:** each has two condensate rows (Mg: spinel 1397 K and forsterite 1354 K; Si: gehlenite 1529 K and forsterite 1354 K). Both rows are folded into the note.
  - **Lu:** the host is written "hibonite" in lower case, as in the copy.
  - **Elements not in the copies:** noble gases other than Ne, Ar, Kr and Xe, plus Tc, Pm, Po-Ac and Pa, are not in any copy. Lodders' Table 8 may not list them either; I could not confirm this.

## 2. earth_core_mantle.csv (77 rows)

- **Core column:** McDonough, W. F. 2003, "Compositional Model for the Earth's Core", Treatise on Geochemistry vol. 2 (ed. Carlson), pp. 547-568, Table 4. GERM indexes it as "McDonough 2004". DOI 10.1016/B0-08-043751-6/02015-6.
  - **Read at:** http://earthref.org/GERMRD/34/ (GERM Reservoir Database, Core reservoir). It was read twice with WebFetch, once sorted by reference, and only the "McDonough 2004 Table 4 (M04-4)" rows were kept. Both reads agreed. The publisher's page and the EarthRef table page for the reference (http://earthref.org/ERR/n:2764,b:aafi0002764tab04/) returned no table body.
  - **Status:** PARTIAL. The 39 elements that GERM's M04-4 lists are included. Every other element has a blank core value. GERM's older McDonough 1998 entry lists 0 for most lithophiles; those zeros were not copied.
  - **Unsure:**
    - **Co:** GERM shows "0.25 ppm". 0.25 wt% (2500 ppm) is almost certainly meant, since GERM's McDonough 1998 row gives 2500 ug/g. It is kept as printed and flagged.
    - **Ti:** "0.03 ppm" is flagged.
    - **O:** M04-4 has no O row. M04-5 gives O = 0 wt% for the preferred model.
- **Silicate Earth column:** McDonough, W. F. and Sun, S.-s. 1995, "The composition of the Earth", Chemical Geology 120, 223-253, Table 5 (pyrolite / primitive mantle). DOI 10.1016/0009-2541(94)00140-4.
  - **Read at:** the pyrolite Python package 0.3.9 from PyPI, file `pyrolite/data/geochem/refcomp/PY_McDonoughSun1995.csv`. It was cross-checked against https://raw.githubusercontent.com/abuseki/georefdatar/HEAD/data-raw/CI_Pyrolite__McDonough_Sun__1995__full.csv, a separate transcription: all 76 shared values match.
  - **Status:** FULL for the trace and minor elements Li-U. O and the noble gases are missing. Major elements are given as element wt% (Mg, Al, Si, Ca, Fe) and converted ×10^4 to ppm. Values in ppb were divided by 1000.
  - McDonough (2003) adopts a silicate-Earth model from McDonough & Sun (1995). Whether its printed silicate-Earth column differs in any value was NOT checked.
- **bulk_earth_ppm:** EMPTY. No readable copy of McDonough's bulk-Earth column was found. GERM's "Earth" reservoir has no McDonough 2003/2004 rows. The values were deliberately not computed from core and mantle mass fractions; that would be a derivation, not a transcription.

## 3. crust_rudnick_gao.csv (73 rows)

- **Citation:** Rudnick, R. L. and Gao, S. 2003, "Composition of the Continental Crust", Treatise on Geochemistry vol. 3, pp. 1-64, Table 11, bulk continental crust column. DOI 10.1016/B0-08-043751-6/03016-4.
- **What was actually read:**
  - The pyrolite 0.3.9 package (PyPI), file `pyrolite/data/geochem/refcomp/BCC_RudnickGao2003.csv` (main source).
  - https://raw.githubusercontent.com/abuseki/georefdatar/HEAD/data-raw/CC__Rudnick_Gao__2014.csv. This is the 2nd-edition table: Rudnick & Gao 2014, Treatise 2nd ed. 4.1, p.42 Table 11, DOI 10.1016/B978-0-08-095975-7.00301-6. It was used for comparison and to fill gaps.
  - Tried: https://www.geol.umd.edu/~rudnick/PDF/Rudnick_Gao_Treatise.pdf (Rudnick's own copy). The tool reads only the first ~18 pages and ends in the upper-crust section, so Table 11 was not reached.
- **Status:** PARTIAL from 2003. The trace elements come from the 2003 copy. Br (0.88) and I (0.71) are absent there and were taken from the 2014 table; their rows are marked. C, H, O, Rh, Te and the noble gases have no value in either copy.
- **Major elements:** converted from wt% oxide to ppm of the element, with the formula in `note`: ppm = wt% × 10^4 × n·A(el) / (n·A(el) + m·15.999). Fe comes from FeO (total Fe as FeO). The pyrolite 2003 copy gives the oxides to one decimal, for example TiO2 0.7, MgO 4.7 and P2O5 0.1. The 2014 copy gives two decimals: 0.72, 4.66 and 0.13. The 2014 ppm equivalent is given in `note` for each, flagged UNSURE. The two-decimal figures are probably what both editions print, but I could not check the 2003 page.
- **Unsure:**
  - **N:** the pyrolite file lists "Na 56 ppm", which can only be N, since Na is a major element. The 2014 table gives N 56. It is stored as N and flagged.
  - **Ru:** 0.6 ppb (2003 copy) versus 0.57 ng/g (2014), noted.

## 4. ci_lodders.csv — NOT WRITTEN

No readable copy of Lodders (2021, Space Sci. Rev. 217, 44, DOI 10.1007/s11214-021-00825-8) CI mass fractions was found. The only Lodders 2021 file found was Table 3, which gives log atomic abundances, not ppm by mass (https://raw.githubusercontent.com/CambridgeUniversityPress/Stellar-Structure-and-Evolution/HEAD/Chapter1/Lodders2020.txt). The Lodders (2003) Table 1 CI column was not reachable in a form I could read whole.

Two CI tables by other authors were seen and could stand in:
- McDonough & Sun (1995) CI: the georefdatar file above.
- Palme & O'Neill (2014) CI: pyrolite `CH_PalmeONeill2014.csv`.

Neither was written as ci_lodders.csv, so that they are not passed off as Lodders.

## 5. ci_mcdonough_sun1995.csv (76 rows, written by the data sheet thread)

- **Citation:** McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223, CI chondrite column. DOI 10.1016/0009-2541(94)00140-4.
- **Read at:** the pyrolite 0.3.9 package (`CH_McDonoughSun1995.csv`) and georefdatar (`CI_Pyrolite__McDonough_Sun__1995__full.csv`), two separate transcriptions. 74 values agree. N and P differ by a factor of 1000. The per-cent-level values (3180 and 1080 ppm) are kept, since CI chondrites hold about 0.3% N and 0.1% P.
- Used in place of Lodders 2021, whose CI mass fractions were not found in a readable copy.
