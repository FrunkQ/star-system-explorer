# GZ1 the travel contract (grand zoom, stage C), 26 Nov 2026

Thread "Build travel between places" (Opus). Design: `design/SSE4-grand-zoom-design.md` sections 5.1 to 5.6 and
5.10, with Alex's rulings in 14A and the three desks note's 13A. No view code, no kernel change, no golden moved.

## What it is

Two files in `packages/ui-contract/src`, tested in `test/journey.test.ts` (16 tests).

**`approach.ts`, the data.**
- `RUNGS`: the seven rungs, `web, cluster, galaxy, stars, system, world, site`, with `RUNG_INFO` (words, the
  picture's width at the rung's stop, the place line scale that draws it today) and `rungOfPlace`, `sharedRung`.
- `sse.approach/1` as `Approach`: `from` (a rung or `"here"`), `stops`, `go_on`, `hud`, `words`, `style`, `pace`,
  `ends`, every key optional. `fullApproach` fills the defaults; `checkApproach` and `approachFrom` read one from a
  save, a postcard, a preset or a pack, the same way `pose.ts` reads a camera.
- Presets: `SHORT_FORM` (the default when nothing is said: from the rung above, no stops, no words, HUD and style as
  the viewer has them, pace `"short"`), `ZOOM_IN_FROM` (question 7: True, stop at each, HUD and captions at stops, go
  on by a press, stately) and `oneSmoothZoom` (stops, HUD and words off together).
- The link form: `withApproach(hash, a)` and `splitApproach(hash)`. The way in rides after the place's own settings
  (`#chart/world/…~screen?site=1,2&from=web&stops=each`, or `#chart/galaxy?from=web`). The shell must call
  `splitApproach` before `parsePlaceHash`; `place.ts` is not changed.

**`journey.ts`, the plan and the camera.**
- `journeyPlan(ask)` lays out a journey: the rungs passed (`path`), timed legs, stops, seams (each with its fade
  window), the total, and notes for the GM's preview. It fails only for a held rung or a "from here" with no here.
- `journeyAt(plan, t)` is the journey at time t: the picture's width, the views to draw (one, or two in a seam, the
  incoming under the outgoing), each view's camera (target, width, eye direction, up, opacity), the stop if resting at
  one, and whether the HUD, the words, the target ring and the width line are up.
- `journeyHold(plan, t, released)` and `stopBefore(plan, t)` are Go on and Back up for the player.
- `JourneyPort` is the port each view will answer (aim, ready, centre). Declared only: GZ3 adds it to `ViewModule`.
- `poseWidthM` and `journeyEnd` turn a bookmark's pose into the journey's end, so a journey ends exactly on it.

## How to build on it (GZ3, GZ4, GZ8, TM7)

- **The player** (GZ3) keeps a journey time and a count of Go on presses, asks `journeyHold` how far it may go, and
  each frame hands `journeyAt(plan, t).views` to the ports. Before `path[i].t_in` it asks that rung's port `ready`;
  if not, it holds the time just before the seam (design 5.4). The plan is plain data: it can be sent to a player's
  screen, kept, or replayed by the trailer rig at any frame rate.
- **The short form** (GZ4) is `journeyPlan({ to, here, shared, approach: { from: "here" } })`: 0.6 s for each change
  of rung, no stops. `sharedRung(a, b)` gives `shared` for two places.
- **Directions** are each rung's own (unit vectors from the target to the eye in that rung's frame, as `Pose.eye_dir`).
  Pass `down[rung] = { target, width_m, arrive, good, leave, up }`. `leave` should be the next rung's `arrive` seen in
  this rung's frame, which is what `place_chain` (GZ2) will give. Left out, all three are the rung's tilted view, so
  the line of sight jumps at a seam by however much the two frames differ: GZ3 should pass real ones.
- **Widths**: `RUNG_INFO.stop_m` is rough (design 5.2's table). Pass `width_m` for the real target (a big system, a
  small moon); the plan narrows any stop that is wider than the one above it and says so in a note.
- **The Run desk's stepper** (TM7) can draw all seven steps from `RUNGS` and `RUNG_WORDS`, Local stars included as a
  pencilled step that cannot be pressed while `RUNG_INFO.stars.held` is set.

## Rulings it carries

- **Local stars is declared and held** (the conserved budget chain, accepted 26 Nov, its section 13: every star nobody
  has looked at changes once at its stage 3). It is in `RUNGS` with `held` set. No journey stops or arrives there; a
  journey through it passes it over with a note; one asked to start there starts at the galaxy, or at the rung below
  when the role knows no higher. Nothing here names or fixes a generated star. When GZ7 draws the rung, deleting
  `held` (or passing `drawn`) makes it an ordinary rung, which a test already covers.
- **The place line is unchanged.** The design said "the two new rungs on `PLACE_SCALES`"; adding them there would have
  drawn a Local stars step on every screen with nothing behind it and changed the place links. The rungs live in
  `RUNGS` instead, beside it; GZ4 or TM7 decides when the drawn line gains them.
- A journey from here rises to the rung both places share, rests there, and comes down (design 5.2). Stops apply only
  on the way down; the turn is a rest, and a stop only if the stops say so.
- A role travels only from as high as it knows (5.8): `ceiling` caps the start, with a note the GM's preview can show.
- Reduced motion (5.10): nothing zooms. Each rung is held two seconds (or as a stop says) and changes by the plain
  fade (`REDUCED_FADE_MS`); the short form is one fade.
- The width line shows while moving unless the way in says HUD never (question 11).
- `pace: { total_s }` is the time in motion; timed waits at stops come on top.

## Not done here, and why

- No view code, no player, no fudge register lines: GZ-F1 to F7 belong to the packages that draw them (GZ3 on).
- `style` is carried but not acted on; the player picks the view style.
- `ends: "landing"` only sets `plan.landing`; GZ3 replaces the last leg with the kernel's flown descent.
