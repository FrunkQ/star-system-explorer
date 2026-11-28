# FL1 and FL0: the settle schedule, and the list of stand-ins

Status: built 28 Nov 2026, thread "Build the shared flourish system" (Opus). Design: `design/SSE4-visual-flourish-design.md`,
accepted by Alex 28 Nov 13:23 UTC (section 10A); files per section 8A. Neither package touches the kernel, a pack or a
golden, and nothing on screen changes yet.

## FL1: the schedule (`packages/ui-contract/src/settle.ts`)

Pure and clock-free, like `RestClock`: callers pass wall times (ms) and the picture clock's rate.

- **The record** a dear layer declares (`DearLayer`): `id`, `moves_with` (clock, camera, place), `own_time_s`,
  `cycle_s`, `stand_in` (the kinds it offers, in order). The builder and the kernel numbers stay with the view, so the
  record is data the declared list can mirror.
- **The four states** (`Settle.state`): Rest when nothing it moves with has changed for 500 ms and the clock is not
  running; Too fast when one picture covers more than `own_time_s` or the world turns in under two pictures
  (`outrunsPictures`, the same test as CLK's `tooFast`); Live when rate × build time < own time (for the camera or
  place, when a build fits in one picture's time); otherwise Carried on.
- **Asks** (`Settle.ask`): at rest one build for the exact date and view, then none; while moving, a lattice date, at
  most one a second (`REBUILD_CAP_MS`), one build at a time, only when the frame has spare time, never while too
  fast, never mid-drag. `properKey` is the one key a proper picture is kept and found by.
- **The lattice** (`latticeStep`, `latticeDate`): anchored at the campaign's epoch, `own_time_s` apart, doubled on a
  slow device until a step outlasts a build and the cap (doubling is exact, so two devices on one spacing agree).
- **Sizes** (`sizeFor`, `SIZE_LADDER` 64 to 1024): built at the size shown; `bySize` orders asks biggest first.
- **The drag rule** (`dragPicture`): carry the nearest kept picture within a step (either way), hold it further, else
  the Too fast picture. `Settle.dragging` says when the clock is being dragged or stepped while stopped.
- **What is drawn** (`drawFor`): the proper picture at rest; Exposure (else Held) when too fast; otherwise the layer's
  first offered kind. Reduced motion turns Carried and Hint off; no GPU leaves Held only.
- **The settle** (`settleWeight`): one opacity fade over `SETTLE_FADE_MS` = 240 ms.

Exported through `packages/ui-contract/src/index.ts` (one line). `REST_AFTER_MS` in `still.ts` and `SETTLE_REST_MS`
here are both 500 ms; FL6 (sharpen at rest) makes `still.ts` take it from here, as 8A says.

### Tests (`packages/ui-contract/test/settle.test.ts`, 12, in `npm run ui:test`)

Each of the design's red-first tests was seen red with its fix removed (28 Nov, in the container):

| Test | Fix removed | Went red |
|---|---|---|
| at rest the state is Rest whatever came before (absolute) | Rest only when the build is cheap | yes |
| the same date by play, a drag and a jump asks the same picture | rest ask on the lattice date, not the exact one | yes |
| the same, mid-drag nothing is rebuilt | the drag guard in `ask` | yes |
| build time zero: never a stand-in for want of the device | the cap folded into the Live test | yes |
| no more than one rebuild a second | the cap line | yes (only through the zoom layer, see below) |
| a stopped clock asks for exactly one build, then none | the "already made" check | yes |

Also held: the hand check of section 5 for Jupiter (desktop 0.1 s, phone 2 s: Live to a day a second, the phone
Carried on at 3 days a second, Too fast at 7), the ladder, the lattice, the drag rule, `drawFor` and the fade.

### Choices the design left open, and one reading

- **Rule 5's test ("with a build time of zero the state is never anything but Live or Rest").** Taken literally it
  clashes with Alex's yes to question 5 (textures blur as a long exposure at speed), since Too fast is a sampling
  limit that no device speed lifts. Built and tested as: with a free build there is never a stand-in for want of the
  device (never Carried on); Too fast still applies past the sampling limit, and only there.
- **The cap and the lattice.** For a clock layer the coarser lattice already keeps rebuilds near one a second, so
  removing the cap alone did not turn the clock half of that test red. The cap's own guard is a zoom layer whose place
  moves every frame with free builds: removed, that half goes red.
- **The governor's "spare time"** is a plain `spare` flag on `ask`; the runtime (FL2, FL3) sets it from the governor.
- **A drag** is a clock movement with the clock stopped; nothing is rebuilt until release, then Rest.
- **No GPU** (`drawFor` with `gpu: false`): Held while moving; the proper picture is still built at rest on the
  processor, as stirred clouds 11A says.

## FL0: the list (`packages/ui-contract/src/stand-ins.json`)

Data only. The eight uses of section 2 that are stand-ins, each with its kind, what moves it, its kernel numbers, its
proper picture, its register row (file and the row's opening words), what says so today, and the package that moves it
onto the schedule. Every register row named there was checked to exist (28 Nov); FL4 turns that into a test.

- **New register rows** (in `docs/fudges/0360-interface-and-player-views/`): FL-F1 sharpen at rest's moving picture,
  FL-F2 the governor, FL-F3 the grand zoom's wait.
- **Rows that already existed:** orbits run on, giants carried by their winds, star cells' blur (all in 0290), the
  zoom's seam (GZ-F1), and a world's finer tiles (`composeFaces`, 0260/0460). The design's section 2 counted that last
  one as having none; it had one.
- **Not a stand-in:** a fast moon (`tooFast`) is drawn where it truly is at every picture, so what is seen is always
  the proper picture. Listed under `not_stand_ins` with its row; its long exposure (CLK3) would join the list.
- **Coming:** the stirred still (`giant.cloud`, PR 739) has no register row yet. FL3 declares it and writes its row when
  it puts it on the schedule.
- **The governor** is driven by frame time, not movement, so its `moves_with` is empty and it says why (`driven_by`).

## Next

- FL2 (the kit, needs AC-ProArt), FL4 (said and checked) and FL7's pure part (warm-up queue, in `settle.ts`) once
  this is on main.
