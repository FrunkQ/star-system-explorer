# XO2 and XO3: what feeds a black hole's disc, and the disc's light point by point

Track XO (exotic objects), design `handover/design/SSE4-exotic-objects-design.md` sections 2.2, 3.3 to 3.5, 8, 9 and 11. Built 26 Nov 2026 by the thread "Build black hole disc light", one pull request for both (XO3 is the second half of the same light).

## What was done

**XO2, the feed** (design 2.2, 3.5): one descriptor on any body gaining matter, never asking what kind of body it is.

- Seven formula blocks in a new family file `packs/base-physics/formulas/feed.json` (loaded with the other base formulas, `crates/sse-atoms/src/formulas.rs`; first-wave count 176 to 183), each with worked examples run on load:
  `sse.feed.eddington` (L_Edd and the rate that gives it), `sse.feed.efficiency` (energy of a circular orbit at the inner edge, Kerr form inside 6 r_g), `sse.feed.inner_edge` (largest of surface, last stable orbit and ξ times the Alfvén radius), `sse.feed.disc_outer_edge` (Paczyński's 0.6 a / (1 + q)), `sse.feed.flow_strengths` (thin, hot and thick strengths from the Eddington share, plus how well a hot flow shines), `sse.feed.capture_from_surroundings` (Bondi and Hoyle). Fitted numbers (ξ, Paczyński's share, the flow joins, Bondi's λ) are block parameters, not typed numbers: the equation checker refuses a bare 0.6.
- `crates/sse-light/src/feed.rs`: the descriptor (`Feed`: rate, Eddington rate and share, efficiency, flows with strengths, disc edges and what set the inner one, light given off, Why, stand-ins, the blocks used). Sources: a given rate, a companion's overflow, or the surroundings.
- Op **`read_light_paths`** (`crates/sse-light/src/paths.rs`, kernel `systems::read_light_paths`): per body and far viewer, the bender (strong when the body's surface is inside the table's 64 r_g), the gravity blocks' sizes (horizon, photon orbit, last stable orbit, shadow and Einstein angles as that viewer sees them), the bend table's hash and layout, the feed, the traced disc, its light by band with the peak band, the viewer's clock rate and tide, marks (stand-ins) and blocks. `table: true` adds the bend table itself (`flat()`). Fixture `black-holes`: design 3.4's three columns.

**XO3, the disc's light point by point** (design 3.3):

- Block `sse.light.disc_shift` (in `light.json`): 1/g = (1 − 3/x)^(−½)(1 − (b/r_g) sin i sin α / x^(3/2)), α counted from the disc's axis as seen towards the viewer's left, the disc turning anticlockwise seen from the viewer's side. This is Luminet's eq. 19 with α's sign flipped; it is exact for no spin because b sin i sin α is the photon's conserved angular momentum about the axis, so the same expression serves every image order.
- `crates/sse-light/src/disc_image.rs`: the picture plane summed ray by ray (192 angles; b every 0.04 r_g inside 12, then 48 steps a decade). Each ray's first landing on the disc comes from the bend table at the crossing angles φ₀ = π/2 + atan(cos α tan i) and φ₀ + π (`landing`); temperature from the thin-disc law, shift from the block, each piece a blackbody at gT with area 4 dA (the far viewer's isotropic-equivalent factor). Pieces are binned by seen temperature, 64 bins a decade, each keeping its σT⁴ exactly, into a new light part `Part::Thermals`, so the band recipes colour it with no new colour code.
- `read_light`: an accretion's optional **`tilt_deg`** traces the disc as that viewer sees it. Without it the disc stays the isotropic `Part::Disc` with its old stand-in, so no existing golden moves.
- Exposure note: the brightest point's visible-band brightness against the Sun's surface (the sheet's T_sun), with a sentence the renderer can show.

## Results (fixture `black-holes`)

| Fixture | Feed | Disc as seen | Light |
|---|---|---|---|
| 10 Suns, fed by a Sun-like companion at 2.4 × 10¹⁵ kg/s, seen 80° from the axis (Luminet's view) | 0.098 of Eddington; thin 0.998; inner edge 6 r_g (last stable orbit); outer edge 0.6 a / 1.1 | approaching side 7.1 times as bright; brightest point at 7.8 r_g, 4.8 × 10⁶ K seen at 6.4 × 10⁶ K (g = 1.33); first ring carries 0.3% of the light; 3.2 × 10⁴ times the Sun's surface in visible light (the design's "dimmed 40,000 times") | peak X-rays (8 × 10³⁰ W), visible 2.3 × 10²⁷ W |
| 10⁸ Suns at 7.3 × 10²² kg/s (1.2 Suns a year), seen 30° from the axis | 0.30 of Eddington; thin 0.97, thick 0.03; disc to a stated 1,000 r_g | side ratio 2.3; brightest at 8.6 r_g, 1.1 × 10⁵ K | peak ultraviolet (7 × 10³⁸ W); visible 3.7 × 10³⁷ W, above a Milky Way's whole light |
| 4.15 × 10⁶ Suns in hot gas (130 per cm³, 1.3 × 10⁷ K), seen from 8.2 kpc | Bondi capture 7.6 × 10¹⁷ kg/s, 7.5 × 10⁻⁵ of Eddington; hot flow 1.0 | the thin disc is a few hundred kelvin: nothing to see | 3 × 10³¹ W spread from X-rays to millimetres (the radio ring's 1.3 mm sits in the mm band), visible under a millionth of its Eddington light; shadow 51.9 µas across |

## Checks

`cargo test -p sse-light disc_tests` (7 tests), `cargo test -p sse-atoms` (the blocks' worked examples), `cargo test -p sse-kernel contract`, `scripts/check.sh`.

| Check (design 11 and 12) | Result |
|---|---|
| Fixtures reproduce the three columns of 3.4 as band totals | stellar hole peaks in X-rays, quasar in ultraviolet and outshines a galaxy in visible light, starved centre almost dark in visible light, hot flow dominant |
| Approaching side brighter by the stated factor | side ratio equals g⁴T⁴ between the two brightest points to 10⁻⁹; brightest point on the approaching side and blueshifted; face on the ratio is exactly 1 |
| Absolute check of the picture sum | a disc 2,000 to 60,000 r_g out (light hardly bends there) seen face on gives twice the plain Shakura and Sunyaev luminosity to 2%, and at 60° half of that to 2%: the image-plane sum is right in absolute terms, not as a ratio |
| Luminet's view | rays straight up the picture land on the far side after almost half a turn (the arch), inside half the flat-space distance; the first thin ring shows; a ray inside the shadow's edge is swallowed |
| Chart and picture agree | `read_light` with `tilt_deg` gives the same band powers as `read_light_paths`' disc, to 10⁻¹² |
| Last stable orbit, efficiency, Eddington | 6 r_g; 5.72% and about 32% at a = 0.998; 1.257 × 10³¹ W per Sun |
| Inner edge by rule, no kinds | hole: last stable orbit; white dwarf: surface; neutron star with 10²⁷ A m²: field, about 1,500 km |

Gates seen red: the flat-disc check fails if the area factor 4 dA is changed to 2 dA (it reads 0.5 of the wanted total), and the Luminet test fails if the sign of α given to `sse.light.disc_shift` is flipped (the side ratio drops to 0.14: the brighter side is then the receding one).

## Through the physics in force (ER1)

Merged after ER1's side-door ledger, so the new code reads through its doors and adds no ledger line. The feed's and the disc's blocks come from `sse_physics::now()` then `.blocks()`, with their shipped settings (each parameter's default, in `feed::run_block`, until ER2's `physics::run`). The exposure note reads `T_sun` from the sheet in force; with no Sun on the sheet (the contract runs on base-physics alone) `times_sun_surface` and `sun_surface_visible_w_m2_sr` are null and the note gives the brightest point's brightness in W m⁻² sr⁻¹. The fixture bodies live in `crates/sse-light/src/scenario.rs` (the crate's shipped scenario). Tests put the shipped physics in force (`disc_tests::in_force`).

## Golden

Area `exotic` only: three new cases in `tests/contract/areas/exotic.json` (the fixture, a bare hole, an unknown fixture), blessed into `tests/golden/areas/exotic.jsonl`. `tests/golden/contract-reach.txt` gains one line per new block. No line of the core `tests/golden/contract.jsonl` moved (the contract runs on the frozen base-physics pack). `packs/base-physics/pack.json` gains the new kinds (regenerated by `SSE_BLESS=1 cargo test -p sse-atoms`).

## Fudges and citations

`docs/fudges.md` section "Exotic objects": Newtonian disc brightness, the fixed picture grid and two orders, far viewer (no cosmological redshift on the quasar), hot flow glow, thick flow drawn as thin, the argued flow joins, Bondi as an upper limit (the starved centre glows several hundred times what is measured, though still under a millionth of its Eddington light), pure-hydrogen Eddington, the efficiency's spin case, the quasar's stated edge; and why the trace is Rust. `docs/citations/exotic-objects.md`: ten new rows. The Consensus search is out of allowance until 1 Dec 2026, so most are marked unverified (from memory); the master list has them (`scripts/citations.py --add`).

## For XO4 (the renderer's `light-path` stage)

Everything the shader repeats, so its picture and these numbers agree:

1. The table: `read_light_paths` with `table: true` gives `bend_table.flat` (per row: b, captured, end angle, turn, then 256 cells of u = r_g / r), keyed by `table.hash`. Index formulas: `sse_gravity::bend::row_of_b` and `phi_of_col`.
2. Per pixel: b and α in the picture (α from the disc's axis as seen, towards the viewer's left), the crossing angles φ₀ = π/2 + atan(cos α tan i) and φ₀ + π, first hit inside the disc's edges wins (`sse_light::disc_image::landing`). Beyond 64 r_g a straight ray, r = b / sin φ₀.
3. At the hit: T from `sse_light::disc_temperature` (mass, the feed's disc rate `Feed::disc_rate_kg_s`, inner edge), g from `sse.light.disc_shift`, colour as a blackbody at gT through the band's recipe; brightness times g⁴.
4. Exposure: `image.exposure.times_sun_surface` and its note (null when the sheet in force has no Sun; `brightest_visible_w_m2_sr` is always there).
5. Shadow and ring sizes on screen: `shadow_angle_rad` (and the √27 edge in the table layout).
