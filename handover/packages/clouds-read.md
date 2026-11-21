# Leftover clouds in resolved_system (small kernel package, 21 Nov)

Asked by the chart room shell: grown systems read back through
`resolved_system` showed no asteroid belts, because F5's leftovers
(`run.leftovers.clouds`) were never surfaced after adopt.

- `resolved_system` now returns `clouds` for formation-sourced systems:
  `{address, kind, around, source, a_min_au, a_max_au, largest_km, over_1km}`.
  `around` is null for clouds round the star (belts, scattered disc, Oort);
  otherwise the world id, with axes in AU around that world.
- Read from the cached formation run (rebuilt from the adopt event when
  missing), so nothing new is written to the chronicle and replays are
  unchanged. The field is omitted when empty, so V3 and facts systems read
  exactly as before.
- Not done: clouds after a time jump (F6 upheaval or ageing does not move
  them yet), and clouds for V3-imported systems (the UI reads those from the
  V3 data directly).
- Test: `adopt_age_and_read_back_a_system` (tests.rs) checks at least one star-orbiting
  cloud with a sensible span.

- Follow-up (21 Nov, asked by the chart room): each cloud also carries `shape` (`disc` for Rayleigh-distributed tilts, with `i_sigma_deg`; `shell` for isotropic ones, `i_sigma_deg` null, the name the night sky design gives it in Y0), from F5's own `Tilt`. The Oort cloud comes out as a shell.
