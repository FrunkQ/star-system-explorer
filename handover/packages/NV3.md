# NV3: the object menu everywhere (V3 gap 38, continued)

Status: part 1 done 24 Nov 2026 (thread "Build object selector and menus").

## Part 1 (done): System 3D
- Each body's pin in the 3D and True views carries `data-address`, so the shell's one watcher (`shell/src/object-menu.ts`) gives it the menu on a right-click or a held press, and a lit-up tag (NV4) glows on it too.
- A right-click on the canvas near a body (not on its pin) asks for the nearest body's menu by sending `sse:object-menu` `{address, x, y}`, the rule for views drawn on a canvas. A right-click never turns the camera or flies.
- Check: `tests/ui/object-menu.e2e.mjs` (3D case: pin and canvas, frame unchanged). `tests/ui/system3d.e2e.mjs` still passes.

More's edit group (Copy, Cut, Paste here, Add here…) now takes its words and keys from `OBJECT_ACTIONS` via `actionById` (`moreWords`, `moreKey` in App.svelte), so it says what the object menu says. Checks: `clip.e2e.mjs`, `walk.e2e.mjs`.

## Part 2 (next)
- Build's tool strip (`shell/src/chart/Stage.svelte`, chart room's file: agree first) takes its labels and keys from `OBJECT_ACTIONS` the same way.
- Ships and stations: once they are drawn with an element or a canvas hit test, give each `data-address` (or send `sse:object-menu`). `menuFor` already takes any kind; the App handler (`watchObjectMenu` block in `shell/src/App.svelte`) looks the address up in `scene.subjects` and needs a branch for ship and station addresses.
- Optional tidy: move that App block into `shell/src/nav/`. It calls App's own copy, cut, paste, describe and world functions, so it moves with them or takes them as props; not worth doing alone.

Screenshot: project files `sse4/handover-screens/NV3-screens/`.
