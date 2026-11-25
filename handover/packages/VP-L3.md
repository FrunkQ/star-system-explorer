# VP-L3: the colour of light inside a world's shadow (light track, 25 Nov 2026)

Visual polish design section 5.8 (win 4): a world with air bends red light into its own shadow; one colour per
world from the kernel's air, so an eclipsed moon turns copper round an Earth and stays black round an airless world.

## What was built

- Three formula blocks in `packs/base-physics/formulas/shadow.json` (family `sse.shadow`); the atom count goes to 146:
  - `sse.shadow.grazing_column`, N = n √(2π r H);
  - `sse.shadow.refraction_bend`, θ = ν √(2π r / H) (Baum and Code 1953);
  - `sse.shadow.refraction_dilution`, φ = 1 / (1 + D θ / H).
- `crates/sse-light/src/shadow.rs` `shadow_light(ShadowAsk, table, sheet)`. It steps through heights above the limb
  (8 per scale height, up to 16 scale heights) and keeps the heights whose bend brings starlight to the middle of the
  shadow at the asked distance, from some part of the star's disc. Each kept height passes the star's light through
  its grazing column: Rayleigh scattering from `air::cross_section`, and the sheet's gas absorption through
  `gas::pass` (`light_through_gas`). The result is weighted by the spreading and coloured by the eye
  (`colour::colour_of`). `air::refractivity_of` is new and public.
- Op `read_shadow_light {items: [ShadowAsk]}` returns `shadow_light_read`; errors come back as `bad_shadow`. It is
  in perf chunk light. Each item gives `lit`, `colour` (the eye's `Colour`, absent when unlit), `star_colour`,
  `heights_m`, `transmission_550`, Why, stand-ins and references.

## Checks

- sse-light `shadow_light::the_moon_in_earths_shadow_is_copper`: light skimming 0.5 to 3.7 km up, bent 0.85°,
  1% of green getting through; colour #FF5500, red over green over blue.
- `an_airless_worlds_shadow_stays_black`; `mars_is_too_thin_to_light_phobos` (bend 0.01°).
- sse-kernel `read_shadow_light_colours_an_eclipse`, and two contract cases (an Earth and an airless moon; a bad radius).

## Goldens

`tests/golden/contract.jsonl` gains the two new cases (on purpose).

## For the drawing

The renderer's eclipse shadow (section 5.8's moon-on-planet and planet-on-moon formula) tints its umbra with
`colour.linear_srgb` when `lit`, scaled by the share of the star's disc hidden. Brightness inside the shadow is not
given yet: see F65.

## Next

Clouds and haze along the limb (they hide Earth's lowest kilometres and darken real eclipses), the bend by colour, the
shadow's edge, and brightness across the shadow.
