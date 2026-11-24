# RK13: the Flow view and the Power card

Plan: `handover/design/SSE4-atomic-rockets-plan.md` section 16 and question 18 (a general Flow view, the Power card
first), 19 (broken books as a dashed band from "Nowhere"). Kernel side was RK9 parts 1 to 3 (PRs 389, 390).

## What landed
- **`packages/ui-contract/src/flow.ts`**: the general layout. Any ledger hands rows `{from, to, amount, kind, dashed?, k?}`;
  `layoutFlow` puts nodes in columns (given, or longest path from the sources via `flowRowColumns`), sizes bands by
  amount, draws a band under 1% of the largest column as a hairline (plan 16, "honest about scale"), and runs top to
  bottom when asked (phones). It never sums physics.
- **`packages/views/flow`**: `Flow.svelte` (draws a layout with theme tokens; Curious depth names only, Science and
  Engine add figures) and `Power.svelte`, the Power card.
- **`packages/ui-contract/src/power.ts`**: `powerFlowRows` turns the kernel's bus rows into the card's three columns
  (Comes from, Used by, Ends up as): each draw on the electric bus is shared among the sources in proportion to what each
  makes; a plant's own heat is a part ("Lost in plant"); heat ends at one radiator node per temperature; propellant is
  "Fuel"; "Nowhere" rows are dropped for players. `powerReadings` gives the readings: thrust, power to spare, a
  radiator area per temperature (hot, warm, cool; `A = W / (faces x e x sigma T^4)` with the base-ships pack's radiator
  rule and the sheet's `sigma_SB`), how visible (watts given off, and the flux a million km away against `S_sun`).
- **Session `shipPower`** (kernel `ship_power`) on `Session` and `ViewSession`.
- **Which radiator**: the kernel's heat rows name the part but not its temperature, so `heatTemperatures` reads each
  part's `reject_k` from the ship record that was asked about, and a part with none goes to the one temperature in
  `heat_by_k` no stated part explains (the kernel's default). If that cannot be told, the heat ends at "Radiators".
  Kernel change left for the golden lane: a `reject_k` on heat rows (the contract golden holds a `ship_power` answer,
  response 361, so it changes that hash); then `heatTemperatures` becomes a fallback.
- **Mount**: one "Power" button and panel on the Ships and stations card (`views/constructs/src/Constructs.svelte`),
  agreed with "Build ship manifest and crew roster". Shown for any construct with a ship record, every seat.

## How it behaves
- A ship with fitted `power[]` or `engines[]` gets the flow; a typed-figure ship gets one line saying to fit parts on
  the Build desk.
- Live, nothing kept: a "Starlight as at" slider (0.3 to 10 au from the Sun) when a source runs on sunlight, and a
  switch per load. Each change re-asks `ship_power` with the edited record.
- GM seat: a dashed band from "Nowhere" and the kernel's sentence (`nowhereWords`, the balance's `says` as tooltip).
- Engine depth: the ledger's `why` lines under the readings.

## Tests
- `packages/ui-contract/test/flow.test.ts`: layout (columns, widths, hairline, vertical), radiator area, and against
  the real kernel: SSE3's Hall tug (5.2 MW from Nowhere, GM only), a Dawn-like solar craft thinning at 3 au, switching
  a load off giving the engine more thrust.
- `tests/ui/power.e2e.mjs`: the Power button on a Traveller courier's card opens the card, which says it flies on typed
  figures; no sideways scroll at phone width.

## Not done / next
- Engine depth "press a band to open its Equation block" waits for the ship formulas as named Equations (RK3).
- "Show it without" (FT question 10) and throttle are not on the card yet.
- No stock ship has fitted engines yet, so the browser shows the card on the Paper spacecraft pack's ships (RK6) or an
  SSE3 import with power plants.
- Other Flow users (a world's energy budget, an element's journey, a habitat's air) are their owners' to add.

## Integration points
- Any view: `import { Flow } from "packages/views/flow/src/index.ts"` with rows from its own ledger.
