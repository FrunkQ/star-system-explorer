# PB8: the clean output link

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 9, question 13 (accepted: OBS out in 4.0 as an ordinary screen on an ordinary role, with a transparent option).

## What
- `seats.ts`: `outputLink(join, clear?)` adds `clean=1` (and `name=Output` unless named) to a role's join link; `clear` adds `ground=none`.
- Players' page with `?clean=1`: no bar, ask, sound bar, footer text, "Put away" or steering mark; the page is a 16:9 picture letterboxed in its window, with no pointer. `&ground=none` makes the page and body transparent for an OBS browser source.
- It joins its role like any screen: the hello carries `output: true` (`ScreenHello.output`, `BoardScreen.output`), and the Switchboard marks the box "Output". It takes lines, holds and steering like any other screen; viewers see only what its role knows.
- Switchboard: under each role while on air, "Stream <role>: output link · clear ground", each copying its link.

## Checks
- `board.test.ts`: one new test (16 pass). `ui:check`.
- `switchboard.e2e`: the role offers the link; an output screen opened at 1600 by 1000 has no controls, is 16:9 and has a clear ground, and shows as Output on the board.
- Screenshots: project files `sse4/handover-screens/PB8-screens/`.

## Later
- The renderer's own transparent ground for the WebGPU stage (the 3D and True views still paint their own sky).
- A layout made for streams (one big picture) comes with PB13 layouts; today the output shows the role's ordinary screen document.
