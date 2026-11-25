# The view controls at every scale

Made from the screens' own control specs by `tests/ui/view-controls-page.ts`; do not edit by hand (`node tests/ui/view-controls-page.ts --write`).

## What each switch means, everywhere

- **View style** says how a place is drawn. **Chart** is the diagram: readable marks, names and lines on the chart room's paper. **3D** turns the same diagram so you can look at it from any side. **True** draws what is really there in light: surfaces, stars, gas, dust, against black space. **Wireframe** is coming with the view styles.
- **Seen from** sets where the camera sits in a view that turns: **Above**, **Tilted**, **Edge-on**, or **Free** (drag to turn, as you like). A view drawn flat is always seen from above.
- **Scale** is **Readable** (sizes and distances squeezed so everything can be read at once) or **Real** (everything in true proportion, so small worlds become points and gaps become huge).
- **HUD** lays names, orbit lines, rings and outlines over the picture. **Off** leaves a photograph; **On** is a heads-up display over it. A diagram always shows its labels, so HUD is only offered where a view can look like a photograph. Pack overlays (a hex grid, territory) draw only with HUD on.
- **Band** is the light the place is seen in: **Visible**, **Infrared**, **Ultraviolet**, **X-ray**, **Radio**, or **Mag** (the magnetic field), and last **Composite**: several bands laid into one picture, as observatory press images are, chosen from a tick list with presets (each band scaled to its own brightest; the legend lists every layer and its colour). A band other than Visible is mapped (it has no colour to see) and says how. A player's seat only gets the bands its ship's sensors carry, in Composite too; a phone lays at most two bands at once.

The rule (Alex, 23 Nov 2026): one set of controls, and similar levels of drawing at every scale, from the cosmic web to a lava tube. Where a switch is not offered yet, the app says why on the switch itself, and the same words are below.

## What each screen offers today

✓ is offered. —n is not offered yet, for reason n below. Seen from, Scale and Band are as offered in the screen's turning style (3D or True) where it has one.

| Scale | Screen | Chart | 3D | True | Wireframe | Seen from | Scale | HUD in | Visible | Infrared | Ultraviolet | X-ray | Radio | Mag | Composite |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Cosmic web | Cosmic web (foam) | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | 3D, True | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| Galaxies nearby | Galaxies nearby | ✓ | —8 | —9 | —1 | Above | Readable | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| Galaxy | Galaxy chart | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable | 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | —10 | ✓ |
| Local stars | Sector map | ✓ | —11 | —9 | —1 | Above | Readable | Chart | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| Between stars | Voyages | ✓ | —12 | —9 | —1 | Above | Readable, Real | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| System | System chart | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sky from a world | Window | ✓ | —13 | —9 | —1 | Above | Readable | Chart | ✓ | ✓ | —3 | —4 | —5 | —6 | —14 |
| World | Body (globe) | ✓ | —15 | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, True | ✓ | —16 | —16 | —16 | —16 | ✓ | ✓ |
| On the ground | Land (surface) | ✓ | ✓ | ✓ | ✓ | Free | Real | 3D, True, Wireframe | ✓ | —17 | —17 | —17 | —17 | —18 | —7 |
| Inside a world | Cutaway | ✓ | —19 | —9 | —1 | Above | Readable, Real | Chart | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| Side by side | Size comparison | ✓ | ✓ | ✓ | ✓ | Above, Tilted, Edge-on, Free | Readable, Real | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 | —7 |
| Down a lava tube | Caves | not built yet: the kernel's lava tubes (survey_caves) drawn underground come after the walk |

### Why a switch is not offered

1. Wireframe is coming with the view styles.
2. Infrared (heat light) is not drawn here yet.
3. Ultraviolet is not drawn here yet.
4. X-ray is not drawn here yet.
5. Radio is not drawn here yet.
6. Mag (magnetic field) is not drawn here yet.
7. Composite needs two bands drawn here.
8. 3D cells come with the galaxy charts' Free view.
9. True is coming: the photoreal views are held for now.
10. The galaxy's magnetic field is not drawn yet.
11. The sector map is a flat hex grid.
12. Voyages are charted flat for now.
13. The sky is drawn as a flat map for now.
14. Composite is coming to this view: it draws one band at a time for now.
15. The globe is already 3D: Chart draws it.
16. The globe is drawn in visible light for now.
17. The ground is drawn in visible light for now.
18. The field at the ground is not drawn yet: the Globe's Mag shows the world's field.
19. The cutaway is a slice: it is drawn flat.
