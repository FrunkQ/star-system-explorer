# MP0: audit of every physical number typed into code

Track MP (master data sheet, `handover/design/SSE4-matter-pack-design.md`). MP0 lists every constant of
nature, convention, reference object, substance, element and isotope value that is typed into crates,
packs and the renderer, with file, value, unit and source, and every place two of them disagree. It
changes no behaviour and no golden.

## What was built

- `scripts/audit-matter.py`: a textual scan of `crates/`, `packs/` and `packages/` (`.rs .ts .json .svelte .mjs`,
  skipping build output, goldens and fixtures). Python 3 standard library only.
  - A catalogue of 57 quantities (20 constants of nature, 8 units and conventions, 21 reference-object values
    and ratios, 15 substance values, 5 isotope half-lives), each with its canonical value, unit, source and a
    relative tolerance. Every numeric literal with at least 3 significant digits that falls within tolerance of
    a catalogue value (in any of its usual units, such as kg/mol and g/mol) counts as a typed copy.
  - A key scan of pack JSON for matter properties (density, molar mass, phase points, latent heat, optics,
    half-life, thermochemistry).
  - A name scan of Rust `const` items that hold physical or matter values (`G`, `AU`, `RHO_*`, `M_SUN*`,
    `R_EARTH*`, `SIGMA_*_PA` and the like).
  - String literals and comment lines are skipped, so a paper's volume number never counts. Hits in test code
    are flagged apart.
- `handover/packages/MP0-audit.md`: the generated table (sections A to D below). Regenerate with
  `python3 scripts/audit-matter.py --markdown handover/packages/MP0-audit.md`.
- `--json OUT` writes every hit for tools. `--retired FILE` takes a JSON list of quantity ids and exits 1 if any
  of them is still typed outside `crates/sse-reference` and `packs/base-reference`: this is the CI check for
  MP3 to MP6 (each package adds the ids it retired to the list, then CI shows they are gone).

## Totals (21 Nov)

| Count | Value |
|---|---|
| typed copies of catalogue constants | 658 (453 outside tests) |
| pack matter properties | 420 in 10 pack files |
| Rust matter or physical consts | 206 |
| quantities typed with more than one value | 22 |

## Coverage of the design's section 2

| Section 2 row | Where the audit finds it |
|---|---|
| Climate substances (5) | A: T3, P3, molar masses, latent heats; C: `packs/base-climate/pack.json` substances |
| Light gases (13) | A: molar masses; C: `packs/base-light/pack.json` gases (refractivity, King factor) |
| Life species (26) | C: `packs/base-life/pack.json` `life_chemistry` (free energies and enthalpies); read by `crates/sse-life/src/chemistry.rs` |
| Formation's six materials | D: `crates/sse-formation/src/disc.rs:46-83` condensation temperatures and densities |
| Galaxy rings' six nucleosynthesis groups | D: `crates/sse-cosmos/src/patch.rs` budgets (group shares, no catalogue constants) |
| Heat isotopes (5) | A: half-lives; D: `crates/sse-atoms/src/heat.rs` (shares, heat per kg; Ruedas 2017) |
| Surface rock, ice, strengths, 273.15, ice conductivity, colours, cloud decks | D: `crates/sse-surface/src/lib.rs:478-505` (`RHO_ROCK` 3500, `RHO_ICE` 930, `SIGMA_ICE_PA`), A: `T0C`; cloud decks in `regimes.rs:686` |
| Weather magma, Earth air, giant air | C: `packs/base-weather/pack.json:495-503`; A: `M_air` |
| Traveller gas mixes (23) | `packs/traveller/pack.json:7925` `gas_mixes` (names and ranges only, no numbers to audit; MP5 maps the names to substance ids) |
| Renderer gas-id map | `packages/renderer/src/atmosphere.ts:21` `KERNEL_GASES` (12 gas names to kernel ids; MP6 reads them from the sheet). The renderer also types `m_u` at `atmosphere.ts:73` |
| `sse-core` `quantity::consts` (13) | A: every row marks `crates/sse-core/src/quantity.rs` as a typed place |

## Disagreements that are not rounding

Section B of the audit lists all 22. Most are the canonical value at a shorter precision ("rounded") and only
need the sheet. These are the ones where a different number is written, and what the sheet should hold:

| Quantity | Written | Where | Sheet value and reason |
|---|---|---|---|
| Solar mass | 1.98892e30 | `crates/sse-surface/src/lib.rs:45` | 1.9884099e30, IAU 2015 B3 GM_sun over CODATA 2018 G. 1.98892e30 is the pre-2010 textbook value |
| Solar mass | 1.98847e30 | `crates/sse-cosmos/src/nucleus.rs:43`, `sse-formation` disc, growth and others (16) | as above. 1.98847e30 uses CODATA 2014 G (6.67408e-11); 30 ppm off |
| Solar mass | 1.989e30 | `crates/sse-formation/src/accrete.rs:55` | as above (rounded, but paired with the wrong mass ratio below) |
| Sun-to-Earth mass ratio | 332775.64 | `crates/sse-formation/src/accrete.rs:54` | 332946.0487 (IAU 2015 GM ratio). 332775.64 is 1.989e30 / 5.977e24, 0.05% off |
| Sun-to-Jupiter mass ratio | 1048 | `crates/sse-cosmos/src/epochs.rs:75` | 1047.3486 (IAU) |
| Earth mass | 5.972e24, 5.97e24 | about 30 places | 5.9722e24 (GM over G); the shorter forms are rounding, kept here because there are so many |
| Earth equatorial radius | 6378137 | `packs/base-light/pack.json:870`, a `sse-light` test | 6378100 (IAU 2015 nominal). 6378137 is the WGS84 ellipsoid, a different definition; the sheet holds both, named apart |
| Earth radius | 6.371e6 and 6.3781e6 both called `R_EARTH` | `sse-surface`, `sse-atoms` (mean) against `sse-core` (equatorial) | known from the design: the sheet names mean and equatorial radii apart |
| Earth mass parameter | 3.986004418e14 | `packages/shell/src/documents/sample-sol.v3.json:150` | a V3 sample file holding the IERS value, which agrees to 7 digits; leave it (V3 data) |
| Jupiter mass | 1.8979016e27 | `packs/base-atlas/pack.json` (6 examples) | 1.89813e27 (IAU GM over G); 1.89819e27 (NSSDCA) is in a test. The atlas value is 0.02% low, source unknown |
| Moon mass | 7.34556e22 | `packs/base-atlas/pack.json` (6 examples) | 7.346e22 (NSSDCA), matching 7.3458e22 from the DE440 GM. The atlas value is 0.006% low |
| Moon semi-major axis | 384466527.7 m | `packs/base-atlas/pack.json` (6 examples) | 384400 km (NSSDCA mean distance); the atlas value is a computed osculating figure |
| Water molar mass | 18.01528 g/mol | `packs/base-light/pack.json:574` | 18.0153 (IUPAC 2021 weights; the extra digits overstate precision) |
| Dry air molar mass | 28.9595 g/mol | `packs/base-light/pack.json:478`, `sse-atoms` tests | 28.9644 (US Standard Atmosphere 1976); 28.9595 is an older dry-air composition. The weather pack's 28.97 and the renderer test's 28.96 are rounding. MP2 picks one cited composition and works the molar mass out from it |
| Water triple-point pressure | 611 and 612 Pa | `packs/base-climate/pack.json:165`, `packages/shell/src/documents/kestrel.scene.json:116` | 611.657 Pa (NIST); the kestrel scene value is a scene input, not a constant |
| Standard gravity | 9.81 | `sse-life` (3), `sse-nav`, `sse-surface`, `packs/base-surface` (14 in all) | 9.80665 (exact by definition) where it means standard gravity; where it means "Earth's surface gravity" the sheet derives it from Earth's GM and radius |
| Gas constant | 8.314 | `crates/sse-place/src/model.rs:313`, `crates/sse-surface/src/regimes.rs:743` | 8.314462618 (exact) |
| Solar temperature | 5770 K | `packages/ui-contract/test/light.test.ts:104` | 5772 K (IAU nominal); a test tolerance only |

## Things that look wrong but are not

- `packs/base-light/pack.json:610` carbon monoxide at 28.0101 g/mol sits within a whisker of nitrogen's
  28.0134; the catalogue holds both, so the CO row is no longer counted as nitrogen.
- The Julian year (31557600 s) and the sidereal year (31558149.8 s) are both catalogued; only tests use the
  sidereal one.
- Round numbers (44.0, 3e8, 1000) are not counted: they carry fewer than 3 significant digits after zeros are
  stripped, and are almost always not the constant they look like.
- The textual scan cannot see a value built by arithmetic (`G * M_SUN`) or a value read from a pack and
  converted. Those read a typed copy that the audit does count, so the root is still listed.

## The "from memory" values

`packs/base-light/pack.json` says the refractivity of ammonia, sulphur dioxide, carbon monoxide and neon is
"from Kaye and Laby at 589 nm (from memory, verify)". MP0 lists them (section C); MP2 checks them against a
source before they enter the sheet.

## Next

- MP2 (cited first-wave data) builds the sheet rows from section A's canonical values and section C's pack
  values, checking each source; its log lists every value that changes.
- MP3 to MP6 move each reader to the sheet and add the quantity ids they retire to a list that
  `scripts/audit-matter.py --retired` checks in CI.

Goldens: none changed.
- MP22 (`packages/MP22.md`) turns this audit into the ledger `MP0-typed-copies.txt` and the `--ledger` check.
