# Scratch: the roster prototype (conserved chain, item 5). NOT FOR MERGING.

Branch `claude/project-thread-h51rzm` only. Nothing in `crates/` is touched; both crates here are
their own Cargo workspaces, so the main workspace, `scripts/check.sh` and every golden never see them.

- `roster-proto/`: the prototype. `cargo run --release -- [n_exact]` prints the report;
  `cargo test --release` runs its checks. `CC_BREAK=split` or `CC_BREAK=stateful` breaks it on
  purpose: the closure and order checks then go red.
- `roster-wasm/`: the same code built for `wasm32-unknown-unknown` and timed under Node:
  `cargo build --release --target wasm32-unknown-unknown && node bench.mjs target/wasm32-unknown-unknown/release/roster_wasm.wasm 100000`.

What it does, on system `u:f0/sys:16` of seed `cc-roster-1` (a shattered world's debris belt, four
outer belts, four comet clouds, five solid worlds and a giant):

1. Splits each belt by element group into classes (core, mantle and crust fragments; melted,
   primitive and icy bodies; icy and rocky comets) that add back to the belt exactly.
2. Lays a keyed drain sketch (per class, per size band, per time bin, per cause, per sink) over a
   keyed Feistel shuffle of each band's members, so a member's fate and a world's j-th arrival are
   each a few sums away, in both directions, at any band size (bands of 9e9 comets work).
3. Lists a world's craters from those arrivals and reads each back to a named impactor with its
   origin, the cause that loosened it (background drain or a particular passing star) and its
   travel time.
4. Keeps books per class in three tiers (exact member sums for the largest 1e5, exact counts at
   band-mean mass below, gravel as a sum) and checks closure at every moment.
5. Applies deeds (mining a named body, mining gravel into a pin) and shows the two universes differ
   only by those deeds.

The stand-in numbers (class shares, drain laws, sink shares, travel times, the pass rate and kick)
are marked in `roster-proto/src/system.rs`; they are what the per-split specs should replace.
The run report and the wasm timings are in project files, `sse4/roster-proto/`.
