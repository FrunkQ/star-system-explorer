# PB9: screens on this computer and Split (part 1)

Thread "Build Run desk playbook and players' side", 25 Nov 2026. Design `handover/design/SSE4-playbook-and-switchboard-design.md` sections 3.2 and 15 (accepted).

## What
- ui-contract `displays.ts`: `SPLITS` (Whole, Two side by side, Three one large and two small, Four two by two, each a CSS grid), `displayLink` (a role's join link marked `local=1`, named for the display, split if asked), `partLinks` (a split window's parts: the same link unsplit, numbered `part=1..4`, each named "Wall TV 2"), `displaysFrom` and `displayMap` (the browser's screen details as plain labels and a small map in real positions), `screenKey` (one screen id per room, and one per part, since the parts share the tab's storage).
- The wire: `ScreenHello.local`, so a window on one of the GM's displays is marked "· this computer" on the board (the board already had `BoardScreen.local`).
- Players' side: `?local=1` says so in the hello. `FillScreen.svelte` offers "Fill this display" (or F) in a local top-level window, since only a press in that window may go full screen; never inside a part. `?split=two|three|four` mounts `SplitScreen.svelte` instead of a screen: a grid of frames, one players' screen per part, each joining with its own id, box, role, line, dial and hold.
- Switchboard: "On this computer…" on the Screens heading opens `run/OnThisComputer.svelte`. Choose the role; "Find the displays" asks the browser (Window Management) and draws each display as a box in its place; Open puts a window on it. Where the browser cannot or will not list them, "Open a window to drag there". The open windows are listed with Split (Whole, Two, Three, Four; changing it reloads that window split) and Close. Windows are this visit's only (`run/displays.svelte.ts`), never kept with the campaign.

## Checks
- `packages/ui-contract/test/displays.test.ts` (4 tests). `ui:check`, `ui:test`.
- `switchboard.e2e`: the browser refuses to list displays; the fallback opens a window that offers to fill the display and appears as "This computer · this computer"; Split to Three gives three frames and three boxes "This computer 1" to "3"; only the frame offers to fill; Close shuts the window. players, p2p, table and embed e2e pass.
- Screenshots: project files `sse4/handover-screens/PB9-screens/` (on-this-computer, split-three).

## Later (part 2)
- "Open in its own window" for the Switchboard itself: needs the board shared between the GM's two windows (a BroadcastChannel port for TablePort).
- The phone layout of the board is PB2's one column; nothing new here.
- PB11 (a camera in, on local screens only) builds on these windows.
