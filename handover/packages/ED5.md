# ED5: Reach it and Pin it (editing design section 8)

## Part 1 (25 Nov): Reach it on the Orbit box

- `packages/ui-contract/src/reach.ts`: `reachIt(session, {system, address, field, at_yr, lo, hi, log, target, read, within, tries})`.
  Opens a scratch branch (`openBranch`), and bisects the input's range: each try is one `branch_set` of the field and
  one `resolved_system {branch}` (kernel BC1), reading the result with `read` (`surfaceKelvin(address)` reads
  `t_surface_now_k` from the surfaces). The branch is discarded in a `finally`, so the campaign's state hash does not move.
  Returns `{ok, value, got, tries}` or `{ok: false, why, closest}` (how close it can come: the nearer end of the range).
  It assumes the result moves one way with the input (true for warmth against distance); the kernel's sweep (K1) will
  replace the search, same answer.
- `Box.svelte`: on the Orbit box, main line, GM editing, when the kernel gives a surface temperature: "Surface about
  X K. Reach [260 K] by moving the orbit". The search range is an eighth to eight times the present size (log).
  The answer is set through the box's usual `commit` (one set_value, one Undo; what followed names it "… for a
  surface about 260 K"). No honesty mark: the world stays physical.
- Tests: `packages/ui-contract/test/reach.test.ts` (Sol: Mars to 260 K moves in to 1.0 to 1.5 AU in under 26 tries,
  state hash unchanged; 5000 K is out of reach and names the near end). `tests/ui/editing.e2e.mjs` desk: Mars, 260 K,
  lands near 1.03 AU, Undo takes it back. Screens: project files `sse4/handover-screens/ED5-screens/`.

## Part 2a (25 Nov): Reach it from the handle sheet

- A handle lists the results it can reach (`reach` in `ui-contract/src/handles.json`, keys of the sheet's `results`):
  the world's Orbit reaches `t_surface_now_k`, its Mass reaches `gravity_m_s2`. A pack's handle reaches the same way.
- `packages/views/editing/src/Reach.svelte` draws one line per result under the strip (Handles) or in the box (Box):
  "Gravity 0.38 g. Reach [ ] by changing Mass". The figure typed is read by `readFigure` in any unit of the result's
  kind ("1 g", "9.81 m/s²", "15 °C"). The search runs in the kernel field's units (`fieldValue`) from an eighth to eight
  times the value now; `resultFrom(address, key)` in reach.ts reads the result. The Box's inline form from part 1 is gone.
- e2e: Mars to 9.81 m/s² by mass lands at 0.282 Earth masses ("gravity 1 g"), then Undo.

## Part 2b (25 Nov): yes-or-no results

- A handle's `reach` may list a yes-or-no result. The world's Mass lists `core.dynamo_now`, which the results sheet
  calls "a field of its own" or "no field of its own". `Reach.svelte` shows a button such as "Give it a field of its
  own by changing Mass".
- `reachIt` takes `want(got)`. The result is read as 1 or 0 (`resultFrom` follows dotted keys into the surface entry
  and turns booleans into numbers), the target is 0.5, and the bisection ends on the bracket end that gives the
  wanted answer, so it is the nearest value that does. When an end of the range has no result (no core in a world that
  small), that end draws in toward the middle, up to four times.
- Checked against the kernel: Mars keeps a dynamo today from about 0.165 Earth masses (0.107 now). Test in
  `reach.test.ts`, and an e2e step on the Mass strip, then Undo. Screen: `ED5-screens/reach-field-desk.png`.

## Part 2c, first step (25 Nov): locked, by orbit

- The Orbit handle's `reach` in handles.json adds `locked`, so its Reach offers "Give it locked to its star by changing
  Orbit" (or "turns freely") beside the surface. No code: Reach's yes-or-no rows are general.
- Checked against the kernel (`reach.test.ts`): Mars is locked to the Sun today from about 0.564 AU in.
- e2e: editing (desk), the Orbit box shows the locked row; the surface row's locators are now scoped to their row.

## Part 2c, rest (queued)

- Pin it: force a result, carrying the honesty mark (GM view only).
- Reach it on other results: "make the year 365 days" (a typed year on the size strip already does it; a reach needs
  the year among the kernel's surface results), "warm enough for liquid water" offering Orbit, Air or the star's mass.
- From a what-if: needs a branch inside a branch, or a scratch copy of the what-if's edits.
- Swap the bisection for kernel K1's sweep when it lands.
