# Status: Heat flow

Owner thread: "Fix low heat flow in worlds" (opened 26 Nov 2026 by the coordinator). Covers: HF, the heat a world makes and loses (thermal history under a lid, the heat loss every surface reader uses, each world's own tidal response).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`.

Why the track exists: three threads on 26 Nov found worlds too cold inside (Io about a fifth of its heat, Earth under half, Enceladus a third). They are separate faults: the surface readers use the heat made, not the heat lost, so Earth's own cooling is missing; and every world shares one tidal response, k2/Q = 0.3/100. Switching readers to the heat lost first exposed a third fault under both: every lid world's mantle sat at the 2,200 K melt cap from about 1 Gyr to today.

| Package | State | Log |
|---|---|---|
| HF1 lid worlds vent at their solidus: under a stagnant lid the mantle can be no hotter than the dry peridotite solidus (Hirschmann 2000) at its lid's base, the lid as thick as the rock that conducts what the mantle loses; melting carries the rest out as volcanism and the history counts it in the surface loss | done (PR 660, on main 27 Nov 2026 07:27 UTC) | packages/HF1.md |
| HF4a one tidal response block `sse.tides.maxwell_response`: a world's Im k2 and Re k2 from its rigidity, viscosity, density, gravity, radius and tidal frequency (homogeneous Maxwell body); nothing reads it yet; shared with the small-moon shape thread | done (with HF1, PR 660) | packages/HF4a.md |
| HF4b melt in the thermal history, built per the accepted design (`design/SSE4-melt-and-heat-design.md`) in turns: HF4b.0 numbers and blocks (sheet rows, formula blocks, reach lines only), HF4b.1 the lid as kept state with heat fields per step, HF4b.2 melt; then C3 derived start temperature | HF4b.0 in review | packages/HF4b0.md |
| HF4c tides read each world's own response: `sse.tides.eccentricity_heating` takes Im k2 from HF4a fed by HF4b's state; the classifier's own copy (k2 0.3, Q 100) goes | after HF4b | |
| HF2 every world reads the heat it loses: regimes, life's vents, plumes, stains and plates read the thermal history's surface loss (made plus the world's own cooling), the history and the regimes marching forward together | next | |
| HF3 folded into HF4a and HF4c (26 Nov: Io's heat needs melt, so tides cannot be fixed before HF4b) | folded | |

## The chain of calibrated fudges under lid worlds (found 26 Nov, before HF2's lid half)

Making every world read the heat it loses (HF2) works for Earth (20 TW made, 35 TW lost, about 46 measured, unverified) but breaks 11 lid-world surface tests (the Moon's lava plains and tubes, Mars's rift, Europa's chaos, crater renewal). Each link below was calibrated with the others in place, so none can be fixed alone:

1. **Steady lid from day one.** `core_history` uses the stagnant-lid scaling (Solomatov and Moresi 2000) from the magma ocean's end, as if the lid were already fully grown. A young lid is a cooling half-space and loses k ΔT / √(πκt) (Turcotte and Schubert 2014, eq. 4.124), so real young lid worlds lose heat fast. Without it, the Moon and Mars store their early heat instead of losing it.
2. **Dry mantle 100 times stiffer** (`dry_viscosity_factor` 100, `degas_gyr` 1; fudge register). Under a lid, the mantle dries within about 1 Gyr and its convected flux falls about fivefold, so lid mantles warm back up to their melt limit (2,000 to 2,150 K today against about 1,600 to 1,700 K in published Mars models).
3. **Melt cap, not melt.** HF1 replaced the 2,200 K cap with the solidus at the lid's base, but still pins the mantle there: no melt fraction, no melt-weakened viscosity, no crust taking heat-producing elements up. So no tidally heated world can hold the partly molten layer that dissipates Io's tides.
4. **Volcanism read against heat made.** Lid-world volcanism (plains rates, shields, flood lavas, tubes; `regime_features` q_ref 0.06 and its siblings) are "rule-of-thumb values for lid worlds" calibrated against radiogenic heat made. Physically, volcanism is the melt the history vents.
5. **Mars's field date rests on 1 and 2.** With young lids cooling as half-spaces (tried, not committed), Mars's mantle cools to about 1,600 K by 0.5 Gyr, its core-mantle flux collapses (lower mantle viscosity read at the upper mantle's temperature times 100), and its field dies 4.5 Gyr ago, outside the 3.6 to 4.1 Gyr ago that `core_tests` guards. Chase to cause; do not move the range.

Order agreed with the coordinator (26 Nov): HF3 (tides), then the lid worlds properly (one mechanism: young lids, melt, volcanism from vented melt), then HF2 lands on top. No second heat number in the meantime.

## What was tried before the melt design (26 Nov, fixtures in `sse-surface/src/scenario.rs`)

1. **Surfaces reading heat lost (HF2, not committed).** Earth 20 TW made, 35 TW lost (about 46 measured, unverified). The Moon, Mars and the icy moons store their early heat: 0.013 to 0.018 W/m² lost against 0.05 to 0.08 made, failing 11 surface tests.
2. **Young lid as a cooling half-space (not committed).** q = max(steady stagnant-lid flux, k ΔT / √(πκt)). Early Mars loses 24 TW at t = 0; its mantle falls to 1,625 K by 0.5 Gyr, then warms back to 2,150 K as it dries. Its core-mantle flux collapses (lower-mantle viscosity read at the upper mantle's temperature × 100), so its field dies at 0.02 Gyr, outside the guarded 3.6 to 4.1 Gyr ago. The Moon's field dies at 0.7 Gyr.
3. **Io as a homogeneous Maxwell body** (μ 6.5e10 Pa, ρ 3,530, g 1.796, R 1,821.6 km, n 4.11e-5).
   - A = 19μ/2ρgR ≈ 53. Elastic k2 is 0.028. Peak Im k2 is 0.74, at η ≈ 2.7e13 Pa s.
   - At HF1's solidus temperature (about 1,980 K), η ≈ 1e19 to 1e21, so Im k2 ≈ 3e-6.
   - Lainey's 0.015 (from memory) needs η ≈ 3e11 at μ 6.5e10. With μ softened to 1e9 it needs η ≈ 1e15, giving about 0.009.
4. **Heating balancing convection** (Fischer and Spohn style, Ra_c 1,000, whole-mantle ΔT) overshoots: Io 22 W/m², Europa 1.4. Enceladus and the Moon have no hot equilibrium (peak heat 1.4 and 0.019 W/m²).
5. **Orbital equilibrium** (Meyer and Wisdom 2007 for Enceladus: about 1.1 GW × 18,000 / Q_Saturn) explains the measured Io and Enceladus heat. It needs the host's Q and the resonance partner, which the kernel does not hold. If an honest Io needs them, log it; never stand a constant in for them.
