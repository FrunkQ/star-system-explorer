# classify_preview: a drafted game table over the open campaign (21 Nov)

This was asked for by the agents track (Orac drafts game tables as class atlas schemes, PR 67). The accept card needs to show which worlds land in which row before the GM loads the table.

## The op
- Request: `{"op":"classify_preview","packs":[...],"scheme":"<id>","scope"?:"<address>","at_s"?:n}`.
  - `packs` is a candidate stack, bottom to top, in the same form as `load_stack`.
  - Only its sift content (axes, schemes, classes) is read.
- Reply: `classify_previewed {scheme, atlas_hash, at_s?, bodies, unclassified, failing_examples}`.
  - `bodies` holds one row per body in address order: `{address, base, fit, modifiers}`.
  - A body here is an address with `body.kind` or `body.mass`.
  - `failing_examples` lists the candidate scheme's class examples that come out wrong (`Atlas::check_examples`).
- Errors: `bad_pack`, `bad_stack` (atlas validation), `unknown_scheme`, `no_universe`, and `bad_address` for a scope outside the universe.
- Nothing is loaded or written. The stack in use, the chronicle and the state hash are unchanged.
- It runs on the GM machine only, because subjects see the full facts plus the record, as `sift` does. Cost: Light.

## Verified by
`classify_preview_labels_the_campaign_under_a_candidate_stack`:
- with two adopted formation systems, a drafted mass table puts every star in `heft/star` and every world in `heft/world`;
- the call is repeatable, and the state hash is unchanged afterwards;
- scope narrows the rows to one system;
- a flipped example is reported;
- an unknown scheme and a missing universe are refused.

## Found on the way
The shipped base atlas (`packs/base-atlas`) leaves every formation-adopted body unclassified in `physical`. Its classes need `makeup.*` facts, which adopt does not publish yet. Only the `physical/eccentric` modifier fires. Worth a later package: publish makeup fractions from the formation run at adopt.

## Goldens
`tests/golden/contract.jsonl` changed on purpose: the hello op list with the new op added, and two new cases appended (a drafted table over `u:aging/sys:1`, and an unknown scheme).
