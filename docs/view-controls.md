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
| Cosmic web | Cosmic web (foam) | ✓ | —1 | —2 | —3 | Above | Readable, Real | always labelled | ✓ | —4 | —5 | —6 | —7 | —8 |
| Galaxies nearby | Galaxies nearby | ✓ | —9 | —2 | —3 | Above | Readable | always labelled | ✓ | —4 | —5 | —6 | —7 | —8 |
| Galaxy | Galaxy chart | ✓ | ✓ | ✓ | —3 | Above, Tilted, Edge-on, Free | Readable | 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | —10 |
| Local stars | Sector map | ✓ | —11 | —2 | —3 | Above | Readable | Chart | ✓ | —4 | —5 | —6 | —7 | —8 |
| Between stars | Voyages | ✓ | —12 | —2 | —3 | Above | Readable, Real | always labelled | ✓ | —4 | —5 | —6 | —7 | —8 |
| System | System chart | ✓ | ✓ | ✓ | —3 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, 3D, True | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Sky from a world | Window | ✓ | —13 | —2 | —3 | Above | Readable | Chart | ✓ | ✓ | —5 | —6 | —7 | —8 |
| World | Body (globe) | ✓ | —14 | ✓ | —3 | Above, Tilted, Edge-on, Free | Readable, Real | Chart, True | ✓ | —15 | —15 | —15 | —15 | ✓ |
| On the ground | Land (surface) | ✓ | ✓ | ✓ | ✓ | Free | Real | 3D, True, Wireframe | ✓ | —16 | —16 | —16 | —16 | —17 |
| Inside a world | Cutaway | ✓ | —18 | —2 | —3 | Above | Readable, Real | Chart | ✓ | —4 | —5 | —6 | —7 | —8 |
| Side by side | Size comparison | ✓ | —19 | —2 | —3 | Above | Readable, Real | always labelled | ✓ | —4 | —5 | —6 | —7 | —8 |
| Down a lava tube | Caves | not built yet: the kernel's lava tubes (survey_caves) drawn underground come after the walk |

### Why a switch is not offered

1. The cosmic web is a slice for now: 3D comes later.
2. True is coming: the photoreal views are held for now.
3. Wireframe is coming with the view styles.
4. Not seen in infrared here yet.
5. Not seen in ultraviolet here yet.
6. Not seen in x-ray here yet.
7. Not seen in radio here yet.
8. Not seen in Mag here yet.
9. 3D cells come with the galaxy charts' Free view.
10. The galaxy's magnetic field is not drawn yet.
11. The sector map is a flat hex grid.
12. Voyages are charted flat for now.
13. The sky is drawn as a flat map for now.
14. The globe is already 3D: Chart draws it.
15. The globe is drawn in visible light for now.
16. The ground is drawn in visible light for now.
17. The field at the ground is not drawn yet: the Globe's Mag shows the world's field.
18. The cutaway is a slice: it is drawn flat.
19. The lineup is drawn flat, side by side.
