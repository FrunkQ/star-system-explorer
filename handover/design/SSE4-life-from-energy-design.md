# SSE4: life's spread, population and pace from the energy it can get (track LE)

Status: **SHAPED BY ALEX 26 Nov 2026 23:50 UTC** (section 0): 4.0 builds a broad-brush baseline, the detail and all tech go in the 4.1 bucket. Question 5 of section 10 is still open. Design only; nothing here is built by this note. Packages for builders are in section 9; the 4.1 bucket is section 9A. LE is a code only, never shown in the product.

Read with: `SSE4-life-design.md` (boxes B, C, D, E), `SSE4-evolution-design.md` (sections 3 and 5, the gradient catalogue and opportunity times prize), `status/names-and-life.md` (L3), `status/kernel.md` (E1, E2, E4 to E6 built as opt-in), the Energy flow view (memory `sse4-heat-flow-view`, PR 670, track HV).

Alex's brief (26 Nov 2026):

- 23:18 (quoted by the coordinator): "if negative it is doubtful life could happen... use this energy balance to determine how fast life could spread - use it to determine max population".
- 23:19: "its another useful feed intp that - energy availoable on planet wilkl cotrol teh pace of develoopment/evolution as it is the "first step" of teh energy gradient".

Figures marked *unverified* are recalled, not re-read this session: the Consensus paper search is out until 1 Dec 2026. Each must be confirmed against the paper when its pack entry is written.

---

## 0. Alex's answers (26 Nov 2026, 23:50 UTC), and what they change

His words, verbatim:

1. "depends what people eat... people/creatures/trees/insects - Tech can "change the earth"... people wouldn't as just biomass available. The life system should be able to distribute that energy to the various types of life. I see it more as a "speed control". Needs to be better."
2. (ditto mark: read as "same as 1")
3. "4.1. is where we develop life HARD to get xenobiology better - so this is maybe "half of this we do now" - the broad brush strokes - the details we can focus on later!"
4. "tech we held off to 4.1 didnt we? The tech impact to planets and beyond was gonna be held. Dump all teh good ideas in that bucket and use this to get the "baseline" to run all this in place. Knowing what will come will heklp us bound that baseline"

Yes: tech was held to 4.1 (ruling of 23 Nov 17:49 in `SSE4-future-tech-design.md`: "4.1 alien life will have to have culture and tech as part of life").

What changes, as read by this thread:

- **No headcount query in 4.0.** A single "how many people" figure is wrong because it depends on what they eat, and tech changes the world. Instead the life system **shares the captured energy out among the kinds of life by what each eats** (trees and other producers, insects, grazers, predators, decomposers), and reports each kind's share. People are one more kind with a diet, added in 4.1 with tech. Sections 5.3 and 5.4 move to the 4.1 bucket; question 2 (ceiling or likely) goes with them.
- **Energy is above all a speed control.** The surplus sets how fast life spreads, how fast it evolves and how fast it recovers, all in one place. Section 1 point 6 and section 7 are read that way: one "speed" read-out per world, which 4.1 refines.
- **4.0 is the baseline, broad brush.** Keep-alive, the energy ledger, the shares by kind of life, spread in the deep-time history and recovery clocked by pace go in now, with simple cited tables where the detail is not worth it yet. The fine models (two-regime pace, spread on the campaign clock, transfer between worlds, xenobiology per chemistry, people and tech) go in the 4.1 bucket (section 9A), with the seams the baseline leaves for each.
- **Question 4** (replace the 0.25 pace power) is detail, so it moves to 4.1; the baseline keeps 0.25, already cited and to be entered in the fudge register. **Question 6** (outside power) is tech, so 4.1. **Question 3** is answered by the split above.

---

## 1. The short answer

1. **One number decides all three: the power life can capture, set against the power it costs to stay alive.** Call the gap the *surplus*. No surplus: the habitat is closed, with its reason. A small surplus: life is there, thin and slow. A big surplus: life is dense, spreads as fast as its biology allows and evolves fast. This is Alex's "first step of the energy gradient" in code: free energy arriving, the share life captures, then everything downstream paid out of that.
2. **A negative world heat balance is not the test.** A planet that is cooling (more heat out than in) can be full of life: Earth is one. What life needs is *free* energy: sunlight's low entropy, or chemicals that have not yet reacted. Section 3 says how the Energy flow view should show both, so a GM is never misled by the heat arrows.
3. **Most of the machinery exists, but mostly in the opt-in model.** The keep-alive test (power against maintenance) and "most living matter the power can feed" are built only in the gradient model `gradients/1`, which no kernel op runs yet. The default `listed/1` model has neither. Section 2 has the checks.
4. **Where the energy goes**: the captured power is shared among the kinds of life by what each eats, level by level, with rough numbers by size class from the metabolic theory of ecology (section 5). A headcount for people waits for 4.1, because it depends on diet and tech.
5. **Spread** has two legs (section 6): a front moving through one habitat (Fisher's wave, speed from growth rate and mixing), and jumps between habitats along the habitat graph (seeds arriving times the chance they take hold, both paid by energy). At the history's 5 Myr steps most fronts are instant; deep rock and sub-ice links are not. On the campaign clock (years) both legs matter, which is the 4.1 dynamic life feature.
6. **Pace** already follows living production (section 7). Energy acts as a speed control: the baseline clocks recovery after a catastrophe in the same "Earth-Myr of opportunity" the innovations use and shows one speed line per world; 4.1 drives pace from the numbers of section 5 instead of a tuned power of production.
7. **The 4.0 baseline** (section 9): keep-alive, the energy ledger, where the energy goes, the speed read-out, broad-brush spread in the deep-time history and recovery by pace. **The 4.1 bucket** (section 9A): pace from numbers, spread on the campaign clock, transfer between worlds, people and tech, xenobiology in depth.

---

## 2. What exists today (read from main at 7984c7d, 26 Nov 2026)

The coordinator passed on findings from the Energy flow thread. Checked against the code:

| Finding | Verdict | Where |
|---|---|---|
| Life publishes, per habitat, the power each gradient offers against the maintenance of a minimal population, and closes habitats that fall short | **True only in `gradients/1`**, which is opt-in (`life_model` defaults to `listed/1`) and not run by any kernel op; the kernel's `life_view` calls the plain history with the listed model | `budget.rs:100-105`, `gradients.rs:143`, `:400`, `world_life.rs:582`; `status/kernel.md` "next: turning E2 and E4 on in the kernel's coupled runs" |
| Max biomass (`max_biomass_gc`) is given | **True per gradient, `gradients/1` only.** The habitat's standing biomass is still production x a residence time, and is never compared with it, so the two numbers can disagree | `gradients.rs:170`, `budget.rs:404` (`finish`, shared by both models) |
| The default model has a "too little energy" test | **Only for feeding levels**: production density below `min_viable_g_m2_yr` (0.1 g C per m² per year) gives zero levels, but any production above zero still counts as life | `budget.rs:408-411` |
| Evolution pace follows opportunity, which grows with living production, temperature and habitat variety | **True.** Per step: (production / 8 x 10^16 g C per year)^0.25 x exp(−0.65 eV / kT against 288 K) x (open habitat kinds / 7). Production is the smaller of the energy cap and the nutrient cap | `history.rs:826-833`; pack `life_history` (exponent cited to Lynch 2007, clonal interference) |
| The gradient's prize drives which metabolism comes next | **True in `gradients/1` only** (E2, opt-in) | `history.rs:857-890` |
| Spread is instant | **True.** Origin in one habitat gives life in every open habitat of the world in the same step | `history.rs:11`, `:623` |
| Habitat links carry a transfer rate | **No.** `rate_per_s` is always `None`; the field waits for "box B" | `habitat.rs:118-127`, `:491` |
| Recovery after a catastrophe | A fixed 10 Myr time constant (Kirchner and Weil 2000) on every world, whatever its pace | `history.rs:835-849`, pack `recovery_myr` |
| Maximum population | **Missing.** Guilds carry production, intake, feeding level and a largest body mass (`max_mass_kg`), never a count | `biosphere.rs:37-56` |

So the coordinator's reading was right about the shape and wrong about what a user sees today: what the Life screen shows runs on the listed model, with no keep-alive test.

---

## 3. The energy ledger life reads

### 3.1 Heat balance and free energy are different books

The Energy flow view (HV) shows a world's heat: sunlight absorbed, heat from inside, heat radiated. Those always nearly balance, and their sign says whether the world is warming or cooling. It says nothing about life.

Life lives on the *quality* of the energy, not the amount: sunlight arrives from a 5,800 K surface and leaves as 255 K infrared, and that drop is the gradient. The share of radiation that can in principle do work is Petela's factor, 1 − 4/3 (T₀/T☉) + 1/3 (T₀/T☉)⁴, about 0.93 for sunlight used at 288 K and about 0.87 for a 3,000 K red dwarf *[Petela 1964; unverified]*. Chemical gradients (hydrogen with carbon dioxide at a vent, oxygen with methane in air) are free energy too, and the gradient catalogue already computes them from ΔG at local conditions.

### 3.2 Five lines per habitat, and for the world

One read-out, the same words on every screen:

| Line | Meaning | From |
|---|---|---|
| **Arriving** | Free energy reaching the habitat: usable-band light (Petela-weighted) plus chemical supply | climate light at the habitat (asked of the climate thread by HV) and box B's supplies |
| **Offered** | What life of this chemistry could tap: the open gradients | gradient catalogue (or the listed metabolisms' energy caps) |
| **Captured** | What the biosphere actually runs on, after nutrients (Liebig) and the metabolisms it has evolved | production power, `power_w` |
| **Keep-alive** | Maintenance of the living matter at the habitat's temperature (Arrhenius) | `maintenance_w_per_gc` (Tijhuis et al. 1993) |
| **Surplus** | Captured minus keep-alive at a minimal population. Zero or below: closed, "too little energy to keep anything alive" | new, LE1 |

Each line answers "why?" with its numbers. The world's lines are sums. This is the "life's share" branch the Energy flow view wants: arriving and captured beside the heat arrows, at a labelled scale, because life's share of sunlight is about a thousandth on Earth (about 130 TW captured of about 120,000 TW absorbed; *unverified, from Field et al. 1998 production at about 40 kJ per g C*).

### 3.3 Standing biomass, one number

Today standing biomass = production x residence time (`budget.rs:404`). The physical ceiling is the mass the captured power holds at maintenance. LE1 makes standing biomass the smaller of the two and says which limited it:

biomass = min(production x residence, captured power / maintenance per gram)

On Earth's land the residence time binds (wood is cheap to keep); in a cold, starved aquifer the maintenance ceiling binds. One rule, both regimes, no special case.

---

## 4. "If negative it is doubtful life could happen"

That becomes the **keep-alive test**, and it moves out of the opt-in model into the shared step both models use (`budget::finish`):

- surplus ≤ 0: the habitat is closed, reason "about 3 x 10⁻¹⁸ W per gram arrives here; staying alive at 2 °C costs about 10⁻¹⁵ W per gram" (numbers from the habitat);
- surplus small: open, flagged *thin*: feeding levels zero, slow spread, slow pace (sections 5 to 7 all fall out of it);
- the maintenance figure is the one `gradients/1` already uses (1.25 W per C-mol at 25 °C, activation 69.4 kJ per mol, Tijhuis et al. 1993), moved from `life_gradients` to one shared block so both models read the same number.

Doubt stays honest: measured maintenance powers span seven orders of magnitude, from deep sediment to laboratory cultures *[LaRowe and Amend 2015, read 19 Nov]*. Life in the deep biosphere runs far below laboratory maintenance *[Hoehler and Jørgensen 2013]*. So the pack keeps two values, *laboratory* and *starved* (the deep-sediment basal power), and the test uses the starved one for "can anything live here at all" and the laboratory one for "can a population grow". A habitat between the two is "alive, barely": the honest middle ground for Europa or deep crust.

---

## 5. Sharing the energy among kinds of life (was: maximum population)

After Alex's answer (section 0), 4.0 builds 5.1 and 5.2 as a read-out of each kind of life's share and rough numbers by size class, for the kinds the life suite already has. 5.3 (the GM's headcount query) and 5.4 (civilisations) are in the 4.1 bucket. In 4.1, people are a guild with a diet like any other, and tech adds the "brought in" line to the ledger.

### 5.1 The block

Power reaching feeding level n is the captured power times the transfer efficiency to the power n − 1 (already in the pack: 0.1, Lindeman 1942). A consumer of body mass M at body temperature T needs, per individual,

B = b₀ x M^¾ x exp(−E / kT)

*[Kleiber 1932; Gillooly et al. 2001, Brown et al. 2004; E about 0.65 eV, already in the pack]*. b₀ has three published values: single cells, cold-blooded animals and warm-blooded animals, an order of magnitude apart each *[Hemmingsen 1960; unverified]*. So:

N_max = (power at level n) x share / B

- **share = 1** gives the ceiling: every joule at that level goes to this one kind. The screen says "at most".
- **share** from the guild's own share of its level (the guild already carries `share`) gives a *likely* figure.
- A **stated per-head power** replaces B for anything that is not an animal at rest, such as people, who eat about 100 W of food *[2,000 to 2,500 kcal per day; textbook]*.

The block is a small editable cited block, b₀ values and presets live in the reference pack (read via Look up), nothing in code.

### 5.2 Body sizes inside a guild

A guild has a largest mass (`max_mass_kg`, from gravity and the medium) but no typical one. Damuth's energy equivalence says each size class of a level takes about the same power, so density falls as M^−¾ *[Damuth 1981; unverified]*. The guild's power is spread evenly over log-mass classes from its smallest member (cell size for microbial grades, a pack value per grade) to its largest, and each class gets its count. That gives a size spectrum (how many 1 kg, 10 kg, 100 kg animals) without inventing creatures, in keeping with the "they exist" scope (V4 first stage is function, not appearance).

### 5.3 The GM's question: "how many could live here?"

One query: a body, a consumer (preset or own: mass, warm or cold blooded, or a per-head power; the level it eats at; where it lives), and optionally a share. The answer: the ceiling, the likely figure, which limit bound it (light, nutrients, temperature, the level), and what would raise it ("eating plants instead of meat: x 10"). "Implausible is not a limit": a GM can ask for 50 billion people; the answer says how much of the world's production that takes and what is left for everything else.

Checks: Earth's ceiling for people eating plants directly, taking all production, comes out near 10¹² (130 TW / 100 W); published human carrying capacity estimates run from under 10⁹ to about 10¹² with most between 4 and 16 billion *[Cohen 1995; unverified]*, and today's human appropriation of production is about a quarter *[Haberl et al. 2007; unverified]*. So the ceiling must sit at or above the top of the published range and the likely figure, with a human share of about 0.25 and eating level 1.2, inside it. Damuth's mammal densities must be met within a factor of three for grazers of 1 to 1,000 kg on Earth's land.

### 5.4 Civilisations

Food is the first ceiling, not the only one. A culture that powers greenhouses from fusion feeds more people than its sunlight does. The query takes an extra power source (watts, from where) so the same block answers "with orbital mirrors" or "with fusion farms", and the result says it is no longer the world's own budget. The full technosphere stays L5's.

---

## 6. Spread

### 6.1 Leg one: a front through one habitat

Inside one connected habitat, life spreads as a travelling wave. Its speed is v = 2 √(r D) *[Fisher 1937; Skellam 1951]*: r the growth rate of a thin population, D how fast the medium mixes things.

- **r** from the metabolic theory: the most a population can grow scales as M^−¼ x exp(−E/kT) *[Savage et al. 2004; unverified]*, capped by energy near the keep-alive line: r = min(r_max, surplus per gram / build cost per gram). Rich habitat: biology sets the speed. Starved habitat: energy does. That is the honest answer to "how fast could life spread": energy decides it at the margin, biology above it.
- **D** by medium, a cited table in the reference pack: ocean eddy mixing, wind for spores, rivers and animals for seeds, pore water in rock (molecular diffusion, about 10⁻⁹ m² per second). Entries that are only orders of magnitude are in the fudge register.
- **Fill time** = the habitat's span / v.

Order-of-magnitude checks *[all unverified]*: microbes cross an ocean in years (r about one a day, D about 10³ m² per second gives about 0.2 m per second); postglacial trees moved 100 to 1,000 m a year *[Clark 1998, "Reid's paradox"]*, so a continent in 10⁴ to 10⁵ years; microbes in rock pores move centimetres a year, so a kilometre of crust takes about 10⁵ years and a deep crust several Myr.

So at the history's 5 Myr steps, the oceans and land fill at once (today's "spread is fast" is right for them) while deep rock does not. On the campaign clock nothing is instant.

### 6.2 Leg two: jumps between habitats

The habitat graph already has links (vertical, lateral, embedded). LE gives each link a colonisation hazard, the same waiting-time form as origin (keyed draw, monotonic in every dial):

hazard = (propagules sent per year) x (chance one takes hold)

- **propagules sent** = source standing biomass x the medium's export share x the link's transfer rate (the `rate_per_s` field finally filled: sinking particles, upwelling, wind, ice cracks, porewater flow; cited table by contact and medium pair);
- **chance it takes hold** rises with the target's surplus per founder and is zero where the target is closed or needs a capability the arriving life lacks (land needs `land`; that rule exists).

Energy enters twice: a rich source sends many seeds, a rich target lets them stay. A thin sub-ice ocean seeded through cracks from a busy ocean above gets life quickly; the same ocean fed only by a starved crust may wait a long time.

### 6.3 What changes in the history

- Life arrives in habitats one at a time with dates and causes ("reached the deep crust 40 Myr after origin, seeded from the vent field"), instead of all at once.
- Refuges become real: a catastrophe that spares only the deep crust leaves the world to be re-seeded upward by leg two, and the recovery event cites the refuge.
- Origin in a deep habitat followed by slow spread up is a story the GM can find.
- The same form serves box C's transfer between worlds (rocks thrown between planets), with ejecta exchange from the impact history as the link rate (LE8).

---

## 7. Pace of evolution

### 7.1 Does the existing pace answer Alex's 23:19 point?

Mostly, yes. Opportunity per step already grows with living production, and production is captured energy (or the nutrient cap when nutrients bind, which is also right: energy you cannot build with is not opportunity). Under `gradients/1` the prize term adds exactly Alex's idea: an untapped gradient's size speeds its own discovery.

Three things are not honest yet:

1. **Production stands in for population.** The pack raises production to the power 0.25, citing clonal interference. The underlying quantity is the supply of new variants: individuals x generations per year. A biosphere of whales and one of bacteria with the same production differ by many orders of magnitude in both.
2. **The 0.25 is one exponent for every size.** Population genetics gives two regimes: small populations adapt in proportion to their size (waiting for mutations), large ones only with its logarithm (mutations compete) *[Gerrish and Lenski 1998; Desai and Fisher 2007; unverified]*. One fixed power is fair in the middle, wrong at both ends, and the starved worlds Alex is asking about are the small end.
3. **Recovery ignores pace.** Every world gets Earth's 10 Myr after a catastrophe, while innovations already wait in "Earth-Myr of opportunity". A world evolving a hundred times slower should recover a hundred times slower.

### 7.2 The change

- Opportunity supply = sum over guilds of N (section 5, likely figure) x generations per year (from M and T, Gillooly et al. 2002 *[unverified]*), passed through the two-regime form: linear below a crossover, logarithmic above it, with the crossover a cited pack value.
- Normalised so present-day Earth gives one Earth-Myr per Myr, as now, so every capability rate in the pack keeps its meaning.
- Recovery clocked in Earth-Myr of opportunity, so Kirchner and Weil's 10 Myr stays the Earth figure and other worlds scale from it.

Risk, stated plainly: early Earth had far less life, and the linear regime makes it slower than today's 0.25 power does. The E2 check (Earth outline in at least 59 percent of seeds, anoxygenic photosynthesis soon after origin) must still pass with the constants as published. If it does not, that is a finding about the model, reported to Alex, not a reason to tune the crossover; the 0.25 power then stays, entered in the fudge register.

---

## 8. Desks and screens

- **Explore**: the Energy flow view's life branch (arriving, captured, keep-alive) and a Life screen card "How many could live here", with the ceiling and the likely figure in words.
- **Edit**: the population query with own consumers; "Seed life here" (4.1) with the front drawn growing on the campaign clock.
- **Play**: players see the population figure only once surveyed (box I's survey tier), and a seeded or invasive spread as a story event.
- **Phone**: the five ledger lines as a list; the query as a short sheet.

Each GM screen takes its `rail` entry, per the top-level layout.

---

## 9. Packages: the 4.0 baseline, in build order

All kernel work is in `sse-life`, `packs/base-life` and the reference pack, with goldens in the life area unless noted. One at a time, each with its checks. Broad brush: simple cited tables where the detail is for 4.1, each such table entered in the fudge register as "baseline, refined in 4.1".

| Id | Delivers | Needs | Verified by |
|---|---|---|---|
| **LE1** | Keep-alive test in the shared `finish` step (both models), maintenance in one shared cited block, standing biomass capped by power over maintenance | none | A habitat with supply below maintenance is closed with its reason in both models; Earth's totals stay inside L1a's ranges; Europa closed or open with its numbers; test seen red with the cap removed |
| **LE2** | The energy ledger per habitat and world on `life_view`: arriving (with the Petela factor), offered, captured, keep-alive, surplus, and a sixth line **brought in**, always zero in 4.0 (tech's seam) | LE1; climate light at the habitat for "arriving" (HV's ask), shown as not published until it lands | Lines sum to the world; captured ≤ offered ≤ arriving on 10,000 random worlds; Earth captured about 130 TW within a factor of three |
| **LE3** | **Where the energy goes**: each guild's power by its diet (a list of what it eats: a gradient, or other guilds, with Lindeman's 10 percent per step), its share of the world's captured power, and rough numbers in broad size classes (cells, insect-sized, animal-sized, tree-sized) by energy equivalence and Kleiber | LE1 | Shares sum to captured power on every world; Damuth's mammal densities within a factor of ten; a gap, never a zero, where production is unknown |
| **LE4** | **The speed read-out**: fill time per habitat from Fisher's front (r from metabolic theory capped by the surplus; mixing by medium from a short cited table), and one world line "life here moves and changes at about N times Earth's pace" from opportunity | LE1 | Section 6.1 orders of magnitude; r falls to zero smoothly at the keep-alive line; Earth reads 1 |
| **LE5** | Spread in the deep-time history, broad brush: link rates from a short table by contact and medium, colonisation as a keyed waiting time (seeds sent x chance of taking hold), fill time added, dated colonisation events with causes, refuges re-seeding | LE4 | Earth's surface habitats filled within one step of origin; deep crust later; a sterilising event with one surviving refuge re-seeds the rest from it; dials monotonic; deterministic. Moves goldens: a core golden lane slot if `life_view` is in core |
| **LE6a** | Recovery after a catastrophe clocked in Earth-Myr of opportunity (10 Myr on Earth, slower where life is slower) | LE4 | Earth's recovery unchanged within one step; a world at a tenth of Earth's pace recovers about ten times slower |
| **LE-U1** | Energy flow life branch (HV thread) | LE2 | screens desk and phone; every line answers "why?" |
| **LE-U2** | Life screen card "Where the energy goes" (shares by kind of life, the speed line in words) | LE3, LE4 | screens desk and phone; press budget unchanged |

LE1 to LE4 change no dated history (only closures, biomass and new fields). LE5 and LE6a do, so they take golden slots.

## 9A. The 4.1 bucket, and the seams the baseline leaves for it

Every good idea that is detail or tech, kept here so the baseline is built to take it:

| Idea (4.1) | What it adds | Seam the baseline leaves |
|---|---|---|
| **LE6b** pace from individuals x generations | The two-regime supply of section 7.2 instead of the 0.25 power | Opportunity is one block whose input is LE3's guild table |
| **LE7** spread on the campaign clock | Seed or introduce life at a date; fronts and arrivals at campaign time; invasions; terraforming stories | LE5's waiting-time form takes any source of arrivals, not only a neighbouring habitat |
| **LE8** transfer between worlds | Rocks thrown between planets carry life (box C) | The same form, with ejecta exchange as the link rate |
| **LE9** people and tech | A people guild with a diet; "how many could this world feed, eating what, with what tech"; ceilings and likely figures (old sections 5.3, 5.4, questions 1, 2, 6); outside power (fusion farms, mirrors); tech changing the planet, and its impact beyond the planet | LE2's "brought in" line; LE3's diet lists; box H's fluxes, which already take any source |
| **LE10** xenobiology in depth | Per-chemistry maintenance, growth and size rules; finer size spectra; body plans | Maintenance and the metabolic constants are one shared block per chemistry (LE1), not code |
| **LE11** two maintenance values | Starved and laboratory, with an "alive, barely" middle (question 5) | LE1 reads its maintenance from one block that can hold both |

---

## 10. Questions for Alex

Answered 26 Nov 23:50 (section 0): 1, 2, 3, 4 and 6. Still open:

5. **Two maintenance values (starved and laboratory), giving an "alive, barely" middle?** Recommend: 4.0 uses the one value the gradient model already has (laboratory, Tijhuis et al. 1993), and the starved value comes with 4.1's xenobiology (LE11). In 4.0 some deep-crust and Europa-type habitats will read closed when the deep biosphere suggests they could be barely alive; the reason given will say so.

---

## 11. Sources and how far each was checked

Nothing in this list was re-read on 26 Nov 2026 (paper search out until 1 Dec). Items marked *read* were read on 19 Nov for the evolution design. Everything else is recalled and must be confirmed when its pack entry is written.

- Petela 1964, "Exergy of heat radiation", Journal of Heat Transfer.
- Field, Behrenfeld, Randerson and Falkowski 1998, "Primary production of the biosphere", Science (already cited by L1a).
- Tijhuis, van Loosdrecht and Heijnen 1993, "A thermodynamically based correlation for maintenance Gibbs energy requirements in aerobic and anaerobic chemotrophic growth", Biotechnology and Bioengineering (already in the pack).
- LaRowe and Amend 2015, "Power limits for microbial life", Frontiers in Microbiology (*read*).
- Hoehler and Jørgensen 2013, "Microbial life under extreme energy limitation", Nature Reviews Microbiology.
- Lindeman 1942, "The trophic-dynamic aspect of ecology", Ecology.
- Kleiber 1932, "Body size and metabolism", Hilgardia.
- Hemmingsen 1960, "Energy metabolism as related to body size and respiratory surfaces, and its evolution", Reports of the Steno Memorial Hospital.
- Gillooly, Brown, West, Savage and Charnov 2001, "Effects of size and temperature on metabolic rate", Science (already cited).
- Gillooly, Charnov, West, Savage and Brown 2002, "Effects of size and temperature on developmental time", Nature.
- Brown, Gillooly, Allen, Savage and West 2004, "Toward a metabolic theory of ecology", Ecology.
- Savage, Gillooly, Brown, West and Charnov 2004, "Effects of body size and temperature on population growth", American Naturalist.
- Damuth 1981, "Population density and body size in mammals", Nature.
- Cohen 1995, "Population growth and Earth's human carrying capacity", Science.
- Haberl et al. 2007, "Quantifying and mapping the human appropriation of net primary production in Earth's terrestrial ecosystems", PNAS.
- Fisher 1937, "The wave of advance of advantageous genes", Annals of Eugenics.
- Skellam 1951, "Random dispersal in theoretical populations", Biometrika.
- Clark et al. 1998, "Reid's paradox of rapid plant migration", BioScience.
- Gerrish and Lenski 1998, "The fate of competing beneficial mutations in an asexual population", Genetica.
- Desai and Fisher 2007, "Beneficial mutation-selection balance and the effect of linkage on positive selection", Genetics.
- Lynch 2007, *The Origins of Genome Architecture* (already cited for the 0.25 power).
- Kirchner and Weil 2000, "Delayed biological recovery from extinctions throughout the fossil record", Nature (already cited).
