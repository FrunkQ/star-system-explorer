# SSE4: outside models and the kernel as a module (track OM)

Date 24 Nov 2026. Status: **BANKED, NOT ACCEPTED, 24 Nov 2026: revisit when a real use case appears.**
Alex, 24 Nov 20:40 UTC: "don't bother... let's find a proper use case first - bank the idea - store it away
for the future. It's overcomplex and likely unneeded." So: nothing here is a ruling, the thirteen questions of
section 15 are unanswered, **no package in section 13 is queued and no builder is to start on it.** The note is
kept so the thinking is not redone. OM is a code only, never shown in the product. It was written against
`SSE4-design-handoff.md` (H), `SSE4-build-plan.md` sections 1 and 2, `SSE4-record-logic-programs-design.md`
(track J), `SSE4-equation-block-design.md`, `SSE4-matter-pack-design.md` and `SSE4-new-physics-demo.md`, and
was checked against main at 5bbc39a; check it again before reviving it.

Alex, 24 Nov: "does it make sense to have an API block? Basically, an open data receiver and data sender, so
somebody else can use the kernel ... and perhaps pull their own code into a flow." Then: "it's more that a
scientist can put together a deep data model and get it to do something in SSE ... just local machine is maybe
the best security check." Then: "just means they can use external code in their physics flows. They can
approach us to build a code block of it if needed."

## 1. The short answer

Yes, with one change of shape. It is not a block that calls out. It is **a block that reads recorded answers,
plus an app-side caller that fills the recording from a model on the GM's own machine.** Seen that way, most
of it is already decided: the handoff calls this "Class B" work (H9.2: computed once on the GM's machine,
travels as recorded data, players never recompute it) and "results as data" (H6.2, ruling 3 of H14). This
note turns those two rulings into something a scientist can use in an afternoon.

Four things come out of it.

1. **The Outside model block.** A scientist's own code, in any language, answers questions a flow asks. It
   is the try-it-out route. Its values are marked "outside" and say where they came from.
2. **One recording, three ways to fill it.** A table file, a file round trip (the "ask sheet"), or a live call
   to a small program on the same machine. All three end as the same recorded answers, so a campaign replays
   and shares the same way whichever was used.
3. **The path to a native block.** When a model proves its worth, its recording is already most of the
   submission: the table, the units, the ranges and the test cases. Section 11.
4. **The kernel as a published module**, with its request list as generated documentation and a stated
   meaning of "stable". Section 9. This is separate work from 1 to 3 and can go first or last.

## 2. What exists today (checked on main at 5bbc39a, 24 Nov)

| Thing | State | Where |
|---|---|---|
| The kernel talks only through named requests; `hello` returns the version, the protocol name and every request with its cost | built: protocol `sse.kernel/1`, about 120 requests | `crates/sse-kernel/src/protocol.rs` (`ops()`), `lib.rs` (`Request::Hello`) |
| The same requests run natively, in a Node worker thread and in a browser worker, byte for byte | built, tested by the contract golden | `tests/golden/contract.jsonl`, `packages/kernel-client` |
| TypeScript types generated from the Rust types | built | `packages/contracts` |
| The kernel packaged for other people | **not built**: the packages are marked private and export source, the `sse` command has three subcommands (harness, two V3 imports) | `packages/kernel-client/package.json`, `crates/sse-cli` |
| Determinism classes A and B; Class B runs on the GM machine and others fold the recorded events | built for suites | `crates/sse-suite` (K10), H9.2 |
| Every value carries where it came from (catalogued, resolved, derived, authored, authored-impossible) and a validity (in range, extrapolated, impossible input) | built | `crates/sse-flow/src/flow.rs` |
| Packs carry their own sources, fudge list and a reproduction report the kernel runs on load | built (ND6) | `crates/sse-kernel/src/trust.rs` |
| Equation, Model number and Look up blocks; every letter has a line from a source | built or in hand (ND3, MP) | equation and matter pack designs |
| A general numeric table block (a grid of numbers with a stated interpolation, usable by any pack) | **not found**: the data sheet interpolates its own rows, and the logic "look up in a table" is a name-to-value map | `crates/sse-logic/src/blocks.rs` |
| A content security policy on the site | **none set** | `worker/index.js`, `wrangler.jsonc` |

## 3. The proposal, tested

The coordinator's proposal had three rules: the call happens in the app and never in the kernel; the answer is
recorded as a dated, sourced value so replays and players use the recording; it is marked as from outside and
approved by the GM per address. Sending out was "a flow posting facts to an approved address". I tested each
against the kernel as built. The first and third rules hold. The second needs changing, and so does sending.

| # | Test | Result |
|---|---|---|
| 1 | **Can a flow wait for an answer?** Flows are worked out on demand, inside the kernel, with no clock and no network, on every device. | No. So the block cannot "call". In the kernel it is a read: "is there a recorded answer for exactly these inputs?" If yes, that is its value. If no, its `known` output is false (as Look up already does for a missing property) and the ask is listed. The app sees the list, gets answers and records them; the next read finds them. Two steps, never a wait. |
| 2 | **"Dated value".** A flow's answer depends on its inputs, not on a date. A star that brightens over a time-lapse gives a different input on every frame. | A dated value is the wrong key. Answers are stored **by model, model version and the exact inputs**. The date and who asked are kept as history, not as the key. |
| 3 | **Many asks.** A 1,000-frame time-lapse or a 5 billion year look-ahead would make thousands of asks, and inputs that differ in the last decimal place never match a stored answer. | A model may declare a **grid** for an input (say temperature every 50 K). The kernel then asks only for grid points and interpolates between them with a stated rule. A recording with a grid is simply a table being filled as it is needed. This is what makes a live call and a table pack one mechanism instead of two. |
| 4 | **Generating new places.** Resolve must be order-independent and must work on a player's device for a system the GM has never opened (H6.1). | An outside block in a generating flow would give different universes depending on who looked first. **Refused at load**, in words, unless the model's whole domain is already recorded (a complete table). Live asks are allowed only on flows that describe a thing that already exists. |
| 5 | **Solve loops and orbit windows.** A block inside an iteration is asked an unbounded number of times with inputs that depend on earlier answers. | Not allowed live. A complete table is allowed. The load check says which. |
| 6 | **Long runs** (look-ahead, time-lapse). | Two passes. The run goes through using the fallback where an answer is missing and returns the list of asks. The app gets them answered and runs again. Repeated until no new asks, with a cap on passes. The kept result is always a run that read only recordings, so it replays exactly. |
| 7 | **Players and the state hash.** | Recorded answers are events written by the GM, the single writer, and streamed like any authored event, so they are inside the universe hash players check. A player's device reads them and never calls anything. A missing answer on a player's device shows as "not known". |
| 8 | **Golden tests and offline use.** | Unaffected. No base pack may contain an Outside model block (a check), CI never calls out, and a campaign with recordings works with no connection. |
| 9 | **The model gives a different answer the second time** (it is random, or it was changed without a new version). | The app asks a small share of questions twice and compares. A mismatch stops asks for that model and tells the GM. A model that needs randomness takes a seed as a declared input, wired from a keyed draw, so the kernel still owns the dice. |
| 10 | **Sending out from a flow.** Flows are worked out whenever something reads them, many times or never. | A flow must not send: it would post at random. **Sending is an effect**, fired by a trigger or a Playbook preset on the GM's machine (track J and PB), once per event, and never again on replay. Section 8. |
| 11 | **A pack that names an address.** | A pack from a stranger could then point the GM's browser anywhere. **A pack never holds an address.** The pack holds the model's description and recordings; the connection is a setting on the GM's own machine. This is the main security rule. |

So: the proposal stands with rule 2 rewritten (answers keyed by inputs, optionally on a grid), with live asks
kept out of generating flows and loops, and with sending moved from flows to effects.

## 4. What a scientist does

The first route needs no programming on the SSE side and no connection at all.

1. **Describe the model**: a model card (section 5). SSE has a form for it in Workshop, and Orac can draft it
   from a paper or from the model's own source.
2. **Give SSE answers**, by whichever route suits the model:
   - **A table.** Run the model over a grid at home and drop the file in. Done.
   - **An ask sheet.** Wire the block in, let SSE list the questions this universe actually asks, save that
     list as a file, run it through the model anywhere (a laptop, a cluster, overnight), drop the answers back.
   - **Live.** Start the small companion program beside the model (section 7). SSE asks as you work.
3. **Wire the block** into a flow, in place of or beside the built-in block, on a what-if branch. Every value
   downstream is marked "outside" and its Why names the model, its version, its source and when it was asked.
4. **Look at what changed**: the reach report already built for packs says which worlds change, before and after.
5. **Keep it**: "Freeze as a pack" writes the model card and the recording into a pack. Anyone can load it,
   with no model, no connection and no key, on a phone. That pack can go to Explorers.
6. **Offer it for building in** if it earns it (section 11).

## 5. The model card

A model card is pack data, never code. It is to an outside model what the atom spec is to an atom.

```json
{ "outside_model": "lab.kopparapu.inner_edge",
  "version": "2.1.0",
  "name": "Where a wet world starts to boil dry",
  "inputs":  { "star_temp": { "kind": "star.effective_temperature", "unit": "K", "valid": [2600, 7200], "grid": 100 },
               "world_mass": { "kind": "body.mass", "unit": "earth mass", "valid": [0.1, 5], "grid": "log 8 per decade" },
               "seed":       { "kind": "draw.seed", "optional": true } },
  "outputs": { "flux_limit": { "kind": "climate.runaway_flux", "unit": "W/m2", "spread": "flux_limit_spread" } },
  "between_grid_points": "linear",
  "source":  { "text": "author, year, journal", "doi": "...", "where": "the code release, tag v2.1" },
  "licence": "CC BY 4.0",
  "made_by": "who wrote the model and who wrote this card",
  "repeatable": "always" ,
  "time_per_ask": "medium",
  "without_it": "sse.climate.runaway_limit",
  "examples": [ { "star_temp": 5780, "world_mass": 1, "flux_limit": 1.107, "within": 0.01 } ],
  "explain": { "curious": "...", "science": "..." } }
```

The example is illustrative only: the id, the numbers and the source are placeholders, not a real model's.

- **Inputs and outputs** name a quantity kind and a unit, exactly as an Equation's do. The kernel converts
  units both ways and refuses a wire whose unit does not agree. A model is asked in its own declared units,
  so the scientist never converts by hand.
- **Valid range.** Inside it the value is in range. Outside it the default is not to ask at all and to use
  `without_it`; the card may instead say "ask anyway", and the value is then labelled extrapolated.
- **Grid** (optional, per input): linear step, or so many points per decade. With a grid the kernel asks only
  for grid points. Without one, only the exact input is ever asked.
- **Spread** (optional): an output may name a second output as its uncertainty, shown in Why.
- **Source and licence** are required. A card with no DOI or address loads, but is graded "assumed".
- **Examples** are required: at least one, run whenever a route is connected or a table is loaded, and shown
  as a reproduction report beside the published numbers, the way ND6 does for packs.
- **`without_it`** is required: the built-in block or box that answers when there is no recording. It may be
  "nothing", in which case the value is "not known" and whatever reads it says so.
- **Repeatable**: "always", or "with the seed". A card that says neither is refused.
- No address, port, file path, key or command appears in a card. A card that contains one is refused.

**What the mark says.** A new source label, `outside`, joins the five that exist, and travels downstream the
way "authored" and "fictional" already do. Like the other honesty marks it shows on the GM view only. Why on
any marked value reads: the model's plain name and version, who made it, its source, the inputs it was asked
with, in range or extrapolated, on a grid point or between two, when it was recorded and by which route, and
one fixed sentence: "SSE checked this model's units, range and examples. It has not checked its physics."
Every Outside model in a stack is listed in that pack's fudge list automatically, as fitted and assumed Model
numbers already are, with what it stands in for.

## 6. The recording

- **The key** is a hash of the model id, its version, and the inputs in their declared units, written in the
  kernel's canonical form. Same inputs, same key, on every device.
- **An answer** is the output numbers, exactly as received, plus history: when, the route (table, ask sheet,
  live), and the hash of the examples report at the time. Numbers must be finite. Text is never taken from a
  model.
- **Stored** as events in the chronicle, written only by the GM's machine, in batches so a table of 100,000
  points is one event with one content hash and not 100,000 events.
- **Shared with players** as part of the normal stream. Players read, never ask.
- **When inputs change**, nothing is thrown away: different inputs are a different key, so a new ask. Old
  answers stay true for the inputs they were given.
- **When the model changes**, its version changes, so every key changes. A campaign pins the version it
  uses, as it pins pack versions. Moving to a new version is an explicit act with a reach report ("412 values
  on 38 worlds will be asked again"), on a branch until kept. The old recording is kept with the old version.
- **When the card changes** (a unit, a range, a grid), that is a new version by rule; the load check compares
  the card's hash with the one the recording was made under and refuses a silent change.
- **Freeze as a pack** writes the card and the recording out. A frozen pack whose grid is complete over the
  valid range is a **complete table** and is ordinary deterministic pack data: it may be used anywhere,
  including in generating flows and loops (tests 4 and 5).

This needs one general part SSE does not have yet: a **Table block** for packs, a grid of numbers over one to
four inputs with a stated interpolation, read through `sse_core::math`. It is useful well beyond this track
(the light design's line grids, stellar tracks, anything "heavy physics as tabulated results"). A complete
recording is exactly a Table. One mechanism: the Outside model block is a Table block whose table may have
holes, with a way to fill them.

## 7. The three routes, compared honestly

| | Table file | Ask sheet (file round trip) | Live, same machine |
|---|---|---|---|
| The scientist needs | to run a sweep and save a file | to read one file and write one | to keep a small program running |
| Works in every browser, on a phone, offline | yes | yes | no: desktop, and see the browser notes below |
| Covers | only what was tabulated | exactly what this universe asks | anything, as you explore |
| Long-running models (hours, a cluster) | natural | natural | poor |
| Feels like | loading data | a batch job | the model is "in" SSE |
| Security surface | none beyond reading a file | none beyond reading a file | a local connection, section 12 |
| Can drive generating flows | yes, if complete | no | no |
| Build cost | smallest | small | largest |

The honest reading: **the table and the ask sheet give a scientist almost everything**, with no install, no
connection and no browser risk. The live route adds the feeling of exploring with the model beside you: drag
a planet, see the model's answer arrive. That is worth having, and it is the "wow" for a demo, but it is
the third thing to build, not the first.

**What "local machine only" means in a browser.** The live route is a request from the SSE page to a program
listening on the loopback address of the same computer (`127.0.0.1`), and nowhere else.

- The companion is one small file (Python first; standard library only) that the scientist runs beside their
  model. They write one function, `answer(inputs) -> outputs`, and the card. It speaks a tiny protocol,
  `sse.model/1`: describe (returns the card), ask (a batch of questions), and for slow models a job with
  progress and cancel.
- Chrome and Edge: since about Chrome 142 the browser itself asks the user "allow this site to reach apps on
  this device" before any page may reach loopback (Chrome's "Local Network Access" permission, name
  `loopback-network`). That is a second lock, made by the browser, that SSE cannot bypass. Chrome's own
  documents list gaps I could not close from reading alone (dedicated workers, WebRTC), so the call is made
  from the page, not a worker, and OM6 starts by testing it for real.
- Firefox: I believe it reaches loopback from a secure page today, and may add its own permission prompt.
  Not confirmed from a source; to be tested.
- Safari: WebKit's own bug list still shows "do not treat loopback as mixed content" as open, so a secure
  page probably cannot reach `http://127.0.0.1` at all. Safari users get the table and the ask sheet. To be
  tested, not assumed.
- Not offered: reading a folder on disk continuously (works only in Chromium browsers, and gives little the
  ask sheet does not), and anything that needs an installed browser extension.

**A fourth route, later: the model inside the page.** A model compiled to WebAssembly, or a Python model
under a Python-in-the-browser runtime, could run in a locked worker with no network at all. It needs no
companion and works in Safari. It is the handoff's "plug-in" route (H6.2 route b), ruled "later, only if
someone actually needs it". It stays later: it is a large download, it limits what libraries a model may
use, and it is still Class B. Noted as OM-L1.

**Wider than this machine** (a lab server, a cloud endpoint) is not in this design's first release. If it
comes, it is HTTPS only, one address at a time, approved by the GM with the address shown in full, and asks
then leave the machine, which the approval must say. Noted as OM-L2. The ask sheet already covers "my model
lives on a cluster" with no connection at all.

## 8. Sending out

Different purpose, different part of the engine.

- **For a scientist who wants numbers out**, the right tool is not a sender at all: it is **Sweep and
  export**. Pick a flow, pick inputs and ranges, get a file (CSV and JSON) with every value, its unit, its
  source label and the stack hash that produced it. Deterministic, offline, and citable. This is also how an
  ask sheet is written, so it is one piece of work.
- **For a live table** (a second program that follows the game: a VTT such as Mappadux, stage lights, a lab
  notebook, an overlay) there is the **Send out effect**. It is an effect in the sense of track J: fired by a
  trigger or a Playbook preset, on the GM's machine only, once per event. The GM chooses which facts it
  carries from a picker, sees the exact message before approving, and approves the address. It respects
  "Known to": by default it can carry only what the table already knows. A replay never sends again; the
  record keeps "sent, to where, hash of what". Loopback only at first, as in section 7.
- **For a program that wants the whole universe**, the answer is section 9: run the kernel yourself.

The Run desk's "OBS out early" (ruling of 21 to 24 Nov) and the Switchboard already route screens. Send out
routes facts, not screens, and should be agreed with the Run desk thread so there is one "outputs" place in
the interface and not two.

## 9. The kernel as a published module

**What gets published.**

| Piece | For | Notes |
|---|---|---|
| `@starsystemx/kernel` on npm (name is question 9) | browser and Node | the wasm, the worker entry, the typed client and the generated types, in one package; today's `kernel-client` and `contracts` made public and built to plain JavaScript |
| `@starsystemx/base-packs` on npm | the same | the base packs as data, versioned with the kernel that reads them |
| the `sse` command, as downloadable binaries | servers, scripts, Python, R, Julia | a new subcommand `sse talk`: reads one request per line, writes one response per line. Any language that can start a program and read lines can drive the kernel, with no bindings to maintain |
| the Rust crates | Rust users | from the repository at first; crates.io later if anyone asks, since publishing about 40 crates is a chore |
| the request reference | everyone | generated, never hand-written: one page per request from `ops()` (cost, edge behaviour), the Rust types (shape) and the contract golden (a real example of each, which a test already guarantees exists) |
| the conformance file | anyone who builds it themselves | `tests/golden/contract.jsonl` shipped with the module: run it and you know your build gives the same bytes |

Licence as now: Apache-2.0 with the NOTICE crediting Alex. "No simulation runs on the server" (H3) is a rule
about SSE's own service and does not stop someone else running the kernel where they like.

**What "stable" means.** Three separate promises, so none is over-claimed.

1. **Shapes.** Each request gets a stability word in `ops()`: `stable`, `trial` or `internal`. Within
   `sse.kernel/1`, a stable request never loses a field or changes a field's meaning; it may gain optional
   ones. Breaking a stable request means `sse.kernel/2`, with /1 answered for at least one further release.
   `trial` may change in any release and says so in its documentation. `internal` is not documented.
2. **Numbers.** The same kernel version with the same pack stack gives the same bits on every device,
   forever. Across kernel versions numbers may change, because the physics improves; each change is listed in
   the release notes with its reason, which is what the golden rule ("change only on purpose, reason in the
   package log") already produces. A user who needs fixed numbers pins a version.
3. **Versions.** The package follows ordinary version numbering, stays at 0.x while SSE4 is alpha, and every
   0.x release may break `trial` requests. 1.0 is when the first set of stable requests is declared.

First stable set, proposed: `hello`, `load_stack`, `open_universe`, `resolve`, `derive`, `explain`,
`history`, `state_hash`, `set_authored`, the branch requests and jobs. That is enough to open a universe,
look at a system, ask why, change something and step time. Everything else starts as `trial`. Marking the
set is a small kernel change; the eight requests that have no cost row yet (START-HERE) get theirs in the
same change.

Publishing to npm from CI is a workflow edit, so it needs Alex's own yes, and an npm organisation he owns.

## 10. What Orac may and may not do

Orac's rule does not change: it proposes change requests, the kernel checks them, the GM accepts (H13).

| Orac may | Orac may not |
|---|---|
| draft a model card from a paper, a README or the model's source, as a change request | approve a connection, or see or hold the pairing code |
| wire an Outside model block into a flow, on a branch, as a change request | cause a live ask. Its previews read recordings and `without_it` only; new asks start after the GM accepts |
| write the companion file and the sweep script for the scientist to run themselves | run anything on the machine |
| read recordings and Whys, compare a model with the built-in block, write up where they differ | supply answers. An LLM never computes physics (H13), so Orac is never a route |
| fit an Equation to a recording, with the misfit stated, as the first step of section 11 | send out, or add or change a Send out effect's address |
| draft the "offer for building in" issue (section 11) | publish a pack that contains a recording without the GM's licence confirmation |

Bring-your-own keys are never sent on any outside route. Tars learns the Outside models screen so it can
walk a newcomer through the table route.

## 11. From an outside model to a native block

The outside block is the trial. If a model is good, SSE should not depend on someone's laptop for it. The
ladder, each rung optional:

1. **Outside, live or by ask sheet.** One GM's what-if.
2. **Frozen pack.** A table anyone can load. For many models this is the end: a good table is real physics,
   used the same way as the rest of SSE's tabulated results.
3. **Equation.** If the behaviour fits a published relation, it becomes Equation blocks and Model numbers:
   visible, editable, no code, in a pack. Orac can propose the fit; the recording is its test.
4. **Native block.** Rust, in the project, by the usual road: an issue using `docs/atom-spec-template.md`,
   golden cases agreed first, then code.

**What the project needs from the author** for rungs 3 and 4, and where it comes from:

| Needed | Already in hand from the outside trial |
|---|---|
| the formula, or the table with its interpolation rule, or the algorithm in plain steps | the frozen table; the formula is the author's to give |
| every input and output with its unit | the model card |
| the range where it holds, and what happens outside it | the model card |
| a citation with DOI, and where in the source each relation is | the model card's source, to be checked against the master citation list |
| the right to include it: Apache-2.0 for code, CC BY or equivalent for tables, in writing | the card's licence line starts this; code needs the contributor agreement |
| test cases: the paper's own numbers, at least one real object, and the edges | the examples, plus the recording itself |
| how close a cheaper native version must be (a fidelity contract, H12) | the author states the tolerance; the recording is the control |
| what it replaces and what fudges remain | `without_it`, and the pack's fudge list |
| how heavy it is | measured during the trial |

"Offer for building in" on the Outside models screen gathers these into one bundle and a drafted GitHub
issue. So the trial is not thrown away: it is the evidence packet.

## 12. Security review

The boundary: **the GM's machine, the loopback address, numbers only, the GM's say-so.**

| Threat | Answer |
|---|---|
| A downloaded pack makes the browser call somewhere | A pack cannot hold an address (section 5). Connections are settings of this machine and are not saved in a campaign, a pack or an export. The caller refuses any address that is not literally loopback, and a content security policy on the site (there is none today) limits where the page may connect at all, so a bug in the caller cannot widen it. |
| Another web page talks to the scientist's companion | The companion listens on loopback only, answers only requests whose origin is the SSE site, checks the host name it was called by (against DNS tricks), and needs a pairing code it prints when started, which the GM types into SSE once. The code lives in memory for the session. |
| The companion sends something hostile back | Answers are numbers for the declared outputs and nothing else: checked against the card, finite, size-capped. No text from a model is shown, no markup, no code, no addresses. A wrong shape is dropped and reported, as the suite socket already does for a rogue suite (K10). |
| Asks leak the campaign | An ask carries the declared input numbers and the model id. No names, no addresses of places, no notes, no player data, no keys. The first live ask for a model shows the GM exactly what will be sent. |
| A player's device is made to call or send | The caller and the sender are not in the players' page at all (a build check), and a kernel in a player seat refuses the two recording requests. Opening someone else's campaign shows its outside blocks as "recorded answers only". |
| A recording claims false authority ("from the NASA model") | The mark never says more than is known: "recorded by this campaign's GM from a model described as ...". SSE vouches for units, range and examples, and says so. Signed tables from a known author are a possible later step (OM-L3), not a first-release promise. |
| Someone publishes another person's model output | Freezing for Explorers asks the publisher to confirm the right to share, and carries the card's licence. This is the same care as the white dwarf spectra (new-physics demo, 22 Nov). |
| Flooding and hangs | A cap on asks per run and per minute, a time limit per ask, a visible queue with Stop, and the two-pass rule's cap on passes. |
| The recording is tampered with in transit or in a save | It is content-hashed in the chronicle and inside the state hash players check. |
| Send out leaks secrets | Loopback only; GM machine only; the GM picks the facts and sees the message; default limited to what the table knows; never re-sent on replay. |
| The module on npm is tampered with | Published only from CI on a tagged release, with provenance; the conformance file lets anyone check a build. |

Left open on purpose: SSE cannot stop a GM running a bad model on their own machine, and should not try.
What it guarantees is that the model cannot reach beyond the numbers it returns, and that nobody else's
machine ever runs or calls it.

## 13. Packages (not queued: banked, see the status line at the top)

Order: OM1 to OM4 make the table route whole and are the first useful release. OM5 adds the ask sheet. OM6
and OM7 add live. OM8 and OM9 are independent of all of them. Kernel packages that add requests append
cases to the contract golden and take their turn in the golden lane.

| Pkg | What | Owner | Needs | Done when |
|---|---|---|---|---|
| OM1 | **Table block** for packs: one to four inputs, linear or log grids, stated interpolation through `sse_core::math`, `in range` and `known` outputs, load checks, Why shows the bracketing points | kernel | none | a table of a known function reproduces it within the stated error; the same bits on two devices; holes give `known = false`; fudge entry for interpolation |
| OM2 | **Model card and the Outside model block**: the pack section, load checks (units, range, grid, examples, `without_it`, no address), the `outside` source label and its downstream mark, automatic fudge list rows, refusal in generating flows and loops unless the table is complete, no Outside model in any base pack (check) | kernel | OM1 | a stack with a card and a complete table loads and derives; the mark reaches a downstream value and not a sibling; each refusal has a test with its sentence; existing goldens unchanged |
| OM3 | **The recording**: requests to list asks for a derive or a run, to record a batch of answers, to pin and change a model version with a reach report, to freeze a recording as a pack; player seats refuse the writes; two-pass long runs with a cap | kernel | OM2 | record, replay and state hash agree across GM and a player kernel; a run with missing answers reports them and the second pass has none; version change is a branch until kept |
| OM4 | **Outside models screen** in Workshop (gives itself a `rail` entry in its screen document, no tab): the card form, drop a table, examples report, the block's face and Why, asks waiting, Freeze as a pack. Mockups first, by a design thread, for desk, phone and Night chart | chart room or a new builder | OM2, OM3; mockups | a newcomer loads a table and sees a marked value with its Why, by browser check; figures through `figures.ts` |
| OM5 | **Sweep and export, and the ask sheet**: run a flow over chosen inputs to CSV and JSON with units, labels and stack hash; save pending asks in the same format; read answers back with the examples check | kernel for the sweep request, OM4's owner for the screen | OM3 | a round trip through a ten-line script fills a recording; a sweep of a built-in block matches derive bit for bit |
| OM6 | **Live route**: first a written browser test of loopback from the deployed site on Chrome, Edge, Firefox and Safari (results in the package log, design section 7 corrected to match); then the caller in the GM shell (loopback only, pairing code, origin and host checks, queue, limits, repeat check), the site's content security policy, the build check that the players' page has no caller | chart room | OM3, OM4; the policy header is a deploy setting, not CI | each row of section 12 that names the caller has a test; live asks fill the same recording as OM5 |
| OM7 | **Model kit**: the single-file Python companion (`sse.model/1`: describe, ask, jobs), a worked example model, a page of instructions, and the same example as a table and as an ask sheet script. R and Julia only on request | agents thread or a new builder | OM6 | a scientist's function is live in SSE in under ten minutes from the page, timed once by someone who did not write it |
| OM8 | **Send out effect**: effect kind, fact picker, message preview, approval, "Known to" default, sent record, never on replay; agreed with the Run desk thread as one outputs place | Run desk | track J effects, SV; OM6's caller | a preset sends one message to a local listener; a replay sends none; a secret fact cannot be picked without lifting the default |
| OM9 | **Published module**: stability word per request and the first stable set; the generated request reference; `sse talk`; public npm packages built to JavaScript with the wasm inside; the conformance file shipped; release notes from package logs | kernel, with the handover thread for the docs page | Alex's yes to the publish workflow and an npm organisation | a fresh folder with only `npm install` opens a universe and derives a value in Node and in a browser; the same through `sse talk` from Python; the reference page builds from code and fails CI if stale |
| OM10 | **Orac and Tars**: the tools and limits of section 10, each "may not" with a test | agents | OM2 to OM5 | Orac drafts a card and a wired block as one change request; a preview makes no ask |
| OM11 | **Offer for building in**: the bundle and the drafted issue of section 11 | agents | OM5, OM10 | the bundle for the example model fills every row of the atom spec template or names what is missing |

Later, not scheduled: OM-L1 the model inside the page; OM-L2 addresses beyond this machine; OM-L3 signed
tables; a Python package that wraps `sse talk`.

## 14. Fudges and limits to record when built

- Interpolating between grid points is a stand-in for asking the model at the exact input. Fudge register,
  per Table and per card, with the rule used. Real physics that would replace it: the exact ask, or a native block.
- `without_it` standing in where a model has no answer yet. Shown on the value, never silent.
- A live model's purity is sampled, not proven (test 9).
- Browser support for the live route is whatever OM6 measures, stated on the screen per browser.

## 15. Questions for Alex (unanswered: the idea is banked)

1. **Shape.** The block reads recorded answers and never calls; the app fills the recording. Recommend yes:
   it is the only shape that keeps players, replays and the golden tests exactly as they are.
2. **Order of the three routes.** Table first, ask sheet second, live third. Recommend yes: the first two
   work in every browser with no security surface, and the live route's browser behaviour needs testing.
3. **Local machine only** for the first release of live asks and of Send out, with wider addresses a later
   package behind per-address approval. Recommend yes, as you suggested.
4. **A pack never holds an address**; connections belong to the machine and are not saved in campaigns.
   Recommend yes. The cost is that a GM re-pairs after a restart.
5. **No live asks in flows that generate new places or inside loops**; a complete table is allowed there.
   Recommend yes. Without it two GMs could get different universes from the same seed.
6. **A general Table block** (OM1) as the base, so a recording and a table are one thing. Recommend yes: it
   is "one general mechanism" and other tracks want it.
7. **The mark**: a new "outside" honesty mark, GM view only, with the fixed sentence "SSE checked this
   model's units, range and examples. It has not checked its physics." Recommend yes.
8. **Sending**: Sweep and export for scientists; a Send out effect for live tables, fired by triggers and
   presets and never by flows. Recommend yes to both, with Send out scheduled after the Run desk's own work.
9. **The module's public name**: `@starsystemx/kernel` and `@starsystemx/base-packs` on npm, under an
   organisation you create. Recommend yes; the alternative is an unscoped `sse-kernel`, which reads less
   clearly as yours.
10. **"Stable"** as the three promises of section 9, with the short first stable set listed there and the
    package at 0.x while SSE4 is alpha. Recommend yes.
11. **When.** Recommend: OM9's request reference and stability words now-ish, since they are cheap and help
    every new Claude as much as outsiders; OM1 to OM5 after the landing run-through milestone; live and Send
    out after that. None of it in the way of 4.0. Your call if you would rather have the live demo sooner.
12. **Model kit language**: Python first, others only when asked for. Recommend yes.
13. **The name on screen**: "Outside model" for the block and "Outside models" for the Workshop screen.
    Recommend yes; "API block" says how it works, not what it is for.
