# V3 parity: every V3 feature, and what V4 does with it

Status: K0 draft for Alex's review. Source read: V3.1 (`sse-prod-310`) README feature list, `docs/dev/v4-scope.md`,
`docs/dev/engine-map-carry-forward.md`, `docs/dev/faults-the-old-shape-allowed.md`, `docs/dev/technical-overview.md`,
`docs/dev/ARCHITECTURE.md`, `src/lib/types.ts` (save shape) and `src/lib/io/saveShape.spec.ts`.

Verdicts:

- **Keep**: V4 does the same job, usually through a flow or a pack instead of bespoke code.
- **Replace**: the need stays, the V3 mechanism goes; the V4 mechanism is named.
- **Drop**: not carried forward, with the reason.

The rows name the V4 work package (build plan section 3) that owns each verdict. Nothing here changes V3, which goes to
prod and then sleeps.

## 1. The three lessons V3 itself asks V4 to design against

These come from V3's own `faults-the-old-shape-allowed.md` and are already met by the K1 to K6 kernel. They are the
reason several rows below say Replace rather than Keep.

| V3 lesson | Where V4 meets it |
|---|---|
| Authored and derived state in one object (`DERIVED_FIELDS` drift, a reconciliation overwriting a GM's composition, a parked ship rewriting its node) | Chronicle (K3): authored edits are events; derived values come from flows and live only in the derived cache. A save holds authored events only. |
| A quantity with several producers, or none so consumers infer it (five places inventing a magnetic field, thrust inferred by differencing) | Packs (K5): one publisher per (hook, quantity kind), overrides only through an explicit `replaces`; a second publisher is a load error. |
| A visual path nothing exercises; instruments that report intent | Harness (K1, K6): every op has a golden contract line; meters report measured work units, not intent. View packages carry "a human looked" fixtures. |

V3's other warning, "the generation duplication problem is the main risk": V4 has one generator (K7 resolve and P2 formation), and every import path (P3, P4) feeds the same chronicle as authored or catalogued events.

## 2. Systems, generation and physics

| V3 feature | Verdict | V4 |
|---|---|---|
| Procedural generation from an HR-diagram star pick, four dials (metallicity, disc mass, dynamical history, rarity) with green/amber/red bands | Replace | Probability cloud resolved on look (K7 resolve, P2 formation). The four dials survive as named flow inputs with the same banding; the bands become validity labels (in-range, extrapolated, impossible-input). |
| Generation has no timeline (V3 v4-scope's "one theme": Accrete-style formation with a timeline, the stellar timeline slider G17) | Replace | Time is first-class: formation and ageing are chronicle events; the timeline slider reads the chronicle (T1, T2). G17 is not a separate item, as V3 itself says. |
| Hand placement: a new body binds to the dominant influence under the cursor; the picker offers viable types, every gate a switch | Keep | System chart editor (U2). Gates become flow validity labels; switching one off is an authored edit tagged impossible. |
| Physics engine: makeup (metal/rock/carbon/ice/gas) gives density; temperature, atmosphere and cloud decks, oceans and ice, albedo feedback, dynamo magnetism | Keep | Atom library (P1, then P5 as issues arise), each layer an editable flow published by the base physics pack. |
| Light reaching the ground: spectrum filtered by air and clouds; land, sea, sky and plant colour from it | Keep | Spectral atoms (P1, P5) and the render colour pipeline (V6). |
| Stars classified from position (temperature and size), named in plain words with an example, impossible stars kept and labelled | Keep | Stellar atoms (P1). "Kept and labelled" is the general V4 rule: authored-impossible is a source, never a refusal. |
| 50 to 60 planet types with fingerprint bands, borderline call shows its working, reclassify by hand, pin survives reload | Keep | Classification rules as pack data (K8); a pin is an authored event. |
| Orbital mechanics: Δv to land and ascend, LEO/GEO/Hill, Lagrange points, Roche, rock and soot lines, HZ, frost lines, CO₂ ice line, eclipses, resonance-aware stability | Keep | Orbital and zone atoms (P1). |
| Binaries orbiting a shared centre, Lagrange points as real places | Keep | Barycentre nodes in the address hierarchy (K2) and orbital atoms (P1). |
| Overrides tab: set what physics would not, labelled as an anomaly | Keep | Authored edit with `impossible: true` (already in the kernel protocol). Honesty marks always show on the GM view. |
| Magnetospheres from a pressure balance; aurora oval from the same number | Keep | Atoms (P5) plus a view layer (V6). |
| Weather, lightning, aurorae, stellar flares and dose | Keep | Atoms (P5), surfaced as features on, in and above worlds (P4). |
| Life as a tag; vegetation colour from local light | Replace | Life design through the suite socket (K10) and the biosphere view (U7): metabolism-based, simplified Evolution Lab behind its interfaces. V3's tag stays as an import mapping. |
| Megastructures (seven types) obeying the physics: a swarm dims its star and every world follows; knobs; hard requirement greys a row, amber advises | Keep | Constructs in the base content (X3d) with hooks into the same atoms. The "greys and says why" rule is the validation report. |
| Rule packs, atmospheres, liquids, biospheres, fuels, drives, sensor bands as editable registries | Replace | Layered packs (K5): base, game-system, universe, physics packs that stack at campaign setup. |
| Stars that jet or shed a shell shown in-system | Keep | Views (V6). |
| Toytown factor and visual scaling multiplier | Keep | View settings, never stored as physics. |

## 3. Starmap, scale and time

| V3 feature | Verdict | V4 |
|---|---|---|
| Starmap of many systems in 3D, flat 2D option, z-axis distances | Keep | Galaxy and patch charts (U6) and map views (V9); flat map is a view option. |
| Square, hex, subsector and Traveller grids, snap to grid, Traveller mode | Keep | Grid overlays as view config; Traveller hex map from the Traveller pack (X3c). |
| Your own image pinned behind the stars | Keep | View layer. |
| Real-sky import (NASA Exoplanet Archive, SIMBAD; four presets or centre and radius) | Keep | Real-data service (X5) feeding `catalogued` events. Real catalogue only when building the Milky Way, per Alex. |
| Independent Display and Actual time, data-driven editable calendars tied to one anchor | Keep | Chronicle time is seconds from the origin; calendars are pack data mapping to it (T1, U4). |
| Hover a star to summarise its system | Keep | GM workspace (U2, U6). |
| Links between systems, routes | Keep | Routes as authored relations, drawn on the charts (U6). |
| Bundled example starmaps | Keep | Rebuilt as V4 examples once P2 and X5 land. |

## 4. Ships, travel and autopilot

| V3 feature | Verdict | V4 |
|---|---|---|
| Constructs with mass, crew, power, fuel, engines, cargo, sensors; template library (hard SF, Expanse, Aliens, Mothership) | Keep | Construct records (X3d); templates move into universe packs. |
| 3D hulls (GLB, STL, OBJ) and six public-domain NASA craft | Keep | Assets shown in the 3D views (V6). |
| Transit planner: Lambert transfers, system-wide journeys, interstellar Realistic, Massless, Relativistic and Jump; intercepts; stress graph | Replace | Astrogation console (V8) on NAV (T3): players command the ship's computer, the GM approves. Same solvers as flows, with physics-derived difficulty and hazards. Alex: today's planner is clunky even for GMs. |
| Docking at elevator stations, ring rims and hulls, priced approach | Keep | Construct relations (X3d), priced by the NAV atoms (T3). |
| Autopilot standing orders (Mine, Transport, Patrol, Explore, Escort), per-ship character, Ship's Log | Replace | Intents (V5) scheduled in the chronicle, routed by NAV (T3). The log is a chronicle projection, so it cannot drift from what happened. V3's fault of a parked ship rewriting its own node cannot recur. |
| Drifting under gravity | Keep | Coast integrator as a heavy block with tabulated results on weak devices (T3 drift states). |
| Dogfighting | Drop | Out of scope per Alex; a local RPG matter. |

## 5. Views, players and tables

| V3 feature | Verdict | V4 |
|---|---|---|
| Player views served live as presets (text, document, 2D, 3D), six shipped presets (The Guide, Datapad, Console, CRT Terminal, Holo Table, Projection) | Keep | View contract (V1); player views load code from the web, the GM machine streams data (V4). All six presets return as default player views, themed per universe. |
| Redaction: per-object and per-description visibility, secret tags, the tag disclosure ladder (hidden, anonymous, open) | Replace | GM-definable player data views (K9 disclosure tiers and data views, U5 screen): filters over tags and data with a default, per-planet adjustable. The disclosure ladder carries over as filter semantics. |
| OBS greenscreen Projection following the GM's camera | Keep | A seat (V10) that follows the GM's camera. |
| Interactive 2D visualiser with playback, focus, sensor overlays, five views of each world (to standing on the surface) | Keep | Views (V1, V6); surface view grows into the landing pipeline with height maps from history. |
| Surface view: ground, sky and light; sight distance and lamp reach in metres | Keep | Features on worlds (P4) plus views (V6). |
| Size comparison, as a GM and player view | Keep | View (V1, V6). |
| Interface skins (six, colour-blind-friendly, light, amber terminal), skin editor | Replace | GM view has two looks: chart room (light) and traditional (dark). Player views carry their own themes (V2). |
| Click any unit to swap it, players inherit | Keep | Unit registry (K2) plus view preference. |
| Printable GM and Player reports | Replace | Reports as screens (V3) built from the finite view stack (V1), which Orac can assemble (A2). |
| Mappadux bridge (`/bridge`, `?embed=1`, allowlisted origins) | Keep | Embedding protocol re-specified on the V4 view contract (V1, V4). The exact origin allowlist rule carries over. |
| Foundry and Owlbear shims (wanted, not written in V3) | Keep, later | Same embedding protocol. |
| Low-power switch, memory gauge with crash save | Replace | Capability tiers (K6) decide what runs locally; "just do less" with clear edges. Crash safety comes from the chronicle being append-only. |
| Phone to desktop layout, floating controls folding to a puck | Keep | U1 and every U package; every main UI fits all device types. |

## 6. Data, sharing and tools

| V3 feature | Verdict | V4 |
|---|---|---|
| Save and load systems or a whole starmap as JSON, or `.sse.zip` with assets and `ATTRIBUTIONS.md` | Replace | V4 save is the chronicle plus manifest (K3). The one-way V3 importer (P3) reads both V3 forms; field list in `docs/v3-save-fields.md`. |
| `bundleFormat` integer and `/shipped-content.json` | Keep | V4 has `sse.save/1` with its own format integer, and a shipped-content manifest generated from the packs. |
| A save carries the GM's registries, not the app's (B112, G89) | Keep | Packs: a save names its stack by hash and carries only the GM's own pack. |
| Explorers site: share, browse, open in one click, copy and paste objects with the cartographer credited along the chain | Keep | Explorers site holds V3 and V4 entries with a V3/V4 label (X2). Credit chain is an event field. |
| Cut, copy and paste anywhere, moving whole branches, one undo step | Keep | Editor operations as one authored event batch (U2). |
| Undo and redo, named steps, last twenty survive reload | Keep | Chronicle branches give undo for free (V3's own observation `UI-C7`). |
| GM notes never shown to players | Keep | Authored field, never in any player data view. |
| Traveller import from travellermap.com, UWP entry, UWP block | Replace | Traveller pack (X3a to X3d) with the full World Builder's Handbook treatment, as V3's v4-scope asks. Reference `rtrm/TravellerSystemGenerator` is recorded, not reviewed or licence-checked. |
| Universe Sandbox (`.ubox`) and SpaceEngine (`.sc`) import, import review showing differences | Keep | Converters (T4), importers and exporters, with the review as a difference report. |
| AI descriptions via OpenRouter, style and tags | Replace | Minerva on the bring-your-own-key provider adapter lifted from Ovid (A1, A3). See `docs/ovid-agents.md`. |
| Newton panel "show the working" with a `/physics` reference | Replace | Explanation bundles with curious, science and engine lenses (K4, U3), over every flow. Physics pipelines are the educational centre of V4. |
| Diagnostic bundle | Replace | Tars, the support agent, reads the run manifest and harness report (A4). |
| Custom images for bodies, stars and constructs | Keep | Assets. |

## 7. Dropped outright

| V3 item | Why |
|---|---|
| The dead `generationEngine` field and the removed accrete selector | Already dead in V3; its idea returns as the V4 timeline generator, not as a selector. |
| V2 Points of Interest and Constructs of Interest | Already gone in V3; tags replaced them. The importer maps any leftovers to tags. |
| `defaultMakeup` | V3 says it is wrong at both ends of its range. V4 composition comes from the formation flow; the importer keeps an authored makeup and recomputes the rest. |
| Per-campaign `rulePackId` and `rulePackVersion` pinning by editor side effect | Replaced by the explicit pack stack hash in the manifest. |

## 8. For Alex to confirm

1. The six V3 player presets return as V4 default views. I recommend yes; they are the most loved part of V3.
2. Interface skins reduce to chart room and traditional on the GM side, with skins living only in player views.
   Recommend yes, per your 17 Nov ruling.
3. Autopilot standing orders become scheduled intents and are not ported as V3 code. Recommend yes.
