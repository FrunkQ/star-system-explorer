# SSE4 future technology: defined by the problems it solves

Status: **accepted 23 Nov 2026**. Alex answered "yes to all" at 16:15 to the fourteen questions in section 12 (read by the coordinator as covering this list among three open at once; he can correct that). Written by the design thread. Track letters **FT**. Design only; the packages are in section 13. Every figure still passes the citation check before it ships. A mockup goes with it: "SSE4 ship styles and tech levels" (one generated ship in five styles and at three tech levels), source in `/mnt/project-files/sse4/ships/styles-and-tech-mockup.html`.

**Revision, 23 Nov 18:48 (Alex):** the ten times per level rule was "a simple rule of thumb for ideas. Not a hard fact. Apply intelligently and improve if possible." Section 5.2 now gives the size rule by family of part; its four questions (15 to 18) were accepted 23 Nov 19:20 (Alex: "size rules make sense", read by the coordinator as yes to all four).

**Revision, 23 Nov 20:14 (Alex: "that all makes sense - push on", read by the coordinator as yes to all seven):** a critical review of the accepted design found that it tested each number alone, while physics lives in how the numbers relate. Section 15 adds seven improvements, questions 19 to 25, all accepted. Where section 15 and an earlier section differ, section 15 wins.

**Ruling, 23 Nov 17:49 (Alex):** "4.1 alien life will have to have culture and tech as part of life. So all these systems can be extended then and we can have a lighter touch." So 4.0 keeps civilisation capability light: levels and advances are set by hand or by a pack. Automated research, tech trees and a civilisation growing its own technology are 4.1, grown from life.

Read with: the voyages design (drive facts, FTL as a pack rule), the nebulae and light design (every object publishes its light by band, section 5A sensing), the matter pack design (the master data sheet and its one "Look up" door), the programs design (track J), the naming design (settings inherit down the tree), and the ship generator in `packages/renderer/src/ship-design.ts` (parts sized by physics, a style only arranges and dresses them).

Sources below were **[from memory, verify]**; the citation check (FT1, 23 Nov, section 14) has now read or re-worked each one, and corrected those that were off. A figure marked *computed* is this note's own arithmetic, re-run in the check; *textbook* means a standard value not read from a paper.

## 1. What Alex asked for (23 Nov, 15:46 to 16:00)

- "Maybe that is how we can define advanced tech... which 'gotchas' have they solved to reduce its size? Like the old GURPS tech ladder - any tech can appear at a lower tech level but it is 10 times the size."
- "Define 'what problem it solves' ... That effectively defines which 'physics we can skip'. eg: ultradense energy sources (fuel tanks are a thing of the past), heat dissipation (no radiators... easy aerobraking)."
- "Not JUST drives! Although FTL is a mega-punch there. Material strong enough to build a ring world or beanstalk. They open up as the physics allow."
- "Maybe ships can be seen through other lenses too - massive xray plumes."
- "Constructs = land and mega and ships and asteroids with engines, etc... similar rules across ALL."
- "Megaconstruct availability is defined by tech progress ... it will have 'tech x' not available... develop it? So we tie constructs to civilisation capability."

Already agreed by him the same day: technology is the list of problems solved; a tech ladder per setting; skipped rules always visible; styles stay cosmetic.

## 2. The idea in one paragraph

Everything anyone builds in space fights the same short list of **limits**: heat has to leave, thrust needs something to throw, energy has weight, crews break, materials only bear so much, nothing outruns light. SSE4 already sizes a ship from those limits (tanks from fuel mass, radiators from waste heat). So technology needs no model of how a future device works. A setting only says **how far each limit has been pushed**, and the same sizing blocks that draw today's ship draw tomorrow's. Push the heat limit and the radiators shrink; push it to nothing and they are gone, with no rule that says "hide radiators". Push material strength and a beanstalk becomes buildable on Mars, then on Earth, and a ringworld stays out of reach until a setting says otherwise in so many words.

**Technology is physics scoped to a builder.** A physics pack changes the universe for everyone (a different G). An advance changes what one civilisation's constructs can do. Natural bodies never read it.

## 3. Four words

| Word | Meaning | Example |
|---|---|---|
| **Limit** | One of the standing problems of building things, in plain words, with the law behind it on its Science line | "Heat has to leave" (second law of thermodynamics; Stefan-Boltzmann radiation) |
| **Figure** | A number with a unit that a sizing block already reads, and that a limit holds back | radiator temperature in kelvin; waste heat per watt of jet; a material's strength for its weight |
| **Mark** | A cited point on a figure's scale: **built** (flown or in service), **allowed** (no known law forbids it, nobody has built it), **ceiling** (where a named law stops it) | exhaust speed: built 4.5 km/s chemical, allowed about 0.03 c for a fusion torch, ceiling c |
| **Advance** | Pack data: a named thing a civilisation can do. It sets one or more figures, sits at a level on the setting's ladder, and is carried by a part with a size | "Fusion torch", "Woven carbon cable", "Gravity plates", "Jump drive" |

No switches. An advance never turns a rule off; it moves a figure, and the block still runs. This is the processes-not-classes rule applied to machines, and the same shape as "shielding is an effect, not a switch" and "no can-land flags". A radiator block handed zero waste heat returns zero area, and nothing is drawn.

Where a figure passes its ceiling, the advance is **fictional by arithmetic**, not by someone remembering to tick a box. The card names the law it breaks (section 8).

## 4. The limits

Ten to start. The list is pack data (`base-tech`), so a setting can add one, as long as it points at a figure some block reads.

| # | Limit, in plain words | Science line | Figures it holds back | What it shapes | Past the ceiling |
|---|---|---|---|---|---|
| 1 | **Heat has to leave** | Second law; radiated power goes as area times temperature to the fourth | waste heat per watt of jet or of power; radiator temperature; a heat shield's peak heating and total load | radiators (2 TW of waste at 1500 K needs about 3.9 km² of panel radiating from both faces at emissivity 0.9, 7 km² of one-sided black panel; computed); heat shields and aerobraking; how bright it is in infrared | heat sinks that send it "nowhere" |
| 2 | **Thrust needs something to throw** | Conservation of momentum; the rocket equation | exhaust speed (ceiling c); thrust per watt with nothing thrown (ceiling 1/c, a photon drive, about 3.3 N per GW) | tanks, mass ratio, staging; lifting off a world at all | reactionless and gravity drives; contragravity |
| 3 | **Energy has weight** | Mass-energy; energy per kilogram of fuel, power per kilogram of plant | energy per kg (chemical 13 MJ, fission about 8×10¹³ J, deuterium and helium-3 about 3.5×10¹⁴ J, ceiling c² = 9×10¹⁶ J for matter with antimatter); power per kg of power plant | fuel tanks, reactor size, range | anything richer than annihilation |
| 4 | **Crews break under hard acceleration** | Physiology of the crew's species; inertia cannot be screened (equivalence principle) | sustained and peak g for the crew, from the species, raised by couches and liquid immersion | trip times; engine size; the length of a launch track (3 g to orbital speed at Earth is about 1,000 km of track; computed, 7.8 km/s gives 1,030 km) | inertial compensators |
| 5 | **Crews need weight** | Spin gravity: radius is g over spin rate squared; comfort caps the spin rate | highest comfortable spin (up to about 4 turns a minute for anyone, 6 for trained residents; Globus and Hall 2017); lowest healthy gravity | spin rings and tumbling pairs (1 g at 2 turns a minute needs a 224 m arm, at 4 a 56 m arm; computed) | gravity plates |
| 6 | **Nothing outruns light** | Special relativity | none: this one has no figure to push, only a pack rule | travel between stars; message delay; a crew's age against home | every FTL drive and FTL radio; already a per-setting rule in the voyages design |
| 7 | **Materials only bear so much** | Strength for weight (pascals over density); bond energy sets a ceiling near 50 to 100 MJ/kg | a substance's strength for its weight, from the master data sheet (steel about 0.25 MJ/kg, aramid about 2.5, carbon nanotube about 45, allowed and not built at length) | tank and structure mass; how hard a hull can be pushed; pressure hulls and domes; tower height; spin habitat radius; tethers; every megastructure (section 6) | ringworld floor, rigid shells round a star |
| 8 | **Radiation gets through** | Stopping of charged particles and gammas by mass in the way | shielding in g/cm² for a given dose; a magnetic screen's bending strength (allowed) | mass round the quarters; storm shelters; how close a ship may pass a hot star or a giant's belts | force screens |
| 9 | **Crews need air, food and time** | Mass balance of a closed habitat; ageing | supplies per person a day; how much is recycled (ceiling all of it); room per person; whether crews can sleep the trip (allowed, not built) | habitat and stores size; greenhouse area; who is awake | stasis with no upkeep |
| 10 | **Anything warm can be seen** | Every body radiates at its temperature; the light design's sensing arithmetic | none of its own: it reads the heat and drive figures | detection range by band; hiding | cloaks, which must say where the light goes |

Notes on the list:

- **The crew limits belong to the cargo, not the ship.** A drone has no limit 4, 5 or 9, so it comes out small and fast with no fiction at all. An alien species with a different tolerance changes ships with no tech entry. This is why limit 4 reads the species.
- **Limit 6 has no single number to push, so its advance is a rule.** The rule still has a rating (Jump-2 against Jump-4, a warp factor), so it too comes in degrees. Section 5.1 gives its form.
- **Left out on purpose:** build time and cost (the economy pack, 4.1), minds and computing, medicine, weapons and armour (no dogfighting). A pack may add a limit for any of them later through the same shape.

Most of these figures already exist. `base-ships` `drive_heat` carries waste per jet and radiator temperature per fuel; the ship record carries exhaust speed, acceleration and shielding; `NavShip` carries shield heating limits and `structure_g`; strength for weight is a substance property that belongs on the master data sheet and is read through "Look up". FT1 mostly names what is there and gives each figure its marks.

## 5. The tech ladder

1. **A setting defines its own ladder**: how many levels and what they are called. Traveller has its TL numbers; "A working Sol" might have three ("orbital", "torch", "deep"). The base pack ships a neutral ladder with the real past on its lower rungs, so "Sol today" is simply a level.
2. **Every advance sits at a home level**, real ones included. A chemical rocket, a fission reactor and a fusion torch are advances like any other; only their marks differ.
3. **Every advance is carried by a part with a size**: a drive, a reactor, a cable, a plate under the deck, a heat sink. The pack gives the part's size at the home level, usually per watt, per newton or per tonne of ship.
4. **Short of the home level, the advance is immature, and its part comes out bigger.** How much bigger is section 5.2: where history or physics says, the advance's own figure is worse and the sizing block gives the size; where nothing is known, the part is ten times the mass and volume per level short, which was Alex's rule of thumb and stays the editable default. One level short is a prototype. It is never forbidden; the app shows the size and lets the GM build it. This is the ruling "danger is drama, never forbidden" applied to ambition.
5. **For an advance with no separate device** (a better material), the immature figure is the material's own strength for weight, and the structure block does the rest. This matters: a cable a fifth as strong does not make a beanstalk five times heavier, it makes it many thousands of times heavier, because taper is exponential. A flat multiplier on size would get that wrong.
6. **Past the home level**, the figure keeps improving until it meets its cited ceiling (section 5.2).

Because the generator already draws every part from its size, a prototype looks like one: the same ship with one grotesquely swollen system.

### 5.2 How much worse an early version is (revised 23 Nov 18:48, accepted 19:20)

Alex: "The 10 times was a simple rule of thumb for ideas. Not a hard fact. Apply intelligently and improve if possible."

**The improvement: immaturity acts on the advance's figure, not on the part's size.** This is the same move as the rest of the design (an advance moves a number, the block still runs), so the size of an early version falls out of physics and differs by kind of part, as it does in real history. Four things follow that a flat multiplier cannot give: a weak early cable costs exponentially, not ten times; early shielding costs nothing extra, because mass is mass; an early engine also drags bigger tanks behind it; and a prototype can never come out better than physics allows.

**What history says, by family of part.** Figures checked by the data sheet thread on 23 Nov (sources in section 14.1).

| Family | What the first working generation managed, against the mature one | Examples | Default for one level short |
|---|---|---|---|
| **Engines and power plants** (drives, reactors, generators): output for their mass | about a seventh | rocket engine thrust for weight: V-2 about 22, Merlin 1D over 150 (about 180 claimed after its 2013 upgrade), so 7 to 8 times. Jet engines: Jumo 004 about 1.2, Eurofighter's EJ200 about 9 with reheat (6 dry). Space reactors: SNAP-10A, the only one the US flew, about 1.2 W per kg; designs since run from 1 to 7 W per kg for small Kilopower-class units to about 33 for the 100 kW SP-100, none of them flown | figure divided by 7 |
| **Figures held down by a nearby ceiling** (exhaust speed for a given fuel, conversion efficiency, how much a habitat recycles) | half to two-thirds, because the ceiling was close from the start | chemical exhaust speed: V-2 about 2.0 km/s at sea level (2.3 in vacuum), the best hydrogen engine 4.6 km/s in vacuum (RL10B-2, 465.5 s), so the V-2 had about half; ceiling near 5 for hydrogen and oxygen. Heat engines against Carnot | two-thirds of the way from the old figure to the mature one |
| **Materials in bulk** (strength for weight of something you can buy by the kilometre) | about a fifth; the laboratory figure arrives decades before the bulk one | carbon fibre: Shindo's first PAN fibres of the early 1960s 0.5 to 1 GPa, the first big commercial grade (Toray T300, 1971) 3.5 GPa, the best now (T1100G) 7.0 GPa. Nanotubes today: single tubes above 100 GPa, centimetre bundles 80 GPa, the best spun fibre 4.2 GPa, about a twenty-fifth | figure divided by 5 |
| **Stores of energy** (batteries, fuel handling) | about a third | the first lithium-ion cell (Sony, 1991) 80 Wh per kg, about three times that now in mainstream cells; 450 Wh per kg in specialist silicon-anode cells | figure divided by 3 |
| **Plain mass** (radiation shielding by matter, ballast, reaction mass itself) | the same | a g/cm² of shielding stops what it stops in any century | no change |
| **Devices with a least size set by physics** (a critical mass, a plasma big enough to hold its heat) | not smaller, at any maturity | reactors do not shrink below criticality; fusion machines grew as they improved | the advance states its floor |
| **Fictional devices and anything unknown** (gravity plates, heat sinks, an FTL drive) | nothing to go on | | Alex's thumb: ten times the mass and volume per level short, editable per setting and per advance |

**What "first generation" means here.** The first version that does the job at working scale: the first engine that flew, the first fibre sold by the tonne. It is not the first laboratory sample, which is usually far worse (the first carbon fibres were a seventh to a fourteenth of today's), and not a later refined grade, which is usually within a factor of two. The materials default of a fifth sits between those on purpose, and is the weakest-founded number in the table.

**Three rules round it off.**

- **A prototype is never worse than what they already have for the same job.** The immature figure is floored at the best older advance the civilisation holds. Where an early version would only match the old way, the reading says so ("no better yet than their fission plant"), and still lets the GM build it.
- **Past the home level the figure improves by half as much again per level until it meets its cited "allowed" or "ceiling" mark**, then stops. This replaces "half the size per level for two levels": the stop is where physics puts it, not at a count. Fictional devices, having no ceiling, keep the old rule (half the size per level, two levels).
- **A level is the setting's own step**, so every default above is per level and editable on the Technology page. Where a ladder gives its levels rough spans of years, the defaults read as "per generation of about forty years", which is what the history above spans.

The mockup now swells its fusion torch by seven and its fictional heat sink and field drive by ten.

**Questions for Alex on this revision (all accepted 23 Nov 19:20)**

15. **Immaturity acts on the advance's figure and the size falls out**, with the flat ten times kept only where nothing is known. Recommended: yes.
16. **The family defaults in the table** (engines a seventh, materials a fifth, stores a third, near-ceiling figures two-thirds of the way, plain mass unchanged, fictional ten times), each editable. Recommended: yes, subject to the citation check.
17. **A prototype is never worse than what they already have**, and the reading says when an early version is not yet worth building. Recommended: yes.
18. **Past the home level, improve until the cited ceiling** instead of stopping after two levels. Recommended: yes.

### 5.1 Advances are partial, and civilisations differ (Alex, 23 Nov 16:39)

"Different civilisations can (in theory) have different solutions to the same problem ... radiators 10 times as efficient as we have now, 1,000, 100,000x."

**Every advance is a factor on a number, never "solved".** "Solved" is only what a very large factor looks like. Heat, worked from the same radiator block (figures computed here from the Stefan-Boltzmann law for a panel radiating from both faces at emissivity 0.9, and to be re-run in the citation check):

| Heat shed per m² of panel | Panel temperature | Mark | The mockup's 43 GW needs |
|---|---|---|---|
| today, about 0.8 kW (a room-temperature panel) | 300 K | built | 52 km² |
| 10 times | about 530 K | built | 5.2 km² |
| 1,000 times | about 1,700 K | not yet built | 52,000 m² |
| about 23,000 times | 3,695 K, where tungsten melts: the ceiling for a solid panel | ceiling | 2,300 m² |
| 100,000 times | about 5,300 K, hotter than any solid | fictional, unless the advance is a different kind of radiator (droplets, dust) with its own marks | 520 m² |

So a setting picks a point on a scale, the card shows the factor ("sheds heat 1,000 times better than today's panels"), and the ship's radiators shrink to match and vanish only when the area rounds to nothing. Fuel is the same: its data (energy per kg, exhaust speed) is the factor. So is cable: strength for weight is the factor.

**Two civilisations may answer the same limit differently.** Advances belong to a civilisation (section 7), and several advances may address one limit. One navy runs hot solid panels, its neighbour sprays droplet sheets, a third has a fictional heat sink. Each is its own row on the Technology page with its own figures, marks, part size and look, and a construct uses its builder's. Their ships differ for a reason before any style is applied.

**The light-speed advance is an FTL rule, held by a civilisation like any other advance.** Its form is the one accepted in the example packs design (section 4A): a family (jump by rating, hop with a tether, speed by formula, rate per distance, gates, fixed points), a rating on the ship, and six fields: where a leg may start and end, what the crew does, how long it takes, what it costs, what can go wrong, and how messages travel. Consequences:

- **Several FTL rules can coexist in one setting.** One power jumps by rating, its neighbour uses gates it found and cannot build ("found, not built"), a third has none and flies for real. The voyage finder already prices each leg by one means, so a ship is simply offered the legs its own rule allows.
- **It sits on the ladder.** An FTL rule has a home level, so a power one level short can have a jump drive ten times the size (the default for fictional devices, section 5.2).
- **Messages are part of the rule**, so two neighbours can differ in how fast news travels as well as ships.
- **A setting-wide rule is the common case and stays easy**: put the rule on the setting and every civilisation inherits it.

## 6. Every construct, one rule set

"A construct is just a construct." The limits do not know what they are sizing.

| Construct | Limits that bite hardest | What the same blocks give |
|---|---|---|
| **Ship** | 1, 2, 3, 4, 5 | tanks, radiators, spin ring, shield, as built today in the ship generator |
| **Station or habitat** | 5, 7, 8, 9 | largest spin radius is strength for weight over g: about 3 km in steel cable at a working stress with soil and air to carry (O'Neill 1974: his largest model, radius 3,200 m, spinning once in 114 s), about 1,000 km in nanotube (McKendree 1996, as summarised: 1,120 km) |
| **Ground facility** | 1, 3, 7, 9 | heat goes into air or ground where there is any, so radiators shrink on a world with air; dome span from skin tension; tower height from strength for weight over local g; crush depth in an ocean or a giant's air; a launch track's length from limit 4 |
| **Asteroid with engines** | 2, 3, 7 | the rock is the reaction mass or carries it; the push it can take is its own strength, and a rubble pile's is tiny (Sánchez and Scheeres 2014), so the answer is "years of gentle thrust", from physics |
| **Beanstalk (space elevator)** | 7 | taper and cable mass computed per world from its gravity and spin, the way aerobraking is computed per ship per world. The Moon opens with fibres that exist, such as aramid and M5 (Pearson, Levin, Oldson and Wykes 2005); so does Mars by the same taper sum (Zylon tapers about 12 times with no margin, computed); Earth needs nanotube-class cable (Edwards 2000: carbon nanotube composite at 130 GPa; Pearson 1975 for the tapered tower itself). No can-build flag: the app says "the cable would outweigh the planet" where it would |
| **Orbital ring** | 3, 7 | the ring itself is in free fall, faster than orbit, and carries stationary skyhooks magnetically, so it needs power and active control, not a miracle material (Birch 1982) |
| **Dyson swarm** | 3, and mass | no fiction at all; it needs a planet's worth of matter, which is a real amount that can be mined out (the resources ruling) |
| **Stellar engine** | 2 | a mirror gives thrust L(1 − cos θ)/2c for a rim angle θ, at most about L/2c for one spanning half the sky, about 6×10¹⁷ N for the Sun (Shkadov 1987; Badescu and Cathcart 2006): real, and almost nothing. Attached as a thrust block, per the megastructure ruling |
| **Ringworld** | 7 | a 1 g ring at 1 AU needs strength for weight of g times radius, about 1.5×10¹² J/kg, some 30,000 times nanotube and far past the bond ceiling. Fictional by arithmetic; the card says exactly how far |

So megastructures are not a special catalogue. Each states its figures, and what a civilisation can build falls out. The megastructure design note itself stays with the Traveller and kernel threads as ruled; this note supplies its tech side.

## 7. Civilisations carry capability

**Who holds tech.** A setting, a nation, a species or any group in the tree carries: a level on the setting's ladder, and exceptions ("has gravity plates early", "never took up fission"). It inherits down the tree exactly as naming and styles do, so one inheritance mechanism serves all three. A shipyard inherits from its nation unless it says otherwise.

**Capability is a dated fact.** Time is first-class, so a civilisation's level and each advance it gains are dated events on its timeline, like the field history in the magnetic design. Scrub the clock back and they cannot yet build it. Look ahead and they can. A construct records who built it, when, and at what level.

**What a construct needs is computed, never typed.** A design's needs are read off its parts: the advances whose figures it leans on. Nobody maintains a "requires" list that can drift from the truth.

**Availability is a reading with three outcomes, none of them "no".** On the Build desk, any construct design looked at through a civilisation shows one of:

| Reading | Meaning | What the GM can do, one press each |
|---|---|---|
| **They can build this** | every advance held at their level | build it |
| **One level short: Woven carbon cable** (or two, three) | an advance not held yet | **Build the prototype**, at the size section 5.2 gives for each level short, shown before committing; or **Develop it** |
| **Past anything known: needs 30,000 times their strongest material** | no advance on this setting's ladder reaches the figure | **Add an advance to the setting** (marked fictional if past the ceiling); or **Place it as found** |

**Develop it.** In the core app this is a story decision with one honest lever. "Develop it" adds a dated event, "the Vesh gain woven carbon cable in 2310", at the campaign's now or a chosen date. From that date the reading changes, existing prototypes stay as built, and new builds come out at the proper size. No research points in the core. "Develop it" stays the one hand-pressed dated event in 4.0 (a pack may also write the same event as plain data). Automated research and tech trees are 4.1, grown from life with its culture, per the ruling above; nothing in 4.0 builds hooks for them. Orac can propose the event and its consequences ("this opens a beanstalk on four of their worlds").

**Found, not built.** Any construct may carry parts above its owner's level. They work, they cannot be copied or repaired, and the card says "found". This gives the ancient artefact, the captured drive and the gift from elder aliens with no new machinery, and it is authored ink like any GM placement.

**The teaching use.** Because the lower rungs are real history and the marks are cited, the same screen answers "why have we not built a space elevator?" with a number. That is the education half of the project, free.

### 7.1 Where it is set up (Alex, 23 Nov 16:23: "where we set up the universe factions, calendars, languages ... it is in effect a civilisation capability")

Yes: tech is set up where the rest of a civilisation is set up, and nowhere else. Two places, both already designed, and no new tab:

- **The setting's ladder, on Reference under "This setting".** A **Technology** page sits beside Units, Calendars and Names (matter pack design 7A, naming design 7). It shows this setting's levels, every advance at its home level with its marks and its limit, and the two size factors. Like its neighbours it is a convention of the setting, it shows the pack each row came from, and it never touches nature's pages.
- **Each civilisation's own page.** A civilisation is an actor in the record (`/actor:vesh`, and powers nest: `/actor:vesh/actor:third-fleet`), the same thing the territory overlays design hangs holdings on and the naming design hangs a scheme on. Its page gathers everything it carries or inherits, one row each, with the same "inherited from" wording throughout: names and language, calendar, units, ship and construct style, holdings, and **tech level with its exceptions and dated gains**. Changing the level here is the "Develop it" event of section 7. A child (a fleet, a shipyard, a colony) shows the parent's value greyed until it is given its own.

So one inheritance rule and one page per civilisation serve naming, calendars, units, styles, territory and tech. The Build desk's availability reading (section 7) reads from this page and links back to it; it is not a second place to set anything.

## 8. Honesty: always visible

On the GM view, every construct card gains one line, **Leans on**, listing its advances as chips in three inks:

- **Built**: plain. Flown or in service today.
- **Not yet built**: the allowed range. No law forbids it. A new, softer note than the existing marks (question 5).
- **Fictional**: past a ceiling. The chip names the law: "Gravity plates: weight without spin, breaks the equivalence principle". A reactionless drive's "Why?" also says that above 1/c of thrust per watt it gives out more energy than it takes in, because that is true and a GM may care.

Further:

- A voyage leg, a pass or a landing that depends on a fictional advance carries the fictional flag, the same flag FTL legs already carry in `sse-nav`.
- The fudge register gains a second kind of row, **setting fiction**, written automatically for each fictional advance in use and attributed to its pack. Engine shortcuts and a setting's liberties stay visibly different things on one screen.
- **Show it without.** One press on the card redraws the same ship with a chosen advance removed: the radiators it would need, the spin ring, the tanks. It costs nothing, since it is the same generator with the figure put back to its allowed mark, and it is the clearest picture of what an advance buys. In the mockup.
- Players see none of the marks. To them a gravity plate is a fact of their world.

## 9. Seen in other bands

The light design already rules that every object publishes its light by band and that drive output is in "whatever bands the physics pack gives a drive". This design makes that fall out of the same figures, per part:

| Part | Light it publishes | From |
|---|---|---|
| Radiators | thermal glow at their temperature: 1500 K peaks near 1.9 micrometres, bright in infrared and dull red to the eye | limit 1 figures |
| Hull and quarters | faint thermal glow at room temperature; reflected starlight | area and temperature |
| Plume | from the exhaust's energy per particle: chemical and nuclear thermal exhaust is a few thousand kelvin (visible and infrared); a fusion torch's plasma is tens to hundreds of keV, so it is an X-ray source with a faint visible tail; antimatter adds gamma rays | exhaust speed and fuel |
| Reactor and shadow shield | gamma and neutron leakage on the unshielded side | limit 8 figures |
| Heat shield in a pass | glow from peak heating, as `passGlow` does now | the pass |
| Radio | transmitters and radar, as the light design | construct data |

Consequences that need no new rule:

- The ship picture gets the renderer's band switch. In X-ray a torch ship is a searchlight and its hull is invisible; in infrared its radiators are the whole picture.
- **Solving heat also hides you**, and solving thrust removes the plume. So a high-tech ship is quiet in every band unless its advances say otherwise. That is a large consequence for sensing, and it should be a choice, not an accident: every fictional advance must state what light it gives off instead (a jump flash, a gravity drive's glow, or plainly "none"), and "none" shows on the card.
- A megastructure publishes too: a Dyson swarm is a star gone dim in visible and bright in mid infrared, which is how one would be found from another system by the remote survey methods already designed.

## 10. Styles stay cosmetic, on every construct

A style may choose the order and grouping of parts inside the ranges it gives, the form language (round, faceted, ribbed, grown), plating and panel lines, colour, markings and lights, and how a radiator's area is cut into wings, fins or sails. It may never change a size, and it may wrap but never hide a part that must see the sky: radiators, plume, shield, sensors, scoops.

Styles are written against a small vocabulary of parts (pressure vessel, tank, truss, panel, nozzle, dish, leg, shield, ring), not against "ships", so the same style dresses a station, a ground base or a cable anchor. A style is assigned to a nation or species and inherits down the tree, as already ruled.

So two navies at the same level differ by style, and two ships in the same style differ for a reason. The mockup shows five styles (Utilitarian, Sleek, Retro industrial, Organic, Military) across three levels.

## 11. How the example packs read

| Pack | Level and advances | Result |
|---|---|---|
| Sol today | the real rungs only | everything "built" |
| A working Sol, near future | fusion torch, closed habitats: all "not yet built", nothing fictional | long spines, vast radiators, spin rings; bright in infrared and X-ray |
| Retro future | slow drives, sleeping crews ("not yet built") | big tanks, small quarters, crews ageing by ship time |
| Traveller | its own TL ladder: fusion plants that shed little heat, gravitic thrust, gravity plates, compensators, jump | compact closed hulls; five fictional chips, each naming its law |
| Bridge console | formula FTL, compensators, heat sinks | sleek; nearly dark unless the pack gives the drive a glow |
| Megastructures | one rising row of material strength, each structure from its paper | a ladder you can watch open: ring habitat, Mars beanstalk, Earth beanstalk, then the fictional ones |
| Pulp solar system | "atomic rockets" richer than any real fuel | honest fictional marks on cheerful ships |

## 12. Questions for Alex (all answered yes, 23 Nov 2026)

1. **No switches: an advance moves a figure, and the same block still runs** (zero waste heat gives zero radiator), so nothing is ever "skipped", only pushed, and "fictional" is read off whether a figure passed its cited ceiling. Recommended: yes. It is the processes-not-classes rule for machines.
2. **The ten starting limits in section 4**, as editable pack data, with weapons, computing, medicine and build cost left out for now. Recommended: yes.
3. **One ladder for real and imagined tech alike**, with real history on the lower rungs, so a steam-age nation can have a reactor at a hundred times the size and "Sol today" is just a level. Recommended: yes.
4. **Past the home level, parts shrink.** Recommended: yes, by a gentler editable factor, half the size per level beyond, stopping after two levels, so old tech gets handier without vanishing. (Revised 23 Nov 18:48, see section 5.2.)
5. **A third, softer honesty note, "not yet built"**, for the range between what flies today and what physics forbids (fusion torches, nanotube cable, sleeping crews), beside the existing "fictional". Recommended: yes. Without it a fusion torch and a warp drive would wear the same mark, which undersells the physics.
6. **Capability lives on a civilisation and inherits down the tree** (setting, species or nation, shipyard), dated on its timeline, with what a construct needs computed from its parts and never typed. Recommended: yes.
7. **Availability never says no.** It reads "can build", "N levels short, prototype at ten times the size per level", or "past anything known", each with its one-press ways forward. Recommended: yes. (Size rule revised 23 Nov 18:48, see section 5.2.)
8. **"Develop it" is a dated story event in the core**, set by the GM or proposed by Orac, with no research points; game packs and later the economy pack may drive the same event by program. Recommended: yes. (Narrowed 23 Nov 17:49: driving it by program is 4.1, see the ruling at the top.)
9. **"Found, not built"**: a construct may carry parts above its owner's level that work but cannot be copied or repaired. Recommended: yes; it is nearly free and it is half of science fiction.
10. **"Show it without"** on the card: redraw the ship with an advance removed. Recommended: yes, and capture it for the trailer.
11. **Every fictional advance must state the light it gives off, even if "none"**, so stealth is a visible choice of the setting and never an accident of solved heat. Recommended: yes.
12. **Crew weight becomes a sized part**: without gravity plates a long-voyage crewed ship gets a spin ring or a tumbling pair, sized from spin comfort. The generator does not draw this today. Recommended: yes; it is the most visible difference between low and high tech after radiators.
13. **Fictional advances get their own "setting fiction" rows in the fudge register**, written automatically and attributed to their pack. Recommended: yes.
14. **Styles**: the five in the mockup as the starter set in `base-ships`, written against the part vocabulary so they also dress stations and ground bases. Recommended: yes to Utilitarian, Sleek, Retro industrial and Military now; Organic is the hardest to do well and could follow. Tell me which you like and which to drop.

## 13. Packages, once accepted

| Package | What | Suggested owner |
|---|---|---|
| FT1 | `base-tech` pack: limits, figures with cited marks, the neutral ladder, real advances, each with a part family; the family maturity defaults of section 5.2 and the size multiplier for fictional devices; strength for weight on the master data sheet | matter data sheet thread, with citation checks |
| FT2 | Kernel: advances on a civilisation, set by hand or by a pack, inherited and dated, with no growth hooks (growth is 4.1); needs computed from parts; the size rule of section 5.2 (figure maturity by part family, floored at the best older advance held, with the size multiplier as the second path for fictional devices); the availability reading; fictional by ceiling | kernel thread |
| FT3 | Ships read figures from advances instead of per-fuel rules; the fictional flag on legs and passes that lean on a fictional advance | travel thread |
| FT4 | Generator: spin ring, prototype swelling, "show it without"; styles stage 5 against the part vocabulary | renderer thread |
| FT5 | Light by part and the band switch on the ship picture; stated light for fictional advances | light thread with the renderer |
| FT6 | "Leans on" on the card; the Build desk's availability reading and its three ways forward; "setting fiction" rows | chart room thread |
| FT7 | Megastructures on this footing: per-world beanstalk and ring sums, thrust blocks | Traveller and kernel threads (their design note) |
| FT8 | Ladders and advances in the example packs: Traveller's TL table first | Traveller thread |

## 14. Citation check (FT1, data sheet thread, 23 Nov)

Every source read where it could be (publisher record, the paper, or the author's report); "as summarised" means only a secondary page was read. Arithmetic re-run with the master data sheet's constants.

| Source | Reference | What it supports here | Status |
|---|---|---|---|
| O'Neill 1974 | O'Neill, G. K. 1974, Physics Today 27(9), 32 to 40. doi:10.1063/1.3128863 | steel cables "for conservatism", largest model R = 3,200 m, period 114 s; "no sharp upper limit", titanium allows larger | read (NSS reprint); DOI as cited elsewhere |
| McKendree 1996 | McKendree, T. L. 1996, Nanotechnology 7(3), 204 to 209. doi:10.1088/0957-4484/7/3/007 | nanotube habitat radius about 1,120 km (diamondoid 461 km) | as summarised (encyclopaedia); was "1995" and "hundreds of km", corrected |
| Globus and Hall 2017 | Globus, A. and Hall, T. 2017, NSS Space Settlement Journal, June 2017 | up to 4 rpm acceptable (56 m at 1 g); 6 rpm for residents with training; the older 1 to 2 rpm assumption | read |
| Pearson 1975 | Pearson, J. 1975, Acta Astronautica 2(9-10), 785 to 799. doi:10.1016/0094-5765(75)90021-1 | the tapered orbital tower | metadata read; content via Edwards 2000 |
| Edwards 2000 | Edwards, B. C. 2000, Acta Astronautica 47(10), 735 to 744. doi:10.1016/S0094-5765(00)00111-9; NIAC phase I report | carbon nanotube composite ribbon, 130 GPa assumed with a safety factor of 2; steel taper 1.7×10³³, Kevlar 2.6×10⁸ | read (NIAC report) |
| Pearson et al. 2005 | Pearson, J., Levin, E., Oldson, J. and Wykes, H. 2005, Lunar Space Elevators for Cislunar Space Development, NIAC phase I final report | aramid and M5 suffice; taper about 4, or none | as summarised; was "Pearson 2005" |
| Mars elevator | computed here | Zylon (3.8 MJ/kg) tapers about 12 times with no margin, 150 at a factor of 2; aramid 45 and 2,000 | computed; no paper read |
| Birch 1982 | Birch, P. 1982, Orbital ring systems and Jacob's ladders I, JBIS 35, 475 to 497 | ring in free fall, skyhooks held electromagnetically, active control and power; the ring bears little structural stress | read; "moving ring" reworded |
| Shkadov 1987 | Shkadov, L. M. 1987, Possibility of controlling solar system motion in the Galaxy, 38th IAF Congress, Brighton | the mirror thruster | abstract record read |
| Badescu and Cathcart 2006 | Badescu, V. and Cathcart, R. B. 2006, in Macro-Engineering, Springer, 251 to 279. doi:10.1007/1-4020-4604-9_12 | F = L(1 − cos θ)/2c (their eq. 34) | read |
| Sánchez and Scheeres 2014 | Sánchez, P. and Scheeres, D. J. 2014, Meteoritics and Planetary Science (accepted February 2014; volume and pages not read). doi:10.1111/maps.12293 | "small but non-zero cohesive strength"; about 25 Pa only in the unread part | read in part; do not ship a number from it until read |

| Figure | Check | Result |
|---|---|---|
| strength for weight: steel 0.25, aramid 2.5, nanotube 45 MJ/kg | 2 GPa / 7,850; 3.6 GPa / 1,440; 63 GPa / 1,340 | holds. Now on the master sheet as mixtures steel-wire, aramid, pbo-fibre, carbon-nanotube (tensile_strength and density). Aramid read from DuPont 2017 Table II-1: Kevlar 49 yarn 3.0 GPa (2.1 MJ/kg), resin strand 3.6 GPa (2.5); the sheet holds the yarn. PBO (Zylon HM) 5.8 GPa / 1,560 = 3.7 MJ/kg, read from the Fiber-Line datasheet. Nanotube 47 from Yu et al. 2000 (Science 287, 637, doi:10.1126/science.287.5453.637), citation confirmed, abstract not read. Steel still textbook from memory, marked so on the sheet |
| bond ceiling 50 to 100 MJ/kg | ideal nanotube about 100 GPa / 1,300 | about 77; holds |
| chemical 13 MJ/kg | hydrogen-oxygen, 120 MJ/kg of hydrogen at 1 in 9 by mass | 13.3; holds |
| fission 8×10¹³ J/kg | 200 MeV per 235 u | 8.2×10¹³; holds |
| D-He3 3.5×10¹⁴ J/kg | 18.35 MeV per 5.03 u | 3.52×10¹⁴; holds |
| annihilation 9×10¹⁶ J/kg | c² | 8.99×10¹⁶; holds |
| photon drive 3.3 N per GW | P/c | 3.34; holds |
| Wien peak at 1500 K | b / T | 1.93 µm; holds |
| radiator 3.9 km² for 2 TW at 1500 K | σT⁴ = 287 kW/m² | 7.0 km² one side at emissivity 1; 3.9 km² only two-sided at 0.9; assumption now stated |
| 224 m arm, 1 g at 2 rpm | g / ω² | 223.6 m; holds |
| ringworld 1.5×10¹² J/kg | g × 1 au | 1.47×10¹²; holds (about 33,000 times nanotube) |
| launch track 1,000 km at 3 g | v² / 2a, v = 7.8 km/s | 1,034 km; holds |
| fusion plume tens to hundreds of keV | burning D-T and D-He3 plasmas run at about 10 to 100 keV | holds; textbook |

### 14.1 Citation check of section 5.2 (data sheet thread, 23 Nov evening)

| Figure in 5.2 | Source read | Finding |
|---|---|---|
| V-2 thrust for weight about 27 | McCutcheon, K. D., U.S. manned rocket propulsion evolution part 2.2: V-2 propulsion, Aircraft Engine Historical Society (enginehistory.org, rev. 2024): 55,000 lb thrust, engine 2,484 lb, "22" | corrected to about 22. The 27 matches Astronautix's 931 kg engine mass, which cites nothing. Deutsches Museum gives 245 kN and 400 kg, probably without the turbopump |
| Merlin 1D about 180 | SpaceX press release (2013), quoted by Space.com and Spaceflight Now: vacuum thrust for weight "exceeding 150"; 180 from Musk's SES-8 teleconference, 24 Nov 2013 (as summarised) | "over 150, about 180 claimed" |
| Jumo 004 about 1.2; fighters 7 to 9 | Smithsonian NASM A19670124000: 8,807 N, 739.4 kg (1.21); MTU EJ200 brochure: 20,000 lbf with reheat, 13,500 dry, about 2,204 lb (9.1 and 6.1) | holds; EJ200 named. F119 and F135 weights not found |
| SNAP-10A about 1 W/kg; later 3 to 15 | US DOE ETEC SNAP overview: at least 500 W (over 600 at first), under 950 lb with instruments and shielding (1.2 to 1.4 W/kg); Mason, L. S., NASA (NTRS 20180005431): Kilopower-class 1 to 7 W/kg; Manvi and Fujita 1986, SP-100 users handbook (JPL D-2185): 100 kWe for about 3,000 kg (about 33 W/kg) | corrected: later designs 1 to 7 W/kg small, about 33 large, none flown. The engine family's "a seventh" rests on the engines, not the reactors |
| V-2 exhaust about 2 km/s against 4.5 | AEHS USSRC V-2 page: 203 s at sea level, 239 s in vacuum; National Museum of the USAF fact sheet (thrust and flow give about 195 s); RL10B-2 465.5 s (L3Harris, read for FT1) | holds; vacuum against vacuum is 1.95 times, so "about half" |
| carbon fibre about 1.5 GPa in the 1960s, 7 now | Shindo, A. 2000, J. Ceram. Soc. Japan 108, S35 (reprint of his early-1960s work, abstract read): 5,000 to 10,000 kgf/cm² at best, 0.5 to 1 GPa; Toray T1100G data sheet (2021): 7,000 MPa; Toray carbon fibre card: T300 3,530 MPa | corrected. First fibre to best now is 7 to 14 times; first commercial grade to best now is 2 times. The materials default of a fifth sits between |
| nanotube single tubes against yarn | Peng, B. et al. 2008, Nature Nanotechnology 3, 626 (abstract): single tubes above 100 GPa; Bai, Y. et al. 2018, Nature Nanotechnology 13, 589 (abstract): centimetre bundles over 80 GPa; Taylor, L. W. et al. 2020, Carbon (Rice University news, as summarised): fibre 4.2 GPa | figures added; tube to fibre about 24 times |
| lithium-ion about 80 Wh/kg first, about 270 now | Crabtree, G., Kócs, E. and Trahey, L. 2015, MRS Bulletin 40, 1067: Sony's first cell 80 Wh/kg, energy density up "by a factor of three" since; Amprius 2022 press release: 450 Wh/kg silicon-anode cells | holds as "about three times"; the 270 had no source read and is replaced by Crabtree's factor |

Still from memory: the hydrogen-oxygen exhaust ceiling "near 5 km/s".

## 15. Keeping the books: seven improvements (accepted 23 Nov 20:14)

Alex asked for an honest critical look at the accepted design. The core stands. Its weak point was that "fictional" was read off one figure passing its own ceiling, and a setting can stay under every ceiling and still break physics, because the laws are relations between figures. These seven changes close that, make the numbers easier to set, and make "scientific" a tested claim. Figures marked *computed* are this note's arithmetic and pass the citation check like the rest.

### 15.1 Balance checks: check the books, not only each ceiling (question 19)

Every construct design runs a short list of **balance checks**, each a small cited block that relates figures the design already has. A check that fails is named on the card, the same way a figure past its ceiling is.

| Check | Relation | What it catches |
|---|---|---|
| **Energy pays for the exhaust** | exhaust speed is at most √(2 × energy per kg of what is thrown and burned); the relativistic form near c | a "chemical" fuel with a torch's exhaust speed. Hydrogen and oxygen at 13.4 MJ/kg (the sheet's value for water vapour, section 14.2) give at most 5.2 km/s (*computed*); the best flown engine reaches 88% of it, which replaces the cited "near 5 km/s" ceiling with a computed one |
| **Jet power** | jet power = ½ × thrust × exhaust speed, and it is at most what the plant delivers | thrust and exhaust speed typed independently that need more power than the ship makes |
| **Energy in, energy out** | plant output = jet power + waste heat + stored + other loads | waste heat set to nothing while the plant's efficiency says otherwise |
| **Momentum** | thrust = mass thrown per second × exhaust speed, or P/c for light alone | thrust with nothing thrown and no stated partner (section 15.3) |
| **Heat flows downhill** | section 15.2 | a hot radiator cooling a cool room for free |

Consequences:

- **A ceiling may be computed.** Where a relation gives the ceiling from other figures, the mark says "computed from" and names them, and it moves when they move. A cited ceiling stays where no relation gives one (a melting point, a bond energy).
- **"Fictional by arithmetic" now means: a figure past its ceiling, or a balance that does not close.** The chip names the check and by how much: "exhaust carries 40 times the energy its fuel holds".
- **Still never forbidden.** A failed check is an honesty mark and a "setting fiction" row, not a refusal.
- The list is pack data (`base-tech`), so a physics pack can add a check with a new law.

### 15.2 Heat comes in grades (question 20)

The radiator table in section 5.1 let panel temperature rise freely. It cannot: a radiator runs no hotter than the thing it cools, unless a heat pump is paid for, and the pump's work becomes more heat.

- **Waste heat carries its temperature.** Each source of waste heat states how hot it is: a drive or reactor at its working temperature, electronics near 350 K, a crew's quarters near 300 K.
- **A radiator's temperature is at most its source's.** The radiator figure an advance gives (how hot a panel can run) is a cap, not the answer. Heat is shed at the lower of the two.
- **A heat pump is a part.** Lifting heat Q from T-cold to T-hot costs work of at least Q × (T-hot − T-cold) / T-cold (Carnot), and that work is shed too. The block sizes the pump and adds its power to the plant's loads.
- **Visible result:** a high-tech ship with tiny hot radiators for its drive still carries a modest cool panel for its quarters and electronics. 100 kW of low-grade heat at 300 K needs about 120 m² of two-sided panel (*computed*, emissivity 0.9), whatever the panel material can stand. This is true of real designs and is a second thing the eye can read.
- The 5.1 table stays right for drive and reactor heat, which is nearly all of a torch ship's 43 GW.

### 15.3 A fictional advance says where everything goes (question 21)

Question 11 made every fictional advance state its light. The same now holds for the three books it may break. Each fictional advance fills four plain fields, and "nowhere" or "nothing" is allowed but shows on the card:

| Field | Question it answers | Example |
|---|---|---|
| Light | what it gives off, by band | "a blue flash on entry" |
| Heat | where the heat ends up | a heat sink: "stored in the sink, dumped in port; holds 40 hours" or "nowhere" |
| Momentum | what it pushes on | a gravity drive: "the nearest large mass" or "nothing" |
| Energy | where the energy comes from, or goes | a reactionless drive: "the ship's plant, with the surplus from nowhere" |

A stated partner has consequences the blocks can use (a sink that fills makes heat a clock on a fight or a pass; a drive that pushes on a planet needs one nearby). An honest "nothing" stays a plain fictional mark. Either is a choice a setting makes on purpose.

### 15.4 One advance per figure per part, stored as the real quantity (question 22)

- **Stored value.** A pack stores a figure as the physical quantity with its unit (1,700 K, 45 MJ/kg, 0.03 c). "Sheds heat 1,000 times better than today's panels" is a reading the card computes against the built mark, never the stored value. This removes the doubt over what a factor multiplies (temperature, or heat per m², which goes as its fourth power).
- **No stacking.** Two advances that set the same figure are alternatives. A part uses one, chosen by the builder or by the generator as the better for the job, and never the product of both. Advances on different figures combine only through the sizing blocks and the balance checks.
- **Several limits, one figure.** A figure held back by more than one limit (an engine's power for its mass answers to heat and to material strength) lists each on its Science line.

### 15.5 Set a number by what it buys (question 23)

A GM does not know what 1,700 K means. On the Technology page (section 7.1), each figure of an advance shows:

- **A scale with its marks on it**: built, allowed and ceiling as ticks, the setting's value as the handle, the ink changing as the handle crosses a mark. Dragging past the ceiling is allowed and turns the chip fictional.
- **A reference construct redrawn live beside it**: the setting's own reference ship (or a station or cable for a material), from the same generator as "show it without", with the one-line reading "radiators 52 km² to 52,000 m²".
- Later, not in the first build: asking Orac "what would it take to lose the radiators?", which searches the same scale.

This follows the interface style: the common act (nudge a figure, see the ship change) is one drag away, and the kelvin stay on the Science line.

### 15.6 Prove it on real hardware (question 24)

The claim "the same blocks that draw today's ship draw tomorrow's" gets a standing test. At the "Sol today" level the blocks should reproduce about ten real craft within a stated tolerance, and one level short should give something like the first generation.

| What is reproduced | From | Proves |
|---|---|---|
| ISS radiator area from its heat load | limit 1, low-grade heat (15.2) | the radiator block and heat grades |
| Saturn V and a modern reusable launcher: mass ratio and stage masses from exhaust speed and payload | limit 2 | the rocket equation and tank sizing |
| A deep-space probe's power source mass (an RTG) and a solar craft's panel area at its distance | limit 3 | power for mass |
| Shuttle and capsule entry heating | limit 1, heat shields | the aerobraking sums |
| A station's shielding and supplies per person a day | limits 8 and 9 | habitat sizing |
| The V-2 as "one level short" of a modern engine | section 5.2 | the family defaults |

- **Tolerance is stated per row and logged.** A row that misses is a finding, either a block to fix or a fudge to register, never a number to tune quietly.
- **Data comes from the real spaceflight pack** (its own thread, started 23 Nov on Alex's ask to gather every craft and station that has flown). This test draws its craft from that pack once it exists; until then FT1 holds the ten by hand, cited.
- It is also teaching content: a page that shows the engine getting real machines right.

### 15.7 The family defaults are rules of thumb, and logged as such (question 25)

The defaults of section 5.2 (engines a seventh, materials a fifth, stores a third, two-thirds of the way for near-ceiling figures, ten times for the unknown) rest on one or two examples each. They are editable guesses with history behind them, not measurements. So they live in the **fudge register** as rules of thumb, each with its examples and the note "small sample", and not on the cited data sheet beside constants. The examples themselves (V-2, Merlin, T300) stay cited.

### Questions (all accepted 23 Nov 20:14)

19. **Balance checks run on every design**, and "fictional" also means a balance that does not close. Recommended: yes.
20. **Waste heat carries its temperature**; a radiator runs no hotter than its source unless a heat pump is paid for. Recommended: yes.
21. **Every fictional advance states its light, heat, momentum and energy**, with "nowhere" allowed and shown. Recommended: yes.
22. **Figures stored as real quantities; advances on one figure are alternatives and never multiply.** Recommended: yes.
23. **A scale with marks and a live reference construct for each figure** on the Technology page; the Orac search later. Recommended: yes.
24. **A real-hardware test of about ten craft** at "Sol today", with stated tolerances, drawing on the real spaceflight pack. Recommended: yes.
25. **Family defaults logged in the fudge register as rules of thumb.** Recommended: yes.

### What changes in the packages

| Package | Added by section 15 |
|---|---|
| FT1 (data sheet thread) | balance checks as pack rows with their relations; computed ceilings marked "computed from"; a temperature on every waste heat source; the four "where it goes" fields on fictional advances; figures stored as quantities with units; about ten real craft, cited, for the test until the spaceflight pack supplies them; family defaults moved to the fudge register |
| FT2 (kernel thread) | the balance check block and the widened "fictional" reading; heat grades and the heat pump in the radiator block; one advance per figure per part, no stacking; the real-hardware test in the suite |
| FT3 (travel thread) | ships state waste heat by grade; legs and passes carry the fictional flag for a failed balance as for a passed ceiling |
| FT4 (renderer thread) | the cool habitat radiator as its own drawn part; the reference construct for the Technology page |
| FT6 (chart room thread) | the figure scale with marks and the live reference construct; the failed check named on "Leans on"; the four fields shown on a fictional chip |
| FT8 (Traveller thread) | the four fields filled for each fictional advance in the example packs |

### 14.2 Citation check of section 15 (data sheet thread, 23 Nov night)

| Figure in 15 | Worked from | Finding |
|---|---|---|
| hydrogen-oxygen at 13.3 MJ/kg gives at most 5.2 km/s | the master sheet: water's heat of formation (liquid, −285.83 kJ/mol, Thauer 1977) less its latent heat of vaporisation (2.468 MJ/kg, NIST) over its molar mass from the atomic weights: 13.4 MJ/kg of water vapour; √(2 × 13.4 MJ/kg) = 5.18 km/s | holds (5.2). The base-tech pack now carries it as a computed ceiling on exhaust speed for hydrogen-oxygen, built from the sheet, and its hydrogen-oxygen energy mark is the vapour (lower heating) value. The best flown engine (RL10B-2, 4.57 km/s) reaches 88% of it |
| 100 kW at 300 K needs about 120 m² of two-sided panel at emissivity 0.9 | Stefan-Boltzmann with the sheet's σ: 1e5 / (2 × 0.9 × 5.670e-8 × 300⁴) = 121 m² | holds, for a panel seeing cold sky only; sunlight or a warm planet below adds load |
