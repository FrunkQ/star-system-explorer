# SC1: the star class gallery, and the star picker (Alex, 26 Nov 2026)

Alex: "we should have a gallery for them too... so we can see if they look right and you can see the actual size to
each other or scaled to same size... wanna see the cells on a overinflated stars". Then: "That gallery could be the
picker... only need this level of selection on build your own star systems and adding a new star... the option of
leaving it for the user to manually populate or be clever and guess what happened from the star birth to now".

## What
- Reference › Classes › "What kind of star is this?": every star kind as a tile drawn live in True, a whole page at
  once (12, 24 or 48), each on its own clock. Kinds as chips (families); **Same size** or **Actual size** (each star
  against the largest on the page; one too small for a pixel is a point that says how small, "1/2,300 of the
  largest"); remembered per device. A tile opens a card: the star large, with Whole star or × 2 close-up, its life in
  words, its surface (cells across, size, life, whether this size shows them, the larger cells under them, the lumpy
  limb), and what the drawing leaves out (a brown dwarf's cloud bands).
- Neutron stars and black holes are "Not drawn yet", with the reason (the exotic objects renderer), never a disc.
- Make a system › Grow a new system › **Choose the star from the gallery…** opens the same gallery as a picker. After
  picking: **Work out what happened since its birth** (the formation run with the star held at its mass, age and
  metals: `star_pins {primary_mass, count: 1}`, `inherit_pins {age_gyr, feh}`, no kernel change) or **Fill it in
  myself** (a system with that star alone added at once, for the GM to add worlds by hand).

## How
- `views/reference/src/star-gallery.ts`: each tile's star from `describe_star` (mass, age, [Fe/H]), then
  `read_activity` from its mass, radius, light and temperature (spin, spots, flares, convection). No system, no
  branch, nothing written (the state hash is checked unchanged). Clock per star: one cell's life every 4 s, or one
  turn every 30 s for a star with no cells.
- `views/reference/src/star-stage.ts`: one GPU canvas over the page, every star an item in one straight-on scene,
  faces from `liveFace` through `trueLook` (the system's True door). The canvas is black where no star is and laid
  over the tiles with `mix-blend-mode: screen`, so the page shows through and each star sits on its tile's black sky.
- The class list: the base atlas's "star" scheme (thread "Classify every star and remnant"; families with a kind per
  spectral letter, each kind's sample on its first example as `star.sample_mass_msun`, `star.sample_age_yr`,
  `star.sample_feh`), read by `tilesFromScheme`. **Until that scheme lands, `STAND_IN`** lists 18 kinds by mass and
  phase, aged to the middle of the phase (no older than the Sun where the phase allows). Captions use
  `describe_star`'s `now.class.code` once it exists, else the phase label.
- "Fill it in myself" goes through the V3 importer's door (a one-node `system.json`), the one way an authored
  system enters the chronicle today. **Seam:** a kernel op that authors an empty system round a given star would be
  cleaner; the system's source then reads `facts` rather than `v3`.

## Giants' few huge cells (the physics, not paint)
- The renderer drew only granules: ten pressure scale heights wide (kernel `read_activity`), so a red supergiant got
  about 23 across and red giants 57 to 90, all small and even. The kernel also gives a **larger scale**
  (`supergranule_size_m`, 25 granules; `network_cells`; `supergranule_lifetime_s`) that nothing drew. On a red
  supergiant it is wider than the star (2.7 cells over the whole surface), on a Sun-like red giant 16 to 33, on the
  Sun 5,580.
- Now drawn (`star-churn.ts` `churningBigCells`, `star.ts` `bigCells`, `star-now.ts`), churning like granules, at a
  temperature contrast from how fast they flow: δT/T = γ (v/c_s)² (mixing length; Böhm-Vitense 1958, unverified).
  Sun 0.5% (barely seen, as in white light), red giants about 7%, the supergiant about 7%. Fewest 4, contrast capped
  at 20%: fudge F-S31. Every True view of a star shows it (system True, Body screen, size comparison).
- `FACE_STEPS` gains 512, so a close-up of a red giant can resolve its 57 to 90 cells across (a cell is drawn from 6
  pixels). System3D keeps its own list (256 at most).

## Findings for other tracks (not changed here)
- The life line gives a 25 solar-mass star at 7.5 Myr a radius of **6,090 R☉** at 2,650 K; the largest known red
  supergiants are about 1,500 R☉ (from memory, unverified). Probably the Hurley fits past their range or the giant
  phase sketch (F-S7, F-S8). Kept out of the stand-in list; told the coordinator.
- The kernel caps the convective Mach number at 1 for every giant (`speed_m_s` = sound speed), so granule speeds and
  lives on giants are the cap's, not the physics'.

## Checks (26 Nov 2026)
- `packages/renderer/test/star-big-cells.test.ts`: contrast law, few cells on a supergiant, and a 256 px supergiant
  face whose blocks a tenth of the disc across differ by over 5% while the Sun's stay under 1%. **Seen red with the
  large cells removed** (spread 0.010 against the 0.05 floor).
- `packages/ui-contract/test/star-gallery.test.ts`: every stand-in kind is the phase its tile says, no star older than
  the universe, remnants not drawn, supergiant cells few and the Sun's many, hot stars turn instead; sizes switch;
  the atlas scheme to tiles. State hash unchanged.
- `tests/ui/star-gallery.e2e.mjs` (desk and phone, headless Chromium, WebGL2): all 17 drawable kinds drawn on one
  canvas, remnants not drawn, both sizes, the supergiant's card says its cells show, close-up; then the picker: a K
  dwarf picked, grown at 4.6 Gyr, and added alone. `tests/ui/reference.e2e.mjs` expects the star scheme last.
- Screenshots: project files `sse4/handover-screens/SC1-screens/`.
