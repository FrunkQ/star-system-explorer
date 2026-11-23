# ND6 kernel half: packs carry their own sources, fudges and reproduction report

Design: `handover/design/SSE4-new-physics-demo.md` 2.5 and section 3 (row ND6), accepted by Alex 22 Nov
(question 7). Equation design section 3: fitted and assumed Model numbers are listed among the pack's
fudges automatically. Owner of this half: Build kernel resolve and rules. The Packs screen half (show
them before the pack is added) is the chart room's.

## What landed

- **Pack manifest** (`sse-pack`, module `trust`): three optional lists, left out of the JSON when empty so
  every existing pack hashes as before.
  - `sources`: `{id, text, doi?, url?, licence?}`. Each needs words and a DOI or an address.
  - `fudges`: `{id, what, stands_in_for, replace_with?, flows?, source?}`. `flows` must be this pack's
    own flows or boxes; `source` must be one of its sources.
  - `reproduction`: `{name, source, where?, flow, inputs, expect: {kind: {value, unit?, tolerance}}}`.
    Inputs are written like a flow's golden case (SI or `{value, unit}`); `tolerance` is relative
    (0.05 is 5 %), the pack's own stated closeness. The flow may come from any pack in the stack.
  - Shapes are checked with the stack (`StackRejected`, pack named).
- **On load** (`sse-kernel::trust`): after every flow compiles and its own golden cases pass, each pack's
  reproduction checks run through the compiled flows. One outside its tolerance refuses the stack with the
  numbers side by side: `test.paper: reproduction check "A world like ours": body.surface_gravity:
  facts gives 9.8 m_s2, test.paper gives 9.81 m_s2, 0.2 % off, the pack allows 0.010 %`. A check that
  cannot run says why.
- **Fudge list**: the pack's declared fudges, then every `sse.model.number` node in its flows and boxes
  graded `fitted` or `assumed`, with id `<flow>/<node>`, its value, unit, grade and source.
- **Op `pack_reports {pack?}`** (read only, trivial): one report per pack of the stack in force,
  `{pack, version, licence, sources, fudges, reproduction: [{name, source, where, flow, pass, rows:
  [{kind, unit, published, got, tolerance, off, pass, error}]}], reproduced}`. An unknown pack is
  `unknown_pack`. Perf chunk: "Working out a system", beside `manifest`.
- `sse-flow`: `Flow::run_case(cx, inputs)` (one case's inputs to an evaluation), now shared by golden
  cases and reproduction checks; `ParamValue` and `param_si` exported.

## For the chart room (Packs screen)

In a trial (`session.trial`), after `load_stack` on the scratch kernel, call `pack_reports` with the
added pack's id: it shows sources, fudges and the reproduction rows before anyone accepts. A refusal
already arrives as `stack_rejected` with the side-by-side line in `issues`.

## Tests

`crates/sse-kernel/src/trust_tests.rs`: a paper-shaped pack (a fitted Model number, a declared fudge, two
sources, a check through the base planet flow) loads and reports; outside its tolerance it is refused with
both numbers; a check on a missing flow says why; bad sources, fudges and citations are refused; an unknown
pack has no report; a pack without the lists serialises without them. Contract golden: three requests
appended (all packs, one pack, an unknown pack); of the earlier lines only `hello` (the op list) and the unknown-op error (the list of ops) change, as with every new op.
