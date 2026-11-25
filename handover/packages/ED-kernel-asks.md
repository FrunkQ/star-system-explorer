# ED kernel asks K1, K2, K3, K15, K17, with class_sample, from_star and the observe merge

Kernel thread ("Build kernel resolve and rules"), 24 and 25 Nov 2026. The K numbers are the editing design's
kernel list (`handover/design/SSE4-editing-design.md`, "Kernel"), not the old K1 to K10 build packages.
Rides the PR after golden split part two (`GS2-golden-split-part-two.md`).

## What landed

- **K1 `sweep`**: `start_job` with `JobSpec::Sweep {system, address, field, from, to, count 2..64, log?,
  results[], packs?, scheme?, refine? (default 4, max 8), at_s?}`. Each step sets the field on a throwaway
  branch, resolves the system, reads each result (a fact, or a dotted path into the surface), the class label,
  and the stability verdict. Where the label or verdict changes between two steps it bisects (geometrically
  when `log`) `refine` times. Result `{address, field, steps, tipping_points[{what, from, to, between}]}`.
  Cancellable; `CancelJob` drops the branch. Test: Mars tips to "watch" between 2.531 and 2.562 au.
- **K2 `expects`** `{system, address, fields (1..32), at_s?}`: the worked-out value of each field with that
  field left out, on a throwaway branch. `body.radius`, `planet.surface_temperature` and
  `body.surface_gravity` map onto the resolve's own results. Reply `Expected{address, fields[{field, set,
  expects, from | why}]}`.
- **K3 makeup all the way through**: a hand-made world with a mass and a makeup but no radius gets one from
  `sse.interior.rocky_radius` (Zeng et al. 2016), core share from metal over metal, rock and carbon (0.32
  when unknown), widened by half the ice share. Worlds more than half gas get none (no giant radius block
  yet). `core_mass_fraction` and `water_fraction` follow the makeup.
- **K15 resonance protection**: a new block `sse.stability.resonance`. For each neighbour pair the kernel tries
  every p:q up to six outer turns and third order. A pair is held when its period ratio sits inside the
  resonance's width, which is δa/a = (16/3 f μ e^(p−q))^½ (Murray and Dermott 1999 section 8.7, f an
  assumed 1.5) for p > q, or the Hill scale (μ/3)^⅓ for a shared orbit (Dermott and Murray 1981). It is not
  held inside the overlap zone, max(1.3 μ^(2/7), 1.8 (μ e)^(1/5)) (Wisdom 1980; Mustill and Wyatt 2012).
  A held pair's Hill verdict of 2 or 3 drops to 1, saying which resonance, and a `resonance` check is
  listed. Two orbits of the same size are now a shared orbit, not a refusal. Being inside the width is
  needed, not enough (the resonant angle must librate), so a held pair is never "lasts". Tests: Pluto is
  held 3:2 by Neptune and nothing else among the planets; a Trojan shares Jupiter's orbit; 20% outside it
  the resonances overlap and the crossing orbits read 3 again.
- **K17 `elements_in`**: orbital elements typed against another plane (a pole and a zero direction),
  about the pair's shared centre, or true on a date, converted to the system's plane and the host, with the
  edits to send. Nothing is written.
- **`class_sample` {packs, classes (up to 128)}**: a sample world per class for the class gallery, hand-set on
  a "class samples" branch from the class's most central example, then resolved; per sample its fit, the
  example's fit, the fact that holds the fit down, and whether it has ground. Of the shipped atlas's 69
  classes, 27 keep their class once resolved; 42 are held down by a fact the campaign does not publish yet.
- **`from_star`** on `run_formation`: form the planets round an existing drawn star; the system address becomes
  the star's.
- **Observe stacks**: views from several observe packs merge by id, a later pack replacing an id.
- **ops()**: six missing rows (hold, who_holds, holdings_in, set_tech, tech_of, tech_reading) with contract
  cases.

## Citation corrections (Alex's Consensus runs, 25 Nov)

Claims fixed to match the papers (corrections-01, 02-05 and 06-09 in project files `sse4/citations/`):
nucleus seed mass (now the 1e4 to 1e6 range, fudge F60) and the hot-flow threshold as alpha squared with
alpha as pack data (`disc_alpha`, 0.1); L ∝ ṁ² credited to Narayan and Yi 1995; Antonucci "much, not all";
Kormendy and Ho (holes track bulges) and Reines and Volonteri (2.5e-4) in galaxy.rs and the pack; Bullock's
spin is λ′; Tegmark's 1e6 is baryonic; Lada and Lada measured 50 to 1000 M☉; Skumanich's inverse square
root; Frankel's title; Boylan-Kolchin's journal and DOI; Hills 1981's 1e4 au; Cassini (not Juno) for
Jupiter's 7.5 W/m²; Zeitlin's dose inside the spacecraft; ice conductivity in W/m/K; Konôpková on solid iron;
Ojakangas's 25 km bound (the test's bound with it); Venus ionopause heights; McDonough and Sun's 20 TW as our
sum; Mars's volcanism date as our reading; Park's Ceres (rock from volatiles); Sirius A at 24.74 L☉ (Bond et
al. 2017; the flow case's expected value moved with it); Kesler and Wilkinson's tonnage at all depths
(fudge F61). The Petit, Laskar and Boué DOI was already right.

## Goldens

Re-blessed on purpose: the contract golden moves with the new ops' cases, the six ops' cases, the resonance
block (compiled-in formulas change the manifest hash), the K15 contract case, and the citation strings that
the contract captures (cosmology pack text, reach laws, formula references).
`tests/golden/nucleus.json` moves in the last digits only: the hot-flow threshold is now alpha squared
(0.1 × 0.1 = 0.010000000000000002 in floating point) instead of a typed 0.01, which shifts the digests and the
last place of three luminosities and Eddington ratios.

Stack: `class_sample` and sweeps resolve a system inside their own op. Going through `dispatch` a second
time stacked two of its large debug-build frames under a resolve and overflowed the test thread's 2 MB, so
they call `Kernel::resolved_system_op` (the `resolved_system` arm, moved out of `dispatch`) directly.
