# MP1c part 2: stars follow a changed constant, stamped ESTIMATED

Design: `handover/design/SSE4-matter-pack-design.md` section 7A.3 (what a
changed constant reaches). Part 1 is `MP1c-part1.md`.

## What it does
- **Stellar homology** (Adams 2008, JCAP 08, 010, doi:10.1088/1475-7516/2008/08/010).
  - Main-sequence luminosity: radiative diffusion at a fixed opacity per mass gives L ∝ (a c / κ)(G μ m_p / k_B)⁴ M³. With a c = 4σ, at fixed mass that is L ∝ σ G⁴ / k_B⁴.
  - Radius: hydrogen ignites at a core thermal energy that nuclear physics sets, and the virial theorem gives k_B T_c ∝ G M μ m_p / R. So R ∝ G.
  - `sse_atoms::star::homology_luminosity_factor` and `homology_radius_factor` multiply the fitted relations (Eker 2018, Demircan and Kahraman 1991). With the real constants they return exactly 1.0, so every golden is bit-identical.
- **Reach.** `sse.star.ms_luminosity` and `sse.star.ms_radius` move from "cannot follow" to `Estimated`, and each names its law. The lifetime t = t☉ (M/M☉)/(L/L☉) and the age brightening are formulas on the luminosity, so they **follow**. The energy per kilogram of fuel is nuclear, so the lifetime moves only through L.
- **The stamp.**
  - `derive` values gain `estimated: [law, ...]` while the constants in force differ from ours and the value rests on a rescaled block.
  - `Evaluation::upstream_atoms(kind)` (sse-flow) gives every atom a published value rests on.
  - The field is left out otherwise, so no golden moves.

## Verified by
- `doubling_g_makes_the_sun_estimated_sixteen_times_brighter` (kernel). At twice G:
  - the Sun is 16 times as bright and twice as wide;
  - its lifetime is a sixteenth and its T_eff is √2 times as high;
  - all four values carry the Adams 2008 law, and the real kernel stamps none.
- At 4.57 Gyr, the same flow refuses, because such a Sun lives 0.625 Gyr. The test uses 0.1 Gyr, and that refusal is the right answer.

## Not yet
- Earth's two radii named apart (`R_EARTH` is equatorial in sse-core, mean in sse-surface and sse-atoms). The rename touches the owning threads' crates (surface, weather). I left it for them as each converts to the handle (design package table), so no golden moves underneath them.
- The lint (MP0's `audit-matter.py --retired` in CI) needs Alex's yes for the CI change.
- Opacity (κ) and the nuclear ignition energy are held fixed. Changing α or m_p would move them, and those constants are not in the handle yet.

## Goldens
`tests/golden/contract.jsonl`: only the `read_reach` G case changes, on
purpose. Four star blocks leave "cannot follow": two move to `estimated` and
two to `follows`. Every other line is identical.
