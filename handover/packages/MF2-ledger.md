# MF2: the field ledger, part 1 (Make test blocks; rocky worlds publish their field)

This follows the magnetic fields design (`handover/design/SSE4-magnetic-fields-design.md`, sections 4 and 4.1; MF2 row, accepted 22 Nov), with the proven core thread as owner. Part 1 gives the ledger's Make test as blocks and runs it on MF1's core history, so every rocky world with an iron core publishes its field facts. Part 2 adds giants and ice giants: their dynamo top is found by walking down the conductivity, once the data sheet thread's sigma_elec rows land (metallic hydrogen profile from French 2012, liquid iron, seawater and ionic water).

## Blocks (`packs/base-physics/formulas/dynamo.json`, sums in `sse_atoms::ledger`)

- **`sse.dynamo.flow_speed`**: u = c Ω D Ra_Q*^b, with Ra_Q* = (α g q / ρ c_p) / (Ω³ D²), c = 0.85 and b = 0.41 (Christensen and Aubert 2006). It also gives the local Rossby number u / (Ω D). The geometric factor (outer over inner radius) is left out, so that a full sphere has an answer.
- **`sse.dynamo.reynolds`**: Rm = u D μ0 σ, and the free decay time τ = D² μ0 σ / π² (Keep). σ is the layer's own conductivity, so no list of field-making materials is needed.
- **`sse.dynamo.onset`**: 0 to 1, rising from Rm 40 to 60 (dynamos need Rm above about 40 to 50).
- **`sse.dynamo.rossby`**: Ro = P u / D, and the factor on the field. That factor is 1 below saturation at 0.13, and (0.13 / Ro)^1.26 above it (Reiners et al. 2022). Every planet is saturated.

All four are held to their sums by `ledger_blocks_match_the_field_sums`. That test also checks Earth's core flow at 0.1 to 3 mm/s, Rm in the hundreds to thousands, a dead core's field gone in tens to hundreds of kyr, and that salty water gives no dynamo. The atom count is now 126.

## Rocky worlds (`sse-surface` `core.rs`)

- **Make test each step.** MF1's core history runs the Make test at every step on the liquid shell, whose depth is the core radius less the inner core. The steps are:
  - the flow from the dynamo's power, the core's gravity, α and c_p;
  - Rm with σ = 1e6 S/m (`CoreSettings.sigma_core`, a stand-in until the sheet's liquid iron row, which Pozzo et al. 2012 put at about 1.5e6);
  - the onset;
  - the spin factor.
- **Field.** The field is Christensen's law times onset times the spin factor.
- **New step fields.** `CoreStep` gains `flow_m_s`, `rm` and `onset`.
- **Dated reasons.** A dynamo that dies with power left but Rm below onset says so, with its flow speed and Rm.
- **Spin.** For a locked world the dynamo turns with the orbit's period; otherwise with the body's own spin, or the climate's 24 h default.
- **Published facts.** `core::field_facts(env, body)` gives:
  - `magnetic.dynamo_field`, `body.surface_magnetic_field` and `body.magnetic_moment`;
  - `interior.dynamo_radius`, `interior.dynamo_shell_thickness` and `interior.dynamo_flow_speed`;
  - `interior.magnetic_reynolds` and `interior.dynamo_onset`.

  The surface dipole uses the existing `surface_dipole` sum with the block's dipole ratio (`magnetic::DIPOLE_RATIO`, F-S29 until MF3's shape; a test holds it to the block's default).
- **Kernel.** `resolved_system` adds these facts for every world with an iron core unless the record already holds them. `read_field` with `system` uses the ledger's moment where the record has none.

## Checks

- `core_tests::the_ledger_makes_earths_field_by_outrunning_the_leak`: Earth's core flows at 0.05 to 5 mm/s with Rm 100 to 20,000 and onset 1, and on every world a field runs exactly when onset is above zero.
- `core_tests::rocky_worlds_publish_their_field_facts`: Earth's moment is 9.0e22 A m² against the measured 7.7e22, with 35 μT at the equator. Mars, Venus and the Moon publish zero.
- MF1's checks still hold (Earth never stops, Mars dies young, Venus is not cooling, the Moon dies early).

## Goldens changed on purpose

- `tests/golden/surface.json`: every core history step gains flow, Rm and onset.
- `tests/golden/contract.jsonl`: envelopes, the new facts in resolved systems, and stack hashes.
- The `base-physics` pack and its fixture gain the four blocks and their kinds.

## Fudge register

- The liquid iron conductivity is a typed stand-in (1e6 S/m) until the sheet row lands. Part 2 points it at Look up.
- The flow scaling drops Christensen and Aubert's geometric factor.
- The 24 h spin is used for a world that publishes none.
