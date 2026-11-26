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

## Not done yet (later RW7 parts)
- **The real run on Alex's PC.** The output files and their sizes, and which of them ship in the app and which go to the store (RW8).
- Base-sol's full orbits at a date for its moons, from JPL's mean elements (the planets and the Moon are RW17's series).
- Belt masses by group, which need cited totals (the paper search is back on 1 Dec).
- Sizes for the irregular moons.
- J2 for the other planets.
