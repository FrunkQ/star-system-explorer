# Status: Heat flow

Owner thread: "Fix low heat flow in worlds" (opened 26 Nov 2026 by the coordinator). Covers: HF, the heat a world makes and loses (thermal history under a lid, the heat loss every surface reader uses, each world's own tidal response).

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`.

Why the track exists: three threads on 26 Nov found worlds too cold inside (Io about a fifth of its heat, Earth under half, Enceladus a third). They are separate faults: the surface readers use the heat made, not the heat lost, so Earth's own cooling is missing; and every world shares one tidal response, k2/Q = 0.3/100. Switching readers to the heat lost first exposed a third fault under both: every lid world's mantle sat at the 2,200 K melt cap from about 1 Gyr to today.

| Package | State | Log |
|---|---|---|
| HF1 lid worlds vent at their solidus: under a stagnant lid the mantle can be no hotter than the dry peridotite solidus (Hirschmann 2000) at its lid's base, the lid as thick as the rock that conducts what the mantle loses; melting carries the rest out as volcanism and the history counts it in the surface loss | in review | packages/HF1.md |
| HF2 every world reads the heat it loses: regimes, life's vents, plumes, stains and plates read the thermal history's surface loss (made plus the world's own cooling), the history and the regimes marching forward together | next | |
| HF3 each world's own tidal response: Im(k2) from the world's interior instead of one k2/Q for all; Io, Enceladus and Europa as checks, not special cases | after HF2 | |
