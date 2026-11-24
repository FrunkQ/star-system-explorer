# MOI1: every world's moment of inertia factor

This was asked by "Build giant bands, clouds and hazes" on 24 Nov. Their Darwin-Radau flattening block (`sse.shape.spin_flattening`) needs C = I/(M R²) for each body, so that Saturn's and Jupiter's flattening is a result.

## Blocks (`packs/base-physics/formulas/interior.json`, sums in `sse_atoms::inertia`)

- **`sse.interior.inertia_two_layer`**: a core of mass share q and radius ratio x inside a uniform mantle. The formula is C = 2/5 [q x² + (1 − q)(1 − x⁵)/(1 − x³)] (Stacey and Davis 2008). A uniform ball (or a core reaching the surface) gives 0.4.
- **`sse.interior.inertia_giant`**: an n = 1 polytrope, C = 2/3 − 4/π² ≈ 0.261 (Hubbard 1984). The share f of the heavy elements Z that sits in a small core is taken out: C = 0.261 (1 − f Z). The default is f = 0.5, a dilute core (Wahl et al. 2017).

`inertia_blocks_match_their_sums` holds the blocks to the sums. The atom count is now 128.

## Fact

`resolved_system` publishes `body.moment_of_inertia_factor` for every giant (`deep_envelope`) and every world with an iron core. A value already in the record wins.

- **Giants:** Z comes from the mass-metallicity relation at the default star metal share (MP8c).
- **Rocky worlds:** C uses MF1's core radius and the body's core mass fraction.

Solar System values (`sol_tests::worlds_publish_their_moment_of_inertia_factor`):

| World | Computed C | Measured |
|---|---|---|
| Earth | 0.344 | 0.3307 |
| Mars | 0.342 | 0.3644 |
| the Moon | 0.338 | 0.3929 |
| Jupiter | 0.243 | about 0.25 |
| Saturn | 0.230 | about 0.21 |
| Uranus | 0.189 | about 0.22 |

## Known gaps (fudge register)

- **Catalogued worlds' core share.** Every catalogued world takes the default core mass fraction of 0.32, which puts Mars and the Moon too low.
- **Measured values as rows.** The measured values should become catalogued rows in base-sol (`scripts/sol_extra.py`, SOL track), so they win: Mercury 0.346 (Margot 2012), Venus 0.337 (Margot 2021), Earth 0.3307, the Moon 0.3929, Mars 0.3644 (Konopliv 2011).
- **Ice giants' heavy elements.** The mass-metallicity relation gives the ice giants too much heavy element for an envelope model. Their C is low.
