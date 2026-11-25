# ED2: the sweep behind the strip

Design: `handover/design/SSE4-editing-design.md` section 3, point 5 (the strip knows where things change). Kernel: K1
`sweep` (PR 546), a job: `start_job {kind: "sweep", system, address, field, from, to, count, log, results}` then
`step_job` until `job_done`; per step `{value, results, class, verdict}` and `tipping_points [{what, from, to,
between}]`, narrowed by halving. It runs on a scratch branch the kernel throws away.

## Part 1 (25 Nov): marks on the Orbit and Mass strips

- `Session.sweep(ask, onprogress)` (ui-contract `session.ts`, optional on `ViewSession`): starts the job and steps it
  two resolves at a time, so other asks get in between; GM only.
- `ui-contract/src/sweep.ts`:
  - `sweepAsk(handle, at, lo_si, hi_si, count)` runs the handle's field over the strip's range in the kernel's units,
    log where the strip is, asking for the yes-or-no results the handle can reach (`reach` in handles.json).
  - `sweepMarks(handle, swept)` returns the strip's marks in SI: each verdict tipping point ("does not last from
    here", "worth watching from here", "lasts from here"), each class change, and each yes-or-no result flipping
    between two steps ("turns freely from here", "a field of its own from here"). Words say what lies to the right.
- Handles.svelte: when a world's Orbit or Mass strip opens on the Build desk (main line, GM), the sweep runs over the
  strip's range (12 steps) and the marks go on the strip (`marks` on Strip, `swept` on Box). "Running orbit across the
  strip (4 of 16)…" shows meanwhile (`data-sweeping`); a failure says why (`data-sweep-why`). It runs again only
  when the value moves, and never two at once for the same value: the kernel names its scratch branch after the
  body and field, so a second sweep started before the first ended shared, and lost, its branch.
- Strip.svelte: marks' words go in rows under the line when they would print over each other.
- Checked against the kernel (`test/sweep.test.ts`): across 0.25 to 8 AU, Mars stops lasting from about 0.35 AU,
  is worth watching from about 0.51 AU, and turns freely from about 0.55 AU; the state hash does not move.
- e2e: editing (desk), the Orbit box waits for "turns freely from here" and checks the hash. Screen: project files
  `sse4/handover-screens/ED2-screens/orbit-sweep-desk.png`.

## Part 2a (25 Nov): classes on the Mass strip's marks, the sweep at the campaign's now

- The Mass strip's sweep passes the class atlas (`packs/base-atlas`, the stack's or the one the shell ships;
  `atlasPacks`) and the `physical` scheme, so the kernel labels each step and the strip marks where the class
  changes, in the atlas's own words (`classNamesIn`): "rocky from here", "no longer rocky from here" when no class
  fits past it. Only a world with its makeup set gets these: the class regions read every part of the makeup (one
  missing and no class fits), which is why the Makeup box (ED9) writes all five. The makeup is part of the sweep's key,
  so a change of makeup sweeps again.
- The sweep's edits and reads are at the campaign's now (`at_s`, sent as a whole number of seconds). Before this they
  were at time zero, so a later edit on the main line hid the swept value. Example: Undo sends a mass back at now, and
  every step then read the same mass and marked nothing. The Makeup box's `expects` call takes the same `at_s`.
- A sweep that throws lets the next one run, and says why.
- Checked against the kernel (`test/sweep.test.ts`): Mars with Earth-like makeup is rocky across its strip and no
  longer rocky past about 12 Earth masses; the state hash does not move. e2e (editing, desk and phone): Mars set like
  Venus, the Mass strip shows a rocky mark. Screen: project files `sse4/handover-screens/ED2-screens/mass-class-marks-desk.png`.

## Next

- Pencil figures along the strip while held, read from the sweep's steps instead of a pause's resolve (ED3 part 2b's
  year and quick look come from the same steps).
- "What followed" across the range: the results row for each step, drawn as a small line under the strip.
- Swap Reach it's bisection (ED5) and Could be's per-handle search (ED10) for the sweep's steps.
- A sweep of a makeup part (ED9 part 2's tipping points on the bar's joins).
