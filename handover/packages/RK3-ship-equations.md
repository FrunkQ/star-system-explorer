# RK3 (ship sizing share): ship sizing formulas as named Equations

Plan: `handover/design/SSE4-atomic-rockets-plan.md` section 5 and the RK3 row (question 4). Agreed split: the travel
thread takes the flight formulas in `sse-nav`; this share is ship sizing (thrust power, radiators, and the rest of what
the ship designer sizes parts with).

## What landed
- **`packs/base-physics/formulas/ship.json`**, family `sse.ship` (home agreed with the kernel thread: only
  base-physics carries formula families; base-ships keeps the data), five cited blocks with worked examples:
  - `sse.ship.jet_power`: P = ½ F v_e (Sutton and Biblarz 2017, section 2.3). 530 N at 20 km/s gives 5.3 MW.
  - `sse.ship.stowed_volume`: V = m / ρ, for tanks and holds alike. A tonne of liquid hydrogen fills 14.1 m³.
  - `sse.ship.crew_volume`: V = n h / g (NASA-STD-3001 Vol. 2). Four people at 25 m³ in modules half free: 200 m³.
  - `sse.ship.waste_heat`: Q = w P, the drive family's waste share from base-ships.
  - `sse.ship.radiator_area`: A = Q / (n ε σ T⁴), σ from the sheet (Gilmore 2002, chapter 6). 1 MW at 600 K,
    ε 0.9, two faces: 75.6 m².
- Sixteen `ship.*` quantity kinds described in `sse-atoms` `KIND_DESCRIPTIONS`; `packs/base-physics/pack.json`
  regenerated (`SSE_BLESS=1 cargo test -p sse-atoms`) and copied to `tests/fixtures/packs/base-physics.json`.
  First-wave atom count 122 to 127.
- The TypeScript sums that still do this work (renderer `ship-design.ts` `needsOf`, ui-contract `radiatorArea` on
  the Power card) are listed in `docs/fudges.md` under "Candidates to move into flows": they should read a kernel op
  that sizes a ship through these blocks, as entry and landing already do through `ship_at_world`. A test checks the
  Power card's radiator sum against the block's worked example meanwhile.

- **`reject_k` on the kernel's heat rows** (`sse-nav` power `Flow`, RK13's follow-on): each heat row says the
  temperature it is rejected at, so the Power card no longer guesses which radiator it goes to.

## Golden
Changes on purpose: `tests/golden/contract.jsonl`, the base-physics pack hash and atom count in every case that loads
base-physics, and the `ship_power` answer (its heat rows gain `reject_k`). No world's values change. In the golden lane (one at a time, coordinator's order).

## Next
- A kernel op (or a field on `ship_at_world`) giving the ship's sizes through these blocks; the renderer reads it.
- Engine depth on the Power card: press a band or a reading to open its block.
