# CLK1: everything that moves on screen moves on the campaign's clock

Status: done 26 Nov 2026, thread "Run all animation off the clock". Asked by Alex (26 Nov, 19:19 UTC, looking at the
star gallery and the belts in the system view): "I feel we should be running those at the speed of the clock... so
advancing time fast animates a sun, planets whizz round and asteroids/belts circle at their correct speeds."

## What drove each motion before (the inventory)

| Where | What moved | Driven by before | Now |
|---|---|---|---|
| System True (`System3D.svelte`) | the star's spin, spots, cells, flares, ejections | its own clock, six hours a second from page time (`STAR_SECONDS_PER_SECOND`), whatever the ruler did | the campaign's clock |
| System True | belt members (Kepler shear, inner edge laps the outer) | their own clock, thirty days a second (`BELT_SECONDS_PER_SECOND`) | the campaign's clock |
| System 3D and True | worlds round their orbits | the kernel's `orbits_at` phase, ten answers a second while playing (so they jumped) | the same phases, run on smoothly between answers at each body's own period |
| Size comparison globes (`Lineup3D.svelte`, the Body globe path) | spin, a giant's bands and storms, a star's face | page time: Same pace (40 s a turn) or "True pace" (the quickest 40 s a turn, rest in proportion), star six hours a second | Same pace kept as a labelled aid; "True pace" became **On the clock** (real days on the campaign's clock); the star's face on the clock |
| Land (`Surface.svelte`) | sea waves | page time (real time, but never stopping and never speeding up) | the campaign's clock (wrapped to a day for the GPU's floats) |
| Land | the sun's hour | already the world's clock at the time in view | unchanged |
| Star gallery (PR 641, not on main yet) | cells, spin, flares | its own clock per star, said in its caption | not touched here (see below) |
| Interface motion: camera glides, the ruler's glide, Cosmic web's slow yaw, Galaxies nearby and Formation replays, the players' table drift (`PlayerChart` animate, `PlayerOrbit` ship) | presentation | page time | unchanged: they are the interface or a replay of their own timeline, not the world; the players' drift is in the fudge register already |

## How it works

- **One clock** (`packages/ui-contract/src/picture-clock.ts`): `PictureClock { epoch_yr, at_s, wall_ms, rate }` on the
  workspace (`ws.clock`). Time is counted in **seconds since the campaign's epoch**, not years: a year near a star's
  age only resolves about 30 s in a 64-bit float, which would make real-time motion stutter. `rate` is clock seconds
  per real second; 0 is stopped.
- **Who sets it.** The ruler's Play calls `onrate` with its step a second (and 0 on Pause); the workspace's `setRate`
  re-anchors without a jump. An App effect makes the clock follow `now_yr` however it moves (steps, drags, bookmarks,
  saves). While running, a ruler tick that agrees with the clock keeps the anchor (`followTime`), so the picture is
  not knocked about by the ruler's ten-a-second rounding. The Run desk's clock (`CampaignClock.svelte`, TM3a) sets
  it the same way through its `onrate` (Run ▸ at its rate, 0 when stopped); its tick also now works the date out
  from where Run started, as the ruler does (adding a quarter second at a time to a year near a star's age was lost
  to rounding, so real time never moved on).
- **The ruler's play was also fixed**: it added each frame's step to a year near a star's age, losing up to half of
  it to rounding at an hour a second. It now works the time out afresh each frame from where play started.
- **Rates come from the physics**, never the clock: the star's `rotation_period_s`, cell lifetimes and flare times from
  the kernel's activity; belts at √(GM/a³); worlds at the period from `orbits_at` (`periodsFrom`, carried on the scene's
  `orbit.period_s` and `orbit.at_s`); globes at their own day; waves at their own period.
- **Stopped means still.** With the clock stopped nothing in a world moves, and the system view's note says so and how
  to start it. The size comparison's Same pace is the one view that turns while stopped, as an explicit, labelled aid
  ("an aid for comparing, not their real speeds"); press On the clock for the real thing.
- **Faster than the pictures** (`tooFast`, under two pictures a cycle): drawn where it truly is at each picture (never
  slowed to look smooth), and the note names it: "Phobos goes round faster than the screen can show, so it is drawn
  where it truly is at each picture and may seem to jump or run backwards." The star's cells instead blur toward their
  average as a picture stands for more of their lives (`granuleBlur`, already there), like a long exposure. Places
  are drawn up to thirty pictures a second; the star's face (drawn on the CPU) ten.
- Fudge register rows: True star clock (rewritten), granuleBlur (reason), Belts in True (clock), Turning globes
  (paces), and three new: Orbits run on between the kernel's answers, Motion faster than the pictures, The sea's
  clock wraps each day.

## The star gallery (PR 641)

The gallery shows stars outside any campaign and outside time (a reference page, each star at a chosen age), so it has
no campaign clock to follow. Its own per-star clock is said in its caption, which is the "explicit, labelled choice"
the coordinator asked for. Recommendation for Alex: keep it, and once 641 is on main let its caption show the rate it
runs at in the same words (`rateWords`). Not changed here, to stay out of an open PR.

## Fit with the time pipeline (track TP, PR 651)

The time scales design (`design/SSE4-time-scales-design.md`, section 9) makes this clock the drawing code's view of
its one session clock (`view_s` and `rate`). Kept to its rules: the clock holds no absolute time and no `_yr` field,
only seconds since the campaign's epoch and the anchor's wall time. Its one remaining weakness is upstream: the time in
view arrives as `now_yr`, a double that resolves about 30 s near a star's age, so each re-anchor (a step, a drag) can
land up to 30 s off; between anchors it is exact. TP1 (dates as integers in the kernel and bigint seconds on screen)
removes that; `followTime` then takes seconds instead of years.

## Not done (follow-ups)

- Shadows and eclipses (EC1, `orbit.r_m`) still move at the kernel's ten answers a second; the worlds' drawn phases
  run on smoothly between them, so at high rates a shadow can lag its world by a tenth of a second of play.

- Flares are read for the campaign's first year (`readLiveStar`'s window): later dates show a star with no flares.
  The window should be read again as the clock leaves it.
- A time exposure for motions faster than the pictures (each fast body drawn as the arc it sweeps in one picture).
- The flat chart and the players' screens still move at the kernel's ten answers a second.
- The ruler has no real-time step (its smallest is an hour a second); the Run desk's clock has real time. At real time the date itself still moves in about 30 s steps (a year near a star's age); the picture runs smoothly between them. TP1 (dates as integers) fixes the date.
- The star's time is its age plus clock seconds in one float (about 30 s resolution near 4.6 Gyr): fine for spin and
  flares, but at real time the Sun's ten-minute cells step in 30 s jumps. Passing the cells the clock seconds alone
  would fix it (star-now.ts, which PR 641 also edits).

## Checks

- `packages/ui-contract/test/picture-clock.test.ts`: stopped is still; real time resolves half a second near a
  star's age; changing the rate never jumps; ticks keep the anchor and a jump moves it; orbits run on at their mean
  motion; a motion faster than the pictures is named; the rate in words.
- `tests/ui/system3d.e2e.mjs`: the star stands still while the clock is stopped and the note says so; Play turns it
  and the note says "a day a second".
- `tests/ui/compare.e2e.mjs`: the pace button is On the clock, and with the clock stopped it says the globes stand still.
- `npm run ui:check` clean; `npm run ui:test` 1,090 of 1,090.
- Browser runs on Chromium: compare, time, walk, shell, globe and system3d pass. system3d had failed at the close-up
  clicks on main too (line 141): in True the note runs to more lines than the gap under the close-up bar and covered
  "‹ Kestrel". The bar now sits over the note (`z-index`), so the way out of a close-up is never covered.
- Looked at in the browser: stopped, Vault's pin holds; at a day a second it moves a little every tenth of a second;
  in True at a month a second the belts turn with the inner edge ahead and the note reads "Everything moves on the
  campaign's clock, 30 days a second." Screens in project files `sse4/handover-screens/CLK1-screens/`.
