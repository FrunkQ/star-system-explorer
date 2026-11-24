# U7 part 2: the LifeFlow board

Status: done (read only), 24 Nov 2026 (thread "Build names gallery and life board"). Part 1 is `packages/U7.md`.

## What it does

The Life screen's biosphere view gains **Story | Board** beside the lens buttons (param `show`, default `story`).
Board draws the life pipeline the kernel ran for the world as a flow of eleven stages, each wired to the stages it
takes from:

Habitats → Energy budget → Origin → Opportunity → Innovations → Ecosystem → Traits and Fluxes → Signatures → Story
hooks, with Catastrophes (from Habitats) feeding Innovations and the Ecosystem.

- Each node shows the stage's result in a few words from `life_view` (production, when life began, Earth-Myr of
  opportunity, inventions, blows and recoveries, guilds and levels, producer colour, O₂ and CH₄, candidate and
  confidence or "blind from outside", hooks) and how many dated events and dials it has. A stage with nothing for
  this world is drawn faint and dashed (a false alarm has no ecosystem; Europa keeps no timeline).
- Picking a stage (click, Enter or Space) lights its wires and neighbours and opens its card: what it does, what it
  takes from, the dated events it made with the kernel's reasons (or its results, or the hooks told in the chosen
  lens), and the dials it leans on as knobs.
- Dials sit with the stages whose events cite them (origin with Origin, oxygenic and complex cells with
  Innovations). The knobs are read only, and say why, until the kernel's `life_view` takes dial values (kernel queue
  item 6: a what-if with no events).
- Drawn at its own size so the words stay readable; a narrow desk column scrolls inside the board. On a phone it is a
  list in order, each row saying what it takes from.

Nothing is computed in the view: `ui-contract/src/life-flow.ts` only sorts the kernel's view into stages
(`lifeFlow`) and places them by their longest path in (`placeStages`).

## Checks

- `ui-contract/test/life-flow.test.ts` (real kernel): all five examples sort into the eleven stages, every event in
  one stage, every dial on a stage, wires always run forward, nodes inside the board; Earth's stages all live with
  origin and oxygenic dials, Europa blind with a hook, the desert with no ecosystem and no origin.
- `tests/ui/biosphere.e2e.mjs`: Board on the desert, Earth (eleven stages, wired, Innovations' events and read-only
  dial, Origin by keyboard), Europa's hidden-life hook, back to Story; on a phone the list and a card with no
  sideways scroll.
- Screens: project files `sse4/handover-screens/NM6-screens/u7-board.png`, `u7-board-phone.png`.

## Next

- Live knobs: when `life_view` takes `dials`, the board re-asks on a knob's change and marks what moved (kernel
  queue item 6).
- Worlds on the chart when adopted systems carry habitat graphs (as part 1).
- Habitat names in plain words belong in the kernel's reason text (the origin event still names an address).
