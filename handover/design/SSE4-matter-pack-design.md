# SSE4 master data sheet (the matter pack): constants, elements and substances in one place, real by default, yours to change

Date 21 Nov 2026. Status: **accepted by Alex, 21 Nov 2026** ("get the table, we were risking drift"; then "yes to all" on the questions, with units added to the sheet and the periodic table coloured by where each element was made). Section 12 holds the rulings. Audience: Alex, the coordinator, and the build threads that will take the packages in section 10.

Alex's question (21 Nov): are our element, phase and atmosphere tables fully populated, and should there be a "periodic table" and a compound table that show the real data while letting people add their own "unobtainium" or change what we use?

Alex's follow-up (21 Nov): "A single user editable master data sheet makes sense. The core data we need for the sim. Also universal constants, etc, should all be in there I guess. A Constants page which holds all known scientific data relevant to us... peeps I guess can see what happens if you change G :)"

---

## 1. The short answer

1. **No, they are not fully populated, and there is no shared table at all.** Each part of SSE4 typed in the handful of substances it needed. Section 2 lists what the code holds today (checked against `main` at 1f66399).
2. **Yes, build it, as one "matter pack".** One row per element, one row per substance, each row carrying every property any block reads, each value with its source. Every block reads that one table. The climate, light, life, formation and surface packs stop keeping their own copies.
3. **Real is the default and is always visible.** A value from nature shows its paper. A value you changed shows "authored" with the real value beside it. A substance that does not exist in nature shows "fictional". The marks are on the GM view only, as everywhere else.
4. **A made-up element is not a special case.** It is a row with fewer or more properties filled in. A block that needs a property the row lacks skips that substance and says so. So "unobtainium" can be a pure mining resource (name, weight, how common), or it can rain, colour a sky, heat a core or feed life, depending only on how much of its row is filled in.
5. **Elements ride inside the six materials formation already tracks.** Nothing tracks 118 budgets through a system's history. How much gold or unobtainium a body holds is worked out when someone asks, from its makeup, its star's inheritance and where each element goes when a world separates into core, mantle, crust, sea and air. That is the resource inventory the tags design asked for (TG11), done once for every element instead of one block per resource.
6. **Universal constants are on the same sheet** (section 7A). G, c, the Boltzmann and Planck constants and the rest become data that every block reads, instead of numbers compiled into the code and retyped in a dozen crates. A changed constant is marked like any other authored value. What changing one can honestly show is limited, and the sheet says exactly where the limit is: some blocks follow the change, some are estimated by a cited scaling law, and some are tables computed with the real constants that cannot follow.
7. **The chart room gets a Reference screen in the Workshop**: a Constants page, a periodic table, a substance browser with a phase diagram, and "Why?" on every value. The best lens on it is "where was this atom made?", answered from this star's own galactic history.

---

## 2. What the code holds today

| Where | What is there | How it is stored |
|---|---|---|
| Climate (`packs/base-climate`, `substances`) | 5 condensables: water, carbon dioxide, methane, ammonia, nitrogen. Triple point, latent heat, molar mass, condensed density, cloud colour | pack data, sourced to the NIST WebBook |
| Light (`packs/base-light`, `gases`) | 13 gases (dry air, N2, O2, Ar, CO2, H2, He, CH4, H2O, NH3, SO2, CO, Ne): molar mass, refractivity, King factor. Four of them are marked "from memory, verify". A list of named spectral lines with a free-text species | pack data |
| Life (`packs/base-life`, `life_chemistry`) | 26 species (gases, dissolved forms, ions, three organics) with formula, charge, free energy and enthalpy of formation; one biochemistry (carbon in water) with a typed-in temperature window | pack data, sourced to Thauer |
| Formation (`sse-formation`) | 6 material groups: metal, rock, carbon, water, CO2, CO. One condensation temperature each (1400, 500, 135, 47, 20 K) and one density each (7800, 3000, 2000, 1000, 1500, 900) | constants in Rust |
| Galaxy (`sse-cosmos`, rings) | 6 nucleosynthesis groups: alpha, iron peak, C and N, s-process, r-process, potassium-40 | budgets per ring |
| Heat (`sse-atoms`, heat) | 5 isotopes: U-238, U-235, Th-232, K-40, Al-26, with share, heat and half-life | constants in Rust, well cited |
| Surface (`sse-surface`, `packs/base-surface`) | rock 3500 and ice 930 kg/m3, their strengths, ice melting at 273.15 K, ice conductivity; 13 surface colours (basalt, granite, ice, lava and so on); giant cloud decks as 5 rows picked by temperature thresholds | constants in Rust and pack data |
| Weather (`packs/base-weather`) | magma 2500 kg/m3, Earth air 0.02897 kg/mol, giant air 0.0023 kg/mol | pack data |
| Traveller (`packs/traveller`) | 23 named gas mixes, each a list of gas names with ranges | pack data |
| Renderer (`packages/renderer`) | its own map of gas names to kernel gas ids | TypeScript |
| Constants (`sse-core`, `quantity::consts`) | 13 values: G, c, the astronomical unit, day and year, the Stefan-Boltzmann constant, and the Sun's, Earth's and Jupiter's reference sizes | compiled into the kernel; and retyped about 60 more times in other crates and tests |

What this adds up to:

- **No periodic table.** No element has a row anywhere. "Iron" exists as a spectral line label, a share of a material group and a dissolved ion, with nothing joining them.
- **The same substance is typed in several times.** Water's molar mass is in the climate pack (in kg) and the light pack (in g); its formula is in the life pack; its ice density and melting point are Rust constants in the surface crate; its condensation temperature in a disc is a Rust constant in formation. Nothing checks they agree. Today they do agree to rounding, by care rather than by construction.
- **Related numbers come from no shared curve.** Carbon dioxide "condenses at 47 K" in formation and has a triple point of 216.58 K in climate. Both are right, at different pressures, but neither is read off one vapour-pressure curve. The ice line sits at 263.15 K in climate, 263 K in wear, and 266 and 271 K in cover.
- **Whole families are missing.** No volatile ices beyond the five (no CO, O2, Ar, SO2 frost, no ethane). No minerals or rock types with properties (rock is one density). No salts beyond one molar mass. No absorption bands per gas (the greenhouse is a typed number). No table of what each element is, weighs, or how common it is.
- **Constants have drifted already.** Earth's radius is 6378.1 km in the core (equatorial) and 6371 km in four other crates (mean), under the same name. Earth's mass is GM/G in some crates and a typed 5.9722e24 kg in others. One crate keeps its own copy of G. No Boltzmann, Planck, gas or Avogadro constant has a single home; each user types its own.
- **Some gates hide in here.** The giant cloud decks pick "ammonia ice" or "water ice" by a temperature threshold. Under the processes rule they should come from each cloud species' own vapour-pressure curve.

The weather review already planned a shared volatile table (`sse.volatile.species`, package W8). This design is that idea widened to all matter, and W8's table becomes part of it (section 6.2).

---

## 3. The design in one page

**One pack, `packs/base-reference`, read through one crate, `sse-reference`.** It sits at the bottom of the physics layer beside `base-physics`. It is the master data sheet: the facts about nature the simulation needs, and nothing else. It holds six tables.

| Table | A row is | Shipped (first wave) |
|---|---|---|
| **Constants** | one constant of nature, or one agreed convention | about 25: G, c, h, the Boltzmann constant, the elementary charge, electron and proton mass, the atomic mass unit, Avogadro's number, and those worked out from them (Stefan-Boltzmann, gas constant, radiation constant, fine-structure constant) |
| **Reference objects** | one agreed yardstick used as a unit | the Sun, Earth, Jupiter and Moon: mass, radii (equatorial and mean, named apart), luminosity; the astronomical unit, parsec, day and Julian year |
| **Elements** | one chemical element | all 118 |
| **Isotopes** | one nuclide that something reads | about 30: the heat producers, the fuels (deuterium, tritium, helium-3), the common dating clocks |
| **Substances** | one pure thing with a formula: a gas, an ice, a mineral, an ion, a molecule | the 45 or so the code uses now, then the volatile ices, then minerals as the surface stages need them |
| **Mixtures** | a named recipe of substances with its own measured bulk properties | dry air, sea water, basalt, granite, peridotite, the chondrite types, iron meteorite, regolith, tholin, formation's six materials, the Traveller gas mixes |

**What is not on the sheet.** Fitted and tuned model numbers (a wind scale fitted to Earth, a crater law's exponent, a star-formation efficiency) are not facts about nature. They stay in the pack of the model they belong to, with their entry in the fudge register, and remain editable there as they are today. The sheet's "used by" lists link to them. Cosmological parameters (the Hubble constant, the density shares) already live in `base-cosmology` and are shown as a page of the same screen without moving.

**A row is a bag of property groups, every one optional.** Nothing is required beyond an id, a name and (for a substance) a formula. Each value carries its unit, its source and its validity range.

**Every block declares what it reads.** A climate block says "I need a vapour-pressure curve and a latent heat". The kernel then knows, for any substance, which blocks can use it and which will skip it, and can say so in "Why?".

**Three honesty states per value**, GM view only:

| State | Meaning | What the GM sees |
|---|---|---|
| real | measured or derived from measurement; cited | the value, its paper, and the citations register's check status (verified, corrected, unverified) |
| authored | a real substance's value that a pack or the GM changed | the new value with the real one beside it and a reset |
| fictional | the element or substance does not exist in nature | the value with what it was modelled on, if anything |

Two existing marks also apply. **Hypothetical** (the kernel's existing "speculative" evidence kind) is for real science that is not yet measured: predicted superheavy elements, a calculated high-pressure phase. **Impossible** is for a value that breaks a law: it is kept and marked, never silently fixed, as elsewhere in SSE4.

**Overrides are per value, explicit and layered**, exactly as packs already work. A physics or universe pack adds rows or replaces single values with a declared `replaces`. The GM's own edits live in the campaign pack. Base data updates still flow through for every value nobody touched.

**Anything computed from an authored or fictional value carries the mark**, by the same downstream propagation packs already have (K5), made finer so it follows single values instead of whole packs. Real worlds that never touch the changed value stay unmarked.

---

## 4. What a row holds

### 4.1 Element

Symbol, name, atomic number, standard atomic weight; cosmic abundance (protosolar, on the scale the code already uses, Asplund et al. 2009); **where it is made** (shares across the galaxy model's six nucleosynthesis groups); **where it goes in a disc** (50% condensation temperature and the material it rides in: metal, rock, carbon, the ices or the gas); **where it goes in a world** (how it splits between metal and rock, rock and melt, rock and water, and whether it escapes to air); its isotopes; its spectral lines (links into the light pack's line list).

The middle three are what make the table more than a poster. They are the route from a supernova to an ore body.

### 4.2 Isotope

Mass number, share of its element at the Sun's birth, half-life, decay heat, daughter. The five heat producers move here from `sse-atoms` with their citations unchanged.

### 4.3 Substance

| Property group | Holds | Read by |
|---|---|---|
| identity | id, formula as element counts, charge, names in the three wordings (physics, plain, story). Molar mass is **worked out from the formula**, never typed | everything |
| phases | triple point, critical point, vapour-pressure curve (the Fray and Schmitt 2009 fits for ices), melting curve with pressure, latent heats, density of each phase | climate, weather, moving ices (W8, S17), formation's condensation lines, cloud decks, habitability |
| thermal | heat capacity per phase, conductivity, expansion | climate heat capacity, ice shells, interiors, speed of sound |
| mechanical | strength, flow law (ice and nitrogen-ice creep), viscosity, hardness, compression curve | roundness, glaciers, wear (S9), interiors |
| optical | gas refractivity and King factor, absorption bands, reflectance and colour of the solid and liquid, cloud particle colour | air optics, greenhouse, surface colour, renderer, spectrum chart |
| chemical | free energy and enthalpy of formation per state (gas, dissolved, solid), solubility in water, acid and redox behaviour | life's energy gradients, ocean chemistry, weathering |
| cosmochemical | which formation material it counts in, condensation temperature at disc pressure (checked against its own vapour curve) | formation |

Dissolved forms are states of the substance, not separate rows: `h2_aq` becomes hydrogen in the "dissolved" state. Ions are their own rows.

### 4.4 Mixture

A recipe (substances by mass or by volume) plus the bulk properties that a recipe cannot give: density with porosity, solidus and liquidus, strength, hardness, colour. Where a mixture has no measured value, a stated mixing rule gives one, and that rule is a registered fudge.

### 4.5 The shape of a value

```
"triple_point_k": { "v": 273.16, "src": "nist-webbook-h2o", "valid": null }
"vapour_pressure": { "fit": "fray-schmitt-2009", "coeffs": [...], "valid_k": [100, 273.16], "src": "fray-schmitt-2009" }
```

`src` names one entry in the citations register, so a paper is recorded once and its check status shows everywhere it is used. A pack override adds `"replaces"` and its own `src` (for a fictional value, the source can be "modelled on platinum" or "the setting's own canon").

### 4.6 Checks at load, so the table cannot disagree with itself

- Molar mass equals the sum of the formula's atomic weights.
- A vapour-pressure curve passes through its own triple point; the latent heat agrees with the curve's slope within a stated tolerance.
- A mixture's recipe sums to one. Every reaction conserves elements and charge.
- Every substance id named anywhere in the stack (a gas mix, a reaction, a spectral line, a cloud deck, a resource kind) exists. This is the tag design's "every referenced tag is declared" check, applied to matter.
- A failed check on base data fails the build. A failed check on an authored or fictional row loads with the value marked impossible and the reason shown.
- A lint in CI: no substance constant may be typed into a crate. The retired constants (rock and ice density and the rest) are listed, and a test walks the crates for them.

---

## 5. From a supernova to a mine: how an element reaches a resource

Each step exists today in coarse form. The matter pack joins them.

1. **Galaxy.** Each ring of a galaxy keeps six budgets (alpha, iron peak, C and N, s-process, r-process, potassium-40). The element row says which budgets make it. So a star's inheritance gives the abundance of every element, not just the six. R-process elements are already patchy from ring to ring, because neutron-star mergers are rare draws: one region can be gold-rich and the next poor.
2. **Disc.** Formation shares the disc's solids into six materials by temperature. Each element rides in the material it condenses with. The six condensation temperatures and densities stop being Rust constants and are read from the substance rows. (Later, a smooth condensation sequence per element can replace the six steps; section 10, MP14.)
3. **Body.** Formation gives every body its mass of each material. An element's mass in the body is the material's mass times the element's share in that material, scaled by the star's inheritance. Trace elements are shares **inside** a material. They add no mass of their own, so adding a trace element changes no bulk fact.
4. **Inside the world.** When a world separates, each element follows its affinity: iron-loving elements sink to the core, rock-loving ones stay in the mantle and crust, the incompatible ones concentrate in the crust, volatile ones go to sea and air. This is one block of partition shares per element, weighted by how far the world actually differentiated (a process strength, not a class).
5. **Concentration.** The surface stage already knows where ore-forming processes ran (arc copper, orogenic gold, flood-basalt nickel, impact melt). Those become enrichment rules over the element's crustal share, so a deposit is a place where a real process raised the grade.
6. **Inventory.** Kilograms of each element and substance per body, and per reservoir once surveyed. This is TG11's inventory. `resource.*` labels, survey tiers, extraction and stock work exactly as the tags design says.

**Detail levels.** At Glance, nothing new is computed: the bulk makeup already exists. At Orbit, a click works out the body's element totals (a few hundred multiplications). At Survey, the split by reservoir and the deposits. At Site, strata. Totals are conserved between levels, so it is the same world at each (tested).

**Honest limits, to go in the fudge register.** The six-material split is coarse. Partition shares are single typical values, not functions of pressure and oxygen. Stars read only total metals and the alpha ratio: nothing in the element table changes how a star burns.

---

## 6. How a fictional substance flows through each part

The rule is the same everywhere: **a block uses a substance if the row has what the block needs, and skips it otherwise.** The substance card lists both ("used by weather, light, inventory; skipped by life: no formation energy").

| Fill in | And it then |
|---|---|
| name, atomic weight, how common (a number, or "made like gold, one thousandth as much"), what it rides in | exists in every star and body in the right amounts, patchy if tied to a rare source; appears in inventories; can be surveyed, labelled and mined out. **Changes no physics.** This is plain unobtainium |
| density | shifts a body's bulk density if there is enough of it |
| a heat-producing isotope | warms interiors, so more volcanism and longer-lived geology where it is common |
| phase data (triple point, vapour curve, latent heat) | can be an ice, a sea, a rain or a cloud: weather and the moving-ices block treat it like any volatile |
| optics | colours skies, clouds and ground; adds a line to the spectrum chart, so it can be **found from afar by its spectrum** |
| formation energies and a reaction | becomes an energy source or a waste product for life; a solvent with a liquid range becomes a candidate biochemistry |
| partition shares and an ore process | concentrates in cores, crusts or deposits instead of being evenly spread |

### 6.1 Formation and interiors
Reads condensation and density from substance rows, isotopes from the isotope table. A fictional material that condenses moves the place its "snow line" falls. Everything downstream of a fictional row is marked.

### 6.2 Climate and weather (W8 folds in here)
`sse.volatile.species` is not a separate table. It is the weather track's **view** of the matter table: every substance whose row has a vapour curve and latent heat. The climate pack's five-row `substances` list is retired. Ground frost, cloud and the disc's ice lines all read one curve per substance, so they cannot disagree. The giant cloud decks become "which species' vapour curve crosses this air's temperature profile, and where" in place of the five temperature thresholds.

### 6.3 Light
The gas table moves to the optical group. Spectral lines name an element or substance id instead of free text. O4's metal lines read abundances from the star's inheritance through the element table. The renderer's private gas map goes: it reads `read_air_optics` and substance colours only.

### 6.4 Life
`life_chemistry.species` reads the chemical group. A biochemistry's solvent window and minimum pressure are read from the solvent's own phase row instead of being typed, so "life in liquid ammonia" or in a fictional solvent gets its window from physics. Life stays "plausible by default": a fictional solvent enables nothing unless a biochemistry row is also declared for it.

### 6.5 Resources and universe packs
- A resource kind (TG11) names an element, substance or mixture id. Its natural amount comes from section 5 with no block written per resource.
- A universe pack that wants its element **to occur naturally** ships the element row (marked fictional). It then appears wherever its rules put it, in every system, and stays deterministic.
- A universe pack that wants it **in particular places** uses a dressing rule to place a "deposit" object carrying `stock` of it. That is the existing mechanism: it changes no physics and needs nothing new.
- A GM who wants one special lode pins the inventory on that body (authored), as the tags design already says.
- Made goods (packaged food, antimatter) stay resource kinds with no matter row. Nothing changes for them.

### 6.6 Smaller readers
Sound (speed of sound from heat capacity ratio and molar mass), ship propellants and fuels (density and energy), the class atlas (its `air_main` and makeup axes name substance ids), the V3 importer and the converter (V3 gas names map to substance ids; an unknown V3 gas imports as a name-only authored substance and is listed in the report, instead of being dropped).

---

## 7. Changing the real data

- **Any real value can be edited** on the Build desk. The row shows "authored", the real value stays beside it, and one click resets it.
- **Matter is campaign physics, not a per-screen choice.** The table is part of the pack stack and its hash. An edit after a campaign has started goes through the existing physics change request: the kernel lists which known worlds change ("14 charted worlds read water's freezing point"), shows before and after, and nothing changes until accepted.
- **Swapping whole datasets is a pack.** A different solar abundance table, or a "carbon-rich universe", is a small physics pack that replaces those values. Real alternatives stay real (cited); invented ones are marked.
- **What cannot be edited here.** Nuclear burning and how stars evolve with composition are not exposed in 4.0: stars read total metals and the alpha ratio from fitted tables. This is registered as a fudge, not hidden.

---

## 7A. Constants, and what changing G can honestly show

### 7A.1 What goes on the page
- **Constants of nature**, from CODATA: each with value, unit, uncertainty and source. Constants that follow from others (Stefan-Boltzmann from h, k and c; the gas constant from k and Avogadro's number) are **worked out, never typed**, so changing Planck's constant moves them too. A load check keeps them consistent.
- **Reference objects**, from the IAU's nominal values: these are yardsticks, not laws. Changing "the Sun's luminosity" here changes what "1 solar luminosity" means, not the Sun in a Sol system.
- The two dimensionless numbers that really set what a universe is like are shown worked out at the top: the fine-structure constant, and the gravitational coupling (G times the proton mass squared over h-bar c). The page says in one line why they matter: a change of units is not a change of physics, a change in these is.

### 7A.2 One source, read by everything
Every block declares the constants it reads, as it declares its matter properties. The 13 compiled constants and the 60 or so retyped copies are replaced by one handle onto the sheet. With nothing edited, every result and every golden hash is identical to today (the acceptance test of MP1c). Where two copies meant different things under one name (Earth's equatorial and mean radius), they become two named rows.

### 7A.3 The three kinds of block
A changed constant reaches a block in one of three ways. Each block declares which, so the list below is generated by the kernel, not written by hand.

| Reach | Which blocks | What a change does |
|---|---|---|
| **Follows** | every block that is a formula taking the constant as an input: orbital periods and speeds, surface gravity, escape speed, Hill spheres and the Roche limit, tides, the radius at which a body pulls itself round, pressure with depth, air scale height and escape, black-body light and equilibrium temperature, light travel time, Doppler and redshift, hydrogen's spectral lines, the Chandrasekhar mass | the right answer for the new value |
| **Estimated** | tabulated results that have a published scaling with the constant: main-sequence luminosity, radius and lifetime (stellar homology relations; Adams 2008, "Stars in other universes", doi:10.1088/1475-7516/2008/08/010), the mass limits for stars, planet radius from mass | the real-universe table, rescaled by the cited law, and stamped ESTIMATED (the stamp long time jumps already use) |
| **Cannot follow** | results computed or calibrated with the real constants that have no honest rescaling: detailed stellar evolution tracks and the giant phases, how systems form (growth rates and fits tuned to simulations), nucleosynthesis yields and galaxy growth, the climate numbers fitted to Earth, empirical crater and erosion laws, gas opacities and most spectral lines, measured material properties (a new G does not tell us water's new freezing point, and a new fine-structure constant would change all chemistry) | keeps the real-universe result, and is marked on the GM view as "computed with the real constants: not following your change" |

### 7A.4 What it honestly is
Changing G gives a true answer to "what would these orbits, this gravity, this air and these tides do?", an estimate of "what would the stars do?", and no answer to "what universe would have formed?". The result is a hybrid: worlds built by real-universe formation, then run under changed laws. That is still a good teaching tool (double G and the year shortens by a factor of root two, the sky's stars burn hotter and die sooner, small moons round off), and it is the honest limit of a model that ships its heavy physics as tables. The screen says this on the change itself, with the three lists.

So a changed constant always shows a **reach report** before it is accepted: follows (n blocks), estimated (n, with the law), cannot follow (n, with why). The same report is one click away afterwards from the mark on any value.

### 7A.5 Where a change lives
- A changed constant is **authored**, with the real value beside it and a reset, and everything computed from it carries the "uses edited constants" mark on the GM view.
- The natural home for "see what happens if you change G" is a **what-if branch** (they exist): change it there, compare before and after on the chart and the ruler, then throw it away or keep it. Keeping it goes through the physics change request like any matter edit (section 7).
- A physics pack can ship a set of changed constants ("a universe with stronger gravity"), marked fictional, stacked like any other pack.
- Over time, blocks move up the table: a tabulated block that gains a scaling law moves from "cannot follow" to "estimated", and one rewritten as a formula moves to "follows". The reach report makes that progress visible and gives each move a test.

### 7A.6 Units and conversion factors (added on Alex's word, 21 Nov)
Units go on the same sheet, as a page beside Constants. Today about 34 units are compiled into the kernel and a few more sit in packs.

- **What is listed.** The seven SI base units; the named SI units built from them (newton, pascal, joule, watt, tesla, hertz); and the working units of astronomy and of play (astronomical unit, light year, parsec, solar and Earth and Jupiter masses and radii, solar luminosity, bar and atmosphere, electron volt, day and year, g, degree and arcsecond). Each shows what it measures, its factor to SI, and where the factor comes from.
- **Factors are worked out wherever they follow from something else, never typed.** A light year is c times the Julian year. A parsec comes from the astronomical unit. An electron volt comes from the elementary charge. A solar radius is the Sun's row in Reference objects. So a unit cannot drift from the constant behind it, and a load check proves it.
- **Fundamentals and factors are kept visibly apart**, in three groups on the page: constants of nature (what the universe gives), definitions and conventions (exact by agreement: c in SI, the astronomical unit, the Julian year, the IAU's nominal Sun), and conversion factors (pure arithmetic).
- **Units can be added, not redefined.** A pack or a GM can add a unit (a league, a setting's "klick", a jump-parsec) as a factor on an existing one; it appears in every display that offers a choice of unit. An existing unit cannot be given a different factor, which is already the pack rule: a kilometre that is not 1,000 metres would only corrupt what people read. Calendars stay where they are, in the time pack.
- **What a changed constant does to units.** The SI units are our fixed measuring stick. Change c in a what-if and the metre stays put, while the light year, which is defined through c, changes length; the page shows that as "follows your change". This is the honest reading of "what if light were slower": everything else is held still.
- **Display only.** Units never change a result. The kernel works in SI throughout; a unit is how a number is shown and typed.

### 7A.7 Your own units, as a setting (added on Alex's word, 21 Nov 22:35)
Two things, both display only:

- **Units you define.** On the Units page, "Add a unit" takes a name, a symbol, what it measures and its size in an existing unit ("1 klick = 1 km", "1 jump = 1 parsec", "1 cycle = 28 hours"). It is saved in the campaign pack, can be shared in any pack, and from then on is offered wherever that kind of quantity is shown or typed. It is add-only, as above.
- **A unit scheme for the campaign.** One setting says which unit each kind of quantity is shown in by default: distances inside a system, distances between stars, mass, temperature, pressure, speed, time spans. SSE4 ships a few schemes (Astronomer's, SI, Everyday, and Traveller's from its pack) and a setting can save its own, mixing built-in units with its own. A lens or a player view may choose a different scheme, so the GM can read kelvin while the crew's console shows the setting's own degrees. Nothing stored changes: the kernel keeps SI, and switching scheme changes no hash.

## 7B. Calendars (added on Alex's question, 21 Nov 22:35: is Reference the place to add calendars, as SSE 3 had?)

**Recommendation: yes, with one condition.** A calendar is a setting's way of counting time, exactly as a unit is its way of counting distance. Neither is a fact about nature and neither ever touches physics. So calendars belong on the Reference screen beside Units, in a second group kept visibly apart from the first:

| Group | Pages | What it is |
|---|---|---|
| **Nature** | Constants, Elements, Substances, Mixtures | what the universe gives; real unless marked |
| **This setting's measures** | Units, Calendars | how this campaign counts and shows things; a convention, shown with the pack it came from |

The honesty marks (real, authored, fictional) stay on the Nature group. A measure only says where it came from: built in, a named pack, or this campaign.

**What exists today.** SSE 3 let the GM make and edit calendars, all tied to one time anchor, with shown time kept apart from actual time. SSE4 keeps the idea underneath: time is seconds from one origin, and a calendar is data that turns those seconds into a date (name, epoch, day length, days in a year, months, weeks, named days, leap rules, a date format). But today a calendar can only arrive inside a pack, only the top pack's calendar is used, and the chart room has no place to see or edit one. (This is from SSE4's code and its V3 parity notes; I did not read the V3 source for this.)

**What the Calendars page adds.**

1. **A list of every calendar the stack offers**, with the pack each came from: Earth's Gregorian calendar, the Traveller pack's Imperial calendar, and any the campaign has made.
2. **Make and edit one**, saved in the campaign pack. The form is the existing calendar data and nothing more, so anything made here can be shared in a pack.
3. **Make a calendar from a world.** Pick a body and the page reads its real day (from its spin), its real year (from its orbit) and its moons' months, and proposes a calendar that fits, with the leap rule that keeps it in step. A colony on a world with a 31-hour day and a 502-day year gets a calendar that works, with "Why?" on each figure. The GM then names the months. This is the physics earning its place on the page.
4. **Several calendars at once.** One is the campaign's and drives the time ruler. Any other can be shown beside it, and a lens or a player view may show a different one, so two cultures can each read the same moment in their own dates. Changing which calendar is shown changes nothing stored.
5. **One anchor.** Every calendar hangs off the same universe clock. A date is never stored, only the moment. "Today in each calendar" is shown down the page, which also makes the anchor easy to check.

**What does not move.** The time ruler stays where it is and still shows the campaign's calendar. The campaign's "now" stays a record event. Calendars from V3 saves keep importing through the importer's existing `temporal` mapping, and now land where the GM can see and edit them.

---

## 8. The Reference screen

One screen in the Workshop, with its own `rail` entry in its screen document (no new tab). Its first page is **Constants** (section 7A): the constants in groups, each with value, unit, uncertainty, source and "used by"; the two dimensionless numbers at the top; and on any edited row the reach report. Then three matter views, and the same cards open from anywhere a substance is named (a gas on a world's air card, a layer in the cutaway, a resource label).

### 8.1 Periodic table
The familiar grid of 118. Fictional and hypothetical elements sit in a **campaign row** beneath it, drawn like the lanthanide rows, each with its mark. Lenses recolour the grid:

- **Where was it made?** This is the **default lens** (Alex, 21 Nov: colour the table by source). Six colours: the Big Bang, cosmic rays breaking up heavier atoms, dying low-mass stars, exploding massive stars, exploding white dwarfs, merging neutron stars; a grey for the elements people make. Most elements have more than one source, so each cell is filled in proportion to its shares, as in the well-known origin-of-the-elements chart (shares from Kobayashi, Karakas and Lugaro 2020, checked by the citations track). With a system selected it answers for **this star**, from the galaxy model's own ledger: how much of its gold came from how many mergers in its part of the galaxy, and when. This is the "left with a PhD" view.
- **How common?** In the cosmos, in this star, in this world, in this world's crust.
- **Where is it in this world?** Core, mantle, crust, sea, air.
- **Where does it condense?** The disc's temperature sequence, with this system's ice lines marked.
- **What is worth mining here?** The inventory, on the Build and Run desks, behind the survey tier for players.

**Origin stories, the place to slip the teaching in (Alex, 21 Nov).** Every element card opens, at Curious depth, with two or three sentences on where its atoms came from, written from the element's source shares and, when a system is selected, from that star's history. Gold: "Almost all of this was made in about a second, when two dead stars, each heavier than the Sun and smaller than a city, spiralled together and merged. The wreck probably collapsed into a black hole. The gold was flung out, drifted for hundreds of millions of years, and was swept into the cloud that made this star. About 40 such mergers seeded this part of the galaxy before it formed." The number is the model's, not decoration. Science depth adds the process names, the yields and the papers; Engine adds the ring's ledger.

Three rules keep these honest:
- **Say what is settled and what is not.** A merger making heavy elements has been seen once directly (the 2017 event GW170817 and its glow). Whether mergers make nearly all the gold, or rare kinds of supernova add a share, is still argued, and the card says so in a line. The remnant "probably" became a black hole, never "did".
- **Same colours everywhere.** The six origin colours are one legend for the whole app: on the periodic table, in a world's composition bar, on a resource card ("this iron: two parts exploding white dwarfs, one part massive stars"), and on the ore in a mine. A crew surveying an asteroid can be told where its platinum was forged.
- **Written once, as data.** The stories live on the element rows in the three wordings (physics, plain, story), so packs can reword them, Minerva can quote them, Tars can answer from them, and a fictional element gets a story from whatever source its row names.

### 8.2 Substance browser
A searchable list with filters built from the data (has a vapour curve, is a mineral, is fictional, is used by weather). The card's centrepiece is the **phase diagram**, with the selected world's surface, cloud tops and sea floor plotted as dots. "Why does it rain methane on Titan?" is answered by where the dot sits. Below it: every property group with values, and "used by" and "skipped by" lists.

### 8.3 Mixtures
Rocks, airs, seas and the six formation materials, each shown as a recipe bar that opens into its substances.

### 8.4 Depth, desks and marks
- **Curious**: a sentence and four facts in words ("Freezes at 0 °C. Ice floats. The commonest ice past the frost line."). **Science**: values with units and sources. **Engine**: ids, publishers, pack and hash.
- **Explore** reads. **Build** edits, adds, resets and takes Orac drafts. **Run** shows only cards reached from play.
- "Why?" on a value shows the source with its DOI or URL, the register's check status, the validity range, and every block that reads it.
- Players see substance cards as ordinary data-view objects under the usual tiers. They never see the fictional or authored marks: to the crew, unobtainium is simply real.

### 8.5 Adding an element or substance
Orac first, people second. "A dense violet metal made in neutron-star mergers, a thousand times rarer than gold, that sublimes at 400 K" becomes a draft row with every value filled in by analogy to named real neighbours, each value marked fictional with "modelled on osmium" as its "Why?". The draft passes the section 4.6 checks before it can be accepted. The form then shows the ladder from section 6: which blocks will use it and what to fill in to switch on the next one. By default a new substance is created as **resource only** (no effect on physics) unless the request asks for more.

A mockup of this screen should be drawn before MP10 is built (question 9).

---

## 9. Standing rules this design keeps

- **Never fake it.** Real values are cited and checked; anything else is marked.
- **Processes, not classes.** "Volatile", "ice", "rock" and "metal" are labels read off a row's curves, not flags a block tests. Phase edges are the real ones (triple and critical points), computed from the row.
- **One mechanism.** One table, one inventory block, one override rule, one mark propagation. No special path for fictional matter.
- **Best calc is no calc.** Element totals are worked out on a click and cached; nothing is tracked through history.
- **Config for AI first.** The schema is flat and declarative, every property has a unit and range, and Orac's drafts are validated by the kernel before display.
- **Packs stack.** Base matter, then physics packs, universe packs, the campaign pack. Honesty marks on the GM view only.
- **Tags never feed physics.** A resource label is read off the inventory; nothing here reads a tag.

---

## 10. Build packages

One pull request each, with its evidence packet. Prefix MP. Design is by this thread on Fable; the building is by the Opus threads named.

| Id | Package | Owner thread | Needs | Acceptance checks |
|---|---|---|---|---|
| MP0 | **Audit.** A script and a table in the package log listing every physical constant and every substance, element and isotope constant in crates, packs and the renderer, with file, value, unit and source, and every disagreement | Verify physics citations | none | the list covers every row of section 2; rerunning it in CI after MP3 to MP6 shows the retired constants gone |
| MP1 | **Schema and reader.** Crate `sse-reference`, pack `packs/base-reference`, tables under `provides.reference`; the value shape; per-value `replaces`; the load checks of section 4.6; blocks declare the properties they read; kernel op `read_matter` (a table, or one row with sources, used-by and skipped-by); the table in the stack hash | Build kernel resolve and rules | K5 | a fixture with a wrong molar mass, a vapour curve that misses its triple point, a recipe summing to 0.9, an unbalanced reaction and an undeclared substance id fails with five readable messages; an override without `replaces` is refused; the same stack hashes the same on two devices; **no golden changes** |
| MP1c | **Constants and units as data.** The Constants and Reference objects tables; the Units table (section 7A.6: the compiled units moved to the sheet, factors worked out from constants where they follow, add-only for packs); derived constants worked out and checked; one handle every block reads; each block declares the constants it reads and its reach (follows, estimated with its law, cannot follow); kernel op `read_reach` for a proposed change; a lint that no crate outside `sse-reference` types a physical constant; Earth's two radii named apart | Build kernel resolve and rules, with each owning thread converting its own crates | MP0, MP1 | with nothing edited, **every golden hash is identical**; with G doubled on the Sol fixture: Earth's year is 1/√2 of a year, its surface gravity doubles, its air scale height halves, and the Sun's luminosity is reported ESTIMATED with the law named; the reach report lists every block in exactly one of the three groups; halving h moves the Stefan-Boltzmann constant by the factor its formula gives; the lint finds none of MP0's retyped copies |
| MP2 | **Data, first wave.** All 118 elements, each with its source shares and its origin story in three wordings (each story states what is settled and what is argued); about 30 isotopes; every substance the code reads today (13 gases, 5 condensables, the 26 life species, rock, ice, metal, magma); the six formation materials and dry air as mixtures. Values equal today's wherever today's is right, so nothing shifts yet. Every value has a register entry with a DOI or URL; the four "from memory" gas values are checked | Verify physics citations, with the kernel thread | MP0, MP1 | no value without a source; the register lists each with its status; a table in the log shows every value that differs from the constant it will replace, and why |
| MP3 | **Volatiles, inside W8.** `sse.volatile.species` as the view of matter rows with a vapour curve; Fray and Schmitt fits for N2, CH4, CO, CO2, H2O, NH3 and the rest of their list as needed; climate's `substances` retired; frost, cloud and ice lines read one curve | Build shared climate and weather | MP1, MP2; W8 | W8's own checks (Mars, Pluto, Triton); water's frost point in the cloud block and in the ground-ice block is the same number by construction; adding a fixture volatile with a curve gives it frost and cloud with no code change; goldens change once, with W8 |
| MP4 | **Light.** Gas optics from the optical group; lines keyed to element and substance ids; the renderer's gas map removed | Build light and nebulae (kernel), Build photoreal renderer (its side) | MP1, MP2 | `read_air_optics` returns the same numbers as before for Earth, Mars, Venus and Titan; a fixture gas with refractivity scatters; a fixture element with a line shows it in `read_spectrum`; no gas id is spelled in the renderer |
| MP5 | **Life.** `life_chemistry.species` from the chemical group; reactions checked for elements and charge; a biochemistry's solvent window read from the solvent's phase row | Build kernel resolve and rules (it owns the E packages) | MP1, MP2 | every E0 reference reaction keeps its published energy; carbon-in-water's window comes out at today's values from water's row; a fixture ammonia biochemistry gets a colder window from ammonia's row |
| MP6 | **Formation, interiors and composition.** The six materials' condensation temperatures and densities, the heat isotopes, and rock and ice strength and density all read from the pack; new op `read_composition` (elements and substances by body, and by reservoir at Survey), with the partition block | Build SSE4 proven core | MP1, MP2 | Sol fixture: bulk Earth iron, magnesium, silicon and oxygen within a stated tolerance of McDonough and Sun; Earth's core holds most of its gold; totals at Orbit equal the sum over reservoirs at Survey; a click on a body that was never surveyed runs no surface stage; goldens change only where a constant was wrong, each listed |
| MP7 | **Galaxy link.** Element rows carry their nucleosynthesis shares; a star's inheritance returns any element's abundance | Build kernel resolve and rules (it owns G) | MP2 | the Sun returns solar values for every element; two rings with different merger counts give different gold; no G golden changes |
| MP8 | **Inventory, as TG11.** TG11's inventory blocks are replaced by one block over `read_composition`; resource kinds name matter ids; enrichment rules for the deposit processes the surface already places | Build SSE4 proven core (blocks), Build Traveller pack (kept kinds and programs) | MP6, TG1 | every TG11 check as written; plus a resource kind added by a fixture pack for an element nobody wrote a block for gets an inventory |
| MP9 | **Authored and fictional matter.** Campaign-pack overrides; the three states and the impossible mark; per-value propagation of marks; the change request with the list of affected worlds; the "unobtainium" fixture pack | Build kernel resolve and rules | MP1, MP6, MP7 | see the end-to-end test below |
| MP10 | **Reference screen.** Constants and Units pages in their three groups; periodic table with the campaign row and five lenses, "where was it made" as the default with cells filled by source share and the one origin legend reused on composition bars and resource cards; origin stories at Curious depth that quote the selected star's own merger and supernova counts; substance browser with phase diagram and world dots, mixtures, edit and reset, "Why?" at three depths; cards open from air, cutaway and resource labels | Rebuild the chart room's top level | MP1, MP2 for reading; MP9 for editing; a mockup first | screenshots at desktop and phone in both GM modes; a `rail` entry and no tab; an edited value shows the real one beside it; "as players see it" shows no fictional or authored mark; keyboard operable |
| MP11 | **Orac drafts matter.** Element and substance rows from a sentence, by analogy, validated by the kernel before display; default "resource only" | Build Orac, Minerva and Tars | MP9 | an invalid draft never reaches accept; every drafted value names what it was modelled on; nothing changes without accept |
| MP12 | **Packs and Explorers.** Universe packs may carry matter rows; a "deposit" template for dressing rules; the pack listing says "adds 2 fictional elements, changes 1 real value" | Build Traveller pack (inside TG13), Integrate Explorers site with SSE4 (inside TG15) | MP9, TG12 | one of the two shipped universe packs carries a fictional element and a dressed deposit; the listing shows the counts from the manifest |
| MP15 | **Measures: your own units, unit schemes and calendars.** "Add a unit" into the campaign pack; unit schemes as pack data with a campaign default and an override per lens and player view; the stack offers every pack's calendar instead of only the top one, with one chosen as the campaign's; the Units and Calendars pages, grouped as "This setting's measures"; make, edit and "make a calendar from this world" | Build kernel resolve and rules (calendar list, schemes, checks), Rebuild the chart room's top level (pages), Build Traveller pack (its scheme; it owns the date formatter) | MP1c; the pack calendar work already on main | a unit added in the campaign pack is offered on every display of that dimension and refused if it redefines an existing unit; switching scheme or shown calendar changes no state hash; two stacked calendars both list, and one moment reads correctly in each; a calendar made from the Earth fixture proposes a 24-hour day, a 365-day year and the Gregorian leap rule to within its stated tolerance, and one made from Mars a 24.66-hour day and a 668.6-day year; a player view set to another calendar shows the same moment as the GM's; a V3 save's custom calendar imports and appears on the page; screenshots at desktop and phone in both GM modes |
| MP13 | **Data, later waves**, each landing with the stage that reads it: minerals and rock types with hardness (S9), compression curves (interiors), salts (S21, ocean chemistry), organics and tholin (S19), absorption bands per gas (greenhouse) | Verify physics citations, with the reading thread | MP2 | per wave: every value sourced; the reading stage's own checks |
| MP14 | **Smooth condensation (later).** Formation shares each element by its own condensation curve instead of six steps | Build SSE4 proven core | MP6 | Sol fixture still matches; the six-material fudge is removed from the register |

**Order.** MP0 can start now. MP1, then MP1c and MP2 side by side. Then MP3, MP4, MP5 and MP6 in parallel, one per thread. MP7, then MP8 (as TG11) and MP9. MP10 can show a read-only table as soon as MP2 lands. MP11 and MP12 follow MP9.

**Two timing notes.** W8 and TG11 are both queued now. If either starts before MP1 and MP2 land, it builds its table in the matter schema's shape inside its own pack, and MP3 or MP8 becomes a move, not a rewrite.

**The end-to-end test (MP9).** A fixture pack adds unobtainium: made in neutron-star mergers at one thousandth of gold, rides with metal, iron-loving, no other property.

1. Two galaxy rings with different merger histories give stars different amounts.
2. In a Sol-like system most of it is in cores, and metal-rich asteroids hold the most per kilogram.
3. A surveyed audience sees `resource.unobtainium` with a banded estimate; an unsurveyed one sees nothing.
4. Mining a rock below the pack's threshold removes the label.
5. Every value derived from it carries the fictional mark on the GM view and none on a player view.
6. **Adding the pack changes no golden hash and no existing fact.** Removing it leaves a valid campaign.
7. Then give it a heat-producing isotope: heat flow rises on worlds rich in it, those worlds' heat and geology are marked fictional, and worlds without it are untouched.

**Cost.** The first-wave table is a few hundred kilobytes of JSON, loaded once. `read_composition` at Orbit is a cheap cost class and never triggers a surface run.

---

## 11. Sources the data waves start from

As recalled by this thread; each is checked against the source by the citations track before it enters the register, as the standing rule requires.

- Constants: CODATA 2018, Tiesinga et al. 2021, doi:10.1103/RevModPhys.93.025010; IAU 2015 Resolution B3 nominal values, Prsa et al. 2016, doi:10.3847/0004-6256/152/2/41
- Atomic weights: Prohaska et al. 2022, IUPAC standard atomic weights 2021, doi:10.1515/pac-2019-0603
- Solar abundances: Asplund et al. 2009 (already the code's scale); Lodders 2021, doi:10.1007/s11214-021-00825-8
- Condensation temperatures: Lodders 2003, doi:10.1086/375492; Wood, Smythe and Harrison 2019, doi:10.2138/am-2019-6852CCBY
- Where elements are made: Kobayashi, Karakas and Lugaro 2020, doi:10.3847/1538-4357/abae65
- Earth's composition and partitioning: McDonough and Sun 1995 (already registered); Rudnick and Gao 2014, doi:10.1016/B978-0-08-095975-7.00301-6
- Vapour pressures of ices: Fray and Schmitt 2009, doi:10.1016/j.pss.2009.09.011
- Thermochemistry: CODATA key values (Cox, Wagman and Medvedev 1989); Thauer (already registered); Holland and Powell 2011 for minerals, doi:10.1111/j.1525-1314.2010.00923.x
- Phase and thermal data: NIST Chemistry WebBook, https://webbook.nist.gov
- Gas absorption: HITRAN2020, Gordon et al. 2022, doi:10.1016/j.jqsrt.2021.107949
- Heat isotopes: Ruedas 2017 and Turcotte and Schubert 2014 (already registered)

---

## 12. Rulings

Alex, 21 Nov 2026, 20:51: "yes to all". Every question below is decided as recommended, and the recommendation is the ruling. He added two things, both now in the design: units and conversion factors on the sheet beside Constants (section 7A.6), and the periodic table colour coded by where each element was made, used to carry the teaching (section 8.1).

1. **Decided, Alex 21 Nov:** build it as one user-editable master data sheet, constants included. W8's volatile table and TG11's inventory fold into it.
2. **First wave: all 118 elements, about 30 isotopes and only the substances the code already reads, with minerals, salts and absorption bands arriving with the stages that need them?** Recommended: yes. A complete periodic table is cheap and is the showpiece; a complete chemistry handbook is not needed and would be unsourced filler.
3. **Elements ride inside formation's six materials and are worked out on a click, instead of formation tracking every element?** Recommended: yes. It keeps "best calc is no calc", changes no golden, and the coarse split is registered as a fudge that MP14 can remove later.
4. **May a fictional substance change real physics (heat a core, rain, feed life) when its row has the properties, always marked?** Recommended: yes, with new substances created "resource only" unless asked for more. The alternative, fictional matter as inventory only, is safer but throws away the fun.
5. **When a real value is edited, should everything computed from it carry a mark, the same way fictional physics does?** Recommended: yes, GM view only, with the wording "uses edited matter" so it is not confused with a fictional substance.
6. **Edits to matter after a campaign has started go through the physics change request, with the list of charted worlds that would change?** Recommended: yes. Changing water's freezing point should never be a silent one-click event.
7. **Fictional elements sit in a "campaign row" under the table with no atomic number unless the pack gives one, and predicted superheavies are allowed as "hypothetical"?** Recommended: yes. It keeps the real 118 visibly intact.
8. **Data licences: cite the primary paper for every value and use NIST and HITRAN as finding aids, with the citations thread keeping a licence note per source (HITRAN-derived band tables checked before they ship)?** Recommended: yes, the same care as the Gaia ruling. Facts are free to use; some compilations set terms on redistribution.
9. **Shall I draw the Reference screen as a mockup (periodic table with lenses, phase diagram card, add-an-element flow) before MP10 is built?** Recommended: yes, once the answers above are in.
10. **Stars: leave nuclear burning out of the editable sheet in 4.0, registered as a fudge?** Recommended: yes. Stars read total metals and the alpha ratio from fitted tables; an editable fusion network is a different project. A changed constant still reaches stars as an estimate (section 7A).
11. **Changed constants work as section 7A says: three kinds of block (follows, estimated, cannot follow), a reach report before the change is accepted, and what-if branches as the usual place to try one?** Recommended: yes. It lets people change G and learn something true, and never pretends the tables followed.
12. **Keep fitted and tuned model numbers off the sheet, in their own packs and the fudge register, linked from "used by"?** Recommended: yes. The sheet stays "what nature gives us"; mixing in our own tuning would blur the line the honesty marks exist to keep.
13. **Call the screen "Reference" (pages: Constants, Elements, Substances, Mixtures)?** Recommended: yes; "Matter" no longer fits once constants are on it.

### Added 21 Nov 22:35, and decided as recommended (Alex, 21 Nov 22:49: "Yes to all")

14. **Calendars get a page on the Reference screen, grouped with Units as "This setting's measures" and kept apart from the Nature pages?** Recommended: yes. It gives calendars the home SSE 3 had, without blurring "what nature gives" with "how this setting counts".
15. **Several calendars loaded at once: one is the campaign's, others can be shown beside it or chosen by a lens or a player view?** Recommended: yes. Today only the top pack's calendar exists.
16. **"Make a calendar from this world": propose day, year, months and leap rule from a body's real spin, orbit and moons?** Recommended: yes. It is cheap, it is real physics, and no other tool does it.
17. **Unit schemes: one campaign default for which units are shown, with a lens or player view able to choose another?** Recommended: yes.

Synced to `handover/design/` with track MP in the build plan (PR 89, then the rulings in a second PR). Next from this thread: the Reference screen mockup (ruling 9), before MP10 is built.
