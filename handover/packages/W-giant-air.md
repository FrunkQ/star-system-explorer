# W-giant-air: the giants' own air and 1 bar temperatures

Thread "Build shared climate and weather", 25 Nov 2026, in PR 418.

## What was wrong
Alex saw Saturn drawn black and white. The giant bands thread traced it:
- **Air:** the weather op (and the survey's climate) built every world's conditions from the climate pack's default air, which is Earth's. The catalogued `atmosphere.composition.*` facts never reached `sse-climate` `Conditions.gases`. Saturn and Uranus read "nitrogen", with nitrogen's 76 K condensation, and a nitrogen collapse check.
- **Temperature:** the Sol giants have no `planet.surface_temperature`, so they sat at their equilibrium temperatures. Saturn was about 81 K, below the 90 K cut in `weather_giants.clouds.chemistry`. That gave it the ice giants' methane-ice top deck, and so no chromophore.

## What changed
- `adopt::catalogued_gases` (the same code as SOL5 in PR 417, so 417 rebases cleanly): a body's air by volume from its `atmosphere.composition.*` facts, catalogued or typed. The weather op's `body_conditions` and the survey's `resolve` (through the new `SurveyOptions.gases`) both pass it on, so they keep one climate.
- Deep-envelope worlds with no measured air take `climate.defaults.giant_gases`, a protosolar mix (Asplund et al. 2009), instead of Earth air. This is logged as a fudge.
- `packs/base-sol`: the giants carry their 1 bar temperatures from the NASA fact sheets (Jupiter 165, Saturn 134, Uranus 76, Neptune 72 K).
- Test: `sol_tests::saturns_top_deck_is_ammonia_from_its_one_bar_temperature`. Saturn's air is its own, its top deck is ammonia ice with the chromophore, and Uranus keeps methane ice.

## Still open
A generated giant's 1 bar temperature is still its equilibrium temperature. The fix is a radiative-convective profile with internal heat, with the deck stack chosen by which condensates saturate (fudge row). It belongs with W11, the giants' decks onto the curves.

## Goldens
Every Sol and V3 world's weather and survey climate now reads its own air when it has one. Mars reads carbon dioxide and Titan nitrogen with methane. Lines that move: the survey, weather and surface goldens, and the contract lines.

## Citation corrections carried here (Consensus batches 1 to 5, 25 Nov)
- O'Neill, Emanuel and Flierl 2015 no longer cites the Jupiter ring of cyclones or the Saturn vortex rule (it predates Juno). It stays as the source for moist convection driving polar cyclones. The 0.1 W/m² internal-heat cut and the a/L_d 30 and 40 cuts are now called SSE4's own (giants.rs rule text, base-weather `polar.source`).
- Brueshaber et al. 2019: the high-Bu regime is the paper's "large cyclonic polar vortex", drawn as a broad cap. Brueshaber and Sayanagi 2021 is named for Bu not cleanly splitting it from the Saturn regime. The `diffuse-cap` id is kept for the drawing.
- Showman and Polvani 2011: the locked world's rising air and cloud shield are Yang, Cowan and Abbot 2013's. Showman and Polvani keep the standing waves and the jet (base-climate `tidally-locked.because`; formations.rs was already right).
The weather goldens move only by these why-texts.
