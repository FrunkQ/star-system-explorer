# Status: edits that reach everything (track ER)

Owner thread: "Design edits that reach everything" (Fable, design only, 26 Nov 2026) for the note. No builder thread yet. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-edits-reach-design.md`, **PROPOSED 26 Nov 2026, not accepted**. Its section 13 holds 6 questions for Alex, each with a recommendation. Readable copy: https://claude.ai/artifact/W7jGCQx6taFgncpojsmt2N

What it is, in one line: an edit to a block, the data sheet or a pack's numbers already has a home (the universe's own pack on top of its stack), but compiled kernel code mostly reads values fixed when the app was built; this track builds one unchanging object from the stack ("the physics in force"), makes it the only door compiled code reads physics through, shuts the side doors with a check, marks every result that rests on an edit, and has the kernel bake tables for the renderer.

Counts on main, 26 Nov 2026, outside tests (the note's section 2 has how each was counted):

- 20 call sites run a block with its shipped settings (the audit said 18; two more are in the V3 importer).
- 161 formula blocks and 13 Rust blocks are built in: a pack can replace a flow, not a block.
- 119 crate files read `sse_core::sheet` (the audit said 117). 35 of them read only units and conventions; the real sweep is about 84.
- The handle carries 15 of the sheet's 74 constants and has 8 real readers.
- 96 call sites in 45 files read a shipped pack's settings instead of the stack's (new finding; 64 are in `sse-kernel`).
- 37 interface files import the generated sheet copy.

Nothing is built. ER is a code only, never shown in the product. Every package leaves `tests/golden/` unchanged except ER3, which moves core hash lines once (one slot in the core golden lane).

| Package | State | Owner (proposed) |
|---|---|---|
| ER0 the note in `handover/design/`, this file, the row in START-HERE | done, 26 Nov | this thread |
| ER1 the physics object and one scope, built at `load_stack`; physics hash | proposed | kernel |
| ER1b the reach gate as a report with a ledger that only shrinks; seen red first | proposed, needs ER1 | kernel |
| ER2 the block door (`physics::run`), the 20 call sites, `provides.blocks` settings in force | proposed, needs ER1 | kernel |
| ER11a `sse.table/1` and the `bake` op; the black hole disc first | proposed, needs ER1; wanted early by the renderer fix jobs | kernel, exotic objects |
| ER4 the sheet's fast table: every single-number row in force, formula blocks may name any row | proposed, needs ER1 | data sheet, kernel |
| ER5a to ER5e the sheet sweep (about 84 files) by crate group; each read is follows, estimated or kept real | proposed, needs ER4 | each owning thread |
| ER6a to ER6c settings from the stack (96 call sites) by crate group | proposed, needs ER1 | each owning thread |
| ER3 blocks load from the stack; a pack may replace a block's formula; base flows from the stack | proposed, needs ER2 and question 5; core golden slot | kernel |
| ER7 caches keyed by physics hash; kept results stamped and worked out again | proposed, needs ER5, ER6 and questions 2, 3; agree with the conserved chain build | kernel |
| ER8 marks from the door: `rests_on`, reach report from observed reads | proposed, needs ER2, ER4 | kernel |
| ER9 the interface: mark, "Physics changed here" with its rail entry, the chip, editing sheet values and block settings | proposed, needs ER8 and question 4; mock-ups to Alex first | planet and star editing, chart room |
| ER10 saves and postcards carry the physics hash and the edit list | proposed, needs ER3, ER8 and question 6 | postcards |
| ER11b onwards each renderer law onto a baked table | proposed, needs ER11a | the thread that owns each |
| ER12 the interface asks the session for constants in force | proposed, needs ER4 | chart room |
| ER13 close: ledger empty, gate a hard check, old doors private | proposed, needs all | kernel |

## Next
- Alex answers section 13 of the note; his answers go in a section 13A there and in `decisions.md`.
- ER1, ER1b, ER2 and ER11a depend on none of the six questions and can open on the coordinator's word.
- The coordinator's routine choices are listed in the note's section 14.
