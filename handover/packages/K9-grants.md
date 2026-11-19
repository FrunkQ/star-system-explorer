# K9 additions: grant op and tag-driven grants (seats gaps from V10)

Status: done (19 Nov 2026). Kernel track, PR into `main`. Asked for by the UI track's V10 log ("Kernel gaps"),
passed on by the coordinator.

## What was built
- `sse_observe::grant(chronicle, address, audience, view, default, at, publisher)`: one `observe.grant` event that
  sets `observe.view[.<audience>]` on a body (everything under it inherits), or with `view = None` unsets it so the
  body falls back to its parent, tags and defaults. With `default = true` it sets the audience's starting view
  (`observe.view.<audience>.default` on the universe root). The event's data names the audience and the view, so
  history reads "Vesh learned Tarn at ...". Before this, V10 wrote these as `authored.set` events.
- Campaign tag grants (`Campaign::tag_grants`, `TagGrant { audience, tag, view }`): bodies carrying `tag` are shown
  to `audience` through `view`. `audience` may be `*`, and `tag` may say `{audience}`, so one rule
  `* / discovered:{audience} / scanned` covers every empire (V10 already records `Audience.tag` as
  `discovered:<id>`). Precedence, walking up from the body: at each level an explicit grant first, then a tag
  rule; then the audience's starting view; then the campaign default. So a grant on a body beats its tag, and a tag
  on a moon beats a grant on its planet. The field is omitted when empty, so existing campaigns, packs, manifests
  and goldens serialise exactly as before.
- Tags come from wherever scenes already take them: authored `tag.*` facts and the stack's rule pack (sift rules),
  so "tags drive discoveries" works with no GM tick: a rule pack that tags what a sensor range covers, or an
  authored `tag.discovered:vesh`, grants the view.
- `effective_view_tagged` (scenes use it); `effective_view` keeps its signature and ignores tags.
- Kernel op `grant` (`Request::Grant { address, audience, view?, default?, at_s }` gives
  `Response::Granted { event_id, view, state_hash }`; errors `unknown_view`, `bad_address`, `no_data_views`).
  TypeScript types regenerated (`packages/contracts/src/generated`). Ops lists in sse-observe and the kernel.

## Checks
- `cargo test -p sse-observe grant`: set, inherit to moons, take back to the starting view, the everyone scene
  untouched; tag rules per audience with `{audience}`, precedence both ways, single-audience rules, no
  `tag_grants` key when empty.
- `cargo test -p sse-kernel grants_through_the_api`: the contract (requests 110 to 116, appended so earlier
  indices hold) grants a starting view and a discovery, the scene shows Tarn and its moon scanned for Vesh, taking
  back restores the earlier state hash, and the two error cases.
- `tests/golden/contract.jsonl` changed on purpose: seven new responses, and the `hello` op list and the
  unknown-op error message now include `grant`.

## For the UI track
- Replace V10's `authored.set` writes on `observe.view.<audience>` with the `grant` op (a `grant` intent kind in
  the session). Discoveries then read as grants in history.
- For automatic discovery, add a campaign tag rule (`tag_grants`) to the campaign's observe setup and tag bodies
  (authored, or a rule pack). The Discoveries table can keep its tick: a tick is a `grant`.

## Known limits
- Tag rules are campaign-wide (in the observe setup); per-seat rules would live in the seat plan if wanted.
- A tag rule grants while the tag is present; there is no record of when a tag first appeared beyond the event that
  set it.
