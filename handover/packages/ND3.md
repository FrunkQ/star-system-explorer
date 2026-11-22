# ND3: maths blocks and the Equation block

Design: `handover/design/SSE4-equation-block-design.md` (all of section 9 accepted by Alex, 22 Nov).
Track ND, the new-physics demo (`handover/design/SSE4-new-physics-demo.md`). Owner: Build kernel resolve and rules.

## What landed

- **Maths blocks** (`crates/sse-logic/src/blocks.rs`): `math.power`, `math.sqrt`, `math.exp`, `math.expm1`,
  `math.ln`, `math.log10`, `math.sin`, `math.cos`, `math.tan`, `math.atan2`, beside the arithmetic blocks.
  One function, `blocks::number`, holds the pinned maths of every number block and is shared with the
  Equation, so a formula and its blocks give the same bits and the same refusals (division by zero, the
  logarithm of a number not above zero, a result too large to hold). `math.power` takes a fixed exponent
  in its config (`"5"`, `"1/4"`, `"-3/2"`), which keeps the dimension, or a wired exponent on pure numbers.
  `expm1` is in the set because the Planck law needs it to match `sse-light`.
- **Units as text** (`sse_core::quantity::parse_unit`, `si_unit`): `g/cm3`, `eV cm3/g`, `W m-2 K-4`,
  `m3 kg-1 s-2`, `km/s`, `Myr`; SI symbols with the prefixes G M k c m u µ n p f, plus g, eV, erg, bar,
  au, pc, ly, yr, day. `logic.const` may now carry a `unit`.
- **Dimension rule from config** (`sse-flow`): `Atom::dim_rule(config)` and `DimRule::Power(p, q)`, so a
  power block's output dimension follows its exponent; `x^(1/2)` of a length is refused in words.
- **Wiring check** (`sse-flow`): `Atom::check_wiring(config, sources)` runs once a flow is wired. The
  Equation uses it to refuse a typed number (`logic.const`) on any of its inputs.
- **The Equation block** `sse.math.equation` (`crates/sse-logic/src/equation.rs`), as the design's section 2
  writes it. On load: a worded name (two words or more, no symbol or id), curious and science lines, a source
  with a DOI or address and where in it (unless marked `logic`), at least one worked example (run on load,
  relative tolerance 1e-6 or the example's own `tolerance`), units read and dimensions worked through the
  formula, bare numbers only 0 to 32 and their ratios, at most 64 operations, every declared letter used,
  every output set once. Lists: an input marked `list` makes the formula run once per entry. The note
  ("Why?") carries the formula, its canonical form, each letter with its worded name, value and unit, the
  number of choices, and the source.
- **The Model number block** `sse.model.number` (`crates/sse-logic/src/model.rs`): value, unit, allowed
  range (refused outside), grade (`derived`, `measured`, `fitted`, `assumed`) and source. Its note marks
  fitted and assumed numbers as fudges, for ND6 to list.
- **Expansion** (`equation::expand(&FlowDoc)`): the same flow with every Equation replaced by maths
  blocks wired as the formula is written. Bare zeros take the dimension of their place (a `logic.const`
  with a unit); `-x` is `x * -1` (exact, sign of zero too).
- **Orac** (`packages/agents/src/orac-blocks.ts`): the box target's form gains the Equation and Model
  number in one entry; the kernel trial refuses an invalid Equation before Accept.

## Tests

- `sse-logic` `equation_tests`: the design's example; every refusal of section 10 in words that name the
  place; a Planck curve as an Equation over 400 wavelengths at seven temperatures equals `sse_light::planck`
  bit for bit; seven formulas of every kind (choices, powers fixed and wired, lines, trigonometry, min, max,
  clamp, unary minus) give the same bits and the same evidence as their expansion at four points; a choice
  reads only the side it takes; dimensions through the power and root blocks; the unit reader.
- `sse-logic` `truth_tables`: each maths block against `sse_core::math` to the bit.
- `sse-kernel` `matter_tests::an_equation_follows_a_looked_up_constant`: escape speed as an Equation with G
  from a Look up block (fixture `tests/fixtures/packs/equation-flows.json`) gives the formula's own value on
  the real sheet and exactly √2 times it with twice G, with no edit to the pack; a typed number in the
  formula refuses the stack.

## Decisions to know

- **A choice works out only the side it takes**, and a line that is not an output is worked out only when
  something needs it, so a guard (`if t <= 0 then 0 else ...`) protects what it guards. Blocks in a flow
  are all evaluated, so where an untaken side cannot be worked out (a division by zero there), the
  expansion reports that error and the Equation gives a value. Wherever the expansion runs, the bits are
  the same. Making flows skip an unused branch is a flow-engine change for later, if it is ever wanted.
- **Conditions** are comparisons (`< <= > >= ==`) joined by `and`, `or`, `not`, written inside the formula;
  they evaluate exactly as the compare blocks do with factor 1. The K8 condition language reads world facts
  by name, which an Equation never does, so it is not imported here.
- **The 32 limit** on bare whole numbers is a judgement: large enough for the mathematics of the formulas in
  the atoms (2, 3, 8, 15, 16, 32/5), small enough that a unit factor (60, 100, 1000) cannot pass as one.
- **Lists are not expanded** into blocks (the maths blocks take single numbers); expand per entry.
- **Parsed per call.** `eval_typed` parses the formula each call; caching the parsed steps at pack load
  comes with ND3b, when base packs hold Equations and the timing check (PF2's bench) applies.
- No golden hash changed: the new blocks are registered but no shipped flow uses them yet.

## Not done here (next)

- The canonical form is computed and shown, but the pack hash is still over the formula as written; hashing
  the canonical form belongs with the pack loader when packs ship Equations (ND3b).
- The block face (formula typeset), the Engine view of the expansion and as-you-type checking are the
  chart room's (U-track) to draw from the note and `canonical`.
- ND3b: moving the Rust atoms into Equations family by family, behind unchanged results, goldens and
  PF2's 5 % timing check.
