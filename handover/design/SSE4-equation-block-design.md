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
    "name": "Swan band shift",
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

- A constant of nature or a property of a substance never appears as a number. It comes in on an input wired
  from the Look up block (`sse.reference.lookup`), so the "no typed constants" check holds for packs as it
  does for code, and a what-if that changes G reaches every Equation by itself.
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

## 9. Questions for Alex

1. **The bare-number rule in section 4** (only small whole numbers and their ratios; everything else is a
   parameter or a look-up)? Recommended: yes. It is stricter than a spreadsheet, and it is what keeps a
   shared pack honest.
2. **Move existing Rust atoms over to Equations in the base packs, gradually**, each behind a test that the
   bits do not change? Recommended: yes, later, as a shrinking list like the typed-constants ledger; not part
   of ND3. It would make most of the base physics readable and editable as formulas, not only parameters.
3. **Its name on screen: "Equation"?** Recommended: yes.

## 10. Acceptance (ND3)

- Power, exponential, logarithm, root and trigonometry blocks exist and match `sse_core::math` bit for bit.
- A Planck curve written as an Equation over a wavelength list matches `sse-light` to the last bit.
- A formula with a unit mismatch, an unknown name, a bare number outside the rule, or more than 64
  operations is refused at load, in words that name the place.
- An Equation and its expansion into blocks give identical results and identical "Why?" evidence.
- Orac drafts a box containing an Equation from a sentence; an invalid one never reaches Accept.
- A what-if that changes a looked-up constant changes the Equation's result with no edit to the pack.
