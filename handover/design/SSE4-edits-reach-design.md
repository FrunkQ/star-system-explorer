# SSE4: edits that reach everything (track ER)

Status: **ACCEPTED by Alex, 26 Nov 2026 18:18 UTC** ("yes to all": the six questions of section 13, rulings in section 13A). Design only; nothing here is built yet. ER is a code only, never shown in the product.

Readable copy with a diagram: https://claude.ai/artifact/W7jGCQx6taFgncpojsmt2N

Alex's rule, in his words (26 Nov 2026, 17:54 UTC): "We are still trying to do ALL physics in 'plain sight' in editable physics blocks - and all that colour data is in the reference pack for tweaking? Still following that line?"

Alex again (18:14 UTC): "Remember, we were moving everything over to the reference data and everything into blocks as separate exercises as we came to them. So there's going to be an ongoing effort to remove all references that are not part of the reference pack."

So the counts in section 2 are **the remaining ledger of that migration, not a fault**. Each area converts its own sites when it is next in that code, as the typed-copies ledger was cleared (matter pack design 3A: label, fail only on new ones, change as you touch, clear up at the end). What this note adds is the thing every area converts *to*, and the check that stops the count growing.

The audit of 26 Nov (thread "Check today's physics is in blocks") measured where that migration stands. This note says how an edit to a block, to the data sheet or to a pack's numbers, made in one universe, reaches every result that depends on it, or says plainly where it could not.

Read with: `SSE4-matter-pack-design.md` (sections 3A, 4.6, 7A), `SSE4-equation-block-design.md` (sections 6, 8A, 8B), `SSE4-postcards-design.md`, `SSE4-conserved-chain-design.md`, `SSE4-visual-polish-design.md` (question 1: everything drawn is a fact from the kernel's numbers, or a camera effect), `docs/fudges.md`.

---

## 1. The short answer

1. **The object already exists: it is the universe's stack.** A universe runs on a stack of packs with the campaign's own pack on top. The stack has a hash, the universe's manifest names every pack by content hash, a save carries the campaign pack, and the interface already drafts, previews and adopts a change into that pack. Nothing new needs inventing to hold an edit.
2. **What is missing is the door.** Compiled kernel code mostly does not ask the stack. It reads values that were fixed when the app was built. There are four such side doors (section 2), not two.
3. **The fix is one object and one door.** When a stack loads, the kernel builds one unchanging object from it, "the physics in force": the sheet, the blocks, the flows and each crate's settings, as this universe has them. Compiled code may read physics only through it. The old side doors are closed by a check, as the typed-copies ledger was.
4. **With nothing edited, every result is the same to the last bit.** That is the acceptance test of every package here. Only one package moves any golden line, and those are hash lines.
5. **"Reaches everything" means "follows, or says it did not".** A formula follows an edit exactly. A relation fitted to our universe cannot follow a changed constant: it is rescaled by a cited law and marked ESTIMATED, or it keeps the real result and says so. That three-way rule is already accepted (matter pack design 7A.3). This design makes it true everywhere instead of in 49 of 256 blocks.
6. **The GM sees it.** Any figure that rests on an edit carries the existing "authored" mark, and pressing it names the edit and offers a reset. One list shows every physics change in this universe and what each reaches.
7. **The renderer draws from baked tables.** The kernel works out a table from the blocks in force; the renderer keeps geometry and lookups. An edit then changes the picture, and the picture can carry the same mark.
8. **Size, honestly.** The object, the block door and the table format are small. The sheet's fast table is medium. The remaining ledger (84 files reading the sheet, 45 files reading shipped settings) is large in total but mechanical. It is not a sweep with its own agents: each area's agent converts its own sites when it is next in that code, and a ledger that only shrinks keeps the count from growing. The interface work is medium. Physics that is still plain Rust and not a block at all is outside this track: an edit cannot reach maths that is not a block.

---

## 2. What the code does today (checked against main, 26 Nov 2026)

The audit gave two counts. Both are about right, and there are two more lines for the ledger beside them. These are sites not yet migrated, counted so the ledger starts from a known number.

| # | Side door | Count on main | What it means |
|---|---|---|---|
| A | Compiled code runs a block by id with its shipped settings | **20 call sites** (audit: 18; the other two are in the V3 importer) | `block(id).eval(inputs, &[])` or the block's own defaults. A setting changed in a flow reaches that flow only. |
| A2 | The block library itself is built in | **161 formula blocks in 33 files**, read with `include_str!`; 13 blocks still in Rust | A pack can replace a flow or a box by id. It cannot replace a block's formula or its settings. So the only way to change a block today is inside one flow. |
| A3 | Compiled code runs base flows from a built-in copy of `base-physics` | the place model, the formation heat flow, the light model | An adopted change to `sse.base.planet-heat` does not reach formation. |
| B | Crate files read the sheet's built-in values (`sse_core::sheet`) | **119 files** outside tests (audit: 117) | These are constants fixed at build time. |
| B2 | The handle that does follow the sheet | **12 files** name it; 8 are real readers, 4 are plumbing. It carries **15 of the sheet's 74 constants** (G, c, σ, k_B and the Cosmos group) | A formula block can follow only G, c, σ and k_B. Any other row it names (`sheet/M_sun`) is frozen. |
| C | The kernel reads a shipped pack's settings instead of the stack's (**new finding**) | **96 call sites in 45 files**, 64 of them in `sse-kernel` | Surface, sea, weather, climate, navigation, small bodies, cosmology, light and hazard numbers all come from the built-in base packs. A universe pack that replaces one of those numbers changes nothing. |
| D | The interface and renderer hold their own copies | 37 TypeScript files import the generated sheet copy; the renderer repeats several laws (PRs 613, 616, 617, 623, 630) | Covered by the coordinator's ruling of 26 Nov: the kernel bakes tables. |

Not all of B is a fault. Of the 119 files:

| What they read | Files | Is it a gap? |
|---|---|---|
| Only units and conventions (year, day, AU, parsec, 0 °C, atmosphere, standard gravity) | 35 | No. A unit is a definition. See question 1. |
| Constants of nature and the Cosmos group | 49 | Yes |
| The Sun, Earth, Jupiter and Moon as reference objects | 47 | Partly: fine as a unit ("solar masses"), a gap as a measured fact |
| Material tables (densities, molar masses, latent heats, half-lives) | 18 | Yes |

(A file can be in more than one row.) So the real sheet sweep is about 84 files, not 119.

What already works and is kept:

- The stack, its hash, `replaces` on every override, the load checks, and the campaign pack (`ui-contract/physics-edit.ts`: draft, preview on a scratch kernel, adopt, withdraw).
- `adopt` writes the stack's hash and every pack's content hash into the universe's record, so the manifest already distinguishes an edited universe.
- Two scopes set for the length of each kernel call: the constants handle (`sse_core::constants::with`) and the sheet for the Look up block (`sse_flow::sheet::with`). They are the right idea, made too narrow.
- The reach report (`read_reach`) and the ESTIMATED law on stellar fits.

---

## 3. The physics in force

**One object, built once when a stack loads, never changed afterwards.** In code: `Physics` (working name), held by the kernel in an `Arc` beside the resolved stack.

| Part | Holds | Built from |
|---|---|---|
| Sheet | Every row of the stacked data sheet, as today's `Reference`, plus a **fast table**: one flat array of numbers with a fixed index for every single-number row (section 5) | `provides.reference` of every pack, bottom to top |
| Blocks | The block library as this stack defines it: each formula block parsed and unit-checked into its list of steps, with its settings in force | the base packs' formula files, then any pack's replacements (section 4) |
| Flows | Every active flow, compiled once | the resolved stack |
| Settings | Each crate's settings block, parsed once (`surface`, `weather`, `climate`, `nav`, `small_bodies`, `cosmology`, `light`, `hazard`, `life` and so on) | `provides.other.<key>`; a stack with none gets the shipped base block, exactly as `sheet_of` does today |
| Edits | The list of what differs from the shipped base: row, block or setting, value before, value after, the pack that changed it | worked out at load by comparing with the shipped base |
| Physics hash | One hash over the four parts above | see 3.2 |

### 3.1 What counts as an edit

Three kinds, all written in a pack with the existing `replaces` rule, so nothing real changes silently:

1. **A sheet value.** As today (matter pack design 4.6).
2. **A block.** New: `provides.blocks`, each entry naming a block id and either new settings ("wherever this block runs, heat spreads over half the world") or a whole new formula with its source. Same load checks as a base formula: units, a cited source for every letter, worked examples.
3. **A settings number.** As pack format allows today, and now actually read.

A flow edit stays as it is. Order of winning, most specific first: a setting typed on one node of one flow; the block's settings in force; the block as shipped.

### 3.2 Two hashes, two jobs

- The **stack hash** stays the universe's identity. It covers everything, including names and postcards.
- The **physics hash** covers only what can change a worked-out number. Caches and kept results are keyed and stamped with it. Adding a name scheme then throws away no physics.

One physics hash for the whole universe is the first step. A finer stamp per result (only the rows and blocks it actually read) would save rework after a small edit, but it is a large piece of machinery. Build it only if the rework after an edit proves too slow on a phone.

---

## 4. How compiled code receives it

**One scope at the kernel's edge, one door inside.**

- `Kernel::handle` puts the physics in force for the length of the call, replacing today's two scopes with one. Inside a call it cannot change.
- Compiled code reads through a small set of functions in one module:
  - `physics::run("sse.climate.equilibrium_temperature", &inputs)`: the block in force with its settings in force. The 20 call sites become this one call; most are already a private helper of exactly this shape.
  - `physics::flow("sse.base.planet-heat")`: the compiled flow in force.
  - `physics::sheet()`: the fast table and the full reader.
  - `physics::settings::<SurfaceSettings>()`: a crate's parsed settings.
- New code may take the object as an argument where that is natural. The scope is the rule; passing it is allowed.

Why a scope and not an argument on every function: threading an argument through about 40 crates changes thousands of signatures for no gain, and the two existing scopes have already shown the pattern is safe (restored on a panic, per thread, nothing leaks between calls).

**What a scope costs, and the three rules that pay for it.** A scope is hidden input, so:

1. **Read at the top, not in the loop.** A stage reads what it needs once, into a local value, before any inner loop. This is the speed rule and the clarity rule at once.
2. **Every cache names its physics.** The climate cache, the survey cache, the cosmology tables, every `OnceLock` (21 outside tests) either moves into the physics object or includes the physics hash in its key. A cache that survives an edit is a wrong answer.
3. **The side doors are shut by a check.** The compiled constants become private to the generator's module; `formulas::loaded()` and each crate's `scenario::settings()` are allowed in tests only. `scripts/check.sh` fails on a new use outside tests. Until the sweeps finish this is a ledger that may only shrink, the same way MP22 closed the typed copies.

**Blocks that are still Rust, and Big Blocks.** Their maths is compiled, so only their settings are editable. Their face says so: "Built in. You can change its settings; its working is fixed." Moving more of them into formulas is the standing ND3b work and the "Candidates to move into flows" tables in `docs/fudges.md`. That work makes more things editable; this track makes whatever is a block reach its results.

**Fitted code must not follow blindly.** Where compiled code is a fit made with the real constants (stellar tracks, the galaxy regulator), reading the edited G would be wrong. It reads the real value on purpose through a named call, `physics::real(Row::G)`, which records "not following" for the mark (section 9). So the sweep is not purely mechanical: each read is one of *follows*, *estimated by a law*, or *kept real*.

---

## 5. The sheet's fast table

Today's generator (`scripts/build-rust-reference.py`) writes each single-number row as a Rust constant. The change:

- It also gives every such row a fixed index (`Row::G`, `Row::M_SUN`, `Row::H2O_LATENT_FUSION` …) and writes the shipped values as one array.
- The physics object holds a copy of that array with the stack's values written over it. Reading a row is one array index.
- The constants handle's 15 hand-written fields are replaced by this table: all 74 constants and the scalar material tables follow, and a formula block may name any row.
- Tables that are not one number (vapour curves, refractive index against wavelength, phase diagrams) are read through the existing reader, already per stack.
- Each row carries its class from the sheet's own groups: **convention** (never moves), **nature**, **cosmos**, **reference object**, **material**.

With nothing edited the array holds the same bits the constants held, so results are identical.

---

## 6. Determinism

The rule stays: **same seed, same stack, same deeds, same universe, on every device.**

- An edit lives in a pack, the pack has a content hash, the stack hash names it, and the manifest records it. Two tables with the same edits get the same universe. This is already true of the record; this track makes it true of the results.
- An edited formula runs on the same step-walking evaluator and the same `sse_core::math` as a shipped one, so it gives the same bits everywhere.
- The physics in force cannot change during a call, and no result may depend on the order in which things were looked at. The cache rule in section 4 is what guards that.
- **An edit is timeless.** The universe is worked out again as if it had always been so (question 2). Change over time inside the story is a different thing and already exists: a law with a date on a constant's row (DM6).
- **Kept results.** Some results are worked out once and kept: the gazetteer, the home galaxy, resolved places. Each is stamped with the physics hash it was made under. When the hash changes, anything generated is worked out again; anything a person authored or the players did is left alone (question 3). This follows from the conserved chain's accepted rule that two copies of a universe may differ only through what players did: if kept results did not follow, a copy that looked before an edit would differ from one that looked after.
- **Before adopting**, the existing preview grows from "which flow tests moved" to "what moves": the count of kept places that will be worked out again, and any authored thing that now breaks a wall. Suggest, never refuse.

---

## 7. Golden tests

- **Unedited is identical.** Every package's first check is that `tests/golden/` does not move.
- **One exception, on purpose.** When blocks load from the stack (ER3), the base physics pack's content must include its formulas, so its content hash and the stack and manifest hash lines in the core golden move once. Numbers do not. That is one slot in the core golden lane, in the coordinator's order. As a gain, the frozen fixture then truly freezes the formulas, which today it does not.
- **A new area golden, `edited-physics`.** One fixture campaign pack over the base stack with one edit of each kind: G doubled, one block's setting, one block's formula, one material value, one surface setting. Absolute checks, not ratios alone: Earth's year becomes 1/√2 of a year; a named temperature moves to a stated value; a named world's result that depends on none of them is bit-identical.
- **The reach gate.** A test stack in which every editable number is nudged by one part in a thousand. Every kernel op is run on the proving worlds, unedited and nudged. An op whose answer does not move must be on a short list that says why (it reads no physics, or it is kept real by name). This finds a side door mechanically, including ones nobody has counted. It starts as a report with a ledger that only shrinks and ends as a hard check. As the house rule requires, it is first run against today's code and seen red.

---

## 8. Saves and postcards

- **A save** already carries the campaign pack. It gains the physics hash, so opening it on a newer base can say exactly which of the GM's edits now sit on a changed base value.
- **A postcard** names its packs by id and version, and a campaign pack is on no shelf. So a postcard from an edited universe carries the **edits themselves**: the short list from section 3 (row or block, before, after). A few dozen bytes each. On opening, they load as a pack through the same loader and checks as any other; a formula in them is data walked by the evaluator, never code.
- The back of the card says "Edited physics: 3 changes" and lists them. If an edit cannot be applied (a block the opener's version lacks), Alex's existing rule holds: the card shows its picture and warns that the view cannot be rebuilt.
- Whether a postcard on the shop may carry edits this way is question 6.
- **Base updates.** Every edit records the value it replaced. When a newer base changes that same value, the edit still wins, and the list says the base moved under it.

---

## 9. How the GM sees that a result came from an edit

All marks are on the GM's view only, as the honesty marks already are.

1. **On the figure.** A number, a card line or a chart mark that rests on an edit carries the existing "authored" mark. Pressing it opens the usual small popup: "Rests on your change to G (twice the real value)" with **Reset** and **Show the block**. Where the working could not follow, the popup says which: "Estimated by stellar homology (Adams 2008)" or "Not following your change to G: this relation was fitted to our universe."
2. **One list.** A screen document "Physics changed here" under Workshop, with its own `rail` entry: every edit in this universe, before and after, the pack it came from, what it reaches (follows, estimated, kept real), and reset one or all. It is the existing change-request panel grown to cover sheet values, block edits and settings, not a second panel.
3. **One chip.** The place line shows "Edited physics" while the physics hash differs from the shipped one. Press it for the list.
4. **In the picture.** A True view drawn from a baked table that rests on an edit shows the same mark on its HUD chip.

**How the kernel knows.** The door does it. The physics object knows which rows, blocks and settings are edited. Each read of an edited thing through the door adds it to a small set for the call; the response carries `rests_on` with those edits. With nothing edited this is one untaken branch per read. First per op (the whole answer rests on these edits), then per value inside flows, which already record what each node read.

This also retires the hand-kept reach list. A formula block's reads are known from its own letters. A Rust block's or a stage's reads are observed at the door and checked against what it declares. The matter pack design already asked for "used by" to be read off the wiring; this is the same idea for compiled code.

---

## 10. Speed (phones matter)

Nothing here is measured yet; these are the expected costs and the checks.

| Change | Cost | Note |
|---|---|---|
| A constant read through the door instead of a compile-time constant | about a nanosecond, and the compiler can no longer fold it | Rule 1 of section 4 keeps reads out of inner loops |
| Running a block through the door | none, or a gain | The steps are walked exactly as today; the id is resolved once instead of searching a list of 161 on every call |
| Settings from the physics object | **a gain** | Several of the 96 call sites re-read and re-parse a shipped pack on each call today; they become a pointer read |
| Loading a stack | a little more | The fast table and settings are built once. Formulas are parsed once today as well |
| Marks | one branch per read with nothing edited | |
| After an edit | kept results are worked out again | Edits are made on the GM's machine at the Build desk. A player's phone only receives a stack and loads it once |
| Baked tables for the renderer | small tables made on demand, kept by physics hash | Sizes set by quality tier (section 11) |

The check is the one the equation block design already set: the kernel's bench flows (a system resolve, a galaxy patch, a year of orbits) within 5 % of their time before the change, on the desktop and the phone tier, for each package that moves a read. A package that fails does not merge until its hot reads are lifted out of the loop.

---

## 11. The renderer boundary

Ruled by the coordinator on 26 Nov under Alex's plain-sight rule: **the kernel bakes tables; the renderer keeps only geometry and lookups.**

**What the renderer may do:** move and project geometry, sample and blend tables, composite, and camera effects (exposure, glare, tone). It may place detail from seeds and parameters the kernel gives it.

**What it may not hold:** a constant of nature, a property of a material, or a law. In practice: anything that would need a citation.

**One table format, `sse.table/1`,** so each fix job does not invent its own:

- an id, the axes with their units and spacing (even, or logarithmic), the channels with their units, and the numbers as a typed array;
- the physics hash it was baked under, the blocks it came from, and its `rests_on` edits, so the picture can carry the mark;
- a size picked by quality tier. A phone gets the small size.

**One kernel op, `bake`:** a table id, the subject (a body, a disc, a sea) and a tier in; a table out. Kept by table id, subject and physics hash. A law whose input varies across the picture becomes a table indexed by that input: disc temperature and shift against radius and angle (XO4); reflectance against angle for a liquid's refractive index (the sea glint, EC1); colour against temperature (stars, galaxies); line colours for comet ions.

**Sampling is the one allowed approximation.** Reading between a table's entries on the GPU is a camera-side step, entered once in the fudge register as a class, with a test per table: the sampled value agrees with the kernel's exact answer within a stated bound at probe points.

**The interface's sheet copy** (`ui-contract/src/sheet.ts`, 37 files) follows the same line: units and conventions may stay as a generated copy; a constant of nature or a material value is asked of the session, which gives the value in force.

**The check:** the wider typed-number lint from the audit's fix list (item 8) is the guard for this boundary. It is a CI change and already with the coordinator; this note does not ask for it again.

---

## 12. Build order

Each package is sized for one agent. "Identical" means `tests/golden/` does not move.

| Package | What | Size | Needs | Owner (proposed) | Check |
|---|---|---|---|---|---|
| **ER1** | The physics object and one scope: built at `load_stack` from the resolved stack (sheet, blocks, flows, settings, physics hash); the two old scopes become views of it. No reader changes yet. | small | none | kernel | identical; physics hash equal for two stacks that differ only in names |
| **ER1b** | The reach gate as a report: the nudged stack, every op, a ledger of ops that do not move. Seen red on today's code. Own test line in CI. | small | ER1 | kernel | the ledger lists today's side doors; it may only shrink |
| **ER2** | The block door: `physics::run`; the 20 call sites go through it; `provides.blocks` with settings in force and `replaces`. | small | ER1 | kernel | identical; a block setting changed in a fixture pack moves the ops behind those 20 sites |
| **ER11a** | `sse.table/1` and the `bake` op with its cache and tier sizes, plus the sampling test helper. First table: the black hole disc (XO4). | small to medium | ER1 | kernel, with the exotic objects thread | the GPU picture agrees with the kernel at probe points; editing `sse.light.disc_shift` changes the table |
| **ER4** | The sheet's fast table: generator writes row indexes and classes; all 74 constants and scalar material rows in force; formula blocks may name any row; the 15-field handle retired onto it. | medium | ER1 | data sheet and kernel | identical; G doubled gives the MP1c results as before; a formula naming `sheet/M_sun` follows an edited row |
| **ER5** (standing) | The sheet ledger, about 84 files: **changed as touched.** Any package that edits a file on the ledger moves that file's reads to the door in the same change and deletes its ledger lines. Each read becomes follows, estimated or kept real. No agent is opened only for this until the clear-up. | large in total, a few lines per package | ER4 | every thread, in its own crates | identical; the file's lines leave the ledger; bench within 5 % |
| **ER6** (standing) | The settings ledger, 96 call sites in 45 files (64 in the kernel crate): **changed as touched**, the same way. An area's first package after ER1 also moves that area's settings block into the physics object, so later packages only change call sites. | large in total, a few lines per package | ER1 | every thread, in its own area | identical; a universe pack that replaces one number in that area's block moves its results |
| **ER3** | Blocks from the stack: formula files become pack data; a pack may replace a block's formula; base flows run by compiled code come from the stack. | medium | ER2 | kernel | numbers identical; **core hash lines move once** (one slot in the core golden lane) |
| **ER7** | Caches and kept results: every cache keyed by physics hash; the `OnceLock` audit; kept results stamped and worked out again on a change; the preview's "what moves". | medium | ER1; agreed with the conserved chain build | kernel | edit then un-edit returns the first universe bit for bit; two copies looked at in different orders agree |
| **ER8** | Marks from the door: `rests_on` on responses, then per value in flows; the reach report derived from observed reads. | medium | ER2, ER4 | kernel | a result behind an edit names it; one that is not, does not |
| **ER9** | The interface: the mark on figures, "Physics changed here" with its rail entry, the chip on the place line, sheet values and block settings editable through the same draft and adopt path. | medium | ER8 | planet and star editing, chart room | walk-through step within its press budget; mock-ups to Alex first |
| **ER10** | Saves and postcards carry the physics hash and the edit list; opening applies or warns. | small to medium | ER3, ER8 | postcards | a postcard from an edited universe opens to the same view elsewhere |
| **ER11b…** | Each renderer law onto a baked table: sea glint and refractive index, comet ion colours, galaxy and star colour, the group's edge into a kernel block. | small each | ER11a | the thread that owns each | sampling test per table |
| **ER12** | The interface asks the session for constants in force; the generated copy keeps units only. | small | ER4 | chart room | a changed G shows in every interface figure that uses it |
| **ER13** | Clear up at the end: one agent converts whatever is left on the ledgers, then the gate becomes a hard check and the old doors go private. Opened when the ledgers are short, or when Alex wants the line closed. | small to medium, by what is left | all | kernel | the ledgers are empty; the check fails on a new side door |

**Order.** ER1, then ER1b and ER2 (cheap, and ER2 alone fixes the 18-site finding for settings). ER11a straight away beside them, because renderer fix jobs are already running and need the shared format. Then ER4. From there ER5 and ER6 are standing rules, not packages in the queue: every thread converts what it touches. ER3 in its golden slot, ER7, ER8. Then ER9, ER10, ER12 and the ER11b ports. ER13 last.

**The ledger rule, in full.** ER1b writes two ledgers from today's code: sites reading built-in sheet values, and sites reading shipped settings or running a block outside the door. `scripts/check.sh` fails when a line is added and when a line names a site that is already converted, so a ledger may only shrink. A new file starts at zero. An edit made in a universe reaches an area as soon as that area's lines are gone; until then the "Physics changed here" list says honestly which areas do not yet follow (it reads the same ledger).

**What is cheap:** ER1, ER1b, ER2, ER11a, ER12. They are a few days of agent work between them and already make block settings and renderer tables honest.
**What is large:** the remaining ledger, about 130 files. It is mechanical apart from the follows / estimated / kept-real choice, none of it moves a golden, and it is paid a few lines at a time by whoever is already in the file.
**What this track does not do:** turn compiled physics into blocks. The stellar tracks, the disc, growth and impacts listed as candidates in `docs/fudges.md` stay as they are until that separate work moves them. Until then their settings are editable where they are pack data, and their maths is not.

---

## 13. Questions for Alex

1. **Units never move.** If a setting doubles G or changes the Sun, "one solar mass", "one AU" and "one year" still mean the same number of kilograms, metres and seconds (the IAU's fixed nominal values). The Sun as a catalogued body can change; the unit named after it cannot. **Recommended: yes.** Otherwise every figure shown in solar masses would shift meaning with an edit, and 35 files of plain unit conversion would need to follow a "constant" that is really a definition.
2. **An edit to physics is timeless.** The universe is worked out again as if it had always had that physics. A change that happens at a date inside the story stays what it already is: a law with a date on a constant's row. **Recommended: yes.** It keeps "same seed and same stack give the same universe" true with no hidden history of when the GM pressed the button.
3. **After an edit, everything generated is worked out again automatically,** including things kept from earlier looks (the gazetteer, the home galaxy, resolved places). What a person authored and what the players did is never touched, and a preview says what will move before the edit is adopted. **Recommended: yes.** It follows from your conserved chain rule that two copies of a universe differ only through what players did.
4. **An edit to a block applies everywhere that block runs in this universe,** not only in the flow where you made it. A value typed on one node of one flow still wins there. **Recommended: yes.** It is what "editable" was taken to mean, and it is the audit's first finding.
5. **A pack may replace a block's formula, not only its settings,** under the same rule as every other override: it says what it replaces, every letter has a source line, and results carry the "authored" or "fictional" mark. The 13 blocks still in Rust and the Big Blocks stay settings-only and say so on their face. **Recommended: yes.** It makes "editable blocks" literally true for the 161 formula blocks.
6. **A postcard on the shop may carry physics edits inside it** (a short list of changed numbers and formulas, shown on the back), without those edits first being published as a pack. Your rule that a shop postcard shows only content in the game or on the store stays for worlds, pictures and names. **Recommended: yes.** An edit is a few numbers, not content, and otherwise every small tweak needs a published pack before a card can be shared. Direct sends to a friend carry the edits either way.

### 13A. Alex's answers (26 Nov 2026, 18:18 UTC): "yes to all"

All six are rulings as recommended:

1. Units never move. The Sun as a body can change; the unit named after it cannot.
2. An edit to physics is timeless. Change at a date inside the story stays a law with a date on a constant's row.
3. After an edit everything generated is worked out again automatically, with a preview first. What a person authored and what the players did is never touched.
4. An edit to a block applies everywhere that block runs in this universe. A value typed on one node of one flow still wins there.
5. A pack may replace a block's formula, not only its settings, under the usual override rule. Rust blocks and Big Blocks stay settings-only and say so.
6. A postcard on the shop may carry physics edits inside it, shown on its back.

With his framing of 18:14 UTC (top of this note): the remaining sites are a ledger, changed as each area is come to.

---

## 14. Routine choices, for the coordinator (agreed as proposed, 26 Nov 18:14 UTC)

- Track code ER and the status file `handover/status/edits-reach.md`.
- `sse.table/1` and `bake` as the shared seam for the renderer fix jobs already running (section 11), so they do not each invent a table format.
- "Table sampling" as one class in the fudge register, with a test per table, instead of one row per table.
- The side-door check in `scripts/check.sh` as a ledger that only shrinks. It adds a check and loosens none.
- One slot in the core golden lane for ER3's hash lines.
- ER7 agreed with the conserved chain build before either starts on kept results.
