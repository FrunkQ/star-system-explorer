# GS2: golden split part two (frozen physics, reach side file)

Alex's yes, 24 Nov 2026 18:07 UTC. Part one (PR 454) masked the op list (golden line 1) and the
unknown-op message (line 41).

## What moved where

- **The contract's physics is frozen.** `tests/fixtures/packs/base-physics.json` is now a frozen
  copy. It no longer has to equal `packs/base-physics/pack.json`, and nobody re-copies it after a
  physics change. The contract (`tests/contract/requests.json`, `$pack:base-physics`) still loads
  it, from the Rust runner and the JS runner alike. A change to the live physics no longer moves
  `tests/golden/contract.jsonl`.
- **The live pack is tested on its own.** The test `base_physics_fixture_is_the_real_pack`
  (`crates/sse-kernel/src/tests.rs`), which checked that the copy matched the pack, is replaced by
  `the_live_base_physics_pack_loads`. The kernel unit tests that used the fixture now load the live
  pack: `tags_tests.rs`, `dressing_tests.rs`, `names_tests.rs`, `matter_tests.rs`,
  `trust_tests.rs` and `sol_tests.rs` in `sse-kernel`, and `early_tests.rs` in `sse-formation`.
  So the live physics stays under test, in those tests and in the area goldens.
- **The reach report's lists have their own file.** The constants reach line (golden line 318,
  `{"type":"reach",...}`) used to carry its `follows`, `estimated` and `cannot_follow` lists, and
  every new block changed that line. The line now keeps its head, and the lists are written as
  `"lists":"one line per block in tests/golden/contract-reach.txt"` (`contract::mask_ops` in Rust,
  `maskOps` in `packages/kernel-client/src/contract.ts`). The entries go one per line to
  `tests/golden/contract-reach.txt`, as `<constant> <list> <entry json>`
  (`contract::reach_entries`). The native contract test writes it on bless and compares it
  otherwise. A new block is now one new line there, so it merges with other new blocks.

## The rule for golden PRs from now on

A change to `packs/base-physics/pack.json` no longer moves `tests/golden/contract.jsonl`; it moves
only the area goldens that read the live pack. The contract golden still moves for a protocol
change (an op's request or reply, a new contract case) and for code or data compiled into the
kernel, such as the block formulas in `packs/base-physics/formulas/*.json`, which `sse-atoms`
builds in with `include_str!`. Adding a block adds a line to `contract-reach.txt` rather than
changing a contract line. Rebless with
`SSE_BLESS=1 CARGO_INCREMENTAL=0 cargo test -q --release -p sse-kernel contract`. A new physics
case that must be seen by the contract has to be placed in the frozen copy on purpose, with its
reason in that package's log.

## Golden change in this package

`tests/golden/contract.jsonl` line 318 now ends in the `lists` note instead of the three lists, and
`tests/golden/contract-reach.txt` is new (202 lines). No other hash moved.
