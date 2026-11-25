# SV3 on the travel screens: pencil and ink on Voyages and the Sector map (25 Nov 2026)

Owner: "Build galaxy charts and travel screens" (travel half). The kit is the pencil thread's (PR 487, `handover/packages/SV3.md`):
`penOf`, `penStyle`, `penRing` and `Pencilled` in chart-grammar, `tableLandmarks` in ui-contract `known.ts`.

## What

On the GM's Build and Run desks, what the table does not know is drawn in pencil on the travel screens, in the same
look as every other chart. Explore and the players' screens draw in ink only.

- **`travelPens(session, places, withLinks?)`** (ui-contract `party.ts`) gives each place's landmark for the table,
  plus the ids of links the GM's record holds that the table ("everyone") is not shown. It compares `links_in` with and
  without the audience, so no new kernel op is needed. A place the kernel cannot answer for is left out and drawn in ink.
- **Voyages in a system** (`views/voyage/src/InSystem.svelte`): every world on the orbit line sits in `Pencilled`, so it
  is faint in a dotted ring if the table does not know it, or outlined in a thin solid ring if the table has heard of it.
  Its name gets `penStyle(..., "name")`. A leg arc that reaches a pencilled world is a dotted pencil line.
- **Between stars** (`Stars.svelte`): this asks only about the voyage's own stars (the start, the pick and the legs' ends),
  so a crowded cell costs a handful of calls. Those stars' dots get their pen. A leg is in pencil when it reaches a
  pencilled star or runs along a link the table is not shown (`VoyageLeg.link`, now typed; the kernel already sent it).
  The leg's line in the ledger is pencilled to match.
- **Sector hex map** (`views/sector`, `views/hex-map`): `HexScene.pens` (by hex) wraps a world's glyphs in `Pencilled`
  and pencils its name. The Sector desk asks only about the shown subsector's worlds (by their system address).

## Checks

- `tests/ui/party.test.ts` "travel pens": each landmark, an unanswerable place left to ink, and a hidden link found.
- `tests/ui/travel-pencil.e2e.mjs` (added to `ui:e2e`): on the sample campaign's Run desk, Ember (GM only) is pencilled
  on the orbit line, with a dotted ring and a pencilled name. Tarn (heard of) gets a thin solid ring. The leg to Ember is a
  pencil arc. On Explore nothing is pencilled.
- The Sector map's pens have no browser check yet. The sample sector's worlds are all known to the table, so a check
  needs a hidden world first (next).

## Next

- A Sector browser check with a world hidden from the table.
- The galaxy chart and Your systems (the galaxy thread) and the routes they draw use the same `travelPens` link set.
