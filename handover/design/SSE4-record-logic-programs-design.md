# SSE4 design: the campaign record, logic blocks, boxes and programs (track J)

Date 19 Nov 2026. Status: **[decided 19 Nov: Alex accepted the design and all five recommendations in section 11, with the classifier ruling in 3.5]**. Audience: the coordinator and the build sessions. This file is the authority for track J. It adds to `SSE4-design-handoff.md` (H) and changes nothing in it except where section 9 below says so.

## 1. Why this exists

Three wishes turned out to be one mechanism.

1. **A persistent player layer.** An RPG campaign needs to remember what happened that physics does not care about: "the players peed on an alien statue here", who is at war with whom, what the party owes the guild. Alex: a general-purpose storage area on every object, plus a way to track political moves.
2. **Logic blocks.** Flows today only do physics on numbers. Classification already is a large piece of logic; "autopilot" (scan, decide, go to the next target, empty the cargo) is another; a game's rules are a third.
3. **Blocks made of blocks.** Multi-use above all: one small part, used everywhere.

A game engine (the 4X assessment that started this) needs exactly these three and nothing else of substance. So SSE4 does not get a game engine. It gets four general parts, and a game is a pack that uses them:

| Part | One line | Section |
|---|---|---|
| The record | notes and kept values on any address, and actors that are not places | 2 |
| Logic blocks | typed values and a small fixed set of logic atoms; one condition language everywhere | 3 |
| Boxes | a flow used as a block inside another flow | 4 |
| Programs | triggers, effects and small state machines that act by sending intents | 5, 6 |

Design rule for the whole track: **nothing here is a second path.** Records are chronicle events. Logic blocks are atoms. Boxes are flows. A program is a seat whose intents come from a flow instead of a person. Every existing guarantee (determinism, explain, data views, GM approval, state hash) therefore applies without new machinery.

## 2. The record

### 2.1 Three things you can keep

**Notes.** A note is an authored event at an address and a universe time: free text, optional tags, optional links to other addresses, who wrote it (seat), and who may see it. It is the "misc data" store. It needs no schema and no pack.

```
kind: "record.note"
subject: [address, ...linked addresses]
payload.data: { text, tags: TagId[], by: SeatId, audience: AudienceId[] | "everyone" | "gm",
                tier?: Tier, supersedes?: EventId }
evidence: "authored" (or "narrative" when Minerva wrote it)
```

Notes are never edited in place: a correction is a new note that `supersedes` the old one; a withdrawn note is superseded by an empty one. History is kept.

**Kept values.** A kept value is a quantity that is *stored*, not derived: a stockpile, a hit-point total, a reputation score, a status, an owner. A pack declares it as a quantity kind with `storage: "kept"`:

```
KeptKind { id,                       "trade.credits", "politics.standing", "x4.metal"
           shape: "number" | "flag" | "choice" | "ref" | "text" | "list",
           unit?, range?, default,
           tier,                     disclosure tier, as for any quantity kind (H6.7)
           writers: [EffectId | IntentKind | "gm"],   who may change it
           describes: one line }
```

A kept value changes only by a `record.set` event, which uses the existing payload `set` so the existing fold reads it with no change. The event's `data` carries the reason (the delta, the effect or intent that caused it), so "why is this 14?" is answered by walking the events, the same as any other "why".

There is also a schema-free corner for the GM and Orac: any address can hold fields named `misc.<anything>`, shape free, tier `secret` unless stated. This is the general-purpose storage Alex asked for. A pack can later adopt a `misc` field by declaring a kept kind over it.

**Actors.** Politics needs subjects that are not places: an empire, a guild, a noble, the player party, a ship's crew. They get addresses in the authored namespace: `u:<id>/actor:<id>`, with children (`/actor:vesh/actor:third-fleet`). An actor is an ordinary addressable thing: it has notes, kept values, tags, names by date and culture, and can be an audience's tag (H8.7), which is how "what the Vesh know" already works.

**Relations** are kept values of shape `ref` or events with two subjects: `owner`, `allegiance`, `at-war-with`, `treaty`, `owes`. A political move is a `record.note` or `record.set` whose subjects are the actors and places involved, so it shows on every one of their timelines.

### 2.2 Rules that keep the record honest

1. **The record never feeds physics by accident.** No physics flow may read a kept value or a note. The only bridge is an explicit effect (5.2) that writes an authored pin on a physical quantity, which is then tagged `authored` or `authored-impossible` like any GM edit. A superweapon that turns a region barren is such an effect; a credit balance can never nudge a climate.
2. **The record never feeds RESOLVE.** Keyed draws for generating the universe take no input from it. What exists does not depend on what players did, unless a GM-visible authored event says so.
3. **One writer rule**, the kept-value form of one publisher per quantity (H5.5): the active pack stack is rejected if two effects claim sole ownership of the same kept kind, and a write from anything not in `writers` is refused.
4. **Removing a pack loses nothing.** Its kept values stay in the chronicle and show as "kept by a pack that is not loaded", readable and exportable.
5. **Everything is dated in universe time.** The record scrubs with the time ruler, branches with look-ahead, and what-if comparisons include it.

### 2.3 Who sees what

Notes and kept values pass through data views exactly as facts do (H6.7). A kept kind has a tier; a note has an audience list or a tier. Scene building drops what the audience may not see. For sealed seats nothing changes: they only ever receive finished scenes. For ordinary seats, a note or kept value whose audience excludes them is **not sent**, the same rule as `secret` authored facts today.

Players can write notes: `add_note` is an intent. The default seat may send it, it is auto-approved for notes the seat addresses to its own audience (a party journal), and the GM sees every note. Anything a player wants on the shared record ("we told the governor we'd be back") goes to the GM for approval like any intent.

### 2.4 What reads the record

- **Cards and the time ruler** show notes and kept values for the selected address; a "Record" lens lists recent changes.
- **Sifters** can match on it (three insults to one actor within a year; a debt past its date), so the story finder surfaces political stories with evidence.
- **Minerva** reads it within the audience's data view. This is her main raw material for "previously, in this campaign".
- **Logic blocks and programs** read it (sections 3 and 5).
- A **relations lens** draws actors and relations over the chart or hex map (allegiance colouring already assumes this).

## 3. Logic blocks

### 3.1 Typed values

H5.2 already says wires carry more than scalars (enumerations, references to addresses). The built flow engine carries only numbers. Track J adds a small closed set of value shapes to wires and ports:

`number` (a quantity, as now) · `flag` · `choice` (a member of a declared enumeration) · `ref` (an address) · `text` · `list<shape>` (finite, ordered) · `record` (named fields, for a thing's facts)

A port with no declared shape is `number`, so every existing atom and flow is unchanged. Connecting mismatched shapes is a validation error at edit time, like mismatched dimensions.

### 3.2 The logic atom set

Small, fixed, shipped in the core, each with explain text in the three lenses. They are marked `class: "logic"`; the rule that every physics atom needs a citation does not apply to them, and the flow editor draws them differently so nobody mistakes logic for physics.

| Group | Atoms |
|---|---|
| Compare | less, at most, equal, near, in range (all with optional hysteresis margin) |
| Combine | and, or, not, count true |
| Choose | select (if, then, else), switch on a choice, table lookup (data table in the pack), first that matches |
| Numbers | add, subtract, multiply, divide, min, max, clamp, round, floor, ceiling (using the pinned maths) |
| Lists | count, sum, any, all, filter by condition, sort by key (ties broken by address), take first N, map through a box |
| World | read a fact, read a kept value, has tag, children of, things within range of, distance between, known to audience |
| Chance | roll: N dice of S sides, or a draw from a table, as a keyed draw (3.4) |

No loops and no recursion. List atoms work over finite lists with a declared maximum length that sets the cost class; a list longer than the maximum is a reported edge, never a silent truncation.

### 3.3 One condition language

The classifier's predicate (all, any, not, has, range, is, compare, near, tag, already built in K8) and the compare and combine atoms above are the same thing in two notations. They become one: a **condition**. It is used, with one evaluator and one explanation format, by:

- classification rules and sifters (as now),
- triggers and guards in programs (section 5),
- `filter` and `first that matches` list atoms,
- data-view rules ("show what carries my audience's discovery tag").

The flow editor can show any condition as blocks and edit it there; the compact document form stays, because Orac writes it well and tables of rules read better as tables. A class region, a rule and a program guard are visibly the same kind of object, so one learnt once is known everywhere. The class atlas itself is not rebuilt as a flow; it is an engine block (3.5).

### 3.4 Chance stays deterministic

A `roll` atom is a keyed draw: `draw(seed, subject address, "<program or flow id>.<node id>.v1", counter)` where the counter is the index of the tick or turn (5.4), never a running generator. Same campaign, same orders, same dice, on every device and in every look-ahead. A GM who wants real dice at the table uses "ask the GM" approval (5.3) and types the result in, which is then an authored value with the roll's place in the trace.

### 3.5 Engine blocks: the classifier and other heavy, unique blocks **[decided 19 Nov, Alex]**

Alex's ruling: the classifier is a "unique block". It can be used *as* a block, but it is a heavy one, like a simulator, and the difference is that it has its own editor, not the block editor.

That makes a third kind of block beside atoms and boxes:

| Kind | Inside | Edited in |
|---|---|---|
| Atom | compiled Rust operator | not editable; specified by a GitHub issue |
| Box | a flow of blocks | the flow editor |
| Engine block | a dedicated engine driven by its own document | its own editor, or the raw document through Orac or an IDE |

- An engine block has the same declaration as an atom (ports, params, validity, cost class, references, explain), so it wires into any flow, box or program decide step. Opening it does not expand into blocks; it opens its own editor.
- **The classifier is the first**: `classify` takes a subject's facts and a scheme and returns the class, its tags and how deep in the region the subject sits; its editor is the Class atlas screen. `place` (reverse placement, "put one of those here") is its second port set. An autopilot asking "is that a freighter or a warship?" or a rule asking "is this world a garden?" uses the same block the charts use.
- Others that already exist as engines get the same wrapper as they are needed, each with its existing editor or none yet: NAV route planning (`plan_route`), feasibility queries (`find`), sifters (`sift`), formation, weather, and any swapped-in simulator (H6.2 heavy blocks). Nothing about those engines changes; they gain a block face.
- Cost and edges apply as for any operation: an engine block above the device's tier is asked of the GM's machine or shown as an edge (build plan section 2). A program's budget counts it.
- Explain passes through: the block's explanation is the engine's own (a class label still answers "why" with its axes and region).

## 4. Boxes: blocks made of blocks

H6.2 names boxes; the built engine does not have them yet. Specification:

```
BoxDoc = FlowDoc with { ports_in, ports_out, promoted_params, explain, tests }, published under id@version in a pack
NodeDoc gains  box?: "id@range"
```

- **A box looks exactly like an atom from outside**: same declaration shape (ports, params, validity, cost class, references, explain). Anything that takes an atom takes a box. That is the whole multi-use rule.
- Boxes nest. The box graph must be acyclic (no box contains itself at any depth); maximum depth 8.
- **Compiled by inlining.** A flow using boxes evaluates as the flat flow it stands for, so results and explain traces are identical to hand-wiring, and nothing about evaluation order changes. Explain shows the box collapsed with a way in.
- Validity is the intersection of what is inside; cost class is the maximum; references are the union; a box flagged `fictional` marks everything downstream, as packs do.
- A box has its own golden cases, so it is tested once and trusted wherever it is used.
- A pack replaces a box with `replaces: id@range`, the same explicit override rule as everything else (H7). This is how "swap the simple climate for the detailed one" was always meant to work; now it has a document.
- Orac writes boxes. "Make me a block that decides whether a world is worth mining" is a box request.

## 5. Triggers and effects

### 5.1 Triggers

A trigger says *when*. Four kinds, all evaluated at time boundaries on the GM's machine, never by wall clock:

| Kind | Fires when |
|---|---|
| `on_event` | an event of a kind arrives for a matching subject (a ship arrives, a note is added, a scan is granted) |
| `on_condition` | a condition (3.3) becomes true for a subject, with hysteresis so it fires once, not every tick |
| `on_time` | a universe time or interval is reached (every week, at turn end) |
| `on_intent` | an intent of a kind is received, before approval (used for validity checks and for held orders, section 6) |

### 5.2 Effects

A flow normally *publishes values*. An effect flow instead *proposes changes*. The closed list of effect kinds:

`set_kept` · `add_note` · `grant` (data view to audience) · `tag` / `untag` · `send_intent` · `create` / `retire` an authored object from a template · `set_pin` (the one bridge to physics, rule 2.2.1)

Effects never touch state directly. Each proposed effect goes through **one gate** on the GM's machine and becomes an ordinary event whose `causes` name the trigger event and whose `publisher` names the pack, program and version. So every automatic change answers "why did that happen?" and can be undone on a branch.

### 5.3 Approval

Every trigger-and-effect pair carries an approval policy, the same idea as a seat's `auto_approve` (H8.7):

- `auto`: applied at once (bookkeeping, a sensor ladder step).
- `ask`: queued for the GM with its explanation; applied on yes. This is "umpire assist".
- `ask_if`: auto unless a condition holds (ask before anything is destroyed).

The GM can change the policy per rule without editing the pack. Default for anything a pack ships: `auto` for `set_kept`, `add_note`, `grant` and `tag`; `ask` for `create`, `retire` and `set_pin`.

### 5.4 Order and determinism

At a time boundary, fired triggers run in a fixed order: phase (declared by the pack), then trigger id, then subject address. Effects from one pass are applied together and may fire further triggers in a next pass, up to a declared pass limit (default 8); hitting the limit is reported with the chain, never silently cut. Everything here is Class A (H9.2) so players' kernels reach the same record from the same events; an `ask` answer travels as the GM's event.

### 5.5 First uses (none of them a game)

- **Traveller sensor ladder** (H7A part 6): `on_condition` "a ship with the right sensor has been in range long enough" then `grant`. This rule was already in the design with no machinery under it; it becomes the first fixture.
- **Consequences**: `on_event` note tagged `insult` on an actor then `set_kept` standing down by one and `add_note` "the Vesh remember".
- **Upkeep and decay**: `on_time` monthly then `set_kept` fuel, supplies, debts.
- **Life and weather hooks** can later raise notes ("first frost on the colony") through the same gate.

## 6. Programs: autopilot and other small minds

A program is a small state machine that belongs to an actor (a ship, a drone, a faction, a non-player empire).

```
ProgramDoc { id, version, owner_kinds: [template or tag],
             params: [...],                       choices and thresholds, promoted to plain controls
             states: [{ id, on_enter?: Effect[],
                        steps: [{ sense, decide: Box|Flow, act: IntentTemplate }],
                        transitions: [{ when: Condition, to: StateId }] }],
             budget: { max_steps_per_tick, cost_class } }
```

Each step is **sense, decide, act**:

- **Sense** reads the world *through the owner's own audience and sensors*. A drone knows what its sensor bands and range have revealed (the light and sensing track O and the data-view ladder), not the GM's truth. An autopilot can be wrong for the same reason a player can, which is the interesting part.
- **Decide** is a box of logic blocks: score the candidates, pick one, choose a mode. Parameters and choices are the "decision tree" Alex described; because decide is a box, a smarter one can be swapped in.
- **Act** sends an **intent**: the same `plan_route`, `scan`, `transfer_cargo`, `add_note` a player seat sends, with the same validation. A program cannot do anything a seat could not.

So **a program is a seat driven by a flow.** It has an audience, a `may_send` list and `auto_approve` scopes, set by the GM when the program is attached to something. An autopilot for the players' own mining drone is auto-approved for that drone and nothing else.

A program runs at ticks: when its current action finishes, when a trigger it names fires, or at a turn boundary. Never continuously, never by wall clock. Because it is deterministic, **look-ahead runs it forward**: "what will the drone fleet have mined in three years", "where will the patrol be when we arrive" are ordinary branches. If the players then do something different, the branch is discarded as usual.

Worked example, the mining drone: state *survey* (scan nearest unscanned body in range; when a body with ore above the threshold is known, go to *travel*); *travel* (ask NAV for the cheapest course; if propellant reserve is below the limit, go to *return*); *mine* (fill cargo; when full, go to *return*); *return* (course to the depot, transfer cargo, go to *survey*). Four states, each decide step a box Orac can write from a sentence.

### 6.1 Worked example: a pirate, a patrol and a simple fight

Three small pieces, all pack data, none of them special code. Together they are what makes a system feel inhabited while the players are elsewhere.

**Who exists.** Programs run only for actors that exist as authored objects. A pirate band is made from a template (8) by the GM, by Orac on request ("put two pirate crews on the Vesh trade route"), or by a pack's seeding effect in a region the GM has marked live. The universe at large is not populated with running programs: that would break the rule that what exists does not depend on what has been looked at, and the cost would be unbounded. A campaign has a declared limit on running programs with its cost class shown.

**The pirate program** (owner: a ship made from the `raider` template; kept values `cargo`, `hull`, `notoriety`, `fuel`):

| State | Sense | Decide | Act | Leaves when |
|---|---|---|---|---|
| lurk | sifter hits for choke points and refuelling stops in the system | box: pick the spot with most traffic and best cover | `plan_route` there, "stay hidden behind the planet" | a contact appears on its sensors |
| size up | contacts in its sensor bands; `classify` each against the pack's ship scheme | box: prize value against risk, with the program's `boldness` param | none | prize chosen (to intercept) or a warship seen (to flee) |
| intercept | the prize's known track | NAV candidates | `plan_route` to intercept; `hail` intent with a demand | in range |
| take | outcome of the engagement rule below | box: loot or leave | `transfer_cargo`; `add_note` | cargo full or hull low |
| flee and sell | known bases | cheapest safe course | `plan_route` to its base; `transfer_cargo` | sold; back to lurk |

The pirate can be fooled: it sees only what its sensors reveal, so a ship running cold behind a moon is missed, and a decoy classified as a freighter gets attacked.

**The patrol program** is the same shape with different boxes: fly a circuit between the system's ports, classify contacts, intercept anything carrying the `raider` tag or a high `notoriety` that its audience knows about, escort freighters when a route's `safety` kept value falls.

**The engagement rule** (trigger and effect, 5): `on_condition` "two ships whose owners are hostile are within weapon range" runs a box: each side's attack is its kept `attack` plus a `roll`; compare with the other's `defence`; `set_kept` hull down by the damage; if a hull reaches zero, `retire` the ship and `create` a wreck with its cargo. Approval is `ask_if` a player seat's ship is involved or anything would be destroyed, so background skirmishes resolve themselves and anything the table cares about stops for the GM. A pack with richer rules (the 4X's ratings, Traveller's ship combat) replaces the box; the trigger, gate and record stay the same. This is deliberately simple-rule resolution of an encounter, not hand-flown dogfighting, which stays out of scope.

**What the universe then shows.** Each loss is an event and a note: `add_note` "the freighter Calla was taken off Tarn's second moon", visible to the audiences who would know (the route's owner at once, others when news reaches them by `activated_at`). A consequence rule lowers the route's `safety` and raises the band's `notoriety`. A sifter surfaces "piracy rising at the Tarn refuelling stop" in the story finder with the events as evidence, and Minerva can write the rumour. On look-ahead the GM sees where the pirates and patrols will be when the players arrive, and the players' own trip planner shows the hazard only if their audience has heard of it.

Agents and programs: Orac *writes* programs and boxes as change requests. No LLM runs inside one. A program is fixed logic and enters the state hash; an LLM never does (H13).

## 7. Turns and held orders (optional, off by default)

For games, and for RPG downtime ("what does everyone do this week?"):

- A pack may declare a **turn**: a named step of universe time and a list of phases.
- In a turn-based session an intent can be **held**: it is validated on arrival (`on_intent`), stored sealed (other audiences do not see it; the GM does), and applied when the GM presses "resolve turn".
- Resolution runs the phases in order under 5.4. The result does not depend on the order in which orders arrived.
- Each seat then gets a turn report: the events its audience can see, which Minerva can word.

Nothing in the default session changes. A campaign with no turn declared never sees any of this.

## 8. Templates

H7A part 7 has stock ships and bases as templates for Traveller. Generalised: a pack declares **templates** (ship, unit, structure, module, station, faction) as a set of kept kinds with defaults, the tags it carries and the programs it may run. `create` makes an authored object from one, in the authored namespace. Traveller's constructs become the first use.

## 9. What changes in what is already built

Checked against `main` at 19 Nov (after PR 6). Everything is additive, but four points need care.

| Area | Built today | Change |
|---|---|---|
| Chronicle (`sse-chronicle`) | event kinds are strings; payload has `set`, `unset`, free `data`; `authored.event` exists | **No code change to the fold.** New kind names `record.note`, `record.set`, `turn.*`. Notes ride in `data`; kept values ride in `set`. |
| Flow engine (`sse-flow`) | nodes are atom or solve; all values are `f64`; no boxes | Add `box` to nodes, shapes to ports, a typed value beside the numeric fast path. **Care 1:** a flow document's content hash is taken over the whole parsed document, so new optional fields must be left out when absent or every existing flow hash and golden hash will change. Acceptance check: all existing hashes identical. |
| Rules and sifters (`sse-sift`) | predicates over one subject's facts and its parent | The predicate becomes the shared condition language. Additive: read kept values and note counts. No rewrite of classification. |
| Data views (`sse-observe`) | grants by audience and tag; scenes filter facts | **Care 2:** scenes must also filter notes and kept values by audience, and the peer-to-peer sender must not send notes an ordinary seat may not see. |
| Intents (`packages/ui-contract`) | a closed TypeScript list of five kinds, checked in the UI | **Care 3:** programs run in the kernel and send intents, and packs add intent kinds, so the intent list has to become a registry the kernel knows (declared in Rust or in packs, exported to TypeScript), not a UI-only union. This is the largest change to built code. |
| Seats (`packages/ui-contract`, shell) | seats for people | A seat may be driven by a program. Additive field. |
| Kernel protocol | requests as listed in H9.1 plus additions | New requests: `add_note`, `set_kept`, `record(address, range)`, `attach_program`, `resolve_turn`. Additive. |
| Traveller pack (X3d, not started) | sensor ladder rule and constructs specified in prose | **Care 4:** X3d should build these on J4 and J7, not as Traveller specials. X3d now needs J4 and J7. |

Nothing changes state hashes of existing universes: a universe with no notes, no kept values and no programs folds to the same hash as before.

## 10. Stages

One package each. `needs` uses build-plan ids.

| Id | Package | Needs | Acceptance checks |
|---|---|---|---|
| J1 | The record: notes, kept kinds and the writer rule, `misc` fields, actors and relations, audience filtering in scenes, kernel requests | K3, K5, K9 | a note addressed to one audience is absent from another seat's scene and from what is sent to it; a kept value folds from its events and explains itself; an unlisted writer is refused; unloading the pack leaves values readable; an empty-record universe keeps its old state hash |
| J2 | Typed values and the logic atom set; conditions shared with K8; `roll` as a keyed draw; the engine block kind, with `classify` and `place` as the first two | K4, K8, C3 | every existing flow hash and golden hash unchanged; truth tables for every atom; a K8 rule and the same condition as blocks give identical results and evidence; rolls independent of evaluation order; `classify` wired into a flow gives the same label and explanation as the class atlas for the fixture worlds |
| J3 | Boxes: documents, nesting, inlining, explain, tests, replace | K4, K5 | a boxed flow equals its hand-inlined twin value for value and trace for trace; self-containing box rejected with a readable report; depth limit reported; a pack replaces a box explicitly |
| J4 | Triggers, effects, the gate and approval policies | J1, J2, V5 | the sensor ladder fixture moves a body up at the right range and time; an `ask` effect changes nothing until approved; a run replays bit-identically; pass limit reported with its chain |
| J5 | Intent registry in the kernel, pack-declared intent kinds, program-driven seats | J4, V10 | the five existing intents behave as before through the registry; a pack adds an intent with a validity condition; a program seat cannot send outside its `may_send` or scope |
| J6 | Programs: state machines, sensing through the owner's audience, ticks, look-ahead | J3, J5, T3 | the mining drone fixture surveys, travels, mines and returns; it never acts on a fact its sensors have not revealed (checked by state inspection); a look-ahead branch equals the committed run; the pirate, patrol and engagement fixture of 6.1 runs for a simulated year with background fights resolved, a player-ship fight stopped for approval, and the program limit enforced |
| J7 | Templates and `create` / `retire` | J1 | a ship made from a template carries its kept values and may run only its listed programs; removing the pack leaves the object valid |
| J8 | Turns and held orders | J4, V10 | three-seat fixture: held orders are invisible to other seats; the resolved state is identical for every arrival order; each seat's turn report holds only what it may see |
| J9 | Interface: record on cards and the time ruler, note entry for GM and players, relations lens, condition and box editing in the flow editor, program editor (state view), approval queue for effects | U2, U3, U4, J1 to J6 | screenshots at desktop and phone widths in both GM modes; a note added by a player appears for the GM and that seat only; a program edited by Orac shows as a diff before accept |

Orac's scope (A2) widens to boxes, conditions, programs, triggers, templates and kept kinds, all as validated change requests.

**Order.** J1, J2 and J3 need only parts that are already built and can start now on the kernel track. J4 waits for intents and approval (built in the UI, see Care 3). J6 waits for NAV. A 4X game pack is not a stage: it becomes possible after J8 and J9 as someone's pack, with the original author's agreement.

## 11. Rulings **[all five accepted by Alex, 19 Nov]**

1. **May players write notes themselves?** Yes: auto-approved for their own party journal, GM approval for the shared record.
2. **May a program act without the GM approving each move?** Yes, inside a scope the GM sets when attaching it (its own ship), and ask for anything that creates, destroys or leaves the scope.
3. **May the record change physics?** Only through the explicit `set_pin` effect, always asking the GM by default and always tagged. Never implicitly.
4. **Should classification be rebuilt as blocks?** No. It is an engine block (3.5): usable as a block anywhere, edited in the Class atlas, not in the block editor. Simple rules and guards still share the condition language.
5. **When?** J1 to J3 now on the kernel track, because the Traveller pack and autopilot both lean on them; the rest follows V5 and T3.
