# V9w: the cosmic web close to real, in 3D

Alex, 23 Nov 2026: "the cosmic web void figure, we wanna be close to real. The current large scale view does look rubbish. I was hoping 3D would make it make sense... it's that 3D spiderweb like construct ... we look like we have a wrinkled rug". Rulings: void fraction toward the measured 77% as a cited editable number; the view must read as filaments, nodes and voids.

## What it does

- **Voids fill 77% of space** (pack `base-cosmology`, `web.void_fraction` 0.77, Cautun et al. 2014, NEXUS+ on the Millennium simulation). The T-web threshold is no longer a fixed 0. It is found so that voids take that share of *space*:
  - Doroshkevich (1970) draws of the smoothed Gaussian field's tidal tensor (keyed stream `cosmos.web.tweb.v1`, 20,000 draws, sorted);
  - each draw weighted by its Zel'dovich room today, |(1 − Dλ₁)(1 − Dλ₂)(1 − Dλ₃)|, at the field's linear spread D σ where it is read;
  - so voids, which spread out, fill more space than their matter's share. A first version set 77% of the *parcels* (a share of matter), which gave the ledger 92% of space and 85% of the matter in voids; the kernel thread caught it.
- **Truncated Zel'dovich** (Coles, Melott and Shandarin 1993; σ ≈ 1 from Melott, Pellman and Shandarin 1994): motion and environments are read from the field smoothed where its linear spread reaches 1, so filaments stay sharp. The slice, the cube and DM3's matter ledger all read this one web with one threshold.
- **Result.** In the draws at D σ = 1: voids 77% of space and about 40% of the matter, sheets 22% and 48%, filaments 1% and 11%. Measured: voids hold about 15% of the matter. One Zel'dovich step at σ ≈ 1 is too linear to empty them further (fudge register, "web.rs T-web void share").
- **The cube.** Kernel `web_slice` with `cube: true` (n 4 to 48 a side) returns sse-cosmos `Volume`: each parcel's position after the step, its environment, its starting density and its room today.
- **The view.** The foam map gains 3D and True (`views/foam-map/src/Web3D.svelte`, `renderer/src/web-cube.ts`):
  - 3D: parcels as dots on the chart's ground, coloured by environment (voids faint, sheets, filaments in ink, knots in the accent);
  - True: parcels as light on black, all alike, so crowded matter glows brightest (old starlight at about 5,000 K; fudge row);
  - it turns slowly until dragged (not with reduced motion); HUD draws the cube's edges; Real shrinks the cube by the scale factor; Play grows it through time;
  - the ledger shows share of space and share of matter per environment; the slice's shares say they are of the matter.
  - 48³ parcels: about 4 to 8 s in the browser for a cube. 100 Mpc reads best (the smoothing is 16 Mpc).

## Goldens (changed on purpose)

- `tests/golden/cosmology.json`: the slice hash and `t_web_counts` change, because the threshold now gives 77% void space instead of a fixed 0.
- `tests/golden/contract.jsonl`: `web_slice` environments; DM3 `matter_budget` environments (cell 0.0.0 now read at the truncated smoothing with room weights); one new request, the cube.

## Checks

- sse-cosmos: `a_void_share_sets_the_threshold_and_holds_at_any_smoothing` (77% ± 8% of space at three smoothings; at σ > 0.7 voids hold at least 20 points less of the matter than of the space), `a_volume_moves_matter_into_filaments_by_truncated_zeldovich`.
- sse-kernel: `a_web_cube_gives_parcels_for_the_3d_view`.
- renderer: `web-cube.test.ts`. e2e: `foam.e2e.mjs` (3D and True at 100 Mpc; void space 0.6 to 0.95, matter below it).
- Screens: project files `sse4/handover-screens/V9w-screens/`.

## Next

- A multiscale classifier (NEXUS+) or a 2LPT step would bring the voids' matter share toward 15%.
- Light from haloes and galaxies (G2, G3) instead of one colour for all matter.
