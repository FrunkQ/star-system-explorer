# SV1: one "Known to" answer and one intent (secrets and visibility design 3 to 7, 11)

## What it is

- **Landmarks.** For each audience a thing stands on a rung (the data view in
  force there, from grants as before) and one of three landmarks:
  `gm_only`, `heard_of`, `known`. The landmark views are the campaign's
  `landmarks: {gm_only?, heard_of?, known?}`, and any left out are found:
  `gm_only` is the built-in view `gm-only` (shows nothing; a pack may define
  its own of that id), `heard_of` the lowest view that shows anything
  (fewest tiers, then fewest shown overrides), `known` the view that shows
  the most tiers. A view that shows nothing is GM only whatever its id.
  (`sse_observe::known`.)
- **GM only means nothing at all.** A subject under a GM only view is left
  out of the audience's scene even if a note or tag would have shown it, so
  it is absent from scene, disclosures and read_names alike (all three are
  built from the scene).
- **One-field exceptions** are kept on the thing as the internal fact
  `observe.except.<audience>.<field>` = landmark: `known` shows the value
  (at the view's precision), `heard_of` shows the line with a placeholder
  ("kept back" when the ladder has none), `gm_only` leaves the line out.
  Not inherited.
- **`set_known`** writes ONE `observe.grant` event for the whole scope
  (`this`, `children`: the thing and every descendant with its own setting
  for that audience, or `{list}`), carrying each address's prior value, so
  `undo_known` puts exactly those back as one more event. A thing already at
  the asked landmark is left alone; nothing to change writes nothing.
  `from_s` is the event's activated_at (later: waits on the clock; earlier:
  known since). An undo is activated at the later of its own date and the
  original's, so undoing a future-dated reveal before it arrives means it
  never arrives.
- **`known_to`** answers per audience (everyone as "The table", then each
  seat's audience): rung, rung_label, landmark, top, exceptions, cover
  (empty until SV5), withheld (fact fields the GM's scene holds that the
  audience is not sent a value for) and from_s (a change still waiting).
  Read off the audience's own scene of that address, so it matches the wire.
- **`known_in`** counts, per audience, the things in bounds at each landmark
  over what the GM's scene holds (the SV4 ledger).
- Intents `set_known` and `undo_known` are kernel built-ins (`at_yr`,
  `from_yr`); the chart room's `intent.ts` sends the same.

## Checks

`crates/sse-kernel/src/known_tests.rs`: a hidden base and a kept-back
population are absent from every byte of the table's scene, disclosures and
read_names; heard_of on a field sends the line, not the value; a scoped
reveal is one event and one undo; counts equal the table's scene; dated news
waits and an undo before it means it never arrives; the shared fixture
`tests/fixtures/sv/known-to.json` (chart room, PR 313): every key it reads is
answered and every request it sends is taken.

## Goldens

`tests/golden/contract.jsonl`, on purpose: the `hello` op list and the
"unknown op" error gain `known_to`, `set_known`, `undo_known` and
`known_in`; `intents_listed` gains the `set_known` and `undo_known` kinds;
four new contract cases (tests/contract/requests.json, after the Vesh
starting view on Kestrel) add four lines: a `known_to` and a `known_in` on
Tarn, and two refusals that write nothing (a bad landmark, an unknown undo).
