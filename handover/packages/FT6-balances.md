# FT6 keeping the books: balance checks as blocks (design 15.1, 15.2, 15.6)

Accepted by Alex on 23 Nov (questions 19, 20 and 24). Before this, "fictional by arithmetic" meant one figure past its own ceiling. Now it also means a balance between figures that does not close. It is still never a refusal.

## What changed
- **Each balance in `packs/base-tech` is an Equation block** (`check`, the same `sse.math.equation` config every physics block uses). Each gives `demand`, what the design asks for, and `supply`, what pays for it, from the design's own quantities, in the units their names say. Names, units and dimensions are checked, and the worked examples run, when the stack loads. A balance also carries:
  - `closes`: `at_most` or `equal`, within `tolerance`;
  - `defaults` for quantities a design may leave out;
  - `look_up` for constants, taken from master data sheet rows, never typed;
  - `says`, the words used when it fails.
- **The five checks** (`scripts/build-base-tech.py`, `CHECKS`):
  - **Fastest exhaust the fuel's energy can pay for** (energy-pays-exhaust). The relativistic form is c·√(1 − 1/(1 + E/c²)²), which is √(2E) at low speed. Source: Einstein 1905, §10.
  - **Power the jet carries away**: ½ × thrust × exhaust speed against the plant's power. It is bookkeeping.
  - **Power out against power in**: jet + waste + stored + loads equals the plant's power, within 2%. Stored and loads default to 0.
  - **Thrust what is thrown can give**: thrust ≤ mass flow × exhaust speed + P/c. Plant power defaults to 0, so thrust with nothing thrown fails.
  - **Work to pump heat uphill to the radiators** (the Carnot heat pump): a radiator hotter than its source needs at least Q(T_r − T_s)/T_s of pump work paid for. Pump power defaults to 0. Source: Clausius 1854.
- **The hydrogen-oxygen rocket sets `fuel_j_per_kg`** to the sheet's lower heating value (`HYDROLOX_VAPOUR_J_KG`), and lists it as fixed, so a chemical drive's books can be checked from the advance alone.
- **Kernel** (`crates/sse-kernel/src/tech.rs`):
  - `Tech.balances` is read from the stack. A balance with no `check` stays text only.
  - `tech_reading` parts take `quantities`, and `needs` is now optional.
  - Each part's checks read, in this order: its quantities, its needs, then what the advances it leans on set. A check runs only when something of the part's own goes in.
  - The reading gains `balances`. Each entry has part, balance, name, law, demand, supply, unit, closes, times, inputs, and words when it fails, for example "Fastest exhaust the fuel's energy can pay for: the exhaust is 7.73 times as fast as its fuel's energy can make it, which breaks conservation of energy".
  - The reading also gains `unchecked`: the checks a part couldn't run, with what they were missing.
  - A failed balance makes the reading `fictional` and turns that part's chips fictional.

## Checks (`sse-kernel/src/tech_tests.rs`)
- **`the_books_must_close`**:
  - A "chemical" drive at 40 km/s fails energy-pays-exhaust (supply 5.18 km/s, 7.73 times) and is fictional. The J-2's 4.168 km/s closes.
  - Thrust with nothing thrown fails momentum.
  - A part with only a thrust lists jet-power as unchecked, missing plant_w.
- **`heat_flows_downhill_unless_a_pump_is_paid_for`**:
  - 100 kW from 300 K to a 600 K radiator needs 100 kW of pump work. Unpaid, it fails; paid, it closes.
  - A 276 K radiator over a 300 K source needs no pump.
- **`real_craft_close_their_books`** (design 15.6): every balance the `real_craft` rows can feed closes.
  - ISS radiators, heat-downhill: ammonia at 276 K under a crew's 300 K. The 300 K is the figure's built mark, the row's one missing quantity.
  - Saturn V S-II and S-IVB, energy-pays-exhaust: the J-2 at 81% of what the fuel pays for.
  - The others are unchecked for want of mass flow or plant power. They are never failed.

## Golden
Nothing moves. Physics doesn't read tech, and the contract has no tech_reading case.

## Next
- Radiator sizing (Stefan–Boltzmann area for a heat load, design 15.2: about 120 m² for 100 kW at 300 K), and the rocket equation and tank sizing for the Saturn V stack rows. These are sizing blocks, not balances, and belong with the ship generator's parts (RK4).
- Mass flow for the Saturn V rows (propellant over burn time) so momentum and jet power can be checked against flown stages.
