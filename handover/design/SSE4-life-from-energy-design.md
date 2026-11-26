# SSE4: life's spread, population and pace from the energy it can get (track LE)

Status: **PROPOSED 26 Nov 2026**, not yet accepted. Design only; nothing here is built by this note. Packages for builders are in section 9. LE is a code only, never shown in the product.

Read with: `SSE4-life-design.md` (boxes B, C, D, E), `SSE4-evolution-design.md` (sections 3 and 5, the gradient catalogue and opportunity times prize), `status/names-and-life.md` (L3), `status/kernel.md` (E1, E2, E4 to E6 built as opt-in), the Energy flow view (memory `sse4-heat-flow-view`, PR 670, track HV).

Alex's brief (26 Nov 2026):

- 23:18 (quoted by the coordinator): "if negative it is doubtful life could happen... use this energy balance to determine how fast life could spread - use it to determine max population".
- 23:19: "its another useful feed intp that - energy availoable on planet wilkl cotrol teh pace of develoopment/evolution as it is the "first step" of teh energy gradient".

Figures marked *unverified* are recalled, not re-read this session: the Consensus paper search is out until 1 Dec 2026. Each must be confirmed against the paper when its pack entry is written.

---

## 1. The short answer

1. **One number decides all three: the power life can capture, set against the power it costs to stay alive.** Call the gap the *surplus*. No surplus: the habitat is closed, with its reason. A small surplus: life is there, thin and slow. A big surplus: life is dense, spreads as fast as its biology allows and evolves fast. This is Alex's "first step of the energy gradient" in code: free energy arriving, the share life captures, then everything downstream paid out of that.
2. **A negative world heat balance is not the test.** A planet that is cooling (more heat out than in) can be full of life: Earth is one. What life needs is *free* energy: sunlight's low entropy, or chemicals that have not yet reacted. Section 3 says how the Energy flow view should show both, so a GM is never misled by the heat arrows.
3. **Most of the machinery exists, but mostly in the opt-in model.** The keep-alive test (power against maintenance) and "most living matter the power can feed" are built only in the gradient model `gradients/1`, which no kernel op runs yet. The default `listed/1` model has neither. Section 2 has the checks.
4. **Population** is the surplus divided among feeding levels and body sizes by the metabolic theory of ecology: a world can say how many grazers of 50 kg it feeds, or how many people (section 5). One block, one GM query.
5. **Spread** has two legs (section 6): a front moving through one habitat (Fisher's wave, speed from growth rate and mixing), and jumps between habitats along the habitat graph (seeds arriving times the chance they take hold, both paid by energy). At the history's 5 Myr steps most fronts are instant; deep rock and sub-ice links are not. On the campaign clock (years) both legs matter, which is the 4.1 dynamic life feature.
6. **Pace** already follows living production (section 7). The honest fix is to drive it from the same population count section 5 makes, instead of a tuned power of production, and to clock recovery after a catastrophe in the same "Earth-Myr of opportunity" the innovations already use.
7. **Sooner than 4.1:** the keep-alive test in the default model, the energy read-out for the Energy flow view, the population query and the within-habitat fill time are all read-outs that change no history. They can land in 4.0. Spread that changes the dated history, and spread on the campaign clock, are 4.1.

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

## 5. Maximum population

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

## 9. Packages, in build order

All kernel work is in `sse-life`, `packs/base-life` and the reference pack, with goldens in the life area unless noted. One at a time, each with its checks.

| Id | Delivers | Needs | Verified by | Release |
|---|---|---|---|---|
| **LE1** | Keep-alive test in the shared `finish` step (both models), starved and laboratory maintenance in one shared cited block, standing biomass capped by power over maintenance, "alive, barely" flag | none | A habitat with supply below starved maintenance is closed with its reason in both models; Earth's totals stay inside L1a's ranges; Europa reads "alive, barely" or closed with numbers; test seen red with the cap removed | 4.0 |
| **LE2** | The five-line ledger per habitat and world on `life_view` (arriving with the Petela factor, offered, captured, keep-alive, surplus) | LE1; climate light at the habitat for "arriving" (HV's ask), shown as not published until it lands | Lines sum to the world; captured ≤ offered ≤ arriving on 10,000 random worlds; Earth captured about 130 TW within a factor of three | 4.0 |
| **LE3** | Population block (Kleiber with three b₀, energy equivalence over log-mass classes); per-guild size spectrum; `life_capacity` query with presets (people, grazer, microbe) and an extra power source | LE1 | Section 5.3 checks; ceiling ≥ likely on every world; a stated per-head power overrides B; never a count where production is unknown (a gap, not a zero) | 4.0 |
| **LE4** | Fill time per habitat from Fisher's front (r from metabolic theory capped by surplus; D table by medium) as a read-out | LE1 | Section 6.1 orders of magnitude; r falls to zero smoothly at the keep-alive line | 4.0 |
| **LE5** | Spread on the deep-time history: link rates filled, colonisation hazards, fill time added, dated colonisation events with causes, refuges re-seeding | LE4 | Earth's surface habitats filled within one step of origin; deep crust later; a sterilising event with one surviving refuge re-seeds the rest from it; dials monotonic; deterministic. Moves goldens: a core golden lane slot if `life_view` is in core | 4.1 (4.0 if a slot is free) |
| **LE6** | Pace from population (two-regime supply), recovery in Earth-Myr of opportunity | LE3, LE5 | E2's Earth outline check; a starved world's recovery slower than Earth's in proportion; section 7.2 risk reported either way | 4.1 |
| **LE7** | Spread on the campaign clock: seed or introduce life at a date, front radius and arrival dates read at any campaign time, story events | LE5, picture clock | Seeding algae on a fixture ocean world gives a front within an order of magnitude of section 6.1; stopped clock, nothing moves | 4.1 |
| **LE8** | Transfer between worlds (box C) through the same hazard, link rate from ejecta exchange | LE5, small-bodies exchange rates | Related biospheres share their biochemistry stamp; rate monotonic in the impact history | after 4.1 |
| **LE-U1** | Energy flow life branch (HV thread) | LE2 | screens desk and phone; every line answers "why?" | 4.0 |
| **LE-U2** | "How many could live here" card and the Edit desk query sheet (Life screen thread) | LE3 | screens desk and phone; press budget unchanged | 4.0 |

LE1 to LE4 change no dated history (only closures, biomass and new fields), so they can follow the opt-in gradient wiring or come before it.

---

## 10. Questions for Alex

1. **"Max population": people, or all creatures?** Recommend both, from one block, with "how many people could this world feed" as the first preset, since that is the GM's question.
2. **Show the ceiling, the likely figure, or both?** Recommend both: for Earth roughly "at most 1.3 trillion, likely about 9 billion", so a GM sees the room and the realistic answer together.
3. **Land the read-outs in 4.0 and the moving spread in 4.1?** Recommend yes: LE1 to LE4 and the two screens now; LE5 to LE7 with dynamic life.
4. **Replace the tuned 0.25 pace power with individuals times generations?** Recommend yes, kept only if Earth's outline still passes with published constants; otherwise keep 0.25 in the fudge register and report why.
5. **Two maintenance values (starved and laboratory), giving an "alive, barely" middle?** Recommend yes: it is what the deep biosphere shows, and it is where Europa-type stories live.
6. **Let a GM add outside power (fusion farms, mirrors) to the population query?** Recommend yes, labelled as no longer the world's own budget: "implausible is not a limit".

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
