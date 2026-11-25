# LR1: a ship that can fly down flies down

Thread "Build landing run-through", 24 Nov 2026. Landing run-through gap LR1 (`handover/status/landing.md`).

## What was wrong
Walking the journey on main 9a5912c: a Traveller far trader (1 g sustained, exhaust 3000 km/s, a bare hull rated
1 W/cm²) made the party's ship and flown to Mars "burned up" on the Land screen: `plan_descent` flew a ballistic
entry from 182 km at orbital speed, the engines waiting for the air to do most of the braking, which a bare hull never
survives (7 W/cm², touchdown 2392 m/s). The same ship's At-world card said "Powered landing from orbit: within limits".

## What changed
- `sse-nav` `descent::plan_down` (new): flies the entry first; when the lander's thrust is over its weight at the
  ground and it carries propellant, flies it again with the engines allowed to light at the entry interface
  (`fly(.., from_orbit: true)`, the same guidance, braking from orbital speed before the air is thick). The powered way
  is taken only when it ends better: a better result (landed, then hard landing, then the rest), or as good with fewer
  danger-level hazards. Then `method` reads `engines-from-orbit` and the first `why` line says what the entry would
  have done. `descend` is unchanged (ship_at_world's entry check and the scenarios still call it).
- `sse-kernel` `plan_descent` calls `plan_down`.
- No golden change: every stock lander that enters well keeps its entry (tested: mars-lander's plan equals its entry).
- Not a fudge: the choice is a crew's (fly the better of two real profiles); both are the existing physics.

## Checks
- `cargo test -p sse-nav descent` (new `a_torch_ship_flies_down_from_orbit`: burned-up by entry, landed from orbit at
  under 1 W/cm²; mars-lander unchanged; a drive under its weight keeps burned-up).
- `cargo test -p sse-kernel plan_descent` (the far trader at the Mars fixture lands, `engines-from-orbit`).
- `SSE_BLESS=1 ... contract`: no golden diff. `scripts/check.sh` passes.
- `tests/ui/run-through.e2e.mjs`: Beowulf lands at Dribeque Planitia, "Lands safely: engines from orbit, 7 min 50 s".
- Screenshots: project files `sse4/handover-screens/LR-screens/` (`rt-4-player-site.png`).

## Later
- A deorbit burn and a coast to the entry interface are not flown (the descent starts at the top of the air in a low
  orbit, as before).
