# TP4: every process says its pace (the pace ladder)

Kernel, 26 to 27 Nov 2026. Track TP (one time pipeline); design `handover/design/SSE4-time-scales-design.md` sections 6, 9 and 10, package row in section 11.

## What it does
Each piece of time-dependent physics now has a declaration as pack data beside its block, and the kernel can list them as one table, the pace ladder, ordered from the fastest. The clock (TP7) will read it to know how fast each thing on screen is worth watching; `happenings` (TP6) will read the declared marks. Nothing here changes any number another op works out: the physics never slows, the clock does.

## The declaration (`provides.other.processes` in `packs/base-time`)
`{about, list: [...]}`; each entry:

- `id`, `says` (plain words), `how` (what the code does today, with its known gaps);
- `shape`: `line`, `stream` or `march`, and no fourth;
- `anchor`: one of the named anchors of design section 4 (`time.origin`, `time.present_s`, `campaign.now_s`, `system.birth_s`, `time.j2000_s`, `calendar.epoch_s`, `orbit.epoch_s`). `system.birth_s` and `time.present_s` arrive as facts with TP2; until then the declaration names the anchor the process will read;
- `tau`, the process's own time, from physics and never a typed number. One of:
  - `{block, output}`: a block of the library, run through the block door (`sse_physics::run`, ER2), its output port checked to be a time;
  - `{equation}`: an Equation (`sse.math.equation` config, with its source, examples and explain lines, checked on load like any other);
  - `{driver}`: another declared process whose own time this one follows (section 7, follow);
  - `{marks}`: no pace in physics; it moves only at its own dated events (the campaign's record);
- `subjects`: the reference subjects a block or Equation is worked out at, each input a sheet row `sheet/<id>` read from the data sheet in force;
- `marks`: what it can publish to `happenings` (TP6);
- `grid`: a march's grid rule (required for a march, refused on a line or stream);
- `ops`: the kernel ops that read it.

A pack higher in the stack replaces a declaration by its id or adds one. A declaration of the wrong form is refused by name, with why, and the rest of the ladder still comes back.

## Shipped declarations (six)
| id | shape | anchor | own time from | at | value |
|---|---|---|---|---|---|
| `star.activity` | stream | system.birth_s | `sse.activity.convective_turnover_time` | the Sun | 14.5 days |
| `orbit.kepler` | line | orbit.epoch_s | `sse.orbit.period` | the Moon round the Earth; a world at 1 au | 27.5 days; 365.26 days |
| `star.life` | line | system.birth_s | `sse.star.ms_lifetime` | the Sun | 10 Gyr |
| `system.ageing` | march | system.birth_s | driver `star.life` | the Sun | 10 Gyr |
| `cosmos.expansion` | line | time.origin | Equation, Hubble time 1/H0 (Hogg 1999 eq. 3) | today | 14.45 Gyr |
| `campaign.record` | line | campaign.now_s | its own marks | | none |

The Moon's rung leaves the Moon's own mass out of the pair (the block takes one total mass and a subject input is one sheet row), so it reads 27.45 days against the real 27.32; the subject says so.

## The op
`pace_ladder` (no arguments, read only, needs no universe): `{processes, rungs: [{id, says, shape, anchor, subject, tau_s, tau_from, why, marks, grid, ops, how, pack}], refused: [{pack, id, why}]}`. One rung per subject; a rung whose own time could not be worked out has `tau_s` null and says why (for example a stack without the data sheet: "mass reads sheet/M_sun, which the stack's data sheet does not hold"). Processes paced by their own marks come last.

## The check (a ledger that only shrinks)
`crates/sse-kernel/src/pace_tests.rs`, `every_dated_op_is_declared_or_on_the_ledger`, runs in `cargo test` (no CI change). It reads the `Request` enum in `protocol.rs` as text: a variant with a field whose name ends `_s`, `_yr`, `_myr` or `_gyr` and whose last word says when (`at_s`, `occurred_at_s`, `t_myr`, `age_myr`, `true_on_s` ...) is a dated op. Each must be named in a declaration's `ops` or listed in `handover/packages/TP4-undeclared-ops.txt`. A ledger line whose op is now declared fails until deleted; a line that is not a dated op fails too.

Today: 97 dated ops; 67 declared, 30 on the ledger (weather, formation, surface, navigation, the night sky, the galaxy chart, nebulae, fields, the reads of a resolved system, and the sifters). Each leaves the ledger when its owner declares its process.

## Tests, seen red
- `the_check_fails_an_undeclared_dated_op`: a request `tide_at { at_s }` put into the protocol source fails the check by name; a declared op left on the ledger fails too. Also run for real: with the ledger empty the check listed all 30 undeclared ops and failed.
- `the_live_ladder_works_out_every_rung`: live base-reference and base-time; no refusals; the Moon within 1 % of 27.32 days, the year within 1e-4 of 365.256 days, the Sun's 10 Gyr, the march equal to its driver, the turnover between 5 and 40 days, the Hubble time within 1 % of 14.45 Gyr; fastest first; the record last.
- `a_rung_without_its_sheet_row_says_why`; `packs_add_replace_and_are_refused_by_form` (a fourth shape, a march with no grid, an anchor not named, a missing driver, an op that does not exist, a typed number as tau: six refusals; a replaced declaration comes from the top pack).

## Goldens
New area `time` (`tests/contract/areas/time.json`, `tests/golden/areas/time.jsonl`): the ladder with the sheet, and without it. Its packs are frozen fixtures: `tests/fixtures/packs/base-time-processes.json` (a copy of the live pack today; BX1's `base-time.json` is the older pack, before the declarations, and its `pairs` area keeps it) and `tests/fixtures/packs/time-sheet.json` (the eleven sheet rows the subjects read, copied unchanged from base-reference, group notes left out). The core golden (`tests/golden/contract.jsonl`) did not move.

`sse-time`'s own golden (`tests/golden/time.json`) moved by one line, on purpose: its `manifest` hash is the hash of the base-time pack, which now carries the declarations. The chronicle head, the run digest and every event and window are unchanged.

## Not in TP4
- The 30 ledgered ops' processes: each owner declares its own (weather's day, formation's disc lifetime, the galaxy's orbit at the Sun's radius, which needs the galaxy's mass inside it as a stack value).
- Per-subject ladders (the pace of this system's own bodies): TP7 asks for the processes in view; the declarations already say which ops read them.
- Marks themselves: TP6.

## A trap found on the way (fixtures and the browser)
The first push hung CI's wasm job for three hours. The cause: the `time` area's stack hash differed between the native kernel and the worker. JavaScript writes a whole number between 2^53 and 1e21 in plain digits (`4.560563969097498e+17` goes out as `456056396909749800`), which the kernel reads as an integer, while the native runner read the fixture's float. So a pack value of that kind hashes differently on the web and natively. The test then never reached `worker.terminate()` and kept node alive. The fix: the two fixtures are written as JavaScript writes them (`JSON.stringify` of the parsed file), and the Hubble example in base-time is written as the integer. The same hazard is in any shipped pack with such numbers (base-reference's parsec row, for one). Its stack hash on the web differs from the native one. Not fixed here; reported to the coordinator.
