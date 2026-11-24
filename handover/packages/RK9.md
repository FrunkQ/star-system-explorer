# RK9: the power ledger (four buses)

Track RK (Atomic Rockets plan, section 11), second UI track's share.

## Part 1 (24 Nov): the ledger in NAV

`crates/sse-nav/src/power.rs`, tests in `power_tests.rs`. No golden change:
the new Ship fields are optional and skipped when empty, so every saved ship
loads and writes as before.

A Ship now may carry:

- `power[]`: `{id, kind, electric_w, thermal_w?, reject_k?, mass_t?, law}` with
  `law` one of `{kind:"constant"}` (default), `{kind:"sunlight", ref_flux_w_m2}`,
  `{kind:"decay", half_life_y, since_s}`, `{kind:"fuel", fuel}`.
- `engines[]`: `{engine, count}` where `engine` is a pack engine id or an engine
  written in place: `{name, fuels, thrust_n, exhaust_km_s, energy_from
  (fuel|electric|beam|sunlight), electric_w?, efficiency?, reject_k?, mass_t?,
  exhaust_range_km_s?}`.
- `loads[]`: `{name, electric_w, reject_k?, mass_t?, role?, off?}` (`role` is a label, "weapon" or "sensor", for grouping bands in the Flow view).

`power::ledger(sources, mounts, loads, defs, Conditions{flux_w_m2, t_s})`
returns Flow rows `(from, to, amount_w, kind)` for the Flow view (RK13, chart
room), watts made and wanted, spare (negative when short), each engine's
thrust, jet and electric share, heat to reject by temperature, the
`nowhere_w` shortfall and why-lines. Loads are served first; electric engines
share what is left and their thrust scales with the share
(F = 2ηP/v_e at full draw; `thrust_from_power`). Where an engine's stated jet
power (½Fv_e) is more than η times its draw, the difference is a row from
"Nowhere": the ship still flies on its stated numbers (danger is drama), and
the GM sees the books do not close. `readings(ledger, mass_t)` gives accel (g)
and thrust-weighted exhaust speed. Ships with no `engines` keep their typed
`accel_g` and `exhaust_km_s`; nothing reads the ledger in place of them yet.

Checked against: Dawn's NSTAR (92 mN at 2.6 kW, Isp 3100 s, η 0.54; thrust at
3 au with a 10 kW array falls to the share left after the bus), Voyager's
MHW-RTGs (470 W, Pu-238 half-life 87.7 y: about 325 W at 47 years; the real
craft lost more to thermocouple ageing, which is not modelled, see fudges),
SSE3's Hall thruster (530 N at 20 km/s is a 5.3 MW jet on 0.1 MW: 5.2 MW from
nowhere), RL10 (110 kN, no electric bus). Every watt in ends as jet or heat
(checked in the solar test).

Fudges: decay sources follow the isotope only (no thermocouple ageing);
fuel-cell sources are not drawn down here (the propellant bus counts mass in
NAV's rocket equation as before); efficiency left out counts as 1, the most
generous check.

## Next

- Part 2 (kernel thread): ship import runs the balance check and shows the
  Nowhere band; NAV's `accel_g` and `exhaust_km_s` read from `engines[]` when
  present, at the voyage's flux.
- RK12 and RK14 (light thread) can build on `power::ledger` now.
