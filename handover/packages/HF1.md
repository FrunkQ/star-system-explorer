# HF1: lid worlds vent at their solidus

Thread "Fix low heat flow in worlds", 26 Nov 2026. Code: `crates/sse-surface/src/core.rs` (`core_history`).

## The fault

The MF1 thermal history caps a mantle at 2,200 K ("melting vents the excess", a stand-in for a melt model). Under a stagnant lid the convected heat is small, so every lid world's mantle warmed to the cap within about 1 Gyr and sat there to today: Mars, the Moon, Venus and Mercury all ended at 2,200 K, and the heat the cap removed was counted nowhere. Earth (moving plates) never reached it.

## The change

- Under a stagnant lid the hottest a mantle can be is the solidus at its lid's base. The solidus is the dry peridotite fit of Hirschmann 2000 (T °C = −5.104 P² + 132.899 P + 1120.661, P in GPa, 0 to 10 GPa; coefficients from memory, the Consensus search is out until 1 Dec, so **unverified**). The lid's thickness is the rock that conducts what the mantle's top loses, k ΔT / q. A hotter mantle loses more, so its lid is thinner and its base melts sooner; the self-consistent temperature is found by bisection each step (an explicit estimate oscillated step to step).
- Moving plates keep the 2,200 K cap: there, ridge melting is part of the plates' own cooling.
- Heat that would lift the mantle past that limit leaves as volcanism (a heat pipe, Moore and Webb 2013) and is now part of the step's `q_surface_tw`; before, it vanished.
- The history still marches on the body's own grid from the magma ocean's end to its age; no new clock.

## What moved (fixtures, today)

| World | Mantle before → after | Surface loss before → after |
|---|---|---|
| Mars | 2,200 → 2,150 K | 1.55 → 3.7 TW |
| Moon | 2,200 → 2,047 K | 0.30 → 0.3 TW |
| Mercury | 2,179 → 2,017 K | 0.71 → 0.7 TW |
| Venus | 2,200 → 2,200 K (episodic, keeps the cap) | 7.1 → 22 TW |
| Earth | unchanged | unchanged (35 TW) |

Mars's field still dies about 3.9 Gyr ago. **Mercury's dynamo still dies at once** (0.06 Gyr after its start, was 0.02): its known miss is not this cause. Earth's inner core is unchanged (still about 3,000 km against 1,221; a separate fault, the nucleation fudge).

## Honest state

Lid mantles are still hot (2,000 to 2,150 K against roughly 1,600 to 1,700 K in published Mars models): the stagnant-lid flux is small mostly because of the dry-mantle factor of 100 (fudge register), which a mantle water budget should replace. Not widened here.

Goldens re-blessed on purpose: `tests/golden/surface.json` (core history numbers only: mantle temperatures, surface loss, dynamo dates for lid worlds) and four core lines of `tests/golden/contract.jsonl` (two `formation_ran`, one `system_resolved`, one `surface_surveyed`: the field and core facts of lid worlds). Core golden lane: merges alone.

Citations added to `core::REFS`: Hirschmann 2000 and Moore and Webb 2013, both written from memory (DOIs included), **unverified** until the paper search returns on 1 Dec.
