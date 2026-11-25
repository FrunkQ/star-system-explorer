# PB5: holding cards as pack data

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design sections 6.2 and 6.3 (accepted, question 11: yes).

## What
- New pack `packs/base-presentation/pack.json` (`sse.base-presentation`, kind presentation): `provides.other.holding_cards`, schema `sse.holding-cards/1`, with the four plain cards and the design's 24 jokes (the first is Alex's). Served beside the shell by `scripts/build-shell.sh` as `packs/base-presentation.json`; it is not on the kernel stack, so no state hash or golden changes.
- `ui-contract/src/holding.ts`: `holdingCardsOn(packs)` (a later pack adds cards, or with `"replace": ["joke"]` swaps a kind for its own, so a setting's screens speak in its voice); `CardDeck` gives the same words for a card while any line holds it (every screen of a role agrees) and never the same joke twice running; `cardLabel` for the GM's side.
- `board.ts`: the fixed lists are gone; `cardWords` falls back to "Stand by" when no deck is given. `TablePort.state.cards`.
- Share: the deck is built from the base presentation pack then the stack's packs, settled on every push, and fed to `boardWire`. Before this, every change to the board drew a new joke.
- Switchboard: the line card (the select beside Release) on Everyone, each role and each screen: Plain, A joke, Dark, My own words… (typed for this moment), and every card by name in two groups. A click holds with the kind used last; a particular card or typed words are for this moment only.

## Checks
- New `packages/ui-contract/test/holding.test.ts` (4 tests). `ui:check`; `ui:test` 563 pass, 1 fail that is main's stale `reference-constants.json` (fixed by the data sheet's PR 445).
- `switchboard.e2e` now checks the base pack's plain words, a particular joke and typed words reaching the held screen. table, players, playbook and steer e2e pass.
- Screenshots: project files `sse4/handover-screens/PB5-screens/`.

## Later
- Editing the list on the look card (a fifth row, "Holding cards") comes with PB10 looks; until then a pack edits it.
- The Switchboard's line label says "held: a joke", not the joke itself.
