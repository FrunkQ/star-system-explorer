# SV3: pencil and ink on the charts, the shared draw style (24 Nov 2026)

Owner: "Build chart pencil and Atlas look" (formerly "Build names gallery and life board"), taken over from the
renderer at the coordinator's word. Design: `design/SSE4-secrets-and-visibility-design.md` sections 2, 4 and 9.

## What

One look for "the table does not know this" on every chart, so the same secret reads the same everywhere:

- `chart-grammar/src/pencil.ts`:
  - `penOf(landmark)` gives `ink` (known), `heard` (heard of) or `pencil` (GM only).
  - `penStyle(pen, "mark" | "line" | "name")` gives inline styles, so SVG and HTML charts need no shared stylesheet:
    - pencil: a faint mark, a dotted muted line, and a muted italic name with a dotted underline;
    - heard: an outline-only mark, a thin muted line, and a muted name.
  - `penRing(pen)` gives the ring round a mark: dotted for pencil, thin solid for heard. `PEN_WORDS` gives the words for screen readers and hover.
- `chart-grammar/src/Pencilled.svelte` wraps any SVG mark with its pen and ring.
- `ui-contract/src/known.ts`: `tableLandmarks(session, addresses, at_s?)` reads the table's landmark for many things at once. Anything the kernel cannot answer for draws in ink.
- No colour carries the meaning, so the look works in the Night chart, in one-colour themes and for every colour vision.

## Where it is used now

- **Body globe** (`views/body/src/Body.svelte`): on the GM's Build and Run desks, a world the table does not know sits in a dotted ring, with "In pencil: not known to the table. The players' screens do not show it". A world the table has only heard of gets a thin solid ring. On Explore, and for players, it is plain.
- **Territory areas** (OV4, `Overlays.svelte`): a holding the table does not know has no tint, a dotted edge and a quiet name. Its key row is in pencil.

## Left to the owners of the other charts (they agreed to adopt the kit)

- The system chart and its 3D view: the chart room thread, after its PR 428. They already draw pencil, so they move onto the kit with the same look.
- Links, voyages and the sector chart: the travel thread.
- The galaxy chart and Your systems: the galaxy charts thread. The galaxy chart places no systems yet.
- The foam map has no things a GM keeps secret. Constructs already draw their list in pencil and can take `penStyle`.
- The time ruler's pencil ticks and the Layers row's eye: the chart room, with SV4 and OV8.

## Checks

- `tests/ui/known.e2e.mjs`: Ember (GM only) opens on the Run desk as a globe in pencil with its words, and in ink on Explore.
- `tests/ui/symbols.e2e.mjs`: the sample map's secret holding is drawn in pencil only on Build.
- Screens are in project files: `sse4/handover-screens/SV3-screens/known-body-pencil.png`.
