# The `sift` op (for the agents)

Status: done (19 Nov 2026). Kernel track. Asked for by the agents track (Minerva writes from sifter hits; handover notes A2 and A3 list it as a gap). Engine: K8's `sse_sift::sift`.

## What was built
`sift {sifters?, audience?, scope?, at_s}`, GM machine only:
- Runs the stack's sifters (`provides.other.sifters`, loaded with the rules; all of them, or only the ids in `sifters`) over the campaign at `at_s`, at or under `scope` (default: the universe root).
- **GM (no audience, or `gm`):** the universe facts plus the record's kept values, authored tags and rule tags with their evidence, and the chronicle, so event patterns run and evidence carries the event ids.
- **An audience:** the kernel builds that audience's scene (the same `scene` op the chart room uses) and sifts only what it shows: exact values, never placeholders or bands, and the tags the scene lists. Event patterns are skipped and named in `skipped`, because players never see the chronicle; so no event id reaches an audience.
- Response `sifted {audience?, hits, skipped}`. Each hit: `{sifter, label, subject, evidence: [{address, field, value, event?}], significance: {score, persistence_s, reach, reversibility, rarity_p, components}}`, best first (ties by sifter id, then address), as K8 orders them.
- Errors: `no_universe`, `unknown_sifter`, `bad_address` (a scope outside the universe), and the scene's own errors for an audience (for example a stack with no data views).

**Fixture** `tests/fixtures/packs/rpg-sifters.json`: `heavy-world` (mass above 5e24 kg, a scanned-tier field), `sleeping-threat` (`gm.threat`, secret) and `granted` (an event pattern).

## Checks
- Kernel test `sift_through_the_api`: the GM sees both fact hits with event ids; one sifter and a scope; an unknown sifter and an outside scope are refused; the crew's long-range view gives no hits and skips `granted`; after a scan the crew gets `heavy-world` only, with no secret field and no event id; before the scan, nothing.
- Contract: 5 new cases (lines 188 to 192); line 1 lists the op. TypeScript types regenerated.

Golden changes (on purpose): `contract.jsonl` line 1 and the new lines 188 to 192. No engine golden changed.

## Limits
- Rarity is not measured here (no clouds are passed), so `rarity_p` is null and that component is 0. Passing the resolved clouds is a later step if Minerva wants rarer things first.
- An audience's tags carry no rule evidence (the scene gives names only), so a tag-based hit for a player shows the tag, not the facts behind it. That is on purpose: the facts may be ones the player has not been shown.
