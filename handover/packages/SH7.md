# SH7 Names carried, part 1 (track SH)

Design: `handover/design/SSE4-spaceflight-history-design.md` section 8.5 and rulings 5 and 10 (no invented monument;
Earth memorials as text). Data: `store-packs/sol-today/memorials.json` (RS9 part 3, PR 506), named in the pack's
`provides.other.memorials`.

## Done (25 Nov)

- `packages/ui-contract/src/namescarried.ts`: `namesCarried(doc, crews)` reads the memorials in minor-planet number
  order, with who each is for (the pack's people by id), its sources, its naming date or the pack's reason there is
  none, and its orbit elements. `namedAt(c, t)` (a name is carried from its date; always, when the date is not known),
  `sinceWords`, `sharedSinceWhy` (one reason said once for the list), `carriedLine`.
- `orbitAt(elements, t)`: a two-body Kepler solve about the Sun from the pack's JPL Horizons osculating elements, in
  their frame (heliocentric ecliptic J2000). For the chart layer and "tonight" in part 2. Fudge register row "Names
  carried": no planetary pull, so it drifts from the real place away from the elements' epoch.
- The In memory page has a "Names carried" part after the Earth memorials, as text, the same whatever date is in view,
  with each memorial's sources under Sources. The 21 minor planets the pack lists show
  (the pack has no naming dates yet, so its reason is said once).
- Tests: `packages/ui-contract/test/namescarried.test.ts` (order, names, dates, the orbit against Kepler's period,
  vis-viva and the inclination; the real pack's two sources, people and orbits). `tests/ui/rollcall.e2e.mjs` checks
  the rows when the pack gives memorials. Screenshot: project files `sse4/handover-screens/SH-screens/SH7-desk-names-carried.png`.

## Next (part 2)

- The "Names carried" layer on the system chart and the Moon and Mars globes (off by default; on in the tour and when
  arrived at from In memory), each a small ringed mark with the name in the serif face, placed by `orbitAt`. The chart
  has no layer seam yet: agree one with the chart room ("Make the SSE4 test drive work end to end") rather than editing
  `chart/`.
- "Tonight" in the Sky view: needs Earth's place on the date and a constellation table (IAU boundaries, sourced), and an
  H magnitude per body for "too faint for the eye"; ask the pack thread for H with its source.
- Naming dates: the pack thread to read each from the Minor Planet Center / WGSBN citation (two sources), so a name
  appears on its day. Until then every name shows always, with the pack's reason.
- Hadley at Survey and Site (the Fallen Astronaut and plaque) waits on the pack's `object_left` rows and site levels.
