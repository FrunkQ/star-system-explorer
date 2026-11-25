# W-heights: how high each cloud layer sits

Asked 24 Nov 2026 by "Build giant bands, clouds and hazes" for VP-A4 (cloud shells with shadows offset by the sun's angle). Owner: "Build shared climate and weather".

## What the weather cells now carry

Every layer of a weather field cell gains `base_m` and `top_m` above the reference surface. The reference is the ground on rocky worlds and the 1 bar level on giants, where a negative value means below it. A zero is left out.

- **Worlds with a surface:** the descent column's own bands (`hooks.rs` `level_bounds`): 0 to 0.24, 0.24 to 0.83, and 0.83 scale heights up to the tropopause. The field and the column therefore agree. Earth's low layer tops out near 2 km and the middle one near 7 km. Next step: the low layer's base at the lifting condensation level, from the cell's temperature and humidity.
- **Giants:** each deck's base is where its vapour first saturates going up the dry adiabat, x p = p_sat(T(p)).
  - T = T₁ (p / 1 bar)^κ, with κ = R/c_p of the gas mix, isothermal above 0.1 bar.
  - The vapour curves come from the data sheet through `sse_climate::ebm::p_sat`.
  - z = H ln(1 bar / p), and the top is half a scale height above the base.
  - Decks with no vapour curve or listed abundance (ammonium hydrosulphide, and water on giants with no water listed) take pack fallbacks of 5, 2 and 0.7 bar. A deck is pushed half again deeper to keep the order.

## Results

| World | Top deck | Base, bar | Measured, for checking |
|---|---|---|---|
| Jupiter | ammonia ice | 0.73 | no deeper than 0.7 ± 0.1 outside the Great Red Spot (Braude et al. 2020) |
| Saturn | ammonia ice | 1.49 | about 1.5 to 2 (from memory, to check) |
| Uranus | methane ice | 1.28 | about 1.2 (from memory, to check) |
| Neptune | methane ice | 1.42 | about 1.5 (from memory, to check) |

No number was tuned to these: they follow from the 1 bar temperature, the gas mix and the sheet's vapour curves.

## Tests

- `deck_bases_sit_where_their_vapour_saturates`:
  - Jupiter's ammonia base is 0.5 to 0.9 bar;
  - Saturn, Uranus and Neptune fall inside wide windows;
  - the decks are in order on every fixture;
  - the cell carries rising heights.
- `field_layers_carry_the_column_heights`: Earth's layers match the column bands.

## Goldens

The weather field and giants goldens gain `base_m` and `top_m` on every layer. That is the only change, made on purpose.
