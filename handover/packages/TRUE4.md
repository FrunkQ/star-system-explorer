# TRUE4 part 1: surface stains, hued from the data sheet's optics

Track TRUE-surface, third V3 audit R15 and R16 ("tholins and coloured frosts"), with the coordinator's Mars finding (Mars reads grey-brown, not red). One mechanism for all of them: the kernel says what coats the ground, and the data sheet's measured optics give its colour. No colour is typed.

## What
- `crates/sse-surface/src/stains.rs` (proven core reviews it; cover.rs only reads it): a world's stains as process strengths. Each is `{kind, material_id, share, why}`.
  - Rust (`mgs-1`) from oxidising air (O2, CO2, H2O, less reducing CH4 and H2) times age, plus standing water.
  - Tholin (`titan-tholin`) from CH4 partial pressure times starlight times age.
  - Sulphur (`sulfur`) on a dry rock world from its heat flow.
  - Frost (the exchange's gas id) from the climate exchange's perennial ice.
  - Nothing is read off an air the climate only assumed (the pack's default mix).
- `crates/sse-kernel/src/adopt.rs`: `resolved_system`'s surface records carry `stains` for each world whose tiles were read, from the orbit atlas already in the cache.
- `packages/renderer/src/sheet-optics.ts`: an n,k table to a seen colour. It uses Hapke's equivalent-slab grain albedo, mixes grains by cross-section, takes the diffusive reflectance r0, and integrates through CIE 1931 in the star's blackbody light with the eye adapted to it.
- `packages/renderer/src/stain-look.ts`: each stain's hue from its sheet row (`readMatter({id})`), laid over the bare ground at the kernel's albedo by its share; water, ice and magma are left as they are. A row with no optics is not drawn. `readTrueFace` (true-air.ts) reads the stains, and `trueSurface` (true-look.ts) takes them.
- Fudge rows: the four stains.rs rates, and the stain drawing (one grain size, one share per world). Citations in docs/citations/surfaces.md, "Surface stains".

## What it shows now
- No golden moves: no contract case reads tiles of a world whose air is its own.
- In the catalogued Solar System, the survey's climate still runs on the default mix (sse-surface climate.rs `x.gases` unset), so no world is stained yet. Proven core takes the fix as SOL5: each body's `atmosphere.composition.*` facts reach the climate through SurveyOptions. With it, Mars, Venus and Earth rust, and Pluto and Triton take tholin.
- Rows: titan-tholin is on the sheet. Mars dust (mgs-1), hematite and water ice arrive with the data sheet's PR 475. SO2 frost and sulphur are this track's next rows (MP34, agreed with the data sheet thread).

## Checks
`crates/sse-surface/src/stains_tests.rs`:
- Mars rusts and the Moon and Mercury do not.
- Titan is thoroughly tholin-coated, Triton a little, and none without methane.
- Io is sulphur-painted.
- Stains grow smoothly with dose.
- A guessed air stains nothing.

`packages/renderer/test/stains.test.ts`:
- Clear grains are white.
- Tholin from the sheet is red over green over blue.
- A few % of tholin tints ice.
- Stains tint ground, not sea.
- A row without optics is not drawn.
- The GPU's cube-face pictures (VP3 `composeFaces`, and `withFiner` when finer tiles arrive) are stained the same way as the flat picture; sea is not stained.

## Where the stain is drawn
`trueSurface` stains the flat equirect picture (`stainPrepared`) and the GPU's face pictures (`composeFaces(tiles, stains)`). The face pictures keep their stains on `Composite.stains`, so a finer tile laid in later (`withFiner`) is stained alike. Bare ground (cover none) is stained and every other cover is left alone. That includes ice: frost has its own cover in the tiles.
