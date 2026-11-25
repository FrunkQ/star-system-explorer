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

## Part 2 (queued)

- Pin it: force a result, carrying the honesty mark (GM view only).
- Reach it on other results: gravity by mass, "make the year 365 days" (already a typed year on the size strip),
  "make it locked", "warm enough for liquid water" offering Orbit, Air or the star's mass.
- From a what-if: needs a branch inside a branch, or a scratch copy of the what-if's edits.
- Swap the bisection for kernel K1's sweep when it lands.
