# ER1: the physics in force, and the side-door ledger

Track ER (edits that reach everything). Design: `handover/design/SSE4-edits-reach-design.md` sections 3, 4 and 12. Built 26 Nov 2026.

## What landed

- **New crate `sse-physics`.** `Physics` is built once when a stack loads and never changes: the stacked data sheet (`Reference`), the block library (the shipped formula blocks until ER3), the compiled flows, every `provides.other` settings block by key (bottom first, with the pack it came from), and a **physics hash**.
- **The physics hash** covers only what can change a worked-out number: the sheet's tables, every pack's units and quantity kinds, the block library (formula file texts and every block id and version), flows, boxes, and every settings block whose key is not in `sse_physics::NOT_PHYSICS` (`about`, `credits`, `holding_cards`, `lenses`, `names`, `notice`, `overlays`, `views`, `vocabulary`). Anything not listed counts as physics on purpose: a key wrongly counted costs a cache refill, a key wrongly left out keeps a wrong answer. The stack hash is unchanged and stays the universe's identity.
- **One scope.** `sse_physics::with(&physics, constants, f)` puts the physics in force for one call and sets the two older scopes from it (`sse_core::constants::with` and `sse_flow::sheet::with`), so they are now views of it. `Kernel::handle` uses it; the DM6 dated handle is still read from the call's date. `sse_physics::now()` gives the physics in force.
- **Kernel.** `Kernel.physics` is built at `load_stack` (bare before any stack) and exposed as `Kernel::physics()`. The kernel's flows are held in one `Arc` shared with it.
- **No reader changes.** No compiled code reads through the new door yet; that is ER2 (blocks), ER4 and ER5 (the sheet) and ER6 (settings).

## The side-door ledger (the check the coordinator approved, design section 4 rule 3 and section 12)

- `scripts/audit-side-doors.py` counts, per file outside tests, four kinds of side door: `block` (a block run with its shipped settings), `sheet` (the sheet's built-in values), `sheet-units` (the same, in a file that reads only units and conventions; kept apart under Alex's 26 Nov ruling that units never move), and `settings` (a shipped pack's settings instead of the stack's). The script's header gives each kind's exact patterns.
- The ledger is `handover/packages/ER-side-doors.txt`, written once from today's code. `--check` fails on a new site and on a converted site whose line is still in the ledger; with `--base` it also fails if any ledger line is higher than on main. `--shrink` rewrites the ledger after you convert sites and refuses if any count would rise.
- In `scripts/check.sh` (against `origin/main` when it is known) and as this package's own line in `ci.yml`.
- `scripts/test-audit-side-doors.py` builds small trees and sees the check go red on a new site, red on a stale line, red on a grown ledger against the base, and green after `--shrink`.

### Counts on 26 Nov 2026 (this scan's rules)

| Kind | Sites | Files |
|---|---|---|
| block | 35 | 23 |
| sheet | 269 | 78 |
| sheet-units | 42 | 27 |
| settings | 48 | 31 |

These are counted by the script's textual rules, so they differ from the design's hand counts (20 block call sites; 119 sheet files of which 35 units-only; 96 settings sites in 45 files). The `block` count includes the lines that take the library or a block's shipped settings, so one call site is often two lines. The `settings` count counts a pack read with `include_str!` once where it is read, not at each use of the constant holding it, so it is lower than the design's 96. ER1b's reach gate, which runs every op with the physics nudged, is the mechanical check that finds what a textual scan misses.

### How an area's agent shrinks it

Convert the reads in a file you are already changing to the physics in force, run `python3 scripts/audit-side-doors.py --shrink`, and commit the ledger with the change. A new file starts at zero. If the check says NEW SIDE DOOR, read the value through `sse-physics` instead of adding a line; the ledger never takes new lines.

## Checks

- `tests/golden/` unchanged (nothing is read through the new door yet).
- `sse-physics` tests: a stack differing only by the names pack has the same physics hash; a settings number or a sheet value changed gives a different one; the hash is stable; `with` sets both views and restores everything, also on a panic.
- `sse-kernel` `physics_tests`: the same through `load_stack` (the stack hash moves for a names pack, the physics hash does not; the strong-gravity fixture moves the physics hash).
- `scripts/check.sh` green.
