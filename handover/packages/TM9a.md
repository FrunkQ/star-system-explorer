# TM9a: Fill the screen on the Run desk (track TM)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 2 (the table's Fill the screen row) and question 11 (accepted 25 Nov 2026).

## What it does

- On **Run**, **F** fills the screen: the picture alone. The bar slides up out of sight, and the clock, the status line, the view controls, the tools, the card, the key and the zoom go. The browser goes full screen too where it allows it (a refusal leaves the window filled anyway).
- Moving the pointer to the **top edge** (an 8 px strip) slides the bar back; leaving the bar slides it away again. The place line, Out and In, the desk switch, See as, Ask and More all work from there.
- **F** again or **Esc** brings everything back. Leaving full screen in the browser, or switching to Explore or Build, does too. A short line says so for the first moments ("The picture alone. F or Esc brings everything back; the bar is at the top edge.").
- On Explore and Build, F does nothing (the key is free for them).
- New shell part `fill-screen` (desks: Run) in `SHELL_PARTS`; the shell asks `partShows("fill-screen", desk)`. The keyboard page lists F and Page Up, Page Down (Run desk).
- Reduced motion: the bar appears and goes without the slide.

## Left for later

- The system chart keeps its usual centre (a little left of middle, leaving room for the card) when the card is gone. Centring the picture when filled is a small change in the chart's view spec, for the chart room.
- A window the GM opened for a display (design section 2, "F, on a window the GM opened for a display") is the table's, not this desk's.

## Checks

- `tests/ui/stepper.e2e.mjs` gains a Fill section: F on Run fills (no clock, no view controls, the bar above the top edge, the picture the whole window); the pointer at the top edge brings the bar back; Esc ends it; F on Explore does nothing.
- The picture check was seen red with the grid fix removed (the stage slid into the bar's empty row and drew at 0 px high), green with it.
- `desk.test.ts` lists the new part.
- Screens: project files `sse4/handover-screens/TM9a-screens/` (4 filled, 5 the bar back at the top edge).
