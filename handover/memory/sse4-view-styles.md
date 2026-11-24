---
name: sse4-view-styles
description: View styles (Chart 2D/3D, True photoreal, Wireframe, in-universe filter looks) are one user-chosen, per-device setting for every seat
metadata:
  type: project
---
Alex, 22 Nov 2026 00:45 to 00:46: view styles are one OR the other, never the diagram overlaid on a photoreal view. The existing 2D/3D selector becomes one style switch: Chart 2D, Chart 3D, True (photoreal), Wireframe, and later in-universe looks (filters over a base style, e.g. Alien's low-poly green CRT). A capable PC may edit in True; a phone stays on Chart and is advised against True.

Ruling: unlike V3, where view options were only on player views, the style is simply a property each user selects, filtered by their machine's capability, whether GM, player or anyone else. There is one list and one mechanism, with no GM-only or player-only styles. It is remembered per device, a new device starts on Chart, and a plain warning appears before a weak device turns on True.

Alex, 22 Nov 00:47: a style can be chosen per panel or lens. One view, a player view included, may show the same data several times, each through a different lens (e.g. a chart panel and a green-CRT wireframe panel). The device choice is the default a new panel starts from; a panel keeps the style it is given, saved with the view.

Alex, 22 Nov 00:47: hold off implementing view styles until it makes most sense (when True covers more scales and player views are being built). The design is recorded; no style switch, panel picker or in-universe looks yet.

Alex, 22 Nov 12:50, yes to the three open questions: a capable PC still starts on Chart (True is opt-in); in True, names and outlines show only on hover; Wireframe is its own style, not a mode of Chart 3D.

**Why:** SSE4 does not differentiate seats for presentation, and photoreal is heavy on weak hardware.

**How to apply:** styles are data (`sse.view-style/1`: id, label, needs tier, cost hint, in_universe, recipe per scale, filter chain), served by the renderer's `styles()`. The chart room thread owns the switch, the per-device memory and the warning. The renderer owns the registry, `styleAdvice`, the True recipes per scale and the filters. Design: handover/design/SSE4-view-styles.md. It applies at every scale, including galaxies and foam (V9 True views). See [[sse4-renderer-track]].

Alex, 23 Nov 12:55 to 12:56 (Scale switch): a second switch beside View style on every view, "Scale" with Readable (sizes and spacing exaggerated so things can be seen, what V3 called Toytown, the default) and Real (sizes and distances as they truly are; hover names and outlines find tiny worlds). Alex: "Scale/Real - great!". Labels are plain data (SCALE_WORDS in ui-contract view-controls.ts) in case he meant different words. Matters most in True; diagrams are fine on Readable. Renderer thread builds it (system True first).
Alex, 23 Nov 13:02 to 13:05 (HUD switch): one switch for all the clutter (orbit lines, names above worlds, outlines), "when off it looks like a photo, when on a heads-up display overview"; it "follows through on ALL view styles" and "at EVERY scale". Built as HUD: Off/On in ViewControls (HUD_WORDS); unchosen it is off in True and on in diagrams, then holds once chosen. Also asked for: a star in True that looks dynamic, with flares that are real (from the star's activity), queued after Scale/HUD.
