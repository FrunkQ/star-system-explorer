# Star System Explorer 4: design and build handoff

Status: design baseline for the coding session. Date: 17 Nov 2026. Owner: Alex (FrunkQ).
Companions: `SSE4-galaxy-design.md`, `SSE4-life-design.md`, `SSE4-surface-design.md`, `SSE4-surfaces-and-rendering.md` (research: rendering, hardware scaling, software choices), `SSE4-build-plan.md` (same folder), the "SSE4 mockups" canvas (every screen referred to here), "SSE4 decisions for Alex" (the reasoning trail), "SSE4 high-level design".

This document is written so that a session with no other context can start building. Where something is a decision by Alex it is marked **[decided]**. Where it is my recommendation and Alex has not ruled, it is marked **[recommended]**. Where it is unproven and needs evidence before anyone relies on it, it is marked **[prove first]**.

Rules for the coding session that are not design: never push without Alex's word; SSE commits are authored as FrunkQ <frunk@frunk.net>; real user files stay outside the repo; the repo is https://github.com/FrunkQ/star-system-explorer on a V4 branch or folder agreed with Alex; V3 stays untouched and live.

---

## 1. What SSE4 is

A total rewrite with its own purpose-built engine **[decided]**. Two purposes, and every feature should serve one of them:

1. **The editor and physics pipelines are astrophysics education.** A keen amateur learns real astrophysics while building things for a game, a story or curiosity. The physics is real, visible and editable. Never fake it.
2. **Reports and player views are a space RPG and storytelling tool.** Relatable, surfacing interesting stories and information from the same facts.

One sentence for the architecture: **everything in the universe is a probability cloud that resolves deterministically when someone looks at it, records what it resolved as a causal history, and is then advanced, edited, explained and shown through declarative flows, packs and views.** The same machinery serves stars and planets now, and galaxy evolution and exobiology later **[decided: one target architecture, "probabilities to deterministics"]**.

Principles, in priority order:

1. Deterministic and procedural. Same universe seed, same packs, same authored edits, same engine version gives the same universe on every device. No hidden randomness, no wall clock, no dependence on what the UI looked at first.
2. Lean into the physics. Every derived number can answer "why?" with the flow that produced it, its inputs, units and source.
3. The GM can override anything. Physically impossible edits are allowed and tagged, never blocked **[decided]**.
4. Data, not code, is what users share. Flows, packs, screens and universes are declarative documents.
5. Don't recreate V3. Improve it. Players command computers; they never hand-pilot.
6. No scale is privileged. One system, five points across three galaxies, or a whole galaxy are equally valid **[decided]**.
7. Fast and simple to edit. Fancy lives in player views.
8. Claims are labelled. Exact, approximate and "consistent reconstruction" are never confused (section 12).

Out of scope for the first release: the exobiology suite and the galaxy evolution suite themselves (the sockets for them are in scope), dogfighting, selling/licensing mechanics, native apps.

---

## 2. Decisions already made

| Topic | Decision |
|---|---|
| Engine | Own engine. Ovid Next is inspiration for the shape (node editing, named agents), not a runtime. |
| Language | Rust kernel compiled to WebAssembly; TypeScript for UI, views, agents, Cloudflare. "Full Rust" allowed if beneficial; my ruling is in section 3. |
| Licence | Apache-2.0 with a NOTICE file crediting Alex, for SSE4 and Evolution Lab (17 Nov; replaces GPL). |
| Hosting | Cloudflare, new V4 subdomain. V3 stays on its URL; beta pushed to prod and then sleeps. |
| GM look | "Chart room" cartographic look is the default and the light mode; a traditional dark mode exists. GM view only. Lean into the cartographic feel. |
| Player views | Every player view is an editable object with supplied defaults. Infinitely customisable, assembled by Orac from a finite stack of combinable views. |
| Player data views | GM-defined labelled filters over player views; a default; per-planet adjustment; a single "scanned" button. Players hacking their own instance for data is acceptable. |
| GM lenses | Saved presets for what the GM view shows (Basic, Advanced, Building, Running, thematic such as Political, and every player data view as a lens). One mechanism with player data views. Section 8.6. |
| Seats | One shared player view by default. Optional extra seats for per-player fog of war, multi-screen and crew roles, hidden until asked for. Section 8.7. |
| GM workspace | Built from the same views, lenses and screens as player screens, shipped as a fixed curated layout. Section 8.8. |
| Surfaces | Every world gets a deterministic surface built from its history; one height map is the single source for every surface picture; it resolves on survey. `SSE4-surface-design.md`. |
| Orac-first configuration | Every UI element has rich, schema-described settings for Orac to work with; people get a few promoted controls. Section 8.9. |
| GM offline | Player views go read-only showing the last snapshot. |
| V3 | One-way import V3 to V4. Explorers site holds both, labelled V3 or V4. Space Engine and Universe Sandbox 2 converters need rework. |
| Packs | Layered and stackable: game-system packs, universe packs, physics packs. "Traveller in Star Wars with magic" must work. |
| Classification | Extendable rules in packs, not code. |
| Time | First-class. Look ahead to a fate and optionally move system time there. Top use case: ageing the Solar System. Deep-time ageing is a GM edit function for capable hardware. |
| Universe | Deterministic from Big Bang to heat death, no particle simulation. Real catalogues and measurements only in the real Milky Way, fetched dynamically there as pins (18 Nov). Edits change a patch going forward. |
| Sky | Every view's background should be the likely real sky for that place and time (section 10). |
| Agents | Tars (support, allowed to be funny), Orac (designer/builder), Minerva (describing and storytelling). Bring your own key. Reuse the Ovid agent system. |
| Devices | Each device shows its optimal version. Some views need GPU filters. |
| Pace | No release date pressure. V4 must bring new things, not parity alone. |

---

## 3. Language and runtime ruling

**Rust for everything that must be deterministic; TypeScript for everything a person looks at. Not "full Rust".** [recommended, within the freedom Alex gave]

Why Rust for the kernel is now firm rather than a preference:

- Players run the same kernel as the GM and the GM streams only data. That only works if the kernel gives bit-identical results on a GM's desktop Chrome and a player's phone Safari. JavaScript cannot promise this: `Math.sin`, `Math.exp`, `Math.pow` differ between engines, and string ordering depends on locale. WebAssembly arithmetic is specified exactly, and a Rust kernel carrying its own maths library (`libm`, pinned) gives the same bits everywhere. Evolution Lab's own concept audit flagged exactly this weakness in a TypeScript core.
- Universe resolution, deep-time ageing and route planning are tight numeric loops over large arrays. Rust in WebAssembly gives predictable speed with no garbage collection pauses, and the same crates run natively for tests, the command-line tool and any future server.
- Dimension-checked quantities, exhaustive enums and no implicit nulls catch the class of unit and state bugs that V3 accumulated.

Why not Rust for the UI: the editor must be quick to iterate, accessible, and good on phones; the web platform already does text, layout, input and accessibility well; Evolution Lab's reusable components are Svelte; and view plug-ins by the community are far more likely in TypeScript. Rust UI frameworks in the browser are heavier and less mature for this kind of dense editing surface.

Stack:

- **Kernel**: Rust 2021+, `wasm32-unknown-unknown` via `wasm-bindgen`, run in a dedicated Web Worker. No threads required for correctness; optional worker pool for independent patch resolves (results are order-independent by construction, so parallelism cannot change output).
- **Numbers**: `f64` with pinned `libm` for transcendental functions; no fused multiply-add, no fast-math; fixed iteration counts or deterministic convergence tests; canonical NaN policy (a NaN leaving an atom is an error, never a value). Ledgers that must close exactly (mass budgets claimed from a parent cloud) use integer minor units.
- **Hashing**: SHA-256 for content addresses and state hashes (not 32-bit checksums). BLAKE3 acceptable if pinned.
- **Serialisation**: canonical CBOR (deterministic encoding) for kernel state, chronicle and network frames; JSON with a canonical form for human-edited documents (flows, packs, screens). One schema source: Rust types with `serde` + `schemars`, TypeScript types generated (`ts-rs` or from the JSON Schema). Never hand-write a type twice.
- **Shell and views**: TypeScript, Svelte 5, Vite, built as static files for Cloudflare Pages. Reason for Svelte: V3 and Evolution Lab both use it and Evolution Lab's shared components then mount directly.
- **Rendering** [recommended by the rendering research, 18 Nov]: three.js `WebGPURenderer` (one shader source for WebGPU and the WebGL2 fallback) with SSE4's own planet layer; SVG/Canvas 2D for charts and the lowest tiers. Tiers are `text`, `canvas`, `gpu-basic`, `gpu-high` and `still` (progressive photo-quality rendering while the camera rests, with a "take photograph" action). A view declares the tiers it supports; a start-up probe picks one; a frame-time governor steps quality down and up by the recipe's degrade order and a small badge says what was reduced and why. Details in `SSE4-surfaces-and-rendering.md`.
- **Edge**: Cloudflare Pages (static apps, kernel `.wasm`, catalogue tiles on R2), one Worker for signalling/relay and one for the explorers site API. No simulation runs on the server.

---

## 4. Architecture at a glance

```
            packs (declarative, layered)        authored edits (GM)
                      |                                |
                      v                                v
+------------------------------- KERNEL (Rust, worker) ------------------------------+
|  substrate: ids/addresses · quantities · keyed draws · chronicle · stores · hashes  |
|  engines:   RESOLVE (clouds -> facts) · DERIVE (flows) · EVOLVE (time, branches)   |
|             QUERY (feasibility) · EXPLAIN (why) · SIFT (significance, stories)     |
|             OBSERVE (player data views) · NAV (routes, hazards)                    |
|  suites:    physics (stars, planets, orbits)  [first]                              |
|             cosmos (galaxy evolution) · life (exobiology) [sockets, later]         |
+----------------------------------+--------------------------------------------------+
                                   | one message API (section 9)
        +--------------------------+---------------------------+
        v                          v                           v
  GM shell (chart room)      player apps (web-loaded)     CLI / tests (native)
  editor, flow editor,       views, screens, astrogation   golden hashes, import,
  deep time, story finder    intents back to GM            pack validation
        |                          ^
        +---- P2P data channel ----+   (data only: authored edits, chronicle deltas,
                                        clock, data-view state, state hash)
```

Three layers, strictly separated:

- **Substrate** knows nothing about stars or life. It provides identity, quantities with units, randomness, history, storage and hashing.
- **Engines** are generic operations over the substrate. They know about clouds, flows, events and time, not about astrophysics.
- **Suites** supply the domain: atoms (compiled operators), flows, rules, cloud definitions, sifters and default views. The physics suite ships first. A suite is the unit of "drop in an exobiology evolution suite later".

Dependency rule: suites depend on engines, engines on substrate, never the reverse. The kernel imports no UI, DOM, network or clock. The UI never computes physics.

---

## 5. Substrate

### 5.1 Addresses and ids

Everything has a stable hierarchical address. It is the key for seeds, storage, network topics and explanations.

```
u:<universe-id>
  /fil:<i,j,k>            filament cell        (tier 6)
  /gal:<n>                galaxy               (tier 5; group is a relation, not a container)
  /patch:<level>:<morton> galaxy patch, octree (tiers 4 to 3: cluster, neighbourhood)
  /sys:<n>                star system          (tier 2)
  /body:<path>            star, planet, moon, belt, ship, station (tier 1)
  /shell:<id>             interior, surface, ocean, atmosphere layers
  /region:<id>            plate, biome, basin, city, aerostat band
  /feature:<id>           a thing in a region
```

Rules: addresses never change once issued; a child's address is derived from its parent's and its index within the parent's resolve, never from creation order in the UI; authored objects get addresses in an `authored` namespace under their parent so they cannot collide with generated ones; renaming changes a label, never an address. Everything addressable can be renamed by the GM in place, at every tier: galaxies, patches, stars, worlds, regions, surface features, guilds, events and epochs. A rename is a dated authored event; the generated name is kept as the original; an object may hold several names (by date, culture or audience).

Tiers are a convention of the cosmos and physics suites, not a kernel enum. A life suite continues the same tree downward (`/habitat:`, `/lineage:`).

### 5.2 Quantities

`Quantity { value: f64, unit: UnitId, dim: Dimension }` with SI base internally and display units chosen by packs (AU, solar masses, Earth radii, bar, Traveller codes). Every wire in a flow carries a quantity kind: an id such as `planet.equilibrium_temperature`, a dimension, a valid range, and a short human description. Connecting mismatched dimensions is a validation error at edit time, not a runtime surprise. Non-scalar kinds: curves (spectra, profiles), distributions (for clouds), enumerations, references to other addresses.

### 5.3 Keyed draws

One master seed per universe **[decided]**. Every random number is a pure function:

```
draw(universe_seed, address, stream_name, counter) -> u64     (counter-based generator, e.g. Philox or a SHA-256-derived stream; pick one, pin it, version it)
```

No generator object is ever shared or advanced by anything else. Consequences that the code must preserve: resolving patch A before or after patch B changes nothing; adding a new decoration stream to stars cannot reshuffle planets; a player device can resolve one body without resolving its neighbours. The stream name includes a version (`planets.count.v1`) so a pack can change an algorithm without silently changing old universes.

### 5.4 The chronicle

An append-only causal history. Everything that happens is an event; nothing is overwritten.

```
Event {
  id: content hash
  kind: "resolve" | "authored.set" | "authored.event" | "derive.milestone" | "evolve.transition"
        | "commit.branch" | "observe.grant" | suite-defined kinds
  subject: Address[]                 what it affects
  occurred_at: UniverseTime          when it happened in the universe
  activated_at: UniverseTime         when its effects begin (light-time, delays)
  recognised_at: SessionStamp        when it entered this chronicle (ordering only, never physics)
  causes: EventId[]                  why
  payload: typed data                deltas, the facts fixed, the envelope consumed
  evidence: "catalogued" | "derived" | "resolved" | "authored" | "authored-impossible" | "narrative"
  publisher: pack id + flow/atom id + version
}
```

Three times are kept distinct (borrowed from Evolution Lab: occurred, activated, recognised). `recognised_at` must never influence a physical result.

State at time T for address A = fold of the events affecting A up to T, plus derived values computed from that state. Checkpoints (content-hashed snapshots) make this fast; they are a cache, never the truth.

### 5.5 Stores

- **Authored store**: the GM's edits as events. Small. This is what a saved universe mostly is.
- **Resolved store**: facts fixed by resolving clouds. Reproducible from seed + packs + authored events, so it can be dropped and rebuilt; it is saved anyway so old universes do not shift if a later pack version changes.
- **Derived cache**: outputs of flows. Always discardable.
- **One publisher per quantity**: for a given address and quantity kind, exactly one flow (or one authored pin) publishes the value under the active pack stack. Conflicts are a pack validation error, shown to the user, never resolved silently.

### 5.6 Run manifest

A shareable universe is identified by: universe seed, seed algorithm version, kernel version, the exact pack stack with content hashes, the authored chronicle hash, ordering/quantisation version, and optional checkpoint hashes. The explorers site stores this manifest; the V3/V4 label lives beside it.

---

## 6. The engines

### 6.1 RESOLVE: clouds to facts

A **cloud** (Evolution Lab calls the summary a formation-context envelope) is what a parent knows about children it has not looked at yet: typed distributions and budgets.

```
Envelope {
  address, tier, epoch
  budgets:      conserved totals the children must be drawn from (gas mass, stellar mass, metals by group, angular momentum, count expectations)
  distributions: age, metallicity, density, velocity dispersion, radiation field, ...
  legacies:     what died here before (supernova counts by type, neutron-star mergers, AGN episodes)
  pins:         authored or catalogued children that must appear
  uncertainty:  declared spread, used for honest labelling
}
```

`resolve(envelope, child_address) -> (facts for the child, the child's own envelope)`.

Hard rules:

1. **Deterministic and local.** Uses only the parent envelope, the child address, keyed draws and pins.
2. **Order-independent.** Children are defined by index from a partition of the parent's budget that is itself drawn deterministically up front (a stick-breaking or hierarchical split keyed on the parent address), so resolving child 7 never requires resolving children 0 to 6 beyond that cheap partition.
3. **No double counting.** A child claims from the parent's budget; the sum of all children equals the parent budget in integer minor units. Pinned children (authored, catalogued) claim first.
4. **Resolution never depends on zoom or UI.** The UI may request resolution; it cannot change the result.
5. **Resolved facts are recorded** as a `resolve` event, so they persist and can be explained ("this system is metal-rich because two generations of supernovae enriched this patch; here is the patch's history").
6. **Coherence across time.** A cloud at epoch T2 must be consistent with what was resolved at T1. Resolution therefore draws the child's whole life line (formation time, evolution, end) once, and any epoch is a read of that line. [prove first: this is the hardest property; see section 12.]

Evolutionary galaxy model **[decided in outline]**: a patch's envelope at time T is produced by a simplified, editable flow from its earlier envelope and its neighbours' summaries: gas inflow, star formation, enrichment from stellar deaths, feedback. No particles. Stars that resolve carry positions and velocities and so move, pass each other and die; cross-patch effects (a supernova, a passing star) are events that modify the neighbouring envelopes going forward.

Edits: a GM edit is an authored event with an `occurred_at`. It changes the patch from that time forward. It never rewrites earlier resolved facts. If it contradicts physics it is tagged `authored-impossible` and everything derived from it inherits a visible "impossible input" badge; the GM explains it as magic.

### 6.2 DERIVE: flows

A **flow** is a directed graph of typed quantity wires through **atoms** (compiled, tested operators), grouped into **boxes** (named sub-flows) and attached to a **hook** (where in a pipeline it runs, e.g. `planet.climate`). This is the node editor and it is the education surface.

```
AtomDecl { id, version, inputs: Port[], outputs: Port[], params: Param[],
           validity: ranges where the physics holds, cost_class, references: citations,
           explain: short text + formula in display form }
FlowDoc  { id, version, hook, nodes: [{atom|box|solve, params}], wires, publishes: QuantityKind[], tests: golden cases }
```

- Atoms are Rust. Flows are data. A user changes physics by rewiring atoms, changing parameters, or swapping a box, never by writing code.
- `Solve` nodes handle implicit systems (climate equilibrium, interior structure) with a fixed method and a deterministic stopping rule.
- Outside an atom's validity range the result is still produced but labelled `extrapolated`.
- **Heavy blocks**: an astrophysicist with a beefy PC can swap a simplified box for a detailed one. Two routes. (a) The detailed box runs on their machine and its *results* are saved as data in the pack (tables keyed by inputs), so everyone else gets the answer without the cost. This ships first. (b) Later, sandboxed WebAssembly component atoms with a declared interface and a determinism test, signed in a pack. [recommended, phase 2; needs a trust decision.]
- New base physics blocks get specced and raised as GitHub issues **[decided]**; the spec template is the `AtomDecl` plus golden test cases and citations.

### 6.3 EVOLVE: time, branches, look-ahead

- **Universe time** is a signed quantity in seconds from the universe's origin, displayed in whatever calendar a pack defines.
- **Epoch stepping**: quiet stretches are advanced analytically (stellar evolution tracks, secular orbital theory, cooling curves). Interesting transitions open a detailed window (a close encounter, a star leaving the main sequence, an engulfment). N-body integration is used only inside windows, with a symplectic fixed-step integrator and deterministic ordering.
- **Honesty about chaos**: beyond a system's predictability horizon a detailed future is "one plausible future" and is labelled so. It is still deterministic and repeatable.
- **Branches**: a look-ahead is a branch of the chronicle from now to a target time. Nothing is committed. "Move system time there" commits the branch (`commit.branch` event). This is the one mechanism behind deep-time sliders, "what happens to this star", fate of a civilisation, and what-if comparisons (control versus changed, from the same checkpoint).
- **Ageing the Solar System** is the acceptance scenario: main sequence brightening, red giant branch, mass loss and orbit expansion, which planets are engulfed, asymptotic giant branch, planetary nebula, white dwarf cooling with its colour over time, and pollution of the white dwarf by infalling debris. Each step must answer "why?".
- Deep-time ageing is a GM function and may be slow on weak hardware; it is cancellable and reports progress in deterministic work units, never by wall clock.

### 6.4 QUERY: feasibility and the green regions

Two read-only question shapes, both from Evolution Lab's feasibility contract:

- **Location-first**: "what could be here?" Given a patch or system, return the distribution of outcomes without resolving it.
- **Outcome-first**: "where could I find this?" Given a wish (calm, enough metals, a K dwarf with a temperate rocky world, a Population III remnant), score regions and return candidates. This is the galaxy map's green regions and the "find me a home for my story" tool.

Rules: queries never mutate anything until the user commits a pick; a committed pick records that it was *selected for* an outcome (so nobody mistakes a hand-picked paradise for a typical system); a search that finds nothing within its budget reports "not found within budget", never "impossible".

### 6.5 EXPLAIN: why

Every derived value and every event can return an explanation bundle:

```
Explanation { subject, value, publisher (pack, flow, atom, version),
              inputs: [{quantity, value, source: authored|resolved|derived|catalogued, explanation_ref}],
              formula_display, validity: in-range|extrapolated|impossible-input,
              causes: EventId[], lenses: { curious, science, engine }, references }
```

Three cumulative lenses over the same facts (Evolution Lab's help-lens idea): **Curious** (no background assumed), **Science** (the mechanism and its assumptions), **Engine** (ids, versions, determinism). Explain mode in the editor, Tars, and Minerva all read this one structure. Text is derived from facts; an LLM may rephrase it but may not add numbers.

### 6.6 SIFT: significance and stories

Sifters are declarative patterns over the chronicle and current facts that surface things worth a GM's attention: "a moon inside the Roche limit within 40 million years", "the only water source between two jump points", "a world whose day equals its year", "this star dies in the campaign's lifetime". Each hit carries its evidence and a significance score computed from persistence, reach, reversibility and rarity (rarity comes from the cloud's own distributions, so "unusual" is measured, not asserted). Game-system and universe packs add sifters. The story finder and Minerva consume hits. Epochs are named after the fact by sifters, never scheduled in advance.

### 6.7 OBSERVE: player data views

This is the knowledge layer: the difference between what is true and what a given audience has been shown.

- Packs give every quantity kind and tag a **disclosure tier** from a small ordered ladder. Default ladder: `obvious` (seen from afar: it exists, rough size, colour), `scanned` (orbit, mass, atmosphere class, temperature, hazards), `surveyed` (interiors, resources, biomes, settlements), `secret` (GM only). Packs may relabel or add rungs.
- A **player data view** is a named filter the GM defines:

```
DataView { id, label,                      e.g. "Long-range sensors", "Scanned", "Imperial survey on file"
           show_tiers: Tier[],
           tags: { mode: "all"|"none"|"listed", list: TagId[] },
           overrides: [{ quantity_kind|tag, show: bool, precision: "exact"|"rounded"|"band"|"hidden" }],
           unknown_style: "omit"|"placeholder" }      placeholder shows "not yet scanned"
```

- A session has a **default** data view. Any address may carry an override, inherited by its children (set it on a planet and its moons follow unless they have their own).
- **The scanned button**: each campaign names one view as "before" and one as "after". The button on any body flips that body between them. That is the whole everyday interaction. Everything else is setup that a game-system pack supplies ready-made.
- Optional refinement, same mechanism: overrides per audience (a party, or one player's screen) rather than for everyone.
- Enforcement is **soft by default** **[decided: hacking your own instance is fine]**: players hold the seed and packs, and the filter is applied when their kernel builds scene descriptions. This keeps bandwidth tiny. `secret`-tier *authored* facts are simply not sent until disclosed, which costs nothing and keeps GM plot notes off player machines.
- Changing a data view is an `observe.grant` event, so "when did the players learn this" is part of the campaign's history, and Minerva can use it.

### 6.8 NAV: astrogation

Kernel service, not a view. Given ship capabilities, origin, destination, departure window and constraints, return a small set of candidate courses with: burn schedule, time, propellant with reserve, peak acceleration, radiation dose, heat load, debris and gravity hazards, communication delays, and a difficulty rating with reasons. Players command the computer ("fastest", "save fuel", "stay hidden behind the planet") and choose; they never fly by hand **[decided]**. "Ship not under power" is the same service asked "what are my ways out from this drifting state?". Courses are events in the chronicle once the GM approves.

---

## 7. Packs

One format, several roles, stacked in a declared order **[decided: layered]**.

```
PackManifest { id (namespaced), version, kind[], requires: [{id, range}], kernel_range,
               provides: { atoms_used, flows, clouds, quantity_kinds, units, tags, tiers, rules,
                           sifters, data_views, screens, skins, catalogues, vocab, tests },
               seed_namespace, licence, content_hash }
```

| Layer (bottom to top) | Holds | Example |
|---|---|---|
| Base physics | quantity kinds, units, core flows, cloud definitions | shipped with SSE4 |
| Physics packs | extra or replacement flows and clouds; may be flagged `fictional` | "Force physics" that creates force worlds; hard-SF drives |
| Universe packs | seed, catalogues, authored places and histories, names | Milky Way with real stars; Star Wars |
| Game-system packs | classification rules, codes, ship rules, data-view ladders, sifters, report templates | Traveller (UWP), generic d20 |
| Presentation packs | screens, skins, vocabulary | "retro CRT bridge", "paper handouts" |
| Suite packs | a whole domain: life, technosphere, galaxy evolution | Evolution Lab exobiology |

Where packs appear in the UI **[decided 18 Nov]**: the stack itself (order, overrides, conflict checks, adding packs) is campaign **setup**, done once and rarely revisited; it does not sit on working screens. Working screens only *select from* what is stacked: the screen composer has a small "this screen draws on" picker for themes, codes and names, and lenses pick thematic data the same way. Physics is the campaign's and is never chosen per screen.

Stacking rules: later layers override earlier ones by id; an override must be explicit (`replaces: id@range`), never accidental; the resolved stack is validated so each quantity kind has exactly one publisher; every value remembers which pack published it; a pack flagged `fictional` marks everything downstream of it with an evidence label so real and invented physics are never confused in explain mode. Packs are declarative data only. They never contain executable JavaScript.

**Classification as rules**: predicates over facts producing tags and codes, with hysteresis where needed (`temperate ocean world`, UWP digits, "garden world"). Tags are always derived from stored quantities; a tag is never the only record of a fact.

**Vocabulary layers**: one fact set, three wordings, chosen per view: **physics** (exact), **plain** (ordinary description), **story** (evocative, may use explicit analogy, never contradicts the facts).

### 7A. The Traveller pack: the first game-system pack **[decided 18 Nov: Traveller first; other systems are left for their communities to build on the same pattern]**

This is also the worked example of what a game-system pack is. It carries over what V3 already does for Traveller and rebuilds it on the pack machinery. I have not read the V3 source: for every item marked **(V3)**, the build session must read V3's tables and behaviour and port them, not reinvent them.

**It is a bundle of five parts**, installed together and stackable separately:

| Part | Kind | Contents |
|---|---|---|
| Rules | game-system pack | UWP and extended codes, trade codes, travel zones, bases, units, calendar, vocabulary, sensors, data-view ladder, lenses, reports, sifters |
| Jump-era physics | physics pack flagged `fictional` | jump drive, reactionless manoeuvre drive, gravitics, fuel rules |
| Presentation | presentation pack | hex map as the default map, symbology for starports, bases and zones, a console theme, handout screens |
| Constructs | library | stock ships, starports and bases as templates |
| Importers and exporters | converter code in `/converters`, not in the pack (packs stay declarative) | UWP lines, sector files, Travellermap |

Published setting data (the official sectors) is **not bundled**. Users fetch it themselves through the importer.

**1. Codes from physics (world to UWP).** Classification rules (section 7) read stored facts and publish: the UWP digits (starport, size, atmosphere, hydrographics, population, government, law, tech), trade codes, travel zone, bases, the PBG digits (population multiplier, belts, gas giants), and allegiance. Size comes from diameter, atmosphere from pressure and composition, hydrographics from ocean cover, population and the social digits from the technosphere facts or GM authoring. Every digit answers "why" through explain mode like any derived value, and digits update when the world changes or ages. Rule tables are data, so an edition is a table set (see licensing below).

**2. Systems from codes (UWP to star system) (V3).** The reverse direction, which V3 users rely on. A UWP line, with PBG and stellar data when present, becomes a set of **pins and constraints** on the system's envelope: size digit to a diameter range, atmosphere digit to a pressure range and a gas-mix class, hydrographics to an ocean fraction band, PBG to the count of belts and gas giants, stellar data to the stars. RESOLVE then builds the rest of the system around them (orbits, the main world's position, moons, other worlds), choosing the most physically consistent arrangement that honours the codes.
- Everything created this way carries the source tag **`traveller-generated`**, and each value keeps its evidence label. Where a code cannot be squared with physics (a tiny world holding a dense atmosphere, a water world outside any plausible zone), the code wins, the value is recorded as `authored-impossible`, and the GM view shows the honesty mark with the reason. Nothing is silently "fixed". **[Alex: tagged Traveller-created, to account for the dodgy physics]**
- A GM can ask for the opposite at any time: "make this physically sound" proposes the smallest changes to the codes that remove the impossible marks, as an Orac change request.
- Random generation by the book (roll a subsector) is the same path with the dice replaced by keyed draws, so a generated subsector is reproducible from its seed.

**3. Hex map as the default map (V3).** The **Hex map** is a view in the core stack (it is code, so it cannot live in a pack); the Traveller pack makes it the default map and configures it: one hex is one parsec, subsector 8 by 10, sector 32 by 40, hex numbering in the usual four-digit form, world glyphs for starport class, gas giant, bases, zone rings and allegiance borders, jump and trade routes, and jump-range highlighting from the selected ship. Other packs can use the same view with different hex sizes and glyphs. It follows symbology (8.4A) and lenses (8.6) like any view.
- Flat maps and a 3D galaxy: an imported hex position is an authored 2D position. The pack places each system on a plane with a small deterministic offset in the third axis so the Sky and 3D views still work, tagged as authored. A campaign can instead project a real 3D patch onto hexes for a "Traveller-style map of a physical region".

**4. Importers and exporters (V3).** UWP lines pasted or typed; sector and subsector files in the common community formats (legacy SEC, tab-delimited T5 second survey columns, sector metadata XML); and the **Travellermap importer (V3's importer agent)**: pick a sector or subsector by name, fetch it through that site's public API on the user's request, map it through part 2, and report what was imported, what was impossible and what was guessed. Orac assists only with irregular lines and never invents data silently. Export writes the same formats from any campaign, including poster-ready data for a subsector. Import is a set of authored events, so it can be undone and replayed.

**5. Gas mixes (V3).** Named atmosphere presets as composition and pressure envelopes, one or more per atmosphere code: standard, thin, dense, very thin, trace, each tainted variant with a named taint (low oxygen, high oxygen, pollutants, biologicals, sulphur compounds, radioactivity, particulates), exotic, corrosive, insidious, and the unusual cases. Each lists what a traveller needs (filter mask, compressor, vacc suit, protective suit) as a derived tag. They are ordinary pack data that the atmosphere flows consume, so a preset is a starting point the physics then carries forward. Port V3's list.

**6. Sensors and what players can know (V3).** Range bands as distances, sensor kinds (visual, thermal, electromagnetic, active radar and lidar, neutrino, densitometer and the like, per the rule table), and for each kind and band what it reveals. This plugs straight into player data views: the pack ships a ladder (long-range, scanned, surveyed, detailed survey) and a rule that moves a body up the ladder when a ship with the right sensors is in range for long enough. Charts can draw the range rings. Port V3's ranges.

**7. Ships, drives and travel.** Stock ships and other constructs as templates **(V3 has standard constructs)**: tonnage in displacement tons, manoeuvre rating in g, jump rating, fuel, sensors fitted, crew and passengers, cargo. NAV uses the manoeuvre rating for real transit times inside a system. The fictional physics pack adds: jump of N parsecs taking about a week with the rule's variation drawn deterministically; the 100-diameter limit as a real geometric constraint computed from actual bodies, so stars and gas giants cast jump shadows and the astrogation console shows where a ship can safely jump from and arrive; fuel per jump, gas giant skimming and water refuelling with real orbits and times. Starports by class, highports, naval and scout bases are station templates with services derived from class and tech level.

**8. Units, calendar and words.** Parsecs, displacement tons, g, credits, tech and law levels, the Imperial day-year date form as a calendar definition over the campaign clock, and a vocabulary layer that words reports in the game's terms.

**9. Lenses, reports, sifters.** Lenses: Trade (codes and routes), Political (allegiance), Travel zones, Starports and bases, Jump range. Reports and handouts: the UWP line, a world data sheet, a subsector listing, a library-data entry Minerva writes from the facts, a pre-jump briefing. Sifters: mains (chains of worlds one jump apart), choke points, worlds whose codes hide an interesting contradiction, good refuelling stops, red-zone candidates.

**Licensing and naming [open, Alex to rule].** Traveller is a trademark of Far Future Enterprises and the published sectors are their setting. Safe default: ship the rule tables in the form published under an open licence (the Cepheus Engine reference document), describe the pack as compatible with Traveller under the owner's fair-use policy with its required notice, never bundle official sector data, and treat Travellermap fetches as the user's own request with that site's attribution. Edition differences (Classic, Mongoose, T5) are alternative table sets inside the same pack.

**Stages** are X3a to X3d in the build plan. Evidence in brief: codes round-trip (world to UWP to world keeps every digit); a reference subsector imports with every impossible value tagged and none silently altered; the same seed rolls the same subsector on every device; jump shadows match hand-calculated cases; removing the pack leaves a valid campaign with the physical facts intact.

---

## 8. Views, screens and skins

### 8.1 The view contract

A view is a plug-in that turns data into something seen. One contract for all of them:

```
ViewModule {
  id, version, tiers: ("still"|"gpu-high"|"gpu-basic"|"canvas"|"text")[],
  inputs: TopicQuery[]                 what it subscribes to (addresses, quantity kinds, event kinds, time range)
  params: schema                       what a screen may configure
  intents?: IntentSchema[]             optional things a user can ask for (never direct mutation)
  mount(el, ctx): { update(scene), dispose() }
}
```

- Views receive **scene descriptions**: plain serialisable data built by the kernel after the data-view filter. A view never reads kernel state directly and never computes physics.
- Most views are passive. Interactive ones send **intents** ("plot this course", "request scan of Tarn", "vote for course B"). Intents go to the GM, who approves, edits or declines. Approval becomes an event. This is how controls are handed to players safely **[decided]**.
- Adding a view must not require touching the kernel. This is the extensibility test for the architecture.

### 8.2 The finite view stack (first release)

| View | Shows |
|---|---|
| Chart | 2D orbital chart, any tier, cartographic |
| System 3D | bodies, orbits, routes, hazards in space |
| Body | globe with layers: surface, biomes, plates, cities, clouds, aerostats |
| Cutaway | interior and atmosphere shells to scale or log depth |
| Sky | the view out of the window from any place and time, annotated or photoreal |
| Map | tier maps: neighbourhood, cluster, galaxy with green regions, filament foam |
| Timeline | history and futures, events, epochs, branches |
| Route | astrogation console: candidate courses, hazards, reserves, crew votes |
| Table | sortable facts |
| Card | one subject's key facts |
| Readout | single value, gauge or status light |
| Text | prose from vocabulary layers and Minerva |
| Graph | relationship lenses (dependency, trade, later ancestry and food webs) |
| Surface | terrain from orbit to eye height, landings, walking; many styles (see surface design) |
| Media | GM-supplied images and handouts |
| Console | command line to the ship's computer |

### 8.3 Screens

A **screen** is a declarative layout document that composes view instances. Reports are screens. Player displays are screens. Orac writes screens; nobody writes code.

```
ScreenDoc { id, label, audience: "gm"|"players"|"player:<id>",
            layout: grid|flow|stack|page,          page = printable report
            items: [{ view: id, params, bind: TopicQuery, area }],
            data_view?: DataViewId,                defaults to the session's
            skin: SkinId, vocab: "physics"|"plain"|"story" }
```

### 8.4 Skins

A skin (users will call it a theme) is tokens (colour, type, spacing, line weights, corner and frame shapes, sound cues) plus an optional chain of GPU post-process filters (scanlines and phosphor glow, hologram, paper grain, bloom). V3's simple skin colouring survives as the token part **[decided]**. Skins never change data.

Player views are expected to be themed per science-fiction universe far more often than they use the chart look **[decided]**: a retro green-screen terminal for an Aliens-style game, a panel-and-pill starship interface for a Star Trek-style game, clean high tech, a friendly guidebook for a Hitchhiker's-style game. So a theme must be able to restyle every view in the stack without the view knowing: views draw only with tokens and named frame parts, and the theme supplies them. A theme also sets the default vocabulary layer and the default screen layouts for its universe, so picking "green phosphor" gives a whole coherent bridge, not just colours. Themes live in presentation packs and universe packs. Shipped themes are original designs in a genre's spirit with generic names; they do not copy a franchise's actual artwork, fonts or logos, and community packs carry their own responsibility for that. The chart look is offered to players as one theme among the others.

### 8.4A Symbology: how facts become marks, separate from the theme **[decided 18 Nov]**

Alex's point: the chart look is more than a skin, because it says what a planet *is* by shape and shading, where V3 says it by colour. Those are two different **encoding grammars**, and a theme should be able to sit on top of either. So what users call a skin is two documents, chosen independently:

- **Symbology**: which fact drives which visual channel. This is the grammar.
- **Theme** (8.4): tokens, frames and filters. This is the dress.

```
Symbology { id, name, inherits?: SymbologyId,
            bindings: [{ applies_to: object kind (star, planet, moon, belt, station, route, region ...),
                         channel: "shape"|"fill_pattern"|"colour"|"size"|"outline"|"ring"|"glyph"|"label_style",
                         from: quantity or tag (planet class, temperature, habitability, allegiance, survey state ...),
                         scale: categorical table | ordered ramp | thresholds,
                         fallback: value }],
            legend: auto | hand-ordered }
```

Two base symbologies ship, and everything else inherits from one or mixes them:

| Symbology | Planet class shown by | Feels like | Notes |
|---|---|---|---|
| **Diagrammatic** | shape and shading (hatching, stipple, banding, solid), in one or two inks | the chart room | default for the GM chart view; works in one colour, so it suits print, e-ink, green phosphor and colour-blind users |
| **Colour-coded** | colour on plain discs, V3's table | SSE as people know it | default for a "Classic SSE" theme; V3's colour table is imported as the starting palette |

- **Mixing** is per channel, which is why it is easy. Shape by class *and* colour by class (redundant, the most readable). Shape by class with colour freed for something else: habitability, allegiance, survey state, danger. Colour by class with shape for object kind only. A lens (8.6) may name a symbology, so the Political lens can keep shapes and recolour by polity.
- **The colour-coding screen from V3 is reused** as the editor for the colour channel: a table of classes with swatches, edited in place, saved to the symbology document. The same pattern gives a shape-and-shading table for the diagrammatic channel. Read V3's screen and colour table from the V3 source; do not reinvent the classes or colours.
- **Themes declare what they can carry**: `supports: { colours: n | "full", patterns: bool, shapes: bool }`. A one-colour theme (green phosphor) collapses the colour channel to brightness steps or patterns by a stated rule, so a colour-coded symbology still reads there. A theme may also offer its own palette for the colour channel, used when the symbology says "theme palette".
- **Legend** is generated from the active symbology, so any mix explains itself. It is a view part that can be placed on any screen.
- Views never hard-code an encoding: a view asks "mark for this object" and gets shape, fill, colour, size back. Symbology is presentation only: changing it never changes the state hash. It follows 8.9: full schema for Orac, a few promoted controls for people (base grammar, "colour means", edit colours).
- `Screen.skin` becomes `theme: ThemeId, symbology: SymbologyId`. Presentation packs carry both. A V3 import maps a campaign's custom colours into a colour-coded symbology.

### 8.5 The GM interface **[decided: chart room, lean in]**

The GM view is fixed in structure and comes in two modes: **Chart room** (light, default) and **Night chart** (dark, the traditional mode with the same structure). Design language: paper ground, ink lines, a survey grid, serif italic place names, monospaced values, one accent colour for the selection and for anything authored, hatching for uncertainty and unresolved clouds, stamped labels for evidence (`RESOLVED`, `AUTHORED`, `IMPOSSIBLE`, `ONE PLAUSIBLE FUTURE`). The metaphor is exact: an unresolved region is unsurveyed territory on the chart, resolving is surveying, and an edit is a correction inked onto the map with a date. The same chart grammar is used at every tier, so rewriting a galaxy's history is the same few gestures as moving a planet: pick the place, pick the time on the ruler, ink the change.

Editor rules: edit in place on the chart; cards pin to bodies; values are typed where they are shown; the time ruler is always present and doubles as the history; every number has a "why?" one tap away; no modal wizards; full keyboard operation; phone layout is the same card and ruler without the wide chart. Must not look like generic AI-generated dashboard UI.

### 8.6 Lenses: what the GM sees, for the job in hand **[decided 17 Nov: customisable, with defaults, saved and reused]**

How much is shown on the GM view depends on what the GM is doing. Building a map wants everything; running a session wants a little, fast; a political question wants borders and allegiances and almost no physics. A **lens** is a saved preset that decides this. It is one mechanism shared with player data views, not a second system: a player data view (6.7) is the "what data" part of a lens, so every player data view is automatically available as a GM lens ("see it as the players see it", including each body's scanned state).

```
Lens { id, label, icon,
       data: DataView | DataViewId,          which tiers, tags and quantity kinds show, and at what precision
       cards: [{ subject_kind, fields: [quantity_kind | tag | relation], density: "line"|"compact"|"full" }],
                                             what an object's card shows when it comes up, and in what order
       layers: [LayerId],                    what is drawn on the chart: orbits, zones, hazards, routes, borders, trade, life, unsurveyed hatching, labels
       colour_by?: quantity_kind | tag,      what the one accent colour and hatching encode in this lens
       tools: [ToolId],                      which editing and planning tools are on the bar
       view_as?: audience }                  set on "as players see it" lenses
```

- **Shipped defaults**: *Basic* (name, class, a handful of headline facts; for newcomers and quick looks), *Advanced* (everything, all layers), *Building* (rich data, all editing tools, unsurveyed hatching and evidence stamps prominent), *Running* (refined: what matters at the table now, player knowledge marked, routes and hazards, no editing clutter), plus one *As players see it* lens per player data view.
- **Thematic lenses** come from packs or the GM: *Political* (polities, borders, allegiance, population, ports), *Trade*, *Hazards*, *Life*, *Resources*. The same map, a different question. Thematic data are ordinary facts and tags (authored or from a universe or game-system pack), so a lens needs no special support beyond naming them.
- **Make your own**: start from any lens, switch fields, layers and tools on or off, save under a name. Orac can build one from a sentence. Lenses are stored in the GM's profile, travel with a universe if the GM chooses, and can be shipped in presentation and game-system packs.
- **Switching** is one tap or one key from anywhere, and never changes data, time or selection. Editing is allowed in any lens; fields a lens hides are untouched, not cleared.
- **Honesty marks** (authored, impossible, fictional) **[decided 18 Nov]**: always shown on the GM view, whatever the lens, including when the GM looks through an "as players see it" lens (drawn there as a GM-only overlay). On player views they can be hidden: they are ordinary disclosure items a player data view may switch off, and are off by default for players, so a GM's magic or invented physics does not announce itself at the table.
- Lenses apply at every tier (system, patch, galaxy, foam) and to cards, tables and the time ruler's event filter alike.
- Scaling: a lens is also the answer to small screens and weak devices. A phone defaults to *Basic* or *Running*; a lens declares the cost class of its heaviest layer so the device edge rule applies.

### 8.7 Seats: more than one view at once **[decided 18 Nov: supported, but never in the way]**

Default stays exactly as simple as V3: one shared player view. A GM who never needs more never sees more. Everything below appears only after the GM chooses "More seats" in session settings (not on the main bar, not one click).

Use cases this must serve: per-player fog of war (a turn-based 4X game where each empire sees only what it has discovered); several screens showing different takes on the same context; crew stations with roles (astrogation, navigation, sensors).

One concept covers all three. A **seat** is a named place at the table:

```
Seat { id, label,                     "Table" (the default, everyone), "Empire: Vesh", "Sensors", "Wall screen"
       audience: AudienceId,          whose knowledge it shows; default "everyone"
       screens: [ScreenDocId],        one or several; each can open in its own window or device
       lens?: LensId,                 optional override of the audience's data view
       may_send: [IntentKind],        what this seat can ask for (a sensors seat may request scans; astrogation may propose courses)
       auto_approve?: [{intent, scope: Address[]}],   what is applied without asking the GM, and where
       sealed: bool }                 see below
Audience { id, label, tag?: TagId }   a knowledge group; "everyone" always exists
```

- **Knowledge is per audience.** Every `observe.grant` event (6.7) names an audience; today's behaviour is the audience "everyone". The Scanned button gains a small "for: everyone ▾" chooser only when more than one audience exists.
- **Fog of war by tag.** An audience can be tied to a tag (an empire). A data-view rule "show what carries my audience's discovery tag" then does the whole job: the GM tags a system as discovered by Vesh (or the game-system pack's rules do it when a Vesh ship arrives) and only Vesh seats see it. No screenshots, no waiting until everything is discovered.
- **Multi-screen** is one seat with several screens, each a window: chart on the wall, astrogation on a tablet.
- **Roles** are seats with a screen and a `may_send` list. Players still command, never pilot.
- **Joining**: each seat has its own join link or code. The default link goes to "Table".
- **Sealed seats.** Ordinary seats use soft filtering (players hold the seed; hacking your own instance is accepted). That is wrong for competitive fog of war, where one player peeking spoils the game for others. A sealed seat receives only finished, filtered scene data from the GM's machine and never the seed. Cost: the GM's machine builds those scenes, so it carries more load, and sealed seats need the GM online. This brings back, for sealed seats only, the strict option that was dropped as a general feature. [recommended; needs Alex's yes]
- **UI rule**: with one seat, the session panel shows one line ("Everyone sees: Bridge approach") and a quiet "More seats" link. With several, a seat list replaces the line. Nothing else in the app changes.

### 8.8 Is the GM interface just another interface built from the same parts?

**Built from the same parts: yes. Freely rebuildable by default: no.** [recommended]

- The GM workspace is assembled from the same view modules, lenses and screen documents as player screens. The GM is the seat whose audience is "all knowledge" and whose intents are applied at once. Every edit, by anyone, is an intent that becomes an authored event; there is no second, private editing path. This keeps the view contract honest (if the GM's own tools need something, players' tools can have it), and it gives co-GMs, an assistant running sensors, or a 4X player allowed to edit only their own empire, all for free through `auto_approve` scopes.
- But the GM workspace ships as a fixed, curated layout (chart room, cards, time ruler, tool bar). Reasons: the editor has to be fast and learnable, documentation and Tars need a known layout to point at, bug reports need a common baseline, and a blank "build your own editor" canvas is exactly the confusing complexity most users do not want. Day-to-day tailoring is done with lenses (8.6), which change content without moving the furniture.
- The door stays open: because the workspace is a screen document underneath, an advanced "duplicate and rearrange workspace" option, extra GM windows on a second monitor, and pack-supplied GM workspaces (a 4X umpire desk) need no new architecture. They are a later, opt-in feature.
- Consequence for builders: editing tools are views that send intents; the flow editor, pack manager and session manager are views too, restricted to seats with GM authority. Nothing in the shell may reach the kernel except through the same client API the player app uses.

### 8.9 Configuration is built for Orac first, people second **[decided 18 Nov]**

Applies to every part of the UI, not only rendering: screens, views, styles, themes, lenses, data views, cards, reports, workspace panels.

- Every UI element is driven by a declarative document with **far more settings than a person would want to face**, each described in a machine-readable schema: name, type, range, default, unit, one-line meaning, cost class. This is deliberate "hanging configuration".
- **Orac does the hard work.** From a sentence it writes or edits these documents and gets the result most of the way there, offering variants side by side. Every change is a validated diff (schema, tier limits, cost) that the GM accepts or declines, as with all change requests (section 13).
- **People tweak.** Each document *promotes* a few settings as plain controls (sliders, toggles, pickers). Everything else stays reachable through Orac or the raw document. The UI never grows a wall of options.
- Documents **inherit** (`extends`), so a family of looks shares a base and a tweak is a small difference.
- None of this can change data, physics or state hashes. It is presentation only.
- For builders: when adding any view or panel, do not hard-code a look. Expose its numbers in the schema with meanings Orac can read, pick three to five to promote, ship sensible defaults. The Surface view's style pipeline (`SSE4-surface-design.md` 7.1) is the reference pattern; "skins" in 8.4 are now theme tokens plus per-view style documents of this kind.

---

## 9. Kernel API and peer-to-peer

### 9.1 Kernel messages (worker boundary; same shapes natively)

```
load_stack(packs[])                      -> validation report, stack hash
open_universe(manifest | new{seed})      -> root envelope summary
set_authored(event)                      -> affected addresses, new state hash
resolve(address, depth)                  -> facts + child envelopes (recorded)
derive(address[], quantity_kinds[])      -> values with validity labels
subscribe(topic_query, data_view)        -> stream of scene deltas
history(address, range)                  -> events
branch(from, to_time, options)           -> branch id, progress, result summary; commit(branch) / discard(branch)
query_feasible(location|outcome, budget) -> candidates, never mutates
plan_route(request)                      -> candidate courses
explain(subject)                         -> explanation bundle
sift(scope)                              -> significance hits
state_hash(scope)                        -> SHA-256
export(scope) / import(document)         -> manifests, V3 import, converters
```

All long calls are cancellable and report progress in work units.

### 9.2 Peer-to-peer

- Player apps and the kernel load from Cloudflare. The GM's machine serves no pages **[decided]**.
- The GM is the single writer. Over the existing P2P data channel the GM sends: the run manifest (seed, pack stack hashes), authored events as they happen, the session clock, data-view state, approved intents, and results of anything players must not or cannot recompute.
- Players run the same kernel and resolve and derive locally for their own views.
- **Determinism classes**: Class A (the Rust kernel: bit-exact everywhere) may be recomputed on any device. Class B (anything not proven bit-exact across devices: a detailed heavy block, an external suite such as today's TypeScript Evolution Lab core, anything LLM-written) is computed once on the GM's machine and travels as recorded chronicle data. Players never recompute Class B.
- Versions are pinned per session: a player whose cached kernel differs is told to reload, never allowed to drift.
- A periodic state hash from the GM lets a player detect divergence; on mismatch the player requests a snapshot rather than guessing.
- GM disconnects: views go read-only on the last snapshot with a clear banner; intents queue is disabled **[decided]**.
- Signalling and optional relay through one Cloudflare Worker. No campaign data is stored server-side except what a user publishes to the explorers site.

---

## 10. Real sky backgrounds

Alex asked whether every image can have the likely real background: charted stars and galaxies plus a deterministic sprinkling up to the density that patch should have. **Yes, and it falls out of the cloud model.**

1. **Charted objects** are real data fetched on demand wherever a campaign sits in the real universe **[decided 18 Nov, widening the earlier ruling: real data applies only in the real Milky Way and its surroundings, but there it is pulled in dynamically rather than only bundled in a pack]**. The Milky Way universe pack declares *which* catalogues and versions apply; the data itself comes as tiles through SSE4's own Worker and R2 cache (`SSE4-surfaces-and-rendering.md` 3.3 is the authority). Practical sources: Gaia DR3 for stars (a magnitude-limited subset, tiled by sky cell and distance so only what is needed is fetched), a bright-star list for names, and a galaxy catalogue such as 2MRS or OpenNGC for the extragalactic sky. Licences are permissive with attribution; confirm each before bundling. The same route brings in known exoplanets with their measured orbits, masses and radii (NASA Exoplanet Archive) and Solar System elevation and imagery (MOLA, LOLA and others). Every measured value is a pin with evidence `catalogued`; anything unmeasured about a real object is resolved around the measured facts and tagged as fill. Catalogue versions are pinned in the run manifest, so every peer and every later replay resolves the same way; moving to a newer release is an explicit GM action that shows what changed. Offline, cached real data is used and the rest falls back to procedural with the edge shown.
2. **Deterministic fill**: every patch already has an envelope with stellar density, mass function, ages and dust. The fill generates only the stars the catalogue is known to miss (fainter than its completeness limit at that distance, or outside its volume). Same no-double-counting rule as any resolve: catalogue entries are pins that claim from the budget first.
3. **Far field**: beyond the range where individual stars matter, the galaxy's own model (disc, bulge, dust lanes) is integrated along sight lines from the observer's position to give the band of the galaxy, and the tier above supplies other galaxies and, at the top, the foam.
4. **Rendering**: baked to a cube map per vantage point and epoch, in a worker or on the GPU, then reused. Inside one system the background does not change measurably; it is rebaked when the vantage moves between systems or time moves far enough for stars to drift. Proper motion over deep time comes free because resolved stars carry velocities.
5. A star seen in the sky is the same object you can travel to: sky points carry addresses, so clicking one in the Sky view can open its chart.

Cost and risk: catalogue tiles are the only large download (tens of megabytes for a naked-eye-plus sky, more for telescope depth; serve from R2, cache locally). Filaments as glossy 3D foam use the same far-field path with simple maths **[decided]**.

**Renderer and borrowed code [decided 18 Nov]**: the renderer is rebuilt on three.js with WebGPU (WebGL2 fallback). Shaders and techniques are ported from OpenSpace (MIT) and Gaia Sky (MPL-2.0); the applications themselves are not ported. Files derived from Gaia Sky stay as separate files under MPL with their headers, alongside the Apache-2.0 code, and both are credited in NOTICE.

---

## 11. Suites, and how Evolution Lab fits

Alex wants to drop in a detailed exobiology evolution suite and a galaxy evolution suite on the same architecture. I read Evolution Lab's architecture, high-level design, implementation direction, concept audit, and the design repository's architecture, contracts and ADR-013. The fit is close, because SSE4's substrate above is deliberately the same shape as Evolution Lab's contracts.

| SSE4 | Evolution Lab equivalent |
|---|---|
| Address tree, tiers | scale-recursive nodes, habitat graph |
| Keyed draws, one master seed | INV-SEED, INV-RANDOM, CTR-RUN named streams |
| Chronicle with three times | INV-CHRONICLE, INV-TIME3, CTR-HISTORY |
| Envelope and resolve, budgets claimed | CTR-ABSTRACTION, JIT materialisation, formation-context envelope (PROBE-007) |
| Branch, commit, checkpoints | CTR-CHECKPOINT, fork manifests |
| Feasibility queries | CTR-FEASIBILITY-QUERY, INV-CONDITIONING |
| Explanation bundle, three lenses | CTR-EXPLANATION-BUNDLE, CTR-HELP-VIEW |
| Sifters, epochs named after the fact | CTR-SIFTING, CTR-MARKER |
| Evidence labels | CTR-EVIDENCE-LABEL |
| Declarative packs, no code | INV-RULEDATA, CTR-RULEPACK |
| Player data views | CTR-SIGNATURE's "what a given observer can detect" |
| Views get plain data | CTR-VIEW, INV-PRESENTATION |

**A suite is**: a manifest; a *requirement profile* (the typed, unit-bearing inputs it needs from the levels above, exactly Evolution Lab's CTR-PROVIDER-REQUIREMENT); what it publishes back (fluxes, events, signatures, tags, view models); its clouds, flows, rules and sifters; its views; and its determinism class.

**The suite socket** (build this in the first release even though no external suite ships):

```
SuiteAdapter {
  manifest, requirement_profile, determinism_class
  provide(address, time_range) <- kernel hands a pinned, content-hashed input dataset (CTR-ENV shape)
  advance(to_boundary)         -> events, fluxes back to the environment, signatures, checkpoints
  resolve / query_feasible / explain / sift   same shapes as the kernel's
}
```

The kernel is the **coordinator**: it owns the master seed, the clock, the next common time boundary, ordering and persistence, and it commits both sides' outputs at each boundary. The physics suite is the **authoritative physics provider**. A life suite owns living state and returns fluxes (it may change an atmosphere; it may not override pressure or photon energy). This matches Evolution Lab's stated ownership, with SSE4's kernel playing the roles that design gave to "System Lab" and "SSE" together. [recommended; Alex should confirm that System Lab's role folds into the SSE4 kernel's physics suite.]

**Tensions and how to settle them:**

1. *Language (superseded by 11A: the life suite is built natively and simplified first).* Evolution Lab's core is TypeScript by its own decision; SSE4's kernel is Rust. Do not port Evolution Lab now. Mount it through the suite socket as Class B: it runs on the GM's machine in its own worker and its results travel as chronicle data. If it later proves itself and needs to run on player devices, port its hot mechanisms to Rust atoms then. Its Svelte views mount in the SSE4 shell as they are.
2. *Licence (superseded by section 14: one permissive licence for both is now recommended).* Evolution Lab is Apache 2.0, SSE is GPL. Apache-2.0 code may be included in a GPL-3.0 work but not in a GPL-2.0-only one. So SSE4 should be GPL-3.0-or-later, and the shared contract schemas should live in a small Apache-2.0 package both projects depend on, so Evolution Lab never has to import GPL code.
3. *"Declarative only" versus Rust atoms.* No conflict: atoms are the fixed, reviewed operator library; everything a user shares is data.
4. *Unproven parts.* Evolution Lab's own audit says computation-saving reduction and JIT consistency are not yet demonstrated. SSE4 inherits the same risks in the galaxy model. Section 12 treats them as experiments with pass/fail evidence, not assumptions.

A galaxy evolution suite is the cosmos suite's simple flows replaced by richer ones, using the same envelopes. An exobiology suite hangs habitats and lineages under regions. A technosphere or civilisation suite continues the same way. None requires a kernel change if the socket is right; that is the test.

### 11A. A working Evolution Lab from the start, and how it grows **[decided: it must work; start simplified, more procedural generation than simulation]**

The life suite is built natively on the SSE4 substrate (Rust atoms, declarative flows and packs), using Evolution Lab's contracts, vocabulary and UI components. Evolution Lab's TypeScript prototype is the reference and test bed, not a dependency. The interfaces below are fixed at stage L0 and do not change as the insides grow; that is what lets a procedural sketch be replaced by simulation one box at a time.

**The interfaces (fixed from L0):**

```
LifeRequirementProfile   what the suite needs per habitat, typed with units:
  medium and phase, solvent identity, temperature and pressure ranges and variability,
  energy gradients (spectral irradiance at the habitat, chemical, tidal, radiogenic),
  material reservoirs, volume/area, persistence (how long conditions have held), connectivity to
  other habitats, radiation and shielding, the chronicle of physical events affecting it
LifeEnvelope             the cloud for a habitat, region or world at an epoch:
  probability life is present, origin time distribution, energy throughput budget, biomass class,
  capability set distribution (which transformations exist), complexity measures,
  fluxes returned to the environment, signature strengths, uncertainty, evidence label
resolve(LifeEnvelope, address) -> guilds/lineages (aggregate populations), their capabilities,
                                  habitat occupancy, relations (ancestry, resource, habitat, capability)
advance(to_boundary)       -> events, fluxes, signatures, updated envelopes, checkpoints
query_feasible / explain / sift   same shapes as the kernel's
```

The physics suite owns physical truth. The life suite may return fluxes that change an atmosphere or surface; it may never override pressure, photon energy or conservation. The kernel coordinates: at each common time boundary it hands the life suite a pinned physical dataset, takes back fluxes and events, and commits both.

**Stages. Each one ships, each one is honest about what it is.**

| Stage | What it does | How | Label shown to users |
|---|---|---|---|
| L0 Sockets | Interfaces above, habitat graph derived from regions and shells, an empty life suite that returns "no life model installed" | kernel work only | none |
| L1 Biosphere sketch | For any world: is there life, since when, how much, what it lives on, what it does to the air and surface, what it looks like from orbit; on resolve, a set of aggregate guilds with capabilities and a dated history | Procedural: energy and material budgets cap what is possible; keyed draws choose within the caps; history events are generated and hung on real physical events in the chronicle (impacts, stellar brightening, ocean loss) | plausible reconstruction |
| L2 Living planet | Life changes the planet and the planet changes life: oxygenation, albedo, biosignatures; signatures feed player data views (a biosignature at Scanned, ecology at Surveyed) | Fluxes exchanged at epoch boundaries through the coordinator; still procedural inside | plausible reconstruction, coupled |
| L3 Simulated windows | Chosen intervals run Evolution Lab's real mechanisms: resource and energy networks, populations, innovation, extinction and recovery | The procedural generator stays as the coarse model; simulation is the fine model used in detailed windows; each swap needs a fidelity contract showing the two agree on named observables | simulated, within stated limits |
| L4 Open evolution | Lineage change, wrapping into higher-order nodes, independent origins meeting, alien chemistry packs | Evolution Lab's roadmap, mechanism by mechanism | per mechanism |
| L5 Technosphere | Culture, civilisation, technology as further packs over the same primitives | same socket | authored or simulated, per pack |

Why this works: the L1 generators are themselves flows made of atoms, so "how likely is life here" is a visible, editable pipeline in the same node editor as climate, and replacing a box with a simulated one is the ordinary heavy-block mechanism. Nothing downstream (views, data views, sifters, Minerva, the galaxy map's green regions) can tell which stage produced a fact except by its evidence label.

Evolution Lab the product becomes a workbench shell over this same suite: Rule Workshop, Experiment Library, tuning and qualification stay there, and it is where L3 mechanisms are proven before they are promoted into SSE4. Its concept audit's two experiments (model adequacy, then a real computation-saving reduction) are the gate for L3.

The galaxy evolution suite grows the same way: G1 procedural envelopes from simple flows (in the first release), G2 coupled enrichment and feedback between patches, G3 detailed blocks swapped in with fidelity contracts.

**Heavy blocks, in plain terms.** Some physics is too slow for a phone. Two ways to let an expert use a detailed version. *Results as data*: the expert runs the detailed block on their own machine and the pack carries a table of its answers; everyone else looks answers up. Safe, fast, works everywhere, but only covers the inputs that were tabulated. *Plug-in*: the pack carries the detailed block itself as a sandboxed WebAssembly module that runs on every device; it covers any input, but it is code from a stranger, it may be slow on weak devices, and it must be tested for identical results everywhere. Ruling: results as data first; plug-ins later only if someone actually needs them.

---

## 12. What must be proven, and how claims are labelled

Three different promises, never blurred (from Evolution Lab's audit):

- **Exact replay**: same manifest gives the same bits. Required of the whole kernel. Tested by golden hashes on Chrome, Firefox, Safari (desktop and iOS) and native.
- **Bounded agreement**: a cheap model matches a detailed one on named observables within a declared error. Required wherever a simplified flow stands in for a detailed one (epoch stepping versus n-body, a patch summary versus resolving every star).
- **Consistent reconstruction**: detail invented on demand that agrees with everything already recorded but is not claimed to be what a full simulation would have produced. This is what resolve does. Label it as such in the Engine lens.

Each simplification ships with a **fidelity contract**: the user question it serves, the observables, the tolerance, the validity range, and the detailed control it was checked against. If a reduction R and fine model F and coarse model G do not satisfy R(F(x)) ≈ G(R(x)) on the named observables, the reduction is not used there.

Experiments to run early, each returning evidence before more is built on top:

1. **Cross-device determinism**: 10,000 resolves and derives hashed on five browser/OS pairs plus native. Pass: identical hashes.
2. **Order independence**: resolve the same patch's children in random orders and from different zoom paths. Pass: identical facts.
3. **Budget closure**: resolve all children of sample envelopes; totals equal parent budgets exactly, with pins.
4. **Temporal coherence**: settled by the frames and promotion rule in the galaxy design, section 8.3. Test: a promoted system is bit-identical when read at 20 different times in random order, and neighbourhood statistics in any frame match the envelope.
5. **Edit locality**: an authored edit in one patch changes nothing outside its declared causal reach.
6. **Epoch stepping versus n-body** on reference systems over stated horizons, with error reported.
7. **Performance**: resolve cost per tier, a galaxy map feasibility query, an ageing run, on a named low-end phone and a mid laptop, measured in work units and observed time.

---

## 13. Agents

LLMs never compute physics, never sit inside a deterministic path, and their output never enters a state hash.

- **Tars** (support, formerly Zen): read-only, with a dry sense of humour (a humour setting the user can turn down is the obvious joke to reuse). Answers from documentation, explanation bundles and the current universe's facts.
- **Orac** (builder): proposes **change requests**: schema-valid candidate documents (a flow edit, a pack, a screen, a data view, a sifter, a new atom spec as a GitHub issue draft). Each is validated (schema, units, publisher uniqueness, golden tests) and shown as a diff with its effects before the GM accepts. Orac cannot mutate anything directly.
- **Minerva** (storytelling): reads sifter hits, explanation bundles and the disclosure state, and writes narrative in the story vocabulary. Her text is labelled `narrative`. She may not state a number that is not in the facts she was given, and she respects data views (she will not tell players what they have not scanned).

Model access **[decided]**: bring your own key. Reuse the Ovid Next agent system for Orac and Minerva (the coding session should read it in `C:\Development\ovid-next` and lift its provider adapter, key handling and tool-calling pattern rather than reinvent them). Keys stay in the user's browser and are never sent to SSE servers or to other peers. The app must be fully usable with agents switched off.

---

## 14. Rulings closed on 17 Nov (second round)

1. **Licence [decided].** Apache-2.0 for SSE4 and Evolution Lab, with a NOTICE file naming Alex (FrunkQ) that every copy must keep. All the code is Alex's own and credit is what he wants. Outside contributions are accepted under a simple contributor agreement so he keeps the freedom to relicense or sell.
2. **System Lab.** Folded in. SSE4's physics suite is the physics provider; the flow editor and explain mode are what System Lab, Newton and the physics explainer become. There is one kernel, one seed, one clock. Evolution Lab continues as a workbench that hosts the same life suite for experiments (section 11A).
3. **Heavy blocks and device edges [decided].** Results-as-data first. Weak devices do less and can be fed newly calculated data when connected; capable devices compute internally and work better offline. The edge of what a device can do must always be visible to the user with the reason. Mechanism: cost classes and capability tiers, build plan section 2.
4. **Agents.** Bring your own key, Ovid's system reused, support agent is Tars.
5. **Strict data views.** Not needed. Dropped.

---

## 15. Repository layout

```
/crates
  sse-core        ids, addresses, quantities, units, keyed draws, hashing, canonical encoding
  sse-chronicle   events, stores, checkpoints, branches, manifests
  sse-flow        flow documents, validation, scheduler, solve nodes, explain traces
  sse-atoms       the physics operator library (each atom: code, decl, golden tests, citations)
  sse-resolve     envelopes, partitioning, budgets, resolve, feasibility queries
  sse-time        epoch stepping, stellar tracks, secular orbits, n-body windows
  sse-nav         route planning, hazards, drift
  sse-sift        rule predicates, classification, sifters, significance
  sse-observe     disclosure tiers, data views, scene building
  sse-suite       suite socket and coordinator
  sse-kernel      the message API tying the above together
  sse-wasm        wasm-bindgen surface for the worker
  sse-cli         native tool: validate packs, run golden tests, import V3, hash universes
/packages
  contracts       generated TypeScript types and JSON Schemas (Apache-2.0)
  kernel-client   typed worker client
  shell           GM app (Svelte): chart room, flow editor, time, story finder, packs, sessions
  player          player app: screens, views, intents
  views/*         one package per view module
  skins/*         token sets and filter chains
  agents          Tars, Orac, Minerva adapters and prompts
  p2p             session, sync, state hash, snapshot
/packs
  base-physics, milky-way, traveller, default-presentation, examples/*
/workers          signalling/relay, explorers API
/import           v3, space-engine, universe-sandbox-2
/docs             this document, atom spec template, ADRs
/tests            golden universes, cross-device hash harness, fidelity contracts
```

Conventions: stable ids on everything, namespaced; every schema versioned; an ADR for any decision that changes this document; atoms are added by issue, spec, tests, then code.

---

## 16. Build order

Each phase ends with evidence, not a demo. Do not start a phase on top of an unproven one.

**Phase 0. Foundations.** Workspace, CI, `sse-core`, `sse-chronicle`, canonical encoding, keyed draws, worker client, cross-device hash harness. *Evidence*: experiment 1 passes.

**Phase 1. One system, real physics.** `sse-flow`, first atoms and flows for star, orbit, planet bulk, climate, atmosphere, interior shells; authored store; explain; V3 importer; chart room shell with in-place editing and the time ruler (static time). *Evidence*: a V3 system imports and its key derived values match V3 or the difference is explained; every value answers "why?".

**Phase 2. Flow editor and packs.** Node editor over real flows, pack stack with validation and one-publisher rule, classification rules, vocabulary layers, Orac change requests. *Evidence*: a physics pack changes a result with a visible diff and badge; Traveller codes come from a pack, not code.

**Phase 3. Players.** View contract, first views (Chart, System 3D, Card, Table, Text, Readout), screens, skins, P2P session, data views and the scanned button, GM-offline behaviour. *Evidence*: a phone player sees the same facts as the GM by state hash; scanning a planet reveals the right fields; a new view is added without touching the kernel.

**Phase 4. Time and travel.** `sse-time`, branches and look-ahead, the ageing scenario, `sse-nav`, Route and Timeline views, intents and approvals, sifters and the story finder, Minerva. *Evidence*: experiments 5 and 6; the Solar System ages to a white dwarf with every step explained.

**Phase 5. Zoom up.** `sse-resolve` at patch and galaxy tiers, cosmos flows from early universe to now, Map view, feasibility queries and green regions, Sky view with catalogue and fill, Milky Way pack. *Evidence*: experiments 2, 3, 4 and 7.

**Life suite track (runs alongside):** L0 sockets in Phase 1, L1 biosphere sketch in Phase 3 to 4, L2 living planet in Phase 5, L3 onward after Evolution Lab's two experiments report.

**Phase 6. Open up.** Suite socket exercised by a second, external suite, explorers site with V3/V4 labels, converters, pack sharing, signing if wanted.

V3 parity items to check off against the V3 codebase (the coding session should read `C:\Development` on Alex's machine for V3 and Ovid Next; I worked from Alex's capture of V3 and the Discord history, not the V3 source): generation, editing, reports, transit and autopilot (replaced by NAV), player sharing, skins, importers and exporters. Sample V3 saves for the import tests come from the Explorers site **[Alex, 18 Nov]**. Traveller features to carry over are listed in 7A. Alex will set up the V4 subdomain as soon as the repo has code and a documented install and run command, so the README must give install, run and build commands from the first milestone.

---

## 17. Mockup index

On the "SSE4 mockups" canvas. Treat them as intent, not pixel specifications; the Chart room board is the reference for the GM look.

Main editor looks (Chart room is the chosen one; Instrument informs the dark mode; Strata's cutaway becomes the Cutaway view), phone editor, routing by drag, ship not under power, physics pipeline with an Orac change request, deep time across systems, one event in 2D/3D/text, what travels between peers, explain mode, story finder, look out the window, astrogation console, architecture block diagram, zoom-up tiers with the cosmic timeline, genesis from Big Bang to a resolved patch, **player data views with the scanned button**, **Orac assembling a player screen from the view stack**, **the pack stack**, **symbols and themes (the same system diagrammatic, colour-coded and mixed, with the class table editor; the colours on that board are placeholders, use V3's table)**.
