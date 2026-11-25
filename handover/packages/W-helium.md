# W-helium: a helium sea on a rogue planet

Thread "Build shared climate and weather", 25 Nov 2026. This is Alex's sea test, from the seas ruling of 24 Nov: "a rogue planet in a void with a helium sea that just works".

## What changed
- **Liquid-only volatiles** (`sse-climate` `volatile_species`). A substance with no triple point but with a lambda point and a `phases.saturated_liquid` table joins the volatiles, with `liquid_only: true`. For now that is only helium. Its anchor is the lambda point. `ebm::p_sat` reads the curve: ln p is linear in 1/T between rows, and the end segments are carried on beyond the table. `frost_k` bisects that curve. Every volatile now carries `critical_k` from the sheet.
- **The sea liquid by phase** (`sse-surface` `climate::liquid_by_phase`). When nothing names the liquid and water does not fit, the sea is the most abundant gas of the air whose liquid is stable at the surface: above its triple point (helium has none, so it is stable down to 0 K), below its critical point, and with a vapour pressure no higher than the air's. The climate's assumptions list says so. Surface cover reads the same name (PR 418).
- **Wave damping** (`sea.rs` `trains`): each train carries `damping_s` = 1/(2νk²) (Lamb 1932, art. 348), from the liquid's own viscosity. Helium rings about 40 times longer than water for the same wave.

## The test
`sea::helium_tests::a_rogue_planet_holds_a_superfluid_helium_sea`: a world with no orbit and no sunlight, held at 1.8 K by an internal flux of σT⁴ (about 6e-7 W/m²) under 0.02 bar of helium. Nothing in the test names helium as the sea. The climate picks it, and the card shows:
- liquid helium;
- `superfluid` true;
- a 2.8 mm ripple cut-off under 1 g;
- wave trains in each band;
- the damping ratio against water.

## Known gaps (fudge rows)
- Helium's latent heat comes from the curve's end points, its density from the curve's coldest row, and its droplets are drawn white.
- The vapour curve below 2.18 K is extrapolated.
- The damping uses the sheet's single viscosity. For He II the normal-fluid share would make it weaker still.
- The sheet holds no liquid optics for helium yet, so its colour says it is not worked out.
- In the editor, the GM sets a rogue world's internal heat by hand.

## Goldens
Sea lines gain `damping_s` per train. Any world whose climate now finds a liquid by phase gains a sea.
