# R2b part 1: today's weather on the Body globe (clouds, haze, storms)

The weather track's `weather` op has served cloud fields since W5. This package draws them: the Body globe shows
today's clouds and haze over the same surface, and the words name the cover, the rain and the largest storms.

## What was built
- `packages/ui-contract/src/session.ts`: `session.weather(ask)` calls the kernel op `weather` (a fixture `world`, or
  a `body` of an adopted `system` at `at_s`; `want`, `day`, `tile`, `n`). Presentation only: nothing is recorded.
- `packages/renderer/src/weather.ts`:
  - `readWeather(tiles, systems, meta)`: the six level-0 face tiles as one map with a direction lookup (shared with
    the time lapse through `dirIndex` and `nearestCell`, now exported from `timelapse.ts`).
  - Each cell's clouds become one reflectance. A layer is a non-absorbing slab, R = (1-g)τ / (2 + (1-g)τ), from
    Bohren 1987 (doi:10.1119/1.15109), with g = 0.85 for droplets (Hansen and Travis 1974,
    doi:10.1007/BF00168069). Cover weights each layer, and the layers stack high over middle over low by the adding
    method. Aerosols form one haze slab (g = 0.7), tinted by the colours the weather block gives.
  - `withWeather(prepared, map)`: the clouds go over the ground by the same adding method (light bouncing between
    cloud and ground included), so the palette colour and the physical albedo change together. The chart recipes
    and True lighting both draw it. Cells blend with a Gaussian of half a cell, so the coarse tiles do not show as
    blocks; each cell's value is still the kernel's.
  - `narrateWeather`:
    - cloud cover and its main types and condensates;
    - the mean rain or snow a day, and the wet-day share (1 mm or more, the ETCCDI mark; virga is left out);
    - the three largest live storms, placed along their tracks on the day.
  - `weatherAt(lon, lat, radius)`: a cell's layers, rain and any storm over the point.
- `packages/renderer/src/runtime.ts`: `GlobeRun.setWeather(map | null)` and `weatherAt(lon, lat, radius_km)`. A time
  lapse wins over the weather, since it shows its own past.
- `packages/views/body/src/Body.svelte`:
  - A reactive `weather` object (`on`, `loading`, `error`, `day`, `show()`, `hide()`) reads the campaign's day from
    `campaignNow()` and then the six tiles (32 cells a face) and the storm list.
  - An interim `[data-weather-controls]` node ("Today's weather" or "Clear the weather") that the chart room can
    replace; it hides while a time lapse is shown.
  - `lapse.view(lon, lat)` turns the globe to a point, as the chart room's BodyTimeline asked, for clicked event
    marks.
- `docs/fudges.md`: three rows, which are:
  - clouds are laid on the ground and not drawn at height (no shadows, until part 2);
  - cover is area-weighted, with one g for every condensate;
  - the haze is one tinted slab.

## Evidence
- `node --test packages/renderer/test/weather.test.ts` passes 4 of 4:
  - the slab and adding sums: zero depth reflects nothing, τ = 10 reflects 1.5/3.5, and a mirror stays a mirror;
  - clouds whiten the ground only where the kernel puts them, and the prepared samples stay unchanged;
  - a storm is found along its track on the day, and named in the words;
  - the kernel's Earth: mean cloud reflectance between 0.1 and 0.5, storms named, the same tiles each time.
- `tests/ui/render.e2e.mjs` (R2b block):
  - "Today's weather" on the sample Earth reads in about 1 s and brightens the globe;
  - the globe's words read "cloud over about 67% of the globe (mostly stratocumulus, cirrus, nimbostratus, made of
    water); 3.5 mm of rain or snow a day on average, a wet day (1 mm or more) over 71%. Storms: a cyclone
    3,175 km across at 49°N 140°W; …";
  - "Clear the weather" takes it off, and the state hash is unchanged.
- Screenshot: `R2b-screens/render-weather-chromium.png`. The weather block's own zonal structure shows: cloudy
  storm-track bands, clearer subtropics and stratocumulus decks. Cyclones raise the cloud reflectance from about
  0.13 to 0.35 across their size.
- `npm run ui:check` is clean and `npm run ui:test` passes. `scripts/audit-matter.py --ledger` adds no typed copy:
  the day length is read from `chart-grammar`'s `DAY_S`.

## Limits and next
- Part 2, the GPU tiers: a cloud shell at the layers' heights with shadows on the ground and limb thickening, wind
  shown as moving cloud between days, and finer tiles (level 1) when zoomed in.
- Storms show only where the weather block's field carries them. Their spiral shape needs finer tiles, which is a
  weather-track question.
- Giants: the op's giant tiles (bands) go through the same path, but no giant has a Body globe yet.
