# WS1: a giant's winds from one stream function

Design: `handover/design/SSE4-stirred-clouds-design.md` section 3.1, accepted by Alex on 27 Nov 2026. Status row: `handover/status/true-air.md`.

## What was wrong

Two of a giant's winds piled cloud up and thinned it out, and stirring cloud through them would make that visible:

- **The circumpolar jet ran round a latitude circle.** Near a polygon (Saturn's hexagon) the cloud edge followed the polygon, but the wind still blew due east. Carried by that wind, the hexagon would smear into a ring in one turn. Along a side, the wind crossed the side at 15°.
- **A sheared oval's wind used the round direction inside a stretched distance.** It was not the wind of any stream function. Over Saturn's most stretched oval (aspect 1.46), the divergence came to 2.3e-5 s⁻¹, a tenth of the vortex's own turning.

## What changed (`crates/sse-weather/src/giants.rs`)

**`banded_wind`.** The bands and the jet now come from one stream function, ψ = h Ψ(ρ). Ψ′ is the zonal wind at the latitude whose polar distance is ρ. The winds are u = −(1/a) ∂ψ/∂φ and v = (1/(a cos φ)) ∂ψ/∂λ, so they have no divergence whatever ρ is.
- With no polygon, ρ is the distance from the pole and this is `zonal_wind` exactly. Jupiter, Uranus and Neptune are unchanged.
- With a polygon, ρ = r + (r_jet − E(λ)) T:
  - E is the polygon's edge along this longitude.
  - T is 1 on the edge, so the jet's contour is the polygon itself.
  - T falls to nothing at the pole as 2x² − x⁴, with x = r/E, so the wind stays finite there.
  - Outward, T falls away as a Gaussian over two jet deformation radii, the reach of the jet's pull on the cloud.

**`polygon_edge_slope_km`.** The edge's slope with longitude.

**A storm's winds** now come from the Gaussian vortex's stream function, Φ = −q v R exp((1 − r²)/2).
- r is measured in the storm's ellipse, in its own frame (`storm_xy`: azimuthal equidistant about the storm, so the frame is one function of place).
- The winds are taken from Φ's slopes across a hundred-thousandth of the storm's radius.
- The storm's cover uses the same r (`storm_r`).
- `radial` is gone.

## Tests (`giants_tests.rs`), both seen red on the old code

- `ws1_a_sheared_ovals_winds_pile_nothing_up`: over Saturn's most stretched oval, divergence below 1e-8 s⁻¹ (absolute). It was 2.3e-5, and is now 1e-9, the differencing's own error. The vorticity is about 2e-4 s⁻¹.
- `ws1_the_hexagons_wind_runs_along_its_sides`: on the hexagon's edge a quarter of the way along a side, the wind runs along the side within 0.5°. It crossed at 15.00° before. Divergence there is also below 1e-8.

## Golden

`tests/golden/weather-giants.json` (crate golden): Saturn's field digest is re-blessed for the polygon's winds and its ovals' winds and cover. Jupiter, Uranus and Neptune do not move. `tests/golden/contract.jsonl` does not move.

## Fudges

None new. The polygon taper's widths (zero at the pole, two deformation radii outside) reuse the cloud edge's existing pull.
