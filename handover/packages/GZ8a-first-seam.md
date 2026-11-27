# GZ8 part 1 (with the player of GZ3): the first seam, cosmic web to galaxies nearby, 27 Nov 2026

Thread "Build the crash zoom between places" (Opus). Design: `design/SSE4-grand-zoom-design.md` sections 5.3 to 5.7;
the contract it plays is GZ1 (`packages/GZ1-travel-contract.md`). Alex's yes, 27 Nov 11:42 UTC, with his principle:
every level on the way down looks equally filled in, never only the destination. No kernel change, no golden moved.

## What it is

- **The player**, `shell/src/journey/Journey.svelte`: plays a `journeyPlan` over the chart room. One square picture
  aimed at the target; the journey's time is camera travel (real seconds), never the campaign's clock. The place line
  across the top is the progress bar (`journeySteps`: passed in ink, current underlined in the accent, coming
  pencilled, a held rung between in italic with its reason, the rest off). Zoom in / Go on (Space or Enter), Back up,
  Leave (Escape). At the start stop a caption card; while moving the width line ("about 16 million light years across
  · 1.5 × 10²³ m", through the figures door). If a level below is not read when its seam comes, the journey holds just
  before the seam with "Reading galaxies nearby…" (`readyHold`), never half-faded.
- **Where it is offered**: More > "Zoom in from the cosmic web", and a link whose way in starts at the web
  (`#chart/cluster?from=web`, GZ1's link form). Nothing else on the chart room changed; no press budget moved.
- **The ports** (design 5.3), as props on each level's own drawing component, so the zoom draws the same data the
  level's own screen draws:
  - `views/foam-map/Web3D.svelte` `aim` (eye and target in the cube's units) and `inner` (see below). On a journey
    it does not turn by itself, takes no drag and draws only the picture.
  - `views/cluster-map/Nearby.svelte` `aim` (reach and turn) and `onread` (the port's ready). On a journey: only the
    svg, no reach buttons or card, every catalogued galaxy out to the square's corners, no ground of its own.
  - `shell/src/journey/JourneyWeb.svelte`: reads home (`home_galaxy`), then the web cube and its galaxies centred on
    home (200 Mpc, 48 a side, the web screen's light floor but never above half home's mass), once per session.
- **Pure parts**, `ui-contract/src/journey-play.ts` (tests `test/journey-play.test.ts`): `readyHold`, `journeySteps`,
  `widthLine`, `webEye` (eye distance from width and the cube's field of view, `WEB_CUBE_FOV`), `nearbyAim` (reach
  and turn from width and line of sight, the inverse of `projectNearby`'s turn; a test checks the two ports put the
  target on the same pixel with the same width and line of sight), `homeInCube`.

## Home in the web cube

The cube's galaxies are catalogued haloes moved by the cube's own Zel'dovich step (`web_light::cube_galaxies` calls
`web.displaced` on each halo's catalogue place); `home_galaxy` gives home's catalogue place, before the step. So
`homeInCube` takes the cube galaxy with home's mass now (to a millionth) nearest home's own place: the same halo,
found through the step. No stand-in; GZ15 (one place per halo) makes it a plain look-up.

## The seam, and Alex's "filled in" rule

The two levels' frames share axes (the web's axes are the galactic frame's, fudge F37), so the journey's line of
sight means the same in both and nothing turns at the seam. The first run cross-faded the whole web away: at the seam
the web's picture is a 200 Mpc deep column of galaxies, the galaxies nearby only the catalogue's 12 Mpc, so the lower
level looked emptier. Counted at the seam on the default universe: 57 cube galaxies lie within 12 Mpc of home against
70 in the catalogue, so the data agree; the difference was depth. Now from the seam on the web is drawn at the
bottom at full light past the catalogue's reach, and only the light within the reach fades (Web3D `inner`) while the
catalogue's galaxies fade in over it. One GPU canvas for the web throughout (two WebGPU canvases drew nothing in
headless Chromium).

Fudge GZ-F1 is registered in `docs/fudges.md` with what does not line up yet: galaxies near home jump as they fade
(catalogue before the step, cube after it, up to about 7 Mpc, GZ0), the cube is perspective and the galaxies nearby
flat, and the two draw a galaxy's brightness by different rules.

## After the first real-GPU capture (AC-ProArt, 27 Nov)

The web drew on the GPU (WebGPU, `data-tier` gpu). Three fixes from the frames: the seam read as a hard cut, so the
fade now spans 0.22 factors of ten either side of the seam's width (`SEAM_FADE_DECADES`, the recovered mock-up's
blend, about 0.9 s at stately) and never less than a third of a second; the square overflowed the window by its
room's padding; the width line's power of ten rounded 4.8 × 10²³ up to 10²⁴ and now gives the leading figure. The
width line also stays up at the end. Frames: Alex's PC, `C:\Development\sse4-project-files\sse4\handover-screens\GZ8a-screens\`.
Found on the way: `scripts/serve.mjs` answers 404 for "/" on Windows (the path ends in a backslash).

## Checks

- `node --test packages/ui-contract/test/journey-play.test.ts` (5 tests).
- `tests/ui/journey.e2e.mjs`, in CI and `ui:e2e`: the link opens the player, it holds until the web is read, Space
  travels, the width line, the seam, the end on the galaxies nearby with the web behind, Back up, Escape, More.
- Headless Chromium draws the web only on WebGL2 (`--use-gl=swiftshader` without WebGPU); real-GPU looks are for
  Alex's PC.

## Next (this thread)

1. Captured sequence on the PC for Alex.
2. One exposure at the seam (GZ-F2) so a galaxy is the same brightness on both sides.
3. Galaxies nearby to the galaxy (Galaxy 3D port), then the system, world and site; the local stars rung when GZ7
   lands. Needs from other threads, in order: GZ15 or GZ2 so galaxies sit in one place at both levels; GZ13 so a
   system has a star slot to aim the galaxy's camera at; GZ7 for the local stars.
4. Captions from each level's facts and stops in full (GZ9); the short form on the place line (GZ4).
