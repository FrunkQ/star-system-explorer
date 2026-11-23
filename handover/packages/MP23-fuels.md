# MP23 Ship fuels on the sheet

Asked for by the renderer thread for ship hulls (PR 219, V3 gap 14): a tank's
size follows from the fuel mass and the fuel's liquid density, so the
densities belong on the sheet, not typed into the hull code.

## What landed

- `isotopes` rows carry `mass` (u, AME2020): h1, h2 (D), h3 (T), he3, he4.
- The reader (`crates/sse-reference`) lets a formula name an isotope that has
  a mass; molar mass uses the isotope's own mass. Elements still win when the
  key is an element.
- `phases.density_liquid` at the normal boiling point (kg/m3, with the
  temperature in `at_K`):
  - h2 70.85 at 20.369 K (Leachman 2009, the normal-hydrogen EOS; CoolProp agrees);
  - o2 1141.2 at 90.188 K (Schmidt and Wagner 1985);
  - d2 163.5 at 23.31 K, new substance "deuterium" (Richardson 2014 EOS through
    CoolProp; marked computed);
  - he3 59.0 at 3.19 K, new substance "helium-3", **unverified, recalled
    and not read**; on the fudge register (docs/fudges.md, "Master data sheet") until a source is read. Sources tried 23 Nov: Kollar and Vollhardt (arXiv cond-mat/9906222) stops at 2.5 K; Wikipedia's liquid helium table has no density; CoolProp has no helium-3.
- Mixtures by mass for base-nav's fuel ids: `fuel-hydrogen`,
  `fuel-hydrogen-oxygen` (O/F 6:1 by mass, so 1/7 and 6/7; authored, a typical
  engine ratio, not a law), `fuel-deuterium`, `fuel-helium-3`.
- **No antimatter row on purpose.** Antimatter is not stored as a liquid of
  known density; how a setting holds it (Penning traps, frozen antihydrogen) is
  the ship pack's call. A hull asking for `fuel-antimatter` gets "no row" and
  should treat the tank as authored.

## Checks

- `ship_fuels_have_densities_and_isotope_masses` in `crates/sse-reference/src/tests.rs`.
- Generated files rebuilt (`build-base-reference.py`, `build-ui-reference.py`,
  `build-rust-reference.py --check`). No golden hashes change: nothing in the
  kernel reads these rows yet.

## Follow-ups

- Read a source for liquid helium-3's density and change its mark.
- Sources new here are graded unverified until the citation check reads them.
