# SV7: agents and secrets (track A's half)

Design: `handover/design/SSE4-secrets-and-visibility-design.md` section 11. The Traveller pack thread does the pack half (its rungs and sensor rules).

Status: part 1 done (24 Nov 2026, agents track). Part 2 waits on the kernel's SV5.

## Part 1: Orac's bulk reveals, `packages/agents/src/orac-known.ts` (24 Nov)
- A `reveal` target: `{audience, landmark, things, field?, with_inside?, from_yr?, explain}`. The landmarks are the eye's
  three (gm_only, heard_of, known); `things` are addresses from the catalogue entry `things` (the chart's subjects).
- The check reads the kernel's own `known_to` for each thing at the campaign's now: an audience the kernel does not
  list comes back with the list it does ("everyone (The table)", then each seat's), a thing Orac was not given is
  refused, and a draft that would change nothing is refused. Checking writes nothing.
- Accept sends `set_known`, the same intent the "Known to" eye sends: one event over a `{list}` scope (one undo), or
  one per thing with `scope: "children"` when `with_inside` (one undo each; the summary says so).
- Orac's handbook (`secrecy`, `tools`), the Tars shelf and the Orac tab name reveals. A cover story is not a reveal;
  the form tells Orac to say so.
- Proved by `packages/agents/test/orac-known.test.ts` on the real kernel with the Kestrel player facts and
  `tests/fixtures/packs/tarn-observe.json` (Tarn and Vault go from Heard of to Known as one change; a wrong
  audience, an unknown thing, a bad landmark and a no-op refused; Orac's loop fixing the audience).

## Next: part 2, after the kernel's SV5 (precision `told`)
- Minerva writes from the cover for the audience and from the truth for the GM: her facts come from the audience's
  scene, so a told value is what she sees; add a check that no kept-back value (`withheld` in `known_to`) appears in
  her text for a seat, and that the GM's text names a cover as a cover.
- Orac drafts a cover story as its own target ("Told instead"), not through `reveal`.

## Files
- `packages/agents/src/orac-known.ts`, `packages/agents/test/orac-known.test.ts`
- `packages/shell/src/App.svelte` (`oracHost`: `revealTarget`, `placeEntries`), `packages/agents/src/OracTab.svelte`,
  `orac-handbook.json`, `tars.shelf.json`

## Golden hashes
None.
