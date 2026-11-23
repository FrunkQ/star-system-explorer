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

Next: cross-check masses and dates against Wikidata (CC0) and NSSDCA; then RS2 parts from GCAT's engines.tsv, stages.tsv and lvs.tsv.
