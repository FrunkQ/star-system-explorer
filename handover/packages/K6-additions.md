# Kernel API additions (kernel track, 18 Nov 2026)

Changes to the K6 message API after M1, made on the kernel track (`claude/project-thread-dzkz4p`) at the UI track's request (U3 and V1). The protocol id stays `sse.kernel/1`: every change is additive. New ops are new variants; the one new response field is optional and left out when unset.

## 1. Stack changes are chronicled (U3 gap 1)
**Problem:** `load_stack` after `open_universe` changed the physics without trace. The run manifest kept the packs from open time and the state hash did not move, so adopting physics could not be replayed.

**Now:**
- A successful `load_stack` while a universe is open appends a `stack.adopt` event at the universe root (`sse_kernel::STACK_ADOPT`):
  - evidence `authored` (the GM's choice), timeless (occurred and activated at the origin)
  - it sets `stack.hash` and `stack.packs` (each pack as `id@version#content_hash`) on the root
  - so the state hash and the authored chronicle hash both move
- The universe keeps its current pack list, and the manifest is rebuilt from it.
- `stack_loaded` carries `adopted: { event_id, state_hash, manifest_id }` when a change was recorded.
- Loading the same stack again records nothing, and a rejected stack changes nothing.
- New op `manifest` returns the open universe's run manifest as it stands (`{ manifest, manifest_id }`).
- Test `stack_changes_are_chronicled_and_replay`:
  - the same session on two fresh kernels gives identical responses
  - adopting physics moves the state hash and the manifest id
  - a rejected stack leaves the manifest unchanged

## 2. Scenes and the Scanned button through the API (V1 request)
- **Pack key:** `provides.other.observe` = `{ disclosure, views, campaign }` (`sse_observe::ObserveSetup`). The highest pack that provides one wins whole, and it is validated at `load_stack` (the campaign's three views must exist). U5 edits it there.
- **`scene { audience, scope, data_view?, at_s }`** returns `{ scene, scene_hash }`:
  - `scene` is the sse-observe `Scene` JSON, unchanged in shape
  - `audience: "gm"` is the GM seat, with honesty marks
  - `data_view` shows every subject through one view, whatever the grants say (a lens or a preview); this is new `sse_observe::build_scene_through`
  - tags are authored `tag.<name> = true` facts, taken out of the facts before building, plus the stack's K8 rule tags (`provides.other.rules`)
- **`scan_toggle { address, audience, at_s }`** returns `{ event_id, view, state_hash }`. It is K9's `scan_toggle`: a chronicled `observe.grant`, a second tap undoes it, GM only.
- **Contract (native and through the worker):** fixture pack `tests/fixtures/packs/tarn-observe.json` holds the Tarn world's setup. The world is entered with `set_authored`. The responses are then checked:
  - the `everyone` scene equals `tests/fixtures/observe/tarn-before.json` exactly
  - after one tap it equals `tarn-after.json` exactly
  - a second tap gives the before scene again
  - the GM scene has the secrets and the player scene does not
  - an unknown view and an address in another universe give readable errors
- Test `scene_tags_come_from_stack_rules`: the K8 fixture rules tag a world `habitable` in a GM scene.
- Not yet: `subscribe` with deltas (`sse_observe::diff` exists; wire it with V4).

## Golden change (on purpose)
`tests/golden/contract.jsonl` was re-blessed. Earlier lines changed only where intended:
- `hello` lists the new ops
- the unknown-op error lists the new variants
- response 41 (`load_stack` while the Tarn universe is open) now carries `adopted`

Responses 44 to 97 are new: manifest, re-adopt, history, a rejected stack, and the Tarn scene cases. The TypeScript contract types are regenerated (`packages/contracts/src/generated`, new `StackAdopted.ts`).

## Shared files touched
- `crates/sse-kernel` (`Cargo.toml` gains `sse-observe` and `sse-sift`; `lib.rs`; `protocol.rs`; `tests.rs`)
- `crates/sse-observe` (`pack.rs`, `build_scene_through`)
- `tests/contract/requests.json`
- `tests/fixtures/packs/tarn-observe.json`
- `tests/golden/contract.jsonl`
- `packages/contracts/src/generated/*`

## Numbers across the JS boundary (found by the worker contract)
Two traps, both now avoided in the fixtures, and both apply to every client:
1. **Integer-valued floats.** JavaScript cannot send `281.0`; `JSON.stringify` writes `281`, which the kernel reads as an integer. A pack or an authored value written as `10.0` in a Rust-side fixture therefore hashes differently from the same file sent by the UI. Fixtures now write such numbers as integers, and the native test compares scenes number-by-value (`as_js` in `crates/sse-kernel/src/tests.rs`). Readers that need a number already accept both (`sse-observe` `as_f64`).
2. **serde_json's default float parse is not correctly rounded** (G0 log has the first sighting). `1.2e27` parses one ulp off, re-serialises as `1.2000000000000001e+27`, and then a native round trip and a JS round trip disagree. The contract fixture now uses `1.9e27`. Every float literal in `tests/contract/requests.json` and `tests/fixtures/packs/` was checked against the exact parse.
**Resolved:** both traps are now fixed at the kernel boundary for every client; see `numbers.md`.

## U3 gaps 2 to 5 and the draft scene preview (after C1)
- **Gap 2, describe atoms**: new op `atoms { ids? }` returns one record per atom in id order. Each record lists the inputs and outputs (name, quantity kind, dimension as text such as `m^2 kg s^-3`, valid range), the parameters (name, dimension, default, allowed range, description), the cost class, the references and the explain text. An unknown id gives `unknown_atom`. Cost class trivial.
- **Gap 3, readable validation text**: a parameter outside its range now reads `lum.exponent: value 99 is outside the allowed range 1 to 6` instead of `... range Some((1.0, 6.0))`. This touches the shared K4 crate (`sse-flow/src/flow.rs`, one message, and its test). Ids in messages stay quoted.
- **Gap 4, explain beyond published quantities**: this was already possible through `node.port` (K4's `explain_port`) for any node, published or not. A contract case now proves it on a node whose output is published by nothing (`toy-quiet` pack, `dens.density`).
- **Gap 5, flow tests at load_stack**: each flow's own golden cases run when a stack loads. A failing case refuses the stack, naming the case, the quantity, the value it got and what it expected. An edited campaign flow must therefore carry cases that hold (its editor re-baselines them on purpose). The shipped packs pass, and no golden moved.
- **Draft preview** (UI track): `scene { ..., observe? }` builds the scene with a draft `provides.other.observe` setup instead of the stack's. It records nothing, so the stack's scene and the state hash are unchanged. A malformed draft gives `bad_observe`.
- Contract cases 98 to 109 pass: natively, through the Node worker, and in Chromium (110 of 110). New fixtures: `tests/fixtures/packs/toy-bad-test.json`, `toy-bad-range.json`, `toy-quiet.json`. The contract golden changed only by these new lines, plus the op list in `hello` and the variant list in the "nonsense" error.

## Look-ahead through the API (T2, 19 Nov, UI track's request for the chart room's ageing screen)
- **`look_ahead { system, target_myr, settings? }`** returns `looked_ahead`:
  - `{ branch, label, system, now_myr, target_myr, birth_year, horizon_myr, events, snapshots, work_units }`
  - `system` is an `sse-time` `SystemSpec`: star mass, z, age now, birth year, planets with J2000-style elements, optional debris belt. `tests/fixtures/time/solar-system.json` is one. Its `system` address must be in the open universe.
  - `target_myr` is the star's age to run to. It must be later than the age now.
  - Settings come from the stack's `provides.other.time` (`packs/base-time`). Passing `settings` overrides them for a preview.
  - `events` are `FateEvent { kind, age_myr, year, subjects, why, level, plausible_future, numbers }`. The kinds in the base pack are `oceans_lost`, `leaves_main_sequence`, `red_giant`, `engulfed`, `planetary_nebula`, `cooling_colour` and `pollution`.
  - `snapshots` are `FateSnapshot { age_myr, year, star { phase, luminosity, radius_rsun, teff, mass, colour, rgb }, planets [{ name, a_au, e, i_deg, engulfed }] }`, one per epoch step.
  - Star phases read `main sequence`, `subgiant`, `red giant`, `white dwarf`.
  - Ages beyond `horizon_myr` are "one plausible future" and should be drawn as such.
  - Years are plain JSON numbers; star age t falls in year `birth_year + t x 1e6`.
- **What the run writes:** T2's `look_ahead` on a chronicle branch: the authored `time.request`, then derived `evolve.transition` and `time.<kind>` events. The main chronicle is untouched. The returned branch works with `branch_state_hash` and `discard` like any branch. `commit` also works, but without moving "now".
- **`commit_look_ahead { branch, move_now_myr? }`** returns `committed { event_id, state_hash }`. It is T2's `commit`; with `move_now_myr`, an authored `system.now_year` fact moves the system's "now" to that star age. A branch that is not an open look-ahead gives `unknown_branch`.
- **Errors:**

  | Code | When |
  |---|---|
  | `no_time_settings` | no pack provides time settings and none were sent |
  | `bad_settings` | the settings sent do not parse |
  | `bad_system` | the system spec does not parse |
  | `bad_address` | the system is not in the universe |
  | `bad_target` | the target is not after the age now |
- **Cost:** class medium; it runs in one call. Ageing the Sun to a cold white dwarf is about 0.2 M work units: about 12 s in a debug native build, well under a second optimised. If a slower device needs progress or cancelling, make it a job (`JobSpec`) next; `sse_time::branch::look_ahead` already reports work units and can cancel.
- **Not yet:** building the `SystemSpec` from the universe's own facts. The UI sends it from what it shows. A later package derives it from resolved systems (F track output).
- **Tests and contract:**
  - Test `look_ahead_through_the_api`: the Sun aged to 13 Gyr; events with reasons; main-sequence to white-dwarf snapshots; years; main chronicle untouched; branch hash differs; discard; commit with now moved; `time.request` in the system's history; the error cases.
  - Contract lines 117 and 118 are new: `no_time_settings` and `unknown_branch` through the worker. The op list in `hello` and the variant list in the "nonsense" error changed with them.
  - New TS types: `FateEvent`, `FateSnapshot`, `FateStar`, `FatePlanet`, exported from `packages/contracts`.
- **Naming for ops other tracks add:**
  - snake_case, verb first, one variant per op, additive under `sse.kernel/1`.
  - Long runs return a branch, or a `job` for progress.
  - Suggested for the physics track: `import_v3 { save }` → `imported { report, ... }` and `run_formation { ... }` → `formation_run { ... }`, each with its own error codes.
- **Shared files touched:**
  - `crates/sse-kernel` (`Cargo.toml` gains `sse-time`; `lib.rs`; `protocol.rs`; `tests.rs`)
  - `packages/contracts/src/index.ts` and `generated/*`
  - `tests/contract/requests.json`
  - `tests/golden/contract.jsonl`
