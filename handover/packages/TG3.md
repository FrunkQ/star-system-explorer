# TG3: authored overrules derived

Tags design (`design/SSE4-tags-design.md`) section 4 ("the GM always wins,
in one way") and section 7, package TG3. Built 21 Nov 2026 by the kernel
thread.

## What it delivers
- `RuleSet::apply_authored` (sse-sift) runs the stack's rules under the GM's authored tags:
  - an authored `tag.x = true` forces the tag on. It carries the authored mark: rule `authored`, with the fact as its evidence, in place of the rule's;
  - an authored `tag.x = false` holds the tag off, whatever its rules say;
  - rules that read a tag (one rule building on another) see the authored answer;
  - clearing the authored fact (null, or the fact unset) gives the rule's answer back, with its evidence. Derived tags are never stored, so nothing needs restoring.
- `sse_sift::authored_tags` reads `tag.<name>` true and false out of a subject's facts.
- The kernel's `scene` and `sift` ops use it. Before this, an authored `false` did nothing and an authored `true` lost to the rule.
- This changes the label only. To change the world, pin the fact (the existing GM override, with its honesty marks).

## Verified by
`the_gm_overrules_a_rule_tag_in_scenes_conditions_and_sifters`, with the Tarn fixture stack:
- the rules give `habitable`, `garden` (built on habitable) and `roche-danger`, and the sifter fires;
- holding `habitable` and `roche-danger` off removes all three tags from the scene and empties the sifter;
- forcing `habitable` on where its rule fails shows it with the authored mark, and `garden` still builds on it;
- clearing the authored facts restores the rules' answer and the sifter's hit.

## Goldens
None changed. No golden authors a `false` tag, and no golden authors a `true` tag that a rule also sets.

## Not done
- Triggers and programs read authored tags only, not rule tags (as before). An authored `false` keeps a tag out of their conditions, but they do not see derived tags yet. That gap predates TG3 and belongs with TG5 or a later package.
- The scene lists tag names only. Showing the authored mark on the card is TG10 (chart room), and the mark is available through `sift` and `rules.apply_authored`.
