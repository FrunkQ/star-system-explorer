# GZ5: Galaxies nearby rebuilt (grand zoom design section 7)

Owner: "Build galaxy charts and histories". Design: `handover/design/SSE4-grand-zoom-design.md` sections 7.2 to 7.6,
package GZ5 in section 13 (Stage B, after the kernel tie-up, before the zoom).

## Part 1 (25 Nov 2026)
- The cluster map's shared view controls are now `CLUSTER_CONTROLS` (`views/cluster-map/src/controls.ts`): Chart, 3D;
  Above, Tilted, Edge-on (Free comes with True); Readable and Real; HUD in 3D. Chart is today's cube-by-cube
  catalogue, unchanged, with Find a home. 3D is `Nearby.svelte`.
- `Nearby.svelte` reads the kernel's sky once (`sky_from_here` with `galaxies: true, stars: false`): every galaxy
  `galaxies_seen` knows within 12 Mpc of home plus home's satellites from `satellites_seen`. Measured in node on the
  default universe: 427 galaxies, about 4 s. Each galaxy is placed at its direction from home times its distance
  (`nearbyAt`), so this view and the night sky agree; drawn by its own facts: colour from B-V (Ballesteros 2012 to a
  temperature, the chart's star ramp), half-light radius, axis ratio and position angle as seen from home.
- Two reaches: the group (1.5 Mpc) and the neighbourhood (12 Mpc). Real draws true sizes (a point of light at least);
  Readable multiplies every size by one stated factor (8 in the group, 20 in the neighbourhood), said on a chip.
- HUD: names (biggest first, none overlapping), dark-halo r200 rings scaled from home's own r200 (kernel satellite
  walk) by the cube root of mass, home's halo ring, scale bar in light years. Stalks to our disc's plane: all in the
  group, galaxies of 10^10 suns of stars or more in the neighbourhood.
- The card: type in words, stars and star formation, dark halo, distance and light-travel time, half-light radius,
  a satellite's orbit words, and "Chart this galaxy" for a catalogued halo.
- Counted, not drawn: satellites torn into streams or shells and those merged into home; haloes under the
  catalogue's floor.
- Pure code in `ui-contract/src/galaxies-nearby.ts`, tested in `galaxies-nearby.test.ts`; e2e step in
  `tests/ui/cluster.e2e.mjs` (3D, both reaches, several types, the card, Edge-on and Real). Screens:
  project files `sse4/handover-screens/GZ5-screens/`.

## Part 1b: bound pairs (25 Nov 2026)
- Under HUD, a dashed line joins two galaxies bound to fall together, with "meet in about N Gyr" on it and on the
  card ("Bound to"). Read from the kernel's `halo_cell` for each cell holding a galaxy on view (each halo's nearest
  big neighbour, `bound`, `approach_gyr`), one cell at a time in the background, cached; `boundPairs` keeps one line
  per pair with both on view. The e2e finds 24 pairs in the neighbourhood of the default universe.

## Part 2 (26 Nov 2026): True with type pictures, Free
- **True** (the View style control; was held): space in black, each galaxy in its own light. A galaxy a few pixels
  across or more is its **type picture**, made by rule in the new `renderer/src/galaxy-types.ts` from the kernel's
  facts (type, half-light radius, axis ratio and position angle as seen from home, B-V colour, star formation as
  blue knots by the True galaxy's own young-light share; Sersic bulge and disc, log-spiral arms, a bar, keyed
  lumps; fudge GZ-F14). Smaller ones are glowing points in their colour. Brightness on the page by absolute V.
  **Our own galaxy is its real face** from the kernel's galaxy chart (`galaxyFace`, the same picture Galaxy 3D's
  True draws), laid in our disc's plane and turned with the view. HUD is off by default in True and brings back the
  plane, stalks, names, rings, pairs and the edge. A chip says what is drawn and that it is visible light only.
- **Free** (Seen from): drag the map to turn it about our disc's axis and tip it from above to edge-on
  (`turnOf`, `projectNearby` with a turn; starts at the Tilted view). Works in 3D and True.
- Tests: `renderer/test/galaxy-types.test.ts` (b_n absolute values, half an elliptical's light inside R_e by
  integration, tilt and turn, knots only where stars form, keyed patterns, colour); `galaxies-nearby.test.ts` (Free
  matches the fixed views at their turns). E2e: True in the group with type pictures and our real face, Free
  dragged. Screens: project files `sse4/handover-screens/GZ5-screens/` (part 2 names start `gz5-nearby-true`).
- `docs/view-controls.md` regenerated (True and Free now offered here).

## Honest limits
- Shapes are drawn as seen from home (the sky's tilt), not from an axis in space (GZ14 gives one).
- One place per halo is not yet true: the web cube moves haloes by the Zel'dovich step, `galaxies_seen` does not
  (grand zoom gap 4, GZ15).
- ~~The sky is read at the present day~~ Corrected 26 Nov (GZ6 part 1): the kernel reads the sky at the campaign's date by default; the chip now says so.
- Galaxy names are catalogue labels ("Halo 2, cell 0.-1.1") until the naming track gives galaxies a scheme.
- In 3D, space is an inverted panel (ink ground) so the theme tokens hold; True draws it black.
- The shell's Free hint mentions sliding and zooming; this view turns and tips only (zoom is the reach buttons).
- Bands: True draws visible light only; the kernel's per-band magnitudes are not used yet.

## Next
GZ6 part 3: the biggest neighbours' real faces read in the background in place of their type pictures ("Histories read: 2 of 3"), the web's threads behind. Bands in True.
