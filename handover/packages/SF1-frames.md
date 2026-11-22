# SF1: surface time-lapse frames (`surface_frames`), part 1

Asked for by the renderer thread (22 Nov) for Alex's video of system generation, showing geology, volcanism,
glaciation and climate working together. The renderer builds the time-lapse layers on the Body globe against this shape.

`sse-surface/src/frames.rs`; kernel op `surface_frames` (`crates/sse-kernel/src/frames.rs`).

## Request and reply
`{op: "surface_frames", fixture | system + body, at_s?, options?, detail?: "orbit" | "survey", frames_max?}` returns
`frames_made {frames: {body, label, source, at_s?, frames: {version, detail, epoch_myr, grid {n, cells}, dirs,
frames, events, pending, why}}}`.

- **grid**: the S2 plate grid, a cube sphere with n cells a face, 6·n² cells, face-major then row-major (`plates::Grid`).
  Orbit is the coarse grid (32 a face at pack defaults); Survey is the fine one (64).
- **dirs**: the cell centres as unit vectors, x y z per cell, given once.
- **frames**: one per plate snapshot, every `snapshot_every` steps (Earth: 45 frames over 4.5 Gyr). Each holds
  t_myr, sea_level_m, plate (u16), continental, elevation_m, vent (0 to 1), ice_m, temp_k, rain_m_yr,
  runoff_m_yr and why. A world whose plates never moved gets 13 evenly spaced frames on one fixed surface.
- **frames_max** thins the frames evenly and always keeps the last (the contract asks for 1 to keep its golden small).
- **events**, in date order: the plate events (S2: rift, collision and the rest, with plates), eruptions (shield
  volcanoes, with dir and radius_km), flood-lava (lava plains), and the 200 largest listed impacts (dir and
  diameter_km). Each has why (empty for now, as the renderer asked).
- **pending**: what is not yet dated: ice-age onsets, worn elevation, oceans over time, sea ice, and, for a world
  with a fixed climate value, its temperature history.

## How
- `plates::history_recorded` is the same plate run as `history`, but it keeps every cell's crust at each snapshot
  (`Snapshot.cells`, never serialised). Recording changes nothing the run computes, so the surface goldens hold.
- Elevation is `plates::cells_heights` (split out of `cell_heights`) at the snapshot date, taken from that date's
  sea level for today's ocean volume. These are tectonic heights: S3 wears only the last window.
- Vent is the most recent of a cell's arc (counted at half), plume and flood dates, decaying as exp(−Δt / span),
  where span is the time between frames. Shields active at the date and plains laid within the span also light
  their footprints.
- Climate is the one climate (`climate::world_climate`) at the frame's own date. The world's age is set to the
  date, so the host star's light and the envelope's mean temperature are those of then. Each cell's values come
  from `water_at` at its latitude and height above the sea: temp_k is t_k, rain_m_yr is precipitation, and
  runoff_m_yr is runoff on land.
- An ice sheet covers land whose ice balance is above zero. Its thickness follows Nye's perfectly plastic profile,
  H = √(2 τ₀ x / (ρ_ice g)) with τ₀ = 100 kPa, where x is the distance in from the ice edge (counted
  breadth first over the grid).
- Cost: Earth at the Orbit level takes about 1 s native.

## Checks
- `frames_tests::earth_frames_span_its_history`:
  - one value per cell in every field, all finite;
  - dates strictly increasing and ending on the epoch;
  - plates moving under more than 5% of cells;
  - land between 5% and 50% today, an ice sheet thicker than 500 m, the warmest cell above 290 K;
  - events in order, pending listed;
  - reproducible.
- `frames_tests::frame_limit_keeps_the_last`.
- `frames_tests::still_worlds_get_frames`: the Moon gets frames on one surface, with impacts and flood-lava events.
- Contract: Mars, and Jupiter (refused: no ground). `contract.jsonl` is re-blessed on purpose for them.

## Results
- Earth (Orbit): 45 frames and 486 events. Today it has 20% land and 10% of cells under ice, with a mean of 288 K.
  The surface fixtures' climates are fixed values, so their temperature history is pending. A world from a formed
  system (a star climate) gets its faint young star.

## Still to do
- Worn elevation per frame: wear each frame's window, or carry S3 through the whole history.
- Ice-age onsets as events (W9 part 2's `IceAgeModel::onsets_yr`, inside the orbit-history window), and land per
  latitude strip for the climate (W7c's `land_by_lat`, `coasts_by_lat`) from each frame's heights.
- Oceans and air over time (atmosphere history), and sea ice.
- base64 f32 arrays (`_b64` names) if JSON size bites at the Survey level.
