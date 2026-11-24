# SF1: surface time-lapse frames (`surface_frames`), parts 1 and 2

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
  t_myr, sea_level_m, plate (u16), continental, elevation_m, vent, arc, ridge (each 0 to 1), ice_m, temp_k, rain_m_yr,
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
- Vent is the most recent of a cell's plume and flood dates; arc its last date over a trench; ridge its ocean crust's
  birth. Each decays as exp(−Δt / span), where span is the time between frames. Shields active at the date and plains laid within the span also light
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

## Part 2 (24 Nov): the land of then, and ice ages

**Land strips per frame.** Each frame's climate now reads that frame's own land and coasts by latitude strip (W7c's `climate::with_land`), from its tectonic heights above that date's sea level. The ocean fraction follows too, so an early, nearly all-sea Earth has a sea world's climate, not today's continents under a fainter Sun. Dry worlds are unchanged.

**Ice-age onsets as events** (`kind: "ice-age"`):
- The kernel gives the frames an orbit track over the 400 kyr window that ends at the epoch. This is `ice_ages::orbit_inputs_from` with a start of −window; T1's secular theory runs either way in time, and the survey's own track still starts at the epoch.
- On the last frame's climate and heights, W9 part 2's model (`climate::ice_ages`, `IceAgeModel::onsets_yr`) dates where each land cell's ice balance turns from losing to gaining.
- Onsets are grouped by date. One is listed when it starts ice on at least 0.5% of the land; smaller ones are a mountain glacier's flicker.
- The text gives the share of the land, and why names the track.
- A world with no track (no sibling planets, a moon, or a fixture) keeps "ice-age onsets" in pending, with the reason.
- `Resolved.track_before` carries the track; `surface_frames` gains `track_before: Option<&OrbitTrack>`.

Check: `frames_tests::ice_ages_and_the_land_of_then` gives Earth one inclination mode and the Moon's pull, as the wear test does. The onsets fall inside the window, and without a track they are pending. That gives 2 onsets in 400 kyr; Earth's first frame is all sea, against 1180 land cells now.

Fudge register: the spin integration starts the window with today's tilt and perihelion, though the window ends today. Over 400 kyr Earth's tilt drifts about a degree, inside the drawn sibling inclinations' own spread.

Goldens: `contract.jsonl`, re-blessed on purpose for the Mars frames case, whose pending line now gives the reason ice-age onsets wait (no orbit track).

## Still to do
- Worn elevation per frame: wear each frame's window, or carry S3 through the whole history.
- Oceans and air over time (atmosphere history), and sea ice.
- base64 f32 arrays (`_b64` names) if JSON size bites at the Survey level.
