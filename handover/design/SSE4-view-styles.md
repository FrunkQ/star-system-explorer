# SSE4 view styles: Chart, True, Wireframe and in-universe looks

Proposed 22 Nov 2026 by the renderer thread, from Alex's steers of 22 Nov 00:45 and 00:46. The status is **proposed**. Section 2's "every seat alike" and "style per panel" rules are Alex's rulings; the numbered questions in section 6 are open.

## 1. The ask

Alex wants one view **or** the other, never the diagram laid over a realistic picture.
- A capable PC can let the GM edit in a photoreal view.
- A phone stays diagrammatic, and the app advises against turning photoreal on there.
- The choice sits in the existing 2D / 3D selector, which gains a "True" option and others such as wireframe.
- These are styles that filters can later run over to give an "in universe" look. Alex's example is Alien: low-resolution polygons on a green CRT.

## 2. The model: one setting, every scale

**View style** is a single setting. It applies at every scale: system chart, body globe, cluster, galaxy and cosmic web ("foam").

| Style | What it draws | Who it suits |
|---|---|---|
| Chart (2D or 3D) | The diagram as today: symbols, labels, honest marks | Every device; the default |
| True | Physical appearance from the renderer's recipes: real colours, light and air, stars and galaxies by their light | Capable devices |
| Wireframe | Geometry only: orbits, meshes and outlines, with no fills | Any device; also the base for the in-universe looks |
| In-universe looks (later) | A base style plus a screen filter: green CRT, Trek panel, low-poly | Player views and trailers |

A style is **data**: a `sse.view-style/1` document. It carries id, label, `needs` (the lowest tier that runs it well), a cost hint and an `in_universe` flag, and it names a recipe per scale (the existing `sse.render-recipe/1`) and an optional filter chain, so a universe pack can add its own styles. The filters work on the finished frame:
- palette quantise;
- scanlines and phosphor glow;
- pixelate;
- polygon reduction (applied to the mesh, before the frame is drawn).

**Every seat alike (Alex, 22 Nov 00:46).** In V3, view options existed only on player views. SSE4 does not make that split. The view style is a property each user chooses for themselves, filtered by what their machine can do, whether they are the GM, a player or anyone else. There is one style list and one mechanism, with no GM-only or player-only styles.

**Style per panel (Alex, 22 Nov 00:47).** A style belongs to each panel or lens in a view, not only to the whole screen. One view, a player view included, may show the same data several times through different lenses: the system as a chart in one panel and as a green-CRT wireframe in another. The user's device-level choice is the default that a new panel starts from. A panel that has been given a style keeps it, and the panel is saved with that style as part of the view. Every panel is costed against the same frame budget, so on a weak device several True panels step down together, and the quality badge says so.

## 3. Choosing and remembering

- **Where.** The 2D / 3D selector becomes one style control: `Chart 2D · Chart 3D · True · Wireframe`, plus any pack styles in a short menu. Its place on each screen is the chart room thread's call.
- **Remembered per device.** Each user's choice is stored on their own device, the same way for every seat, and does not travel with the campaign.
- **Default from the device.** Chart is always the default. True is offered but never switched on automatically.
- **Filtered by capability.** Each style carries `needs`, the lowest tier that runs it well, and a cost hint. The style control lists every style and marks the ones above this device. Picking a marked style shows the warning below. A style this device cannot run at all, such as True on a text-tier device, is shown greyed out, with the reason.
- **The warning.** On a device whose measured tier is below gpu-high, choosing True shows one plain line with two buttons: "This device draws True slowly; it will step down quality and may run warm. Chart is recommended here. [Use True anyway] [Keep Chart]". Choosing "Use True anyway" is remembered, so the line does not nag.
- **Not built yet.** Where a scale has no True recipe yet (galaxy and foam until V9), the screen shows Chart and the quality badge says "True is not built for galaxies yet". The setting itself stays on True.
- **Editing in True.** Every object stays clickable. Hover and selection show the object's name and an outline, and nothing else is drawn over the picture. Edits work exactly as in Chart.

## 4. Split between threads

| Part | Thread |
|---|---|
| Style control (device default and per panel), per-device memory, the warning line, styles saved in a view's panels | Chart room top level |
| `sse.view-style/1`, the style registry with `styles()` for the control, the filter chain, the advice function `styleAdvice(style, tier)` returning ok, warn or unavailable with its words, and True recipes at each scale | Renderer |
| True galaxy and foam (V9): 3D galaxies drawn by their stars, gas and dust, and the cosmic web as light, both reading the same kernel data the diagrams use | Renderer, with the galaxy charts thread, which keeps the diagrammatic charts (U6) |
| Wireframe at the system and globe scales | Renderer (a recipe) |
| In-universe filter packs | Renderer (filters); the packs themselves are pack work |

## 5. Order

**Held (Alex, 22 Nov 00:47): "hold off implementation until it makes most sense."** The design stands as recorded. Building starts where it falls naturally, when True covers more scales and player views are being built.


1. The style registry and `styleAdvice`; Chart, True and Wireframe on the system and globe scales, where recipes exist; the chart room wires its control.
2. True galaxy and foam (V9).
3. The filter chain and a first in-universe look, green CRT, to prove the idea.

## 6. Questions for Alex (all answered 22 Nov 2026: yes to every recommendation)

1. **Default on a capable PC: Chart or True?** Recommendation: Chart, until True covers every scale. A GM switches once and the device remembers. **Accepted.**
2. *(Answered by the ruling: each user chooses on their own device, at every seat.)*
3. **What shows over True while editing?** Recommendation: only the name and an outline on hover or selection. Honesty marks (authored, impossible, fictional) show only in the selection card. **Accepted.**
4. **Is Wireframe a style of its own, or a filter?** Recommendation: a style of its own, because it changes what is drawn and not just how the frame looks. The in-universe looks build on it. **Accepted.**
