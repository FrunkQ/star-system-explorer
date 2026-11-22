# MP2: first-wave cited data for the master data sheet

Track MP, design `handover/design/SSE4-matter-pack-design.md` sections 4, 10 and 11. Depends on MP1 (schema and
loader, kernel thread "Build kernel resolve and rules").

## State, 21 Nov 2026

- Done: the design's section 11 sources checked; register in `docs/citations/matter-reference.md`.
  11 of 12 DOIs confirmed. Rudnick and Gao 2014's DOI is still unseen (the chapter number fits it).
- Done: the four "from memory" gas refractivities in `packs/base-light`. Ammonia (3.76e-4) and neon (6.71e-5,
  the pack rounds to 6.7e-5) match Kaye and Laby through the PDG gas pages. Carbon monoxide and sulphur
  dioxide are still unverified: no page with a value was reached.
- Done: the first-wave sheet, `packs/base-reference/pack.json`, in the shape agreed with the kernel thread
  (design section 4; tables under `provides.reference`). Built by `scripts/build-base-reference.py`
  (`--check` fails if the pack is stale; the script also checks that every `src` names a source in the pack,
  every formula names an element, every recipe names a substance, and ids are unique).
  - 48 constants (the 7 SI defining constants, CODATA 2018, units and conventions, the Sun, Earth, Jupiter
    and Moon; masses derived from IAU GM over G, never typed).
  - 118 elements: IUPAC 2021 standard atomic weights (conventional values for the 14 interval elements;
    none for the 34 without a standard weight), Lodders 2021 protosolar abundances for the 81 she lists.
    `handover/packages/MP2-data/elements.csv` is the input table.
  - 34 isotopes: the 5 heat producers with today's shares and decay heat (Ruedas 2017), fuels, stable
    references and dating clocks.
  - 29 substances: the 13 light gases, krypton and xenon (dry air's traces), the 5 climate condensables and the 26 life species merged by
    substance (dissolved forms under `chemical.states.dissolved`). No molar mass is typed.
  - 11 mixtures: dry air (USSA 1976 recipe with its traces), giant air, surface rock, surface ice, magma, and the six formation materials.
  - 30 sources, each with a DOI or URL; register rows in `docs/citations/matter-reference.md`.
- Nothing reads the pack yet (MP1 is the reader), so no behaviour or golden changes.
- Honesty: a value with no `check` was read from its source. `unverified` means the source is named but the
  number was not read from it in this pass (CODATA's non-exact constants, the NUBASE half-lives beyond the
  heat producers, most Kaye and Laby refractivities). `authored` is a stand-in in the fudge register.

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
| ammonia triple point | 194.95 K (base-climate) | 195.4 K | 194.95 K is the melting point; NIST's triple point is 195.4 K |
| dry air molar mass, worked out | 28.9595 g/mol (base-light) | 28.965 g/mol from the USSA 1976 recipe without trace gases | USSA 1976 quotes 28.9644 with its traces |
| sulphur dioxide molar mass, worked out | 64.066 g/mol (base-light) | 64.058 g/mol | sulphur's 2021 conventional weight is 32.06 (was 32.065) |
| argon molar mass, worked out | 39.948 g/mol (base-light) | 39.95 g/mol | argon's 2021 conventional weight; the standard weight is an interval |
| ice density | 930 kg/m3 (sse-surface) | 917 kg/m3 for pure ice Ih; 930 kept as the authored "surface ice" mixture | the round-body calibration uses 930 |
| other worked-out molar masses | 5 to 7 digits in base-light and base-climate | from formula and IUPAC 2021 weights | all within 6e-5 relative |

## Where each element was made (design section 8.1, Alex 21 Nov)

Every element row now carries `origin` (shares by source, adding to 1) and `origin_story` (physics, plain,
story). `provides.reference.origin_legend` is the one seven-entry legend the whole app uses (Big Bang, cosmic
rays, dying low-mass stars, exploding massive stars, exploding white dwarfs, merging neutron stars, made by
people). Input: `handover/packages/MP2-data/origins.csv`; stories in `scripts/element_origins.py` (13
written by hand, for H, He, Li, Be, C, O, Fe, Ba, Eu, Pt, Au, U and Tc; the rest built from the shares).

**Honesty.** Every share is `check: authored`: an approximate, rounded first pass, not a number read from a
table.
- No per-element table of shares could be read. Johnson's origin chart exists only as images. Kobayashi et
  al. 2020 shows its shares in a figure, and its text gives only group statements (about half of C, N and F
  from AGB stars; about half of the iron-peak elements from Type Ia supernovae), which the light-element
  rows follow.
- Bisterzo et al. 2014's per-element s-process table lies beyond what the fetch tool could read. The heavy
  rows use the classic solar s/r split, as remembered. Check them against that table.
- 49 rows carry an `argued` line, and their stories say so. They cover:
  - every element with a large rapid-process share (given here to neutron-star mergers, seen once in
    GW170817, while rare supernovae may add a share);
  - lithium (the cosmological lithium problem);
  - fluorine and titanium.
- The `decay` rows (Po to Pa) exist only as decay products of uranium and thorium. The rows marked as made
  by people (Tc, Pm and Np onward) have no isotope that survived from the stars.

**Next.** MP7 replaces these static shares with the galaxy model's own ledger per star, which is what lets a
card say "about 40 mergers seeded this part of the galaxy". The citations track should check the heavy
s-fractions against Bisterzo 2014 (VizieR J/ApJ/787/10) and the light rows against Kobayashi 2020's figure
when a reader can open them.

## Units (design section 7A.6)

`provides.reference.units` holds 45 units, each `{id, symbol, name, measures, group, factor | factor_from,
offset?}`, in three groups: `si-base` (7 plus "one"), `si-named` (newton, pascal, joule, watt, hertz, tesla)
and `working`. Ids match the kernel's 34 built-in units, and every factor agrees with the built-in one to
1e-9. A factor that follows from a constant is a formula (`c yr_julian`, `au 648000 / pi`, `e`, `R_sun`,
`pi / 648000`); only pure decimal prefixes and SI identities are typed. New ids: a, mol, cd, n, j, hz, atm,
ev, gee, degree, arcsec. Every constants row now has a `group` (nature, convention or factor). The exact SI
constants stay under nature, flagged `exact`, because they are physics whose values the SI fixed. The au, the
Julian year, the day, the atmosphere, standard gravity and the IAU nominal values are conventions. The parsec
and light year are factors.
