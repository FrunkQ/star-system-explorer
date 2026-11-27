# SSE4: compound ships that separate and rejoin (track CM)

Status: **PROPOSED**, 27 Nov 2026. Design only (Fable); nothing here is built. CM is a code only and is never shown in the product.

Alex's ask (27 Nov 2026, 00:02 to 00:05 UTC, thread "Design compound ships that separate and rejoin"):

> "the simplest mechanism is to create compound ships and then have rules for how they separate and rejoin. And that means one rule can be used for every single one of these use cases": multi-stage rockets, ships within ships, drop ships, landing ships, shuttles, carriers with fighters.

> "You probably want to have the concept of in front, behind, inside, outside, and the hull for where this is placed in order that you can build up a topology that would allow you to actually build multi-stage ships using the ship 3D and [2D picture] designer." (corrected at 00:04: "not floor plan sorry... 2d pic")

> "look - most of these mechanisms exist already... just not likely all joined up"

He is right on both counts, and this note is built on the second one. **Most of the work is joining up what is on main.** The genuinely new pieces are small: one relation (the mount), one equation (staging), one step list (the sequence), and the designer reading the relation.

Read with: `SSE4-real-spaceflight-feasibility.md` (track RS, sections 6 and 6A), `SSE4-spaceflight-history-design.md` (track SH), `SSE4-atomic-rockets-plan.md` (sections 5 and 11, power buses), `SSE4-time-scales-design.md` (track TP, sections 4, 5 and 8), `handover/status/spaceflight.md`, `handover/status/travel.md`, `handover/status/manifest.md`, `handover/packages/RS4-split-join.md`, `handover/interface-style.md`.

Figures in this note are from memory and **unverified**: the paper search is out until 1 Dec 2026. Each one is marked "(unverified)" and is checked by the builder of the package that uses it, from the primary sources the RS track already cites.

---

## Part A. Audit: the history of spaceflight pack, honestly

### A.1 What it is and where it is

The pack is **"Sol today"**, `store-packs/sol-today/` in this repo (7.5 MB, `sse.pack/1`, id `sse.sol-today`, version 0.1.0). It was built by the thread "Build real spaceflight history pack" on 23 to 25 Nov (track RS, design accepted 23 Nov 22:01). Its plan is `handover/design/SSE4-real-spaceflight-feasibility.md`; its state is `handover/status/spaceflight.md`. The view over it (Roll call, In memory, Names carried) is track SH.

### A.2 What exists, and it is a lot

| Item | State |
|---|---|
| Catalogue (RS1) | Done: 7,223 orbital launches, 472 payloads that left Earth orbit, 469 landings and impacts, 770 human spaceflight records, 16 station cores; Wikidata cross-check |
| Parts (RS2) | Partial: `parts.json` has 5 launch vehicles, 13 stages, 17 engines from GCAT; `craft.json` has 13 craft written by hand from NASA sources. A long `to_read` list remains (Shuttle hardware, ISS power and life support, engine masses, Falcon 9 figures) |
| Flight logs (RS3, RS8) | Done to the "big burns and flybys" ruling: 1,235 logs. 44 showcase logs (Apollo 11 with every stage, STS-1, Falcon 9 flight 20, Cassini, Voyager 2, Juno, Dawn, Curiosity, JWST, ISS, V-2, SNAP-10A), 462 departed craft, 770 Earth-orbit flights; error measured against Horizons (Cassini cruise 1.4% at worst; ISS phase drift 720 km a day) |
| Crews (RS9) | Done: 792 travellers, 3,314 dated legs, the people-in-space count, In memory (17 losses, two sources each), 21 memorial asteroids with JPL orbits |
| Roll call, In memory, records, Names carried part 1 (SH1, 2, 3, 5, 6, 7) | Built and merged |
| Split and join, construct half (RS4) | Built in the kernel: `split_construct`, `join_constructs` (dock, undock, merge), parts as `part.<id>.*` facts, mass in `construct.mass_kg` |
| Split and join, log half | Built: `log_flight` with `split` and `join` events seeds the new construct's log with its parent's state |

### A.3 What is missing

1. **It is not installable yet.** It is not on Explorers (the creator hub repo has no trace of it), and nothing loads it into a universe. `pack.json` lists its files under `provides.other.files`, which no loader reads. The only way in is Roll call's address `?spaceflight=<folder URL>`, which fetches `crews.json`, `pack.json` and the logs straight from a folder. So no Apollo 11 construct ever exists in a campaign; the craft are files that one screen reads.
2. **The two halves of split and join were never wired together.** The kernel can split a construct's parts and mass; the flight log can record a split. Nothing calls one from the other (the travel status file lists it as "next"), and the pack's logs split into stage logs without any construct, mass or propellant behind them.
3. **Nothing describes a vehicle as a stack.** Apollo 11 is 9 separate logs whose events name each other. No record says "the S-IVB sits behind the CSM, the LM is inside the adapter". `craft.json` describes craft one by one, and `parts.json` lists stages without what they carry.
4. **The "SSE4 says" line does not exist (RS7, unowned).** The accepted plan has two lines on the chart, flown and computed, and the future-tech test compares them. Computing Saturn V's flight needs the stack's mass falling at each staging, and there is no staging maths (the Atomic Rockets plan lists Staging as "Missing").
5. **Ascent (RS5) is not started.** The pack's ascent pieces say so in their own words: "the ascent itself waits on RS5". Ascent is the place staging bites hardest: stages drop mid-burn.
6. **Shape hints for real hardware (RS6) are not started**, so no real craft has a picture.
7. **Data gaps, all listed in the package logs:** the RS2 `to_read` figures; RS8 part 6 (each flight's own landing point and undocking hour; 62 landing sites are stand-ins); RS9's named craters and Mars hills waiting on a second source, naming dates, the four ground losses waiting on a second source, animals other than Ham and Enos (McDowell lists primates only, so Laika is not in yet), Wikidata ids on everyone; SH4, SH7 part 2, SH8's short address and start card, SH9 tours and the edge of space as a Build value.

### A.4 Is staging the blocker?

**Partly.** For the history as a record (who was where, on any day) staging is **not** the blocker: the logs already split into one log per stage, and Roll call works without any stack. For the pack's other half, the parts-built craft that test SSE4's own physics against what flew, staging **is** the blocker, because items 3, 4 and 5 above all need one vehicle whose mass, propellant and engines change at each separation.

But it is not the only one. In order of what stops a person using it:

1. Not installable (A.3 item 1): a packaging and loading job, no physics.
2. Staging and the join-up of split and join (items 2 to 4): this design.
3. Ascent (item 5): needs this design first.
4. Pictures (item 6) and the data gaps (item 7): steady work, no design needed.

So the honest state is: **the record is about 80% there and usable through Roll call today; the "real craft from real parts, flown by SSE4" half is about 20% there**, and compound ships are what unlocks it. (Percentages are my reading of the package lists, not a measurement.)

---

## Part B. Compound ships

## 1. The short answer

1. **Anything that can ever fly on its own is a construct. Anything that cannot is a part.** A stage, a lander, a booster, a fighter, a shuttle, a drop pod, a station module: each is a construct with its own address, parts, budgets and flight log, from the moment it is made. An engine, a tank, a radiator: parts, as today.
2. **A compound is constructs held together by mounts.** A mount is one relation from a child construct to the one carrying it, saying **where** (in front, behind, inside, on the hull, outside), **at which port**, and **what passes across** (propellant, power, air, people). The compound is the tree of mounts at a given date. There is no compound object to keep in step.
3. **Two rules cover every case: release and capture.** Release ends a mount at a dated second; capture starts one. Staging, deploying a probe, launching fighters, a drop ship leaving, a lander leaving and coming back, docking at a station: all are one of the two, with different data.
4. **Nothing is moved, so nothing is lost.** Each construct keeps its own mass, propellant, stored energy and people at all times; a release or capture moves none of it. What crosses a mount (a stage feeding the engines above it, a station's air, people moving between craft) is an explicit dated transfer with an amount and both ends, and the sum is checked.
5. **The compound flies as one.** Only the root of a mounted tree has a position in its log; everything mounted on it reads "mounted on X, at its front", and is wherever X is. A release seeds the child's log with the parent's state at that second, plus its separation push.
6. **Staging is a sequence, not piloting.** A compound carries a plan of steps (burn on these engines until this tank is empty, release that, coast, burn) that the planner flies. Players never hand-pilot; a GM writes or picks the sequence.
7. **The designer builds from the same mounts.** The 3D designer and the 2D picture lay each construct out as now, then place the children by their mount: in front at the nose, behind at the tail, inside in a bay sized to hold them, on the hull at a station round and along it, outside on a tether or arm. Capabilities drive form, so a carrier's hangar is sized from the fighters it carries.
8. **Dates on the one axis.** Every release, capture and transfer is a record event at a whole universe second (nanoseconds where needed), durations are float differences only, and each publishes a mark through `happenings`.

## 2. What already exists, and what joins it

This is the inventory Alex asked for ("most of these mechanisms exist already"). The right-hand column is the whole of the join-up.

| Exists on main | Where | What CM does with it |
|---|---|---|
| Constructs with facts, templates, retire | kernel J7, `ui-contract/src/constructs.ts` | Every separable thing is one |
| Parts as `part.<id>.*` facts, `construct.mass_kg` | `sse-trigger/src/split.rs` (RS4) | Parts stay parts; only non-separable things |
| `split_construct` (moves parts to a new construct) | RS4 | Kept for building a compound from a flat one (an SSE3 import); a one-off, then mounts take over |
| `join_constructs` dock, undock, merge; `docked.to` | RS4 | Dock and undock become capture and release on the hull; `docked.to` is read as a mount on the hull (no migration of saves) |
| Flight log `split`, `join` events; `Docked {to}` state; seeding the new log | `sse-nav/src/flightlog.rs`, `sse-kernel/src/flights.rs` | `Docked` widens to `Mounted {to, place, port}`; `split` and `join` events are written by release and capture, never by hand |
| Burns with Δv, propellant and engine | flight log `What::Burn` | A compound's burn names which constructs' engines fire and which tanks they draw |
| Rocket equation, propellant for a Δv | `sse-nav` `Ship::dv_m_s`, `propellant_for` | Staging is the same equation over the sequence (section 6) |
| Power buses, sources, loads, waste heat | RK9, `sse-nav` `power`, kernel `ship_power` | A mount can carry a power line; the buses join across it |
| People aboard, `aboard.on`, air and supplies days | manifest MC1, MC5, MC9 | People are aboard a construct, never a compound; moving through a pressurised mount is an `aboard.on` change |
| Fuel kept values, refuelling | MC4, MC5, V8 fuel draw-down | Each construct's tanks are its own kept values; a cross-feed or a refuel at a station is a transfer between two |
| Entry, aerobraking, landing, take-off on a ship's own numbers | `ship_at_world`, `plan_descent` | A released lander or drop ship asks it with its own numbers |
| The ship designer (layout along a spine, styles, recipe), plans and side views | `renderer/src/ship-design.ts`, `ShipPlans` | Lays out each construct as now; places children by mount |
| The station designer, docking points by size | renderer `station-design.ts` (PR 408) | Docking points become hull ports |
| The Ships and stations card, Log, object menu | MC3, MC7, NV1 | Gains a "Carried" fold and Release and Capture actions |
| Dated record events, `happenings` | chronicle; TP6 | Release and capture are events and marks |

New: the **mount** (facts and two events), the **sequence** (step list), **cross-feed lines** on a mount, the **staging equation** as a named block, the designer's **placement** step and **bay** part, and the planner flying a sequence. Everything else is reuse.

## 3. Words

| Word | Means |
|---|---|
| **Compound** | Constructs held together by mounts at a given date. Shown as "the stack", "the carrier and its wing", "the station" by context |
| **Mount** | One construct carried by another: where, at which port, what passes across |
| **Carrier** and **carried** | The two ends of a mount. A stage is the carrier of the stage above it in a rocket only if the design says so (section 4.3) |
| **Root** | The construct at the top of a compound's tree, the one whose log holds the position |
| **Release** | Ending a mount at a dated second |
| **Capture** | Starting a mount at a dated second |
| **Line** | Something a mount passes across: propellant, power, air, people |
| **Sequence** | A compound's planned steps: burns, releases, captures, coasts |
| **Bay** | A part of a carrier with room inside it (a hangar, a payload bay, an adapter) |
| **Port** | A named place on a construct's hull where a mount can be made |

## 4. How a compound is described

### 4.1 The mount, as facts on the carried construct

A mount is facts on the carried construct, because a construct has at most one carrier at a time and many carried. All are ordinary facts, so history, Known to, undo and the record's state hash work with no new machinery.

| Fact | Meaning |
|---|---|
| `mount.on` | The carrier's address |
| `mount.place` | `front`, `behind`, `inside`, `hull` or `outside` (section 4.2) |
| `mount.port` | The carrier's port id, if it has named ports (optional) |
| `mount.own_port` | The carried construct's own port used (optional; a CSM's nose, a lander's top hatch) |
| `mount.station` | For `hull` and `outside`: `along_m` from the carrier's nose, `clock_deg` round its axis, and for `outside` `stand_off_m` |
| `mount.bond` | `fixed` (structure, opened only by a release mechanism), `latch` (a docking or berthing latch), `rest` (sitting in a bay or on a deck with no latch) |
| `mount.lines` | What passes across: any of `propellant`, `power`, `air`, `people`, with optional per-line limits (a pipe's flow, a cable's watts) |
| `mount.push_m_s` | The separation push a release gives, with its source (springs, retro rockets); default 0.5 m/s (unverified, a pack value) |

The carrier carries nothing new except, optionally, **ports** (`port.<id>.place`, `.station`, `.size_m`, `.lines`) and **bays** (a part kind with an inner length and radius, or a volume and a door size). A port's `size_m` is the docking size the station designer already uses.

Two events write and end mounts: `construct.mount` and `construct.release`. Each is one record event with both subjects, at one universe second.

### 4.2 The five placements (Alex's topology)

Every mount says where the carried construct sits relative to its carrier. The words are Alex's; what each means for physics and for drawing:

| Place | Means | Physics | Drawing | Examples |
|---|---|---|---|---|
| **In front** | On the carrier's axis, ahead of its nose | Thrust from the carrier passes through it in compression; it must take the carrier's acceleration (`structure_g`) | Its tail at the carrier's nose, on the axis | Apollo CSM on the S-IVB's adapter; an upper stage on a lower; a launch escape tower on a capsule |
| **Behind** | On the axis, at the tail | It pushes (a stage below) or is towed; if it has engines that fire, its thrust passes into the carrier | Its nose at the carrier's tail | A first stage under a second; a drop tank aft; a tug pushing a barge |
| **Inside** | In a bay of the carrier | Must fit the bay (length, radius or volume) and the door to leave; shielded by the carrier's hull and shield; shares the carrier's air only through a line | Drawn inside the bay, inked through the hull on the 2D picture | Eagle inside the adapter; a payload in the Shuttle's bay; fighters in a hangar; a drop ship in a liner's well; Huygens' pod is not this (it is on the hull) |
| **On the hull** | Fixed to the carrier's outer surface, at a station round and along it | Off the axis, so thrust through it is off-centre; strap-on boosters balance by symmetry (section 6.4); exposed to heating on entry | Placed radially at `clock_deg` and `along_m`, touching the hull | Shuttle orbiter on its tank; boosters on a core; Soyuz at an ISS port; a fighter on a pylon; a small craft docked on a liner |
| **Outside** | Held clear of the hull: on an arm, a tether or a truss, `stand_off_m` away | Like the hull for mass; no heating shadow; a tether carries tension only, so it cannot push | A line drawn from the port to it | Curiosity on the sky crane's bridles; a towed cargo pod; a tethered sensor; a captured asteroid on a net |

"On the hull" and "outside" were one word pair in Alex's message; this note reads them as touching and held clear. If he meant something else, that is question 2.

The five are enough for the cases below because a mount is one relation in a tree: a deep compound (a fighter on a pylon of a shuttle inside a carrier docked at a station) is four mounts, each simple.

### 4.3 Who carries whom

In a stacked rocket every stage could be "behind" the one above, or the payload "in front" of the one below. The rule: **the root is the construct that keeps the mission's identity** (the CSM keeps "Apollo 11", the orbiter keeps "STS-1", the carrier keeps its name), and the tree hangs from it. So Saturn V is the CSM with the S-IVB behind it, the S-II behind that, the S-IC behind that. This matches the pack's logs today, which already keep the CSM's log as "Apollo 11" from the pad to the Pacific (`keeps_identity` on each split).

The root can change at a capture (a fighter returning becomes part of the carrier's tree) and at a release (the released subtree's top becomes its own root).

### 4.4 Templates

A compound template is a pack's list of construct templates and the mounts between them, plus an optional sequence. "Saturn V with Apollo" is one; "Type T carrier with twelve fighters" is another; making one makes all its constructs and mounts in one event. A template can leave a slot empty ("bay 2: empty, holds up to 30 t").

## 5. Budgets, and how they are kept

The rule is the conserved chain's (accepted 26 Nov, PR 604), at ship scale: **every amount lives in exactly one place at a time, and moves only by a dated transfer with both ends.**

### 5.1 What each construct holds

| Budget | Held as | Exists |
|---|---|---|
| Dry mass | sum of its parts' `mass_kg`, or `construct.mass_kg` | yes (RS4) |
| Propellant, per tank and substance | kept values per tank (`fuel.propellant_t`, `fuel.jump_t`, and per tank where the construct has several) | yes (MC4, MC5); per tank is the small addition |
| Stored energy | kept charge per store (batteries, fuel cell reactants), sources by law (RK9) | sources yes; store charge is new, small |
| People and cargo | `aboard.on` actors | yes (MC1) |
| Air and supplies | days per store from who is aboard and the rates (MC9) | yes |

The compound's figures are sums over its tree at the date: total mass, total propellant by substance, people, days of air. They are read, never stored, so they can never drift from their parts.

### 5.2 What crosses a mount

A mount's `lines` say what may cross while it holds:

- **Propellant**: the carried construct's tanks may feed the carrier's engines, or the other way. The Shuttle orbiter's main engines burn from the external tank on the hull; a drop tank behind feeds the ship. A burn records which tanks it drew and how much (the flight log's `Burn` already has `propellant_t`; it gains a list of `{tank, t}` drawn), and the kernel checks the draw against what those tanks held at that second.
- **Power**: the two constructs' buses are one while the mount holds (RK9's ledger is already summed over sources and loads; it gains "sources on mounted constructs with a power line"). A docked Soyuz on station power is this.
- **Air**: the pressurised constructs pool their air and supplies while the mount holds; days are worked out over the pool (MC9 sums over the pool instead of one construct).
- **People**: someone may move between the two constructs. A move is an `aboard.on` change at a dated second (MC5 already logs these); without a `people` line the move is refused with the reason ("no passage between Columbia and Eagle until they are docked"), or offered as a spacewalk once spacewalks exist (later).

**Refuelling and transfers at a station** are the same: a transfer of propellant across a mount with a `propellant` line. V8's refuelling at a dock is this case.

### 5.3 What release and capture do to budgets

Nothing. A release ends the mount and its lines; each side keeps exactly what it held at that second. A capture starts a mount and its lines; nothing moves until a transfer says so. This is why the design is safe: there is no step at which mass can be counted twice or dropped.

**`merge`** (RS4) stays for the one case where two constructs truly become one and will never separate (welding a station module in for good, a GM folding a ship's hull into another). It moves parts and retires the other, as built.

### 5.4 The check

One test, absolute as the house rule asks (engine map PHY-34): for any compound history, at every event, the sum of every construct's mass plus every burn's recorded propellant used since a start date equals the start total, to the kilogram. A second: no tank is ever below zero. Both are run with the fix removed and seen to go red before being trusted.

## 6. Flight

### 6.1 Position

Only a root has a position piece in its log (orbit, course, landed, between stars). A carried construct's log has one piece, `Mounted {to, place, port}`, from its capture (or making) to its release; its position is its root's, plus the station offset, which only drawing needs. `Docked {to}` is read as `Mounted {to, place: hull}`, so every pack log and save already written keeps working.

### 6.2 Release

At a release the kernel writes, in one record event and its seeded log entries (the mechanism `log_flight` already uses for `split`):

1. the mount's end on the carried construct;
2. a `split` event on the carrier's log naming the carried, and a first piece on the carried's own log: the carrier's state at that second, changed by the separation push along the mount's direction (axial for front and behind, radial for hull, along the tether for outside);
3. a mark for `happenings`.

If the carrier is on a course rather than an orbit at that moment, the seed takes the carrier's state from its course (the flight log's `at` already gives position and velocity on a solved course); today that case is refused with a note ("give it its own first piece"), and this removes the refusal.

### 6.3 Capture

A capture needs both constructs in the same place at the same second, within a tolerance: the pack values `capture.reach_m` (default 10 m) and `capture.speed_m_s` (default 0.3 m/s), both unverified and editable. The kernel reads both logs at that second.

- Within tolerance: the mount is made; the captured construct's log gets `Mounted`; the carrier's log gets a `join` event.
- Not within tolerance: steer, never stop. The kernel says how far apart they are and offers a rendezvous: the planner's existing Lambert legs (`plan_voyage`) from the chaser to the target, with the Δv it costs from the chaser's own tanks. The GM can also write the capture "authored", marked, which is how a story moves on without the arithmetic.

### 6.4 Thrust and the stack

A burn on a compound names its engines (on any construct in the tree) and the tanks they draw (through propellant lines). Thrust passes to the root's position; the whole compound accelerates by thrust over the compound's mass at that moment. Strap-ons on the hull: their thrust is counted only if the mount's `bond` is `fixed` (a latch cannot take thrust), and if the strap-ons are not balanced round the axis (their `clock_deg` thrust vectors do not sum to near zero) the planner warns that the stack would turn. It does not model the turn: players never hand-pilot, and steering losses belong to the ascent block (RS5).

### 6.5 Staging: one equation over the sequence

The Δv of a staged vehicle is the rocket equation summed over the burns of its sequence, each with the mass of everything still mounted at the start and end of that burn:

  Δv_total = Σ_k v_e,k · ln(m_start,k / m_end,k)

with `m_start,k` the compound's mass when burn k starts (everything still mounted) and `m_end,k` that minus the propellant burn k draws. Releases between burns lower the next `m_start`. Parallel staging (boosters burning beside the core, then dropped) is the same sum with a blended exhaust speed over the engines firing in that burn, weighted by mass flow. Tsiolkovsky 1903 for the classic form (already cited for `dv_m_s`); the relativistic form in `voyage.rs` is the general case, as the Atomic Rockets plan rules. This is a **named Equation block** (the plan's "Staging: Missing, New Equation"), small and editable, with its fixed-length sum over the sequence.

So the "SSE4 says" line for Saturn V is: the CSM-rooted compound's mass table from its constructs' figures, the sequence from the pack, the staging block for Δv at each event, and RS5's ascent block for the first 12 minutes. The flown line is the pack's log. The difference is the future-tech test (FT2).

### 6.6 The sequence

A compound may carry a sequence: an ordered list of steps, each one of:

| Step | Says |
|---|---|
| `burn` | engines, tanks drawn, until a condition: a Δv, a tank empty, a time, an orbit reached |
| `release` | which mount, and the push |
| `capture` | which construct, at which port, after which rendezvous |
| `coast` | until a time, or a place (periapsis, a body's sphere) |
| `transfer` | an amount across a line (people, propellant) |
| `land`, `launch` | through `ship_at_world` on the construct that does it |

The planner flies the sequence and writes the flight log pieces and the record events it produces, each marked `planned` until the clock passes them in play, as voyages do now. A pack's historical sequence is marked `recorded`, so for the real craft both lines come from the same sequence: the recorded one with the pack's times, the computed one with SSE4's maths. A GM can write a sequence by hand, pick one from a template, or ask Orac to write one from a sentence ("drop the lander at 200 km, pick it up in a week").

### 6.7 Landing ships, drop ships, shuttles

These need no special: a release, then the released construct's own landing through `ship_at_world` and `plan_descent` on its own numbers (heat shield, legs, chutes, which the ship designer already sizes from capabilities). Coming back is a launch from the ground (ascent, RS5, or the existing powered take-off for a small airless world), a rendezvous and a capture. Eagle at the Moon is exactly this: release from Columbia's front, landing, ascent stage released from the descent stage (Eagle is itself a two-construct compound), launch, capture at Columbia's front, then the ascent stage released and left in lunar orbit.

## 7. Dates on the one time line

Following the time design (accepted 26 Nov 23:27 UTC):

- Every release, capture, transfer and sequence step is a record event at a `UniverseTime`: whole seconds from the origin with nanoseconds. No absolute time is ever a float. A staging event at T+2 min 41.6 s is stored as whole seconds and nanoseconds, never as a fraction of a day.
- A sequence step's durations ("burn for 150.7 s", "coast until periapsis") are differences, in floats, and every float is a difference from the step's anchor.
- The pack's logs keep their own seconds and state one anchor, as the time design already provides; no pack data is rewritten.
- Each event publishes a mark through `happenings`: kinds `release`, `capture`, `staging` (a release whose carried construct has spent tanks), `transfer`, with subjects, a `why` ("stage 2 empty, released") and the rank from the sifter. So the clock's "catch" slows down for a launch's stagings on Explore, Event › on Roll call stops at each one, and the Run desk's "what is due" lists the next release.
- Scale: separations are **jumps** in the time design's sense (instant, worked out from conservation at that second). A burn is a short process on its own time (its duration); the ascent block (RS5) marches inside its window with its own step, from its own anchor (the launch second), so jumping to T+5 min and stepping there give the same bits.

## 8. The designer: building compounds in 3D and the 2D picture

Alex: "a topology that would allow you to actually build multi-stage ships using the ship 3D and 2D picture designer."

### 8.1 How layout works today

`designShip` lays one ship's pieces along its spine (x from nose to tail, a radius per piece), from its needs (tanks from propellant mass and density, radiators from waste heat, shields, wings, legs from abilities) and a style and seed. The recipe (style, seed, overrides) is saved, never a mesh. `ShipPlans` draws the top and side views with call-outs. The station designer has docking points by size.

### 8.2 What it gains

1. **Each construct is laid out on its own**, exactly as now. A stage is a ship with no crew and its own tanks and engines; a fighter is a ship; a lander is a ship with legs.
2. **Placement step.** The compound's layout places each carried construct's layout by its mount:
   - in front: its tail at the carrier's nose, on the axis;
   - behind: its nose at the carrier's tail;
   - on the hull: rotated to `clock_deg`, at `along_m`, its side touching the carrier's hull radius there;
   - outside: as the hull, `stand_off_m` further out, with the arm or tether drawn;
   - inside: in the named bay, centred.
3. **Ports** are drawn as small rings on the hull, sized by `size_m`, from the carrier's facts. A construct with ports offers them in the designer as places to drop another construct.
4. **A new part kind, the bay** (`SystemKind` gains `bay`). Capabilities drive form: a bay is sized from what it is meant to hold (the largest carried layout's length and radius, times a clearance that is a style value), or from the GM's typed capacity. An adapter (the Saturn V's SLA, a payload fairing) is a bay whose walls are released with what it holds, which is a fairing: a construct of its own, released in front (or in petals on the hull) before the payload.
5. **Fit checks, warn never refuse**: a carried construct larger than its bay, a door smaller than what must leave, strap-ons that do not balance, a "front" mount on a construct with a drive at its nose: each a note in the layout's `notes`, as the designer already writes.
6. **The 2D picture** draws the compound's top and side views with each construct inked as its own outline, joints drawn as a thin break line, and call-outs naming each construct ("S-IC, first stage", "Eagle, inside the adapter"). Below it, a **sequence strip**: the compound's silhouette after each release, small, left to right, each labelled with its date ("T+2 min 42 s, S-IC released"). Pressing one moves the clock there.
7. **3D**: the same placement in the 3D designer. While the clock moves, released constructs separate along their push and are then drawn from their own logs, as constructs already are. Loss is never a spectacle in the real pack (SH rule 8.1.3): a lost craft's track simply ends.

### 8.3 Building the three examples

**A staged rocket (Saturn V with Apollo).** On the Edit desk, start from the CSM (a ship with crew and a heat shield). Drop the S-IVB behind it: the designer offers its front port. Drop the S-II behind the S-IVB, the S-IC behind the S-II. Drop a fairing-bay (the adapter) between the CSM and S-IVB: it is in front of the S-IVB, and the CSM is in front of it. Drop Eagle inside the adapter. Drop the escape tower in front of the CSM. Tick "propellant" on no mount (each stage burns its own tanks). Write the sequence from the template: burn S-IC to empty, release, burn S-II, release (and the tower), burn S-IVB to orbit, coast, burn S-IVB for the Moon, release the CSM from the adapter's front, capture Eagle at the CSM's nose, release Eagle from the adapter, release the S-IVB. The 2D picture shows the stack and its strip of seven silhouettes; the staging block gives Δv per burn.

**A carried shuttle (STS-1).** Start from the orbiter (the root: it keeps "STS-1"). Put the external tank on the orbiter's hull: strictly the orbiter is on the tank's hull, but the root rule (4.3) keeps the orbiter as root and the tank hangs from it at the orbiter's belly, `clock_deg` 180. Tick "propellant" on that mount, so the orbiter's three main engines draw the tank. Put the two solid boosters on the tank's hull at `clock_deg` 90 and 270: they balance, so no warning. Sequence: burn main engines and boosters together (parallel staging, blended exhaust speed), release both boosters at burnout, burn on to cut-off, release the tank, burn the orbital manoeuvring engines. The same pattern with a carrier aircraft is the orbiter on the hull of a 747 for a ferry flight, and the X-15 on the hull of a B-52's wing.

**A ship docked on the hull (a small craft at a liner, or Soyuz at the ISS).** Start from the liner (or the station). Its designer, or the station designer, gives it ports. Drop the small craft on a hull port with bond `latch` and lines `power, air, people`. On Play, the GM's object menu on the small craft says "Release from Beowulf" and on the liner "Capture…", which lists what is in reach. People move through the port with the Aboard panel's existing move.

Carriers with fighters are the inside case: a hangar bay sized from twelve fighter layouts, each fighter mounted inside with bond `rest` and lines `propellant, power`. "Launch all" is a sequence of twelve releases at a spacing (a pack value for a launch rate, see section 11 on the game pack).

## 9. The real craft, from the one mechanism

None of these needs a special. The left column is what the pack needs (RS feasibility sections 6 and 8); the right is how CM gives it.

| Vehicle | What happens | How |
|---|---|---|
| Saturn V, Apollo 11 | Three stages drop, escape tower, transposition and docking, LM extraction, LM landing and ascent, ascent stage left, SM released before entry | Section 8.3: mounts front, behind, inside; releases; one capture at the Moon; Eagle is two constructs (ascent in front of descent) |
| Space Shuttle, STS-1 | Boosters and tank drop, tank feeds the orbiter's engines | Hull mounts, a propellant line, parallel staging |
| Falcon 9 | Two stages; the booster flies home and lands | Behind mount; release; the booster's own log and a powered landing through `ship_at_world` (and RS5 for the return burns) |
| Voyager 2 | Centaur and the solid kick stage drop | Behind mounts, releases (the pack already splits off the Centaur) |
| Cassini-Huygens | Huygens released, lands on Titan | Hull mount (Huygens sat on Cassini's side; unverified), release, Huygens' own entry and landing |
| Curiosity | Cruise stage, heat shield, back shell, sky crane lowering the rover on bridles, crane flies away | Behind (cruise stage), front (heat shield), inside (rover in the back shell), outside (rover on the crane's tether), releases in order |
| JWST, Juno, Dawn | Upper or kick stage drops | Behind mount, release |
| ISS | About 40 modules joined over 13 years (unverified count), visiting crews come and go | Modules are constructs captured at ports on the hull and never released; each visit is a capture and a release; people move through `people` lines; station air pooled |
| Apollo-Soyuz | Two crews meet through a docking module | Capture at a port, people line, a transfer of people both ways |
| Hubble servicing | The Shuttle captures Hubble into its bay and releases it | Capture inside, release |
| V-2, SNAP-10A | Single craft | A compound of one: nothing to add |

The pack's existing logs already contain these splits and joins as events; CM gives each a construct, its mass and propellant, a mount to end, and a sequence that both lines are flown from.

## 10. Screens on each desk

The desks are Explore, Edit and Play in Alex's words (Explore, Build, Run in the docs).

| Screen | Explore | Edit | Play |
|---|---|---|---|
| **Ships and stations card** | A "Carried" fold: the compound's tree at the date in view, each construct a row with where it is mounted and its mass; compound totals (mass, propellant by kind, people, days of air) | The same, editable: drop constructs onto places and ports, set lines and pushes, write or pick the sequence | The tree at the campaign clock; the GM's Release and Capture actions; players see what Known to shows (a carrier's hangar contents may be secret) |
| **2D picture and 3D designer** | The compound drawn, the sequence strip, pressing a silhouette moves the clock | Placement and bays as section 8; fit notes | Locked (MC3 rule: the ship itself changes only on Edit); the strip and the drawing follow the clock |
| **Voyages and Route** | Δv per burn from the staging block; the sequence's stagings marked on the leg | Plan a voyage for a compound: the planner flies its sequence | The GM commits; the flight log writes release and capture events at their seconds |
| **Roll call** (SH) | Craft grouped by compound ("Apollo 11: Columbia, with Eagle docked"); Event › stops at each staging | | Same |
| **Log** (MC7) | "Released Eagle", "Captured Eagle at the nose", "Moved Armstrong and Aldrin to Eagle" | | Same, at the campaign clock |
| **Object menu** (NV1) | | "Mount on…", "Make a compound template" | "Release", "Capture…", "Launch all" (for a bay), greyed with reasons ("Eagle is 3 km away; plan a rendezvous") |

Depth: Curious shows the tree and the totals; Science adds each construct's budgets, the lines' flows and the staging block's workings; Engine adds the record events and addresses.

Phone: the Carried fold is a list; the sequence strip scrolls sideways; the 2D picture is above the list, shut by default as other pictures are.

## 11. Releases

| In 4.0 | Later |
|---|---|
| The mount, release and capture, budgets and lines, the check (CM1 to CM4) | Spacewalks as people moving without a line (4.2 RPG) |
| The staging block and the sequence, the planner flying it (CM5, CM6) | Damage that breaks a mount (4.2) |
| The designer's placement, bays, ports and the 2D sequence strip (CM7, CM8) | Interior views of bays (Alex: no deck plans) |
| The real pack's showcase craft as compounds, and the "SSE4 says" line for them (CM9, CM10; with RS5 and RS7) | Economy autopilots running carriers and shuttles (4.1) |
| Making the pack installable (CM0) | Launch-and-recovery tactics in combat (4.2) |

**Game-pack features, flagged for Alex** (4.0's only game pack is Traveller): carriers, small craft, drop tanks and launch tubes are common in Traveller's ship rules (from memory, unverified). The **mechanism** (mounts, bays, releases, capture, drop tanks as a propellant line behind) is base and general. Any **Traveller rule** on top (hangar capacity in displacement tons, launch and recovery rates per combat round, the cost of a docking clamp, drop tank prices) is a game-pack feature and goes to Alex before it is built (question 10). It never carries the TAS logo.

## 12. Honesty and fudges

- Separation pushes, capture reach and speed, and bay clearance are pack values with sources where known, otherwise marked estimated, in the fudge register.
- A capture written "authored" without the rendezvous arithmetic is marked so, and the Engine depth says how far apart the logs had them.
- The stack's turning moment from unbalanced thrust is warned, not flown: a fudge row ("thrust off the axis is checked, not flown").
- The parallel-staging blended exhaust speed is exact for the sum of mass flows, and is a named block.
- Nothing in the 2D picture or 3D placement feeds physics: the kernel's figures make the drawing, never the reverse.

## 13. Packages

| Package | What | Needs | Owner | Golden |
|---|---|---|---|---|
| **CM0** | Make Sol today installable: a loader that reads `provides.other.files` into a campaign (catalogue as a list, logs on demand, constructs for the showcase craft), and publishing the pack on Explorers. Roll call reads from the loaded pack, `?spaceflight=` stays as a shortcut | nothing new | spaceflight pack thread with the chart room; Explorers storage is ours (26 Nov ruling) | none |
| **CM1** | The mount: facts, `construct.mount` and `construct.release` events, ops `mount_construct` and `release_construct`; `docked.to` read as a hull mount; ports and bays as parts | RS4 | kernel | core: new ops in the op list (one slot) |
| **CM2** | Flight log joined up: `Mounted {to, place, port}` (reading `Docked` as hull), release seeds the carried log with the carrier's state plus push (including from a course), capture checks reach and speed and offers a rendezvous; `log_flight`'s split and join written by CM1's ops | CM1 | travel thread (sse-nav) with kernel | area `nav` |
| **CM3** | Budgets across mounts: per-tank kept values, stored energy per store, lines (propellant draw list on burns, power buses joined, air pooled, people moves needing a passage) | CM1, MC4, MC9, RK9 | manifest thread with travel | area |
| **CM4** | The conservation check: absolute, run with the fix removed and seen red | CM3 | kernel | test only |
| **CM5** | The staging Equation block, classic and relativistic, parallel staging; cited | CM3 | kernel (with RK3, the ship equations) | core if `dv_m_s` moves to a block; with RK3's slot |
| **CM6** | The sequence: steps as data, the planner flying it (burn until, release, capture, coast, transfer, land, launch), marks to `happenings` | CM2, CM5, TP6 for the marks | travel | area `nav` |
| **CM7** | Designer placement: each construct's layout placed by its mount; ports drawn; the `bay` part sized from what it holds; fit notes | CM1 | renderer | none |
| **CM8** | The 2D picture for compounds: outlines per construct, joints, call-outs, the sequence strip; the Carried fold on the Ships and stations card; object menu actions | CM6, CM7 | renderer, manifest and navigation threads | none |
| **CM9** | The showcase craft as compounds: Apollo 11, STS-1, Falcon 9, Curiosity, Cassini-Huygens, ISS assembly, with constructs, mounts and recorded sequences from the existing logs; the RS2 `to_read` figures they need | CM1 to CM6, RS2 | spaceflight pack thread | none (pack data) |
| **CM10** | The "SSE4 says" line for the showcase craft: the sequence flown by SSE4's own maths beside the recorded one, the difference reported as the future-tech test | CM9, RS5 (ascent), RS7 | spaceflight pack thread with the chart room and future-tech | none |
| **CM11** | Compound templates in packs: a base set (two-stage rocket, lander and orbiter, carrier and wing, station and visitor) for any campaign | CM1, CM7 | pack thread | none |

Order: CM0 can start now and does not wait for the rest. Then CM1, CM2 and CM7 (the three that join up what exists), then CM3 to CM6, then CM8 to CM11. RS5 (ascent) stays its own package in the travel thread and should be designed after CM5, because the ascent block reads the compound's mass table.

## 14. Questions for Alex, each with a recommendation

1. **Anything that can fly on its own is a construct; anything that cannot is a part.** Stages, landers, fighters and modules become constructs from the moment they are made, held together by mounts, instead of parts split off later. Recommended: yes. It is the one rule that makes release and capture move nothing, so budgets cannot drift.
2. **The five placements: in front, behind, inside, on the hull (touching), outside (held clear on an arm or tether).** I read your "outside" and "the hull" as those two. Recommended: yes, with the sky crane's tether as the proof that "outside" is needed.
3. **Release and capture as the only two rules,** with `merge` kept only for permanent fusion. Recommended: yes.
4. **The root keeps the mission's name** (the CSM is Apollo 11 from pad to splashdown; the orbiter is STS-1 with the tank hanging from it). Recommended: yes; it matches the pack's logs already.
5. **What crosses a mount is a dated transfer with both ends** (propellant cross-feed, power, pooled air, people through a passage), checked by one absolute conservation test. Recommended: yes.
6. **Capture needs both craft at the same place and speed within two pack values; when they are not, the app offers a rendezvous, never refuses, and a GM can write it as authored.** Recommended: yes, 10 m and 0.3 m/s as defaults (unverified).
7. **Staging is a sequence the planner flies, never piloted.** GM writes it, picks it from a template, or asks Orac. Recommended: yes.
8. **The designer places constructs by their mount, gains a bay sized from what it carries, and the 2D picture gains a sequence strip.** Recommended: yes.
9. **Make the pack installable first (CM0)**, before any of the compound work, since today it can be reached only through Roll call's address. Recommended: yes; it is the biggest thing between the pack and its users and needs no new design.
10. **Traveller carriers, small craft and drop tanks:** the mechanism is base; any Traveller rule on top (hangar tons, launch rates per round, prices) is a game-pack feature for you to approve before it is built. Recommended: hold the Traveller rules until CM1 to CM8 are on main, then decide.
11. **Release:** CM0 to CM11 in 4.0; spacewalks, damage breaking mounts and combat launch tactics later. Recommended: yes.
12. **Owners:** kernel for CM1, CM4, CM5; travel for CM2 and CM6; manifest for CM3; renderer for CM7 and CM8; the spaceflight pack thread for CM0, CM9, CM10. Recommended: yes, with the coordinator queuing CM1's golden slot.

## 15. Not in this design

- Ascent physics itself (RS5): its own design, after CM5.
- Shape hints for real hardware (RS6) beyond the placement step.
- Spacewalks, damage and break-up.
- Any change to the time axis or `happenings` beyond publishing the four mark kinds.
