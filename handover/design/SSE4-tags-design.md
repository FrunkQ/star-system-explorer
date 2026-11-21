# SSE4 tags: what V3's tags did, and what is left of them

Date 21 Nov 2026. Status: **accepted by Alex, 21 Nov** ("Agree with all tag recs after read through"). Every recommendation in section 8 is a ruling. Design only, no code changed. Packages TG1 to TG15 are in the build plan.

Alex's ask: V3 relied a lot on tags as a kind of abstraction layer, with many use cases, "most valid, many probably overplayed". What happens to them in SSE4, with the moving parts pared back.

Sources read for this: the V3 source and docs on AC-ProArt (`star-system-generator`: `src/lib/tags/*`, `docs/tags-guide.md`, `docs/classification-and-tags.md`, `docs/dev/tag-audit.md`, `docs/dev/architecture-physics-tags-visuals.md`, `docs/dev/engine-map.md` entries TAG-1 to TAG-25 and OVR-3, `SystemProcessor.ts`, the construct and autopilot code); four real V3 saves (Local Neighbourhood, its science fiction twin, Procyon, Regina), 1,643 tags in all; the SSE4 designs; and the SSE4 code as it stands on main (`sse-sift`, `sse-observe`, `sse-trigger`, `sse-logic`, `sse-kernel`, `sse-import-v3`, the packs).

## 1. The short answer

V3's rule was "physics decides, writes tags, and everything else reads tags". Tags were the wiring between every part of the program. That gave V3 one language for everything, and it also gave it most of its tag trouble: six kinds of origin, a strip-and-re-emit cycle on every pass, values with small formats hidden inside them, numbers bolted onto tag definitions, and tags that quietly fed back into the physics that wrote them.

SSE4 already has better wiring: facts with evidence, the class atlas, kept values, conditions, data views. So in SSE4 **a tag is only a label**. It is a named mark on an object or a note that people and game rules can read. It carries no value, it never reaches physics, and it is never the way two parts of the engine talk to each other.

Of V3's shipped tag vocabulary (about 30 engine namespaces, 13 categories, more than 300 keys), roughly:

- a little over half **stop being tags** because they described the physical world. SSE4 has that as facts, and the class atlas reads the words off the facts;
- about a third **stay as tags** because they carry game or story meaning (resource labels read off real amounts, purposes, owners, statuses, factions, hooks, anything the GM types);
- the rest become something more exact (an evidence label, a history fact, an orbit relation, a kept value, a pin's stated reason) or are cut.

What goes for good: tag values, tag lifecycles, tags as renderer input, tags as physics input, numbers attached to tags, "system" categories, inherited tags, and the second kind of GM override.

## 2. What V3 tags really did

Thirteen jobs hung off one mechanism. This is the list that matters more than the list of keys.

| # | Job in V3 | Example | Verdict |
|---|---|---|---|
| 1 | Describe the physical state of a world | `geology/plate-tectonics`, `magnetic/dynamo`, `hydrosphere/ocean` | Valid need, wrong tool. Facts and the class atlas. |
| 2 | Carry physics results to the renderer | `structure/cloud-deck = water overcast 0.664`, `feature/polar-vortex = north 6`, `stellar/jets` | Overplayed. The renderer reads facts and recipes. |
| 3 | Let the GM override physics by adding a tag | hand-add `geology/volcanic-tidal` and the volcanoes are drawn, with no physics behind them | Overplayed. One override: pin the fact. |
| 4 | Feed physics (hidden) | radiation reads the `magnetic/*` tag for shielding; `surface/oxidised` sets albedo inside the thermal solve; lightning reads geology tags; a hand-added pigment tag pins the biosphere draw | Cut. This is the "tag switches physics" case. |
| 5 | Record where a number came from | `spin/axis-inferred`, `origin/generated` | Valid. Evidence labels on the value itself. |
| 6 | Record history | `origin/captured`, `origin/migrated`, `spin/tipped` | Valid. History facts from formation, or authored. |
| 7 | Hold structure | `orbit/lagrange = l4` pins a body to another's orbit; `barycenter/auto` | Overplayed. Orbit relations are orbit data. |
| 8 | Explain a GM override to players | `anomaly/alien-technology: magnetosphere, surface temperature` | Valid, liked. Becomes the pin's stated reason. |
| 9 | Seed adventure hooks by rule and dice | `resource/helium-3`, `science/tidal-laboratory`, `intrigue/derelict-rumour` (about 45 rules, 4 categories) | Valid. Rule tags, with the dice fixed (section 5). |
| 10 | Drive ships and autopilot (hidden numbers) | `status/damaged` halves readiness; `owner/pirate` sets tardiness; autopilot mines `resource/*`, refuels on `frontier/*`, FTL from `drive/*`; ships inherit `drive/*` from engines | Half valid. Labels stay, numbers move to kept values, hardware facts replace inherited tags. |
| 11 | Decide what players see | hidden, anonymous, open per tag; whole category hidden | Valid. Disclosure tiers in data views. |
| 12 | Drive looks, filters and map markers | category colours, Find by tag, highlight tray, chips, pins and flags | Valid. Lenses and the finder. Marker styling pared back. |
| 13 | Free GM labels | "Red Syndicate" | Valid. The core of what a V4 tag is. |

Things the V3 code had to carry because of this, none of which SSE4 needs:

- **Six origins** (physics, rule, authored, manual, inherited, derived), decided partly by the tag, partly by its namespace, with a strip-and-emit pair that had to be guarded at 34 call sites (engine map TAG-1, TAG-2).
- **Values with formats inside**: `"<species> <bucket> <coverage>"`, `"north 6"`, an anomaly's list of overrides, a web link in `origin/hub`, prose in `mega/*`.
- **Load-bearing order**: each rule advanced one shared dice stream, so reordering the rule list re-rolled every saved world (TAG-8).
- **"System" categories**: eight categories could not be deleted because code matched their names by hand, and a dangling match failed silently (TAG-12).
- **Two answers to one question**: `tidal/volcanism` against `geology/volcanic-tidal`, `atmosphere/breathable` against `breathable-human` (the V3 tag audit's own findings).
- **Two kinds of GM override**: a tag override (changes the label and the picture) and a value override (changes the number). They could contradict each other on one body.

One finding from the real saves: **no tag in any of the four files carries V3's `origin` field** (it is optional, and V3 works it out from flags and the namespace). SSE4's importer today decides on `origin` alone, so it drops nothing and keeps every V3 tag, derived ones included, in one opaque `v3.tags` fact that nothing reads. Section 6 replaces that.

## 3. Where SSE4 stands today

The designs and the code already hold most of the mechanism. Nothing below is new.

- **Rule tags** (`sse-sift`, package K8): a pack rule is a condition over facts that puts a tag on a subject, with hysteresis and with the facts it read as evidence. Never stored, worked out when asked.
- **Authored tags**: the fact `tag.<name> = true` on any address. Set by the GM, by a template at creation, or by a program's `tag` and `untag` effects (`sse-trigger`). It is an event, so it has a time and a history.
- **Note tags**: plain labels on notes in the record.
- **Readers**: conditions (`has tag`) in rules, triggers, programs and sifters; data views (a tier per tag, a tag filter per view, tag grants for fog of war); lenses (`colour_by` a tag).
- **Physics**: no flow in `base-physics` reads a tag. The door exists in the block language (`world.has_tag`), and nothing closes it for physics yet.

Gaps found:

1. No tag is declared anywhere. The handoff promises `provides.tags` in a pack; no pack has it, so a tag has no label, colour, group or tier, and a misspelt tag in a rule or a template fails silently. This is V3's "system category" problem waiting to come back.
2. An authored `tag.x = false` does not switch off a rule tag, and an authored `true` loses to the rule. So the GM cannot overrule a rule tag.
3. Tags have no "something is here" rung for players (V3's anonymous).
4. The V3 importer keeps tags as an opaque blob (section 2).
5. The word "tag" is also used in the designs for things that are not tags: "tagged authored-impossible" (an honesty mark on a value) and the "source tag `traveller-generated`" (evidence on generated values). These are marks on values. This document calls them **marks** and asks the other designs to do the same, so "tag" means one thing.

## 4. The V4 tag mechanism in one page

**What a tag is.** A name on an address (body, system, region, construct, actor) or on a note. Present or absent. No value, no number, no text inside. If it needs a value it is a fact (physics), a kept value (game) or a note (words).

**Names.** Lowercase with dots, `group.name`, for example `resource.helium-3`, `status.damaged`, `faction.red-syndicate`. The first part is the group. This matches what the Traveller pack already does (`traveller.prep.oxygen`).

**Two sources, no lifecycle.**

| Source | Set by | Stored | Shown as |
|---|---|---|---|
| Derived | a pack rule over facts, optionally with a seeded chance | never, worked out on demand | with its "Why?": the rule and the facts it read |
| Authored | the GM, a template at creation, a program effect, an import | as a dated event | with who set it and when |

There is nothing to strip, re-emit, guard or migrate. V3's six origins collapse to these two.

**The GM always wins, in one way.** An authored `true` forces a tag on; an authored `false` holds a derived tag off. Either shows the authored mark. This changes the label only. To change the world, pin the fact (the existing GM override, with its honesty marks). There is no second kind of override.

**Who may set a tag.**

- The GM: any tag, on anything.
- A pack rule: only tags its pack stack declares as derivable.
- A program or trigger: through the `tag` and `untag` effects, under the existing policy (auto by default, the GM can change it per rule).
- A player: on their own journal notes only. A tag on the shared record needs GM approval, as notes already do.

**What a tag can cause.** By itself, nothing. It can be **read** by:

- conditions, wherever conditions run: rules (so one tag can build on another), triggers, programs, sifters, tag grants;
- data views (which audience sees it) and lenses (colour, highlight, filter);
- the finder ("nearest world carrying `fuel.skimmable`").

**What a tag can never cause.**

- It is never an input to physics, RESOLVE, formation, weather, life or the class atlas's physical schemes. A physics flow that reads a tag is refused when the pack stack loads.
- It never changes how a world is drawn as a physical thing. Renderers read facts and recipes. A tag may add a marker, a colour or a highlight through a lens, never a cloud deck.
- It carries no hidden numbers. Readiness, discipline, richness and rates are kept values or facts in their own right.

The one bridge from game to physics stays the one the record design already has: a program may react to a tag with an explicit `set_pin`, which asks the GM by default and is marked. So "the cursed world has no magnetic field" is possible, visible, and never accidental.

**How packs declare tags.** One block per pack, `provides.other.tags`:

```
groups: [{ id: "status", label: "Status", about, colour?, applies_to: ["construct"], exclusive: false }]
tags:   [{ id: "status.damaged", label: "Damaged", about, group: "status",
           source: "authored" | "derived" | "either", tier: "scanned", colour? }]
```

- `applies_to` limits where the tag is offered (not where it may exist). `exclusive` means at most one of the group per object (V3's "single").
- `tier` is the default disclosure tier. The data-view machinery is unchanged; this block is simply where the tier is written.
- Every tag named by a rule, template, trigger, program, view, lens or code sheet must be declared somewhere in the stack. If not, the stack fails to load and names the pack and the reference. That replaces V3's undeletable "system" categories with an error you can see.
- The campaign has its own small pack for the GM's tags and groups, in the same shape. A tag typed freely is added to it on first use. So "my factions" is data the GM can export, share and stack like any pack, which is what V3's category files were.

**Visibility.** There is no separate tag visibility mechanism (Alex, 21 Nov: it is just part of the usual data view filters). A tag sits at a disclosure tier exactly as a fact does, and a data view shows or hides it by tier, by its tag filter and by its per-item overrides, all of which exist today. A tag's declaration only supplies its default tier. Two small things bring tags level with facts inside that same machinery: a per-object override, so one use of a tag can sit at a different tier (the syndicate secretly owns this station and openly owns that one), and the placeholder facts already get one rung above the viewer, which for tags is a single neutral "something undisclosed here" marker per object, however many tags are hidden, because three markers would tell players three things are hidden. Tags further away, and GM-only tags, show nothing. Scenes are built in one place, so there is one point of redaction, as V3 learned to need (TAG-9).

**What was pared away, in numbers.** V3: 6 origins, 3 spellings of secrecy, 2 override kinds, 5 value formats, 8 protected categories, 3 number fields on tag definitions, 1 ordered dice stream. V4: 2 sources, 1 tier, 1 override, 0 value formats, 0 protected categories, 0 number fields, no ordering.

## 5. Every V3 tag and its V4 home

"Dropped on import" below means SSE4 works the same thing out from the imported physics; nothing authored is lost. A **hand-added** tag in any of these namespaces is different and is handled by section 6.3.

### 5.1 Physical description: facts and the class atlas

| V3 tags | What V3 hung off them | V4 home |
|---|---|---|
| `geology/*` (plate-tectonics, stagnant-lid, episodic, plutonic, cryovolcanic, volcanic-tidal, inactive), `tidal/*`, `activity/*` | drew volcanoes and vents; lightning read them; hook rules read them | Process strengths in the surface suite (processes, not classes). The atlas shows the word as a modifier. |
| `magnetic/*` (dynamo, induced, tenuous, anomalous, unshielded) | **radiation dose read the tag** | Field strength as a fact; the dose reads the fact. |
| `structure/*` (icy-shell, subsurface-ocean, cloud-deck, supercritical-envelope), `hydrosphere/*`, `volatiles/ices`, `climate/*` | renderer drew decks and ice from the tag value; phase logic | Interior layers, surface phase and weather facts. Cloud decks are weather-suite output. |
| `weather/*` (precipitation, lightning, dust-storms, monsoon), `visibility/*`, `aurora/*` | drawn; aurora tag value carried a hidden 0 to 1 strength | Weather and light suite facts. |
| `surface/age`, `surface/irradiation`, `surface/oxidised` | **oxidised set albedo inside the thermal solve** | Surface facts; oxidation is a quantity the albedo block reads. |
| `shape/*`, `ring/*`, `resonance/*`, `thermal/self-luminous` | ring tiers, flattening | Facts; atlas modifiers ("ringed", "oblate"). |
| `orbit/tidally-locked`, `locked-star`, `locked-planet`, `spin-orbit-resonance` | spin handling | Spin state facts. |
| `stability/*`, `fate/*` | warnings | Stability facts with their reason (already `orbit.stability_why`). |
| `hazard/radiation`, `hazard/orbital-radiation`, `hazard/flaring` | finder, warnings | The `hazards` facts. Travel-zone style warnings are rule tags a game pack derives from them. |
| `stellar/activity`, `jets`, `shedding`, `dimmed`, `ir-excess`, `anomalous` | **the star's picture was drawn from these tags** | Star facts and the light description. |
| `habitability/*`, `biodiversity/pigment`, `land-cover`, `settled`, `ecumenopolis` | body colour on maps; a hand-set pigment pinned the draw | Life suite facts. A chosen pigment is a GM pin on that fact. |
| `flight/ascent`, `flight/fuel-use` | planner hints | NAV facts (delta-v, station-keeping cost). |
| Flat gas tags (`inert`, `greenhouse`, `reducing`, `toxic-human`, `breathable-human`, `corrosive`, `crushing-atmosphere` and about 15 more) | hook rules read some | Atmosphere facts. Words a game needs ("breathable", "tainted") are rule tags in a game pack, as the Traveller pack already does. |
| `feature/polar-vortex` | drawn; seeded, not derived | Weather suite (giants) output; a GM who wants a hexagon pins it. |
| `mega/shadowed-by`, `mega/eclipsed` | none beyond display | Light facts (occlusion). |
| Legacy `planet/*` class tags and V1 display-name tags | none | Cut. The class comes from the atlas. |

The line between an atlas modifier and a rule tag, so nobody has to guess: **if an astronomer would say it, it is an atlas word read off the physics. If a referee or a ship's captain would say it, it is a rule tag in a game pack.** "Tidally locked" is the first. "Amber zone" is the second.

### 5.2 Provenance, history and structure: more exact homes

| V3 tags | V4 home |
|---|---|
| `spin/axis-inferred`, `spin/period-inferred` | The evidence label on that value ("resolved", not "authored" or "catalogued"). Typing a real value changes the label by itself, which V3 had to do by hand. |
| `origin/generated` | Evidence label "resolved" on the body. |
| `origin/captured`, `origin/migrated`, `spin/tipped` | History facts from formation; on import, authored history facts. |
| `orbit/retrograde`, `orbit/double` | Nothing to store: read off the orbit and the hierarchy. |
| `orbit/lagrange = l1..l5` | An orbit relation ("rides L4 of ...") in the orbit data. |
| `barycenter/auto` | Cut. The hierarchy says it. |
| `origin/hub` (value: a link), `origin/hub-route-stood-down` | A note on the object with the link; the credit stays in the campaign's attributions. |
| `mega/*` placement steers (prose in the value) | A note on the construct, written at placement. |
| `traveller/satellite-main-world` and other `traveller/*` | Facts under the Traveller pack's own prefix. |
| `anomaly/*` (12 seeds plus the GM's own) | The **stated reason on a pin**. The vocabulary is a declared tag group `anomaly`; the pin carries the reason; players read "Alien technology: magnetosphere, surface temperature" composed from the reason and the pinned quantities. A pin with no reason shows players nothing, as in V3. |

### 5.3 Game and story meaning: these stay tags

| V3 tags | What V3 hung off them | V4 home |
|---|---|---|
| `resource/*` on worlds (16 raw materials) | autopilot Mine and Explore searched for them; the value was richness | Real amounts from physics are the truth; `resource.*` is a derived quick-reference label read off them, shown to players only after a survey. See 5.5. |
| `frontier/*` (fuel-depot, gas-skimming, life-support, aerobraking, gravity-assist, waystation, ice-mining, prime-claim) | autopilot refuelling | Derived tags `access.*`, same rules over V4 facts. |
| `science/*` (14 hooks) | none | Mostly **sifters** in the story finder, which is the same idea with evidence. A few stay as derived tags where a game rule needs to read them. |
| `intrigue/*` (anomalous-signal, derelict-rumour, uncharted-feature, legend) | none, pure dice, off by default | Derived tags with a seeded chance, in an optional hooks pack, off by default. |
| `resource/*` on constructs (cargo, trade goods, 7 finished goods) | none | Authored tags, same ids as the world ones, so one search spans worlds and ships. |
| `purpose/*` (about 45) | autopilot guessed legs from them | Authored tags `purpose.*`. Port capabilities map to the Traveller pack's own tags where that pack is stacked. Programs read them through conditions. |
| `owner/*` (7) | **carried tardiness**, the default for the Discipline slider | Authored tag `owner.*`. Discipline is a kept value on the ship, with the template giving the default. |
| `status/*` (15) | **carried a readiness multiplier**; three were mirrored from the journey | Authored tag `status.*`. Readiness is a kept value. "In transit" and "adrift" are read from the ship's programme state and are not tags. A pack trigger can lower readiness when a status tag appears. |
| `class/*` hull class (25) | template picker | The template the construct was made from. Kept as a tag `hull.*` only for import and filtering. |
| `drive/*` (5) | FTL ability, inherited from fitted engines | The fitted hardware's facts. No inherited tags. |
| `universe/*`, `tech/*`, `disposition/*` | filters | Authored tags (`setting.*`, `tech.*`, `stance.*`). |
| Free GM tags, any GM category | colours, markers, rules | Authored tags in the campaign's own pack. |

### 5.4 Cut outright

- Tag values and every format inside them.
- GM override by tag in a physics namespace (job 3). Pin the fact.
- Inherited tags and the engine and fuel fields that listed them (`drive_tags`, `refuel_tags`). A fuel says what it can be made from as a condition.
- "System" categories.
- Per-view marker shapes (chips, pins, flags, five staff colours, a marker-size dial). A theme draws tag markers one way; highlight colour still comes from the tag. See question 7.
- `AIContext.tags` (prompt hints, never real tags).

### 5.5 Resources: real amounts underneath, the label on top (added after Alex's question, 21 Nov)

Alex asked whether generation should drive these (an iron-rich asteroid is iron-rich because of how it formed), whether classifying them is the right home because a game may hide classes, whether they should appear only after a survey, or whether to drop them for real values now that something could actually be mined out.

It is not either-or. The two do different jobs, and one is read off the other.

1. **The truth is physics.** Formation already gives every body its bulk makeup (`makeup.metal`, `makeup.rock`, `makeup.ice`, `makeup.carbon`, `makeup.gas`) and mass, and the air and sea give their own species. A resource is an **inventory**: how much of a thing this body holds, in kilograms, worked out from those facts by small cited blocks (iron from the metal fraction, helium-3 from a giant's envelope or from solar wind implanted in old airless regolith, water ice from the ice fraction and surface phase). No dice anywhere. Strata and rock types (S9) later say where in the body it sits and how hard it is to reach; the inventory does not wait for them.
2. **The astronomer's word is the class atlas.** "Metal-rich asteroid" is a class read off the makeup. It needs nothing new.
3. **The miner's word is a derived tag.** `resource.iron`, `resource.water-ice`, `resource.helium-3` are rule tags over the inventory: "there is enough here, at a grade this game counts as worth working". The threshold belongs to the game pack, because what counts as ore depends on the setting's technology. This is the quick-reference pop-up a mining game wants, and the reason to keep it is exactly that: a label you can scan a map for, filter by and hand to an autopilot, with the real numbers one click behind it in "Why?".
4. **Survey gates what players see, with no new mechanism.** The `resources` facts and the `resource` tag group sit at the `surveyed` tier, which is where the Traveller pack already puts resources. Before a survey a crew sees the class at most. A ship's survey is a trigger that grants the surveyed view, and the labels and estimates appear. Data views already support precision, so a quick survey can show a band ("iron: rich") and an assay the figure. A game can hide the whole group or the classes, as Alex notes.
5. **Mining really removes it.** What has been extracted is a kept value on the body, per resource, written by the mining program. Remaining is inventory minus extracted, and the label's rule reads the remainder, so **a mined-out rock loses its label by itself**. Game activity stays in the record and does not disturb physics. Only when extraction is large against the body itself (an asteroid mostly hauled away) does a program use the existing explicit pin on its mass, asked and marked as always.
6. **Ships read both.** A survey or mining program finds candidates by the label, as V3's autopilot did, then reads the remaining amount and grade to choose and to fill the hold. V3's "richness" was a number hidden in a tag value; here it is a real quantity.
7. **The GM still wins.** Forcing `resource.iron` on or holding it off works like any tag (section 4), for "the claim is officially worked out" or "rumoured gold". To make the gold real, pin the inventory.

So the answer to "do we ditch them for real values" is: the real values replace the dice and the hidden number, and the label stays as the thin, hideable, survey-gated summary on top. Resource tags are no longer something anyone authors or rolls in normal play.

Consequences elsewhere in this document: resource rules get no chance (question 1 stands, and now applies to none of the resource group); question 2 is rewritten; package TG8 changes and TG11 is added; on import, V3's rule-set `resource/*` tags on worlds are the one hook family where V4's physics should win (question 3).

### 5.6 Made and imported goods, and placeholders for the economy pack (Alex, 21 Nov: a 4.1 feature, after 4.0)

Alex: some resources are "food", prepacked and perhaps imported, so there are unnatural resources that need the same mechanism; and an economy layer, as a separate pack, is a likely future block, because the universe already produces the uneven distribution that drives economies, and people may eventually take part.

**Made goods use the same inventory, with a different source.** A resource is a kind declared by a pack (`iron`, `water-ice`, `packaged-food`, `antimatter`). Each kind may name a physics block that gives its natural amount; a made good names none. What a place holds is then one sum:

```
held = natural amount (physics, zero for made goods) - extracted + stock
```

`stock` is a kept value per resource kind on any address: a body, a settlement, a station, a ship's hold. It is written by the GM, by a template (a depot starts with fuel), or by a program (a refinery turns ore into metal, a freighter unloads). The `resource.*` label reads `held`, so "food available here" works on a station that physics gave nothing, and the label, survey tier, finder and autopilot behave exactly as for natural resources. V3's seven finished goods (provisions, technology, luxuries, pharmaceuticals, antimatter and the rest) land here.

**What is left open for an economy pack, without building it:**

- Resource kinds are pack data, so an economy pack adds goods without touching the kernel.
- Amounts are real and located: natural inventory by body, stock by place, both changing over time in the record.
- Moving goods is a program that lowers stock in one place and raises it in another, so quantity is conserved and every change has a cause.
- The cost of moving them is already physics: NAV gives delta-v, time and fuel between any two places.

That is the whole input an economy needs: where things are, where they are wanted, and what it costs to carry them. A later pack would add production and consumption rates on settlements, prices as kept values worked out from scarcity and carriage, and trade programs for autopilot ships. Players take part through the existing intents (buy, sell, haul), with secrecy and GM approval as the record design already gives. Nothing in the tag design has to change for it; the only thing to protect now is that stock is general (any kind, any address) and transfers conserve.

**Alex's ruling, 21 Nov:** put the placeholders in now and do a high-level economy design once enough is in place; it is a 4.1 feature. The placeholders and where they are built:

| Placeholder | Package |
|---|---|
| Resource kinds declared as pack data, natural block optional | TG11 |
| Natural inventory per body, in kilograms | TG11 |
| `stock` kept kind for any resource on any address | TG11 |
| Transfers that conserve quantity (a program effect pair, lower here and raise there) | TG11 |
| Ship discipline and readiness as kept values, no numbers on tags | TG9 |
| Carrying cost between two places | NAV, already built |
| Reserved names: the kept-kind prefix `economy.` and the tag group `trade` are kept free for the economy pack | TG1 |

**EC0, the high-level economy design**, starts when TG11, TG9, the program packages (J6) and the Traveller trade data (X3) have landed. It is listed in the build plan as track EC and milestone "4.1 Economy".

### 5.7 Universe packs: a setting adds its own detail to worlds that fit (Alex, 21 Nov: in V1)

Alex: in V3, tags "fill in tech detail" in packs, and being able to swap and distribute that is useful. It should be a separate kind of pack that ships in V1: a universe pack for a fictional setting adds that setting's constructs and bases to worlds that fit its parameters, with perhaps a dozen ready on the Explorers site, so any world-spanning science fiction setting can add its detail.

This is the handoff's "universe pack" layer, and almost all of it is assembled from parts that exist. What V3 did with `universe/*`, `tech/*`, `class/*` and `purpose/*` tags plus hand placement becomes:

**What a universe pack holds.** All existing pack contents, no new kinds:

- **templates** for its ships, stations, bases, settlements and factions, with their facts (thrust, sensor ranges, crew), kept values, tags and programs;
- **tag groups** for its vocabulary (factions, allegiances, tech eras, port grades);
- **rules** that label worlds in its terms ("core world", "rim outpost", "fuel stop");
- optionally a **physics pack marked fictional** for its drives and devices, a **lens** (political map), **player view themes**, and **start-sheet lines**.

**The one new part: dressing rules.** A dressing rule says "a world like this gets one of these":

```
{ id, when: condition over facts and tags, place: template id,
  where: "low-orbit" | "high-orbit" | "surface" | "L4" | "belt" | ..., count: 1, chance?: 0.3,
  tags?: [...], faction?: actor id, explain: "a gas giant with skimmable hydrogen and a port within 2 AU gets a fuel depot" }
```

It is a rule tag whose answer is a thing instead of a label, and it obeys the same laws:

- **Deterministic.** The condition reads facts; any chance is drawn from the universe seed, the address and the rule id (TG4). The same campaign with the same pack stack dresses the same worlds on every device.
- **Explained.** Each placed object's "Why?" names the rule and the facts it read.
- **Lazy.** Worked out when a system is looked at, like everything else. A galaxy is never dressed in advance.
- **Proposed, then adopted.** Until adopted, a placement is a *candidate*: drawn as a ghost marker on the GM's chart ("would hold a naval depot"), stored nowhere, and it follows the facts if the world changes. Adopting makes it a real authored object from the template, which the GM can then rename, move, edit or delete like anything else. Adoption is per object, per system, per region, or automatic on first visit (a campaign switch, off by default, for GMs who want a setting to simply be there).
- **The GM wins.** Hold a candidate off (it never comes back for that world), or place any template by hand anywhere. Same force and hold-off as tags.
- **Never physics.** Dressing reads the world and adds authored objects. It changes no fact. A pack that wants a terraformed world uses the explicit pin, asked and marked.

**Swapping.** Because candidates are derived from the stack, replacing one universe pack with another changes every un-adopted candidate at once. Adopted objects stay (they are the campaign's now, and each remembers the pack and rule that made it); the swap offers to retire the ones the GM never touched.

**Players.** Dressed objects are ordinary objects, so data views, survey tiers and fog of war apply with nothing added.

**Packages.**

| Id | Package | Owner thread | Needs | Acceptance checks |
|---|---|---|---|---|
| TG12 | Dressing rules: `provides.other.dressing`, a candidates reply for a system, adopt and hold-off operations, the pack-and-rule mark on adopted objects, swap handling | Build kernel resolve and rules | TG1, TG4, J5 templates | a fixture pack puts a depot candidate on every giant with a port within 2 AU and nowhere else; same seed, same candidates on two devices and after shuffling the rule list; adopt, edit, then remove the pack: the object stays; hold-off survives a reload; adopting changes no resolved fact and no golden hash; a candidate's "Why?" lists the facts read |
| TG13 | Two shipped universe packs written for SSE4: a generic near-future hard science fiction setting and an original space opera setting, each with templates, factions, labelling rules, dressing rules and a political lens | Build Traveller pack (it already owns stock ships and bases; its own bases move onto dressing rules) | TG12 | each pack dresses the Sol and Kestrel fixtures with at least one port, one base and one faction presence, all explained; stacking either over the Traveller pack loads clean; every tag they use is declared |
| TG14 | Dressing in the chart room: ghost markers, adopt and hold off on the object card, "dress this system" and the first-visit switch under Workshop | Build chart room shell | TG12, TG10 | screenshots at desktop and phone in both GM modes; candidates never appear in a player view before adoption |
| TG15 | Pack shelves on the Explorers site | Integrate Explorers site with SSE4 | TG1, TG12 | see below |

**What the Explorers thread needs for pack shelves (TG15).**

- A new item type, **pack**, beside maps and systems, with a kind (universe, game-system, physics, tag set).
- The listing shows what the pack manifest says: id, version, content hash, what it provides (so many templates, factions, dressing rules, a fictional-physics flag), what it needs (other packs and versions), and its licence.
- A preview: the pack applied to one stock system, as a static picture or a live read-only view.
- "Add to my campaign" hands SSE4 the pack by id and hash (the existing paste and import route), SSE4 validates the stack before accepting, and a refused pack says why.
- The moderation ledger keyed on content hashes already exists; packs use it unchanged.
- The four fan-pack points and the "unofficial fan work" line (next paragraph) are carried by the site's **existing** credits, attribution and statement fields and its report route (Alex, 21 Nov: that mechanism already exists, reuse it). No new form or ledger is designed for packs.

**Rights: relaxed after Alex's reply, 21 Nov.** Alex's point stands: a rule saying a kind of orbital base is found around 20% of certain gas giants is not something anyone owns, the tool is free and educational, and it sends players toward the games and books of the settings they love. Rules, statistics and plain descriptions are not the exposure. What is: logos and trademarks used as branding, art and renders, copied text, and to a lesser degree a setting's distinctive names. The realistic worst case is a takedown request, and each owner's fan policy differs. This is practical guidance, not legal advice.

So **setting-specific fan packs are allowed on the Explorers site** when they:

- carry rules, numbers and plain descriptive names only, with no logos, no art or ship renders, and no copied text;
- name the setting only to say what the pack is for, with an "unofficial fan work, not affiliated with or endorsed by the owner" line shown on the listing and in the pack;
- are free, always (see paid packs below);
- have a report link and a simple takedown route through the existing moderation ledger.

**Paid packs (added 21 Nov, when Alex asked for a charging mechanism on the Explorers site, designed in the thread "Design paid packs on Explorers").** The site will take money, so the line most fan policies draw, non-commercial, now matters and is drawn here:

- **A named-setting fan pack can never be paid.** No price, no tip jar on the listing, no paid tier that unlocks it, no bundling inside a paid pack. The site refuses a price on any pack marked as fan work for a named setting.
- **A paid pack must be original or rights-held.** The seller declares that the setting and everything in the pack is their own or licensed to them for sale, and a paid pack found to be a named setting without that right is delisted and refunded.
- Fan packs sit on their own shelf, visibly apart from anything with a price, so the free fan shelf is not read as a shop window for someone else's setting.

The paid-packs thread owns the mechanism; this section only fixes these three rules for it.

The uploader ticks the four fan-pack points. One more thing worth knowing rather than fearing: distinctive names (ships, factions, characters) are the grey part, which is why "plain descriptive names" is in the list.

Ruled by Alex, 21 Nov (question 10): packs built into SSE4 and featured on the front shelf stay original and generic; named-setting packs are hosted as fan-made and unofficial, and we may write a few as examples. One existing item sits under the same guidance: V3's bundled "Local Neighbourhood (Science Fiction)" map names settings from novels, films and games.

## 6. Importing V3 saves

### 6.1 Deciding what a V3 tag was

Saves do not say, so the importer repeats V3's own order:

1. an explicit `origin`, if present;
2. `derived` flag: runtime state;
3. `inherited` flag: from hardware;
4. `manual` or `coi` flag: set by hand;
5. `source` starting `rule:`: set by a hook rule;
6. otherwise the namespace table from V3's `ENGINE_NAMESPACES` (27 physics namespaces, 6 authored ones, exact keys before prefixes);
7. no namespace match and no flags: treated as the GM's own.

### 6.2 What each kind becomes

| V3 kind | Import |
|---|---|
| Physics-derived (5.1) | Dropped and counted. SSE4 derives it. |
| Runtime and inherited (`status/in-transit-*`, `status/adrift`, inherited `drive/*`) | Dropped and counted. |
| Provenance and history (5.2) | Evidence label, history fact, orbit relation or note, per the table. |
| Rule-set hooks (`frontier/*`, `science/*`, `intrigue/*` on worlds) | **Kept as authored tags** under the V4 ids. They are campaign facts by now, and V4's dice would roll differently. See question 3. |
| Rule-set `resource/*` on worlds | V4 derives resources from the imported makeup (5.5). Where V3 had a resource tag and V4's physics finds none, the tag is kept as an authored (forced) label and listed in the report, so nothing the players were told disappears. Where both agree, nothing is stored. |
| Hand-set construct tags | Authored tags under the mapped ids. `owner/*` also writes the ship's discipline kept value from V3's tardiness if the ship set none; hand-set `status/*` also writes readiness (the lowest of the statuses). |
| The GM's own tags and categories (`coiCategories`, saved tag categories, category files) | The campaign pack: groups with their colours, `single` to `exclusive`, `appliesTo` to `applies_to`. Category rules import where their conditions map to V4 facts; the rest are listed in the report. |
| Visibility | no flag: `obvious` (V3's default was shown); `anonymous`: one rung above the campaign's default player view; `hidden`, `secret`, or a player-hidden category: GM-only. |
| Keys | `ns/slug` to `ns.slug`, through one mapping table; unmapped keys keep their name under their own group. |

### 6.3 Hand-added tags in physics namespaces

These were V3's tag overrides: the GM said a world is volcanic, shrouded or hexagonal against the physics. None of the four real saves has one, so this is rare.

- Where V4 has the fact (a cloud deck, a polar vortex, a pigment, a magnetic field class), the import writes a **pin** on it, marked authored, or authored-impossible if the physics disagrees.
- Where V4 has no such fact yet, the import writes a GM-only **note** on the body ("V3 override: geology/volcanic-tidal") and lists it in the report. Nothing the GM wrote is lost, and nothing is silently turned into fake physics.

### 6.4 The mapping is data, and shared

One table, `v3-tags.json`, in the `sse3-classic` pack: V3 key or prefix, kind, V4 id or target, notes. The kernel importer reads it, and the converter module reads the same file for V4 to V3 export (V4 tags to V3 keys where a row exists, the rest into GM notes, each listed in the loss report). One table, two directions, no second list.

The import report gains a tags section: counts per outcome, and every tag that was not a plain mapping, by body.

## 7. Build packages

Each is one pull request with its evidence packet. Ids use a new prefix, TG.

| Id | Package | Owner thread | Needs | Acceptance checks |
|---|---|---|---|---|
| TG1 | Tag declarations (`provides.other.tags`), the campaign pack, load-time check that every referenced tag is declared; reserve the `trade` tag group and `economy.` kept-kind prefix for the 4.1 economy pack | Build kernel resolve and rules | K8 | a stack with an undeclared tag in a rule, a template, a trigger and a view fails with four messages naming pack and reference; the Traveller pack loads clean once its tags are declared; a freely typed tag appears in the campaign pack; `exclusive` refuses a second tag of the group |
| TG2 | The physics wall | Build kernel resolve and rules | TG1 | a fixture physics flow containing `world.has_tag` is refused at load; adding and removing 50 tags on the Sol fixture leaves every resolved fact and every golden hash unchanged; no golden changes |
| TG3 | Authored overrules derived | Build kernel resolve and rules | K8 | `tag.x = false` removes a rule tag from scenes, conditions and sifters; `true` shows the authored mark over the rule's; removing the authored fact restores the rule's answer with its evidence |
| TG4 | Seeded chance on rules (if question 1 is yes) | Build kernel resolve and rules | TG1 | the draw depends only on universe seed, address and rule id: shuffling the rule list, adding a rule or disabling a group changes no other tag on 500 bodies; a rolled tag's "Why?" says it was rolled and at what odds; chance 0 and 1 behave as never and always |
| TG5 | Tags level with facts in data views (no separate mechanism): per-object tier override, the single "undisclosed" placeholder, one redaction point | Build kernel resolve and rules | TG1 | three hidden tags on one body give one marker to an audience one rung short and nothing to one two rungs short; a GM-only tag appears in no player scene, note, sifter hit, tag grant result or export (one test walks every player-facing reply); the marker carries no id, colour or count |
| TG6 | V3 tag import and the mapping table | Build SSE4 proven core (owns the V3 importer) | TG1, TG3 | every key in V3's registry, defaults and namespace table has a row (a test reads the vendored V3 lists); the four real saves import with zero tags left in `v3.tags`, which is removed; Regina keeps its 29 hand-set construct tags and its 90 rule hooks as authored tags; Procyon's 78 flat and legacy tags are all dropped as derived and counted; a fixture with a hand-added `structure/cloud-deck` becomes a pin, and one with a hand-added `geology/volcanic-tidal` becomes a GM note; visibility maps as 6.2 |
| TG7 | Converter uses the same table, both directions | Make converter a shared SSE4 module | TG6 | V3 to V4 to V3 on Regina returns every hand-set tag and every hook tag; V4 tags with no V3 row land in GM notes and the loss report |
| TG8 | Stock tag packs: `resource.*` labels as rules over the TG11 inventory (no chance), `access.*` rules over V4 facts, in a base hooks pack; optional intrigue pack; declare the Traveller pack's tags; `resources` facts and the `resource` group at the `surveyed` tier | Build Traveller pack (with the kernel thread for the base pack) | TG1, TG4, TG11 | Sol: helium-3 on all four giants and none on rock, water ice on Europa and not on a gas giant (V3's B33 case), gas skimming on giants only; every rule's "Why?" lists the facts read; an unsurveyed audience sees no resource label and a surveyed one sees it with a banded estimate; intrigue off by default |
| TG11 | Resource inventory: cited blocks giving kilograms per resource from makeup, mass, air and sea; resource kinds declared by packs (natural block optional); `extracted` and `stock` kept kinds per resource on any address; held = natural minus extracted plus stock; a mining program step that writes it | Build SSE4 proven core (inventory blocks), Build Traveller pack (kept kind and program) | TG1 | Sol: a metal-rich asteroid fixture holds more iron per kilogram than a carbonaceous one, both traceable to makeup; no inventory block uses a random draw; a station with no natural inventory and a stock of `packaged-food` carries `resource.packaged-food`; moving stock between two places leaves the total unchanged; mining a fixture rock to below the pack threshold removes `resource.iron` with no authored event, and the "Why?" shows inventory, extracted and threshold; extraction changes no resolved fact and no golden hash |
| TG9 | Ships without number-carrying tags: discipline and readiness as kept kinds, status triggers, drive ability from hardware facts | Design general game and story layer (design), then Build Traveller pack | TG1 | a ship tagged `status.derelict` reaches readiness 0 through a visible trigger, and the GM can set readiness differently without removing the tag; no program reads a number from a tag (a check over pack programs) |
| TG10 | Tags in the chart room: one tag list on the object card (derived with "Why?", authored with who and when, force and hold off, tier button), finder by tag, highlight as a lens control, campaign tags under Workshop (a `rail` entry, no new tab) | Build chart room shell | TG1, TG3, TG5 | screenshots at desktop and phone in both GM modes; "as players see it" shows the undisclosed marker; a tag forced on shows the authored mark; nothing in the card offers to change physics by tag |

Suggested order: TG1, then TG2 and TG3 together, TG6, TG11, then the rest as their threads have room. TG2 is small and worth doing early, because it turns "tags never reach physics" from a habit into a check.

After acceptance: copy this file to `handover/design/` in the repo and add the TG rows to the build plan (the story-layer design thread does the design syncs).

## 8. Rulings (the open questions, each decided as recommended, Alex 21 Nov)

Alex accepted every recommendation below, so read each "Recommend" as the ruling. One note: the three paid-pack rules in 5.7 were written minutes before his acceptance; the paid-packs design carries them forward for his own review.

1. **Should rules be allowed a dice roll?** V3's hooks were "condition plus chance". SSE4's rules are pure conditions. *Recommend yes*, as one optional `chance` on a rule, drawn from the universe seed, the address and the rule id only, so order never matters and adding a rule never re-rolls a campaign. It is the one honest place for dice: where no physics exists to decide (a rumour, a legend). Anything physics can decide should have no chance on it; V3 already moved that way (oxidiser, noble gases, hydrocarbons at 100%).
2. **Resources: tags, or real values from physics?** (Alex raised this again on 21 Nov; full answer in 5.5.) *Recommend both, one read off the other.* Real inventories from generation are the truth and can be mined out; the `resource.*` label is a derived, survey-gated quick reference on top, with the threshold set by the game pack. Nobody authors or rolls a resource tag in normal play. The part that is yours to rule: whether the label layer is worth keeping at all. I think yes, because a mining or survey game, the finder and the autopilot all want one word to look for, and it costs one rule per resource.
3. **Imported V3 hooks: keep V3's rolls or let V4 re-derive?** *Recommend keep*, as authored tags, with an import switch "re-roll with V4 rules" for GMs who would rather start fresh. A campaign in play should not change under its GM. Resources are the exception since 5.5: V4's physics decides, and a V3 resource tag survives only as a forced label where the physics disagrees.
4. **Inside the data view filters, what tier does a newly typed GM tag start at?** Visibility itself is settled: tags are filtered by data views like everything else, with no mechanism of their own. The only open point is the default. V3 showed new tags to players. SSE4's rule for facts is "never leak by omission" (unlisted means GM-only). *Recommend GM-only*, with the tier one click away on the tag and a campaign setting for GMs who want V3's habit. Imported V3 tags keep the visibility they had.
5. **A per-object tier override for a tag, as V3 allowed?** *Recommend yes*, as an ordinary data view override keyed on the object, not a new mechanism. "Secretly owns this one" is a real table need that otherwise forces duplicate tags.
6. **May the GM force a derived tag on or hold it off, when that no longer changes the world?** *Recommend yes.* It is cheap, it shows the authored mark, and it is the right tool for "this world is officially listed as a fuel stop". The card should say plainly that it changes the label, and offer "pin the fact instead" beside it.
7. **Cut the per-view marker shapes (chips, pins, flags, staff colours)?** *Recommend yes for now.* One marker style per theme, colour from the tag. It was a lot of surface for a small gain, and themes are the V4 place for that kind of choice. Easy to add back as a theme option if it is missed.
8. **Anomaly reasons on pins: keep the twelve seeds?** *Recommend yes, unchanged*, as a declared group the GM can extend, and keep V3's rule that a pin with no reason tells players nothing.
9. **Use "mark" for honesty and source marks, and keep "tag" for labels?** *Recommend yes*, as a wording fix in the handoff and the Traveller design ("marked authored-impossible", "source mark traveller-generated"). No behaviour changes.
10. **[Ruled by Alex, 21 Nov: agreed as recommended, and reuse the Explorers site's existing attribution and statement mechanism rather than a new one.]** **Do we write and feature packs for named settings ourselves, or leave those to users?** Fan packs are allowed on the Explorers site either way (5.7). *Recommend: the dozen packs built into SSE4 and featured on the front shelf are original and generic settings; packs for named settings live on a clearly labelled "fan-made, unofficial" shelf, and we may write a few of those ourselves to show how it is done.* The difference is small but real: a pack bundled in the app is our product carrying someone's name, and a pack on a fan shelf is fan work. It also means a takedown request never touches the app itself.
