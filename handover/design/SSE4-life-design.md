# SSE4 life evolution design (life suite)

Companion to `SSE4-design-handoff.md` (section 11A fixes the interfaces and stages L0 to L5) and `SSE4-galaxy-design.md`. Date 17 Nov 2026. This document designs stages L1 and L2 in detail: a biosphere for any world, with a history, **without simulation**. Simulated mechanisms from Evolution Lab replace boxes later.

Stance: nobody knows how common life is or how it starts. So the model is strict where physics is known (energy, materials, limits on bodies) and openly adjustable where it is not (origin, the big evolutionary leaps). The user sees which is which. Everything is a flow of atoms in the same editor as climate. Defaults are tuned so Earth's inputs give an Earth-like outline, and are labelled as one data point.

Life here means aggregate populations (guilds, lineages), never individuals or genomes. Default chemistry is carbon and water; solvent and chemistry are pack parameters, not assumptions in code.

---

## 1. The chain

```
world facts (physics suite)  ->  A habitats
                                 B energy and material budget     (known physics: strict)
                                 C origin                         (unknown: an explicit dial)
                                 D opportunity and innovations    (capability graph, waiting times)
                                 E catastrophes and recoveries    (from the physical chronicle)
                                 F diversity and ecosystem shape
                                 G resolve: guilds and their traits (derived from the world)
                                 H feedback on the planet          (stage L2)
                                 I signatures and what observers can tell
                                 J history: epochs, stories, why
                                 K minds and civilisations (sketch; stage L5 owns it)
```

Outputs at two depths, like everything else: a **life envelope** for a world that nobody has examined (cheap, a few numbers and distributions), and a **resolved biosphere** when someone looks (guilds, traits, dated history), recorded in the chronicle.

---

## 2. A: habitats

Rules turn a world's shells and regions into a **habitat graph**: nodes are places life could be, links are routes for material and organisms with a transfer rate.

Default habitat kinds (pack-extensible): sunlit ocean surface layer; ocean interior; sea floor with hydrothermal vents; ice-covered ocean under a shell; land surface by climate band; soils and shallow crust; deep crustal rock with water; cloud decks at a temperate pressure level; lakes of other liquids (hydrocarbon); tidal and terminator zones on locked worlds; caves and lava tubes.

Each habitat gets the requirement profile (handoff 11A): medium, solvent, temperature and pressure ranges and how much they swing, volume or area, how long it has existed continuously, energy fluxes by channel, material reservoirs, radiation dose, and connectivity. Habitats are born, change and die with the planet (an ocean freezes over: the surface habitat ends and a sub-ice habitat begins, linked by descent so life can carry over).

---

## 3. B: energy and material budget (strict)

Life cannot use more free energy or more atoms than the habitat supplies. This box sets hard caps, and it is where the astrophysics pays off.

**Energy channels** (power per unit area or volume, from physics-suite quantities):

- **Light**: photon flux at the habitat in usable bands, from the star's spectrum, the atmosphere's transmission, cloud, ice and water depth. Usable band limits are a property of the photosystem (pack; default about 400 to 1,100 nm allowing multi-photon schemes under red stars at lower efficiency).
- **Chemical from rock and water**: hydrogen and methane from water reacting with fresh rock (scales with new crust production and heat flow), sulphur and iron redox at vents.
- **Oxidants from above**: radiation splitting ice or water (important for ice moons), photochemistry.
- **Atmospheric disequilibrium**: lightning- and photochemistry-made compounds raining out.
- Tidal and radiogenic heat matter through the rock-water channel, not directly.

**Efficiencies**: fixed, cited conversion factors from energy flux to biomass production for each metabolism class.

**Materials**: carbon, nitrogen, phosphorus, sulphur, iron and trace metals. Supply rates come from weathering of exposed rock (needs land or active sea floor), volcanic outgassing, atmospheric fixation and delivery by infall. Phosphorus is usually the limit; a water world with no exposed rock is energy-rich and nutrient-starved, and the model must show that.

**Result per habitat**: maximum net primary production = the smaller of the energy-limited and the nutrient-limited values (Liebig's law). Standing biomass = production × a residence time by habitat kind. A world's totals are sums.

**Limits from conditions**: solvent must be liquid; temperature within the chemistry's window (default water-carbon window roughly −20 to 122 °C); water activity, acidity, salinity and radiation limits are pack values. Outside the window the cap is zero, with the reason recorded.

**Power density caps complexity** (strict, and decisive): the number of feeding levels an ecosystem can sustain is about log(production / minimum viable) / log(1 / transfer efficiency), with transfer efficiency near 10 percent. Large active bodies need a high-power metabolism (on Earth, oxygen). A sub-ice ocean running on a trickle of chemical energy can host microbes and perhaps small slow grazers, not whales, and explain mode can say exactly why.

Checks: Earth today within a factor of three of about 100 billion tonnes of carbon fixed per year; early Earth before oxygenic photosynthesis lower by a factor of hundreds to thousands; published ranges for Europa, Enceladus, Mars subsurface and Titan reproduced in order of magnitude.

---

## 4. C: origin (an explicit dial)

Origin is a waiting-time process per habitat: chance of an origin by time t = 1 − exp(−∫ rate dt).

rate = base rate × setting factors × area.

- **Setting factors** (each a visible, switchable hypothesis with citations): alkaline hydrothermal vents; land pools with wet-dry cycling and ultraviolet light; delivered organics; mineral catalysts. A world offering several settings gets several chances.
- **Base rate** is the honest unknown. One dial, shown in the UI as *Rare, Earth-like pace, Common*, with the default set so an Earth-like early world most often gets life within a few hundred million years. The dial lives in a universe or physics pack, so a setting can declare its own cosmos to be teeming or nearly empty.
- **Transfer**: life can arrive by rocks exchanged between worlds of one system (rate from impact history and escape and survival fractions), and very rarely between stars of one birth cluster. Shared origin is recorded: related biospheres share biochemistry, which later decides whether one world's life is edible or infectious to another's.
- **Multiple origins** on one world are allowed and usually merge or out-compete; a surviving "shadow biosphere" is a rare flagged outcome.
- **Seeding by someone**: an authored event.

The draw is keyed by habitat address. Label: speculative parameter, plausible reconstruction.

---

## 5. D: opportunity and innovations

**Evolutionary opportunity** is not time alone. It accumulates as: (population size, from biomass and cell or body size) × (generations per year, from temperature via metabolic rate) × (habitat variety) × time, discounted by instability. A cold, thin, slow biosphere accumulates opportunity thousands of times more slowly than a warm productive one.

**Capabilities form a graph with prerequisites, not a ladder.** Each capability has: prerequisites (other capabilities and physical conditions), a rate per unit opportunity, a cost, and effects. Waiting time is a keyed draw against accumulated opportunity, so innovations land at specific dates tied to the world's actual history. Convergence is allowed: a capability with a high rate arises many times.

Default capability set (carbon-water pack; all rates are pack data with a short justification each):

| Capability | Needs | Typical effect |
|---|---|---|
| Chemical energy use (several kinds) | origin | base biosphere |
| Light harvesting without oxygen release | light, an electron donor | production up 10 to 100 times |
| Oxygen-releasing photosynthesis | the above, liquid water at the surface | production up again; oxidant released (see H) |
| Nitrogen fixing, other nutrient unlocks | relevant scarcity | raises the nutrient cap |
| Dormancy, spores | variability | survives catastrophes; enables transfer between worlds |
| Complex cells (merger of lineages) | sustained opportunity; high-power metabolism helps | a hard step: low rate, dial |
| Sex and recombination | complex cells | faster innovation |
| Multicellularity | complex cells | arises often |
| Large active bodies | high-power metabolism, enough production | feeding levels, predators, burrowing |
| Hard parts | minerals in solution | fossils, reefs |
| Land colonisation | land, radiation shield (ozone or equivalent) | weathering up (see H) |
| Flight, large size, warm blood, social living, tool use, language | bodies plus physical feasibility (section 7) | diversity; leads to K |

"Hard steps" (origin, oxygenic photosynthesis, complex cells, language) carry their own dials because the science is contested. With all dials at default, most living worlds stay microbial for most of their history, as Earth did. That is a result, not a bug, and the feasibility query lets a GM search for the exceptions.

Non-light worlds follow the same graph and stall where energy stalls.

---

## 6. E: catastrophes and recoveries

The physical chronicle already contains the world's disasters. Each maps to a kill mechanism and a severity per habitat:

large impacts; global glaciations; runaway warming or ocean loss; massive volcanic provinces; loss of magnetic shielding plus stellar flares; a supernova or gamma-ray burst within reach (from the galaxy suite); a stellar encounter's comet shower; the star's own brightening and death; tidal or orbital changes; and self-inflicted ones from H (oxygen poisoning the old biosphere, life drawing down greenhouse gas into a freeze).

Effect: a fraction of diversity lost per habitat (refuges matter: deep rock and sub-ice habitats shelter survivors, so the habitat graph decides who restarts the world); capabilities held by few lineages can be lost; recovery is logistic with a time constant of millions to tens of millions of years, often with a radiation of new forms. Total sterilisation of a world with deep habitats is hard; the model shows why.

Events are recorded with causes pointing at the physical event, so "why did this world's reefs die 210 Myr ago" traces to a named impact or a named supernova in the neighbourhood.

---

## 7. F and G: ecosystem shape and resolved guilds

**Diversity** D(t) grows logistically toward a capacity set by production, habitable area (species-area law, exponent about 0.25), number of isolated provinces (continents, seas, islands of habitat: from the habitat graph), climate variety and time since the last catastrophe.

**Resolve** produces 8 to 40 aggregate guilds for a living world (fewer for microbial worlds), each with a habitat, a role, a share of production, a lineage (ancestry links, origin date) and traits. Roles are filled from the energy channels and feeding levels available: producers per energy channel, grazers, predators per level, decomposers, symbionts and parasites. Relations give the four lenses from Evolution Lab: ancestry, resource (who eats or feeds whom), habitat, capability.

**Traits are derived from the world, then varied by keyed draws inside physical limits.** This is what makes each world's life feel like it belongs there:

| Trait | Derived from |
|---|---|
| Producer colour | the star's spectrum at the surface after the atmosphere: pigments absorb where photons are plentiful, so foliage under a red dwarf tends dark, under a hot star may reflect blue; several solutions are possible and drawn among |
| Maximum land animal size, limb proportions | surface gravity and bone-like material limits |
| Flight | air density, gravity, metabolic power: feasible, marginal or impossible, and for what body mass |
| Tallest plants | gravity and the solvent's hydraulic limit |
| Pace of life | temperature via metabolic rate |
| Day and season strategies | rotation, tilt, eccentricity, tidal locking (terminator migrators, long sleepers) |
| Senses | what carries information there: light bands available, sound speed and absorption in the medium, electric fields in salty water, darkness under ice |
| Body plans in oceans | viscosity and density |
| Armour and shells | dissolved minerals |
| Biochemistry notes | solvent, handedness (a draw), shared or separate origin |
| Use to visitors | edible, toxic, allergenic, infectious or inert, from biochemical relatedness |

Names and descriptions come from the vocabulary layers (physics: "anoxygenic phototroph mat"; plain: "purple shoreline slime"; story: Minerva's words). Pictures, if ever generated, are derived artefacts with recipes, never canonical facts.

---

## 8. H: feedback on the planet (stage L2)

At each common time boundary the life suite returns fluxes; the physics suite decides the consequences. The life suite never sets pressure, temperature or composition directly.

- Gases produced and consumed: oxygen, methane, nitrous oxide, carbon dioxide, hydrogen, sulphur gases.
- **Oxygenation is a balance, not a milestone**: oxygen builds only when biological production plus carbon burial outpace the planet's reducing sinks (volcanic gases, fresh rock). Small, reduced or volcanically busy planets may never oxygenate; the physics suite's interior and outgassing models decide. An ozone layer then follows from physics.
- Weathering enhancement by land life, drawing down carbon dioxide (can cool a world into glaciation).
- Albedo changes from surface cover.
- Carbon burial: sediments, and over time coal, oil and gas analogues as located resources (regions gain deposits; useful to later civilisations and to games).
- Biogenic rock and landforms: reefs and carbonate platforms, soils, iron formations, atolls.

No assumption that life stabilises its planet. It can and does wreck it sometimes; both outcomes fall out of the same fluxes.

---

## 9. I: signatures and what observers can tell

True state, surviving evidence, and what an observer can detect are kept separate (Evolution Lab's signature contract), and this plugs straight into player data views.

- **From afar** (obvious tier): colour and its seasonal change.
- **Scanned**: atmospheric gases out of balance (oxygen with methane), a vegetation-like edge in the reflected spectrum, night-side light or heat for technology. Reported as a *candidate with a confidence*, because physics makes false positives (oxygen from water loss around red dwarfs, methane from rock) and false negatives (life under ice shows nothing). Players can be wrong for good reasons.
- **Surveyed**: ecosystem summary, hazards, guild cards, resources.
- **Evidence of the past**: fossils, isotope and mineral traces, and their survival against erosion and plate recycling. A dead world can still tell its story to a survey team.

---

## 10. J: history and stories

A resolved biosphere has a dated history: origin, innovations, catastrophes, recoveries, colonisations, feedback turning points. Sifters name epochs afterwards from what happened and how much it mattered (persistence, reach, how much it changed capacity, reversibility). They also raise story hooks: a world one innovation short of animals; a biosphere doomed in 40 Myr by its brightening star; two related biospheres in one system; a relic ecosystem in the last deep refuge of a dying world; a false-positive paradise.

Every item answers "why?" in three lenses, down to the physical cause.

---

## 11. K: minds and civilisations (sketch only)

Within L1 and L2 the suite only goes as far as: the capability graph can reach tool use and language on worlds where the prerequisites and physics allow (land or some workable equivalent for fire and metallurgy is recorded as a factor, not a ban), giving an emergence date and a species sketch. What a civilisation then does is authored by the GM, with optional helper flows for population, energy use and detectable signatures. A proper technosphere suite is stage L5 and reuses the same primitives. At galaxy scale, "how many civilisations in this patch at this time" is an envelope like any other, driven by the same dials, so the galaxy chart can shade likely neighbours and a universe pack can decide whether its galaxy is crowded or silent.

---

## 12. Life at larger scales

- **World envelope** (unexamined world): chance of life, likely stage, production class, signature strengths, computed from the same flows using expected values instead of draws. Cheap enough to run for every body in a system on creation.
- **System**: transfer links between worlds.
- **Patch and galaxy**: fraction of systems with life, with complex life, with minds, as functions of the patch envelope and age. Feeds the green regions and outcome-first queries ("a red-dwarf world with a dark-foliage forest biosphere and a breathable atmosphere within 30 parsecs of here").

A committed outcome-first pick is recorded as selected, so nobody mistakes it for typical.

---

## 13. Honesty rules specific to life

- Every dial with weak scientific support is visible, named, and stamped on results that depend on it.
- Outputs carry the label *plausible reconstruction*, never prediction.
- The Earth outline is a calibration point for defaults and must not be hard-coded as a sequence.
- Tags ("garden world", "microbial mats") are derived from quantities, never the only record.
- Fictional packs (silicon life, energy beings, the Force) are welcome and stamped fictional.

---

## 14. Stages and checks (life track)

| Stage | Delivers | Verified by |
|---|---|---|
| L0 | Interfaces, habitat graph from regions, empty suite | schema tests; habitat graphs for Earth, Mars, Europa, Titan, Venus fixtures look right and are stable under hash |
| L1a | Box B: budgets and caps; world envelope numbers on every body | the order-of-magnitude checks in section 3; no result exceeds its cap in 10,000 random worlds |
| L1b | Boxes C, D, E: origin, innovations, catastrophes; dated history | Earth fixture: life early, long microbial era, late large bodies, in most seeds at default dials; moving a dial moves outcomes monotonically; every event has a cause |
| L1c | Boxes F, G: diversity, resolve to guilds, traits, four lenses, vocabulary | trait rules respect limits (no flyers where flight is impossible) across 10,000 worlds; order-independent resolve; guild shares sum to production |
| L1d | Boxes I, J: signatures, data-view integration, sifters, biosphere view and life timeline | scanned view shows candidate plus confidence; a scripted false positive and a false negative world behave as designed |
| L2 | Box H: fluxes and coupling at time boundaries | Earth fixture oxygenates through the balance, not by date; a small reduced world does not; coupled run is deterministic and replayable from its manifest; conservation of exchanged material at the boundary |
| L2b | Larger scales: patch and galaxy envelopes, queries | galaxy-scale fractions consistent with summing sampled resolved worlds within tolerance |
| L3+ | Simulated mechanisms from Evolution Lab, box by box | each with a fidelity contract against the procedural box it replaces |

Fixture worlds (pack data, with cited inputs): Earth through time, Mars through time, Europa, Enceladus, Titan, Venus cloud deck, a tidally locked red-dwarf world, a water world with no land, a super-Earth, a rogue planet with a deep ocean.
