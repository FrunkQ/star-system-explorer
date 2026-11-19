# SSE4 evolution design: life from energy gradients (track E)

Companion to `SSE4-life-design.md` (boxes A to K, stages L0 to L2b, all built except L2b) and handoff section 11A (stages L0 to L5). Date 19 Nov 2026. Design only: no code has been written for this.

This document answers Alex's brief of 19 Nov: what a simple evolutionary system looks like in V4 at first, where it goes to line up with Evolution Lab, what is possible and what is not, and how to build it. It also takes in four points he raised the same day: high oxygen is both a hazard and a boost, so sinks should compete for it; biomes should form naturally, "like Earth, not like sci-fi"; soil and the rock-breaking life that makes it are central; and life rules should key on conditions, not on a "planet" kind, because stars and planets are becoming one body object.

It does not replace the life design. It changes the inside of four of its boxes (B budget, D innovations, G guilds, H feedback), adds two things it lacked (soil, biomes), and sets out the road to Evolution Lab.

---

## 1. The idea in one page

Today the life suite knows five metabolisms because the pack lists five, and one chemistry because the budget code asks `solvent == "water"`. Earth is in the data, but it is also still a little in the shape of the code.

The change is to stop listing what life does and start listing what the world offers:

1. **The world offers gradients.** A gradient is free energy that is there to be had but is not being released on its own: sunlight falling on a surface, hydrogen and carbon dioxide sitting together in vent water, oxygen and methane sharing an atmosphere. The physics suite already tracks the gases, rocks, fluids and light. A gradient catalogue is computed from them.
2. **A chemistry decides which gradients are reachable.** A chemistry is pack data: a solvent, a scaffold element, a temperature window, the elements it needs, and how much energy it costs to build a gram of itself. Carbon in water is one chemistry. Others are other rows of data, each with an honest evidence tag.
3. **A metabolism is a (gradient, chemistry) pair that pays its way.** It exists as a possibility wherever the energy per reaction, the power per unit of living mass and the materials all clear their thresholds. Nobody lists it.
4. **Evolution is the filling of open gradients.** An untapped gradient is an opportunity. The bigger the prize and the more life there is to stumble on it, the sooner it is taken. When life itself makes a new gradient (free oxygen is the great example), new ways of living off it follow, and they pull the gradient back down.
5. **Guilds carry a few traits that selection moves.** No genomes and no individuals: each guild has a handful of numbers (size, temperature optimum, pigment band, water strategy and so on) that drift toward what its habitat rewards, split when a habitat offers more than one way to live, and fail when the world changes faster than they can follow.
6. **Soil and biomes fall out of the same accounts.** Rock-weathering life plus decomposers plus climate plus time make soil. Climate, soil, light and disturbance decide which plant-like strategy wins in each region. A biome is the name of the winner, derived, never assigned.

The rest of this document makes each of those concrete, says what the evidence is, and proposes packages.

**Scope for V4 (Alex, 19 Nov 15:00): "Now it's 'they exist'."** In V4 evolutionary biology is there to do planetary work: manage the gas mix, make soil, set the ground cover and the hazards. What living things look like (appearance, body plans, creature design) is a later version. So this design is split:

- **V4 first stage, function and planetary effect:** gradients and chemistries (sections 3, 4), gradient-filling (5), the oxygen sinks and fire (7), soil (8), cover and biomes as a property of the ground (9), signatures (10). Of the guild traits in section 6, only the ones that change a flux or a cover number are in scope: size class, temperature fit, light band, water strategy, tissue economics, stature, dormancy, disturbance strategy. Each is there because the planet's accounts need it.
- **Later, appearance:** body plans, limbs, senses, colour beyond what the ground reflects, creature sketches and pictures, and Evolution Lab's inherited morphology recipes. The trait rules L1c already built (flight limits, largest animal, senses) stay as they are and are not extended in this track. Section 11 puts appearance on the longer path.

**Evidence tags** used below are Evolution Lab's own six, which the L0 mapping already ties to SSE4's labels: *observed*, *demonstrated*, *inferred*, *plausible*, *speculative*, *fictional*.

---

## 2. What is built, and what this changes

| Box | Built today (L0 to L2) | Changed by this design |
|---|---|---|
| A habitats | Habitat graph from a world's shells; 11 habitat kinds | Habitat rules test conditions on any body's shells, never the body's kind (section 9) |
| B budget | Five listed metabolisms; closure tests written for water; Liebig cap | Gradient catalogue and chemistry packs generate the metabolisms; the five become a check, not an input (sections 3, 4) |
| C origin | Hazard per habitat, one dial | One dial per chemistry; setting factors unchanged (section 4.3) |
| D innovations | Ten capabilities, fixed rates, two hard-step dials | Metabolic capabilities are generated from open gradients, with rate set by opportunity and prize; the non-metabolic ones stay (section 5) |
| E catastrophes | From an input list | Unchanged here (the T track should supply them) |
| F, G guilds | Guilds by role and habitat kind; traits derived once from the world | Guilds carry trait values that move under selection, branch and go extinct (section 6) |
| H feedback | Stand-in air box; oxygen settles at 0.22 to 0.51 | Competing oxygen sinks, fire, soil and derived weathering (sections 7, 8) |
| I signatures | Own sketch of the gases | Reads the gradients life has drawn down or built up (section 10) |
| new | | Soil as a state of land regions (section 8); biomes by region (section 9) |

Everything stays aggregate, dated, caused and labelled *plausible reconstruction*, as now.

---

## 3. The gradient catalogue

### 3.1 What counts as a gradient

For each habitat, at each step, from quantities the physics suite owns:

- **Redox pairs.** Every pair of an electron donor and an electron acceptor present in the habitat's medium, from the species the planet layer tracks (H2, CH4, CO, H2S, S, Fe2+, NH3/NH4+, organic carbon, and the acceptors O2, NO3-, Fe3+, SO4 2-, CO2, plus whatever a pack adds). The energy per reaction is the standard value corrected for the habitat's actual temperature and concentrations: ΔG = ΔG° + RT ln Q. This is the microbiologist's "redox tower" *[observed: Thauer et al. 1977]*, computed instead of memorised.
- **Light.** The photon flux reaching the habitat by band, from the star's spectrum through the atmosphere, cloud, ice and liquid. Light is a gradient because a hot star shines on a cold world.
- **Disproportionation and other single-substance reactions**, where a pack declares them.
- **Supplied oxidants and reductants**: radiolysis products on ice moons, photochemical rain-out, fresh rock meeting water. These are already habitat inputs; they enter as redox pairs with a supply rate.

Each catalogue entry holds: the reaction, ΔG per mole under local conditions, the **supply rate** of the limiting reactant (mol per year), hence **power** (watts), and the **abiotic rate**, how fast the reaction runs with no life present.

The abiotic rate matters and is new. A gradient only stands if the reaction is slow on its own. Hydrogen and oxygen mixed at 300 K sit for years; that is why something can make a living catalysing them. Where the uncatalysed reaction is already fast (hot vents above a certain temperature, photochemistry in a thin upper atmosphere), the gradient is spent before life can reach it, and the catalogue says so in words.

### 3.2 When a gradient can support life

Three tests, all pack values with sources, all reported in "Why?":

1. **Energy per step.** The reaction must yield at least a minimum quantum per turnover, or no energy-storing machinery can run on it. Default about 20 kJ per mole *[observed in syntrophic microbes: Schink 1997; lower values are argued in Hoehler 2004]*. The number is recalled, not re-read; check it at E0.
2. **Power per unit of living mass.** Power supplied, divided by the mass living on it, must exceed the cost of staying alive (maintenance), which rises with temperature. Measured maintenance powers span from about 5 × 10^-20 W per cell in deep Pacific sediment to about 5 × 10^-12 W per cell in laboratory cultures *[observed: LaRowe and Amend 2015, numbers read from the paper]*. So a trickle of power does not mean no life; it means very little life, living very slowly *[observed: Hoehler and Jørgensen 2013; Bradley et al. 2020]*.
3. **Materials.** The chemistry's required elements must be supplied (the existing Liebig step, generalised from "C:P" to the chemistry's own recipe).

### 3.3 From power to living mass

Production = power × yield. For Earth's chemistry, the mass of new cells per mole of fuel rises roughly in step with the energy the reaction releases, across very different microbes *[observed: Roden and Jin 2011; I confirmed the paper but could not read its regression, so the slope is to be taken from the paper at E1]*. The yield is therefore one number per chemistry (joules needed to build a gram, from whatever carbon or other feedstock is to hand), not one number per metabolism.

This is not a new invention. Seager, Bains and Hu (2013) estimate how much biomass a planet would need to produce a given biosignature gas using exactly this chain: Gibbs energy of the reaction, a temperature-dependent maintenance power, hence biomass *[demonstrated as a method; they state it is good to one or two orders of magnitude]*. We adopt the same honesty about precision.

### 3.4 Why believe gradients predict metabolisms

In 1977 Engelbert Broda looked at the table of reactions that release energy and noted two that no known organism used, including oxidising ammonium with nitrite. He predicted the organisms existed. The ammonium one was found in the 1990s and identified in 1999 *[observed: Broda 1977; Strous et al. 1999]*. On Earth, almost every redox pair that clears the thresholds has something living on it. That is the empirical licence for generating metabolisms from the catalogue.

### 3.5 What happens to the five listed metabolisms

They become the **fidelity contract**. With the carbon-water chemistry and Earth's, early Earth's and Europa's fixtures, the catalogue must regenerate oxygenic and anoxygenic photosynthesis, aerobic chemolithotrophy and methanogenesis as its top open gradients, and the totals must stay inside the same published ranges L1a is checked against (Field 1998, Canfield 2006, Chyba and Hand 2001). The listed model stays in the pack as `life_model: "listed/1"`; the new one is `"gradients/1"`. The default switches only when the contract passes, and the golden files are re-blessed on purpose with the reason logged.

A side benefit: two known L1a limits go away. Heterotrophs (life on imported organic carbon) are just the organic-carbon-plus-oxidant gradient, so soils, the ocean interior and the deep crust stop being "unknown". And because a gradient's supply belongs to one habitat, Europa's oxidants stop being counted twice.

---

## 4. Chemistries

### 4.1 A chemistry is data

Following Evolution Lab's own framing ("parameterized by information polymer, structural backbone, solvent, compartments, electron donors and acceptors, catalytic elements and operating conditions rather than by a single silicon-life switch"), a chemistry pack entry holds:

- solvent, and the conditions under which it is liquid (from the physics suite's phase data, not a constant);
- scaffold element and the temperature window in which its bonds are stable enough to hold information but loose enough to react;
- the element recipe (what replaces C:N:P) and build cost per gram;
- limits: solvent activity, acidity, radiation dose;
- which light bands and which redox species it can couple to;
- an **evidence tag** and its sources;
- an origin dial (section 4.3).

The budget's closure tests become questions asked of this entry. Nothing in code says "water".

The general argument that life needs a liquid, a scaffold able to form large varied molecules, and an energy source, but not specifically water and carbon, is from Bains (2004) and Benner, Ricardo and Carrigan (2004) *[plausible]*.

### 4.2 The starting set, with honest labels

| Chemistry | Where | Tag | Basis |
|---|---|---|---|
| Carbon in water | Earth-like surfaces, oceans, ice-shell oceans, deep rock, temperate cloud decks | *observed* | One example: us |
| Carbon in water-ammonia | Cold oceans inside ice moons and dwarf planets | *plausible* | Same chemistry family, lower freezing point; no example |
| Carbon in liquid hydrocarbon | Titan-like lakes | *speculative* | Energy is available (hydrogen with acetylene, McKay and Smith 2005). Membranes are the problem: a proposed nitrogen-based membrane (Stevenson et al. 2015) was later calculated not to self-assemble (Sandström and Rahm 2020) |
| Carbon in concentrated sulfuric acid | Venus-like cloud decks | *speculative* | Some building blocks are stable in it (Seager et al. 2023; Bains et al. 2021); water activity there is far below any known life's limit (Hallsworth et al. 2021, already in the pack) |
| Silicon scaffolds | Very few places | *speculative*, close to fictional in water | Petkowski, Bains and Seager (2020) find silicon cannot replace carbon in water; sulfuric acid is the least bad solvent for it |
| Carbon in supercritical CO2, and others | Deep in dense atmospheres | *speculative* | Pack-extensible; no default entry until someone supplies sources |
| Plasma, dust, nuclear or "energy" life | Stars, nebulae | *fictional* | A plasma-dust paper exists (Tsytovich et al. 2007) but describes self-organisation, not life. Fiction packs only, stamped as such |

### 4.3 Origin, per chemistry

Box C keeps its shape. Each chemistry gets its own base-rate dial, because knowing that carbon-water life started once tells us nothing about whether hydrocarbon life can. Recommended presets, set in the universe pack:

- **Known life only**: carbon-water at "Earth-like pace"; everything else off.
- **Plausible chemistry** (recommended default): carbon-water as above; water-ammonia at the same rate; the *speculative* chemistries at one hundredth of it, so they turn up as rare, flagged finds.
- **Open**: all non-fictional chemistries at the same rate. A galaxy full of strangeness, labelled as a choice.

Results that depend on a speculative chemistry carry its tag wherever they are shown, as hard-step results carry their dial today.

When two chemistries share a world they do not eat each other. Each can only use the other as raw elements. The "use to visitors" trait already distinguishes related from unrelated biochemistry; this extends it.

---

## 5. Evolution as the filling of gradients

### 5.1 Opportunity and prize

Box D today gives each capability a fixed rate per unit of opportunity. For metabolic capabilities the rate becomes:

rate = base × opportunity × prize × reach

- **opportunity** is unchanged (population, generation rate, habitat variety).
- **prize** is the untapped power of the gradient relative to the power the biosphere already runs on. A gradient worth a thousand times the current budget is found fast; one worth a thousandth may never be.
- **reach** is how many new chemical steps the metabolism needs beyond those the biosphere already has (pack data per reaction family; light-driven water splitting has a low reach, which is how oxygenic photosynthesis stays a slow step without a special case).

The waiting-time draw, the keyed threshold and the monotonic-dial guarantee are as now. A capability can still be lost and redrawn.

### 5.2 Why this handles oxygen the way Alex described

When oxygenic photosynthesis arrives, free oxygen starts to build. In the catalogue that creates a set of large new gradients at once: oxygen with organic carbon, with methane, with ferrous iron, with sulfide, with ammonium. Aerobic respiration yields roughly ten times the energy per unit of food that fermentation does (textbook figure, recalled). The prize term is therefore huge and the takers arrive quickly. Each of them consumes oxygen. So:

- oxygen is a **hazard** to the old biosphere (catastrophe severity for habitats holding guilds with no tolerance, already in box E);
- oxygen is a **boost**: feeding levels uncapped, large bodies possible, innovation faster *[inferred: Catling et al. 2005 argue oxygen is the only plentiful oxidant energetic enough for large complex life]*;
- oxygen is **contested**: the new metabolisms are sinks that grow with the gradient, which is negative feedback from life itself, alongside fire and rock (section 7).

None of the three is scripted. They are the same mechanism reading the same catalogue.

### 5.3 Hard steps, reconsidered

The life design gives origin, oxygenic photosynthesis, complex cells and language their own low-rate dials. A 2025 reassessment by Mills, Macalady, Frank and Wright argues that these steps may not be intrinsically improbable at all: each waited for the environment to open a window (enough oxygen, for instance), and humans appeared "on time" rather than by luck *[plausible, contested: it challenges the long-standing Carter and Watson view]*.

The gradient model is that argument in executable form: a step happens when its prerequisites and its prize exist. Recommendation: keep both readings available. Default to **gated** (prize and prerequisites decide, hard-step dials at 1), with the hard-step dials still there for anyone who wants a lonely universe. The complex-cells rate, currently tuned to one data point, gets a physical gate from Lane and Martin (2010): large genomes need far more power per gene than a simple cell's membrane can supply *[plausible]*. That keeps the long microbial era as a result.

### 5.4 Energy epochs name themselves

Judson (2017) divides Earth's history into five energy expansions: geochemical energy, sunlight, oxygen, flesh, fire. In this design those are not a sequence in the code. They are what the sifters would name when a new class of gradient is first tapped, on any world, in whatever order that world allows. A world may stop at the first. A world with no free oxidant never reaches "fire", and the story finder can say exactly why.

---

## 6. A simple evolutionary system: traits under selection

This is the "very simple evolutionary biology system" for V4's first pass. It is aggregate and closed-form, with no per-generation stepping.

### 6.1 Guild traits

Each guild holds up to eight trait values. The starting set (pack data, extensible):

| Trait | What selection rewards |
|---|---|
| Body or cell size | Bigger where power density allows (feeding levels, oxygen); smaller in starved habitats |
| Temperature optimum and breadth | Match to the habitat's mean and swing |
| Light response band (producers) | Evolution Lab's response function: bands over the local spectrum, with building and upkeep costs. Colour is derived from what is reflected |
| Water or solvent strategy (land) | Spend or save, against the region's water balance |
| Fast-cheap versus slow-tough tissue | The "leaf economics" axis: every land plant on Earth sits somewhere on one line from flimsy fast-growing to durable slow-growing *[observed: Wright et al. 2004]* |
| Stature | Height against light competition, gravity and the hydraulic limit |
| Dormancy | Rewarded by variability |
| Disturbance strategy | Resist, resprout or reseed after fire, frost or flood |

Today's "traits from the world" rules (largest flyer, tallest plant, colour options) become the **limits** inside which these values move. They are already built and tested, and they stay.

### 6.2 How a trait moves

Each trait has an optimum set by the habitat (a formula in a pack flow, visible in the editor). The guild's value relaxes toward it:

value(t + Δt) = optimum + (value(t) − optimum) × exp(−Δt / τ), with τ = τ0 / opportunity

So a large, warm, fast-breeding guild tracks its environment in thousands of years and a sparse cold one may take a hundred million. The gap between value and optimum is the guild's **strain**. Strain lowers its share of production. If the optimum moves faster than the guild can follow (a snowball onset, a brightening star), strain climbs past a threshold and the guild is lost: an extinction with a physical cause, recorded like any other event. If strain eases in time, that is evolutionary rescue, also recorded.

### 6.3 Branching and the lineage tree

A habitat sometimes rewards two different values of a trait better than one compromise (two light bands with a gap between them, wet and dry seasons, sun and shade). Adaptive dynamics gives the condition for a lineage to split in that case *[demonstrated in theory: Geritz et al. 1998]*. We use it as a rule: when the niche offered is wider than one guild's breadth can cover and the guild cap is not reached, the guild branches at a keyed date. Both daughters keep the parent's ancestry link. After a catastrophe frees niches, branching fills them, which is an adaptive radiation without a special case.

Resolve becomes two steps that agree with each other: a cheap **envelope** (how many guilds, which roles, expected traits) for a world nobody has opened, and the **stepped history** when someone does. The stepped history is recorded in the chronicle, so it is never recomputed differently.

### 6.4 "Seed everything, let the environment choose"

For filling a habitat with strategies there is a tested precedent. The MIT Darwin ocean model seeds many randomly drawn plankton types into a physical ocean and lets competition sort them; realistic global patterns of who lives where emerge, including the known ecotypes *[demonstrated: Follows et al. 2007]*. SSE4's version: candidate strategies per habitat come from keyed draws inside the physical limits; the ones that pay their way under the habitat's resources persist. Marsland et al. (2019), already in Evolution Lab's reference list, show that the energy flux through such a community controls how diverse and stable it ends up *[demonstrated in models]*, which gives diversity a mechanism instead of the present sketch.

This is where "simple now" and "Evolution Lab later" meet: the procedural version picks winners by a closed-form rule; the Lab version runs the competition.

---

## 7. Oxygen: competing sinks (fixes the 51 percent gap)

### 7.1 The gap

L2's air box has two oxygen sources (carbon burial, hydrogen escape) and two sinks (reducing gases, oxidative weathering). On seeds where land life appears, burial rises and nothing rises to meet it, so oxygen reaches 0.51. The package log says so plainly.

### 7.2 What the science says regulates oxygen

Nobody has a settled answer; the COPSE family of models is the standard working reference *[inferred: Lenton, Daines and Mills 2018]*. The feedbacks in the literature, and how each enters this design:

| Sink or brake | Mechanism | Depends on | Source |
|---|---|---|---|
| **Aerobic decomposers** | More oxygen in bottom water and soil means more dead matter is eaten before it can be buried, so the burial source shrinks | The aerobic-respiration capability (section 5.2), and decomposer guild biomass | Betts and Holland 1991, as summarised in Lenton and Watson 2000 |
| **Fire** | Burns land biomass before it can be buried; moves phosphorus from land to sea | Oxygen level, fuel (land biomass), moisture | Kump 1988 via Lenton and Watson 2000; Belcher et al. 2010 |
| **Weathering suppression** | High oxygen (through fire and plant stress) reduces the boost land life gives to rock weathering, which cuts the phosphorus supply, which cuts production and burial | Land life's weathering factor (section 8) | Lenton and Watson 2000, who find this "the most effective oxygen regulation of all the mechanisms considered", holding oxygen at 19 to 21 percent over 40 million years in their model |
| **Phosphorus and ocean oxygen** | Anoxic sea floors release phosphorus, raising production and burial; oxygenated ones lock it away | Ocean oxygen, which the air box needs as a second reservoir | Van Cappellen and Ingall 1996 |
| **Oxidative weathering** | Already built, scaling with the square root of oxygen | Exposed old organic carbon | Daines, Mills and Lenton 2017 |
| **Oxygen-using chemotrophs** | Methane, iron, sulfide and ammonium oxidisers | Their gradients (section 3) | generated |

### 7.3 Fire as physics

Fire is not life, but it only exists where life has made both the fuel and the oxidant. Burn experiments at controlled oxygen levels give the curve *[observed, numbers read from the paper: Belcher et al. 2010]*: combustion does not sustain below 16 percent oxygen; fire is greatly suppressed below 18.5 percent; it rises steeply between 19 and 22 percent; above that most ignitions spread. They could not find an upper limit to the fire window.

In the design, fire is a flux in the land regions: burned fraction per year = ignition supply (lightning from the weather track, volcanism) × the oxygen curve × a moisture term × fuel load. It removes biomass, lowers burial, makes a little charcoal (which is itself buried carbon), and sets the disturbance trait's reward in section 6. It is also a surface hazard for the S5 hazard work and a visible thing for the weather track's smoke.

One caution: the curve was measured at Earth's air pressure with nitrogen as the filler gas. On a world with different pressure or a different filler, what matters is closer to oxygen's partial pressure and how much inert gas there is to soak up heat. The block will take both and be labelled *extrapolated* outside Earth-like air.

### 7.4 The check

- Earth fixture, all seeds: oxygen settles between 15 and 30 percent after land life, and the level is set by the balance, as the oxygenation date already is.
- Switch fire off: oxygen runs high (today's behaviour reappears, now as a what-if).
- Remove the aerobic-respiration capability: burial stays efficient and oxygen runs high.
- A water world with no land: no fire and no land weathering feedback, so ocean feedbacks alone decide. The result is recorded, whatever it is.
- The reduced world still never oxygenates.
- The ledger still balances to the same tolerance.

Higher-oxygen worlds remain possible where fuel is wet or scarce. That is a feature: a 30 percent world with giant flyers and catastrophic fire seasons is a real place a GM can find.

---

## 8. Soil, and the life that makes it

Alex, 19 Nov: "Fungus and soil! SOIL is SO important! They are great terraforming duo." This section makes soil a first-class product of life plus weathering.

### 8.1 Roles by function, not by kingdom

Two guild roles are defined by what they do to matter, on any world, in any chemistry:

- **Weatherers**: life that gets its scarce elements (on Earth phosphorus, potassium, iron) by dissolving rock. Paid in nutrients, sometimes also in energy (iron and sulfur oxidisers eat the rock's own gradient). On Earth: lichens, mosses, fungi, roots with fungal partners, and plain microbes before any of them.
- **Decomposers**: life on the dead-matter-plus-oxidant gradient. They close the nutrient loop. How completely they close it is the "reuse before burial" number that L1a currently takes as a constant per habitat kind; it becomes their output.

The expectation that most living land surfaces would evolve both is a direct consequence of sections 3 and 5: scarce elements locked in rock are a standing prize, and dead matter beside an oxidant is a standing gradient *[inferred]*. Nothing says they must look like fungi. The life design's box D has "land colonisation" as one capability; it should be read as "a weatherer guild holds a land habitat".

### 8.2 Soil is state, not a type

Evolution Lab's design already says it: "Soil is not an engine type: it is a useful abstraction over a resource/habitat system." So a land region carries five soil quantities, and "living soil" is a derived marker over them:

| Quantity | Grows by | Shrinks by |
|---|---|---|
| Loose depth (regolith), m | Weathering front advancing | Erosion (slope, runoff, wind, ice) |
| Fine fraction (clay and silt) | Chemical weathering | Leaching, erosion |
| Organic carbon, kg per m² | Dead matter not yet decomposed | Decomposition, fire, erosion |
| Nutrient stock (limiting elements) | Weathering release, fixation, dust | Uptake, leaching, burial |
| Water-holding capacity | Derived from depth, fines and organic carbon | |

These follow the five classical soil-forming factors: climate, organisms, relief, parent rock and time *[observed framework: Jenny 1941]*. Climate and relief come from the surface and weather tracks, parent rock from the surface model, organisms from the guilds here, and time from the chronicle. A soil is therefore deterministic from the world's history, like its height map.

### 8.3 The feedback loop

Soil holds water and nutrients, so production rises; more production means more weatherers and more dead matter, so more soil. That loop is why soil and its makers are a terraforming pair, and it has planetary reach:

- **Weathering and climate.** L2 multiplies silicate weathering by a fixed 4 when life holds land. It becomes a derived factor from weatherer biomass and grade. The published range is wide: Schwartzman and Volk (1989) argued for 10 to 1000 times *[plausible, their upper values are disputed]*; a process model of early moss-and-lichen cover alone found weathering comparable to today's *[demonstrated in a model: Porada et al. 2016]*; and the arrival of the first land plants has been argued to have drawn carbon dioxide down far enough to cause the late Ordovician ice ages *[plausible: Lenton et al. 2012]*. So a world can freeze itself by growing soil. The model must allow it and must not require it.
- **Phosphorus and oxygen.** Weathering is the phosphorus tap. Fungal partners that mine phosphorus for plants have been modelled as a control on the whole Palaeozoic rise in oxygen and fall in carbon dioxide *[demonstrated in a model: Mills, Batterman and Field 2018]*. This is the same lever as the third row of the table in 7.2.
- **Carbon store.** Soil organic carbon is a reservoir in the ledger alongside air, ocean and rock.
- **Surface look.** Soil cover and its colour (iron state, organic content) feed the surface track's ground material layer; bare rock, crust, thin soil and deep soil look different from orbit and underfoot.
- **Strata, later.** Buried soils, coal and peat are where the subsurface-layers work, when it starts, gets its biological beds from. For now only totals and located deposits are kept.

Evolution Lab's reference list already has five soil sources (its `REF-SOIL-001` to `005`: Zhu et al., Bonneville et al., Smits et al., Dümig et al., Rombouts et al.) covering fungal attack on minerals and soil formation from biological crusts. They carry over as the pack's citations; I have not re-read them.

### 8.4 A GM tool that falls out

"What if weatherers and decomposers are seeded on this barren, wet, rocky world?" is a look-ahead that returns a soil timeline, the climate consequence and the date the first tall producers could follow. It is a what-if over existing mechanisms, not a terraforming subsystem.

---

## 9. Biomes that form by themselves, on any body

### 9.1 Conditions, not kinds

Habitat rules read a body's shells and regions and ask: is there a medium, is a chemistry's solvent liquid in it, is it inside that chemistry's window, has it persisted, are gradients and materials supplied? They never ask whether the body is a planet. So the same rules open a temperate cloud layer on a cool brown dwarf *[speculative: Yates et al. 2017; Lingam and Loeb 2019]*, the ocean of a rogue planet, or a short-lived wet interior in a young, radioactively heated planetesimal, and close the photosphere of a star with a reason ("no molecule of any installed chemistry is stable at 5800 K"). This is the accommodation the one-body ruling needs from the life suite: `WorldPhysics` becomes a set of conditions supplied by whatever body it is, and no life code branches on body kind.

### 9.2 How a biome arises

Per land region (and per ocean province), at an epoch, the inputs are all things other tracks already produce: temperature and its seasonal swing, water balance, light at the ground by band, soil (section 8), fire and frost frequency, gravity and air density.

The producer guilds present on that world each have a trait set (section 6). In each region the question is which strategies pay their way there and which one ends up on top:

1. Each candidate's net gain is computed from light captured, water spent, nutrient needed, tissue cost and losses to frost, drought and fire. Eco-evolutionary optimality, the idea that plants behave as if they maximise gain minus cost, now predicts real leaf-level behaviour well enough to be put into land-surface models *[demonstrated: Harrison et al. 2021]*.
2. The tallest strategy that can still pay for its height takes the light; others live under or between.
3. The region's **cover** is the result: canopy height, how closed it is, standing mass, production, how seasonal.

A **biome** is a label the class atlas derives from those cover numbers. It is never stored as the only record.

### 9.3 "Like Earth, not like sci-fi"

Sci-fi gives a planet one biome. Earth gives one planet a dozen, arranged by latitude, altitude, distance from the sea and rain shadow. Since the inputs vary by region, so will the result on any world with varied climate.

The Earth check: with Earth's climate fields, soil and today's producer guilds, the regional result should place the broad cover types (closed forest, open woodland and savannah, grassland, scrub and desert, tundra, ice) where they are. Simple climate-only schemes have managed this for a long time (Holdridge 1947), and the first physiology-based global model, BIOME (Prentice et al. 1992), is the direct comparison. Target, to be confirmed against a named land-cover dataset at build: the right broad type over at least 70 percent of land area.

A second check that is easy to miss: fire. Without fire, closed forest would cover roughly twice the area it does; much of the world's savannah and grassland exists because it burns *[demonstrated in a model: Bond, Woodward and Midgley 2005]*. So Earth's map cannot be right unless section 7's fire is feeding the disturbance term here. The oxygen fix and the biomes are one piece of work.

### 9.4 Naming alien biomes

Names are structural first: "closed tall canopy", "open canopy over ground cover", "low mat", "crust", "bare". Where the numbers also match an Earth biome's definition, the Earth name is offered as the plain-language layer ("rainforest"). Colour comes from the producers' light-response trait, so a red-dwarf world's "forest" is dark and nobody had to say so. Packs can add vocabulary. Ocean provinces, ice-shell oceans and vent fields get the same treatment with their own structural names.

### 9.5 Hand-offs

- Surface track: the `biome` tile layer already reserved in the surface design is filled from the regional cover (a pure function of region state and tile key, so any device can build any tile).
- Classification track: biome classes are an atlas like any other, editable.
- Weather track: cover sets albedo, roughness and evaporation back to the weather rules.
- Hazards: fire season, by region.

---

## 10. Signatures, briefly

A biosphere is detectable because it holds gradients open that chemistry alone would close: oxygen beside methane is the classic. Krissansen-Totton and colleagues measured this as available Gibbs energy in a planet's atmosphere and ocean and showed how it changed over Earth's history *[demonstrated: Krissansen-Totton, Bergsman and Catling 2016; Krissansen-Totton, Olson and Catling 2018]*. Because this design already computes every gradient, box I can read its "atmosphere out of balance" signal from the same catalogue the life is eating from, with the abiotic rate giving the false-positive side for free. This ties straight into Alex's long-range survey idea (spectra and transits of distant systems), which is another thread's job; the life suite just needs to publish the numbers.

---

## 11. The road to Evolution Lab

### 11.1 Where Evolution Lab actually is

I read its source and design on Alex's PC (`ARCHITECTURE.md`, the high-level design, the engine map, the implementation direction of 5 Sep 2026, and the core, mechanisms and Alien Lake code). Version 0.11.0 has: a microbial reference with four authored guilds and a closed material ledger; a draft "Alien Lake" with three connected habitats, generic response functions over a numeric field (first used for pigments over a spectrum), a seeded daughter variant, and an exact wrapper that hides and re-expands a sub-community without saving any computation. Its own README says the rates are in model units and uncalibrated, and that open-ended mutation, a generic evolving network, compute-saving reduction and just-in-time content are not built. Its direction document makes two sessions the gate for anything wider: Session 1, a feedback loop that can reject an attractive but wrong model; Session 2, a reduction that really does less work at stated fidelity.

So Evolution Lab is the workbench, as ruled, and it is early. Nothing in sections 3 to 9 waits for it.

### 11.2 Three horizons

**Horizon 1: procedural with selection (this document, packages E0 to E8).** SSE4-native, in Rust, closed-form. Uses the Lab's vocabulary: guild, four lenses, response function, habitat graph, material ledger, evidence tags, markers derived from state.

**Horizon 2: simulated windows (life design L3).** For a chosen place and interval (a lake through one bad millennium, a continent through a recovery) the Lab's real mechanisms run: populations competing for resources with every gram accounted, response functions paying their costs, variants replacing one another. The procedural model stays as the coarse model either side of the window. Each mechanism enters with a fidelity contract: the window's summary must agree with what the procedural box would have said on named observables, or the disagreement is reported and the procedural box is what gets fixed. Needs the Lab's Session 1 and Session 2 results first. Runs on the GM's machine as a heavy block; results travel as data.

**Appearance ("what they look like"), a later version.** Once function is solid, form can be derived from it: a guild's size, medium, gravity, power budget and senses constrain a body plan, and Evolution Lab's design already treats pictures and 3D forms as artefacts derived from lineage facts and an inherited morphology recipe, never as biological state. That work reuses the trait values and lineage tree built here, so nothing in V4 has to be redone for it. It is deliberately not in the package list below.

**Horizon 3: open evolution (L4).** The pieces the Lab's roadmap names and has not built: mutation supply that invents new response functions; transfer of capabilities between lineages; stable partnerships wrapping into a higher-order individual, which would replace the "complex cells" capability with a mechanism; data-defined alien chemistries tested in the Lab's flask before SSE4 trusts them; and two biospheres of different origin meeting. This is research, with no date.

### 11.3 What each side gives the other

- **SSE4 gives the Lab its planet.** The Lab already declares what it needs from a provider (`exobiology/provider-requirements`: spectrum at the surface, temperature, pressure, liquid availability, solvent activity, chemical energy and so on) and has planned dataset versions v2 (forcing through time), v3 (regions) and v4 (fluxes both ways). SSE4's habitat profile is nearly that list. Package E9 writes the exporter, so any SSE4 habitat can be opened in the Lab as a pinned, hashed fixture.
- **The Lab gives SSE4 its numbers.** The yields, costs and time constants in sections 3 and 6 are exactly the kind of thing its tuning harness and held-out seeds are for. A promoted Lab result replaces a pack value, with its evidence attached.
- **The gradient catalogue is shared ground.** It is the Lab's "materials + usable gradient + capabilities + environment" transformation, computed from real chemistry. I recommend the Lab adopt the same catalogue format, so a metabolism means the same thing in both.

---

## 12. What is possible, and what is not

| Claim the system can make | Standing | Why |
|---|---|---|
| How much energy a habitat offers life, by source | **Strict physics** | Thermodynamics and measured fluxes |
| Which ways of living are open or closed there, and why | **Strict**, given a chemistry | Thresholds are measured for Earth life; for other chemistries they are pack assumptions |
| Upper bound on living mass, feeding levels, body power | **Strict upper bounds**, good to an order of magnitude | Section 3; Seager et al. give the same precision |
| What life does to air, rock, soil and climate | **Plausible reconstruction** | Mass balance is strict; rates are tuned to one planet |
| When things happened | **Plausible reconstruction** | Waiting-time draws against real opportunity; dials visible |
| Which biome is where | **Plausible reconstruction**, checkable on Earth | Section 9 |
| Traits: size, colour, pace, strategy | **Plausible**, inside strict limits | Convergence is real (eyes, wings, leaf economics) but not guaranteed |
| Life in other solvents or on other scaffolds | **Speculative**, labelled | No example exists; chemistry arguments only |
| How likely life is at all | **Unknown**: a dial | One data point |

Honestly out of reach, now and for the foreseeable future:

- **Actual organisms.** No anatomy, behaviour or species from first principles. Guild sketches and Minerva's descriptions are reconstructions consistent with the numbers, not predictions.
- **Molecules.** Whether a given alien biochemistry really works. We check thermodynamics and bulk stability; we cannot simulate a cell's chemistry.
- **Genomes and individuals.** By design.
- **Minds.** The capability graph can say when the prerequisites exist. What a mind does is authored.
- **Life from non-chemical gradients.** There is no accepted mechanism for a self-copying system that lives directly off heat flow, magnetic fields or gravity at small scale, though all of those can make chemical gradients that life then uses. Plasma and "energy" beings are fiction packs.
- **Calibration beyond Earth.** Every rate is tuned to one biosphere. The model is strict about what cannot happen and modest about what did.

---

## 13. Determinism and cost

**Determinism** (the repo's standing rules apply unchanged):

- Fixed step sizes (the history's 5 Myr; regional steps on a fixed grid of the surface track's regions). No adaptive stepping inside canonical results.
- Every draw is keyed by address: habitat, gradient id, guild id, trait name, attempt number. Adding a gradient or a chemistry never shifts another's draws.
- Sorted iteration everywhere; no hash maps. Gradient ids are canonical strings built from sorted species names.
- All maths through `sse_core::math`. ΔG needs only `ln`; relaxation needs `exp`; both exist.
- Balances that depend on their own outcome (oxygen with fire) use a fixed number of bisection steps, as the air box already does.
- Golden files change once per package, on purpose, with the reason in the package log. The `listed/1` model keeps its goldens untouched for as long as it exists.

**Cost**, with hard caps so weak devices are safe:

| Depth | Work | Cap | Class |
|---|---|---|---|
| Envelope for an unopened world | Catalogue once per habitat with expected values | 32 gradients per habitat, 3 chemistries | Tens of microseconds; every device, every body |
| World history | 900 steps × habitats × open gradients, plus 40 guilds × 8 traits | as left | About today's L1b cost times three; milliseconds to tens of milliseconds; every device |
| Soil and biomes | Per region, on demand, cached by epoch | 16 candidate strategies per region | Per region about the cost of one surface tile; phones compute the regions in view or receive them as data |
| Simulated window | Evolution Lab mechanisms | GM machine only | Heavy block; results as data |

Where a device does less (regions not yet resolved, windows unavailable), the edge is shown with its reason, per the standing ruling.

---

## 14. Build packages (track E)

Each lands clean with its checks before the next starts. All are kernel-track work in `sse-life` and `packs/base-life` unless noted.

| Id | Delivers | Needs | Verified by |
|---|---|---|---|
| **E3** (do first) | Competing oxygen sinks in the feedback box: aerobic decomposers lower burial, fire curve, weathering suppression, a second (ocean) oxygen reservoir with the phosphorus feedback | L2 | Section 7.4: Earth 15 to 30 percent on all seeds; the four what-ifs behave; ledger balances; reduced world unchanged |
| **E0** | Chemistry pack schema with evidence tags; thermodynamic data table for the tracked species with sources; ΔG block; removal of `solvent == "water"` from code | L1a | Twelve reference reactions within 5 percent of published values; redox tower order reproduced; carbon-water closures give identical results to today |
| **E1** | Gradient catalogue, abiotic rates, the three tests, yield; `life_model: "gradients/1"` beside `"listed/1"` | E0 | Fidelity contract of section 3.5; heterotroph habitats no longer unknown; Europa's supply counted once; 10,000 random habitats never exceed a cap |
| **E2** | Metabolic capabilities generated from open gradients; opportunity × prize × reach; gated default with hard-step dials kept; power gate on complex cells | E1, L1b | Earth outline in most seeds at least as often as today (59 percent); dials still monotonic; aerobic respiration follows oxygen on every seed that oxygenates |
| **E4** | Soil state per land region; weatherer and decomposer roles; derived weathering factor and nutrient reuse; soil carbon in the ledger | E3, P4 regions | Barren-to-soil timeline on a fixture world is monotonic and bounded; Earth's soil carbon within a factor of three of the published total; a seed exists where early land life cools the world |
| **E5** | Guild traits, relaxation, strain, extinction and rescue, branching, lineage tree in the chronicle | L1c, E2 | Limits never broken across 10,000 worlds; a step change in climate gives extinctions that scale with its speed; order-independent; envelope agrees with summed stepped histories within tolerance |
| **E6** | Regional cover and biomes; class-atlas entries; `biome` tile layer hand-off; fire season as a hazard | E4, E5, S track climate fields | Earth broad-type agreement at least 70 percent of land area; no-fire what-if expands closed canopy; a locked red-dwarf fixture gives a terminator ring of cover |
| **E7** | Further chemistries (water-ammonia, hydrocarbon, sulfuric acid) with origin dials and presets; condition-keyed habitat rules for any body | E1, the one-body work | Titan's lakes open only with the hydrocarbon chemistry enabled, tagged speculative; a cool brown-dwarf fixture gets a cloud-deck habitat; a star's photosphere is closed with a reason; carbon-water results unchanged |
| **E8** | Disequilibrium measure from the catalogue feeding box I | E1, L1d | Earth through time shows the published pattern (oxygen-methane today, a different pair before oxygen); a scripted abiotic-oxygen world reads as a false positive |
| **E9** | Exporter from an SSE4 habitat to Evolution Lab's provider fixture; socket for one simulated window with a fidelity contract | E1, K10, the Lab's Sessions 1 and 2 | An SSE4 lake opens in the Lab with a matching hash; one window's summary agrees with the procedural box on named observables or the difference is reported |
| **UE1** (UI track) | Life screen additions: "Energy" panel (what the world offers, who uses it, what is untapped and why), soil in the cutaway, biome map layer | E1, E4, E6, U7 | Screens against live data; every bar answers "Why?" |

E3 goes first because it needs nothing new and closes a known gap. E0 to E2 are the core. E4 to E6 are what Alex will see. E7 is small once E0 and E1 exist. E9 waits on the Lab.

Under the "they exist" scope, the V4 first stage is E3, E0, E1, E2, E4, E6 and E8, with E5 cut down to the functional traits E6 needs (no lineage art, no body plans). E7 and E9 can follow at any time. Nothing here builds appearance.

---

## 15. Open questions, with my recommendation on each

1. **Replace the listed metabolisms, or run beside them?** Beside them behind a pack switch until the fidelity contract passes, then make gradients the default. Keeps every current golden safe.
2. **How open should the default universe be to exotic chemistry?** "Plausible chemistry" (section 4.3): the real unknowns turn up as rare, flagged finds. Alex wants non-carbon life to be able to arise; this lets it without filling every Titan with fish.
3. **Hard steps or gates?** Gated by default, dials kept. It suits "life from gradients", it is the more recent argument, and the dials still let a setting be lonely.
4. **Fire away from Earth-like air.** Use oxygen partial pressure with an inert-gas term, label it extrapolated. The alternative, refusing to compute fire elsewhere, would bring the oxygen gap back on exactly the worlds a GM finds interesting.
5. **Biome names.** Structural names always, Earth names as the plain-language layer where the numbers match, packs free to add. Avoids "jungle planet" while still letting a rainforest be called one.
6. **How much soil to store.** The five numbers per region now. Layers and buried soils when the subsurface work starts, as already ruled for rock strata.
7. **Wait for Evolution Lab?** No. E0 to E8 are SSE4-native. Only E9 waits, and only on the Lab's own two gates.
8. **Should the Lab and SSE4 share the catalogue format?** Yes. It is the one place the two could quietly drift apart.
9. **Who checks the numbers?** Alex mentioned asking geophysics friends about rock. The equivalent here is a geochemist or microbiologist looking over the gradient thresholds and yields (sections 3.2 and 3.3) and an ecologist the strategy costs (9.2). The "Energy" panel is designed to be the thing they can look at and argue with.
10. **Does the complex-cells power gate belong in the default pack?** Yes, tagged *plausible*, with the old tuned rate kept as a fallback dial. It removes a number tuned to one data point.

---

## 16. Sources and how far each was checked

On 19 Nov 2026 every source below was found by web search and its title, authors and venue confirmed. Where I read the page itself, the entry says **read**, and numbers quoted from it in this document can be trusted. Other numbers in this document (the 20 kJ per mole quantum, the tenfold aerobic advantage) are recalled and are marked as such where they appear; they must be confirmed when the pack entry is written. DOIs are given only where I saw them. Sources already cited by the built packs (Kiang 2007, Gillooly 2001, Field 1998, Canfield 2006, Chyba and Hand 2001, Hallsworth 2021) are not repeated.

**Energy and thresholds**
- Thauer, Jungermann and Decker 1977, "Energy conservation in chemotrophic anaerobic bacteria", Bacteriological Reviews. https://pmc.ncbi.nlm.nih.gov/articles/PMC413997
- Schink 1997, "Energetics of syntrophic cooperation in methanogenic degradation", Microbiology and Molecular Biology Reviews. https://pmc.ncbi.nlm.nih.gov/articles/PMC232610
- Hoehler 2004, "Biological energy requirements as quantitative boundary conditions for life in the subsurface", Geobiology. https://www.scilit.net/publications/9add82f897d687ce5fbcbf93c6764e8d
- Hoehler and Jørgensen 2013, "Microbial life under extreme energy limitation", Nature Reviews Microbiology. doi:10.1038/nrmicro2939
- LaRowe and Amend 2015, "Power limits for microbial life", Frontiers in Microbiology 6:718. doi:10.3389/fmicb.2015.00718 (**read**)
- Bradley, Amend and LaRowe 2018, "Bioenergetic controls on microbial ecophysiology in marine sediments", Frontiers in Microbiology 9:180. doi:10.3389/fmicb.2018.00180 (**read**, summary only)
- Bradley et al. 2020, "Widespread energy limitation to life in global subseafloor sediments", Science Advances. doi:10.1126/sciadv.aba0697
- Roden and Jin 2011, "Thermodynamics of microbial growth coupled to metabolism of glucose, ethanol, short-chain organic acids, and hydrogen", Applied and Environmental Microbiology. https://pmc.ncbi.nlm.nih.gov/articles/PMC3067288 (page blocked; regression not read)
- Seager, Bains and Hu 2013, "A biomass-based model to estimate the plausibility of exoplanet biosignature gases", Astrophysical Journal. arXiv:1309.6014 (**read**, in part)
- Broda 1977, "Two kinds of lithotrophs missing in nature", Zeitschrift für allgemeine Mikrobiologie. https://typeset.io/papers/two-kinds-of-lithotrophs-missing-in-nature-4k8350hbft
- Strous et al. 1999, "Missing lithotroph identified as new planctomycete", Nature 400. doi:10.1038/22749

**Oxygen, fire and regulation**
- Lenton and Watson 2000, "Redfield revisited: 2. What regulates the oxygen content of the atmosphere?", Global Biogeochemical Cycles 14:249. doi:10.1029/1999GB900076 (**read**, abstract; Kump 1988, Betts and Holland 1991 and Van Cappellen and Ingall are cited through it)
- Belcher, Yearsley, Hadden, McElwain and Rein 2010, "Baseline intrinsic flammability of Earth's ecosystems estimated from paleoatmospheric oxygen over the past 350 million years", PNAS 107:22448. doi:10.1073/pnas.1011974107 (**read**)
- Belcher and McElwain 2008, "Limits for combustion in low O2 redefine paleoatmospheric predictions for the Mesozoic", Science. https://physicstoday.aip.org/news/new-limits-on-prehistoric-oxygen-levels
- Van Cappellen and Ingall 1996, "Redox stabilization of the atmosphere and oceans by phosphorus-limited marine productivity", Science 271. https://scilit.com/publications/d59775cd37efc7d41f17732a93731679
- Daines, Mills and Lenton 2017, "Atmospheric oxygen regulation at low Proterozoic levels by incomplete oxidative weathering of sedimentary organic carbon", Nature Communications. https://eprints.whiterose.ac.uk/112190/
- Lenton, Daines and Mills 2018, "COPSE reloaded: an improved model of biogeochemical cycling over Phanerozoic time", Earth-Science Reviews. https://eprints.whiterose.ac.uk/125894
- Catling, Glein, Zahnle and McKay 2005, "Why O2 is required by complex life on habitable planets and the concept of planetary 'oxygenation time'", Astrobiology. https://scilit.com/publications/c07ffa637132725776cc127d422d7196

**Other chemistries and places**
- Bains 2004, "Many chemistries could be used to build living systems", Astrobiology. https://scilit.com/publications/d0e3c8b15c8d9e3ca374c8effec36f77
- Benner, Ricardo and Carrigan 2004, "Is there a common chemical model for life in the universe?", Current Opinion in Chemical Biology. https://www.ffame.org/pubs/Is%20there%20a%20common%20chemical%20model%20for%20life%20in%20the%20universe%3F.pdf
- Petkowski, Bains and Seager 2020, "On the potential of silicon as a building block for life", Life 10:84. https://pmc.ncbi.nlm.nih.gov/articles/PMC7345352
- McKay and Smith 2005, "Possibilities for methanogenic life in liquid methane on the surface of Titan", Icarus. https://zenodo.org/records/1259025
- Stevenson, Lunine and Clancy 2015, "Membrane alternatives in worlds without oxygen: creation of an azotosome", Science Advances. https://pmc.ncbi.nlm.nih.gov/articles/PMC4644080
- Sandström and Rahm 2020, "Can polarity-inverted membranes self-assemble on Titan?", Science Advances. https://pmc.ncbi.nlm.nih.gov/articles/PMC6981084
- Bains, Petkowski, Zhan and Seager 2021, "Evaluating alternatives to water as solvents for life: the example of sulfuric acid", Life 11:400. doi:10.3390/life11050400
- Seager et al. 2023, "Stability of nucleic acid bases in concentrated sulfuric acid: implications for the habitability of Venus' clouds", PNAS. https://pmc.ncbi.nlm.nih.gov/articles/PMC10288544
- Yates, Palmer, Biller and Cockell 2017, "Atmospheric habitable zones in Y dwarf atmospheres", Astrophysical Journal. arXiv:1611.09074
- Lingam and Loeb 2019, "Brown dwarf atmospheres as the potentially most detectable and abundant sites for life", Astrophysical Journal. arXiv:1905.11410
- Tsytovich et al. 2007, "From plasma crystals and helical structures towards inorganic living matter", New Journal of Physics. https://phys.org/news105869123.html (news report of the paper)

**Evolution and its pace**
- Mills, Macalady, Frank and Wright 2025, "A reassessment of the 'hard-steps' model for the evolution of intelligent life", Science Advances. doi:10.1126/sciadv.ads5698 (**read**, abstract)
- Lane and Martin 2010, "The energetics of genome complexity", Nature 467. doi:10.1038/nature09486
- Judson 2017, "The energy expansions of evolution", Nature Ecology and Evolution. https://eprints.gla.ac.uk/146683
- Geritz, Kisdi, Meszéna and Metz 1998, "Evolutionarily singular strategies and the adaptive growth and branching of the evolutionary tree", Evolutionary Ecology. https://pure.iiasa.ac.at/5344
- Follows, Dutkiewicz, Grant and Chisholm 2007, "Emergent biogeography of microbial communities in a model ocean", Science. https://darwinproject.mit.edu/emergent-biogeography-of-microbial-communities-in-a-model-ocean/
- Marsland et al. 2019, "Available energy fluxes drive a transition in the diversity, stability, and functional structure of microbial communities", PLOS Computational Biology. https://pmc.ncbi.nlm.nih.gov/articles/PMC6386421

**Soil, weathering and biomes**
- Jenny 1941, *Factors of Soil Formation: A System of Quantitative Pedology*. https://en.wikipedia.org/wiki/Hans_Jenny_(pedologist)
- Schwartzman and Volk 1989, "Biotic enhancement of weathering and the habitability of Earth", Nature 340:457. doi:10.1038/340457a0
- Porada et al. 2016, "High potential for weathering and climate effects of non-vascular vegetation in the Late Ordovician", Nature Communications. https://pmc.ncbi.nlm.nih.gov/articles/PMC4941054
- Lenton, Crouch, Johnson, Pires and Dolan 2012, "First plants cooled the Ordovician", Nature Geoscience. https://ueaeprints.uea.ac.uk/43669/
- Mills, Batterman and Field 2018, "Nutrient acquisition by symbiotic fungi governs Palaeozoic climate transition", Philosophical Transactions of the Royal Society B. https://pmc.ncbi.nlm.nih.gov/articles/PMC5745338
- Evolution Lab `REF-SOIL-001` to `005` (in `evolution-lab-design/docs/REFERENCES.md`, with DOIs; not re-read)
- Holdridge 1947, "Determination of world plant formations from simple climatic data", Science 105. https://www.scilit.net/publications/b09409977906651d88a0373bfb201b97
- Prentice et al. 1992, "A global biome model based on plant physiology and dominance, soil properties and climate", Journal of Biogeography. https://hal-amu.archives-ouvertes.fr/hal-01788308
- Bond, Woodward and Midgley 2005, "The global distribution of ecosystems in a world without fire", New Phytologist. https://pesquisa.bvsalud.org/portal/resource/pt/mdl-15720663
- Wright et al. 2004, "The worldwide leaf economics spectrum", Nature 428. doi:10.1038/nature02403
- Harrison et al. 2021, "Eco-evolutionary optimality as a means to improve vegetation and land-surface models", New Phytologist. doi:10.1111/nph.17558

**Signatures**
- Krissansen-Totton, Bergsman and Catling 2016, "On detecting biospheres from chemical thermodynamic disequilibrium in planetary atmospheres", Astrobiology. arXiv:1503.08249
- Krissansen-Totton, Olson and Catling 2018, "Disequilibrium biosignatures over Earth history and implications for detecting exoplanet life", Science Advances. arXiv:1801.08211

**Evolution Lab** (read on Alex's PC, 19 Nov 2026): `evolution-lab/README.md`, `docs/HIGH_LEVEL_DESIGN.md`, `docs/ENGINE_MAP.md`, `docs/IMPLEMENTATION_DIRECTION.md`, `src/lib/core/types.ts`, `src/lib/mechanisms/responseFunction.ts`; `evolution-lab-design/docs/SOFT_CODED_CONCEPTS.md`, `docs/REFERENCES.md`.
