# SSE4 and Atomic Rockets: what is on the site, where it lands in SSE4, the rights, and the approach

Status: **accepted 23 Nov 2026, 20:25** (Alex: "Agree to all questions", with question 11 changed: weapons come in, as data and "in range" sums only, "we dont fire anything"). Sections 11 to 14 were added the same evening on his word (power as a general input, sensors, weapons, and what SSE3 had); their six questions in section 15 were **answered yes at 21:30** ("yes to all"). Sections 16 and 17 (the power flow diagram, and fog of war from sensors) answer his two follow-ups and carry the questions in section 18, **all answered yes at 22:08** ("Yes to your questions"). Mockup: `/mnt/project-files/sse4/RK-screens/power-flow-mockup.html` (artifact "SSE4 power flow", https://claude.ai/artifact/2k5uSbdv6yKuFNgRedGTDs), screenshots beside it. Research and design only, no code. Track letters **RK** (a neutral working name; nothing here is named after the site until its author agrees).

Alex, 23 Nov 16:25 to 16:28: "this is VERY interesting", "EVERYTHING on that site needs to be in SSE - as a pack? I approach them and ask if they wanna get involved - as they can now fly their rockets properly simulated", "They have rocket equations - we have equation blocks!"

Read with: `SSE4-future-tech-design.md` (track FT: limits, figures, marks, advances, the ladder), `SSE4-voyages-design.md`, `SSE4-equation-block-design.md`, `SSE4-example-packs.md`, `SSE4-matter-pack-design.md`.

**What was actually read.** The site is about 170 pages, many of them over 100,000 characters. Read today, each in part (roughly the first third word for word, the rest as a summary): the home page and full navigation, Preliminary Notes, Footnotes, the author's own page, Rocket Theory, Engine Intro, Engine List 1, Basic Design, Heat Radiators, Mission, the Mission Table appendix, FTL Starships 1, Detection, Artificial Gravity, Realistic Designs A to G, Power Plant, Life Support, Surface to Orbit, Tech Level, Torchships and Useful Tables (21 pages). The Equations appendix would not load (it answered with a redirect notice). Every other page is placed below by its title in the navigation and by general knowledge of the site, and is marked *by title*. Nobody should build from this note's description of a page without opening the page.

## 1. The short answer

1. **About two thirds of the site is physics SSE4 already has a home for**, and the future tech design accepted today is very nearly a machine for running it: the site is a catalogue of engines, power plants, radiators, habitats and launch systems, each with figures, and FT says "an advance is a set of figures, and the same sizing blocks draw the ship". Most of the work is data and citations, not new machinery.
2. **"Everything" splits four ways.** Physics and engineering figures go into the base packs, cited to the primary papers. Design studies (NERVA, Orion, Daedalus, Discovery II and the rest) become flyable ships in one new pack. The worldbuilding essays (history, empires, trade, law, culture, aliens) are reading, not data: SSE4 answers them with features it already has and links out. Space combat, sidearms and deck plans stay out under standing rulings, except detection, which is limit 10 and comes in.
3. **The site shows three things the ten limits miss** and four drive families the stock ships do not cover (section 4). The biggest is that SSE4 has no drive whose thrust is held back by its power plant, which is every electric engine ever flown.
4. **The equations are the best fit of all.** Nearly every closed formula on the site can be a named Equation block. Several already exist in SSE4, but as Rust or TypeScript, not as blocks (section 5). Moving ship sizing into Equations is the piece that makes "their equations fly ships" literally true.
5. **Rights: the site carries a plain copyright line and no licence.** Physics and figures from primary sources are free to use. His text, his tables as he compiled them, and the artwork (most of which is other people's) are not, and a good share of his engine table comes from a commercial board game. Section 7.
6. **There is no email address on the site.** The routes it does give are Mastodon and Patreon. A draft message for Alex is in section 8. Nothing has been sent.

## 2. The site, section by section

"FT" means the future tech design. "Sheet" is the master data sheet (`base-reference`). Verdicts: **base** (goes into SSE4's own packs, cited to primary sources), **pack** (the new pack in section 6), **feature** (SSE4 answers it with something it has or has planned), **link** (reading to point at, with permission), **out** (standing ruling).

### 2.1 Engines and fuel

| Site pages | What they hold | SSE4 today | FT carries | Missing | Verdict |
|---|---|---|---|---|---|
| Rocket Theory, Multi-stage | Momentum, mass ratio, the rocket equation, exhaust speed from chamber temperature, fusion exhaust from energy released, thrust power, best exhaust speed for a trip | Rocket equation (also relativistic), mass flow and jet power, in `sse-nav` and the ship generator | Limits 2 and 3 | Staging; exhaust speed worked from temperature and gas weight instead of typed; best exhaust speed | **base**, as Equations (section 5) |
| Engine Intro, Engine List 1 to 3 | Eleven families: beamed power, chemical, plasma (electromagnetic), ion (electrostatic), electrothermal, sails, antimatter, fusion, nuclear thermal (solid, liquid, gas core), pulse (Orion and kin), and "other". Each engine has a row: exhaust speed, thrust, thrust power, mass flow, engine mass, thrust to weight, efficiency, fuel, reaction mass, specific power | Four drives by fuel: hydrogen-oxygen, hydrogen nuclear thermal, deuterium and helium-3 torch, antimatter. Six stock ships | Each engine is an **advance** with marks (built, allowed, ceiling) and a home level. This is exactly FT's shape | Seven families have no drive at all; see section 4 | **base** for the families and anything flown; **pack** for the paper engines |
| Atomic, Fusion, Antimatter Fuel (*by title*) | Fuel cycles, energy per kilogram, how fuel is made and stored | Energy per kg for chemical, fission, D-He3, annihilation (FT section 4, checked) | Limit 3 | Other fusion reactions (D-T, D-D, proton-boron), neutron share of each (which drives shielding and waste heat), storage mass for antimatter and cold hydrogen | **base**: reactions and propellants on the sheet |
| Torchships | A torchship defined by a figure (jet power of at least about 1 MW for each kilogram of dry ship, after Rick Robinson), why that is hard, and why the waste heat is the catch | Fusion torch ship; brachistochrone courses | Limits 1, 2, 3 together; "show it without" makes the point visually | Nothing structural. "Jet power per kg of ship" is worth showing on the ship card, since it is the one number that says torch or not | **feature**; a good tour |

### 2.2 Ship design

| Site pages | What they hold | SSE4 today | FT carries | Missing | Verdict |
|---|---|---|---|---|---|
| Basic Design, Advanced Design, Sizing, Worked Example | Mass budgets, "every gram counts", propulsion bus plus payload, worked designs | The generator: parts sized by physics, recipe saved | The whole design | Structure mass as a share worked from loads; a worked example to check against | **base**; the worked example is a test case |
| Power Plant | Fuel cells, solar thermal, solar panels, radioisotope generators, fission and fusion reactors, each as kilograms per kilowatt (solar panels from 100 down to about 1.4, radioisotope about 200), sunlight by planet, a flown reactor's mass budget | Power is implied by the drive, not a part | "Power per kg of plant" is a limit 3 figure | **A power plant as its own sized part**, with its own waste heat and conversion efficiency | **base**: needed for electric drives |
| Heat Radiators | Stefan-Boltzmann sizing, view factor for several panels, best radiator temperature, heat pumps, about seventeen radiator types with mass per kilowatt, open-cycle cooling, glow | Radiator area from waste heat at a temperature per fuel | Limit 1 | Radiator **mass** per square metre by type; panels shading each other; a second, cooler loop for the crew; dumping heat into propellant | **base** |
| Atomic Radiation (*by title*) | Dose, shadow shields, shield materials | Shield mass from g/cm², dose rate near belts, in `sse-nav` | Limit 8 | Shadow shield geometry (shield the crew's cone, keep the reactor far away), which shapes a ship more than anything bar radiators | **base** |
| Life Support (read); CELSS, Habitat Module, Human Factors (*by title*) | Kilograms and litres per person a day (food about 2.3 kg, oxygen about 0.84 kg), carbon dioxide limits, recycling plant masses, crew health on long trips, from NASA handbooks | Crew count and quarters on the ship | Limit 9 | The per-person figures and the recycling share as cited rows | **base** |
| Artificial Gravity | Spin radius against turns a minute, Coriolis, comfort limits, precession, spin balance | Not drawn yet | Limit 5; FT question 12 (spin ring as a part), same sources (Globus and Hall; Hill and Schnitzer) | Precession and balance are handling details; leave | **base**, already in FT4 |
| Spaceship Decks (nine pages) | Deck plans, control rooms, sickbay | Plan views with call-outs | Nothing | Nothing | **out**: no deck plans (ruling, 23 Nov) |

### 2.3 Operations and missions

| Site pages | What they hold | SSE4 today | FT carries | Missing | Verdict |
|---|---|---|---|---|---|
| Mission, Orbits | Orbital speed, escape, gravity and drag loss, Hohmann transfers, vis-viva, windows, phase angle | All of it, computed, in `sse-nav`: Hohmann, Lambert, windows, torch courses, escape and capture | Nothing needed | **Slow spirals** for low-thrust ships (weeks of thrust, not a burn); gravity and drag loss on ascent as a shown figure | **feature**; one new leg-maker |
| Mission Table appendix, Useful Tables | Delta-v between worlds, round trips by impulse and by constant thrust, Earth and Moon space | SSE4 computes these for any system, not just Sol | Nothing needed | Nothing | **feature**, and the published tables are **test fixtures**: SSE4's Sol should reproduce them |
| Blast-off, Landing, Embarking (*by title*) | Ascent, descent, boarding | Entry, aerobraking, landing and take-off per ship per world | Limits 1, 2, 4 | Ascent profile through air | **feature** |
| Surface to Orbit | As read: launch sites and the boost from a world's spin, reusable boosters, single stage to orbit. Launch tracks, loops, skyhooks and elevators are further down the page or on Infrastructure (*by title*) | Nothing yet | Limits 4 and 7; FT section 6 already works the elevator, orbital ring and launch track | The spin boost by latitude of a launch site (cheap: SSE4 knows every world's spin); rotating tethers and launch loops as constructs | **base** figures; **pack** for the named concepts |
| Online Calcs, TransHab Calc (*by title*) | Calculators | SSE4 is the calculator | | | **feature** |

### 2.4 Ship types, real designs, places to build

| Site pages | What they hold | SSE4 today | FT carries | Missing | Verdict |
|---|---|---|---|---|---|
| Realistic Designs (five pages), Landers, Space Tugs, Terra Return | Hundreds of design studies with mass budgets: nuclear thermal Mars ships, EMPIRE, fission fragment, Discovery II and so on | Six stock ships; "Sol today" pack planned (real spacecraft) | A design is a recipe; its needs are computed from its parts | The designs themselves as recipes | **pack**: the heart of it |
| General Ship Types (*by title*) | Roles and classes of ship | Roles as programs (voyages 5.1) | | | **feature** |
| Starships, Sublight, Nearlight | Slow boats, generation ships, sleeper ships, relativistic flight, ramscoops | Relativistic legs with ship time and home time | Limits 6 and 9 | Ramscoop and laser sail as drives (section 4) | **base** and **pack** |
| FTL Starships 1 and 2 | FTL as a black box, and why FTL breaks cause and effect | FTL as a per-setting rule, seven families (example packs 4A) | Limit 6 | A line on the FTL chip's "Why?" about causality; nothing to build | **feature**; nothing new |
| Space Stations, Spaceports, Planetary Base, Infrastructure, Space Colony | Habitats, depots, ports | Constructs; territory holdings | FT section 6: one rule set for every construct | Depots as refuel places exist; the habitat designs as recipes | **pack** |
| Mining | What is worth digging and where | Real amounts from physics that mine out; miner program | | Processing plant as a sized part | **feature** |

### 2.5 War, crew and gear

| Site pages | What they hold | Verdict |
|---|---|---|
| Detection | Passive and active sensing, detection range from power and temperature, "no stealth in space" | **base**. This is limit 10 and the light design's sensing sums. Our numbers should agree with the site's cited formula where the assumptions match; a good check and a good tour |
| Weapons (four pages), Defenses | Lasers, particle beams, kinetics, missiles, nuclear warheads, armour | **base**, as reach and effect at range only (section 13; Alex, 23 Nov 20:25) |
| Warship Design, Warship Gallery, Space Fighters, Tactics, Planetary Attack (three pages) | Doctrine and tactics | **out**: players never hand-pilot, no dogfighting, nothing is fired |
| Sensor Deck | Navigation sensors (sextant, star tracker, pulsar positioning), tactical sensors (passive arrays, telescopes, radar, lidar, gravity gradiometer, mass detector, neutrino detector), remote sensing (radar mapping, gamma ray spectrometer). Mostly description, few formulas | **base** (section 12) |
| War, Military, Astromilitary, Quasi-Military, Colony Service, Space Rescue | Organisations and their jobs | **feature**: patrol and rescue are roles (programs); the rest is reading, **link** |
| Crew | Crew posts and sizes | **base** as crew count per role, lightly |
| Sidearms (four pages), Body Armor, Gear, Man Amplifiers, Non-Space Vehicles | Personal kit | **out** of SSE4's scope (a game system pack's business) |
| Space Suits | Suit types | **out** for now; surface wander may want one figure (how long outside) later |

### 2.6 Worlds, nations, history, culture

None of this is data to import. It is the site's essay half, heavy with long quotations from published fiction that are not the author's to license. SSE4 already has, or has designed, a working answer to most of it:

| Site section | SSE4's answer |
|---|---|
| Maps of Space, Weird Astronomy, Worldbuilding, Fabulous Locations | The galaxy, star census, system formation and night sky. SSE4 generates what these pages teach people to fake |
| Tech Level (read: prose on tech scales and uneven tech, no figures), Tech and Society, Future History, Modeling and Calculating History, Alternate History | FT's ladder and dated capability; galaxy histories; Minerva |
| Galactic Survey, Settlement, Colony pages, Planet Roles, Real Colony Sites | Remote survey, explorer program, tags and world hooks |
| Trade, Pirates, Starship Ownership, Future Money, Macroeconomics | Trader and pirate programs now; the economy pack (4.1) |
| Empire, Empire Dynamics, Empire Maps, Space Law, Telecom | Territory overlays; the message rule on FTL (example packs 4A) |
| Measurement, Language, Symbols, Calendars | "This setting": units, calendars, naming |
| Aliens (five pages) | The life design |
| MacGuffinite, Unobtainium, Handwavium (reactionless drives, antigravity, matter transmitters, nuke dampers) | FT's honesty marks. His three words map almost exactly onto ours: unobtainium is "not yet built", handwavium is "fictional". Worth saying to him |
| Respecting Science, Thermodynamics, Misconceptions | The fudge register and "Why?" lines |
| Time Travel, Psionics, Apocalypse, Games, Mythology, Art, Music, Atomic Creations | **out** |

Verdict for the whole group: **feature** plus **link**. With his agreement, each matching SSE4 screen or tour carries a "Read more" link to the right page of his site. That is "everything on the site is in SSE", honestly: the physics runs, and the reading is one press away.

## 3. What SSE4 already covers, in one list

- Rocket equation, relativistic too; propellant by fuel type; refuelling by capability.
- Hohmann, Lambert, launch windows, torch courses, escape and capture; voyages across stars with ship time and home time.
- Ships from parts: tanks from fuel mass and density, radiators from waste heat and temperature, shields from g/cm², heat shields from entry heating, wings from landing speed.
- Aerobraking, entry, landing and take-off per ship per world.
- FT (accepted, not built): ten limits, marks, advances, the ladder, capability per civilisation, "show it without", light by part.
- FT section 6: spin habitats, elevators, orbital rings, launch tracks, stellar engines, each with its paper.

## 4. What the site shows that SSE4 lacks

### 4.1 Limits the ten miss

| Proposed | Science line | Figure | Why the site forces it |
|---|---|---|---|
| **Materials only take so much heat** (a second figure under limit 7, or limit 11) | Melting and softening; exhaust speed of a hot-gas rocket goes as the square root of temperature over gas weight | highest working temperature of a substance, from the sheet | It is the whole story of nuclear thermal engines: solid core is capped by its fuel rods melting, so liquid core, then gas core, then open cycle are the same engine with this one figure pushed. It also caps radiator temperature, which FT currently types in per fuel |
| **Beams spread** (limit 11 or 12) | Diffraction: spot size goes as wavelength times distance over aperture | aperture and wavelength of a sender; power a receiver can take per square metre | Beamed power, laser sails, laser launch and laser-thermal engines all leave the power plant at home, which sidesteps limit 3 with no fiction at all. The same figure sets how fast a link can carry data and how sharp a telescope sees, so it serves comms and survey too. One mechanism, several uses |
| **Some things do not shrink** (a rule on advances, not a limit) | Critical mass; fusion ignition; smallest pulse unit | smallest working size of a part | FT's ladder scales parts up when a civilisation is short of a level, and down past it. A fission reactor or an Orion pulse unit has a floor. Without it, the generator will happily draw a one-tonne Orion |

Considered and left where they are: wear and service life (belongs with build cost, in the economy pack); psychology and crew size (soft, a GM's business); magnetic field strength (a substance figure for superconductors, read by fusion and magnetic nozzles, which fits under limit 7 without a new limit).

### 4.2 Drive families with no home yet

| Family | Why it needs something new | Smallest change |
|---|---|---|
| **Electric** (ion, Hall, plasma, arcjet): flown for decades | Thrust is set by electrical power, so the ship is power plant plus radiators plus thruster. SSE4 ties power to the fuel | A power plant as its own part (section 2.2), and a drive that reads watts. Then one drive can also **trade thrust for exhaust speed** at fixed power ("gears"), which the site covers and FT's figures allow |
| **Sails** (solar, laser, magnetic): solar sails have flown | No reaction mass at all; thrust from sunlight falls with distance squared | A drive whose thrust is a function of where it is. Needs the slow-spiral leg-maker |
| **Beamed** (laser thermal, laser sail, microwave) | Power comes from a construct somewhere else | "Beams spread" above, and a leg that depends on a sender staying lit: a lovely plot hook |
| **Pulse** (Orion, Daedalus, Medusa) | Throws bombs; thrust comes in kicks | Fits limits 2, 3 and 4 as they stand, plus the size floor. The pusher plate and shock absorbers are parts |
| **Nuclear thermal beyond solid core**; fission fragment; nuclear salt water | Same engine, hotter | The heat figure above |
| **Ramscoop** | Collects reaction mass on the way | A scoop already exists for refuelling; this is the same part in flight, and the drag sum says honestly that it mostly does not work |

Also missing, and wanted by the above: **slow-spiral courses** in NAV (months of gentle thrust; the cost is close to the difference of the two circular speeds). To confirm with the travel thread.

## 5. Their equations as Equation blocks

Alex: "They have rocket equations - we have equation blocks!" The Equation block (`sse.math.equation`, design accepted 22 Nov) takes a formula as text with every letter wired from a source. It has no loops, so a closed formula fits and an iterative solver does not.

The honest position today: **the ship and flight maths exists, but not as blocks.** The 71 physics blocks moved to named Equations on 23 Nov were the kernel's. Flight lives in Rust (`crates/sse-nav`) and ship sizing in TypeScript (`packages/renderer/src/ship.ts`, `ship-design.ts`). Nobody can open the rocket equation on a flow and edit it. (From a quick read of the repo today, not a full audit; the owner threads should confirm.)

| Equation on the site | In SSE4 today | Proposed |
|---|---|---|
| Rocket equation and its mass ratio forms | Rust, `sse-nav` (`dv_m_s`, `propellant_for`); relativistic form in `voyage.rs` | Named Equation; the relativistic one is the general case and the classic one its low-speed reading |
| Exhaust speed from specific impulse | Not needed (SSE4 stores exhaust speed) | A unit on the sheet ("seconds of impulse"), not an equation |
| Thrust power, mass flow, jet power | TypeScript, `ship.ts` | Named Equation |
| Exhaust speed of a hot-gas rocket from temperature, gas weight and pressures | Missing (typed per ship) | **New** Equation; inputs from Look up (gas properties) and the heat figure |
| Exhaust speed of a fusion rocket from the share of mass turned to energy | Missing | **New** Equation; inputs from the reaction rows on the sheet |
| Best exhaust speed for a trip (about 0.72 or 0.63 of the delta-v) | Missing | **New**, small; useful to Orac when designing a ship for a route |
| Staging | Missing | **New** Equation (sum over stages is a fixed-length list, fine) |
| Radiator area from waste heat (Stefan-Boltzmann) | TypeScript, ship designer | Named Equation |
| Panels shading each other; best radiator temperature; heat pump power; radiator mass | Missing | **New** Equations |
| Power plant cycle efficiency (Brayton and Carnot bound) | Missing | **New**; this is what sets waste heat honestly instead of typing it |
| Orbital and escape speed, vis-viva, Hohmann delta-v and time, synodic period, phase angle | Rust, `sse-nav` | Named Equations |
| Lambert's problem, Kepler's equation, entry and descent integration | Rust, `sse-nav` | **Stay in Rust** as big blocks with a reason (they iterate), like the 14 the kernel kept |
| Constant-thrust trip time and delta-v (brachistochrone) | Rust, NAV torch courses | Named Equation |
| Spin gravity: radius, rate, Coriolis | FT section 4, not built | **New** Equation, with FT4 |
| Entry heating (Sutton-Graves), ballistic coefficient | Rust, `entry.rs` | Named Equation |
| Detection range from power, area and temperature | Light design's sensing arithmetic | Named Equation; check against the site's cited formula |
| Shield mass for a dose | Rust and TypeScript | Named Equation |
| Beam spot size and power delivered | Missing | **New**, with "beams spread" |
| Life support mass balance per person | Missing | **New**, with limit 9 |

So roughly ten already exist and need to move from code into blocks, about ten are new, and three stay as solvers. This is the same move the kernel made with its 71, applied to ships, and it should ride with FT3 and FT4 (which already rework how ships read their figures) so it is done once.

Rights on equations: a formula is not anyone's property. Each block cites the textbook or paper it comes from, not the site. Where the site credits a contributor for a derivation (it names several), the block cites that person's own published source if there is one, and otherwise we derive it ourselves and say "computed".

## 6. How it becomes packs

One general rule first: **the mechanism goes in the base, the catalogue goes in a pack.** Nothing below is a special for this site.

| Where | What | Needs his permission? |
|---|---|---|
| **Master data sheet** (`base-reference`) | Propellants as substances with density, storage temperature and gas weight; fusion and fission reactions with energy and neutron share; highest working temperature and strength for structural and radiator materials; shield materials; the "seconds of impulse" unit | No. Cited to handbooks and papers |
| **`base-tech`** (FT1) | The added limit or limits; marks for each figure; **real, flown advances** for every family that has flown (chemical variants, ion and Hall thrusters, arcjets, solar sails, solar and fission and radioisotope power) | No |
| **`base-ships`, `base-nav`** | The Equations of section 5; power plant as a part; shadow shield; drives that read watts or sunlight; slow spirals | No |
| **"Sol today"** (already planned) | Real spacecraft as recipes | No |
| **New pack, working name "Paper spacecraft"** (free) | Every well-documented engine and design study that was never built, as advances and ship recipes, each cited to its own study: nuclear thermal from the Rover and NERVA reports, Orion, Daedalus, Discovery II, HOPE, EMPIRE, gas core, fission fragment, laser sails, launch loops and skyhooks. All marked "not yet built". Plus tours: why torchships are hard, why nothing hides, how big the radiators really are | **No** to build it from primary sources with our own selection. **Yes** to follow his selection and order, to use his names for things he named, to quote him, or to call it anything like "Atomic Rockets" |
| **With his agreement: the same pack, as "the Atomic Rockets pack"** | His name on it, his choice of what goes in, "Read more" links from each engine, ship and screen to the matching page, and anything he wants to write for it | Yes, entirely |

On a shelf: a physics and ships pack, free, featured as original while it is ours alone. If it takes his name it is still free (third-party names are never paid, standing ruling) and carries his credit line as he words it.

Why one pack and not several: the content filter on Explorers already lets people find "engines", "ships" and "tours" inside it. It can split later if it grows past what a phone wants to download.

**The pitch this supports.** Every one of those studies stopped at a report. In SSE4 the Discovery II can be flown to a Saturn that has real moons, with its radiators drawn at their true size, glowing in infrared, and its hydrogen actually running out. Then the same ship can be handed to a civilisation one level short and watched to swell tenfold. That is what "fly their rockets properly simulated" means, and it is true the moment FT3, FT4 and the pack land. It is **not true today**: today SSE4 flies six stock ships on four fuels. The approach message should not claim more than is live.

## 7. The rights position, plainly

Guidance, not legal advice.

**What the site says.** Every page read carries the footer "©1995-2023 Winchell Chung". No page read carries a licence, a reuse statement or a fair use note. Preliminary Notes says the site is for "a science fiction writer or game designer", that it is "just me", and that readers send corrections ("the 'stone soup' method of website design"); for disagreements it points people to the ToughSF Discord server and two older discussion groups. So: all rights reserved, by an author who plainly wants the material used to make better science fiction and games, and who has not said how.

**Whose it is. This matters as much as the licence.**
- His own explanations and his arrangement of the tables are his.
- Large parts are quoted from others: long passages of published fiction, essays by named contributors, forum posts. Those are not his to license, and he could not give them to us if he wished.
- The artwork is credited to dozens of artists and studios. Same again. SSE4 generates its own pictures anyway.
- **Many rows of the engine tables come from a commercial board game** (the page credits Philip Eklund's *High Frontier*). Those figures are partly game values. They must not be used as physics, and they are that publisher's, not his.

**The safe route, and the one I recommend whatever he says:**
1. Use the site as the data sheet thread already does for FT1: **as a map to primary sources.** Take each figure from the paper, NASA report or textbook, cite that, and run it through the citation check. Facts and formulas are free; this route needs nobody's permission and gives better numbers.
2. Do not copy his text, his tables as compiled, his selection and ordering, or any image.
3. Choose and order our own content from the sources. Where we end up with the same engines he lists, that is because they are the engines that exist.
4. **Nothing named after the site or its author ships before he agrees**, and that includes "Read more" links presented as a feature, a credit line, and the pack name.
5. If he agrees, get it in writing (an email is enough), saying what he is happy with: the name, links, quoting him, his involvement. Record it in the pack's notice.
6. His agreement covers his own work only. Contributors' essays and all artwork stay off limits unless their owners agree separately.

## 8. Contact route and draft message

**Route.** No email address or contact form appears on the home page, Preliminary Notes, Footnotes or his own page. The home page links two accounts of his: Mastodon, `https://spacey.space/@nyrath`, and Patreon, `http://www.patreon.com/nyrath` (Patreon lets a member message a creator). He also points readers to the ToughSF Discord server, where he is known to take part, but that is a public room and a poor place for a first approach. I have not guessed an address. Recommended: a short Mastodon message asking for the best way to send a proper note, then the fuller message wherever he says.

**Short opener (fits a Mastodon post):**

> Hello! I'm Alex. I've spent years on Star System Explorer, a free, open source star system and universe builder for SF gamers and writers, and I've just fallen head first into Atomic Rockets. The new version sizes ships from real physics (radiators from actual waste heat!) and flies them on real fuel. I'd love to tell you more and ask a favour. What's the best way to drop you a proper note?

**Sent.** Alex sent the fuller message by email on 23 Nov 2026, about 21:40, to an address he found himself. No reply yet. Until the author agrees in writing, the rules of section 7 stand unchanged.

**The fuller message, for Alex to edit and send himself:**

> Subject: Flying your rockets (an open source project that owes you a pint)
>
> Hi Winchell,
>
> I'm Alex, and I build Star System Explorer, a free and open source toolkit for making star systems (and now whole galaxies) for tabletop RPGs and storytelling. I've just spent far too long on Atomic Rockets when I should have been working, and I had to write.
>
> The new version (SSE4, early alpha, at v4.starsystemx.com) has one rule: lean into the physics, never fake it. Every bit of physics is a small editable block with its source cited. Ships aren't drawn, they're worked out: tanks sized from the fuel, radiators from the real waste heat (they come out gloriously huge), heat shields from the entry heating. Then they fly real transfers on real fuel, aerobrake at real worlds, and run out of propellant if you get it wrong.
>
> Which is why your site stopped me in my tracks. You've got the equations; we've got equation blocks that people can open, read and change, and that actually size and fly a ship. You've catalogued decades of engines and design studies that never left paper; I'd love people to be able to fly them. We even seem to have reached the same honesty system from different directions: we mark tech as built, not yet built, or fictional, and every bit of handwavium has to say which law it breaks.
>
> To be clear about how I'd go about it: figures come from the original papers and reports, each cited, and nothing of yours (text, tables, pictures, or the name) goes anywhere near it without your say-so. It's all free and Apache licensed, and anything built around a third party's work stays free for good.
>
> So, two questions. Would you be up for having a look and telling me where I've got it wrong? And would you be open to a proper "Atomic Rockets" pack, credited and linked back to your pages however you like, shaped as much or as little by you as you fancy?
>
> No pressure at all, and thank you either way for the site. It's a ridiculous labour of love and it shows.
>
> Cheers,
> Alex
> (FrunkQ on GitHub: github.com/FrunkQ/star-system-explorer)

Notes on the draft: it claims only what is live today (stock ships, radiators, real fuel, aerobraking), and offers the rest as an intention. "Owes you a pint" and "years on" are guesses at Alex's voice and history; change freely. It does not mention money because there is none: the pack is free.

## 9. Packages, if accepted

| Package | What | Suggested owner | Waits on |
|---|---|---|---|
| RK1 | Sheet rows: propellants, reactions, working temperatures, shield and radiator materials, the impulse unit; cited | data sheet thread, with FT1 | nothing |
| RK2 | `base-tech` additions: the heat figure, "beams spread", the size floor; marks; flown advances for every family | data sheet thread (FT1) and the FT design thread for the note | question 3 |
| RK3 | Ship and flight formulas as named Equations (section 5), done together with FT3 and FT4. Agreed split: the travel thread takes the flight formulas in `sse-nav` (rocket equation and its relativistic form, vis-viva, orbital and escape speed, Hohmann, synodic period and phase angle, brachistochrone, entry heating, ballistic coefficient; Lambert, Kepler and entry integration stay in Rust with reasons); the renderer thread takes ship sizing (thrust power, radiators) | travel and renderer threads, kernel thread for the block side | question 4 |
| RK4 | Power plant as a part; shadow shield; drives that read watts or sunlight; pulse drive parts | renderer thread (generator), travel thread (records) | RK2 |
| RK5 | Slow-spiral courses; a beamed leg that depends on its sender | travel thread | RK4 |
| RK6 | "Paper spacecraft" pack: advances and ship recipes from primary sources, with tours | Traveller thread or a new pack thread; citations by the data sheet thread | RK1 to RK4 |
| RK7 | Published mission tables as NAV test fixtures for Sol | travel thread | nothing |
| RK8 | If he agrees: name, credit, "Read more" links, his edits | whoever owns RK6 | his written yes |

## 10. Questions for Alex (1 to 10 answered yes, 11 changed, 23 Nov 20:25)

1. **"Everything" means four things**: physics into the base packs, design studies into one free pack, essays answered by SSE4's own features with links out, and combat, sidearms and deck plans left out under the standing rulings (detection comes in). Recommended: yes.
2. **Build from primary sources whatever he says**, using the site only as a map, so the work never depends on his answer and his answer only decides the name, the links and his involvement. Recommended: yes.
3. **Add to the limits**: "materials only take so much heat" as a second figure under limit 7, "beams spread" as limit 11, and a smallest working size on advances. Recommended: yes to all three. The first is needed for any honest nuclear engine.
4. **Move ship and flight formulas out of Rust and TypeScript into named Equation blocks**, as one job alongside FT3 and FT4, leaving only the iterating solvers in code. Recommended: yes. It is what makes the Equation block claim true for ships, and it is the strongest line in the approach.
5. **A power plant becomes its own sized part**, so electric drives (the only advanced drives that have actually flown) can exist, and a drive can trade thrust for exhaust speed. Recommended: yes.
6. **The pack's working name is "Paper spacecraft"** until he agrees to anything else, and it is featured as an original. Recommended: yes; tell me if you have a better name.
7. **Engine rows that trace to the High Frontier board game are never used**, even where the site shows no other source; those engines wait until a paper is found. Recommended: yes.
8. **Approach by a short Mastodon message first**, asking where to send a proper note, since the site gives no email. Recommended: yes. If you already know him or his address, say so and skip it.
9. **Send before or after there is something to show?** Recommended: send the short opener now, since a friendly hello costs nothing, but hold the fuller message until one of his favourite design studies actually flies in SSE4 (RK4 and a first recipe), with a clip. "Here is the Discovery II at Saturn" beats any description.
10. **Published mission tables as test fixtures** for Sol (RK7): SSE4's transfers should match them within stated tolerance, and where they differ we find out why. Recommended: yes, it is cheap and it is the "properly simulated" claim checked.
11. *Changed by Alex: weapons come in as science and "in range" sums, nothing is fired; see section 13.* Originally: **Weapons stay out for now**, to be revisited as its own design note once the first game pack wants them. Recommended: yes. If you would rather have lasers and kinetics as physics now (ranges and heat only, no dogfighting), say so and I will write that note.

## 11. Power as a general input (Alex, 23 Nov 20:21)

Alex: "we need 'Power' as a new general fuel - so reactors and power loss - rather than trad rockets. Constructs had the scaffolding for that (power plants) but was never used in engines." This is question 5 made general, and it is to be one model shared with the real spaceflight pack (track RS) and the future tech balance checks (FT section 15).

### 11.1 What SSE3 had (read 23 Nov from `static/rulepacks/starter-sf/`)

| File | Shape | What it gives | What it lacks |
|---|---|---|---|
| `engine-definitions.json` (12 engines) | `fuel_type_id`, `thrust_kN`, `efficiency_isp`, `atmo_efficiency`, `powerDraw_MW`, `drive_tags` | engines as parts a construct lists by id and quantity; the right families (chemical, nuclear thermal solid and gas core, Orion, gridded ion, Hall, plasma, fusion, antimatter) | `powerDraw_MW` was never read by anything |
| `construct_templates.json` (59 constructs) | `systems.power_plants[{type, output_MW}]`, `systems.life_support{consumables_max_person_days}`, `engines[{engine_id, quantity}]`, `fuel_tanks[]`, `can_aerobrake`, `has_landing_gear` | a power plant on nearly every construct | `type` is free text; output never met a draw; no heat, no radiators |
| `fuel-definitions.json` | density, refuel tags, availability | already carried into SSE4's refuel rules | energy per kg |

The future tech jet power check, run on SSE3's own numbers (*computed* here: jet power is half of thrust times exhaust speed):

| SSE3 engine | Jet power | Stated draw | Reading |
|---|---|---|---|
| Gridded ion array (50 N, 12,000 s) | 2.9 MW | 2 MW | gives out 1.5 times what it takes in |
| Hall thruster (600 N, 1,800 s) | 5.3 MW | 0.1 MW | 53 times |
| Plasma drive (5 kN, 4,000 s) | 98 MW | 20 MW | 4.9 times |
| For comparison, Dawn's real ion engine (about 91 mN, 3,100 s, 2.3 kW; from memory, RS will cite) | 1.4 kW | 2.3 kW | 60% efficient, the rest is heat |

So the scaffolding was right and the numbers were never checked against each other. Import keeps every SSE3 field, and the balance check marks these three for the GM to fix or to keep as stated fiction. Nothing is silently changed.

### 11.2 The model: every part is a converter on a short list of buses

One mechanism. A construct has a few **buses**, and every part says what it takes from each and what it puts on each, per second:

| Bus | Unit | Who puts in | Who takes out |
|---|---|---|---|
| **Electric** | W | solar arrays, reactors with generators, radioisotope units, fuel cells, batteries | electric engines, quarters, sensors, radios, heat pumps, lasers |
| **Heat**, with its temperature | W at K | everything, as its losses | radiators; propellant, where an engine dumps heat into its own exhaust |
| **Propellant**, by substance | kg/s | tanks, scoops, ice processors | engines, fuel cells |
| **Jet** | N at m/s | engines | the ship's motion |

A power plant turns fuel, sunlight or decay into electricity and heat. An engine turns propellant plus energy into a jet and heat. A radiator turns heat into light. A laser turns electricity into a beam and a great deal of heat. They are the same kind of thing, so there is no "engine power" special case: **an engine is one more load on the electric bus**, and FT 15.1's "energy in, energy out" is simply the electric and heat buses closing.

This is wider than engines on purpose (think wide). The same ledger later carries air, water and food for limit 9 (SSE3's person-days are a tank on that bus), and ore and product for a mining plant. Those are named here and not built now.

**Where an engine's energy comes from** is one field with four values, and it is the whole of Alex's point:

| `energy_from` | Meaning | Examples | Check that applies |
|---|---|---|---|
| `fuel` | the propellant carries its own energy | chemical, nuclear thermal, fusion torch, antimatter, pulse | energy pays for the exhaust (FT 15.1) |
| `electric` | drawn from the electric bus | ion, Hall, plasma, arcjet, mass driver | jet power is at most efficiency times the watts it is given |
| `beam` | sent by another construct | laser thermal, laser sail, microwave | what arrives, from "beams spread" |
| `sunlight` | taken from the star where the ship is | solar sail, solar thermal | flux at that distance |

What falls out with no further rules:
- **Thrust follows power.** An electric engine given P watts makes thrust 2 × efficiency × P / exhaust speed. A solar-electric ship's thrust falls with the square of its distance from the star, as Dawn's did. A reactor ship's does not.
- **Gears.** An engine that lists a range of exhaust speeds trades thrust for economy at fixed power, by the same formula.
- **Everything competes.** Run the laser or the radar and the engine gets less. A GM can read that off one line.
- **Losses become radiators.** Each converter's loss lands on the heat bus at that part's temperature, and FT 15.2 sizes hot and cool panels separately.
- **Sources follow a law.** Constant (reactor), sunlight (array: watts at a reference flux), decay (radioisotope: a half-life and a start date, so Voyager's power today is computed from the clock), fuel (fuel cell: burns propellant).
- **Size comes from the advance.** A source's mass is its watts times the kilograms per watt of its advance (a limit 3 figure that already exists), so a level-short reactor swells like any other part.

### 11.3 Field names (confirmed by the travel thread, owner of the ship record, 23 Nov 20:33, with six corrections folded in)

SSE4 writes SI with the unit in the name. SSE3's fields stay readable and map across:

| New field | On | SSE3 field it reads | Notes |
|---|---|---|---|
| `power[]`: `id`, `kind`, `electric_w`, `thermal_w`, `reject_k`, `mass_t`, `law` | construct | `systems.power_plants[]`: `output_MW` to `electric_w`; `type` text matched to a `kind` where it can be, kept as the name where it cannot | `kind` is an advance id (`solar-array`, `fission-reactor`, `radioisotope`, `fuel-cell`). `thermal_w` and `reject_k` optional: given both outputs, the efficiency is checked, not typed |
| `law`: `{ kind: "constant" }`, `{ kind: "sunlight", ref_flux_w_m2 }`, `{ kind: "decay", half_life_y, since_s }`, `{ kind: "fuel", fuel }` | power source | none (SSE3 was constant) | default constant. `since_s` is universe seconds, the one clock |
| `engines[]`: `engine`, `count` | construct | `engines[]`: `engine_id`, `quantity` | `accel_g`, `sustained` and `exhaust_km_s` stay on the ship as readings computed from `engines[]`. A ship with no `engines[]` keeps typing them, so every stock and pack ship flies unchanged |
| engine definition: `fuels[]`, `thrust_n`, `exhaust_km_s`, `energy_from`, `electric_w`, `efficiency`, `reject_k`, `mass_t`, optional `exhaust_range_km_s: [low, high]` | pack | `fuel_type_id`, `thrust_kN`, `efficiency_isp` (times standard gravity), `powerDraw_MW` to `electric_w` | `fuels[]` is the same list refuelling already reads. `efficiency` is jet power over energy in. `energy_from` is guessed on import from SSE3's `type` text and marked as a guess |
| `loads[]`: `name`, `electric_w`, `reject_k`, `mass_t` | construct | none | quarters, sensors, radios; each ends as heat. One name, `reject_k`, for the heat's temperature on sources, engines and loads, so radiator sizing reads one field. Masses in tonnes, as the rest of the ship record |

`can_aerobrake` and `has_landing_gear` are read on import as the builder's intent only; SSE4 computes both per world.

One import caution: SSE3's stock list carries a few names that belong to other people's settings (a fusion drive named after one, a famous battle station). Under the named-setting ruling these are renamed on import into the base packs, and kept as typed in a user's own save.

## 12. Sensors (Alex, 23 Nov 20:25: "Do they have sensors? We have a lot of strings in that area we need to harmonise, down to the visualisation of x-rays")

**Does the site have them?** Yes, on two pages. Sensor Deck lists the kinds (section 2.5); Detection gives the range sums. It is mostly description, with few usable formulas, so it is a checklist of kinds to cover, not a data source.

**What SSE4 has today is four vocabularies that do not meet** (read from the repo, 23 Nov):

| Where | What it says a sensor or band is |
|---|---|
| `crates/sse-sense` | The physical one: a wavelength range in metres, an aperture, noise, a stare time. Detection range is computed from the target's light. Cited (radar equation, diffraction, dust) |
| `packs/traveller`, from SSE3's `sensors.json` | Twelve named kinds, each with a **typed range** in metres and a sentence of "targets" and "reveals" (astrometer 50 au, two telescopes, two spectral analysers, passive radio, search radar, thermal, gravimetric, bio scanner, camera, datalink), plus nine Traveller range bands entered as if they were sensors |
| The light table and band recipes | Nine light bands by wavelength: `gamma`, `xray`, `uv`, `visible`, `near-ir`, `mid-ir`, `far-ir`, `mm`, `radio`; recipes `eye`, `camera`, `sho`, `hoo` and one per band |
| View controls and the galaxy renderer | Six view bands: `visible`, `infrared`, `ultraviolet`, `xray`, `radio`, `mag`, with a hand-written map from the nine to the six, and each screen listing its own |

Coming on top: the real spaceflight pack's instruments (band, aperture, field of view, resolution), the remote survey methods (wobble, transit), and FT's light by part (plumes in X-ray).

**The model to follow (Alex, 23 Nov 21:44): "a ship gets a sensor pack (array) now. That is the model to follow."** A ship carries an array of sensors picked from its pack's sensor list, as the Traveller pack's ships already do. That stays exactly as it is. Nothing below adds a second structure: it changes only what one entry in the array *is* (an instrument with a band and an aperture, where today it is a name with a typed range), and the "default sensor fit" of section 17 is simply a named array for a tech level that a ship gets when nobody has chosen one.

**Proposed: one table, one shape, one rule.**

1. **One Bands table on the master data sheet.** Each row: id, plain name, from and to in metres, which view band shows it, its default recipe. The light table, the view switch, every screen's list, every sensor and every recipe refer to a band by id and nothing spells a band any other way. The hand-written map goes. A pack adds a band by adding a row.
2. **One instrument shape**, which is `sse-sense`'s, plus a **method**: image, spectrum, astrometry, active (radar, lidar: adds a transmit power, so it is a load on the electric bus and it lights you up for everyone else), and field or particle methods below.
3. **No typed ranges.** Like "no can-land flags": how far a sensor sees a thing is computed from what the thing gives off and what the instrument can catch. A fusion torch is seen across the system and a cold rock is not, from the same telescope. SSE3's typed ranges import as an **authored range**, kept and marked, so old saves and the Traveller pack keep working until their kinds are given apertures.
4. **Range bands are a measure, not a sensor.** Traveller's adjacent to far are a labelled reading of distance, which belongs with the setting's units ("This setting"), where "2 jumps" already lives.
5. **Things that are not light become channels.** The light design says every object publishes its light by band. The same idea, widened: an object publishes on **channels**, and light by band is the first. Others, each from physics SSE4 already computes:

| Channel | What the target gives off | Instrument | Honest reading |
|---|---|---|---|
| Light, by band | its light (built) | telescope, spectrometer, radar return | built |
| Magnetic field | its field (the Mag band shows it already) | magnetometer | built |
| Gravity | mass over distance cubed | gradiometer | built in orbit; "detect a ship by its mass" is real but reaches metres to kilometres, and the card will say so |
| Particles | belts, wind, reactor leakage | particle counters, gamma spectrometer | built |
| Neutrinos | reactor and fusion power over distance squared | a very large detector | allowed, and absurdly large; a nice "not yet built" |
| Life | nothing defined | "bio scanner" | fictional as a box; real as a spectrum of the air, which the survey ladder already does |

6. **What a seat sees is its instrument list.** Already ruled (a seat only gets bands its ship's sensors carry); now it reads the same list.
7. **Visualisation is the recipe on the band row**, so X-ray looks the same on a galaxy, a world and a ship's plume, and a new composite is data.

## 13. Weapons, as science and "in range" only (Alex, 23 Nov 20:25)

Alex: "Add weapons - Traveller Pack will need them - so lets get the data sorted out from a scientific perspective. Yeah - just 'in range' calcs - we dont fire anything."

**The general thing behind it: effect at range.** A weapon is a converter (section 11) that sends something. The one question SSE4 answers is: *with this weapon, against that target, at this distance, what arrives and is it enough?* Three readings, never a refusal, in the style of the availability reading:

| Reading | Meaning |
|---|---|
| **In range** | what arrives passes what the target can take, with the margin shown |
| **Marginal** | within a stated factor either way; the GM and the dice decide |
| **Out of range** | what arrives, as a share of what is needed |

Nothing is fired, no damage is tracked, no ship is flown by hand. The reading is a fact on a card and a ring on the chart (an overlay under HUD, the same mechanism as the hex grid and territory). A game pack's own rules decide what happens; the physics sits beside them the way a real distance sits beside "2 jumps".

| Weapon family | What arrives at distance d | Figures | Sources to cite (not the site) |
|---|---|---|---|
| **Laser** | power over spot area; spot from wavelength, aperture and pointing jitter. Falls as distance squared | beam power, wavelength, aperture, jitter, efficiency | standard optics (diffraction), the same Equation as "beams spread" and as a telescope's resolution |
| **Particle beam** | as a laser, with spread from the beam's own charge and temperature | energy, current, divergence | accelerator physics texts |
| **Kinetic** (guns, coilguns) | energy per kg is half the closing speed squared (*computed*: 3 km/s is about the energy of its own mass in explosive). The question is not reach but **can it hit**: flight time against how far the target can move | muzzle speed, target acceleration | mechanics; our own derivation of the miss distance |
| **Missile** | a small ship. Can it reach the target is **the same sum NAV already does** for any ship: delta-v, acceleration, time | the missile's own engine and propellant | NAV |
| **Nuclear warhead** | energy per area, yield over 4π distance squared, with no air to carry a blast | yield | Glasstone and Dolan, The Effects of Nuclear Weapons (a US government publication) |
| **What the target can take** | energy per area to heat, melt or boil through its skin | from the master data sheet: heat capacity, heat of melting and boiling, density, thickness | the sheet; the "materials take so much heat" figure of section 4.1 |

What falls out, with no weapons special case:
- **Light lag is the real limit on range**, not power. At one light second the target has moved before the light arrives. The reading shows the chance of a hit from the target's acceleration and the delay, as odds for the dice ("danger is drama"). The site prints a formula for this that its author derived himself and asks readers to check; we derive our own and say "computed".
- **Weapons shape ships through heat.** A 20 MW beam from a 30% efficient laser puts 47 MW on the heat bus (*computed*), so a warship is radiators with a gun attached. This is true, it is visible, and it comes free from section 11.
- **Firing reveals you**, through limit 10, in whatever band the weapon and its radiators glow.
- **Detection, identification and targeting are three ranges**, all from the sensor sums of section 12: enough light to notice, enough resolution to tell what it is, enough precision to aim.
- **The Traveller pack** maps its range bands and weapon classes onto these readings, or keeps its own table as a pack rule shown beside the physics, exactly as jump is.

Still out: tactics, fire control, damage and repair, fighters, boarding, sidearms and body armour.

No new limit is needed. "Beams spread" (accepted, question 3) covers lasers, links, sensors and beamed power with one Equation, and armour is limit 7 with the heat figure.

## 14. Added packages

| Package | What | Suggested owner | Waits on |
|---|---|---|---|
| RK9 | The bus ledger: power sources with laws, engines as parts with `energy_from`, loads; electric and heat buses closing; thrust from power; SSE3 fields read on import with the balance check marking what does not close | travel thread (ship record, NAV), kernel thread (import, checks), with RS writing its real parts to the same fields | accepted |
| RK10 | One Bands table on the sheet; every band string points at it; the hand-written band map removed | data sheet thread, then light, renderer and chart room threads each for their own strings | accepted |
| RK11 | One instrument shape with a method; authored ranges on import; range bands moved to the setting's measures; gravity and magnetic channels | light thread (owns `sse-sense`), Traveller thread for its pack | accepted |
| RK12 | Effect at range: the laser, kinetic, missile and warhead Equations; the three readings; rings as a HUD overlay; weapons as loads on the ledger | light thread for beams, travel thread for missile reach, Traveller thread for its mapping | RK9 |

RK4 (power plant as a part) is absorbed by RK9.

## 15. Questions 12 to 17 (all answered yes, 23 Nov 21:30)

12. **One power model: every part is a converter on four buses** (electric, heat with its temperature, propellant, jet), with an engine's energy coming from fuel, electric, beam or sunlight, and SSE3's `power_plants` and `powerDraw_MW` read straight into it. Recommended: yes. Air, food and ore join the same ledger later.
13. **Sensor ranges are never typed.** They are computed from what the target gives off; SSE3's typed ranges import as marked authored ranges so nothing breaks. Recommended: yes.
14. **One Bands table on the master data sheet** that every band name in the app points at, including the view switch and the recipes. Recommended: yes; it is the harmonising you asked for, and it is mostly renaming.
15. **Channels beyond light**: objects also publish gravity, magnetic field, particles and neutrinos, so a gravimetric sensor or a neutrino detector is real physics with an honest (often tiny) range. Recommended: yes to gravity and magnetic now, particles and neutrinos when a pack wants them.
16. **Weapons give three readings (in range, marginal, out of range) and a ring on the chart**, with a game pack free to keep its own table beside the physics. Recommended: yes.
17. **SSE3 stock items that carry another setting's name are renamed when they come into the base packs**, and left alone in users' own saves. Recommended: yes.

## 16. The power flow diagram (Alex, 23 Nov 21:30: "this whole power bus looks like a cool diagram that could help a user understand?")

Yes, and it is the natural face of section 11: a ledger of who gives and who takes is exactly what a flow diagram draws. In the mockup.

**What it is.** A **Power** card, one press from any construct. Three columns read left to right: *Comes from* (arrays, reactor, the fuel's own energy), *Used by* (engine, quarters, sensors, radar, laser, and what the plant loses itself), *Ends up as* (jet, sent out as radio or beam, hot radiators, cool radiators). Band width is watts. Under it, five plain readings: thrust, power to spare, hot radiator area, cool radiator area, how visible the craft is.

**Why it earns its place.**
- **It is live.** Slide the distance from the star and a solar craft's flow thins and its engine starves. Turn the radar on and the engine gets less. Nobody has to be told that everything competes.
- **It shows the radiators' reason.** The bands that end in "hot radiators" are the radiators the generator draws. "Show it without" (FT question 10) is the same diagram with a band removed.
- **It shows broken books.** Where a balance check fails, the missing power arrives as a dashed band from a node called **Nowhere**, with one sentence saying how much. SSE3's Hall thruster as typed is in the mockup: 5.2 MW from nowhere. That is the honesty mark made visible, and it is what a GM fixes or chooses to keep.
- **It is honest about scale.** A flow under one part in a hundred is a hairline. A torch drive beside its own life support looks exactly as lopsided as it is.

**Depth.** Curious: the picture and the sentences. Science: the figures on each band and the efficiency of each part. Engine: press a band to open the Equation block behind it.

**Players** see their own ship's card with no honesty marks, like any other fact of their world. It is a good engineer's station: what can we turn off to get more thrust?

**Think wide: this is one view for any ledger, not a ship feature.** The same diagram, with other units, draws things SSE4 already computes:

| Ledger | Comes from, used by, ends up as | Already computed by |
|---|---|---|
| A construct's power | section 11 | RK9 |
| A world's energy budget | sunlight and inner heat; air, ocean, ice, life; light and heat back to space | the climate and weather track |
| An element's journey | supernova, star, disc; core, mantle, crust; ore that can be mined out | the matter design, section 5 |
| A habitat's air, water and food | stores and greenhouse; crew; recycled or lost | limit 9, later |
| A star's output by band | fusion; photosphere, corona, wind; light by band | the light track |
| An economy's goods | 4.1 | later |

So the recommendation is a general **Flow** view that any ledger can hand rows to (from, to, amount, kind), with the Power card as its first user. On a phone it runs top to bottom.

## 17. Fog of war from sensors (Alex, 23 Nov 21:30: "needs to just work and be tweaked by those that care")

**The idea: fog of war is not a sensor screen. It is the map.** The physics of sections 11 and 12 decides what each crew can see, and the only thing a player ever meets is things appearing, sharpening and fading on the chart they already use. All the sensor detail stays a step away, for those who care.

It also adds no second system. The secrets design (track SV, accepted today) already has the ladder a thing stands on for each audience, the eye with three states, pencil and ink on the GM's chart, and the rule that sensors "write the same grants a press does". Fog of war is simply **the physics pressing the eye**. SV said nothing spreads knowledge by itself in 4.0; this is the one automatic writer, and it writes dated `observe.grant` events like any other, so it undoes and it scrubs with the clock.

### 17.1 What a player sees: three marks

| Mark | Meaning | From |
|---|---|---|
| **Ink** | known: what it is, where it is | enough light to resolve it, or it is on their charts, or the GM revealed it |
| **Dotted** ("something warm", "a radio source") | noticed, not identified: SV's half-open eye, "heard of" | enough light to detect, not to resolve |
| **Faded, with a widening ring** | last seen: where it was, how long ago, and how far it could have gone since | a contact that went quiet. The ring grows from its last known speed and what its drive could do |

Nothing else is drawn, and nothing leaks by absence (SV's rule). One soft ring round the crew's own ship shows "a craft like ours would be noticed out to here". It swells when they throttle up, which teaches the whole idea without a word.

### 17.2 Making it just work

1. **One switch per campaign, three positions: Off, Simple, Full.** Off: the table sees what the GM reveals by hand, as now. Simple: the default for any game with ships. Full: the same thing with the reasons shown. They are **one calculation**; the switch only changes how much is shown. No mode has different physics.
2. **Every craft gets a sensor fit for free.** A default set of instruments from its tech level and size, as an advance like any other part. A GM who never opens a sensor still gets sensible fog. Fitting a better telescope is for those who care.
3. **What a thing gives off is never set up.** It comes from the power flow: a burning torch is seen across the system, a coasting hull only close by, a station by its radiators. The GM changes what a ship *does*, and visibility follows.
4. **"Why?" on any contact, both ways.** "Seen by its drive plume, in X-ray and infrared, by your main telescope." And on the GM's side for anything a seat cannot see: "Too cold: gives off 200 W, and they would need to be within 4,000 km." One line, in words.
5. **Passive is a pack that is always on; each active sensor is its own switch** (Alex, 23 Nov 21:46). The passive pack needs no attention and shows everything in range that gives itself away: a freighter burning hard, a hot radiator, **someone else's radar**. Each active sensor (radar, lidar) is a separate switch with its cost stated on it: it finds what is close and cold, and it tells everyone in range where you are. A radar is heard much further than it sees, because its signal falls with distance squared on the way to a listener and with the fourth power on the way out and back; that comes from the sums, not from a rule. Switches are orders to the ship, which fits "players never hand-pilot".
6. **Charts count.** Things on a crew's charts (stations, worlds, lanes) are known without being sensed. Buying a chart is SV's "Hand over".
7. **The GM always wins.** A press of the eye overrides the physics either way, and stays as authored ink. Physics never un-reveals what the GM revealed.
8. **Light delay, quietly.** A contact is drawn where its light says it was. In Simple that is just true and unremarked; in Full the card says "as seen 4 minutes ago".

### 17.3 Keeping it cheap

No heavy work for a click, so the sums run only when something changes: a drive starts or stops, a radar is switched, a leg ends, the clock is scrubbed. Each is one small sum per seat per object, from signatures the power ledger has already worked out. Between events nothing is computed; the faded ring grows by plain arithmetic.

### 17.4 On the GM's side

The chart stays in pencil and ink per SV. "See as: the Kestrel's crew" shows that crew's fog exactly. A run of contacts gained and lost is a list on the Run desk ("14:20 lost the freighter, drive off"), which is also what Minerva narrates from. Programs sense through the same fog (the record design already rules that), so a pirate that cannot see a coasting ship does not chase it: hiding works on computer-run ships with no extra rule.

### 17.5 What it does not do

No sensor operator mini-game: counters are switches with stated costs (section 17.7), never a console to play. Surface fog (what a landed party can see) is the same idea with terrain in the way, and waits for the surface work.

### 17.7 Countering sensors: ECM and cloaking (Alex, 23 Nov 21:46: "obvs we need an ECM or cloaking capability to counter these")

A sensor compares what a target gives off with the noise it has to see through. So there are only two things a counter can do: **give off less**, or **raise the watcher's noise**. Everything below is one of those, each is a part or an order with a cost on the power ledger, and all but the last are real physics that needs no new mechanism.

| Counter | What it does | Its cost, stated on the switch | Honest reading |
|---|---|---|---|
| **Run quiet** | drive off, active sensors off, non-essential loads off | you coast; you see less | built |
| **Cold face** | radiators and hull turned so the hot side faces away from one watcher | works in one direction only; anyone off to the side sees you as before | allowed |
| **Heat sink** | waste heat stored instead of radiated | a clock: "holds 6 hours", then it must be dumped, brightly. FT 15.3 already makes a sink say where the heat goes | allowed |
| **Absorbent skin** (Alex, 21:48: "radar absorbant hulls") | the hull returns less of what hits it, in the bands its coating is made for | the coating has mass, and what it absorbs becomes heat on the ledger, so a ship painted black to radar is a little warmer in infrared. Useless against a watcher who sees you by your own heat | built |
| **Shaped hull** | facets and edges send a radar's echo away from where it came from | works from some directions and not others; costs room inside. This is an ability the generator fits, like a heat shield, not a style | built |
| **Hide in the glare** | sit close to something bright in the watcher's view: the star, a world's limb, a ring, another ship's plume | you must be there, and stay there | built. The sensing sum already drowns a faint thing next to a bright one |
| **Jam** (ECM) | a transmitter that raises the noise in a band for every receiver in reach | power on the ledger, and you are now the loudest thing in the sky: they know *something* is there and on what bearing, but not what or exactly where | built |
| **Cancelling** (Alex, 21:48: "active jamming, like noise filtering on audio") | listens to an incoming signal and sends its mirror image, so the echo and the answer cancel at the watcher | power, and a receiver and computer good enough to copy the signal in time. It cancels toward one watcher only; a second watcher elsewhere sees you more brightly, because the energy has to go somewhere. Works on radar and lidar, never on your own heat | built (Alex, 21:49: "all exists now and could be put on ships"). In service on aircraft, but no published figures, so its strength is an authored figure, marked as such, until a primary source turns up |
| **Dazzle** | a beam into one telescope to blind it | power; needs to know where the watcher is; "beams spread" sets the reach | allowed |
| **Decoy** | a small part with a heater and a reflector that gives off a ship's signature | a part with mass, used up; a close enough look tells it from a ship | built |
| **False colours** | a transponder that lies about who you are | holds until someone resolves you well enough. This is the secrets design's "Told instead", unchanged | built |
| **Cloak** | sets what you give off in named bands to a stated share, as a fictional advance | whatever the setting says. It must fill FT's four fields: where the light goes, where the heat goes. "Nowhere" is allowed and is marked fictional | fictional |

**One mechanism under the first two rows: a construct's skin is a substance.** The master data sheet gives a substance its reflectance by band, which SSE4 already uses for the colour and albedo of worlds. A hull reads the same row. So how much radar comes back, how bright the ship glints in sunlight and how much sunlight it soaks up as heat are one lookup, and a coating is a new row on the sheet, not a stealth flag. "Absorbent to radar" and "mirror-bright to stay cool near a star" are the same figure pulled in opposite directions, which is a real trade.

**What paint can and cannot do** (Alex, 22:08: "painting a ship black would work for some sensors... that drive plume is a problem"). A skin only changes light that arrives from outside and bounces off: sunlight in the visible, a radar's pulse, a lidar's beam. It does nothing for light the ship makes itself, and the largest source of that is the exhaust. The plume is outside the hull, so no coating covers it, and its brightness comes straight off the power ledger: whatever the "ends up as" column of the Power card shows leaving as hot exhaust is what a watcher's infrared sensor has to work with. So a black ship coasting can be very hard to find, and the same ship burning is found by its plume at the range the sums give, whatever it is painted with. The counters to a plume are the ones that are already on the list and all of them cost travel: do not burn (run quiet), burn gently (a smaller, cooler plume and a longer trip), or burn with a world or a star between you and the watcher (hide in the glare, or simply out of sight). The switch row says so in a few words: with the drive lit, the skin's reading greys out for the bands the plume shows in.

**All three are parts a ship can carry today** (Alex, 21:49). Absorbent skin, shaped hull and cancelling sit in the base packs at the "built" mark, so they are on offer at the lowest rung of any tech ladder and the ship generator may fit them like any other part. They are not advances a setting has to unlock.

How it looks, kept simple:
- **On your own ship: a short row of switches**, each with its cost in a few words. Passive pack (always on), one per active sensor, Run quiet, then whichever counters the ship actually carries. A ship with none shows none.
- **On the chart, jamming is a hatched patch** over the area it spoils. Contacts inside it drop one mark (known to noticed, noticed to gone). The jammer shows as a bearing, not a point.
- **A decoy is just another dotted contact** until it is resolved, when it inks in as what it is.
- **A cloaked or quiet ship is simply not drawn**, and nothing leaks by absence. The GM's chart keeps it in pencil with a "Why?": "running quiet: gives off 300 W, they would need to be within 5,000 km".
- **The clock is the drama.** A heat sink filling, a jammer's power draw starving the engine, a decoy's battery: each is a number counting down that the GM and the dice can use. Danger is drama, never forbidden.
- **Programs use them too.** A pirate that runs quiet on a lane, a patrol that pings on its beat, a smuggler under false colours are pack programs choosing switches, with no new rule.

One knock-on: FT question 11 said every fictional advance must state its light. A cloak is that rule's main customer, and limit 10 already says a cloak "must say where the light goes".

### 17.6 Packages

| Package | What | Suggested owner | Waits on |
|---|---|---|---|
| RK13 | The Flow view (rows of from, to, amount, kind) and the Power card on it, with the Nowhere band and the five readings | chart room thread, with the renderer for the ship card | RK9 |
| RK14 | Default sensor fit per tech level; signatures from the ledger; the three rungs written as grants on events; the last-seen ring | light thread (`sse-sense`) and kernel thread (grants), on SV1 | RK9, RK11, SV1 |
| RK15 | The three marks on the chart, the Off, Simple, Full switch, "Why?" both ways, the passive pack and a switch per active sensor, the Run desk's contact list | chart room thread and the travel screens thread | RK14, SV2 |
| RK16 | Counters (section 17.7): run quiet, cold face, heat sink clock, jamming as added noise with its hatched patch, decoys, false colours through "Told instead", cloak as a fictional advance; the switch row on the ship | light thread (`sse-sense` noise and signatures), chart room thread (switches, patch), Traveller thread for its pack's ECM | RK14, RK15 |

## 18. Questions 18 to 25 (all answered yes, 23 Nov 22:08; question 21 so defaults to Simple)

18. **A general Flow view**, with the Power card as its first use, that a world's energy budget, an element's journey and a habitat's air can use later. Recommended: yes. Build the Power card first and let the others follow when their owners want them.
19. **Broken books drawn as a dashed band from "Nowhere"**, with one sentence saying how much. Recommended: yes; it is the clearest honesty mark we have.
20. **Fog of war is the secrets ladder climbed by physics**: three marks on the ordinary chart (ink, dotted, faded with a ring), no sensor screen. Recommended: yes.
21. **One campaign switch, Off, Simple, Full, defaulting to Simple** in any campaign that has ships, with a free default sensor fit per tech level. Recommended: yes. Say if you would rather it default to Off so nothing changes for a GM who never asked for it.
22. *Answered by Alex, 23 Nov 21:46:* passive sensors are a pack that is always on and shows everything in range; each active sensor is its own switch; ECM and cloaking are needed as counters. Written into 17.2 and 17.7.
23. **Counters are a short list of real moves** (run quiet, cold face, heat sink, hide in the glare, jam, dazzle, decoy, false colours) **plus cloak as a fictional advance**, each a switch or an order with its cost stated, and no operator game. Recommended: yes.
24. **Jamming drops contacts one mark inside a hatched patch and shows the jammer as a bearing**, so it hides what you are and not that you are there. Recommended: yes; it is what real jamming does and it is easy to read.
25. **A cloak must say where its light and heat go**, like every fictional advance, and "nowhere" is allowed but marked. Recommended: yes.
