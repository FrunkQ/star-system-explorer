# NV1: A menu on every object (V3 gap 38)

Status: done 24 Nov 2026 (thread "Build object selector and menus"). Accepted by Alex on 24 Nov ("yes to all questions" on the second V3 audit, `handover/design/SSE4-v3-gaps.md` item 38).

## What it does

- **Right-click, or press and hold on a touch screen, on any object** opens its menu at the pointer, growing from the object (`opensFrom`). On a phone it is a sheet along the bottom with 44 px rows. The head names the object and what it is ("planet · orbits Kestrel"). The object is selected too, as a press does.
- **One action list.** `ui-contract/src/actions.ts` (`OBJECT_ACTIONS`, `menuFor`) holds every action on an object: its words, group, desks, the kinds it fits, its key, and why it cannot be done now. The shell runs each by id with the same function as the tool, key or More entry that already did it (`runAction` in App.svelte). Add an action here once and the menu has it.
- **Desk-aware.** Explore: Open, Describe, Copy a link, Show the players, Copy, Land. Build: Open, Describe, Copy a link, Copy, Paste here, Add here (stars and planets), Cut. Run: Open, Describe, Copy a link, Show the players, Voyage from, Land on. Belts and zones are areas: only Describe.
- **Greyed with the reason, in words under the item** (a phone has no hover for a title): "Share with the players first (on the Run desk)", "Only a body added to this campaign can be cut; the sample's own come from its file", "Keep or throw away the what-if first".
- **Other desks in one quiet line each** at the foot ("On Build: Paste here, Add here and Cut ›"); pressing it switches desk.
- **Keyboard:** the first item that can be done takes the focus; arrows, Home and End move; Enter picks; Escape closes and gives the focus back.

## One general rule for what is an object

`shell/src/object-menu.ts` watches the whole window. Any element carrying `data-address` stands for an object (the system chart's bodies today). A view that draws on a canvas, with no element per object, sends a bubbling `sse:object-menu` event `{ address, x, y }` instead. Shift + right-click leaves the browser's own menu alone; text fields and the Known to eye (which answers a right-click itself) are left alone too.

## Checks

- `packages/ui-contract/test/actions.test.ts`: items per desk, greyed reasons, the other-desk line, areas.
- `tests/ui/object-menu.test.ts`: the menu stays on screen.
- `tests/ui/object-menu.e2e.mjs` (added to `ui:e2e`): right-click on the chart on each desk, greyed Cut does nothing, the desk line switches desk, Open goes to the world, Night chart, a held press on a phone opens the sheet, nothing runs off the side, and menus write nothing (state hash unchanged).
- place, known, shell and walk e2e still pass.

Screenshots: project files `sse4/handover-screens/NV1-screens/` (Explore, Build and Night chart on a desktop; the phone sheet).

## Not yet (next packages in `status/navigation.md`)

- Objects other than system bodies: the selector's rows, ships and stations once they are drawn on the chart, systems and galaxies above a system, and System 3D (it should send `sse:object-menu` from its raycast).
- Build tools and More still have their own buttons; they should read their words and keys from `OBJECT_ACTIONS` too.
- Run-desk time actions (next eclipse and the like) wait for those features.
