# SSE4 classification design: the class atlas

Written 18 Nov by the design thread at Alex's request. Authority for how SSE4 names what a thing is, and how a user asks for "one of those, here". Read with the handoff (H), sections 6 (RESOLVE, QUERY, EXPLAIN), 7 (packs), 7A (Traveller pack), 8.4A (symbology) and the life design.

Sources: Alex's brief of 18 Nov; SSE 3's own notes in `docs/dev/adding-a-classification.md`, `type-vocabulary-prev4.md` and `faults-the-old-shape-allowed.md` (read by the design thread); current planetary science for the shipped class list.

---

## 1. What is wrong today, and what Alex asked for

SSE 3 classifies with **fingerprints**: bands over a hand-built feature map, a `base` class that competes and `modifier` classes that stack at a score of 0.6 or more. The idea is sound. What hurts:

- Adding one class means touching about eleven places in dependency order (vocabulary list, feature map in code, fingerprint, rarity table, viability gates, picker, 2D shape, 3D look, image map, link map, documents, fixtures). SSE 3 needed a written checklist for it.
- The two directions are separate code. Forward (bands) and reverse (viability gates in generation) can drift apart, so a type can be defined yet never placeable, or placeable yet never recognised.
- A feature exists only if someone wrote a line of code for it. A class about anything else cannot be assigned, or is lost on the next pass.
- The class list itself predates most of what is now known about exoplanets.
- None of it can be seen or edited in the app.

Alex's brief: it serves two purposes, matching physical criteria to a type and placing a requested type by reverse lookup; it is mostly used to add habitable planets, so it must cross-reference atmospheres and biospheres; it must be an editable artefact in the UI that Orac and users add to and tweak; it is an n-dimensional space with classes as regions; would a vector database suit?

## 2. The answer in one paragraph

Alex's picture is the right one: a space with one axis per physical characteristic, and classes as regions in it. SSE4 makes that space explicit and visible as the **class atlas**. **One record per class** is the single source for recognising it, placing it, drawing it, offering it in the picker, generating it and testing it. Both directions read that one record, so they cannot drift. The atlas is pack data, edited on a chart-like screen, by hand or by Orac. A vector database is the wrong engine for it (section 9), but the useful part of that idea, distance and nearness, is kept as exact arithmetic on named axes.

## 3. The space: axes

An **axis** is any quantity the kernel publishes for a body, registered for classification with a scale. No code is needed to add one: if a flow or a suite publishes the quantity, an atlas may use it.

```
Axis { quantity: QuantityKind (one publisher, H5),
       applies_to: body kinds,
       scale: "linear" | "log" | "fraction" | "category" | "flag",
       span: [low, high]          the range used to normalise distance on this axis
       label, unit, explain_ref }
```

Starting axes (the base physical atlas uses these; packs add more):

| Group | Axes |
|---|---|
| Bulk | mass, radius, bulk density, makeup fractions (metal, rock, carbon, water and ice, gas), surface gravity, escape speed, porosity |
| Energy | stellar flux, equilibrium temperature, surface temperature (mean, day, night), internal heat flow, tidal heating |
| Orbit and spin | semi-major axis, eccentricity, obliquity, day length, tidally locked (flag), host kind (star class, planet, none for rogues) |
| Air | surface pressure, partial pressures by gas group (oxygen, carbon dioxide, nitrogen, methane, water vapour, hydrogen and helium, sulphur compounds, others), greenhouse warming, cloud cover, escape regime |
| Surface | surface liquid fraction, which liquid (category), ice cover, relief, activity (volcanic, tectonic regime from the surface design) |
| Shielding | magnetic field, surface radiation dose |
| Time | age, time since last resurfacing |
| Life | biosphere stage and energy budget from the life suite, producer cover, oxygen source (biological or not) |
| Authored facts | things only a GM or generator can say: shape (two lobes), origin (captured), purpose (shipyard). These are inputs, stored as facts with evidence labels, and are axes like any other |

The last row removes SSE 3's trap: a class about shape, history or intent has an axis to stand on, and it survives every reprocess because the fact is stored.

## 4. A class: one record, everything in it

```
Class { id: "planet/ocean/temperate", names: {physics, plain, story}, parent?: ClassId,
        scheme: SchemeId,                      which atlas it belongs to (section 6)
        kind: "base" | "modifier",             base classes compete; modifiers stack (kept from SSE 3)
        meaning: "identity" | "flourish",      identity classes may drive generation and gates; a flourish only changes how a thing reads or draws
        region: [{ axis, core: [a,b], fringe: [a',b'] }      numeric: fully in across the core, fading to zero across the fringe
                 | { axis, any_of: [categories] } | { axis, is: true|false }],
        typical: [{ axis, centre, spread }],   where members usually sit; used only for placing and generating, never for recognising
        needs: [ClassId | condition],          cross-references, e.g. "oxygen above 0.1 bar needs a biological or stated abiotic source"
        rarity: 0..1, generate: weights by context,          the GM's exotic dial (kept from SSE 3)
        look: { mark, shading, colour, silhouette recipe, surface recipe, image?: {file, licence, credit} },    feeds symbology 8.4A and the renderers
        learn: { summary, explain_ref, link? },              the three explain lenses and an optional outside link
        examples: [{ name, values, expect: "in" | "out" }],  known worlds that must or must not match: the class's own tests
        source: pack id, notes, evidence: "science" | "speculative" | "fictional" }
```

- **Parents** give a tree: family, class, variant (terrestrial, ocean world, temperate ocean world). A child inherits its parent's region and narrows it. A body is labelled with the deepest class it fits, and is also a member of every ancestor, so "any ocean world" is a valid request and a valid filter.
- **Core and fringe** replace hard cut-offs. A world just outside a core is "nearly" that class, with a number, instead of flipping to something unrelated.
- **Region versus typical** is the key to going both ways. *Region* says what counts. *Typical* says what to aim for when making one.

## 5. Forward: what is this body?

For each class the body's **fit** is the lowest of its per-axis fits (1 inside the core, falling to 0 across the fringe; a failed category or flag is 0). Lowest, not average, so one disqualifying fact cannot be outvoted.

- Base classes: the deepest class with the highest fit wins. Ties break by smaller region, then by id, so the result is always the same. Below a pack-set threshold the body is **unclassified**, shown as such, with its nearest classes listed.
- Modifiers: every modifier at or above its threshold is added.
- The answer always carries its working: the fit, which axis set it, the runners-up, and for each near miss **what would have to change and by how much** ("0.8 bar more air and it is a temperate ocean world"). This is ordinary EXPLAIN output.
- **Stickiness over time**: a label changes only when the new class beats the old by a margin, so labels do not flicker during ageing. When a label does change, that is a chronicle event with a date ("Tarn stopped being a garden world 1.2 billion years from now"), which the story sifter can use.
- Classification is derived, never stored as the only record of anything (H7). Remove every atlas and the facts are untouched.
- Cost: a few hundred classes times a handful of axes per body is trivial arithmetic. It runs in the kernel with pinned maths, identically on a phone.

## 6. Schemes: more than one way to name a world

A **scheme** is one atlas answering one question. A body gets one base label per scheme. Schemes stack like any pack content.

| Scheme | Question | Ships in |
|---|---|---|
| Physical type | what kind of object is this | base physics pack |
| Habitability | who could live here, and with what equipment | base pack, extended by game and universe packs |
| Small bodies, stars, belts, systems | the same machinery for other object kinds | base pack |
| SSE 3 classic | what SSE 3 called it | import pack, so V3 saves keep their labels (section 11) |
| Traveller codes | UWP digits, trade codes, atmosphere codes | Traveller pack (H7A). Each digit is a small scheme on one or two axes |
| A fictional scheme | "Class M" style letters and the like | universe packs |

**Habitability is not one class.** It is a scheme whose classes are **requirement profiles**: the region of the same space in which a given kind of person can live. "Shirt-sleeve for humans", "humans with a filter mask", "humans in a pressure suit", and the same for any species a pack defines, each as ranges of temperature, pressure, oxygen partial pressure, toxic gases, gravity and radiation. This is where atmospheres cross-reference: the Traveller gas mixes and SSE 3's presets are regions on the air axes, so "tainted" and "needs a filter mask" are the same fact seen through two schemes. Biosphere cross-reference comes through the life axes and through `needs`: an oxygen-rich class needs a source of oxygen, which normally means the life suite has reached oxygen-producing life on that world.

## 7. Reverse: "put one of those here"

The important point: almost every axis a class cares about is a **result** (surface temperature, air, oceans, life), not something that can be set. Setting results directly would be faking the physics. So placing a class is a **search over causes**: find the orbit, mass, starting ingredients, spin and age that, run through the real flows, land inside the class.

The request: a class (or any ancestor), a scope (this system, this zone, this slot, or "anywhere in this region" at galaxy scale), and optional extras (a moon, a named gas mix, a profile such as "shirt-sleeve for humans").

1. **Quick bounds.** Cheap inverse rules narrow the search before anything is run: the flux band that can give the class's temperatures around this star, the mass band that keeps this air for this age, the slots that are stable among the bodies already present. If the bounds are empty the answer is already "not here", with the reason.
2. **Search.** Inside the bounds, a fixed number of candidate causes are drawn from the class's *typical* values with keyed draws, run through the flows, and scored with the same fit as section 5. The best candidates are refined a fixed number of steps. Fixed counts and keyed draws make the result identical on every device.
3. **Cross-references.** Each `needs` is settled by the suite that owns it. A living oxygen world asks the life suite whether such a biosphere can have arisen there by now; if yes the life history is resolved to match, if not see step 4.
4. **Outcome, always one of four, shown plainly:**
   - **Placed**: full fit. It is added as authored inputs plus normal derived results.
   - **Closest here**: best fit below 1, with what stands in the way ("the star is too dim for open water at any stable orbit") and choices: accept the nearest class instead, move or change something named, or look elsewhere.
   - **Needs a helping hand**: possible only with an authored intervention (seeded life, imported air, terraforming). Added with the intervention recorded as an authored event.
   - **Forced**: the GM insists. Values are written as `authored-impossible` and carry the honesty mark (H8.6). Never silent.
5. The picker previews this before commit: every class is shown as available, available with help, or not possible here, with the reason on hover. SSE 3's separate viability gates are gone; this *is* the gate, and it reads the same record as recognition.

**The same question at galaxy scale** is the green-regions workflow Alex described (galaxy design): "where could a temperate ocean world exist" runs step 1 against patch envelopes instead of one system and shades the map. Reverse classification and "find me a home" are one mechanism.

Generation uses the same records: when RESOLVE populates a system, each resolved body is simply classified; the rarity dial and weights bias which *typical* targets are aimed for when the envelope leaves a free choice. A class nobody can ever reach is reported by the atlas checks (section 8), not discovered by players.

## 8. The atlas in the UI: an editable chart of what worlds can be

A GM workspace screen in the chart-room style (mockup: "Class atlas").

- **Tree** of schemes and classes on the left, with counts of matching bodies in the campaign.
- **The chart**: pick any two axes and see the classes as inked regions (solid core, hatched fringe), with dots for the campaign's worlds, reference worlds (the Solar System's planets and moons, selected known exoplanets) and the class's own examples. Other axes are held by sliders, so the chart is a slice through the space.
- **The class sheet** on the right: one row per axis with core and fringe handles, typical values, needs, look, rarity, examples. Edited in place.
- **Checks** run live: overlaps between competing classes (shown as a cross-hatched area on the chart), gaps (campaign or reference worlds that fit nothing), examples that now fail, classes that cannot be reached by any placement in a standard set of test systems, and how many campaign worlds a change would relabel, listed before saving.
- **Make a class from examples**: lasso dots or pick worlds, and the atlas proposes the tightest region around them and the axes that best separate them from their neighbours. The user adjusts and names it.
- **Orac** works through the same change requests as everywhere (H13): "add hycean worlds", "split desert worlds into hot and cold", "make a habitability profile for a low-gravity methane-breathing species". Orac's proposal arrives as a diff on the atlas with the checks already run. The schema is rich for Orac, with a few promoted controls for people (H8.9).
- **Try it**: type or drag values and watch the label and the working change; or pick a class and a system and preview the reverse placement.
- Atlases are versioned pack content. A campaign can carry its own overrides. An atlas is shareable on the explorers site like any pack.

Adding a class is now one record on one screen. The eleven-place checklist is retired: picker, generation, placement, symbol, image, explanation and tests all read the record.

## 9. Would a vector database fit? Honest answer: no, but keep the instinct

What a vector database does well is find near neighbours among millions of points in hundreds of learned, unnamed dimensions, approximately. This problem is the opposite on every count:

- **Few, named axes.** Ten to thirty physical quantities with units. The meaning of each axis is the whole point, and "why is this a desert world" must have an answer in bar and kelvin. Learned embeddings cannot give one.
- **Regions, not points.** A class is a volume with edges a GM can drag. A vector store holds points and returns neighbours; it has no notion of a boundary, an overlap or a gap.
- **Exactness.** Approximate search, floating-point differences between devices and index rebuilds are all at odds with "same answer on every peer, forever". Exact arithmetic over a few hundred regions is both deterministic and faster than any index at this size.
- **The reverse direction is not a lookup at all.** Finding causes that produce a class needs the physics run forward (section 7). No stored table of vectors can answer it for a particular star, age and neighbourhood.
- **Weak devices and offline.** Nothing to host, nothing to load.

What is kept from the idea, done exactly:
- **Distance** between any world and any class, and between two worlds, on normalised axes. That gives "nearly a garden world", "the three most Earth-like worlds in this sector", "find me something like Titan".
- **Similarity search** across a campaign or a shared library, as a plain scan (thousands of bodies is nothing).
- **Learning from examples** (section 8), which is the genuinely useful half of the machine-learning instinct.
- Separately and optionally, the agents may use text embeddings to find classes or worlds *by description* ("somewhere bleak and windy"). That is a search aid on bring-your-own-key, never part of resolving or labelling anything.

## 10. The shipped physical atlas: brought up to date

The base list should reflect current planetary science rather than SSE 1's categories. The build session ports SSE 3's fingerprints first (so nothing is lost), then restructures to this tree. Outline, to be refined in the atlas editor:

- **By bulk** (mass, radius, makeup): iron-rich, rocky, water-rich, sub-Neptune (rock with a deep hydrogen envelope), ice giant, gas giant, super-Jupiter up to the brown-dwarf boundary, low-density "puffed" giants; the gap between large rocky worlds and small gas-rich ones is a real feature of the space and shows on the chart.
- **By surface state**, mostly for rocky and water-rich worlds: molten, hot greenhouse, hot desert, temperate desert, temperate with oceans, ocean-covered, deep-ocean-under-hydrogen (hycean), cold desert, ice-covered with open equator, fully frozen, ice shell over an ocean, airless rock.
- **Giant variants by temperature and cloud**: ammonia clouds, water clouds, cloudless, alkali, silicate clouds, ultra-hot.
- **Modifiers** (stack): tidally locked with a day side and night side, high eccentricity, ringed, strongly volcanic, tide-heated, magnetically unshielded, young, ancient, rogue (no star), captured, living (by biosphere stage), inhabited, terraformed, and SSE 3's small-body modifiers (rubble pile, contact binary).
- **Speculative and fictional** classes are welcome and labelled as such in `evidence`, so explain mode never passes them off as observed.

Every class ships with examples from the Solar System and well-measured exoplanets, which double as its tests.

## 11. Carry-over from SSE 3

- Fingerprints map directly: `match` bands become `region` cores (with a default fringe), `base` and `modifier` and the 0.6 threshold carry over, `weight` and rarity carry over, images and credits move into `look.image`, links into `learn.link`.
- The "SSE 3 classic" scheme keeps every V3 class id, so an imported save shows the labels its author knew, beside the new physical type.
- SSE 3's rule that a class is re-derived every pass stays. Its feature map is replaced by the axis registry. Its viability gates are replaced by section 7. Its overlap audit becomes the live checks.

## 12. Stages

| Stage | Contents | Needs | Evidence |
|---|---|---|---|
| C1 | Axis registry, class records, schemes, forward fit with working, stickiness, label-change events; SSE 3 fingerprints imported as the classic scheme | K5, P1 | every Solar System body gets its expected label; each label answers why and what is nearest; same labels on every device; relabel events replay from the manifest; removing all atlases leaves facts unchanged |
| C2 | Base physical atlas restructured (section 10), habitability scheme with requirement profiles, gas mixes as regions | C1, P3 | every class has passing examples; no unintended overlaps; reference exoplanet set classifies as published descriptions suggest |
| C3 | Reverse placement in a system: quick bounds, fixed-count search, cross-references to atmosphere and life, four outcomes, picker preview | C1, K8, L0 | a requested class is placed and then recognised as that class (round trip) on a fixture set of stars; impossible requests return the blocking reason; forced placements carry the mark; identical result on every device |
| C4 | Class atlas screen: tree, two-axis chart, class sheet, live checks, class from examples, try-it panel, Orac change requests | C1, U3, A2 | a new class added on the screen appears in picker, generation and symbols with no code change; an edit lists the worlds it would relabel before saving |
| C5 | Reverse at galaxy scale ("where could this exist") feeding green regions; similarity search across a campaign and library | C3, G5 | green regions for a class agree with brute-force placement on sampled patches |

C1 and C2 belong with the physics milestone, C3 and C4 with the open-physics milestone, C5 with the galaxy milestone.
