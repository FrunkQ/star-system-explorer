# The view controls at every scale

Made from the screens' own control specs by `tests/ui/view-controls-page.ts`; do not edit by hand (`node tests/ui/view-controls-page.ts --write`).

## What each switch means, everywhere

- **View style** says how a place is drawn. **Chart** is the diagram: readable marks, names and lines on the chart room's paper. **3D** turns the same diagram so you can look at it from any side. **True** draws what is really there in light: surfaces, stars, gas, dust, against black space. **Wireframe** is coming with the view styles.
- **Seen from** sets where the camera sits in a view that turns: **Above**, **Tilted**, **Edge-on**, or **Free** (drag to turn, as you like). A view drawn flat is always seen from above.
- **Scale** is **Readable** (sizes and distances squeezed so everything can be read at once) or **Real** (everything in true proportion, so small worlds become points and gaps become huge).
- **HUD** lays names, orbit lines, rings and outlines over the picture. **Off** leaves a photograph; **On** is a heads-up display over it. A diagram always shows its labels, so HUD is only offered where a view can look like a photograph. Pack overlays (a hex grid, territory) draw only with HUD on.
- **Band** is the light the place is seen in: **Visible**, **Infrared**, **Ultraviolet**, **X-ray**, **Radio**, or **Mag** (the magnetic field). A band other than Visible is mapped (it has no colour to see) and says how. A player's seat only gets the bands its ship's sensors carry.

The rule (Alex, 23 Nov 2026): one set of controls, and similar levels of drawing at every scale, from the cosmic web to a lava tube. Where a switch is not offered yet, the app says why on the switch itself, and the same words are below.

## What each screen offers today

✓ is offered. —n is not offered yet, for reason n below. Seen from, Scale and Band are as offered in the screen's turning style (3D or True) where it has one.

| Scale | Screen | Chart | 3D | True | Wireframe | Seen from | Scale | HUD in | Visible | Infrared | Ultraviolet | X-ray | Radio | Mag |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Cosmic web | Cosmic web (foam) | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | 3D, True | ✓ | —2 | —3 | —4 | —5 | —6 |
| Galaxies nearby | Galaxies nearby | ✓ | —7 | —8 | —1 | Above | Readable | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 |
| Galaxy | Galaxy chart | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable | 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | —9 |
| Local stars | Sector map | ✓ | —10 | —8 | —1 | Above | Readable | Chart | ✓ | —2 | —3 | —4 | —5 | —6 |
| Between stars | Voyages | ✓ | —11 | —8 | —1 | Above | Readable, Real | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 |
| System | System chart | ✓ | ✓ | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sky from a world | Window | ✓ | —12 | —8 | —1 | Above | Readable | Chart | ✓ | ✓ | —3 | —4 | —5 | —6 |
| World | Body (globe) | ✓ | —13 | ✓ | —1 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, True | ✓ | —14 | —14 | —14 | —14 | ✓ |
| On the ground | Land (surface) | ✓ | ✓ | ✓ | ✓ | Free | Real | 3D, True, Wireframe | ✓ | —15 | —15 | —15 | —15 | —16 |
| Inside a world | Cutaway | ✓ | —17 | —8 | —1 | Above | Readable, Real | Chart | ✓ | —2 | —3 | —4 | —5 | —6 |
| Side by side | Size comparison | ✓ | ✓ | ✓ | ✓ | Above, Tilted, Edge-on, Free | Readable, Real | always labelled | ✓ | —2 | —3 | —4 | —5 | —6 |
| Down a lava tube | Caves | not built yet: the kernel's lava tubes (survey_caves) drawn underground come after the walk |

### Why a switch is not offered

1. Wireframe is coming with the view styles.
2. Infrared (heat light) is not drawn here yet.
3. Ultraviolet is not drawn here yet.
4. X-ray is not drawn here yet.
5. Radio is not drawn here yet.
6. Mag (magnetic field) is not drawn here yet.
7. 3D cells come with the galaxy charts' Free view.
8. True is coming: the photoreal views are held for now.
9. The galaxy's magnetic field is not drawn yet.
10. The sector map is a flat hex grid.
11. Voyages are charted flat for now.
12. The sky is drawn as a flat map for now.
13. The globe is already 3D: Chart draws it.
14. The globe is drawn in visible light for now.
15. The ground is drawn in visible light for now.
16. The field at the ground is not drawn yet: the Globe's Mag shows the world's field.
17. The cutaway is a slice: it is drawn flat.
