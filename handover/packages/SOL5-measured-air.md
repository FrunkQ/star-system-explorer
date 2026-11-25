# SOL5: catalogued worlds' climates use their measured air

The plumes, frosts and hot glow thread found this on 24 Nov while writing the surface stains. Every survey and Orbit atlas climate ran on "air of the default mix". `SurveyOptions::extra` never set `Extra.gases`, so base-sol's catalogued `atmosphere.composition.*` facts never reached the climate. Venus and Mars were given Earth's 21% oxygen.

## What changed

- **`SurveyOptions.gases`** (sse-surface `survey.rs`) is new and optional, serialised only when set. It holds volume fractions keyed by lower-case formula, and `extra()` passes it through as `Extra.gases`. When it is absent the climate falls back to the default mix and still says it assumed one.
- **`detail::orbit_atlas_with`** takes the survey options. `orbit_atlas` keeps its old signature and calls it with the defaults.
- **The kernel door.** `adopt::catalogued_gases(row)` reads a world's `atmosphere.composition.<gas>` facts and keeps the values that lie between 0 and 1.
  - `survey_surface` (in `survey::resolve`) uses those facts unless the caller gave a mix.
  - The Orbit atlas in `resolved_system` uses them too. The atlas cache key now includes the mix, so editing a world's air rebuilds its atlas.
- Body is unchanged.

## Checks

- Test `sol_tests::catalogued_worlds_breathe_their_measured_air`: the survey climates of Mars, Venus and Earth no longer say they assumed the default mix.
- Golden: a bless run left `tests/golden/` unchanged, because no golden line surveys a catalogued world's climate.

## Next

The stains thread (`stains.rs`) can now read a stain from a measured air instead of skipping it, so Mars' ferric red should appear.
