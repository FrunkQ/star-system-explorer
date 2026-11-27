# EC2: the sea glint's law in the kernel, the liquid's index from the data sheet

Asked by the coordinator's audit (26 Nov 2026) against the rule that physics lives in the kernel's editable blocks
and material numbers come from the data sheet. EC1 had typed water's index 1.333 into the renderer and written Cox and
Munk's law there. Thread "Draw shadows, eclipses and sea glint".

## What it does
- **Block `sse.light.sea_glint`** (`packs/base-physics/formulas/light.json`): σ² = a + b U (Cox and Munk 1954, the
  clean-sea fit, a = 0.003 and b = 5.12e-3 as fitted models) and R₀ = ((n − 1)/(n + 1))² (Born and Wolf 1.5.3).
- **Weather op `want: "glint"`** (`crates/sse-kernel/src/sea.rs` `glint`): for a world asked by system and body (or a
  weather fixture), the waves' mean square slope is the year's mean over each band's 10 m winds (the wind spread
  `sse_climate::wind::expect`), weighted by its sea and area, with the wind scaled by the air-to-liquid density as the
  waves are. The index is looked up on the data sheet (`substances.<liquid>.optical.refractive_index_liquid` at
  550 nm), from this universe's sheet, or the base sheet where it has no row (as `read_bows` does). Null without a
  sea; a liquid without an index is named under `missing`. Earth's fixture gives σ² 0.0346 (wind 6.1 m/s), n 1.336,
  R₀ 0.0207.
- **Renderer**: `GpuItem.glint` and `TrueBody.sea` now carry `slope_variance` and `normal_reflectance`; the
  shader uses them straight. `slopeVariance`, `WATER_INDEX` and `GLINT_WIND_FALLBACK` are gone; one stand-in pair,
  `GLINT_FALLBACK`, is the kernel's own answer for water at 7 m/s and is used only until a world's answer is read.
  `readSeaGlint` (eclipse-read.ts) asks the weather op; the system chart asks for each world whose face shows liquid,
  and System3D passes it to `trueLook`.

## Light time for every caster (from RW17, "Make eclipses happen on time")
- A shadow's caster is read where it stood when the starlight now reaching the world went past it: its place less its
  velocity times the light's time to the world (`eclipse.ts` `seenFrom`; the Moon over Earth, 1.25 s and 1.28 km). Any
  caster, no special case. RW17 found eclipses about 37 s late without it.
- Velocities come from the kernel: the shell asks `orbits_at` for now and 60 s earlier (`MOTION_STEP_S`), and
  `motionsFrom` takes each body's move between them; `orbit.v_m_s` on the scene, `Body3D.v_m_s`, summed up the chain
  in `placedBodies` (a velocity only where the whole chain has one).

## Golden
- `tests/golden/areas/light.jsonl` gains two lines at the end (weather glint for the Earth and Jupiter fixtures);
  nothing existing moves.
- `tests/golden/contract-reach.txt` gains one line, the new block's "cannot follow" (every atom is listed there).
- Approved by the coordinator under Alex's 26 Nov standing word that the coordinator decides kernel changes.

## Checks
- `sse-atoms`: the block's example, and a test with absolute numbers (σ² 0.0542 at 10 m/s, R₀ 0.0204 for water).
- `sse-kernel`: the imported Earth's glint (index from the sheet, R₀ near 0.0204, wind 3 to 15 m/s, σ² 0.02 to 0.09).
- Renderer: `eclipse.test.ts` 9 pass. In the browser, a ring profile swapped after the first frames (new key, new
  width) is read by the planet's ring shadow at once, pixel for pixel the same as a world drawn with it from the
  start; so EC1's binding trap is only the placeholder before first use, and no rebuild was added.

## Not done
- The weather op's refusal now names `glint` too (PR 646, with the storms thread's `patch`).
- The glint is not yet read for the size comparison or the class gallery (they draw at the stand-in).
- The kernel's `observe.reference` in the imported-system test had no optical row for water; the survey's sea optics
  read that sheet too, so its `refractive_index` may be missing in that path. Worth a look by the climate thread.
