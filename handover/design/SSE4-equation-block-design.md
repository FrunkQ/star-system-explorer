# SSE4: the Equation block (package ND3)

Status: **go-ahead from Alex, 22 Nov 2026** ("blocks should be able to do basic math! power, exponential or
logarithm blocks are valid. We can create them if needed. Or just have a general purpose equation, which is
what I thought we had"). This is the short design the new-physics demo plan asked for
(`SSE4-new-physics-demo.md`, question 2). Owner: Build kernel resolve and rules. Section 9 has three small
questions with recommendations; none blocks a start.

## 1. Why

Checked on main, 22 Nov: a box can add, subtract, multiply, divide, look up, select, clamp and sum
(`sse-logic/src/blocks.rs`). It cannot raise to a power, take an exponential or a logarithm. So any relation
past arithmetic needs a new Rust atom, raised as a GitHub issue and written by a maintainer. That is the one
place where "bring in new physics" still means "write code". The 80 physics atoms on main are mostly one
formula each.

Two things close it. They are one mechanism in two notations, exactly as the condition language already is
(`sse-logic/src/condition.rs`: a compact form and the same thing as blocks).

- **Maths blocks**: power, exponential, natural and base-10 logarithm, square root, sine, cosine, tangent,
  arctangent, joining the arithmetic blocks already there.
- **The Equation block** (`sse.math.equation`): one block whose formula is written as text in the pack. The
  kernel can expand any Equation into the maths blocks above and get the same answer to the last bit.

Orac writes the compact form well. A person reads either. Blocks are not code: an Equation has no loops, no
functions of its own, no input or output, and always finishes.

## 2. What a pack author writes

```json
{ "atom": "sse.math.equation",
  "config": {
    "name": "How far carbon's colours shift when squeezed",
    "formula": "shift = alpha * density",
    "inputs":  { "density": { "kind": "atmosphere.photosphere_density", "unit": "g/cm3", "valid": [0, 2] } },
    "params":  { "alpha": { "value": 0.2, "unit": "eV cm3/g", "range": [0.1, 1.6], "grade": "fitted",
                            "source": "Blouin et al. 2019, ApJ 878, 63 (DOI to be added from the citations check)" } },
    "outputs": { "shift": { "kind": "light.band_shift", "unit": "eV" } },
    "source":  { "text": "Kowalski 2010, as cited by Hollands et al. 2026", "doi": "(to be added from the citations check)", "where": "shift proportional to density" },
    "explain": { "curious": "Squeeze the gas and carbon's colours slide towards the blue.",
                 "science": "The C2 electronic transition energy rises in proportion to density." },
    "examples": [ { "density": 0.15, "shift": 0.03 } ] } }
```

Illustrative only: the sources are named as the white dwarf paper cites them, and their DOIs go in through
the usual citations check, not from memory.

This example uses no constant of nature. One that does (escape speed, say) has an input named for the
constant, wired on the flow from a Look up block; nothing about the constant is written in the Equation.

## 3. The language

- Names: the block's inputs and parameters, and nothing else.
- Operators: `+ - * / ^`, brackets, unary minus.
- Functions: `exp ln log10 sqrt abs min max clamp sin cos tan atan2`.
- A choice: `if <condition> then a else b`, with the condition written in the existing condition language.
  Under "processes, not classes" a choice is for a real boundary (a phase change, a sign), not a class gate;
  the editor marks every choice so a reviewer sees them.
- Several outputs: several lines, `name = expression`; a later line may use an earlier one.
- Lists: if an input is a list (a spectrum on its wavelength grid), the formula runs once per entry and the
  output is a list of the same length. Totals use the Sum block that exists. This is what lets a Planck curve
  or an absorption profile be an Equation.
- Not in the language: loops, recursion, user-defined functions, reading the world, random draws. Iteration
  stays with Solve nodes; heavy physics stays with tables and Big Blocks.

## 4. Numbers in a formula

- A constant of nature or a property of a substance never appears inside an Equation, as a number or as a
  hidden reference. It arrives on a **visible wire from a Look up block** (`sse.reference.lookup`) that sits
  on the flow beside it (Alex, 22 Nov: "constants called by a box into this box"). A reader sees a block
  named, say, "The gravitational constant (data sheet)" with a wire into "Speed needed to escape". The
  Equation has an input for it like any other input. So the "no typed constants" check holds for packs as it
  does for code, one Look up can feed several Equations, and a what-if that changes G shows on the flow
  exactly where it enters and reaches every Equation by itself.
- A number that belongs to the model (a fitted slope, an exponent from a paper) is a **parameter**: named,
  with a unit, a range, a source and a grade (`derived`, `measured`, `fitted`, `assumed`). It shows as an
  editable control like any other parameter. `fitted` and `assumed` parameters are listed automatically in
  the pack's fudge list (ND6).
- Bare numbers allowed in the text: small whole numbers and their ratios (2, 4, 1/2, 3/2), because they are
  part of the mathematics. Anything else is refused at load with "make this a parameter or look it up".

## 5. Units

Every input, parameter and output declares a unit. On load the kernel works the dimensions through the
formula and refuses, in words, what does not agree: adding a length to a time, an `exp` or `ln` of something
that is not a pure number, a power with a dimensioned exponent, an output whose dimension is not the declared
one. Values are converted to SI on the way in and computed in SI. This check is the main protection against
a paper being transcribed wrongly, by a person or by Orac.

## 6. Same answer everywhere

- Evaluated only through `sse_core::math`, in the order written. The kernel never rearranges a formula and
  never fuses a multiply with an add.
- The parsed formula has one canonical form, which goes into the pack's hash.
- Expanding an Equation into maths blocks gives the identical bits; a test proves it for every Equation in
  every shipped pack.

## 7. Trust

- A physics Equation must carry a source with a DOI or address, and where in the source the relation is
  ("eq. 3", "table 2"). A bookkeeping Equation may be marked `logic` and needs none, as logic blocks do today.
- At least one worked example, run on load. For a relation from a paper, a number from that paper.
- Inputs outside their valid range still give a result, labelled `extrapolated`, as atoms do.
- The formula a person sees is drawn from the same text the kernel computes. There is no separate
  "display formula" to drift.
- A pack from a stranger is safe to load: an Equation cannot read, write, loop or call out.

## 8. On screen and cost

- On a flow it is a block like any other. Its face shows the formula, typeset.
- "Why?" shows the formula, then the same formula with this world's numbers in place, then the source.
  Curious shows the sentence; Science the formula; Engine the text, the canonical form and the expansion.
- Editing the text checks as you type (names, units, range) and goes through a change request, like a
  parameter edit. Orac drafts one inside a box through the box check it already has; its catalogue gains the
  language in one entry.
- Cost: at most 64 operations per Equation (cost class trivial); over a list, at most the existing list limit
  of 4096 entries (light). Larger is refused at load: split it into a box.

## 8A. Names that mean something (Alex, 22 Nov)

Alex: "this will probably impact most current blocks, as they are able to have their maths moved into one
and assigned a name that means something. Reading an SSE4 flow should be readable by a keen amateur, and they
kind of know the process by looking at the block names."

So the name is a required part of every Equation, and it is the first thing on the block's face.

- A name says what happens in the world, in ordinary words: "Heat a world receives from its star",
  "Speed needed to escape", "How far carbon's colours shift when squeezed". Not the symbol, not the author
  of the law, not a code id. The law's usual name goes in the Science line ("the Stefan-Boltzmann law").
- Input and output names are words too ("distance from the star", not `a`). The formula keeps short symbols;
  each symbol is tied to its worded name on the block.
- Test for a flow: read only the block names, top to bottom, and you should be able to say what the flow
  works out and in what order. Reviewers and Orac's check both apply it; a name that is only a symbol or an
  id is refused at load.
- Names are text for people and can be translated or reworded by a pack's vocabulary. The id stays fixed.

**Moving the existing blocks over (package ND3b, after ND3).** Most of the 80 Rust atoms on main are one
formula each. Each moves into an Equation in its base pack, under a name by the rule above, behind a test
that every result and every golden hash is unchanged. Done gradually as a shrinking list, like the typed-
constants ledger, one family at a time (`sse.star.*`, `sse.orbit.*`, `sse.planet.*`, `sse.heat.*` first: they
are plain formulas). A block stays in Rust only if it truly cannot be a formula (it iterates, or is a Big
Block), and says why in `docs/fudges.md`. While moving, the block's id, ports and results do not change, so
no flow or save breaks. Owner: Build kernel resolve and rules, with each physics thread for its own family.

## 8B. Speed (Alex, 22 Nov: "this won't impact speed; I guess we could compile if necessary")

- A formula is parsed and unit-checked once, when its pack loads, into a short list of steps. Running it is
  a walk down that list: no text is read at run time.
- Heavy inner work stays in Rust: orbit stepping, the formation pipeline, surface tiles, large tables, the
  Big Blocks. Equations are for the one-formula relations.
- **Timing condition for ND3b.** Each family that moves carries a timing check beside its unchanged-results
  check: the kernel's benchmark flows for that family (a system resolve, a galaxy patch, a year of orbits)
  must run within 5 % of their time before the move, on the desktop and the phone tier. A family that fails
  does not move until the fallback below is in.
- **Fallback: compile at load.** If walking the list is ever too slow, the kernel compiles each formula at
  load into a closure (or a block of WebAssembly), keeping exactly the order of operations as written and the
  same `sse_core::math` calls, so results are identical to the last bit. The pack format, the hash and what
  a person sees do not change; this is a kernel detail and can be added later without touching any pack.

## 9. Questions for Alex

1. **The bare-number rule in section 4** (only small whole numbers and their ratios; everything else is a
   parameter or a look-up)? Recommended: yes. It is stricter than a spreadsheet, and it is what keeps a
   shared pack honest.
2. **Move existing Rust atoms over to Equations in the base packs, gradually**, each behind a test that the
   bits do not change? Recommended: yes, later, as a shrinking list like the typed-constants ledger; not part
   of ND3. It would make most of the base physics readable and editable as formulas, not only parameters.
   **Accepted by Alex, 22 Nov** (section 8A, package ND3b).
3. **Its name on screen: "Equation"?** Recommended: yes.

## 10. Acceptance (ND3)

- Power, exponential, logarithm, root and trigonometry blocks exist and match `sse_core::math` bit for bit.
- A Planck curve written as an Equation over a wavelength list matches `sse-light` to the last bit.
- A formula with a unit mismatch, an unknown name, a bare number outside the rule, or more than 64
  operations is refused at load, in words that name the place.
- An Equation and its expansion into blocks give identical results and identical "Why?" evidence.
- Orac drafts a box containing an Equation from a sentence; an invalid one never reaches Accept.
- A what-if that changes a looked-up constant changes the Equation's result with no edit to the pack.
- An Equation that names a data sheet row inside itself, or has a constant's value typed in, is refused at
  load; every constant it uses shows as a wire from a Look up block.
- ND3b: each moved family passes its timing check (section 8B) as well as its unchanged-results check.
- Every Equation has a worded name; a flow of Equations passes the read-the-names test (section 8A).
