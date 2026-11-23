# RS1: Sol today catalogue (track RS)

Owner: "Build real spaceflight history pack". Design: `handover/design/SSE4-real-spaceflight-feasibility.md` sections 1 to 3.

## Part 1 (23 Nov 2026)

- `scripts/spaceflight/import_gcat.py` reads six GCAT TSV files and writes `store-packs/sol-today/catalogue.json`. The output is sorted and stamped only with GCAT's own update dates, so one GCAT release always gives the same bytes.
- Counts from the GCAT data update of 23 Nov 2026 replace the design's estimates:
  - 7,223 orbital launch attempts (estimated about 7,000);
  - 472 payloads that left Earth orbit, failures included (estimated about 300 craft, before counting failures and every lunar attempt);
  - 469 landings and impacts on other worlds, 257 of them on the Moon;
  - 770 records of human spaceflight vehicles;
  - 16 station cores, in the authored `stations.json` (estimated about 15).
- GCAT keeps one row per orbit phase for each deep-space craft; Cassini has 324 of them. Each craft's count is kept as `phases`, as a pointer for RS8. Those rows are a ready-made source for big-burn flight logs.
- Licence: CC BY 4.0, read on GCAT's home page on 23 Nov 2026. The credit is in `pack.json` and `NOTICE`. The pack is `CC-BY-4.0 AND Apache-2.0`.
- Network: the cloud environment now allows planet4589.org, ssd.jpl.nasa.gov, nssdc.gsfc.nasa.gov and query.wikidata.org (Alex approved, 23 Nov).
- `store-packs/` is outside `packs/`, so nothing loads it. The pack is published on Explorers, not built in.

## Part 2 (23 Nov 2026): Wikidata cross-check

- `scripts/spaceflight/crosscheck_wikidata.py` matches craft by COSPAR id (Wikidata P247), adds each craft's Wikidata id (`wikidata`, CC0) to the catalogue, and writes `handover/packages/RS1-wikidata.md`.
- 336 of 472 craft found. One launch date differs: Pioneer 10, where Wikidata is wrong. 67 masses differ by more than 10 percent, mostly because the two sources quote different masses (wet against dry, with or without a lander, or GCAT's mass for one orbit phase). GCAT stays the source; RS2's parts settle the masses for the showcase craft from primary sources.
- NSSDCA is left for RS2, read per craft for its instruments rather than in bulk.

Next: RS2 parts from GCAT's engines.tsv, stages.tsv and lvs.tsv.
