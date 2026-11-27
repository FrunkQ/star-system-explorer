# VP-A2: giants' storms, poles and curls in True

Design: `handover/design/SSE4-visual-polish-design.md` section 5.4 (accepted by Alex 24 Nov 2026, "yes to all"),
rows "Curls at band edges" and "Storms that turn". Asked again by Alex on 26 Nov ("storms and hexagons... i think they
were coming"). Status row: `handover/status/true-air.md`.

## What was wrong

The kernel already had the storms (W3: ovals, strings, dark spots, outbreaks, Jupiter's ring of polar cyclones,
Saturn's hexagon by a stated rule), but True drew a giant from the six level-0 weather tiles, 24 cells a face, about
4° a cell, painted to 128 × 64. Jupiter's largest oval came out four pixels wide and Saturn's hexagon two blurry rows.
The system view also never read the storm list. VP-A2 had been marked as waiting on VP6, which was done.

## Kernel (`sse-weather`, `sse-kernel`)

- **`weather` op, `want: "patch"`** (`lat`, `lon`, `half_km`, `n` up to 64): the same cells as the tiles, closer, on
  the azimuthal equidistant plane round a place (`rules::patch_dir`; at a pole, north is towards `lon` + 180°). Works on
  rocky worlds too (the rule block's `cell_at`). Returns `FieldPatch {lat_deg, lon_deg, half_km, n, cells}`.
- **Ovals take their shape from the shear.** `WeatherSystem.aspect` (east-west over north-south; left out when 0):
  ovals and dark spots get the steady elliptical patch in shear, ζ/ω = λ(λ − 1)/(λ + 1) (Moore and Saffman 1971, as
  Marcus 1993 uses it; derived here from the Kirchhoff ellipse's inner flow, papers not re-read: paper search is out
  until 1 Dec, so unverified). ζ is the bands' own vorticity at the storm, ω the core vorticity 2√e v/R of the weather
  block's Gaussian vortex. Shear of the opposite sense gives λ < 1, and past ζ/ω = 2√2 − 3 the ellipse is held at
  √2 − 1. `giant_cell` measures distance in that ellipse (same area). Jupiter's ovals come out 1.1 to 1.5, Saturn's
  1.3 to 1.5.
- **Backwards spinners** (design check 10): `GiantLayout.retrograde` (left out when false); every zonal wind and every
  vortex's sense is mirrored. No fixture spins backwards, so no golden moved for it.
- **The circumpolar jet edges a zone**: like the other prograde jets it sits on a zone's poleward edge, with a belt
  inside it, following its polygon (edge a quarter of the jet's deformation radius wide, pulling over two). Before
  this, Saturn's hexagon interior and the band outside it were both belt, so the kernel's polygon could not be seen.
- Tests in `giants_tests.rs`: the relation recovered from every drawn oval's aspect, plus one absolute value; an oval's
  contour is longer along the wind; a backwards spinner's winds and polar cyclone are mirrored; a patch's cells are
  the giant's own, the hexagon's mid-side edge sits at r cos(π/6), `patch_dir` bearings.
- Golden `tests/golden/weather-giants.json` re-blessed (crate golden): systems digests for Saturn and Neptune (ovals
  and dark spots gain `aspect`), field digests for Uranus and Neptune (the polar jet's zone edge at 65°).

## Renderer (`packages/renderer/src/giant-detail.ts`, new)

- `readGiantDetail` reads the day's systems, the climate's jets (K-a) and the patches `patchesFor` asks for: one round
  each pole with anything reaching it (48 cells), one round each of the twelve largest storms over three pixels (24
  cells), three of their radii out, stretched by the aspect.
- `detailGiant`: the tiles' face stretched to 1024 × 512, the patches laid over it with `paint`'s own stacking
  (`composeAt`, now shared from `true-air.ts`), feathered; each storm that is a vortex of its own (a record holding a
  smaller one at its centre is a group: a ring, a polygon) is wound by its core's winds from the patch, less the jets'
  band wind, for one shear time of the band (capped at one core turn), falling off as exp(−r²/2R²); then the curls.
- Curls: `giant.curls` through the ladder. Its amplitude binding changed from jet shear over jet speed (it blew up
  where a band's wind crosses zero) to `curl_share` = u_e / (u_e + |Δu|), worked from the jets by `curlFacts`; the
  noise is scaled so its RMS displacement is the share times the band's width, worked on a quarter-size grid, and
  left out where the patches cover (drawn detail stays below the facts). `ladder.test.ts` follows the new binding.
- `readTrueFace` makes every giant's face this way (`detail: false` turns it off, `detail_px` sets the width). Paths:
  system True (SystemChart), Body globe (body `surface.ts`), size comparison (Lineup3D, at 256 across because it
  carries every giant by its winds each frame). The flow comes at the same size, so VP-A1's carrying still works.
- Tests `packages/renderer/test/giant-detail.test.ts`: the patches asked for; a patch's bright core lands where it is
  and nothing else changes; a storm winds the way its winds go (the test goes red with the sign flipped); the share.

## Cost

Node, WebAssembly kernel, Saturn: tiles 0.7 s (as before), six patches 0.6 to 0.8 s, the finer face 0.7 s; Jupiter
(18 polar systems) patches 1.0 s, face 1.3 s. In the browser (headless SwiftShader) the first world of Sol reads in
about 10 s, as before this package's reads were added (the tiles dominate).

## Checked against reality (local build, WebGL2)

Screens in project files `sse4/handover-screens/VP-A2-screens/`. Saturn: the hexagon reads as a hexagon at 78° with a
darker interior, its polar vortex at the pole, ovals near 60°N as spirals. Jupiter: the ring of polar cyclones each
wound as a spiral. Known gaps, not this package's:
- The kernel's Jupiter has 13 jets a hemisphere-pair and narrow zones, so its ovals are 2,800 to 4,200 km; nothing like
  the Great Red Spot (16,000 km) is drawn (W3's size rule, half to nine tenths of a zone's width).
- Colours: the chromophore and NH4SH still lack measured optics (TA3, MP33).
- The drawing paths (Body globe, system True, Land) are still three; this lives in the one reader they share.

## Fudges

`docs/fudges.md`: "A giant's storms and poles read closer, and turned (VP-A2)" and "Detail ladder: giant's curls".
