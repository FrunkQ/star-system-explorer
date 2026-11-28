# FL5: the zoom travels on a coarse picture and settles at a stop, with FL7's journey use, 28 Nov 2026

Thread "Build the crash zoom between places" (Opus). Design: `design/SSE4-visual-flourish-design.md` sections 6, 6A
and 8A, with Alex's answers of 28 Nov (questions 8 to 10, "yes if practical"): the journey waits until everything is
ready by default; Go early travels on coarse pictures; filming always waits; kept in memory for the visit only, never
a file; something worth watching while it prepares. Presentation only: no kernel change, no golden moved.

## What it adds

- **The port's answer is proper, coarse or not yet** (`journey-play.ts` `LevelAnswer`). A coarse answer says how many
  things it shows and how many the proper picture shows of the same; by Alex's filled-in rule (27 Nov) it is
  travelled on only when it shows every one of them (`coarseFits`). One with fewer, or one that cannot say, holds.
- **Where the journey may go** (`readyHold(plan, t, ready, how)`, `levelPasses`): proper always; coarse only when the
  viewer went early (Go pressed at the start before every level was ready), never for film.
- **What a level shows** (`levelShows`): a level travelled into on its coarse picture keeps it until the journey rests
  (a stop or the end) and settles onto the proper one there, so nothing changes under the viewer mid-flight; film
  shows proper or holds, every frame.
- **The warm-up of the way down** (`journeyWarm`, `journeyReadNext` on FL7's `warmPlan`): the levels read one at a
  time in the order the journey needs them, the one on screen first. Before this the galaxies nearby began reading at
  mount beside the web's three reads; the kernel answers one read at a time, so it could slot its 9 s between them and
  delay the start. Now it starts once the web is in. A viewer who opened a journey asked for it, so a hot device or
  data saver still reads the next level (FL7's hold-backs).
- **The time told** (`journeyWaitLine`, FL7's `BuildTimes` and `readyLine`): "Getting the way down ready: about 15
  seconds, 0 of 2 levels.", from the design's estimates the first time (`JOURNEY_BUILD_MS`: web 6 s, nearby 8.7 s, from
  GZ0) and the device's own times after (kept in a module-level `BuildTimes`, memory only). A level answered at once
  from memory is not a build and is not recorded.
- **Something worth watching** (`startTurn`, `turnedAboutUp`): the web turns slowly about the page's up while the
  journey rests at its start (0.12 rad/s, the web screen's own rate), not under reduced motion nor for film, and eases
  back to the journey's line of sight within half a second of Go. The place line inks in each level as it becomes
  ready (`data-ready`).
- **The player** (`Journey.svelte`): Go reads "Getting ready…" (plain, not the accent) with the wait line in the start's
  caption, and turns to the accent "Zoom in" when every level is ready; a press before that goes early. `film` (prop,
  or `film=1` in the link) never goes early. Layers carry `data-picture` (proper, coarse or hold) for the checks.

## What is not there yet, and why

- **No level declares a coarse picture yet.** The web is the start level and the wait is its first stop (design 6A),
  and its galaxies are needed to find home in the cube. The galaxies nearby have no honest coarse picture: in Node,
  `sky_from_here` without satellite paths is 9.8 s against 9.2 s with them (694 galaxies both), and halo cells are
  about 250 ms each, so no cheaper read shows the same galaxies; the web cube's own galaxies near home are fewer (57
  against 70 within 12 Mpc) and moved up to 7 Mpc. An honest one needs a kernel option (the neighbourhood's galaxies as
  places, types and masses only, without the light work); told to the coordinator, who routed it 28 Nov to the home galaxy thread as a new kernel op `galaxies_near` (places, drift, type, masses, address, out to a radius, on the `galaxies_seen` path), landing after GZ15 (which gives every halo one place). When it is on main, the galaxies nearby answer coarse from it with `of` the same list's count. Until then a press before the
  galaxies nearby are read travels the web and holds before the seam with "Reading galaxies nearby…" and the time
  left, as the design says. So no stand-in is drawn and no register row is added.
- **The small panes beside the target** (6A, each level's coarse picture as it is read): they wait on coarse pictures.
- **Shedding under memory pressure**: the levels' reads sit in each view's own memory cache (`JourneyWeb`'s per-session
  map, `Nearby`'s `kept` by state hash); there is no memory-pressure signal on main to shed them by yet.
- **The seam on the kit's Fade** (design 6, second row): FL2's kit is not on main yet; `SEAM_FADE_DECADES` stays.

## Tests

`ui-contract/test/journey-play.test.ts`, six new: gone early it travels on a fitting coarse picture and settles at rest;
a coarse picture with fewer galaxies, an unknown count, or none, holds; film never travels on or shows coarse (the
start too); reads in the order needed, the screen first, the hot device's next level; the wait line's time from the
device's own times; the start turn back within half a second. Red first: `coarseFits` made to accept any coarse answer
and film's guard removed, the second and third tests went red.

`tests/ui/journey.e2e.mjs`: only the web reads at first; Go says Getting ready with the wait line; the cluster reads
after the web; the place line inks in; Zoom in once both are ready; opened again in the visit the web is ready at once and the galaxies nearby answer from memory (after whatever the chart room behind asked of the kernel on closing); on a
fresh visit Go pressed early travels and holds before the seam until the galaxies nearby are read; with `film=1` it
never goes early; no frame carries a coarse picture.

Wait line seen in the cloud: the first-time estimate (8.7 s, a desktop's) counts down to "about 1 second" while the
cloud's slower read runs on (about 20 s there); from the second journey it tells this device's own time.
Screens: project files `sse4/handover-screens/FL5-screens/` (0-getting-ready: the web in, Go reading "Getting
ready…", the wait line in the caption, the place line inking in).
