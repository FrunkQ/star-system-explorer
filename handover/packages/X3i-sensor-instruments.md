# X3i: Traveller sensors as instruments (RK11, the pack's share)

Light track's RK11 (PR 329) gave sse-sense one instrument shape with methods (image, spectrum, astrometry, active,
magnetometer, gradiometer) and no typed ranges. This package gives each of the Traveller pack's twelve sensor kinds
an `instrument` in that shape, so the kernel's `detect` computes how far each reaches a given target.
Alex, 23 Nov: sensors "just work", tweakable by those who care.

## Data

`provides.other.sensors.kinds[]` gains:
- `instrument`: an sse-sense Sensor (name, kind, band, aperture, method, stare, and the method's own figures:
  resolving power, transmit power, noise density). V3's range is kept in it as `authored_range_m`.
- `range_source: "authored"` beside the existing `range_m`. The hour-in-range triggers still read that typed range
  until a trigger can ask `detect` for the pair.

| Kind | Method | Figures |
|---|---|---|
| astrometer | astrometry | visible, 0.5 m, 10 min |
| telescope_15au / telescope_2au | image | visible, 1 m / 0.3 m |
| spectral_5au / spectral_1au | spectrum | visible, R 1000, 1 m / 0.3 m |
| bio_scanner | image | near infrared, 0.5 m |
| passive_rf | image (radio) | 3 to 30 cm, 5 m dish, 1 MHz |
| search_radar | active | 3 cm, 3 m, 100 kW |
| thermal_ir | image | 3 to 14 µm, 0.3 m |
| gravimetric | gradiometer | 10 mE/√Hz, 10 min |
| visual_cam | image | visible, 2 cm |
| datalink | none | a handshake: keeps V3's 500 m (authored) |

The figures are game defaults for a typical ship's fit, logged as fudge F51.

## What the numbers say (kernel test `the_traveller_sensors_compute_their_ranges`)

The target is a sunlit 15 m, 2,000 t ship 1 au from a Sun-like star, with 400 K radiators and a 100 W transponder.

- Telescopes and the thermal tracker see it out to about 0.3 au.
- The radar sees it out to about 3,600 km, and its pulse is heard much further.
- The gravimetric grid finds it only within about 570 m.

Against a small ship, V3's typed ranges were sometimes far too long (radar 500,000 km, gravimetric 25,000 km) and
sometimes far too short (the thermal tracker's 50,000 km). The computed figures are honest.

A detection is not an identification: the 2 cm camera *detects* the ship at 470,000 km. Telling what it is still
needs the knowledge ladder.

## Checks

- Every kind is detected through `detect`.
- Each reach is computed, apart from the datalink's.
- V3's range is kept and marked authored.
- The pack's range bands label each detection.
- Radar: heard further than it sees, and its power is reported for RK9.
- A spectrum reaches less far than an image through the same telescope.
- Golden change: the Traveller pack hash only (`tests/golden/traveller-codes.json`); `contract.jsonl` is untouched.

## Next

- Triggers that ask `detect` for the observer and target pair instead of reading `range_m`. This needs a trigger
  predicate on a detection (kernel lane).
- Fits per hull and tech level, once the future tech ladder (FT8) is in.
- The radar as a load on the power ledger (RK9).
