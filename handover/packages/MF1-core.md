# MF1: the core's heat history and what powers the dynamo

Design: `handover/design/SSE4-magnetic-fields-design.md`, MF1 (accepted 22 Nov). Completes B1d part 2. Retires F-S30.

## What it does

Every rocky world with an iron core now carries `SurfaceEnvelope.core` (`sse_surface::core::CoreHistory`,
version `core-history-v1`): a dated history of its mantle and core temperatures, the heat leaving the core,
the heat conduction alone carries along the core's adiabat, the power left to stir a dynamo, the inner core's
radius and the field inside the core, every 20 Myr from the end of its magma ocean to now. Events
`dynamo-start`, `inner-core-birth` and `dynamo-death` each carry a dated sentence saying why.

The chain, one step at a time (Stevenson et al. 1983; Driscoll and Bercovici 2014):

1. **Core size from iron.** The core's density is iron's Vinet equation of state (Dewaele et al. 2006) at its
   mean pressure, less a share for light elements; radius and pressure are iterated together. No fitted
   exponent. Earth 3,431 km (real 3,480), Mars 1,779 (InSight 1,830), Mercury 1,886 (2,020), Moon 358 (about 350).
2. **The mantle** loses heat through its top by mobile-lid or stagnant-lid convection (Solomatov and Moresi
   2000), read from the world's own regime timeline, and gains it from radioactivity and tides (the envelope's
   own heat). A stagnant lid dries the mantle (1 Gyr e-folding) and a dry mantle is 100 times stiffer
   (Hirth and Kohlstedt 2003); moving plates keep it wet.
3. **The core-mantle boundary layer** carries q = k ΔT / δ with δ from the critical Rayleigh number.
4. **The core** pays for that from its heat capacity and, once an inner core exists, from freezing (latent and
   gravitational energy). It starts 400 K above the mantle liquidus at its top (Andrault et al. 2011).
5. **The inner core** nucleates when the centre is 150 K below the alloy's melting point (Anzellini et al. 2013,
   lowered by light elements), then follows the melting curve against the core adiabat T(r) = T_cmb exp((R_c² − r²)/D²).
6. **Dynamo power** = the convected flux (`sse.dynamo.convected_flux`: q − kαgT/c_p) plus compositional buoyancy
   (`sse.dynamo.compositional_power`: light elements from the inner core, magnesium oxide from the core top,
   O'Rourke and Stevenson 2016). Positive power runs a dynamo; its strength is Christensen's (2010) law via
   `sse.magnetic.dynamo_field`.

New blocks in `sse-atoms` (`magnetic.rs`): `sse.dynamo.convected_flux` and `sse.dynamo.compositional_power`
(71 atoms). The body-magnetism flow keeps its interface: its `dynamo_heat_flux` input is now documented as the
power these blocks give, which MF2 wires from the history into resolve.

## Evidence (crates/sse-surface/src/core_tests.rs)

| World | Result | Target |
|---|---|---|
| Earth | field from 4.4 Gyr ago, never stops; inner core born 0.90 Gyr ago; core heat flow today 7.9 TW against 12 TW along the adiabat (a thermally stable top, kept going by inner-core buoyancy) | continuous 3.5 Gyr; inner core 0.5 to 1.5 Gyr old |
| Mars | dynamo from the start, dies 3.96 Gyr ago, core stays liquid | off between 4.1 and 3.6 Gyr ago |
| Venus | dies 2.14 Gyr ago, "the core is not cooling fast enough (3.8 TW leaves it, conduction alone carries 9.7 TW)", no inner core | off today, core not cooling |
| Moon | dies 4.06 Gyr ago | lunar dynamo ended by about 3.5 Gyr ago (strong phase) |
| Speed | about 0.5 ms per world (release) | under 1 ms |

Known misses, on the fudge register: Mercury's dynamo dies 4.4 Gyr ago (it has a weak field today, thought to
need a stable layer or iron snow: MF2's per-layer ledger); Earth's inner core today is about 3,000 km, not
1,221 km, because the nucleation jump lands on a melting curve nearly parallel to the adiabat. The tuned values
(melting drop per light element, nucleation undercooling, dry stiffening, lower-mantle factor) are each one
fudge row in `docs/fudges.md` (Interiors and heat), all stand-ins until MP21's iron rows and a mantle water budget.

## Goldens

Re-blessed on purpose: `tests/golden/surface.json` (every envelope now carries its `core` history, so the
envelope hashes move) and `tests/golden/contract.jsonl` (envelopes and the new quantity kinds).
`packs/base-physics/pack.json` and its fixture copy gain the five new quantity kinds; the body-magnetism flow
keeps its ports. Atom count 69 to 71.

Typed copies (MP22): core.rs reads G from `sse_core::constants`, and Earth's mass and radius and the gas
constant from the crate's existing labelled copies (made `pub(crate)`, ledger third column updated), so no
copy is added.

## Next

MF2 takes `q_dynamo` per layer into the field ledger and resolve; MF3 reads the dated events for field history;
MF10 reads the field history for crustal stripes.
