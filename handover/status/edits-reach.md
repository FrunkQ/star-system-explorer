# Status: edits that reach everything (track ER)

Owner thread: "Design edits that reach everything" (Fable, design only, 26 Nov 2026) for the note. No builder thread yet. Until one exists, each package below belongs to the thread named in its row, and that thread records it in its own status file too.
Design: `design/SSE4-edits-reach-design.md`, **ACCEPTED by Alex 26 Nov 2026 18:18 UTC** ("yes to all": the six questions of its section 13; rulings in its section 13A and `decisions.md`). Readable copy: https://claude.ai/artifact/W7jGCQx6taFgncpojsmt2N

What it is, in one line: an edit to a block, the data sheet or a pack's numbers already has a home (the universe's own pack on top of its stack), but compiled kernel code mostly reads values fixed when the app was built; this track builds one unchanging object from the stack ("the physics in force"), makes it the only door compiled code reads physics through, shuts the side doors with a check, marks every result that rests on an edit, and has the kernel bake tables for the renderer.

Alex, 26 Nov 18:14 UTC: moving everything to the reference data and into blocks was always "separate exercises as we came to them". So the counts below are the **remaining ledger of that migration, not a fault**: each area's agent converts its own sites when it is next in that code, and a ledger that only shrinks keeps the count from growing.

Routine choices agreed by the coordinator, 26 Nov 18:14 UTC (the note's section 14): the code ER, the shrink-only side-door check in `scripts/check.sh`, one core golden slot for ER3, ER7 agreed with the conserved chain build. The coordinator is opening ER1 and has pointed the renderer fix jobs at the note's section 11.

The ledger on main, 26 Nov 2026, outside tests (the note's section 2 has how each was counted):

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
| ER1 the physics object and one scope, built at `load_stack`; physics hash; the shrink-only side-door ledger (`scripts/audit-side-doors.py`, `packages/ER-side-doors.txt`) | done, 26 Nov, PR 644; log and "how to fix a red ledger check" in `packages/ER1-physics-in-force.md` | kernel (thread "Start the reference pack migration") |
| ER1b the reach gate as a report with a ledger that only shrinks; seen red first | proposed, needs ER1 | kernel |
| ER2 the block door (`physics::run`), the 20 call sites, `provides.blocks` settings in force | proposed, needs ER1 | kernel |
| ER11a `sse.table/1` and the `bake` op; the black hole disc first | done, 26 Nov (thread "Bake kernel tables for the renderer"); log and how to add a table in `packages/ER11a-bake-tables.md` | kernel, exotic objects |
| ER4 the sheet's fast table: every single-number row in force, formula blocks may name any row | proposed, needs ER1 | data sheet, kernel |
| ER5 (standing rule, not a queued package) the sheet ledger, about 84 files: changed as touched; each read is follows, estimated or kept real | starts once ER4 lands | every thread, in its own crates |
| ER6 (standing rule) the settings ledger, 96 call sites: changed as touched; an area's first package after ER1 moves its settings block into the physics object | starts once ER1 lands | every thread, in its own area |
| ER3 blocks load from the stack; a pack may replace a block's formula; base flows from the stack | proposed, needs ER2; core golden slot | kernel |
| ER7 caches keyed by physics hash; kept results stamped and worked out again | proposed, needs ER1; agree with the conserved chain build | kernel |
| ER8 marks from the door: `rests_on`, reach report from observed reads | proposed, needs ER2, ER4 | kernel |
| ER9 the interface: mark, "Physics changed here" with its rail entry, the chip, editing sheet values and block settings | proposed, needs ER8; mock-ups to Alex first | planet and star editing, chart room |
| ER10 saves and postcards carry the physics hash and the edit list | proposed, needs ER3, ER8 | postcards |
| ER11b onwards each renderer law onto a baked table | proposed, needs ER11a | the thread that owns each |
| ER12 the interface asks the session for constants in force | proposed, needs ER4 | chart room |
| ER13 clear up at the end: one agent converts what is left, then the gate is a hard check and the old doors go private | opened when the ledgers are short | kernel |

## Next
- ER1 is done (PR 644). ER1b (the reach gate) follows in the same thread; ER2 goes to a fresh agent; then ER4.
- ER11a is done: `bake` and `sse.table/1` are the seam for every ER11b port (the band table, sea glint, comet ions, star and galaxy colour); `packages/ER11a-bake-tables.md` ends with the four steps for adding a table.
- Every package's questions are answered, so nothing in the table waits on Alex. ER9 still shows him mock-ups first.
- The coordinator's routine choices are listed in the note's section 14.
