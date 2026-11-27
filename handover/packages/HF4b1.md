# HF4b.1: the lid as kept state

Track HF (heat flow), owner thread "Fix low heat flow in worlds". Design: `handover/design/SSE4-melt-and-heat-design.md` sections 3.2, 6.1 and 7. Status: `handover/status/heat-flow.md`.

## What changed

- **The lid is kept state.** Under a stagnant lid the thermal history (`crates/sse-surface/src/core.rs`) now keeps the lid's thickness from the march's start.
  - The lid's base sits `sse.heat.lid_base_temperature` below the mantle (a_rh 2.24).
  - The surface gets the lid's own conduction (`sse.heat.lid_conduction_with_burial`, nothing buried yet).
  - The lid grows by the Stefan rule of `sse.heat.lid_growth`, stepped backward (the positive root of its quadratic), so it is stable at any step.
  - It starts at the thickness the Stefan rule gives over the first step.
  - The mantle still loses only what convection brings it (`top`, unchanged). So a young lid's surface heat falls as 1/√t while the mantle's loss barely moves.
  - Under moving plates the lid is the plate: the rock that conducts what the mantle loses, with no store of its own.
  - The blocks are read through the door (`sse_physics::in_force()`), so an edited block reaches the march. sse-surface now depends on sse-physics.
- **The grid is the body's own** (6.1).
  - The first step is 1 Myr, each next one 1.25 times the last, capped at `step_myr` (20).
  - New settings: `grid_first_myr`, `grid_growth`.
  - The march runs to the first grid point at or past the body's age. The step published for its age is read between the last two points (`CoreStep::between`). Events after the age are dropped, and `dynamo_now` and `why_now` are those of the last grid point within it.
- **Heat fields at every step**, W/m² of surface, named as in section 7:
  - `heat_lost`, `heat_conducted`, `heat_erupted`, `heat_made`, `heat_radiogenic`, `heat_tidal`, `core_heat`, `core_freezing`, `core_cooling`, `mantle_cooling`, `lid_cooling`, `lid_km`.
  - `crust_km` and the `melt_*` fields are published as zero until HF4b.2.
  - `q_surface_tw` is now `heat_lost` times the area, so the heat screen's existing reads follow.
  - `core_history` takes the heat made as (radiogenic, tidal).

## A departure from the design, and why

The design has HF1's clamp (the mantle can be no hotter than the solidus at its lid's base) read the kept lid. Built that way, the clamp breaks Mars, a guarded limit:
- A young kept lid is a few km thick, so the solidus there is near the surface solidus (about 1,420 K).
- The clamp vented 500 K of a 1,950 K mantle in the first 1 Myr step (70 W/m² erupted).
- That left the mantle too cold to drive the core, and Mars's field died at birth (4.50 Gyr ago, against the guarded 3.6 to 4.1).

No melt supply could carry that much heat that fast. The clamp is HF1's stand-in for melting, and HF4b.2 replaces it with melting itself (supply-limited, Darcy drainage). So in this package the clamp still reads HF1's steady lid. The kept lid drives only the surface's conduction and the lid's own store. Mars's field now dies 3.95 Gyr ago.

## Tests (sse-surface `core_tests`)

- `heat_fields_close_at_every_step`: the four closures of section 7 at every step of every fixture with a core, to 1e-12 of the largest flow in the sum.
- `a_young_lid_cools_as_a_half_space`: Mars between 2 and 15 Myr after its start. Surface heat falls by the √t ratio (within 15 per cent), the mantle's loss changes by under 10 per cent, and the lid is under 50 km and growing. Seen red with the surface reading the mantle's own flux (HF1's behaviour): it fell 1.01 times against 2.59.
- `read_at_a_date_equals_the_line_read_there`: Mars read at 3.0 Gyr is the 4.5 Gyr line up to there, read between its grid points, bit for bit.
- The guarded tests hold: Mars's field dies young, Earth's never stops and its inner core is young, Venus has no field.

## Goldens (core lane)

`tests/golden/surface.json` (core history numbers: every world's grid and its surface heat) and the core lines of `tests/golden/contract.jsonl` that carry core numbers. The reasons: the new grid, the lid's conduction at the surface, and the new fields.

## Cost

About 250 grid points to 4.5 Gyr, two block runs a step. The whole sse-surface suite runs in the same time as before (about 60 s, release).
