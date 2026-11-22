# Paste support for the Build desk: one-event batches, withdraw and restore

Top-level UI design section 8.2: "Paste places it where the pointer is, at the time on the ruler, as one authored event batch and one undo step". The chart room thread asked for this on 22 Nov 2026, and the kernel thread built it the same day.

## What it does
- **`set_authored_batch {edits: [{address, field, value, impossible?}], occurred_at_s}`** writes every edit as one `authored.set` event and replies as `set_authored` does (`authored`, with event_id, affected and state_hash).
  - The whole batch is refused when any address is outside the universe, any value is malformed (for example NaN) or the batch is empty.
  - A batch cannot mix impossible and possible edits, because the event carries a single evidence kind. The UI sends impossible edits as a batch of their own.
  - Tag edits keep TG1's rules. They are checked against the facts at that time plus the batch's own earlier tags on the same address. Undeclared names join the campaign's tags; `declared_tag` lists them, comma separated.
- **`withdraw {address, at_s}`** (Undo) writes the authored fact `withdrawn: true`. From `at_s` on, the address and everything under it are left out of `scene`, `resolved_system` (through `worlds_at`) and `systems`. Nothing is deleted, and `history` keeps the event. An address with no facts at that time is refused.
- **`restore {address, at_s}`** (Redo) writes `withdrawn: false`, which brings the subject back with every fact it had.
- `crates/sse-kernel/src/withdraw.rs` holds the rule: a subject is withdrawn when it or any address above it carries the mark. The scene never shows the bookkeeping field itself.

## Verified by
- `a_paste_is_one_event_and_undo_and_redo_keep_history` checks:
  - four edits on three addresses make one event;
  - withdrawing a body at t = 5 removes it and its moon from the scene at t = 10 but not at t = 0, and the system count drops;
  - restoring it brings back the same scene;
  - the history holds all three events;
  - a withdrawn system leaves `systems`.
- `a_bad_edit_refuses_the_whole_batch`: an address outside the universe, a mixed batch and an empty batch each leave the state hash unchanged, and withdrawing an address with no facts is refused.

## Goldens
`tests/golden/contract.jsonl` gained four cases on purpose: a batch, a refused batch, a withdraw and a restore. The hello op list and the bad_request variant list now name the three new ops.
