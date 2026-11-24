# NM6 part 6: layer choice in lenses

Status: done, 24 Nov 2026 (thread "Build names gallery and life board"). Naming design 4.3: which name layer a chart
shows is a view setting, "common, else designation" by default, settable per lens.

## What it does

- A lens may carry `names`: the name layer that labels the chart (`designation` for a hard-science chart, `local`
  for a native's chart, or any layer a pack names). Absent means the common name, else the designation (what the
  scene already carries). `checkLens` checks it; `saved` keeps it, so it travels in lens packs.
- The lens editor (Make your own lens) has "Names" under "On the chart" (`chartNameLayers`).
- The shell relabels the scene every view draws (`shown` in App.svelte) through `namesInLayer` (read_names per
  subject; a subject with no name in that layer keeps its label) and `relabel` (labels only, nothing else). It
  re-reads on every revision, so a rename or a new naming scheme shows at once.

## Limits

- Names are read as the session's seat (the GM's view reads every layer). A player lens on the GM's desk shows the
  GM's names in that layer; on a player's own seat, read_names gives only the layers that view discloses.
- The place line and object picker keep the common names (they read the unrelabelled scene).

## Checks

- `ui-contract/test/names.test.ts`: chartNameLayers, lens check and save keep `names`, namesInLayer skips common and
  leaves out subjects without the layer, relabel changes labels only.
- `tests/ui/reference.e2e.mjs` (desk, Build): after naming Tarn's designations in Vesh, a lens "Survey names" with
  Names "Designation only" labels Tarn on the chart by its Vesh designation, and no longer as Tarn.
- Screen: project files `sse4/handover-screens/NM6-screens/names-lens-desk.png`.
