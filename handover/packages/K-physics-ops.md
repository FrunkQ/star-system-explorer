# Kernel ops from the physics track: import_v3 and run_formation (19 Nov 2026)

Asked by the UI track through the coordinator: two chart room screens ("import my V3 save", "generate a system")
need kernel ops only the physics code can back. Names follow the kernel track's convention (snake_case, verb
first, additive under `sse.kernel/1`; see `K6-additions.md` on the kernel track's branch).

## Ops
- `import_v3 { save, universe_id?, reconstruct? }`: `save` is the text of a V3 `starmap.json` or `system.json`
  (unpack a `.sse.zip` first). Response `v3_imported { universe, facts, report, reconstructed }`:
  - `facts`: address, then field, then `{ value, evidence }`, as the importer reads them (P3);
  - `report`: the difference report (`sse.import-report/1`: counts, differences with tolerance and `via`,
    unclassified and dropped fields, honesty marks, warnings, state hash);
  - `reconstructed` (only with `reconstruct`: candidate histories to search, 1 to 64): one record per known
    system, the F9 summary plus `run` (the solved `SystemRun`, history labelled reconstructed) and `matches`.
  - A save that is not V3 gives `error { code: "import_failed" }`.
- `run_formation { seed, system?, block?, birth?, now_gyr?, envelope?, inherit_pins?, star_pins?, pins? }`:
  one system through the single pipeline. Inheritance from a cosmos `birth` (G0's `SystemBirth`, read at `now_gyr`,
  default 13.8) or a patch `envelope`; neither means the stand-in solar neighbourhood. `block` is `modern`
  (default), `classic` or `classic-corrected`. `pins` is F9's `Pins` (`worlds: [{label, mass_earth, a_au, e?}]`,
  `tries`). Response `formation_ran { system, pipeline, run, pinned?, surfaces }`: `run` is the `SystemRun`
  (history, growth, upheaval, leftovers, protoplanets, worlds, heat, events, notes); `pinned` the F9 fit
  (attempt, seed, fit, fit_first, matches, removed); `surfaces` one S0 surface envelope per world.
- Both are pure: nothing is written to an open universe. Recording an import or a generated system as chronicle
  events is for the screen that adopts it (set_authored, or a later "adopt" op if the kernel track prefers one).
- Cost classes: `import_v3` light (heavy with `reconstruct`), `run_formation` medium.

- `read_world_stack { body?, fixture?, stack? }` (added later on 19 Nov for the Cutaway (Body) view): a body's
  world stack (`sse.world-stack/1`, P4). Served from the P4 fixtures (`earth`, `europa`, `mars`, `tarn`, `titan`,
  `venus`) by `fixture` name, or by `body` when it is a fixture's address; a given `stack` is checked instead.
  Response `world_stack_read { body, source, stack, ladder, notes, problems, invalid }`: `source` is
  `fixture:<name>` or `given`; `stack` is the stack as stored (fixture `sources` kept); `ladder` the "above, on
  and within" lines (`depth`, `text`, `where`, `key`); `notes` and `problems` from `physics_notes()`; `invalid`
  from `validate()` (each `{at, message}`). Nothing else gives `error { code: "no_world_stack" }` listing the
  fixtures. Light. A given `stack` is echoed as parsed (numbers as f64), so the worker and native
  kernels return the same bytes. When the surface track builds stacks from history (S7 onwards), the same op serves them with
  `source` `built:<epoch>`; the response shape stays.

## Notes for the UI
- A `run_formation` response is 70 to 300 KB of JSON for a sun-like star. Some seeds form no planets: a high
  `history.inheritance.fuv_g0` (a nearby massive star) strips the disc in under a Myr; `run.growth.events` says
  so ("gas-gone"). Pin `star_pins.primary_mass` to get a sun-like system.
- `crates/sse-import-v3`: `Fact` now derives `Serialize` (needed for the response).

## Checks
- `cargo test -p sse-kernel`: same request gives the same response; a pinned Earth at 1 au is honoured; worlds
  and surfaces line up; bad block and bad save are protocol errors.
- Contract (`tests/contract/requests.json`): five new cases, then four for `read_world_stack` (re-blessed again:
  `hello` and the unknown-op message gain it, plus four lines). `tests/golden/contract.jsonl` re-blessed on
  purpose: the `hello` op list and the unknown-op error message (which lists the variants) gain the two ops, and
  five new lines. TS types regenerated (`packages/contracts/src/generated/Request.ts`, `Response.ts`).
- Harness: kernel API through a Web Worker, 115 of 115 responses identical to native (Node and Chromium);
  `npm test` and `npm run typecheck` pass.

## Integration
- The kernel now depends on `sse-formation`, `sse-import-v3`, `sse-cosmos` and `sse-surface`.
- PR 5 (kernel track) also adds variants to `protocol.rs` and lines to the contract; whichever merges second
  keeps both sets and re-blesses `contract.jsonl` and the generated TS.

## survey_surface (19 Nov, after S7)
Serves S2 to S7 for one world, so the Body globe and later ground views can read cover, ice, wetness,
deposits, built places and the feature register.
- **Request** `survey_surface { fixture? | system + body, at_s?, options?, tiles?, quick?, limit? }`:
  - `fixture`: a surface fixture by label (`Earth`, `Moon`, `Venus`, `Jupiter`, ...; any case). Needs no universe;
    surveyed under the fixed seed `sse.survey-fixture`.
  - `system` + `body`: a body of an adopted system in the open universe, read at `at_s` (default the system's
    now) exactly as `resolved_system` reads it (the shared helper is `adopt::worlds_at`), under the universe seed.
  - `options`: sse-surface's `SurveyOptions`: `rotation_hours` (default: a locked world's orbital period, else
    the climate's default), `obliquity_deg`, `life` (0 lifeless, 1 Earth), `settle` (built kinds to place),
    `reshape` (S7 GM edits, `op`-tagged), `names` (GM names), `renames`, `wind_m_s` (aerostat drift).
  - `tiles`: `[{face, level, x, y}]`, at most 24, built surveyed with S7's layers (`wetness`, `deposit`,
    `structure`, `colour`).
  - `quick`: a 16-cell plate grid and 32-sample tiles (tests, weak devices).
  - `limit`: features and deposits listed, default 200, 0 for all. Remarkable features first, then the named
    landscape (oceans, continents, ranges ...), then craters by size; deposits by size. Kept items stay in register
    order; `register.features_total` and `deposits_total` always give the full counts.
- **Response** `surface_surveyed { survey }` with `body, label, source (fixture:<name>, formation, v3 or facts),
  version (surface-survey-v1), epoch_myr, at_s?, stages, envelope, deposit_kinds, climate, air, deposits,
  deposits_total, built, built_kinds, applied, register, aerostat or aerostat_refused, tiles, refused`.
  A giant (`cloud-deck`) gets stages `[envelope, cloud-deck]`, no layers and no register.
- **Errors**: `unknown_fixture` (lists them), `no_surface_body`, `no_universe`, `bad_request` (neither form, or
  bad options), `too_many_tiles`, `survey_refused` (a stage refused, e.g. an unknown mechanism in a pack).
- **Cost**: heavy. Native release at pack defaults: Earth 5.6 s, Moon 3.3 s, Mars 4.3 s, Venus 7.9 s (plates,
  wear and the register dominate); quick is about 0.1 to 0.3 s. The kernel keeps the last 4 surveys keyed by
  (seed, body, epoch, quick, options), so asking for more tiles or another `limit` of the same world is fast.
  Changing any option (a rename included) re-runs the stages.
- **Code**: `crates/sse-surface/src/survey.rs` (`survey_world`, the one chain every caller should use, and
  `day_hours`), `crates/sse-kernel/src/survey.rs`, `adopt::worlds_at` (factored out of `resolved_system`; no
  behaviour change).
- **Checks**: `sse-surface` `survey_tests` (every fixture surveys; giants alone have no register or layers);
  `sse-kernel` `survey_surface_serves_layers_register_and_tiles` (fixture Earth with settling, a 40 km range
  tagged impossible, a bad tile key refused, same request same bytes, a rename changes the register and not the
  tiles; Jupiter; refusals; a formation-adopted body with `limit`). Contract: five new cases (Earth quick with a
  pin and a tile, Jupiter, an unknown fixture, a formation body, a missing body); `contract.jsonl` re-blessed
  on purpose (the `hello` op list, the unknown-op message and the five lines); TS regenerated (new
  `SurfaceTile.ts`).
- **Next**: a rename or GM edit as a chronicle event (so the register and edits come from the universe rather
  than the request) belongs with the GM approvals flow; until then the UI sends them as options.

## plan_route (19 Nov, T3)
NAV through the kernel; see `T3.md` for the model, the request and response, errors and checks. Contract: six cases
(Sol by chemical ship and torch, the drift at 1 au, an unknown ship, a course in the adopted `u:kestrel/sys:40`, the
same body twice); `contract.jsonl` re-blessed on purpose.

## plan_descent (19 Nov, S5)
`{op: "plan_descent", fixture | system + body, at_s?, options?, quick?, lander, lat_deg, lon_deg, entry?}`.
Surveys through the `survey_surface` cache, judges the site (`sse_surface::site`), builds the air from the survey's
air profile, and flies `sse_nav::descent::descend`. `lander` is a base-nav template (`capsule`, `mars-lander`,
`titan-probe`, `moon-lander`) or an object; `entry` is `{altitude_m, speed_m_s, flight_path_deg}`. Returns
`descent_planned {descent: {body, label, source, at_s?, lander, entry, site, air_at_site, gravity_m_s2, hazards,
verdict, descent: {outcome, events, telemetry}}}`. Errors: `unknown_lander`, `no_ground`, `bad_request`, plus the
survey's. Heavy. See packages/S5.md.

## survey_caves (21 Nov, S14a part 2)
`{op: "survey_caves", fixture | system + body, at_s?, options?, quick?, lat_deg, lon_deg, radius_km?}`.
Surveys through the `survey_surface` cache and places the lava-tube passages within `radius_km` (default 5,
at most 25) of the site from the world's tube fields (`sse_surface::caves::passages`). Returns
`caves_surveyed {caves: {body, label, source, at_s?, caves: {version, lat_deg, lon_deg, radius_km, level,
spacing_m, fields, passages, why}}}`. Errors: `no_ground`, `bad_request`, plus the survey's. Heavy (the survey;
the passages are tens of ms). See packages/S14a.md.
