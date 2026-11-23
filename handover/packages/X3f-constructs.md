# X3f: Ships and stations on the chart room (V3 gap 13)

Track: Traveller pack. Stage 5. Date 23 Nov 2026. Gap list item 13 (`/mnt/project-files/sse4/SSE4-v3-gaps.md`): "a ship is a construct record placed with Add here; its card uses the depth levels; templates are universe packs".

## What a GM can do

- With a pack that gives templates stacked (the Traveller pack has 21: ships, stations, sites), **Ships and stations** appears on the rail at system scale (Travel) and at world scale (This world). Rail `needs: ["templates"]`.
- The screen lists every construct in the system (by template kind or the `construct` tag; retired ones drop out), each with where it is.
- **Add here…** opens the library from the button, grouped by kind in the pack's own words, with each template's line of explanation. Where: the world in view by default, else the system; any body can be picked. An optional name. One click adds it (kernel `create`, at the campaign's now: facts and tags as one universe event, kept values in the record).
- The card follows the depth dial: Curious leads with up to four `construct.*` facts and the template's sentence; Science shows every fact; Engine adds the address and the kernel's bookkeeping (`template.id`, pack, version). Fictional facts carry the fictional mark.
- In edit mode a GM edits any fact in place (one `set_value` per change, so undo and history work as for any fact).
- **Take out of play…** retires it with an optional reason (kernel `retire`); nothing is deleted.
- **Open the world it is at** goes to its world.

## What changed

- `ui-contract/constructs.ts`: `ConstructTemplate`, `Created`, `byKind`, `kindWords`, `kindOf`, `constructsIn`, `factOf`, `templateOf`, `BOOKKEEPING`. Session methods `templates`, `createFrom`, `retire` (optional on `ViewSession`).
- View package `packages/views/constructs` (`sse.view.constructs`; `words.ts` for fact labels and units from the field suffix).
- Screen document `constructs.gm.json`; the shell's game screens hook is now `screens.game.ts` (GAME_VIEWS, GAME_SCREENS: sector map, codes, ships and stations).
- Kernel: no change. The stock-ships test now also reads the GM scene and finds both objects with their template ids and facts.

## Checks

- `tests/ui/constructs.test.ts`: kinds and words, finding constructs, template lookup, fact words, rail on and off and within RAIL_MAX.
- Kernel `traveller_stock_ships_are_templates_that_sense` extended.
- Not run here: browser e2e (no wasm-bindgen in this container).

## Rough edges and next

- Constructs are not yet drawn on the system chart: they have no orbit facts. A placed ship should sit on its world's orbit (or a station in orbit around it); that belongs with item 2's "Add here" on the chart (chart room thread) and the voyage join.
- Flight profile ("can it land here, and why not") waits for the travel thread's `ship_at_world` hazards (information for the GM, no rolls).
- Templates from Explorers (universe packs) arrive with X2.
- Megastructures (item 15) build on this: a construct with physics blocks attached. A design note comes first.
