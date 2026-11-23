# X3g: the Traveller standard ships, the Mongoose notice, and World hooks

Example packs design (`/mnt/project-files/sse4/SSE4-example-packs.md`, section 4.1 and the table in section 3), Alex
"yes to all", 23 Nov. First items of its build order: the standard ships and the notice (Traveller pack), and the
World hooks tag set.

## Standard ships

`scripts/build-traveller-pack.py` (`STARSHIPS`, `SMALL_CRAFT`). Twelve starships and six small craft:

| Template | Hull dT | Jump | Thrust g | Cargo dT | Crew | Berths | Role tags |
|---|---|---|---|---|---|---|---|
| courier (Scout and courier) | 100 | 2 | 2 | 3 | 1 | 3 | explorer, courier |
| trader (Free trader) | 200 | 1 | 1 | 82 | 4 | 6 | trader |
| far_trader | 200 | 2 | 1 | 64 | 5 | 6 | trader |
| subsidised_merchant | 400 | 1 | 1 | 200 | 5 | 9 | trader |
| subsidised_liner | 600 | 3 | 1 | 120 | 9 | 21 | trader |
| yacht | 200 | 1 | 1 | 15 | 4 | 6 | |
| safari_ship | 200 | 2 | 1 | 11 | 5 | 6 | explorer |
| seeker | 100 | 2 | 2 | 25 | 2 | 2 | miner |
| lab_ship | 400 | 2 | 2 | 10 | 5 | 15 | explorer |
| patrol (Patrol cruiser) | 400 | 3 | 4 | 20 | 11 | 9 | patrol |
| mercenary_cruiser | 800 | 3 | 3 | 50 | 15 | 30 | patrol |
| corsair | 400 | 2 | 4 | 160 | 10 | 10 | pirate |
| launch | 20 | | 1 | 13 | 1 | 4 | |
| ships_boat | 30 | | 6 | 3 | 1 | 10 | |
| pinnace | 40 | | 5 | 6 | 1 | 16 | |
| cutter (Modular cutter) | 50 | | 4 | 30 | 2 | 4 | |
| shuttle | 95 | | 3 | 59 | 2 | 20 | |
| fighter | 10 | | 6 | 0 | 1 | 0 | |

- Existing ids are kept (courier, trader, far_trader, patrol, shuttle); labels now use the game's role names. The
  shuttle is now the 95 dT game craft (it was V3's generic 40 t shuttle). Heavy lifter stays as V3's generic craft.
- **Checking the figures.** Hull, jump and thrust are the long-published ratings. Cargo, crew and berths are round
  values written for this pack; no rulebook was at hand (the fan wiki refused automated reading), so they are in the
  fudge register as authored and unverified. A referee edits a template or a ship to match their edition.
- Each is a construct template (`create` puts one in a system; facts `traveller.hull_dt`, `traveller.jump`,
  `traveller.thrust_g`, `traveller.cargo_dt`, `traveller.crew`, `traveller.berths`, and `construct.nav_ship` naming
  its ship) and a ship for the voyage planner and the ship designer: `provides.other.nav_ships.ships`.
- **Pack ships, a general mechanism.** Any pack may list `nav_ships: {ships: {id: sse-nav Ship}}`. ui-contract
  `packShips(packs)` lists them in stack order (a higher pack's ship with the same id wins). The kernel already flies
  a ship object (`plan_voyage`, `plan_route`, `ship_at_world`), so no kernel change was needed.
- **How the numbers are made** (`nav_setting`, all in the fudge register): dry mass 4 t per displacement ton; the
  in-system drive is flown and drawn as a fusion torch (3000 km/s, as base-nav's rescue cutter) with 5% propellant,
  so the designer gives the hot ships their radiators, until the future tech packs (FT7, FT8) let a setting skip
  reaction mass and waste heat as marked fictions; the jump is the setting's rule (fictional): a tenth of the hull's
  volume in liquid hydrogen per parsec (density from the sheet's h2 row), a week a jump, whole parsecs, from 100 star
  diameters out. Cargo mass is the hold's volume at base-ships' stowage density, so the designer's hold is exactly
  the cargo tonnage in dT. No deck plans, no publisher art: hulls come from the ship designer.

## Notice

Mongoose's Fair Use Policy (2025 PDF, linked from https://www.mongoosepublishing.com/pages/traveller-licensing)
asks fan work to carry its notice. The pack's `about` no longer names Far Future Enterprises, and
`provides.other.notice {text, source}` carries the policy's notice word for word. `packSummary` reads `notice` (any
pack may carry one) and the Packs screen shows it under the pack as a "Notice" disclosure (`data-pack-notice`).

The policy also asks that Mongoose be notified (an email to its sales address) and says the contents are for
personal, non-commercial use. Alex agreed (23 Nov) to both: he emails Mongoose himself, and the Traveller pack alone
is now CC BY-NC 4.0 (`licence` in the pack, `packs/traveller/LICENCE.md`, a line in the README); the rest of the
repository, the build script included, stays Apache-2.0.

## World hooks

`scripts/build-hooks-packs.py` writes `packs/world-hooks/pack.json` (`sse.world-hooks`, presentation): thirteen
`hook.*` labels, each an original one-line story prompt (`about`) with the physics it read (`why`), derived by rules
over a body's facts under both namings (SI and the chart room's): long night, backwards sun, on its side, crushing
air, thin air, furnace, oxygen rain (at the sheet's O2 boiling point), ocean world, heavy world, featherweight, wild
orbit, sun-filled sky, giant in the sky (reads the parent's mass). Labels only, never physics; tier scanned. Offered
on the Packs screen ("Add world hooks"), not in the opening stack.

Facts it can read are the ones a world carries (catalogue and chart room); tidal locking needs a lock fact or the
orbital period, which bodies do not carry yet, so "tidally locked, people live on the terminator" waits for it.

## Checks

- `crates/sse-kernel/src/voyage.rs` `the_traveller_standard_ships_plan_voyages`: all 18 ships plan Earth to Mars in
  Sol, and every starship plans a voyage between stars with no leg longer than its jump rating (jump-1 ships with no
  star within a parsec say so).
- `crates/sse-kernel/src/hooks_tests.rs` `world_hooks_read_off_sol_and_kestrel`: Venus, Mars, Uranus, Mercury, the
  Moon, Earth (none), the Sun (none); Kestrel's moons of Vault, Hollow, the inner world.
- `tests/ui/traveller-ships.test.ts`: the list is whole, templates point at ships with matching ratings, every role
  has a ship, a higher pack's ship wins; the designer draws every ship in every style with a hold of exactly the
  cargo dT and a jump tank of a tenth of the hull per parsec; the notice is Mongoose's and the Packs screen lists it.
- Golden `tests/golden/traveller-codes.json`: the pack hash only (new templates, ships and notice).

## Next

- The voyage pickers (Between stars, the route console) list pack ships beside base-nav's stock (travel thread,
  asked); then a picture of the standard ships in both styles.
- Tidal lock and terminator hooks once bodies carry a lock fact.
