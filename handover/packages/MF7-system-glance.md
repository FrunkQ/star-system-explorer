# Gap item 7: which worlds are shielded, at a glance (kernel side)

Light track, 23 Nov. This is gap item 7 in `handover/design/SSE4-v3-gaps.md`, the data that the Mag band draws at system scale (MF14c). Alex agreed on 23 Nov that the obvious gap items could go ahead.

## What it does

`{"op":"read_field","system":"u:…/sys:…","at_s":…?}` reads every planet of an adopted system. It answers with the usual `field_read`, one FieldRead per planet, each with `name` and a `glance`:

| Field | Meaning |
|---|---|
| `held_off_by` | "field", "air" or "surface": what stops the star's wind |
| `bubble_m`, `bubble_radii` | how far out it does so |
| `shielded` | true when the world's own field holds the wind off (the dark core of V3's shading; `bubble_m` is the pale extent) |
| `ground_dose_share` | cosmic-ray dose on the ground as a share of open space (0.5 means only the planet itself shades it) |
| `belts_m` | inner and outer edge of the electron belts, or null |
| `field` | "measured" (the Solar System, MF14a fixture), "computed" (`body.magnetic_moment` above 0), "none" (a moment of 0: dynamo off) or "unresolved" (no moment fact yet) |
| `dynamo_now` | MF1's core history: whether a dynamo runs now (the reason is in `why`) |

## Where each value comes from

- **Moment:** the `body.magnetic_moment` fact (A m², MF2).
- **Tilt:** the `field.tilt` fact (degrees, MF3), 0 when absent.
- **Air:** the `atmosphere.surface_pressure` and `atmosphere.scale_height` facts.
- **Gravity:** the `body.surface_gravity` fact, or the surface envelope's gravity.
- **Wind:** the `sse.light.stellar_wind` block at each orbit.
- **The Sun's planets:** matched by name, they keep their measured fields.

The key names were agreed with the proven core thread on 23 Nov. The read fills in by itself as MF2, MF3 and the air facts land.

## Stand-ins (fudge F45)

- The wind is the block's solar defaults, whatever the star.
- A world with no moment fact is read as unmagnetised.
- A world with no air facts is read as airless.
- A giant's solid radius stands for its cloud tops.
- Moons are left out.

Today a grown system's worlds all read "unresolved" and airless, because MF2 and the air facts are not on main yet.

## Checks

- Test: `read_field_for_a_whole_system` in `crates/sse-kernel/src/tests.rs`.
- No golden changed: the contract has no adopted-system field read.
- Code: `crates/sse-kernel/src/fields.rs`.
