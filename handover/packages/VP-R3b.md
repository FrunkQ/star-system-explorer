# VP-R3 part 2 and VP-R2: one ring per world, with its own structure and light

Track SB (visual polish design 5.9, section 12), thread "Give planetary rings real structure", 26 Nov 2026.

## Why
Alex, looking at Saturn in True (shadows thread, PR 617): "those rings are far too uniform... we did have a plan to mix
them up a bit?" Then: "rings with real structure should be 1 ring! but it can describe all its own subrings"; "generate
plausible and they can pick the one they want; in initial gen we just pick the one our generator says to be
consistent"; and "any way for a crazy cult to paint a ring of Saturn red (John Varley)?".

What the code did before: Saturn was five catalogue bands (D, C, B, Cassini Division, A), each one flat optical depth
and one albedo, so each band drew as an even stripe. The resonance cuts lived in the renderer (`ring-structure.ts`),
and with only Saturn's major moons in base-sol they cut nothing. Generated giants did get a ring in formation (F5,
Canup 2010), but `cloudSubjects` skipped clouds round a world, so no generated ring was ever drawn.

## What was built
- **Kernel** `crates/sse-small/src/ring.rs` (`structure`), settings in `packs/base-small-bodies` (`rings`), op
  `ring_structure` (`crates/sse-kernel/src/small.rs`, protocol, ops cost row). One pure function of its ask: the
  host (mass, radius, J2 or flattening and spin), the sculptors (moons; later a star's planets round a belt), the ring
  (edges, mass, makeup, grain size, contaminant, zones), a `variant`, a sample count and a window. Or `{system,
  cloud}` for a ring formation made: the kernel builds the ask from the run (the world, its moons, the ring's three
  largest members through `Cloud::resolve`).
  - Zones: catalogued or authored ones from the ask (authored mass shares renormalised to the ring's mass), else 2 to
    5 seeded zones of differing density holding the cloud's mass. Each zone's surface density from its optical depth
    and grain size, so zone masses sum to the ring's mass.
  - Marks: first-order inner Lindblad resonances solved with J2 (bisection on m(Omega - n_s) = kappa); an edge where
    the resonant torque beats the viscous torque (Tajeddine et al. 2017); otherwise a density-wave train, wavelength
    from the dispersion relation (Tiscareno et al. 2007 eq. 5) and damped by the viscosity (Shu 1984); a gap round
    each embedded moon big enough to clear one, with its edge waves' wavelength (3 pi x half-width) and amplitude.
  - Profile: optical depth per sample with four sub-samples per pixel: the zone's depth times a seeded fractal kept
    inside the zone's measured range, times each wave train (averaged over the pixel, so trains finer than a pixel
    average out), zero in gaps. Octaves finer than two samples are left out and the sum is normalised over all of
    them, so a finer window keeps the coarse shape and adds detail.
  - Colour: water ice dirtied by a sheet contaminant (default titan-tholin at 1 micrometre), mixed by cross-section in
    single-scattering albedo (Hapke); the contaminant's albedo from its refractive index by the equivalent slab. A
    catalogued albedo solves for the dirt share; a makeup gives it directly. A zone painted with fine hematite turns
    red (the Varley case).
  - Phase numbers for the drawing, and each zone's dust share from its depth.
  - Marks name their sculptor by the ask's id (`sculptor`, which survives a GM renaming the moon) and by name
    (`sculptor_name`, for the words); the profile editor thread (RB3a) reads the id.
- **Scene**: `resolved_system` clouds carry `around_address` for rings; `make.ts` hangs a ring subject on its world;
  Sol's ring facts carry their printed depth range (`ring.optical_depth_low` and `_high`).
- **System True**: `views/system-3d/src/ring-read.ts` asks the kernel once per system and set of rings for every ringed
  world (8,192 samples); `SystemChart.svelte` attaches the answer to the body; `true-body.ts` draws the kernel's
  profile instead of the bands; `rings.ts` `ringLook` lights each pixel (VP-R2), and `gpu.ts` shows it through a
  second, premultiplied texture refreshed when the angles move. The profile texture (face-on cover in alpha, what
  PR 617's ring shadow reads) is unchanged in kind.
- Removed `renderer/src/ring-structure.ts` and its test: the physics now lives once, in the kernel.

Drawing paths: system True only. The size comparison and the class gallery still draw rings from their bands (old
path, fudge "Rings in True"); the Body globe and Land draw no rings. The merge of the three drawing paths should take
the kernel ring through.

## Evidence
- `crates/sse-small/src/ring_tests.rs`, 11 pass, against Saturn with recalled moon masses (unverified until paper
  search is back, 1 Dec 2026): Mimas 2:1 moves onto the B ring's edge with J2 (outside 0.4% without, inside 0.2%
  with) and holds it at a torque ratio above 10; Pan clears a 327 km Encke gap with edge waves every 1,542 km and
  about 1.7 km amplitude, and the profile is clear there; Daphnis clears 42 km; Janus 2:1 to 7:6 launch wave trains
  that start 18 to 32 km long and die away over 170 to 440 km; the B ring's depth spans more than a factor of two
  inside its catalogued 0.4 to 2.5; zone masses sum to the ring's, 2.3e19 kg against Cassini's 1.54e19 kg (recalled);
  albedos held to 0.02 and grains redder than blue; hematite paint reddens a band, thicker paint more; variants change
  only the seeded part; a finer window keeps the coarse mean within 25% and adds structure; a generated ring gets
  seeded zones holding its mass.
- `crates/sse-kernel/src/small_tests.rs`: a formed ring (seed kernel-form-81) is one ring with zones hung on its world
  by address, the same every time, and another variant keeps the marks; an ask by stated inputs answers and a bad one
  says why.
- Contract area `small-bodies`: three cases appended (a Saturn-like ask; a ringed system adopted as sys:2, seed
  kernel-form-21; its ring by cloud). Only `tests/golden/areas/small-bodies.jsonl` moved (3 lines added).
- `packages/renderer/test/rings.test.ts`: from the lit side the densest band is brightest, from below it goes dark
  and a middling one glows; the surge is about a third at zero phase; backlit dust is over ten times brighter than
  front-lit. `tests/ui/system3d-ring-moons.test.ts`: the view's ask, a formed ring by cloud, and True drawing the gap.

- Screens (project files `sse4/handover-screens/VP-R3b-screens/`, SwiftShader, 1440x900 at 2x): `before-*` from main
  at 4db47a8f, `after-*` from this branch, `saturn-before-after.png` crops side by side. The kernel ring is greyer
  (grain physics instead of one tan albedo) with fine radial banding inside each band; the Cassini Division is kept.
  The "below" shots did not get the camera under the ring plane (the drag only spun it), so the unlit side is shown by
  `rings.test.ts`, not by a screen. No Encke or Keeler gap yet: Pan and Daphnis are not in base-sol.
- Golden: only the small-bodies area moved (3 lines added, re-blessed after marks gained `sculptor_name`); no core
  contract line.

## Next
- Saturn's small moons (Pan, Daphnis, Atlas, Prometheus, Pandora, Janus, Epimetheus) and J2 on base-sol rows, from
  the NSSDCA sheets (the cloud session could not reach nssdc.gsfc.nasa.gov): until then the app's Saturn shows its
  bands' fine structure and the Janus-less resonances only, no Encke or Keeler gap.
- Fold Sol's ring bands into one subject in lists and cards (the drawing and the kernel's answer are already one).
- Wavy edges drawn round the ring (2D texture near an embedded moon), second-order resonances (Mimas 5:3), a finer
  window as the camera closes in, particle sizes (K-c) and phase tables (K-e) to replace fudges SB-ring-dust and
  SB-ring-phase.
