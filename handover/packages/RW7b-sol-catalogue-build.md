# RW7b: the Solar System catalogue build script

Thread "Add every Solar System body", 26 Nov 2026. Track RW, package RW7 (`design/SSE4-real-catalogues-design.md` 6.1, 7.2, 7.3 and 11), second part. Part one (RW7a, PR 663) added Saturn's small moons and J2 to base-sol.

## Built
- `scripts/build-sol-catalogues.py`, Python standard library only, in two halves:
  - `fetch CACHE` (needs JPL, so it runs on Alex's PC): every asteroid and comet from the SBDB Query API, paged 50,000 rows at a time and sorted by SPK id, plus JPL's satellite mean elements and physical parameters pages. It saves the raw answers and the retrieval date.
  - `build CACHE OUT` (no network, no clock): writes `sol-small-bodies` and one `sol-moons-<plane>` file for each reference plane JPL uses, in text form and packed with `sse catalogue pack`. It also writes `report.md`.
- Every value comes from the saved JPL answers. A measured diameter is graded measured. A missing diameter is estimated from H (a comet's nuclear M2 if it has no H) and an albedo: the body's own where measured, otherwise the median measured albedo of its JPL orbit class in the same download. Those rows are graded estimated (fudge RW7-size-from-h).
- A row with no size, or with no perihelion, perihelion time or epoch, is left out and counted, never filled in. A moon with no mean radius on JPL's page is left out and named in the report (fudge RW7-moon-sizes).
- Stops follow the design's 6.1 table: 100, 30, 10, 3 and 1 km. RW2 measures the Expected stop.
- Notables (format flag 1) are read from `scripts/sol-notables.json`, an authored list of bodies visited by spacecraft plus the famous comets. A body is flagged only when JPL's name for that designation matches the list, so a wrong designation shows up in the report instead of flagging the wrong body. Moon rows that are base-sol moons are flagged, because the app already shows them.
- `base_sol` meta column: a catalogue row that is a base-sol body (Ceres, Pluto, and the base-sol moons) names it, so the populations join (RW4) counts it once.
- Credits follow JPL's citation form word for word (RW1, `docs/citations/real-catalogues.md` P1), with the retrieval date filled in.

## Gates, each seen red with the fix removed
`scripts/test-build-sol-catalogues.py` builds from `tests/fixtures/catalogues/sol-build/`, which holds invented rows shaped like JPL's answers. It reads the result back through `sse catalogue show`. It runs in `check.sh` and as one added line in the Rust job of `ci.yml`.
- The size estimate is checked as an absolute value, not a ratio. With `H_TO_D_KM` changed, the test fails.
- The name check: fixture designation 433 carries a different name, and the test fails when the check is bypassed.

## First real run, on AC-ProArt, 26 Nov 2026
JPL accepted pages of 50,000. Asteroids came in 32 pages, 1,589,842 rows, the count JPL gave. Comets came in one page, 4,089 rows. The cache came to 516 MB. The packed small-body file is 333 MB (891 MB as text). There are 1,590,628 rows: 139,688 measured and 1,450,940 estimated. Rows by stop: 100 km 6,719; 30 km 10,094; 10 km 21,145; 3 km 211,519; 1 km 1,089,881. 3,303 rows were left out because they have no size and no magnitude. 46 moons have a JPL mean radius; the other 415 are left out (fudge RW7-moon-sizes).

The run found these, each fixed in the script and covered by the fixture test (seen red against the old script):
- Windows read the JSON files as cp1252, so every file is now opened as UTF-8.
- JPL sends spkid as a JSON number, so it is now turned into text.
- JPL's physical-parameters page leaves `<td>` cells unclosed and uses colspan headings. The old parser read the Moon's radius as a footnote number.
- Nereid's printed GM of 0 means no measurement, so it now reads as absent.
- Puck has two rows on the elements page. The first row is kept and the second is reported.
- JPL names 9P, 81P and 103P without their numbers, so the notable list now uses JPL's names.
- The dwarf planets came out about 3.6 times too big. SBDB has no diameter for them, and the TNO class had too few measured albedos, so they fell back to the all-asteroid median of 0.078. A class's own median is now used with as few as 5 measured albedos. Pluto takes base-sol's measured size. The rebuild showed this made it worse: Eris came out at 10,170 km, because the TNO class's 12 measured albedos are small, dark bodies (median 0.054). The cause is that the large distant bodies are bright and JPL has no sizes for them. So `scripts/sol-measured-sizes.json` now carries the measured sizes and albedos of Eris, Haumea, Makemake, Gonggong, Quaoar, Sedna and Orcus, each quoted from its paper and cited. They take part as measured, their albedos join their class's median, and the build checks each name against JPL's. Pluto also takes its geometric albedo from base-sol.

## Not done yet (later RW7 parts)
- More measured TNO sizes (the "TNOs are Cool" tables) so the class median is not set by a dozen small bodies.
- **The real run on Alex's PC.** The output files and their sizes, and which of them ship in the app and which go to the store (RW8).
- Base-sol's full orbits at a date for its moons, from JPL's mean elements (the planets and the Moon are RW17's series).
- Belt masses by group, which need cited totals (the paper search is back on 1 Dec).
- J2 for the other planets.

## RW7c (27 Nov): moon sizes from Horizons
For each moon that JPL's physical-parameters page gives no radius for, `fetch` now also saves JPL Horizons' object data (`CACHE/horizons/<NAIF>.json`, one request a second, skipping files already saved). `build` takes the radius Horizons prints and grades it estimated, source `jpl-horizons`, because Horizons does not say how the radius was found. A moon with a radius in neither place is left out and listed in the report. The moon files' diameter column now carries a grade and a source per row. The fixture gate was seen red with the Horizons answers removed.
