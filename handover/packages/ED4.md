# ED4: the ghost for Add, Paste and Duplicate (editing design section 5)

## Part 1 (25 Nov): Add with the ghost first

- `Workspace.adding` (`ui-contract/src/view.ts`, never saved): `{host, label, a_au}`. The shell's `openAdd` sets it, with
  `a_au` null, when adding round the system's primary star on the flat chart (Chart or Wireframe style, system screen).
  Round a planet, or in 3D and True, Add here opens straight away as before.
- `SystemChart.svelte`: while `adding` names the chart's centre, a ghost (a dashed dot and its dashed accent orbit)
  follows the pointer with the distance and the host ("1 AU round Sun"), catching round figures and resonances with the
  worlds already there, as the ED3 drag does (Alt runs free). A press anywhere on the chart (captured, so a body under it
  is not picked) sets `a_au`; a touch press works the same (no hover needed). Esc clears `adding` and writes nothing.
- `Stage.svelte` shows the note "Press on the chart where it should go round Sun; … Esc puts it away" with "Type a
  distance" (opens Add here at once). `App.svelte` opens Add here when `a_au` is set; `AddHere` takes `at_au` as its
  first orbit. Closing the dialog clears `adding`.
- Tests: `tests/ui/editing.e2e.mjs` (desk: the ghost and its readout near Earth, Esc writes nothing, a press opens Add
  here near 1 AU); `tests/ui/walk.e2e.mjs` "add a world" budget raised by one press (desktop 4, phone 5) for the press
  on the chart. Screens: project files `sse4/handover-screens/ED4-screens/`.

## Part 2a (25 Nov): what forms here, on the ghost

- While the ghost rests (400 ms) on an orbit round a star, one formation run round a star of that mass says what forms
  there, under the distance: "forms a world of 0.42 Earth masses here" or "no world forms here in one run"
  (`data-add-forms`, SystemChart). It is `formsHere` with one try, the first of the runs Add here then offers, so the
  two agree. Each orbit (three significant figures) is asked once per chart; nothing is written.
- e2e: editing (desk), resting near Earth's orbit. Screen: project files
  `sse4/handover-screens/ED4-screens/add-ghost-forms-desk.png`.

## Part 2 (rest, queued)

- The live panel beside the ghost (Kinds, Something else) in place of the dialog, and the stability verdict on the
  ghost (ST1).
- Round a planet in its close-up (the chart framed on it), and a companion star (ED6 part 2).
- Paste and Duplicate (Ctrl+D) as the ghost; a clip file dropped on the chart; drop on a parent; paste onto a body.
- The family tree before and after (R1) and R3's words for which body it would go round.
