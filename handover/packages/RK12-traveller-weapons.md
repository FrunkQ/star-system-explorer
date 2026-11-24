# RK12, Traveller mapping: ship weapons as effect at range

Atomic Rockets plan section 13 and question 16 (accepted 23 Nov): weapons give three readings (in range, marginal,
out of range) and rings, and a game pack keeps its own table beside the physics. The kernel side (op
`effect_at_range`, `sse_sense::reach`) is the light thread's RK12 part 1 (`packages/RK12.md`).

## What changed

- **`provides.other.weapons` in the Traveller pack** (`scripts/build-traveller-pack.py`, `WEAPONS`):
  - `standard_target`: a small armed hull, 2 mm of aluminium (the sheet's heats), 15 m across, able to pull 2 g.
  - Nine kinds: pulse laser, beam laser, particle beam, particle beam bay, fusion gun bay, meson gun bay, nuclear
    missile, standard missile, sandcaster. Each has `mount`, `about`, and `game` (tech level, optimum band with its
    `from_m` and `to_m` from the pack's range bands, damage dice, source) beside `weapon`, sse-sense's Weapon shape.
  - The game lines are numbers and names from the Cepheus Engine SRD (Samardan Press, Open Game Licence; chapter 8
    armament tables, read 24 Nov). No rules text is copied. The SRD has no stat line for the beam laser, so its band
    is left as medium (above the pulse laser) and its tech level and damage are left out.
  - The meson gun (fictional physics) and the sandcaster (a defence) carry no `weapon` and say why. The game's band
    stands alone for them.
- **Fudge F54**: the physical figures are hand-set game defaults, tuned so each beam is in range or marginal at
  the far edge of its game band against the standard target.

## What the physics says (cheap "cool physics wins")

- The pulse laser (500 MW UV, 0.1 s, 2 m mirror) is in range to about 1,400 km and marginal to 2,800 km: short.
- The beam laser, particle beam, particle beam bay and fusion gun bay reach the edges of medium and long. The bay
  draws five times the energy, so it is still fully in range at 25,000 km.
- A kiloton in vacuum boils through 2 mm of aluminium only within about 55 m. A kilometre's near miss leaves the
  hull unharmed: there is no blast in space, only X-rays falling as the square of distance.
- A 50 kg missile body at 3 km/s carries 4.5 MJ/kg and tears through at any distance it reaches. Its limit is the
  odds and the flight, which is NAV's.

## Checks

`tests::the_traveller_weapons_reach_their_game_bands`: every beam reads its skin from the sheet, is not out of
range at its band's edge, is not in range at four times it, and draws a load. Both missiles are in range at 30 m;
the burst is out of range at 1 km, with its "enough" ring between 30 and 100 m. The two kinds without figures say why.

Golden changed on purpose: only the pack hash in `tests/golden/traveller-codes.json` (the pack gained `weapons`); no reading changes. The golden contract does not read this pack, so it is unchanged.

## Next

- Rings on the chart (renderer and chart room) read from these kinds; the weapon picker on the construct card.
- RK16 pack side: stealth and countermeasure gear onto the counters list.
- Figures per tech level from a ship design (FT8).
