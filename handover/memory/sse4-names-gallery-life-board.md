---
name: sse4-names-gallery-life-board
description: SSE4 thread "Build names gallery and life board" (24 Nov 2026): NM6 parts 4-6 and U7 part 2 merged, what waits on whom
metadata:
  type: project
  modified: 2026-11-24T20:37:08.372Z
---
All merged 24 Nov 2026; status file handover/status/names-and-life.md (owned by that thread), logs handover/packages/NM6-classes.md, U7-part2.md, NM6-names-here.md, NM6-lens-names.md.

- PR 435 class gallery: Reference "Classes" page (ui-contract/src/classes.ts, views/reference/src/Classes.svelte); offers base-atlas and sse3-classic packs when none on stack. SSE4 atlas classes draw plain marks (symbologies only bind SSE3 classic ids).
- PR 450 LifeFlow board: Life screen Story/Board (ui-contract/src/life-flow.ts, views/biosphere/src/LifeFlow.svelte); dial knobs read only.
- PR 474: NamesHere block exported from views/reference; lens `names` field relabels the chart (App.svelte `shown`).

Waiting on others: kernel queue item 6 gives `class_sample {class}` and `life_view` with `dials` (kernel thread will message). Renderer thread is adding turning globes to Classes (Alex asked: every type, controllable count on screen). Galaxy thread mounts NamesHere on "Your systems" (sector chart owner is the Traveller thread). Reported to kernel: several Kestrel subjects share designation "SX 000001 1".

Related: [[sse4-naming]], [[sse4-evolution-design]]
