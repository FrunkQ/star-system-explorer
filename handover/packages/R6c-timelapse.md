# R6c: the time-lapse reads each frame's own world (sea, ice ages, climate)

Owner: "Build surface time-lapse" (R6 part 2, handed over by the renderer thread 24 Nov 2026). Kernel side: proven
core's SF1 part 2 (PR 394): each frame's climate takes that frame's own land, and ice-age onsets are dated events
(kind `ice-age`) over the orbit-history window that ends at the epoch.

## What was built
- `packages/renderer/src/timelapse.ts`:
  - A frame keeps `dry` (the kernel sends `sea_level_m` null on a world with no sea), its own `why` line and its
    lowest height. `hasSeaAt`, `underSea` and `nearestFrame` answer per time. Before this a dry world read sea level
    0 and drew every basin below the datum as ocean.
  - A dry world's ground is bare rock coloured from the frame's lowest cell up (no ocean, no watered green).
  - `iceAges(tl)`: the ice-age onsets and their window, from the first onset (rounded out to a tidy span) to today.
    `iceAgeAt(tl, t)`: the ice age under way. `iceAgeWords`: "Ice age k of n in the last …, begun … ago: …".
    `agoWords`: thousands of years under a million, so times inside the window read sensibly.
  - The ice layer washes the cells the kernel names for an onset (`cells` on the event) pale while that ice age is
    under way. The kernel does not send `cells` yet (asked of proven core); until it does nothing is drawn, nothing
    invented.
  - `pickAt` gains `sea` and `frame_why`; the narration says "no sea" on a dry world and names the ice age under way.
- `packages/renderer/src/runtime.ts`: `frames()` also returns `iceAges` (window and a line each) and each event's
  why lines (plumbing agreed with the renderer thread).
- `packages/views/body/src/BodyTimeline.svelte`: the ice-age onsets leave the main track (they would be one mark at
  its end) for a strip of their own under it: a hollow mark per onset, a fine range over the window, "… ago" ends; a
  chosen ice age shows its line and the kernel's why on the card ("across the world"); the card says "no sea" on a dry
  world; "ago" reads thousands of years inside the window.

## Evidence
- `node --test packages/renderer/test/timelapse.test.ts`: 8 of 8, including
  - a dry fixture: low ground is not sea, pick says no sea, the words say "no sea";
  - ice ages: the window is a tidy 100 thousand years for onsets in the last 90, the one under way is found, the line
    reads "Ice age 3 of 3 in the last 100 thousand years, begun 20 thousand years ago: …", and the kernel's cells
    turn pale only during their ice age;
  - the Solar System's Earth from base-sol through the kernel: 31 ice ages dated, all inside a 500 thousand year
    window; every frame has a sea and its line; the first frame is an ocean world (0% land) whose climate differs from
    today's (288 K against 283 K mean over the cells), since each frame's climate reads its own land.
- Finding for proven core: a catalogued Earth's climate is a fixed value, so every frame's mean surface is 288 K and
  the young Sun's dimming is not in its frames. The kernel says so in pending, and the temperature layer shows it.
- `npm run ui:check` clean; `npm run ui:test` 520 of 520; `render.e2e` chromium passes (its adopted sample Earth has
  no orbit track, so it takes the "no dated ice ages" branch and says so).
- In the built shell, the Solar System's Earth from the start sheet (Orbit detail): one ice age on the strip, 140
  thousand years ago in a 200 thousand year window; the card gives its line and the kernel's Milankovitch why; no
  page overflow on desk or phone. Screens `R6-screens/R6c-ice-desk.png` and `R6c-ice-phone.png` (project files).

## Limits and next
- Ice ages show as dated onsets; how far each one's ice spreads waits on `cells` from the kernel (R6c part 2), and its
  thickness on an ice-age ice field the kernel does not have yet.
- Sea ice, oceans changing volume and worn elevation through time are still the kernel's pending lines, shown on the
  layers they bear on.
- No fudge added: nothing here is computed, only read and coloured. Colour ramps are labels (legend in each layer).
