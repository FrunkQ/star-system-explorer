# MP2: first-wave cited data for the master data sheet

Track MP, design `handover/design/SSE4-matter-pack-design.md` sections 4, 10 and 11. Depends on MP1 (schema and
loader, kernel thread "Build kernel resolve and rules").

## State, 21 Nov 2026

- Done: the design's section 11 sources checked; register in `docs/citations/matter-reference.md`.
  11 of 12 DOIs confirmed. Rudnick and Gao 2014's DOI is still unseen (the chapter number fits it).
- Done: the four "from memory" gas refractivities in `packs/base-light`. Ammonia (3.76e-4) and neon (6.71e-5,
  the pack rounds to 6.7e-5) match Kaye and Laby through the PDG gas pages. Carbon monoxide and sulphur
  dioxide are still unverified: no page with a value was reached.
- Waiting: the rows themselves wait for MP1's schema. Values come from `handover/packages/MP0-audit.md`
  section A's canonical column, and from the pack tables listed in MP0 section C.

## Values that will change when readers move to the sheet

From MP0 (see `handover/packages/MP0.md`, "Disagreements that are not rounding"). The sheet holds the right-hand
value. Nothing changes until MP3 to MP6 move each reader.

| Constant | Today | Sheet | Why |
|---|---|---|---|
| solar mass | 1.98892e30 (sse-surface), 1.98847e30 (sse-cosmos, sse-formation), 1.989e30 | 1.9884099e30 | IAU 2015 GM_sun over CODATA 2018 G |
| Sun-to-Earth mass ratio | 332775.64 (sse-formation accrete) | 332946.0487 | IAU 2015 GM ratio |
| Sun-to-Jupiter mass ratio | 1048 (sse-cosmos epochs) | 1047.348644 | IAU 2015 GM ratio |
| Jupiter mass | 1.8979016e27 (base-atlas) | 1.89813e27 | IAU 2015 GM over G |
| Moon mass | 7.34556e22 (base-atlas) | 7.346e22 | NSSDCA, matches DE440 GM over G |
| dry air molar mass | 28.9595 g/mol (base-light) | worked out from the cited dry-air mixture | molar mass comes from the mixture |
| water triple-point pressure | 611 Pa (base-climate) | 611.657 Pa | NIST |
| standard gravity | 9.81 (14 places) | 9.80665 | exact by definition |
| gas constant | 8.314 (2 places) | 8.314462618 | exact |
| neon refractivity | 6.7e-5 | 6.71e-5 | Kaye and Laby via PDG |
