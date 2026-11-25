# NM6 part 7: marks for SSE4's own class atlas (24 Nov 2026)

Owner: the thread "Build names gallery and life board". Agreed with the chart room thread (owner of the symbologies, V12)
before touching them.

## What

The shipped symbologies bound only SSE 3 classic class ids, so a world classed only by SSE4's atlas
(`packs/base-atlas`: "rocky", "gas-giant/ammonia-clouds", "surface/temperate-ocean") drew as a plain disc on the charts
and in the Classes gallery.

`scripts/add-atlas-marks.py` adds each SSE4 class of the drawn schemes (physical: what kind of object; surface: what its
surface is like; 31 classes) as one more key in every class-keyed table of `symbology.diagrammatic.json` and
`symbology.colour-coded.json` (the mix inherits them). Its shape, shading and colour are its nearest SSE 3 class's
(rocky like terrestrial, small-body/icy like a comet, surface/ice-shell-ocean like subsurface ocean), so both atlases
read alike; its code is its own (Rk, GaA, sTO...), so a one-colour chart still tells every SSE4 class apart.
`physical/ringed` draws a ring as `planet/ringed` does. `symbology.names.json` gains the plain names from the atlas, and
`v3-colour-tokens.json` lists each SSE4 class under its twin's V3 token so a V3 palette recolours both atlases alike.
Legends keep their hand order; an SSE4 class present on a chart joins the end.

The air and habitability schemes are named in the script as not drawn (a world's air is on its card and in the air
lens; who can breathe there is colour-coded's habitability tag colour). The script only adds keys; the documents carry
hand edits `make-symbologies.py` does not know, so that script is not run again.

On a body with several SSE4 classes the first one the tables know draws it (as before): physical classes come first.

## Checks

- `scripts/add-atlas-marks.py --check` fails when a base class of a drawn scheme has no mark, when the table names a
  class or scheme the atlas lacks, when two codes clash, or when the documents are out of step. Run by
  `ui-contract/test/symbology.test.ts`, so CI runs it with ui:test.
- `symbology.test.ts`: every drawn SSE4 class has a look in every diagrammatic class table, takes its twin's shape,
  shading and colour with its own code, rings on physical/ringed, is told apart from every other under every theme
  (diagrammatic and the mix; colour-coded under one-colour themes), and has a name.
- `tests/ui/reference.e2e.mjs`: no two physical tiles in the Classes gallery draw alike. The tiles were widened so a
  three-letter code is not clipped.
- Screens: project files `sse4/handover-screens/NM6-screens/classes-desk.png`.

## Next

- A new SSE4 class: add a line to MARKS (or its scheme to NOT_DRAWN) and run the script; the check says so otherwise.
