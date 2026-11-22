# R6 part 1: time-lapse layers on the Body globe (renderer side)

Asked by Alex (22 Nov) for a video explaining system generation, with plates, volcanism, glaciation, climate and
weather shown together as they interact. Split agreed 22 Nov:
- **Proven core** builds the kernel op `surface_frames`: dated keyframes of per-cell fields on the S2 plate grid,
  plus events.
- **Climate thread** dates the climate fields and the ice-age onsets for that op.
- **Chart room** builds the Body screen controls: layer toggles, the time scrubber with play, event marks, and the
  "why is this here?" card.
- **Renderer** (this package) builds the reader, the frame blending, the layers, pick and the globe API.

## What was built
- `packages/renderer/src/timelapse.ts`:
  - `readFrames(reply)`: the surface_frames reply as typed arrays. It accepts JSON numbers or `<name>_b64` f32
    arrays, sorts frames in time order, checks the grid size, and builds a direction-to-cell lookup.
  - `cellOf`, `bracket(t)`, `valueAt(field, cell, t)`, `seaLevelAt`, `eventsBetween`:
    - Smooth fields (temp_k, rain_m_yr, ice_m, runoff_m_yr) blend between keyframes.
    - Plates, continental crust, elevation and vents show the nearer frame. A blend of two plate maps is not a world
      that existed.
    - A field no frame carries is null, never invented.
  - `LAYERS` (data): Land and sea, Plates, Volcanism, Surface temperature, Rain and snow, Ice and glaciers, Water
    running off. Each layer has a legend and its limits line. `layersFor` marks a layer unavailable when its fields
    are absent, and lists the fields the kernel names as pending.
  - `layerColour`, `onPlateEdge` (plate boundaries), `pickAt` (field values, the nearest dated event and the kernel's
    why lines), `narrateFrame` (the text tier).
  - `recolour(prepared, tl, layer, t)`: the globe's own samples recoloured. Both the canvas and GPU tiers draw it
    unchanged. Away from the last keyframe, the shading normals are the sphere's own, because today's hill shading
    on a past world's coasts would be false.
- `packages/renderer/src/runtime.ts`: the GlobeRun API for the chart room: `setTimeLapse`, `setLayer`, `setTime`,
  `layers()`, `frames()` (keyframe times, and events with lon and lat) and `pick(lon, lat)`. The narration follows
  the time lapse while one is set.

## Evidence
`node --test packages/renderer/test/timelapse.test.ts` passes 5 of 5, on a synthetic 8-per-face reply with three
keyframes, two moving plates, an ice age and an eruption:
- frames come out in time order, and base64 and JSON are read alike;
- every cell is found from its own direction;
- ice blends halfway at the midpoint, while plates take the nearer frame;
- pending rain is named, not invented;
- ice shows white, eruptions glow and warm reads warmer;
- the plate edge is found on the splitting plane;
- pick returns the eruption with its why line;
- recolour makes the pole icy and leaves the equator not, uses flat sphere normals in the past and keeps hill shading
  today;
- the words give the time and the ice-age event.

`npm run ui:test` and `npm run ui:check` are clean.

## Next (part 2)
- `session.surfaceFrames` once the op lands; a real Earth-like run in `render.e2e`, filmed for the trailer.
- The live weather layer from `weather {want: "field"}` (cloud and wind per tile).
- Mountains rising as heights change: per-frame hill shading from the frame's elevation.
- River channels when a flow-routing field is sent.

# R6 part 2: Earth through time on the Body screen

## What was built
- **`session.surfaceFrames({fixture | system + body, at_s?, detail?, frames_max?})`** calls the kernel's
  `surface_frames` (proven core, PR 135).
- **`Body.svelte`**:
  - A reactive `lapse` API that the chart room's `BodyTimeline.svelte` drives: {loaded, loading, error, layer, t_myr,
    layers, frames, load(), close(), setLayer(), setTime(), pick()}.
  - The loader reads the current world's frames and attaches them to the globe runtime. It attaches them again if the
    globe remounts, and closes them when a different world is picked.
  - An interim control in one node, `[data-lapse-controls]`: a "Through time" button, a layer select, a "When" range,
    a "Back to today" button and the layer's limits with the kernel's pending notes. BodyTimeline replaces this node.
- **Volcanism** reads three recency fields (each 0 to 1, where 1 means it happened at this frame): `ridge` (new ocean
  crust, drawn faint), `arc` (volcanoes over sinking plates) and `vent` (plumes, floods, shields and plains, drawn
  bright). These follow proven core's split of vent.
- **Pending notes** come as sentences (the kernel's limit lines). Each note goes to the layers whose fields it mentions.

## Evidence
- `timelapse.test.ts` passes 6 of 6. The new test uses the kernel's Earth (frames_max 12) and checks:
  - every layer has its fields, and collisions are in the event list;
  - today's land share is between 15% and 50%;
  - the south pole is more than 30 K colder than the equator and carries ice;
  - the same request gives the same plates.
- `render.e2e` chromium:
  - "Through time" reads Earth's history in about 2 s;
  - the plates and ice layers draw 3.97 billion years ago and differ from today;
  - the words give the time, the layer and the land share;
  - "Back to today" restores the present;
  - the state hash is unchanged.
- Screens are in `R6-screens/`.
