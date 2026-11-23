# V8: a ship at a world (aerobraking, landing and take-off on the physics)

Second UI track ("Build galaxy charts and travel screens"), 23 Nov 2026.

Alex, 23 Nov: "we need to refine ship aerobraking and landing capability - it was binary and simplistic in SSE3 ...
we can do the real physics now?", then "If the physics says its dangerous - great ... we wanna surface detail and
chances of survival ... rather than forbid" and "we wont DO the skill checks ... we just want enough good information
for the GM to make those judgement calls". So: no flags, no refusals, no rolls. Every act is worked out from the
ship's numbers and the world's air and gravity, and answered with the figure against the limit of the part that bears
it, how close it runs, what happens if it goes wrong, effects on the ship's kept state, and a rough chance with its
basis. The risky act sits beside the safe one.

## What landed

- `crates/sse-nav/src/entry.rs` (new): `at_world(settings, descent pack, ship, body, air, v_inf, arrival)` gives one
  `AtWorld` with these checks:
  - **aerocapture**: one ballistic grazing pass. Column ρ_p √(2π r_p H), ln(v_in/v_out) = column / (2β), solved for
    the pass height that leaves an orbit reaching `capture_apoapsis_radii`. Figures: pass height and speed, speed
    shed, pass seconds, peak heating (Sutton and Graves) and load (q √(4π r_p H) / v), peak g (which depends only on the
    speed shed and the air, not the ship), the corridor (too high: still escaping; too low: below circular speed),
    the burn it replaces, the apoapsis burn that lifts the low point out of the air, and the propellant saved.
    Dangers: too little drag, heating, heat load, crew g (Eiband brief tolerance for the pass time), structure g, and
    the corridor (chance the pass is flown outside it).
  - **entry**: S5's `descend()` flown with a lander built from the ship (mass, area, C_d, nose radius, a parachute from
    `landing.chute`, the drive as the engine, the shield's limit, L/D). Dangers: heating, heat load, crew g,
    touchdown against the gear.
  - **runway-landing** (winged ships): stall speed √(2 m g / (ρ C_L,max S)) at landing mass and the take-off roll.
  - **powered-landing** and **take-off**: thrust to weight, Δv (a low orbit plus a gravity-loss stand-in) against the
    usable Δv, crew g on the burn.
  - Verdicts are `safe`, `risky`, `dangerous`, or `not-here` only where the physics has nothing to act on (no air to
    brake in, no ground under a giant's envelope).
  - Effects say what a bad outcome does to kept ship state for a rules program: `ship.readiness` capped at 0.5 for
    damage, set to 0 for a wreck. Crew harm stays in words.
- New optional `Ship` fields (agreed with the renderer thread, all `skip_serializing_if` so existing plans and goldens
  keep their bytes): `shield {nose_radius_m, max_heat_flux_w_m2, max_heat_load_j_m2?, kind, mass_t?}`,
  `aero {ref_area_m2, cd, lift_area_m2?, cl_max?, l_over_d}`, `landing {legs, max_touchdown_m_s, chute?}` (chute is
  S5's `Parachute`), `crew`, `cargo_t`, `fuel_shares`, `structure_g`. `NavBody` gains `air`.
- Pack `nav.entry` in `packs/base-nav`: hull volume per tonne and C_d for ships with no shape, capture apoapsis, pass spread,
  chance width, brief g table, gravity-loss share, no-gear touchdown, runway speed, γ. Sources added: aerocapture,
  brief_g, stall.
- Sol fixture (`tests/fixtures/nav/sol.json`): air for Venus, Earth, Mars and the four giants (1 bar level, no ground)
  from the NASA fact sheets.
- Kernel op `ship_at_world {fixture | nav | system, at_s?, ship, worlds, v_inf_m_s?}` → `ship_at_world_read {report}`
  (cost Light). An adopted system reads each body's air from S7's own guess (`cover::Air::guess`, the gas the descent
  and cover use; a cloud-deck world has no ground). Default arrival: a Hohmann transfer from the next body in.
- ui-contract: `ship-at-world.ts` (shapes, `verdictWords`, `chanceWords`, `worstDangers`), `Session.shipAtWorld`
  (GM only for an adopted system), `NavShip` and `NavBody` carry the new fields; perf chunk "voyages" lists the op.

## Checked

A 10 t crewed capsule (6 m nose, 4 MW/m² shield, 3,000 m² chute) arriving at Earth from Venus: one pass at about
84 km sheds 0.5 of 11.4 km/s at 1.1 g and about 0.85 MW/m², corridor about 23 km deep. Entry and chute landing at
about 8 m/s. At Mars the same chute leaves it landing far too fast (danger, said, not refused). At Jupiter the pass
heats past the shield. Mercury has nothing to brake in. A fusion torch with no shape numbers is guessed as a sphere
and says so.

Tests: `crates/sse-nav/src/entry_tests.rs` (8), contract cases (3, appended), `packages/ui-contract/test/ship-at-world.test.ts`.

Golden change, on purpose: `tests/golden/contract.jsonl` gains the three new responses and the op list in `hello`
and the unknown-op error message names `ship_at_world`. No other line changed.

## Honest limits (all in docs/fudges.md)

- Isothermal exponential air: heating and g are right for the density met, but pass heights read high on Earth.
- Lift is not flown in aerocapture; a lifting ship has a wider corridor than shown.
- Chances are stand-ins (logistic in figure over limit; normal spread of pass height). Their basis travels with them.
- Powered landing and take-off use a gravity-loss stand-in and ignore drag.
- Stock ships have no shapes yet; the renderer's designer will write `shield`, `aero` and `landing`.

## Next

- Voyages: show these checks on the ship card at each stop, and offer the aerobraked arrival beside the burn with its
  propellant saved and its dangers (the route finder's risky option).
- Mount the renderer's `ShipPicture` on the Voyages ship card and the route card.

## Part 2: on the Voyages screen (23 Nov)

- `packages/views/voyage/src/AtWorld.svelte`: under an in-system voyage, "At <destination>": the ship drawn by the
  renderer's `ShipPicture` (from the ship record the op returns, fuels from `readPropellants`), then each act with its
  verdict in words, the dangers past caution with their rough chance as odds and what happens if it goes wrong, what
  would make it safer, and a "Why?" with the model and the chance basis. Acts with nothing to act on are one line.
- On the GM's seat with an adopted system it asks with `system` (the world's own air from S7); a fixture chart asks
  with the chart's inline system, whose worlds carry no air, so aerocapture and entry say the air is not known there.
- A torch course arrives braked by its drive (`v_inf_m_s` 0); a transfer arrives on the kernel's default (a Hohmann
  transfer from the next body in, not yet the leg's own arrival speed).
- Kernel: no `air` on a body now means "not known", said as such; a pressure of 0 means airless (Sol's Mercury and an
  adopted airless world carry that). No golden changed.
- e2e: `tests/ui/voyage.e2e.mjs` waits for the card, an act and the ship picture.

Limits: the leg's own arrival speed is not passed yet (the voyage plan does not carry it); belts get no card.

## Part 3: the leg's own arrival speed (23 Nov)

- `Course.arrive_v_inf_m_s` (sse-nav): speed over escape at the destination before the arrival burn. For a Hohmann
  transfer it is the textbook second-burn excess, for a Lambert arc |v₂ − v_world| at arrival, and 0 for a torch
  flight braked by its drive. In-system voyage legs carry it as `Leg.arrive_v_inf_m_s`; belt hops leave it unset.
- The Voyages "At <world>" card asks `ship_at_world` with that speed and says how the ship arrives ("arriving on
  this voyage's transfer arc"). A leg without one falls back to the kernel's Hohmann default as before.
- Tests: the Earth to Mars Hohmann arrives at d₂ (2.65 km/s), the cheapest Lambert arc within 10% of it, and a torch
  flight at 0.
- Goldens (on purpose, a new serialised field): `tests/golden/nav.json` (the chemical and torch plan digests only)
  and `tests/golden/contract.jsonl` (three plan_route/plan_voyage responses gain the field). No number moved.
