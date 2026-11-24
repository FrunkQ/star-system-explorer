# MP31: units for the units engine (ED1a)

The editing thread's units engine (ED1a) reads every unit it offers from the sheet's `units` table, so the rungs it asked for are added there. Only `UNITS` in `scripts/build-base-reference.py` changed; `build-ui-reference.py` is widened by the editing thread itself.

## Rows

| Group | Units | Factor from |
|---|---|---|
| `working` | `min`, `tonne`, `litre`, `tnt_tonne` | NIST SP 811 Appendix B (exact) |
| `working` | `m3`, `km3`, `m2`, `km2`, `percent`, `ppm` | SI prefixes (exact) |
| `working` | `kyr`, `r_moon`, `m_moon`, `ocean_earth`, `s_earth`, `arcmin` | formulas on existing constants (`1e3 yr_julian`, `R_moon`, `M_moon`, `M_ocean_earth`, `S_sun`, `pi / 10800`) |
| `imperial` | `mi`, `ft`, `lb`, `degf`, `psi`, `mph`, `lb_ft3`, `ft3`, `gal`, `sq_mi` | worked out in the builder from the exact foot (0.3048 m), inch (0.0254 m) and pound (0.45359237 kg); `psi` uses standard gravity 9.80665 m/s²; the US gallon is 231 cubic inches |

`degf` carries an offset: K = (°F) / 1.8 + 255.372…

The `imperial` group is last in the click order and is never a default.

## Reader change

`sse-reference` now knows the named dimensions `area` (m²) and `volume` (m³), so unit rows can say `measures: "area"` or `"volume"`.

## Source

New source `nist-sp811`: Thompson and Taylor 2008, NIST SP 811, Appendix B.8 and B.9 (verified; the factors are exact by definition).

## Test

`units_engine_rungs_are_on_the_sheet` checks every rung is present, with exact values for the foot, pound, gallon, mile per hour and the Fahrenheit offset.
