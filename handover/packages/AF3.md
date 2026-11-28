# AF3: True as the front door, made quick and keyboard-reachable

27 Nov 2026, follow-up to AF2 (a first visit opens in True). The coordinator asked for both gaps AF2 reported.

## The Solar System's first open in True

Measured on the linked PC's Claude browser pane (AMD Radeon 890M, WebGPU; the Chrome extension was not connected, so
not the RTX 4070 machine), on the live site with True kept as the view: **13.6 s** from pressing "Open the Solar
System" to the place line naming it, against **0.27 s** with Chart. Headless Chromium (software WebGL): 27.6 s.

Cause: the True view starts reading every world's face (surface tiles, air fields, glint) of the base system as soon
as the page opens, behind the start sheet where nobody can see it. The kernel is one worker answering in order, so
the door's 42 catalogue batches queued behind those reads. The main thread was idle most of the time (profile: 16 s
idle of 27.6 s).

Fix: `ws.held` (Workspace, view.ts), set by the shell while the start sheet is up or a door is entering a system. The
True face reader (SystemChart.svelte) waits while held, and stops between worlds once superseded. Headless now 1.4 s.
Gate: `fresh.e2e.mjs` records the page's kernel calls and asserts no face, weather or sky read runs behind the start
sheet; seen red with the hold removed (a `resolved_system` tile read appeared).

Not re-measured on the PC: the fix is not deployed yet. Software graphics (27.6 s before) is noted, not a blocker.
Other True reads (sky, small bodies, rings, lumps) still start on their own; they are light and were not in the way,
but they could take the same hold if they ever are.

## Planets reachable with Tab in True (and 3D)

The 3D view's body pins are buttons, but with the HUD off (True's default) their names are `visibility: hidden`, so
the buttons had no accessible name: Tab landed on nameless buttons and screen readers said nothing. Each pin now has
`aria-label` (its name, and "not known to the table" under the pencil). Enter chooses it, as on Chart.
Gate: `fresh.e2e.mjs` tabs to Earth by name in True and chooses it with Enter; red without the label.

## Checks

`ui:check` clean; fresh, system3d and walk e2e pass in Chromium.
