# HV2b: the outside half of a world's energy flow (and HV2c, a moon's sunlight)

Thread "Show a planet's energy flow over time", 27 Nov 2026. Status: `handover/status/heat-screen.md`.

Alex (26 Nov, 23:18 UTC): "sunlight can heat a core if close enough - we cant miss it. But we can try to show how much is absorbed, how much refelected (albedo visuially)".

## The change

- `heat-flow.ts`: `sunBooks` turns the climate's sunlight books (HV2a, `survey.climate.sunlight`) into Flow rows in watts (W/m² times the world's area): Sunlight and Heat from inside into "Its surface and air"; Reflected to space and Given off as warmth out. It closes because the climate's books close. `sunWhy` says it in words: what arrives, the share reflected, what is soaked up, and how that compares with the heat from inside (sunlight or the inside sets how warm the surface is). Where the climate leaves the inside heat out (Earth today: it is 3,500 times smaller), the words compare with the core history's heat lost instead and say the climate leaves it out; the climate's books are never changed.
- `BodyHeat.svelte`: the panel now has two halves, each drawn to its own scale and labelled so: "Outside: sunlight" (a reflected/soaked-up bar, the flow, the words, and a note that life's share waits on LE2) above "Inside: heat" (HV1's flow). Looking back with the slider keeps the sunlight half at today's and says so (the climate is worked out for today only).
- `Session.surveyHeat` also returns the survey's `climate.sunlight` (null for a world with no climate).

## HV2c: a moon's sunlight (kernel)

Found by drawing the Moon: the climate read sunlight at `orbit.a_au`, which for a moon is its orbit round the planet, so the Moon got about 2 × 10⁸ W/m² (150,000 times Earth's). `sse_surface::Body` gains `star_au` (the planet's distance from the star, set by adoption for a moon; None for a world round a star, so nothing else serialises differently), and `climate::conditions` reads the flux there. Test: `a_moon_gets_sunlight_at_its_planets_distance` (absolute: the solar constant at 1 AU to 1 mW/m²), red before the fix.

Still open, owned by the thread "Fix climate sunlight for other stars and moons" (coordinator, 27 Nov), which builds on this: the climate's sunlight assumes a Sun-like star (`S_SUN / d²`, its "assumed" line says so) instead of the star's own luminosity; a moon's climate "year" is its month (Kepler round the planet); and an adopted moon takes the climate's default albedo (0.30), not its own (the Moon's is about 0.11).

## Check

`packages/ui-contract/test/heat-flow.test.ts`: the outside books close (absolute, a kilowatt in 10¹⁷ W), lay out in three columns, and say the right thing for a sunlit world, a world warmed from inside, a climate that leaves the inside out, and no sunlight; on the real kernel Earth's albedo is 0.2 to 0.4 and it soaks up more than a thousand times its inside heat. `tests/ui/heat.e2e.mjs`: the outside flow, bar and words on Earth, the "today's" note when looking back, and every flow vertical on a phone.

## Golden

None: `tests/golden/contract.jsonl` is unchanged (no moon is surveyed in the contract).

## LE-U1: life's share (added 27 Nov, after PR 680 put LE2 on main)

The coordinator gave LE-U1 (track LE's Energy flow life branch) to this thread. Under the sunlight words, "Life's share" reads the kernel's energy ledger (`life_view` `view.energy.world`, LE2) and draws it as a ladder of bars on a log scale, biggest first: sunlight soaked up (the climate's), usable energy where life could live, what its kinds of life could catch, what life catches, what staying alive costs, and what is spare. These nest and span many powers of ten, so they are never bands of the sunlight flow. The words (`lifeWhy`) say what life catches as a share of the sunlight, whether it pays its keep (the kernel's own surplus, never caught minus kept, because the kernel sums surplus only where life pays its keep), "alive, barely" when every living habitat is barely alive (LE1), "doubtful" when nothing is spare, and the pace of evolution (`lifePace`): Earth-Myr of opportunity gathered per Myr over the last two timeline points (box D's clock, so 1 is the reference pace). The life view is read beside the survey (`Session.lifeView`); a world with no habitats says why.

The Sol door's Earth: about 69 TW caught, 1 part in 1,800 of the 121 PW soaked up; 32 TW keep-alive; pace about 0.43. The V3 synthetic Sol's Earth catches only about 1 GW ("alive, barely") in the node test. That is the life suite's reading of that import, recorded here for the life threads; the view shows it as it is.

`handover/status/life-energy.md` belongs to the life threads: its LE-U1 row needs "built (HV2b log)" from its owner.
