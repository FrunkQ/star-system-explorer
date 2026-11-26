# ER2: the block door

Track ER (edits that reach everything). Design: `handover/design/SSE4-edits-reach-design.md` sections 3.1, 4 and 12. Built 26 Nov 2026. Needs ER1 (PR 644).

## What landed

- **`provides.blocks` in the pack format** (`sse_pack::BlockEdit`): a block id, new settings by name, and what it replaces (`{"pack", "range"}`). Left out of a pack's JSON when empty, so no pack's content hash and no stack hash moves. Example: `tests/fixtures/packs/block-edit-wind-and-heat.json`.
  ```json
  "blocks": [{ "id": "sse.light.stellar_wind", "settings": { "speed_km_s": 880.0 },
               "replaces": { "pack": "sse.base-physics", "range": "^0.1" } }]
  ```
- **The replaces rule for blocks**, the same as for flows: the shipped blocks belong to `sse.base-physics` (their formula files are in it). A pack that edits a block names the pack its current version comes from, at a version range the stack has, and that pack must be lower in the stack. A second pack editing the same block names the first; its settings then layer on the first one's (a setting it does not name keeps the first pack's value). Refused, with the reason, at `load_stack`: an unknown block id, an unknown setting name (the message lists the block's settings), a value that is not a number or is outside the setting's range, a wrong or missing `replaces`, one block edited twice in one pack. A refused stack leaves the kernel's physics as it was.
- **The block library in force** (`sse_physics::Blocks`): every shipped block (formula and Rust) with the settings the stack gives it. Each block's face carries the values in force as its setting defaults, and `eval(inputs, &[])` runs it with the settings in force. `Blocks::edits()` lists each changed setting with the shipped value, the value in force and the pack that set it (for the mark, ER8).
- **The doors** (`sse-physics`):
  - `sse_physics::run(id, inputs)`: one block in force with its settings in force; outputs in port order.
  - `sse_physics::block(id)`: the block in force (its ports and references, and its maths).
  - `sse_physics::registry()`: the blocks in force as a registry, to compile a flow against. A flow so compiled starts from each block's settings in force; a value typed on one node of the flow still wins there (Alex's ruling 4).
  - `sse_physics::memo(key, make)`: a value worked out once per physics in force and kept with it (design section 4, rule 2: every cache names its physics). A new physics starts with nothing kept.
  - `sse_physics::Scope`: puts a new physics in force for the rest of a load.
  - Outside any kernel call, every door gives the shipped blocks with their shipped settings.
- **Kernel.** `load_stack` works out the library in force before compiling flows, compiles flows and runs their test cases against it, builds the physics (`Physics::build_with`), then puts it in force for the rest of the load (codes, adopting a universe) before keeping it. `Kernel.atoms` is the library in force, so the block list the interface reads shows the settings in force.
- **Physics hash.** Covers the block edits (left out of the hash input when there are none, so an unedited stack keeps its physics hash).

## The call sites (ledger kind `block`: 35 lines in 23 files, now 1 in 1)

Converted to the door: `sse-codes` plan, `sse-field` glow, `sse-formation` pipeline (the planet-heat flow, now kept with `memo`), `sse-hazard`, `sse-import-v3` (3), `sse-kernel` adopt, atmos, composition, fields, nav, sea, stability, the kernel's own registry, `sse-light` colour, emit (the light model, now kept with `memo`; `Model::base()` returns an `Arc`), shadow, `sse-nebula` forbidden (2), outflow, `sse-place` model, `sse-sense` fog, reach, `sse-world`.

Left on the ledger, on purpose: `crates/sse-flow/src/flow.rs` line 313, the flow compiler taking each block's setting defaults from the registry it is handed. Every registry handed to it outside tests is now the library in force, so it reads the settings in force; the scan matches it only by its text. It goes when the audit learns to tell that line from a side door (ER1b's owner, or ER13).

Not in ER2: a block's formula from the stack (ER3), base flows from the stack rather than the shipped copy (ER3; formation's planet-heat flow and the light model still read the shipped flow, now compiled against the blocks in force), the light model's base-light pack read (ER6), the sheet (ER4, ER5).

## Checks

- `tests/golden/` unchanged.
- `sse-physics` `block_tests`: an edited setting reaches `run`, the registry's face and the edits list; a node's own value still wins; every unedited block gives the same bits through the door as shipped; the door follows the physics in force; a block edit moves the physics hash; `memo` lives and dies with its physics; a pack without edits writes no `blocks`; every refusal above, with its words.
- `sse-kernel` `physics_tests`: the fixture pack doubles the stellar wind's speed and the field read of an adopted system moves with it: every field-held bubble's nose shrinks by exactly 2^(-1/3) (absolute, to 1e-12); the adopted system, which stands on no edited block, is bit-identical. A bad block edit refuses the stack with the reason and leaves the physics as it was.
- `sse-formation` `pipeline_tests`: under the fixture, every solid world's radiogenic power rises (the potassium setting of the radiogenic heating block); outside it, the shipped answer is unchanged. This also guards the cache: with the old process-wide cache the edited run gave the shipped answer.
- Both behaviour tests were run with the fix removed (the wind read from the shipped block; the heat flow compiled against `sse_atoms::registry()`) and went red.
- `scripts/check.sh` green.
