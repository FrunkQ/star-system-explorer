# SSE4 future technology: defined by the problems it solves

Status: **accepted 23 Nov 2026**. Alex answered "yes to all" at 16:15 to the fourteen questions in section 12 (read by the coordinator as covering this list among three open at once; he can correct that). Written by the design thread. Track letters **FT**. Design only; the packages are in section 13. Every figure still passes the citation check before it ships. A mockup goes with it: "SSE4 ship styles and tech levels" (one generated ship in five styles and at three tech levels), source in `/mnt/project-files/sse4/ships/styles-and-tech-mockup.html`.

Read with: the voyages design (drive facts, FTL as a pack rule), the nebulae and light design (every object publishes its light by band, section 5A sensing), the matter pack design (the master data sheet and its one "Look up" door), the programs design (track J), the naming design (settings inherit down the tree), and the ship generator in `packages/renderer/src/ship-design.ts` (parts sized by physics, a style only arranges and dresses them).

Sources below are **[from memory, verify]** unless a build thread has already cited them in a pack. The citation check applies before any figure ships.

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
| 1 | **Heat has to leave** | Second law; radiated power goes as area times temperature to the fourth | waste heat per watt of jet or of power; radiator temperature; a heat shield's peak heating and total load | radiators (2 TW of waste at 1500 K needs about 3.9 km² of panel); heat shields and aerobraking; how bright it is in infrared | heat sinks that send it "nowhere" |
| 2 | **Thrust needs something to throw** | Conservation of momentum; the rocket equation | exhaust speed (ceiling c); thrust per watt with nothing thrown (ceiling 1/c, a photon drive, about 3.3 N per GW) | tanks, mass ratio, staging; lifting off a world at all | reactionless and gravity drives; contragravity |
| 3 | **Energy has weight** | Mass-energy; energy per kilogram of fuel, power per kilogram of plant | energy per kg (chemical 13 MJ, fission about 8×10¹³ J, deuterium and helium-3 about 3.5×10¹⁴ J, ceiling c² = 9×10¹⁶ J for matter with antimatter); power per kg of power plant | fuel tanks, reactor size, range | anything richer than annihilation |
| 4 | **Crews break under hard acceleration** | Physiology of the crew's species; inertia cannot be screened (equivalence principle) | sustained and peak g for the crew, from the species, raised by couches and liquid immersion | trip times; engine size; the length of a launch track (3 g to orbital speed at Earth is about 1,000 km of track) | inertial compensators |
| 5 | **Crews need weight** | Spin gravity: radius is g over spin rate squared; comfort caps the spin rate | highest comfortable spin (about 2 to 4 turns a minute); lowest healthy gravity | spin rings and tumbling pairs (1 g at 2 turns a minute needs a 224 m arm) | gravity plates |
| 6 | **Nothing outruns light** | Special relativity | none: this one has no figure to push, only a pack rule | travel between stars; message delay; a crew's age against home | every FTL drive and FTL radio; already a per-setting rule in the voyages design |
| 7 | **Materials only bear so much** | Strength for weight (pascals over density); bond energy sets a ceiling near 50 to 100 MJ/kg | a substance's strength for its weight, from the master data sheet (steel about 0.25 MJ/kg, aramid about 2.5, carbon nanotube about 45, allowed and not built at length) | tank and structure mass; how hard a hull can be pushed; pressure hulls and domes; tower height; spin habitat radius; tethers; every megastructure (section 6) | ringworld floor, rigid shells round a star |
| 8 | **Radiation gets through** | Stopping of charged particles and gammas by mass in the way | shielding in g/cm² for a given dose; a magnetic screen's bending strength (allowed) | mass round the quarters; storm shelters; how close a ship may pass a hot star or a giant's belts | force screens |
| 9 | **Crews need air, food and time** | Mass balance of a closed habitat; ageing | supplies per person a day; how much is recycled (ceiling all of it); room per person; whether crews can sleep the trip (allowed, not built) | habitat and stores size; greenhouse area; who is awake | stasis with no upkeep |
| 10 | **Anything warm can be seen** | Every body radiates at its temperature; the light design's sensing arithmetic | none of its own: it reads the heat and drive figures | detection range by band; hiding | cloaks, which must say where the light goes |

Notes on the list:

- **The crew limits belong to the cargo, not the ship.** A drone has no limit 4, 5 or 9, so it comes out small and fast with no fiction at all. An alien species with a different tolerance changes ships with no tech entry. This is why limit 4 reads the species.
- **Limit 6 is the only true on-off.** That is honest: there is no partial FTL. It stays the voyages design's abstract rule per setting.
- **Left out on purpose:** build time and cost (the economy pack, 4.1), minds and computing, medicine, weapons and armour (no dogfighting). A pack may add a limit for any of them later through the same shape.

Most of these figures already exist. `base-ships` `drive_heat` carries waste per jet and radiator temperature per fuel; the ship record carries exhaust speed, acceleration and shielding; `NavShip` carries shield heating limits and `structure_g`; strength for weight is a substance property that belongs on the master data sheet and is read through "Look up". FT1 mostly names what is there and gives each figure its marks.

## 5. The tech ladder

1. **A setting defines its own ladder**: how many levels and what they are called. Traveller has its TL numbers; "A working Sol" might have three ("orbital", "torch", "deep"). The base pack ships a neutral ladder with the real past on its lower rungs, so "Sol today" is simply a level.
2. **Every advance sits at a home level**, real ones included. A chemical rocket, a fission reactor and a fusion torch are advances like any other; only their marks differ.
3. **Every advance is carried by a part with a size**: a drive, a reactor, a cable, a plate under the deck, a heat sink. The pack gives the part's size at the home level, usually per watt, per newton or per tonne of ship.
4. **Short of the home level, the part is ten times the mass and volume for each level short.** The ten is one editable number per setting (and may be overridden per advance). One level short is a prototype. Three levels short is a thousand times: a jump drive the size of a small moon. It is never forbidden; the app shows the size and lets the GM build it. This is the ruling "danger is drama, never forbidden" applied to ambition.
5. **For an advance with no separate device** (a better material), the part is the structure itself: ten times the mass for the same load, which is the same as a tenth of the strength for weight. One rule still covers it.
6. **Past the home level**, parts may shrink by a second editable factor (question 4).

Because the generator already draws every part from its size, a prototype looks like one: the same ship with one grotesquely swollen system.

## 6. Every construct, one rule set

"A construct is just a construct." The limits do not know what they are sizing.

| Construct | Limits that bite hardest | What the same blocks give |
|---|---|---|
| **Ship** | 1, 2, 3, 4, 5 | tanks, radiators, spin ring, shield, as built today in the ship generator |
| **Station or habitat** | 5, 7, 8, 9 | largest spin radius is strength for weight over g: a few kilometres in steel with a working load (O'Neill 1974), hundreds of kilometres in nanotube (McKendree 1995) |
| **Ground facility** | 1, 3, 7, 9 | heat goes into air or ground where there is any, so radiators shrink on a world with air; dome span from skin tension; tower height from strength for weight over local g; crush depth in an ocean or a giant's air; a launch track's length from limit 4 |
| **Asteroid with engines** | 2, 3, 7 | the rock is the reaction mass or carries it; the push it can take is its own strength, and a rubble pile's is tiny (Sánchez and Scheeres 2014), so the answer is "years of gentle thrust", from physics |
| **Beanstalk (space elevator)** | 7 | taper and cable mass computed per world from its gravity and spin, the way aerobraking is computed per ship per world. Mars and the Moon open with fibres that exist (Pearson 1975, 2005); Earth needs nanotube-class cable (Edwards 2000). No can-build flag: the app says "the cable would outweigh the planet" where it would |
| **Orbital ring** | 3, 7 | held up by a moving ring, so it needs power and control, not a miracle material (Birch 1982) |
| **Dyson swarm** | 3, and mass | no fiction at all; it needs a planet's worth of matter, which is a real amount that can be mined out (the resources ruling) |
| **Stellar engine** | 2 | a mirror gives thrust of the star's light over 2c (Shkadov 1987): real, and almost nothing. Attached as a thrust block, per the megastructure ruling |
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
| **One level short: Woven carbon cable** (or two, three) | an advance not held yet | **Build the prototype**, at ten times the size per level short, shown before committing; or **Develop it** |
| **Past anything known: needs 30,000 times their strongest material** | no advance on this setting's ladder reaches the figure | **Add an advance to the setting** (marked fictional if past the ceiling); or **Place it as found** |

**Develop it.** Recommended: in the core app this is a story decision with one honest lever. "Develop it" adds a dated event, "the Vesh gain woven carbon cable in 2310", at the campaign's now or a chosen date. From that date the reading changes, existing prototypes stay as built, and new builds come out at the proper size. No research points in the core. A game pack may drive the same event from a program (track J) or, later, from the economy pack, so a 4X-style pack can have research as a mechanic without the core growing one. Orac can propose the event and its consequences ("this opens a beanstalk on four of their worlds").

**Found, not built.** Any construct may carry parts above its owner's level. They work, they cannot be copied or repaired, and the card says "found". This gives the ancient artefact, the captured drive and the gift from elder aliens with no new machinery, and it is authored ink like any GM placement.

**The teaching use.** Because the lower rungs are real history and the marks are cited, the same screen answers "why have we not built a space elevator?" with a number. That is the education half of the project, free.

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
4. **Past the home level, parts shrink.** Recommended: yes, by a gentler editable factor, half the size per level beyond, stopping after two levels, so old tech gets handier without vanishing.
5. **A third, softer honesty note, "not yet built"**, for the range between what flies today and what physics forbids (fusion torches, nanotube cable, sleeping crews), beside the existing "fictional". Recommended: yes. Without it a fusion torch and a warp drive would wear the same mark, which undersells the physics.
6. **Capability lives on a civilisation and inherits down the tree** (setting, species or nation, shipyard), dated on its timeline, with what a construct needs computed from its parts and never typed. Recommended: yes.
7. **Availability never says no.** It reads "can build", "N levels short, prototype at ten times the size per level", or "past anything known", each with its one-press ways forward. Recommended: yes.
8. **"Develop it" is a dated story event in the core**, set by the GM or proposed by Orac, with no research points; game packs and later the economy pack may drive the same event by program. Recommended: yes.
9. **"Found, not built"**: a construct may carry parts above its owner's level that work but cannot be copied or repaired. Recommended: yes; it is nearly free and it is half of science fiction.
10. **"Show it without"** on the card: redraw the ship with an advance removed. Recommended: yes, and capture it for the trailer.
11. **Every fictional advance must state the light it gives off, even if "none"**, so stealth is a visible choice of the setting and never an accident of solved heat. Recommended: yes.
12. **Crew weight becomes a sized part**: without gravity plates a long-voyage crewed ship gets a spin ring or a tumbling pair, sized from spin comfort. The generator does not draw this today. Recommended: yes; it is the most visible difference between low and high tech after radiators.
13. **Fictional advances get their own "setting fiction" rows in the fudge register**, written automatically and attributed to their pack. Recommended: yes.
14. **Styles**: the five in the mockup as the starter set in `base-ships`, written against the part vocabulary so they also dress stations and ground bases. Recommended: yes to Utilitarian, Sleek, Retro industrial and Military now; Organic is the hardest to do well and could follow. Tell me which you like and which to drop.

## 13. Packages, once accepted

| Package | What | Suggested owner |
|---|---|---|
| FT1 | `base-tech` pack: limits, figures with cited marks, the neutral ladder, real advances; strength for weight on the master data sheet | matter data sheet thread, with citation checks |
| FT2 | Kernel: advances on a civilisation, inherited and dated; needs computed from parts; the size rule; the availability reading; fictional by ceiling | kernel thread |
| FT3 | Ships read figures from advances instead of per-fuel rules; the fictional flag on legs and passes that lean on a fictional advance | travel thread |
| FT4 | Generator: spin ring, prototype swelling, "show it without"; styles stage 5 against the part vocabulary | renderer thread |
| FT5 | Light by part and the band switch on the ship picture; stated light for fictional advances | light thread with the renderer |
| FT6 | "Leans on" on the card; the Build desk's availability reading and its three ways forward; "setting fiction" rows | chart room thread |
| FT7 | Megastructures on this footing: per-world beanstalk and ring sums, thrust blocks | Traveller and kernel threads (their design note) |
| FT8 | Ladders and advances in the example packs: Traveller's TL table first | Traveller thread |
