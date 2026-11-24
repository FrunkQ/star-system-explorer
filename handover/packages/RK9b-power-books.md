# RK9 part 2: the power books in the kernel, and fitted engines setting the drive

This is the kernel thread's half of RK9. Part 1, the ledger in `sse-nav` `power.rs`, is the travel thread's (`RK9.md`). The plan was agreed with the travel thread on 24 Nov.

## What changed
- **`Ship::effective(defs, flux_w_m2, t_s)`** (`sse-nav`):
  - With `engines[]`, the four-bus ledger at that flux and time sets `accel_g` (thrust over the mass aboard, dry plus propellant) and `exhaust_km_s` (weighted by thrust).
  - With no engines it returns `None`, and the typed figures stand.
  - `sunlit_power()` says whether any source follows starlight.
- **Engine designs from the stack**: `provides.other.engines`, a map from id to `EngineDef`, read bottom to top, with later packs winning. A mount whose engine id the stack lacks is named in `why`; it is never a failure.
- **`ship_power {ship, flux_w_m2?, at_s?}`** (Trivial):
  - `ship` is a template id or a ship in full. The flux defaults to 1361 W/m² and the time to 0.
  - It returns `{ship, flux_w_m2, accel_g, exhaust_km_s, ledger, balance, fictional, why}`.
  - `balance` is shaped like `tech_reading`'s: `power-books`, demand (what the stated engines' jets carry), supply (what the ledger gives them), `W`, `at_most`, tolerance 2%, `ok`, `times`, and `says` when the books do not close. When they don't, the ship is marked fictional on the GM view, and it still flies on its stated numbers.
- **The drive in NAV**:
  - `plan_route` and the in-system `plan_voyage` fit the ship by `powered` at the dimmer of the leg's two worlds' starlight.
  - Between the stars, `plan_voyage` reads no world's starlight, so sunlight power counts as none.
  - Each plan carries `power` (the books) when the ship has fitted engines. Plans for ships without fitted engines are unchanged.
  - Fudge register: "Power books (track RK, RK9)".

## Checks (`sse-kernel/src/power_books_tests.rs`)
- SSE3's Hall tug (530 N at 20 km/s on 0.1 MW): 5.2 MW from nowhere, so it is fictional and the books "do not close by 5.2 MW". It still flies at its stated thrust.
- A Dawn-like solar ship closes its books at 1 au. At 3 au its array gives less than the bus takes, so it barely thrusts.
- Earth to Mars reads the flux at Mars (about 590 W/m²), and a typed ship's plan has no `power`.
- An RL10 by id from a stack pack sets 4.5 km/s and its acceleration. An unknown id is named in `why`.

## Golden (on purpose)
`tests/golden/contract.jsonl` changes in three places:
- line 1: the op list gains `ship_power`;
- line 41: the unknown-op message names it;
- the new `ship_power` case at the end (the Hall tug, with a quarters load and 60% efficiency).

No other response changes, because no stack or fixture ship has fitted engines yet.

## Next
- The SSE3 construct reader (`power_plants` output_MW, `powerDraw_MW`) goes in the converter or the pack builder; the travel thread agreed this.
- Voyages shows power spare and the Nowhere mark on the GM seat (travel thread).
