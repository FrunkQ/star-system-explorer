# MC4: fuel gauges in the Manifest

Track MC. Owner thread "Build ship manifest and crew roster". Alex, 24 Nov 14:20, answering "a fuel level on the ship you can change in play, used up by voyages": "fuel gauge (and an edit fuel) to top up, etc- yup go for it!"

## What it does

- The Manifest side of the Aboard panel shows a gauge per tank the ship has: reaction mass (`wet_t - dry_t`, labelled with its fuels) and, with a jump drive, jump fuel (`jump.tank_t`, labelled with the jump fuel). The ship's numbers come from the kernel (`shipRecordOf`: `ship_at_world` with no worlds resolves a stock id or a pack ship).
- The level is a fact on the construct: `construct.propellant_t` and `construct.jump_fuel_t`, in tonnes. Unset reads as full, the kernel's own default ("propellant now, default full"; a jump drive's "fuel aboard now"). A level above the tank reads as full.
- The GM sets a level or presses "Top up" on any desk: fuel is what is aboard, so it changes in play (Alex's desk rule, MC3). The fields are drawn, not listed in the card's facts.
- `fuelNow(ship, kept)` (ui-contract manifest.ts) returns the ship with `propellant_t` and `jump.fuel_t` at the kept levels: a voyage planned from it starts from what is in the tanks. Joining that into Voyages and drawing the tanks down after a voyage is the travel track's; asked.

## Checks

- `tests/ui/manifest.test.ts`: gauges full by default, kept levels read and clamped, `fuelNow`, no tanks no gauge.
- `tests/ui/manifest.e2e.mjs` (by hand): two gauges on the courier, full; jump fuel set to 5 t in Explore (not listed as a fact), Top up back to 19.838 t.
- No Rust change; no golden change. Screens: /mnt/project-files/sse4/handover-screens/MC3-screens/ (fuel-low-desk.png).
