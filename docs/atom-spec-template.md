# Atom spec template

New physics atoms follow issue, spec, tests, code (design handoff H6.2, build plan P5). Open a GitHub issue with
this filled in; the code lands only after the golden cases are agreed.

```
id:           sse.<domain>.<name>          (stable; bump version on any change of output)
version:      1.0.0
inputs:       name, quantity kind, SI dimension, validity range (where the physics holds)
outputs:      name, quantity kind, SI dimension
params:       name, dimension, default, allowed range, one-line meaning
cost class:   trivial | light | medium | heavy
references:   at least one citation (author, year, journal or book, equation or table)
explain:
  curious:    one or two sentences, no background assumed
  science:    mechanism, assumptions, where it breaks
  formula:    display form
golden cases: inputs -> expected outputs with tolerance, each with its source
              (formula reproduction from the paper, and at least one real object)
fidelity:     if it stands in for a more detailed model: observables, tolerance, validity, control
```

Rules from the build plan: no wall clock, no HashMap/HashSet, maths through `sse_core::math`, no fused multiply-add,
fixed iteration counts for iterative methods (the deterministic stopping rule), NaN is an error, never a value.
Outside the validity range the result is still produced and labelled `extrapolated`; where the formula has no
meaning (an eccentricity of 1 in an ellipse atom) the atom returns an error instead.
