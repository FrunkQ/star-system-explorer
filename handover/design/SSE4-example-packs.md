# SSE4 example packs: one on every shelf

Status: **ACCEPTED in full 23 Nov 2026** (questions 1 to 9 at 16:15, questions 10 to 13 at 16:36, with Alex's additions on where FTL may start and end, now in section 4A). Alex, 16:14: agreed on Star Trek and "do others similarly" (free, none of the owner's words such as LCARS, only on our own library, and he is willing to follow an original's look closely); and "the main thing to get right is the tech level (problems solved) for each universe/faction AND how it handles FTL in the rules". Alex, 16:15: "yes to all". Who builds what is in section 7A. Research and design only; nothing here is built by this note.

Purpose: every shelf on Explorers should hold at least one example, so people see what that kind of pack can be and fold it into their own thinking.

Licence notes below are practical guidance from the owners' published pages, read on 23 Nov 2026. They are not legal advice. Where a page could not be read, it says "not confirmed".

## 0. Scope ruling, 23 Nov 19:50

Alex: "hold that powder... in 4.2 we will go hard into RPG support. So all the game packs can wait a bit. Traveller is our showcase."

- **Traveller is the one game pack for 4.0.** Its standard ships and notice go ahead.
- **Waiting for 4.2, handed to nobody now:** the Mothership game pack and everything in section 4.3A (ship manifest as a play screen, crew roster, character sheet from pack data, table rolls, the failed-jump table), Cepheus Engine, the umpired 4X, and with them the Mothership licence email and the question of in-app dice.
- **Reading, not a ruling:** this covers packs that carry game rules. Setting, look and data packs with no game rules carry on in the order of section 7A: Sol today, World hooks, the small data packs, sky overlays, Pulp solar system, A working Sol, Megastructures, tours, and the FTL rule families in the voyage finder.
- **Settled by Alex, 23 Nov 19:52 and 19:53:** "Author all the looks, those are useful and let GMs use it in their universes properly", and "apart from that all the game specific stuff (except Traveller) is parked for 4.2". So Retro future and Bridge console carry on now as looks, with no game's name or rules. Eclipse Phase and 2300AD wait for 4.2 with the game packs (the coordinator's reading of "game specific", which Alex can correct).
- **Also from Alex, 19:52:** the green screen should be "any colour mono", and a select few of SSE3's player view shaders belong here. Section 4C.

## 1. The short answer

1. **Traveller with the full standard ship list: yes.** Mongoose's fair use policy allows free downloadable aids set in the official universe. Keep our own generated hulls, no deck plans, no logos.
2. **Star Trek: an original pack, free, on our library only, with none of the owner's words (ruled 23 Nov).** It is the one owner here with a record of shutting down free fan projects, so how closely the look follows is a choice to make knowingly (section 4B).
3. **Alien and Mothership: one original "retro future" looks pack, offered as Mothership-compatible.** Mothership has a third-party licence. The Alien RPG has none.
4. **The Expanse: build the living system, as an original setting on real Sol.** No fan policy was found, and the names are the risk. The capability (ships patrolling, mining, trading) needs none of them.
5. **Best named setting to do in full: Eclipse Phase.** It is a populated solar system under a Creative Commons licence that allows free derived work, names included.
6. **Second named setting: 2300AD**, under the same Mongoose policy as Traveller. It is built on the real nearby stars, which is exactly SSE4's ground.
7. Four original examples of our own fill the remaining shelves: Sol today (real spacecraft), Megastructures, Pulp solar system (fictional physics), and a calendars and units set.

## 2. Shelves and pack types today

What the Explorers site has (`src/lib/v4/read.ts`, live since 23 Nov):

| Shelf on the site | Pack kinds | In the repo now |
|---|---|---|
| Campaigns | campaign files | examples in the app only |
| Universe packs | `universe` | `near-future`, `space-opera`, `intrigue` |
| Game systems | `game-system` | `traveller` |
| Physics | `base-physics`, `physics` | 17 base packs; coloured white dwarfs to come (track ND) |
| Tag sets | tags and nothing that computes | `base-hooks`, `intrigue` reads as one |
| Looks and tours | `presentation`, `suite` | `sse3-classic`, `base-names` |

Designed but not on the site yet: Systems, Bodies and things (clips), a Names shelf (NM9), and the separate fan shelf (tags design 5.7).

Things a pack can carry that have **no shelf of their own**, because they ride inside another pack: calendars, unit schemes, ship and construct styles, ship role programs, HUD overlays, sensor and band recipes, view styles, name schemes, class atlases, tours, vocabulary. Question 6 is about how people find these.

## 3. Recommended set

"Have" means it exists in the repo. "Finish" means it exists and needs additions. "New" means to be written. Each line names the general capability it shows, not only the example.

### Campaigns

| Example | Shows | State |
|---|---|---|
| **A working Sol** (original names, on the `near-future` pack): three powers, stations, and ships on patrol, mining, trading and raiding by the six role programs | a living system: programs as seats, fuel and travel times from real physics, the clock running, a ship's log | New, late (needs voyages and role programs) |
| **Sol today**: real spacecraft and stations on their real orbits (the space station, the L2 telescopes, the Voyagers, Mars orbiters) | constructs on real trajectories; education; nothing fictional | New, cheap once constructs sit on the chart |

### Universe packs

| Example | Shows | State |
|---|---|---|
| Near future, Space opera | dressing rules, factions, a political lens | Have |
| **Megastructures**: Dyson swarm, O'Neill cylinder, Stanford torus, space elevator, orbital ring, stellar engine, each from its paper | a construct with attached physics blocks; cited engineering; shape from numbers | New, late stage 5 (design note already ruled) |
| **Eclipse Phase** (fan shelf) | a named setting done properly: habitats and factions placed on the real solar system, under its own licence | New |
| **2300AD** (fan shelf) | real nearby stars with colonies by nation; "renamed by many nations" name schemes | New, after naming packs |

### Game systems

| Example | Shows | State |
|---|---|---|
| **Traveller**, with the standard ships | world codes from physics, hex map, sensors ladder, calendar, jump as a counted measure, stock ships | Finish (section 4.1) |
| **Cepheus Engine** (optional) | the same code sheets under a second, open-licensed rule set, so "game system" is visibly not a Traveller special | New, small, low priority |
| **Mothership compatibility** (with the retro look below) | a game pack that is almost all presentation: stress, slow travel, cryosleep ageing by ship time | New |

### Physics

| Example | Shows | State |
|---|---|---|
| **Coloured white dwarfs** | a paper brought in as visible, cited blocks; "no new code" | Planned (ND) |
| **Pulp solar system**: swamp Venus, canal Mars, breathable everything, as 1930s fiction believed | "a different universe's laws"; the fictional flag and honesty marks; every changed value names the pack | New, small |
| **What if G drifts** | a constant with a time law, and the reach report | New, tiny once DM6 lands |

### Tag sets

| Example | Shows | State |
|---|---|---|
| Base hooks, Intrigue | labels read off a world; GM-only seeded hooks | Have |
| **World hooks** (original wording): one-line story prompts read from physics and history, such as "tidally locked, people live on the terminator" | tags as pure vocabulary that never feed physics | New, small |

### Looks and tours

| Example | Shows | State |
|---|---|---|
| SSE3 classic | a class atlas as a look | Have |
| **Retro future**: a one-colour terminal view style in any colour the GM picks (section 4C), a proximity-ping sensor recipe, chunky industrial ship style (tugs, haulers, refineries) | in-universe view styles as filters, style per panel, ship styles as pack data, sensor recipes | New, after view styles are built |
| **Bridge console** (original design, not LCARS): a player screen theme with a decimal date and a galactic quadrant overlay | themes, a calendar that is not days and months, a HUD overlay that is not a hex grid | New |
| **Tours**: "Grand tour of Sol" and "Life of a Sun-like star" | tours as data; deep time for teaching | New, with tours |
| **Sky overlays**: constellation lines and boundaries, galactic coordinate grid | overlays from real public data on any chart | New, small |

### Names (shelf to come, NM9)

| Example | Shows | State |
|---|---|---|
| Base names; the Oluen and Vesh alien schemes already drafted | the four kinds of naming | Have or drafted |
| **Survey designations** | catalogue-style names as a scheme | New, tiny |

### Calendars and units (inside a presentation pack, "This setting")

| Example | Shows | State |
|---|---|---|
| Traveller's calendar and dT | a setting calendar and unit | Have |
| **Darian calendar for Mars** | a real proposed calendar for another world; "make a calendar from this world" | New, tiny (free use not confirmed, see 6) |
| **Astronomer's units** (parsecs, solar masses, cgs) | a unit scheme for a profession, not a game | New, tiny |
| **Decimal date** (in Bridge console) | a calendar by formula | New, tiny |

### Ship role programs and overlays

Explorer, courier, miner, trader, patrol and pirate ship as the programs inside "A working Sol". The Traveller sector grid is the first overlay; sky overlays and quadrants are the second and third.

## 4. Alex's four ideas, assessed

### 4.1 Traveller with all the standard ships

- The pack has 21 templates, of which 4 are starships (courier, trader, far trader, patrol ship) plus shuttle and lifter.
- The classic list people expect is about 12 starships and 6 small craft: scout and courier 100 t, free trader 200 t, far trader 200 t, subsidised merchant 400 t, subsidised liner 600 t, yacht 200 t, safari ship 200 t, seeker (the mining ship) 100 t, laboratory ship 400 t, patrol cruiser 400 t, mercenary cruiser 800 t, corsair 400 t; launch, ship's boat, pinnace, cutter, shuttle, fighter.
- These map straight onto the role programs: seeker is the miner, corsair the pirate, patrol cruiser the patrol, traders the trader, scout the explorer and courier. So the standard ships are also the cast for a living Traveller subsector.
- **Licence.** Mongoose's licensing page says its Fair Use Policy allows posting "on websites or as downloadable modules so long as it is non-commercial" and covers "small game aids", for material in the official universe. Free and unofficial fits. The page does not speak about logos or ship art, so: tonnage, drive ratings and role names yes; hulls from our own generator; no deck plans, no publisher art, no TAS or Traveller logo, no copied text.
- **One thing to correct.** The pack's notice says Traveller is a trademark of Far Future Enterprises. The Traveller wiki's current notice reads "The Traveller game in all forms is owned by Mongoose Publishing." The notice should be updated to match Mongoose's policy wording (the 2025 policy PDF itself was not read; the Traveller thread should read it when it makes the change).

### 4.2 Star Trek

- What it would need: a console look, stardates, warp speeds as an FTL rule, quadrants, otherwise vanilla. All three are small and all three are general capabilities worth having.
- **Licence: the weakest position of any candidate.** The only published guidance is for fan films; it says "These guidelines are not a license" and covers nothing but films. Free, clearly unofficial fan projects have been stopped: Stage 9 (a free recreation of the Enterprise, cease and desist, 2018) and the Wolf 359 Project (a free fan book, DMCA strike, 2023). The LCARS look is the owner's distinctive design. No fan or third-party policy for the Star Trek Adventures game was found (the Modiphius forum page needs a login; not confirmed).
- **Ruled by Alex, 23 Nov.** Build the "Bridge console" pack: a console theme, a decimal date calendar, an FTL rule where speed is a formula of a drive rating, and a quadrant overlay. Free, on our library only, none of the owner's words. He is willing to follow the original look closely; section 4B says where that starts to carry risk.

### 4.3 Alien and Mothership, shared retro look

- What it would need: the CRT wireframe view style (already recorded as a planned in-universe look), a motion-ping sensor recipe, industrial ship style, slow travel with cryosleep (crews age by ship time, already ruled).
- **Mothership.** Tuesday Knight Games runs a third-party licence that hundreds of supplements use. One creator's account says approval was given by the publisher and a compatibility logo issued. The licence text itself was not readable from here (not confirmed: fee, exact terms). It is the friendliest named game on this list.
- **Alien RPG.** Free League's open licences cover the Year Zero Engine rules and five other games (Dragonbane, Forbidden Lands, Symbaroum, both Coriolis games). Alien is not among them; it is a film licence held from the studio. The Year Zero Engine licence forbids "any Free League brand names or other copyrighted material beyond the SRD".
- **Ruled by Alex, 23 Nov ("do others similarly").** One pack, "Retro future", in Looks and tours, free, on our library only, with no film or game owner's words in it. Apply for the Mothership licence (an email in Alex's name, drafted for him) and add "Compatible with Mothership" when granted.

### 4.3A The Mothership game pack, widened (Alex, 23 Nov 19:28) **[waits for 4.2, see section 0]**

Alex: "a ship manifest screen and a crew, and MS char gen is easy, so possible to run inside SSE, even down to crashed spaceship wreckage and entering a lava tube. I LOVE Mothership! We can even roll the jump encounter when they fail."

So Mothership becomes the example of a game that can be **run inside SSE4**, not only dressed for it. Split by who owns what:

**General, for every game pack (ours, no licence needed):**

| Piece | What it is | Built on |
|---|---|---|
| **Ship manifest** | A screen on any construct: its parts, tanks and stores as real amounts, cargo, berths, what is damaged | the ship's own numbers, kept values (stock), the Ships and stations screen |
| **Crew roster** | People as records on a construct: name, role, where they are, awake or asleep, age by ship time | the record's actors; crew limits from the species (future tech limits 4, 5, 9) |
| **Character sheet from pack data** | A pack declares the sheet's fields and how each is made (rolled, picked, derived). Traveller, Mothership or any game fills the same mechanism | code sheets and kept kinds; dice as keyed draws with the GM's table secret |
| **Roll on a table** | A pack table, a trigger, a keyed draw, the result as a record note through the GM gate | triggers and effects (track J) |
| **Failed FTL has consequences** | The "what can go wrong" field of the FTL rule names a table to roll on | the six FTL fields above |
| **Natural places you can go into** (4.0) | Caves and lava tubes, found from physics where the world has them (lava tubes follow from volcanic history and low gravity), entered at Site level. Wrecks and built interiors are parked to about 4.2, see below | surfaces from history, detail level Site; the interior sites design thread |

**Mothership's own (needs the licence answer before it ships):**

- its character generation: the stats, saves, classes and loadouts by name;
- Stress and Panic and their tables;
- its jump ratings and the jump encounter table;
- its ship terms, and the "Compatible with Mothership" line and logo.

Until the licence answer, the pack is built on the general pieces with placeholder tables of our own wording, so nothing waits.

**One tension for Alex to settle.** On 23 Nov he said the game layer could roll dice and map onto skill checks "but not just yet". Rolling character stats and a jump encounter in the app would be the first use of that. Recommended: allow it here in its smallest form, a table roll that writes a note, with no skill-check engine. The dice design is already ruled (computed on the GM machine, a GM-only secret), and a table roll needs nothing new from it. Skill checks during play stay "not yet". The coordinator is asking him.

**Scope, Alex 23 Nov 19:47:** authored wrecks and built sites you can go inside are parked until about 4.2, when a co-GM makes encounters, pirate skirmishes and wrecks from the living simulation ("this was a flippant one liner that has become a designed feature... maybe 4.2?"). Natural caves and lava tubes stay in 4.0, in the interior sites design thread. No build thread is handed wreck or built interiors now.

### 4.4 The Expanse as a fully working solar system

- This is the best demonstration on the list of "what a pack could be", and the capability is ours: real Sol, torch-ship travel times from physics, fuel, programs running patrol, mining, trade and piracy, a clock you can run.
- **Licence.** No fan or community policy was found for the novels, the show or the Green Ronin game. The game's own licence is narrow (its FAQ says "Our license does not allow us to sell STLs"). The setting's value is in its names (ships, factions, people), which is exactly the grey part our fan rules warn about.
- **Ruled by Alex, 23 Nov ("do others similarly").** Ship it as **"A working Sol"**, an original campaign on the existing near-future pack: Earth, Mars and Belt powers under our own names, our generated ships. Everything people would recognise is physics and geography, which nobody owns. It goes on the front Campaigns shelf, not the fan shelf. If Alex wants a named, populated solar system as well, Eclipse Phase is the one that is licensed for it.

## 4A. Tech level and FTL, pack by pack

Alex, 23 Nov: these are the two things each example must get right, so that SSE4 can be seen to accommodate them. Tech is written in the terms of `SSE4-future-tech-design.md` (proposed, with Alex): each setting has its own **ladder**, and its **advances** push the ten **limits**. "Not yet built" means allowed by physics; "fictional" means past a named ceiling. Limit numbers are that design's: 1 heat, 2 thrust, 3 energy, 4 acceleration, 5 weight, 6 light speed, 7 materials, 8 radiation, 9 air, food and time, 10 being seen.

The setting facts below are from my knowledge of the published games and stories, not re-read today. Whoever builds a pack checks each against the rulebook.

### The FTL rules these packs need

Seven families cover every candidate. The voyages design already has the shape for the first four (a rating on the ship, a need on the leg, time and fuel from the pack's rule, judged per system).

| Family | How it works | Used by |
|---|---|---|
| **None** | Real flight only; the search never leaves the system unless a ship can fly relativistic | Sol today, A working Sol, Megastructures, Eclipse Phase ships |
| **Jump by rating** | One jump of up to N parsecs; time fixed whatever the length; fuel per jump; must start well clear of a world or star | Traveller, Cepheus, Mothership |
| **Hop with a tether** | FTL at a speed set by the drive, but only so far before it must reach a star's gravity well to discharge, so routes are chains of close stars | 2300AD |
| **Speed by formula** | Continuous FTL; speed is a formula of a drive rating; no routes, go anywhere | Bridge console |
| **Rate per distance** | A rating in days per parsec; crews usually sleep | Retro future |
| **Gates** | Legs exist only between fixed gates, which are objects in a system, usually found and not built | Eclipse Phase; the later Expanse stories; Coriolis |
| **Fixed points** | FTL may start or end only at set places in a system (far above the star, a marked point) | many settings; a variation on any of the above |

Each rule also states six things, which are worth having as plain fields on every FTL rule: **where a leg may start and end**, **what the crew does** (awake, asleep, at risk), **how long it takes** (time at home and time aboard, stated apart from fuel; instant is allowed), **what it costs** (fuel), **what can go wrong** (for dice and the GM, never forbidden), and **how messages travel** (only by ship, FTL radio with a delay, or instant). Messages matter as much as ships: a setting where news moves at the speed of a courier plays very differently.

### How long a leg takes, as data

Alex, 23 Nov 16:37: "FTL may still take time, so that needs to be known too." Duration is stated on every FTL rule, apart from fuel, as one of four forms:

| Form | Data | Example |
|---|---|---|
| **Instant** | nothing | a gate you step through |
| **Fixed per leg** | a time, with an optional spread for dice | Traveller: about a week whatever the distance |
| **Per distance** | a time per parsec, read from the ship's rating | Retro future: so many days per parsec |
| **By formula** | a speed from a rating; time is distance over speed | Bridge console: the rating cubed times light speed |

Each rule gives **time at home** and **time aboard**. They are the same unless the pack says otherwise (a jump where a week passes outside and none inside, or the reverse). Both feed what is already ruled: the campaign clock runs by home time and crews age by ship time. The climb to the start place and the fall from the end place are real flight and add their own real time.

### Where a leg may start and end, as data

Alex, 23 Nov 16:36: some rules need a ship to be out of a gravity well, "L points become key as they are dynamically neutral", "sometimes you need a constructed jump gate", and "we want a simple flexible ruleset to cope with all". The limits should be shown on the chart.

So the first of the five fields is a short list of **conditions**, each one line of pack data, with one list for starting a leg and one for ending it (the same list unless the pack says otherwise). A place qualifies when it meets them.

| Condition | Data | What it expresses | Drawn on the chart |
|---|---|---|---|
| **Clear of a body by its size** | a multiple, of diameters or radii; which bodies count (all, or above a size) | Traveller's limit, a set multiple of the diameter of any world or star nearby (100 diameters as I recall it; the Traveller thread checks the rulebook) | a shell round every body that counts |
| **Gravity above or below a level** | an acceleration, and "below" or "above" | "out of the well" in general; and the reverse, a drive that must come **into** a well, as 2300AD's does to discharge | a contour round each body, from the gravity SSE4 already computes |
| **At a named point** | which points: L1 to L5 of a pair, above or below the star's poles, the system's edge | Lagrange points as natural gates, and fixed-point settings | a mark at each point, moving with the clock as the bodies move |
| **At a placed construct** | a template or tag (a gate), and optionally a linked partner | built or found jump gates; a leg exists only between linked gates | the construct's own symbol, with its links on the galaxy chart |
| **Anywhere** | nothing | settings with no limit | nothing |

Notes:

- **All computed, none typed per world.** A multiple of the Sun's diameter at 100 comes out near 0.93 AU, so most of the inner system sits inside the star's own limit and a ship leaving Mercury has a long climb first. That falls out of the star's size; nobody enters it. It also changes as a star swells with age, for free.
- **The climb is an ordinary in-system leg.** Getting from a world to the nearest place that qualifies is flown with real physics, as the voyages design already joins the two scales at the system's edge. This only makes "the edge" a computed place instead of one fixed distance.
- **Shown under HUD.** The limits are one overlay under the HUD switch, on every chart and style, named by the pack ("Jump limit"). With HUD off they vanish like every other line.
- **Danger is drama.** A pack may say what starting inside the limit risks instead of forbidding it; that is the fourth field, "what can go wrong".
- **Size or gravity?** They are two honest ways to say the same idea. A pack picks the one its rules use. Gravity is the more physical and handles close pairs of stars without a special case.

**Checked with the travel thread, 23 Nov:** both fit the voyage finder with no special case. The finder is one shortest-path search over places, and each FTL rule supplies a "legs from here" function (jump and flight today). Gates are a third such function (a leg exists only between two placed gate constructs); fixed points is the same function limited to marked places; speed by formula and rate per distance give time from distance and a rating; hop with a tether is jump by rating with a cap on the hop and a need to reach a star's well. The message rule is not a leg but a separate figure, "how long news takes from A to B": light delay, the fastest courier route (the same search run for a courier), an FTL radio's range and delay, or instant.

### Each pack

| Pack | Ladder | Advances and the limits they push | Honesty | FTL rule | Ratings shown | Messages |
|---|---|---|---|---|---|---|
| **Sol today** | the real rungs | chemical and ion drives, solar and fission power | all built | none | none | light speed |
| **A working Sol** | three levels, such as orbital, torch, deep | fusion torch (2, 3); closed habitats (9); hard couches and drugs for high g (4); spin and thrust for weight (5). No gravity plates, no screens, nothing hides (10) | not yet built; nothing fictional | none; an optional found gate later | exhaust speed and sustained g, in plain numbers | light speed, with delay shown |
| **Traveller** | its own TL numbers | fusion plants that shed little heat (1); gravitic thrust (2); gravity plates (5); compensators (4); cold sleep (9); jump (6) | five fictional chips, each naming its law | jump by rating: Jump-1 to Jump-6, about a week per jump, fuel as a share of the hull per parsec, start 100 diameters out; routes on the hex map | Jump-N, Manoeuvre-N in g, the pack's drive letters | by ship only; a courier network sets the speed of news |
| **2300AD** | one near-future level, by nation | fusion and fission power; no gravity control, so spin habitats (5); real heat and radiators (1) | mostly not yet built; one fictional (the drive) | hop with a tether: about 7.7 light years before discharge, so nations spread along "arms" of close stars. Uses real distances between real stars | drive efficiency (a speed) | by ship only |
| **Bridge console** | a long ladder, several powers on different rungs | matter and antimatter power (3, at the ceiling, so allowed); compensators (4); gravity plates (5); screens (8); heat sinks (1); making supplies from energy (9); formula FTL (6) | several fictional chips; antimatter is "not yet built" | speed by formula: the rating cubed times light speed is the classic scale, the formula is pack data | a factor, with cruise and top values | FTL radio with range and delay |
| **Retro future** | two or three levels, corporate | slow heavy drives (2, 3); sleeping crews (9); little else solved, so big radiators and tanks | not yet built, plus one fictional (the drive) | rate per distance, or jump by rating for Mothership; crews asleep and ageing by ship time | days per parsec, or Jump-N | by ship or slow relay |
| **Eclipse Phase** | one high level, uneven by faction | fusion and antimatter drives (2, 3); making almost anything from feedstock (9); bodies chosen for the job, so crew limits change with the body and a machine body needs no air (4, 5, 9) | not yet built for most; gates are found | ships: none. People travel as data at light speed between habitats. Gates: fixed, found, a handful in the system | none for ships; gate addresses | light speed; scarce instant links |
| **Megastructures** | one rising row of material strength | materials only (7), each structure from its paper | built, not yet built, then fictional by arithmetic | none | none | light speed |
| **Pulp solar system** | "atomic age" | atomic rockets richer than any real fuel (3); breathable worlds are the physics pack's doing, not tech | fictional, cheerfully | none | none | radio |

What this shows, across the set:

- **Factions on different rungs.** A working Sol (a rich inner power and a poorer belt), Bridge console (older and younger powers) and Traveller (worlds from TL 0 to 15) each need capability on the civilisation, inheriting down the tree, which the future tech design proposes.
- **Crew limits belong to the cargo.** Eclipse Phase is the sharpest example: a ship crewed by machine bodies comes out smaller with no tech entry at all.
- **"Found, not built"** is exactly what gates are.
- **Every FTL family at least once**, so the travel finder is shown to be general and not a Traveller special.

## 4B. How closely a look may follow an original

Alex is willing to follow an original's look closely as long as the pack is free, uses none of the owner's words, and lives only on our library. This is so he chooses knowingly. It is practical guidance, not legal advice.

What those three conditions do and do not do:

- **Free** removes the worst case (a claim about money made) but is not a defence. Stage 9 was free and said plainly it was unofficial, and was still told to stop.
- **Not using their words** removes the trademark problem. It does not touch copyright in the designs themselves: a traced screen is a copy of an artwork whether or not it carries the name.
- **Only on our library** keeps it out of app stores and other hosts that act on a complaint automatically. But a pack we write and feature is our own publication, so we cannot answer as a host that merely carries a user's upload. And once paid packs are switched on, the site as a whole is a shop, which weakens "non-commercial", even though the pack itself stays free and on the fan shelf.

The realistic worst case stays what the tags design says: a letter asking for it to come down, and it comes down. The cost is the work lost, and any trailer footage that showed it.

| Step | What it means | Risk | Use |
|---|---|---|---|
| **1. In the manner of** | The genre's cues, our own drawing: rounded bars and flat colour blocks on black; green phosphor text and scan lines; chunky industrial hulls | none worth naming | front shelf, trailer |
| **2. Close homage** | Recognised at a glance. The same layout grammar, but our own palette, proportions, typeface and shapes; nothing traced | low | what we write and feature |
| **3. Near copy** | The same palette, lettering style, corner shapes and panel layouts, matched to a screenshot | a real chance of a takedown letter, highest for Star Trek | only as a fan shelf item, never on the front shelf or in a trailer, and accepted as removable |
| **4. Never** | Their names and words, logos and insignia, exact ship shapes, sound effects, screenshots, their font files, copied text | this is what gets projects closed | not at all |

Three points that apply to every setting:

- **Ships.** Our hulls come from the generator, and a style only chooses form language. A recognisable silhouette (a saucer on a neck with two engine pods, or a particular hero ship) is a design someone owns. Styles should stop at the family resemblance.
- **Fonts and sounds.** A lookalike typeface must be one we are licensed to ship. No sampled sounds.
- **Finding it.** With none of the owner's words, the pack is found by its own name and by what it contains. The existing fan rule allows one plain line saying what a pack is for; question 11 asks whether to use it.

**Recommended:** step 2 for everything we write and feature. Step 3 is open to you for the fan shelf if you want it, most safely for Retro future (a period computer look belongs to nobody) and least safely for Bridge console.

## 4C. Looks from SSE3 worth carrying over

Alex, 23 Nov 19:52: the green screen "should be any colour mono", and "we had a load of VERY cool shaders in SSE3 to totally mess with player views... we don't need all of them but a select few make sense here".

What SSE3 has (read on AC-ProArt, `star-system-generator/src/lib/holo/filters`, `transitions` and `player`). I read the filter schema, the player presets and the plain-CSS twin of the filters. The shader files themselves sit too deep in the folder tree to copy over, so the notes on Night vision and Thermal come from their settings, and the transitions I did not open are judged by name.

- **Three filters:** CRT (phosphor colour, tint, scan lines, vignette, barrel distortion, tearing, a rolling noise bar, skew, rounded corners, invert), Night vision, Thermal.
- **Ten transitions between screens:** fade, wipe, iris, CRT collapse, scanline, static dissolve, typewriter, written, written reveal, blood spatter.
- **A plain-CSS twin of each filter** for parts of the screen that are not drawn by the graphics card.

"Don't copy V3, do it better" applies. In SSE4 a look is a filter chain on a view style (`sse.view-style/1`), so SSE3's one big CRT shader becomes small pieces that stack, each usable alone.

**Untangled, 23 Nov 19:57.** Alex: "you have mixed up transitions, with effects and more there, so make sure you don't mess yourself up." He was right: the first list put several different kinds of thing in one table, and then claimed one dial covered them all. It does not. The kinds below are separate, each with what it applies to and how it is controlled.

### Kind 1: Filters (how a view looks, lasting)

Applies to a view or a single panel, as the filter chain of its view style. Controlled by settings the GM tunes once and saves in a look. Nothing moves at the table.

| Filter | From SSE3 | What it does | Recommend |
|---|---|---|---|
| **Mono, any colour** | the CRT's phosphor colour and tint | One colour the GM picks: green, amber, white, red for alert, blue for a hologram (Alex: "any colour mono") | yes, first |
| **Tube** | the CRT's glass | Scan lines, vignette, curved glass, rounded corners | yes |
| **Grain** | part of Night vision | Sensor speckle | yes, small |

### Kind 2: Palettes (how a reading is coloured)

Applies to a band's reading, inside its recipe. Not a filter: it changes which colour a real number gets, not the picture afterwards.

| Palette | From SSE3 | What it does | Recommend |
|---|---|---|---|
| **False colour** | replaces Thermal | A colour ramp over any real band (Infrared, Mag, X-ray). SSE4 has a real Infrared band, so a fake heat filter would break "never fake it" | yes |

### Kind 3: Signal faults (what the GM does to a feed, live)

Applies to a seat's whole screen, one seat or all. Controlled by **one live dial** in the GM's hand. This is the kind Alex shaped on 23 Nov (19:55 and 19:56).

| Fault | From SSE3 | What it does |
|---|---|---|
| **Analogue** | the CRT's tearing, noise bar and skew | Tearing, a rolling bar, snow, ghosting; at the far end, nothing but static |
| **Digital** | new | Blocks, frozen patches, colour smears, dropped frames; at the far end, blocks and black |

- A look says which kind it uses and its **saved level**. The dial runs from clear, through the saved level, to the far end.
- "A dial that just clears it up from the preset it started at": one drag down cleans the picture.
- "Tune in": the GM starts at the far end and draws the dial down until the picture comes through.
- Pushing past the saved level toward the far end is the coordinator's working choice, which Alex can reverse.
- Later a pack may move the dial from something true (readiness, distance, a storm). The GM's hand wins.

### Kind 4: Screen delivery (how a screen arrives)

Alex, 23 Nov 19:59: "screen delivery rather than text delivery; a transition from nothing is always an option." So the old kinds 4 and 5 are one kind. Delivery is how any screen arrives, words and pictures alike, and it can start from the screen before or from nothing.

Applies to the moment a seat gets a screen: a new view, a new document, a GM push, or a screen coming on. Controlled by two choices saved in the look, **arrives by** and **from** (the screen before, or nothing), and a duration. It plays once and is over. It is not a dial.

| Arrives by | From SSE3 | What it does | Recommend |
|---|---|---|---|
| **Fade** | fade | The quiet default | yes |
| **Scene wipes** | wipe and iris, widened | One delivery with its shape as data: straight in four directions, diagonal, clock sweep, iris in and out, hard or soft edge. Old film grammar that belongs to nobody, named plainly (Alex, 19:56) | yes |
| **Power on** (and off) | CRT collapse | The picture grows from a dot and a line; off is the reverse, to nothing | yes |
| **Static burst** | static dissolve | The screen forms out of static. It borrows the fault's static for a second; it is still a delivery | yes |
| **Typewriter** | typewriter | The screen is typed out, character by character. Works on any screen, best on documents | yes |
| **Scan in** | scanline | The screen is drawn line by line from the top, as a slow terminal or a plotter would. Back in, now that it has a place | yes |
| Written, written reveal | written, written reveal | Handwriting appearing; needs a journal document style SSE4 does not have yet | later |
| Blood spatter | blood spatter | Horror game flavour | 4.2, with the game packs |

Two sorts sit in this one kind, and the difference is only what they do with the screen before. Fade, wipes and Power replace one screen with the next. Typewriter, Scan in and Static burst build the new screen up and do not need an old one. Any of them may start from nothing.

**A seat's screen can be off.** That is a state of the seat, not a delivery: the GM turns a screen off (Power off plays) and on again (the look's delivery plays, from nothing).

### Looks: saved combinations of the kinds above

A look is what a GM picks and a pack ships. It names a view style, its filters and their settings, a palette where it matters, a fault kind and saved level, and a screen delivery.

- **Green terminal** (or amber, or any colour): Mono, Tube, analogue fault at a low saved level, arrives by Power on (or Typewriter for documents).
- **Night vision**: Mono in green, Grain, a hard vignette. No shader of its own.
- **Hologram**: Mono in blue, digital fault at a low saved level, arrives by Static burst.
- **Space opera**: no filter, arrives by Scene wipes.

Retro future ships the first of these with the ping sensor recipe and the industrial ship style. Owner: the renderer thread, with the view styles work.

### Does "one shared amount for everything" hold? No.

I wrote earlier that one amount, played or held, covered faults and every transition. Separated out, that was wrong:

- **Filters** have many settings each and no single amount.
- **Palettes** are a mapping, with nothing to dial.
- **Signal faults** are the one kind with a live dial. Static and tuning in belong here.
- **Screen delivery** has progress from start to end, but that is a clock, not a control. Nobody needs to hold a wipe half way. If the GM wants a screen dark, that is the seat's "screen off" state.

So there are four small mechanisms (filters, palettes, signal faults, screen delivery), each simple, and one dial. The earlier lines about holding a fade or scrubbing a wipe are withdrawn.

## 4D. Looks on the GM's screen

Alex, 23 Nov 19:58: "how the UI is put on the GM's screen to manipulate these at edit time and uses them at run/explore time (do they get a preview window option?)". Mockup, live: https://claude.ai/artifact/MUWHuo5mwY5fTXED7fvaSP ("SSE4 Looks controls mockup"). It follows `interface-style.md`: no new panel family, things appear when they are relevant, and the GM's own chart stays clean.

### Edit time: Workshop › Looks

Looks join the Workshop entries that already exist (Player screens, Styles). No new door.

- **One card, four rows, always in the same order:** Filters, Palette, Signal fault, Screen delivery. Each row reads as one line of words when closed ("Mono, amber · Tube") and opens in place for its settings.
- **The stage beside the card is the preview.** It shows the place the GM was looking at, through the look, so nothing has to be imagined. "Play delivery" plays the arrival. The signal fault row carries the kind and the saved level.
- **"Preview as: Desktop, Phone"** shows the cheap twin a weak device would draw.
- **Start from** any existing look; **Save** to the campaign or into a pack.

### Using a look yourself: Explore, Build, or the GM's own view

Saved looks appear in the **View style** switch after the four built-in styles (Chart, 3D, True, Wireframe), past a divider. This is what the view styles design already says: an in-universe look is a filter chain over a base style, chosen like any style, per device and later per panel. No dial here. A "Tune" link on a look opens it in Workshop.

### Run time: the Seats panel on the Run desk

Each seat already has a row. The row gains three small things, and nothing else moves:

- **Look ▾**: which look this seat's screen wears.
- **Signal dial**: a short slider from clear to lost, with a tick at the look's saved level. It is only there when the seat's look has a signal fault. Drag down to clear the picture up, start at lost and draw down to let them tune in.
- **Send**, which pushes the GM's current view to that seat, arriving by the look's delivery.
- **No Preview button** (Alex, 23 Nov 20:05: "clicking on the box to activate it should just do that"). Pressing a seat's row, or touching its dial, makes it the active seat and its screen shows in the preview window.

Above the rows, for all seats at once: **Send this view** and **Screens off / on**.

### The preview window: yes

- **The active seat's screen shows in a small window pinned at the corner of the GM's stage** (Alex, 20:05: "genius"): : its knowledge, its look, its fault, its deliveries as they play. One seat at a time: whichever row was pressed last. The window can be closed, and pressing a row brings it back.
- It is drawn on the GM's machine from what the seat has been sent, so it shows what the seat **should** be showing. It is not a camera on the player's device, and the caption says so.
- For a full-size look, the desk's existing **See as** menu ("as players see it") gains the seats by name.
- The preview draws the cheap twin when the GM's machine is busy, and only redraws when something moves.
- On a phone the Seats panel is a sheet, the dial is full width, and Preview opens full screen.

### Care for the people watching

- Signal faults never flash the whole screen faster than three times a second.
- A player's reduced-motion setting turns faults into a still picture and deliveries into a plain fade, whatever the GM does. The player's device always wins on this.

### Questions for Alex (4D)

**All seven accepted as recommended, and the looks short list of section 4C with them (Alex, 23 Nov 20:08: "All good here - your recs resonate - so solid").** That includes question 3: a look the GM gives a seat holds by default, with a per-seat "player may change" switch, and the player's accessibility settings always win. Pushing the signal dial past the saved level stands as recommended.

1. **Build looks in Workshop › Looks as one card with four rows (Filters, Palette, Signal fault, Screen delivery), with the stage as the live preview?** Recommended: yes.
2. **Saved looks appear in the View style switch after the built-in four, for anyone?** Recommended: yes. It is the mechanism already designed.
3. **A look the GM gives a seat holds for that seat unless the GM unlocks it?** Your earlier ruling is that view style is each user's own choice. Recommended: keep that for the plain styles, and let a GM-given look hold by default so the table shares one fiction, with a per-seat "player may change" switch. Accessibility settings on the player's device always win.
4. **The signal dial lives on each seat's row, appears only when that seat's look has a fault, and there are "Send this view" and "Screens off" for all seats?** Recommended: yes.
5. **A small pinned preview of one seat at the corner of the GM's stage, plus the seats by name in "See as"?** Recommended: yes.
6. **Fold Typewriter and Scan in into Screen delivery, so there are four kinds, and "from nothing" is a choice on every delivery?** Recommended: yes; this is your correction as I understand it.
7. **The flashing limit and reduced-motion rule above?** Recommended: yes.

Owners, now accepted: the renderer thread (filters, faults, deliveries, the preview's drawing), the chart room thread (Workshop › Looks, the Seats rows, the pinned preview).

Three rules to keep from SSE3, because they were right:

- **Every filter and fault has a cheap twin.** A phone on the older graphics path, or a text panel, gets the plain version (colour, lines, vignette, snow, blocks) and skips what it cannot afford (curved glass, tearing). The style's "needs" and "cost" fields already say which.
- **Settings are pack data with their own controls**, so a GM tunes a look and saves it, and a pack ships tuned ones. SSE3's fully tuned green terminal is the model.
- **Anything that moves says so**, so a still view is not redrawn sixty times a second for nothing.

One thing **not** to carry over: SSE3's built-in presets include a company logo on the "Datapad" cover (the asset is named `builtin-wy-logo`) and a cover reading "DON'T PANIC!" from "Megadodo Publications". Those are other people's marks and words, which is step 4 of section 4B. SSE4's shipped looks use our own.

## 5. Other candidates

| Candidate | Fit with SSE4 | Licence position | Call |
|---|---|---|---|
| **Eclipse Phase** | Populated solar system, habitats everywhere, hard-ish science | CC BY-NC-SA 4.0 on books, text and site. Credit Posthuman Studios, link to eclipsephase.com, free only, and the pack carries the same licence (not Apache). The logo is a trademark, allowed on work that follows the licence. The page does not split setting from rules. | **Do it**, fan shelf |
| **2300AD** | Real stars within about 50 light years, colonies by nation, no jump drive magic beyond a range limit | Named in Mongoose's Fair Use Policy alongside Traveller | **Do it**, fan shelf, after naming packs |
| **Cepheus Engine** | Traveller-compatible rules | Open Game Licence; all SRD text is open content except the names "Cepheus Engine", "Samardan Press" and product titles; must not imply a link to Mongoose. Ship chapter not confirmed. | Optional, small |
| **Mothership** | Slow, dangerous, industrial space | Third-party licence, approval and logo (terms not confirmed) | With Retro future |
| **Coriolis** (either game) | Space opera with a station at its heart | Free League open licence: supplements only, core book required, no new rule sets | Later, if someone wants it |
| **Starfinder** | Science fantasy; weak fit | Paizo's fan policy allows names in free tools but not together with game mechanics | Skip |
| **Stars Without Number** | Sector generator with world tags; close to our tags | Licence not confirmed | Skip the name; write our own World hooks |
| **Orion's Arm** | Megastructures, deep future | No open licence; outside use needs written consent. "Generic transhuman or science fiction concepts" are not claimed. | Skip the setting; Megastructures covers the ideas |
| **Elite Dangerous** | Whole-galaxy game | Terms ask for written permission; no staff answer found | Skip |
| **Star Wars, Warhammer 40,000** | Poor fit and strict owners | Not checked in detail | Skip |
| **"On the Expansion of Space"** (the umpired 4X assessed 19 Nov) | A game pack that is not a role-playing game | Needs its author's agreement | Later; it would be the non-RPG game example |

## 6. What is safe, plainly

- **Safe to write and feature ourselves:** everything original (A working Sol, Sol today, Megastructures, Pulp solar system, World hooks, Retro future, Bridge console, tours, overlays, calendars, units, names). Real spacecraft positions and constellation data are public facts; each source gets a citation when built. The Darian calendar's free-use terms are not confirmed and need a look before it ships.
- **Safe as free fan work with the right notice:** Traveller and 2300AD (Mongoose policy), Eclipse Phase (Creative Commons, with share-alike), Cepheus Engine (Open Game Licence), Mothership (once the licence is granted).
- **Not safe to write or feature ourselves:** Star Trek, Alien, The Expanse by name. A user may upload a fan pack under the four fan points and the takedown route; that is their statement, not ours.
- Standing rules still apply to every named pack: free always, no logos, no art, no copied text, plain descriptive names, the unofficial line, and its own shelf apart from anything with a price.

## 7. Questions for Alex

**Questions 1 to 9: all accepted as recommended (Alex, 23 Nov 16:15, "yes to all"). Questions 10 to 13: accepted (Alex, 23 Nov 16:36, "those 10-13 make sense").**

1. **Complete the Traveller standard ships (about 12 starships and 6 small craft, our own hulls, no deck plans), and update the pack's notice to Mongoose's wording?** Recommended: yes. Goes to the Traveller thread.
2. **Star Trek: build the original "Bridge console" pack (theme, decimal date, formula FTL, quadrant overlay) and publish nothing named Star Trek or styled as LCARS ourselves?** Recommended: yes.
3. **Alien and Mothership: one original "Retro future" pack, apply for the Mothership licence, never name Alien?** Recommended: yes. Applying means an email to Tuesday Knight Games from you or in your name.
4. **The Expanse: ship the living system as the original campaign "A working Sol" on the front shelf, with no Expanse names?** Recommended: yes.
5. **Write Eclipse Phase and 2300AD as our two named-setting fan examples?** Recommended: yes to both, Eclipse Phase first. Note the Eclipse Phase pack must carry CC BY-NC-SA, not Apache.
6. **Finding the small things: no new shelves for calendars, units, ship styles, programs, overlays and sensors; instead a "contains" filter on every shelf, read from the manifest?** Recommended: yes. One mechanism, and a pack that carries three things is found three ways. The Names shelf already planned can become the first such filter or stay a shelf; I would keep it a shelf because names packs are often tiny and stand alone.
7. **Add the four originals: Sol today, Megastructures, Pulp solar system, World hooks?** Recommended: yes to all four. Sol today and World hooks are cheap and early; Megastructures waits for its design note; Pulp is the clearest possible example of the fictional mark.
8. **Skip Cepheus Engine for now?** Recommended: yes, skip until someone asks. It proves a point the code already proves.
9. **Order of building: Traveller ships, World hooks and Sol today first (stage 2 and 3 material), Retro future and Bridge console with view styles, A working Sol and Eclipse Phase with voyages and role programs (stage 5), Megastructures last?** Recommended: yes.

10. **How close should the looks come: step 2 of section 4B (close homage, nothing traced) for everything we write and feature?** Recommended: yes. If you want step 3 anywhere, say which pack; I would allow it only for Retro future.
11. **May a listing carry one plain line naming what it suits ("made for games in the style of ...")?** Recommended: yes for Mothership, Traveller, 2300AD and Eclipse Phase, where a licence or policy covers it; no for Star Trek, Alien and The Expanse, where the pack is found by its own name and contents.
12. **Give every FTL rule the same six plain fields (where a leg may start and end, what the crew does, what it costs, what can go wrong, how messages travel), and ask the travel thread to confirm that gates and the message rule fit the voyage finder?** Recommended: yes. It is the one mechanism behind all seven families.
13. **Each example pack states its ladder, advances and FTL rule in the future tech design's terms, as section 4A does, and that table becomes the brief for whoever builds it?** Recommended: yes, once the future tech design's own questions are answered.

## 7A. Who builds what

In the accepted order (question 9). Each pack is its own work package in the owner's track.

| Order | Pack or piece | Owner thread | Waits on |
|---|---|---|---|
| 1 | Traveller standard ships (about 12 starships, 6 small craft) and the notice in Mongoose's wording (read the 2025 policy PDF first) | Build Traveller pack | nothing |
| 1 | World hooks tag set | Build Traveller pack (it wrote base hooks and intrigue) | nothing |
| 1 | Sol today: real spacecraft and stations on real orbits, each source cited | Build galaxy charts and travel screens | constructs on the chart |
| 1 | "Contains" filter on every shelf, read from the manifest | Integrate Explorers site with SSE4 | nothing; no database change expected, and if one is needed it takes Alex's own yes |
| 2 | Small data packs: Darian calendar (check its terms), astronomer's units, decimal date, survey designations names | Build the master matter data sheet (calendars, units, naming packs) | nothing |
| 2 | Sky overlays (constellations, galactic grid) and the quadrant overlay | Build photoreal renderer (owns the overlay slot) | nothing |
| 2 | Pulp solar system; What if G drifts | Build kernel resolve and rules | DM6 for the second |
| 3 | Retro future look: any colour mono and tube view styles (section 4C), ping sensor recipe, Retro industrial ship style | Build photoreal renderer (view styles, ship generator) | view styles; future tech question 14 |
| 3 | Bridge console look: console theme at step 2 of section 4B | Build photoreal renderer for the style; the chart room thread for the player screen theme | view styles; question 10 |
| 3 | Formula FTL and rate-per-distance FTL rules; the five fields; gates and messages | Build galaxy charts and travel screens | question 12 |
| waits for 4.2 | General game pieces (section 4.3A): ship manifest, crew roster, character sheet from pack data, roll on a table | Build Traveller pack (manifest, roster, sheet); Build kernel resolve and rules (table rolls through triggers) | Alex's answer on in-app rolling |
| waits for 4.2 | Mothership game pack on those pieces, with our own placeholder tables until licensed | Build Traveller pack | the general pieces; the licence answer for Mothership's own names and tables |
| 4 | Natural caves and lava tubes | the interior sites design thread, then the surface side | its design note |
| parked, about 4.2 | Wrecks and built sites you can go inside; a co-GM generating encounters and wrecks from the living simulation | nobody now | Alex, 23 Nov 19:47 |
| 4 | A working Sol: campaign, three powers on different rungs, the six role programs running | Build galaxy charts and travel screens, with templates from the Traveller pack thread | voyages, role programs, future tech accepted |
| waits for 4.2 | Eclipse Phase fan pack (CC BY-NC-SA, credit and link) | Build Traveller pack (universe packs) | gates; the fan shelf on the site |
| waits for 4.2 | 2300AD fan pack | Build Traveller pack | naming packs; hop-with-a-tether rule |
| 4 | Tours: Grand tour of Sol, Life of a Sun-like star | the chart room thread | tours |
| 5 | Megastructures | design note first, as already ruled (Traveller and kernel threads) | future tech accepted |
| skipped | Cepheus Engine | nobody, until someone asks | |

## 8. Sources read (23 Nov 2026)

- Mongoose licensing: https://mongoosepublishing.com/pages/traveller-licensing
- Traveller wiki notice: https://wiki.travellerrpg.com/Traveller:General_disclaimer
- Far Future fair use FAQ (historical, 2001): https://www.travellerrpg.com/pages/ffefaq
- Star Trek fan film guidelines: https://www.startrek.com/fan-films
- Stage 9 shut down, 2018: https://tech.slashdot.org/story/18/09/27/1713206/cbs-shuts-down-stage-9-a-fan-made-recreation-of-the-uss-enterprise
- Wolf 359 Project strike, 2023: https://entertainment.slashdot.org/story/23/08/31/057243/paramount-dmcas-star-trek-fan-project
- Free League licences: https://freeleaguepublishing.com/community-content/free-tabletop-licenses/ and https://www.gamespress.com/Free-League-Expands-Open-Game-License-Across-Five-RPGs-and-Multiple-La
- Mothership licence, second-hand: https://meatcastlegameware.itch.io/voidtouched/devlog/339424/officially-licensed and https://thealexandrian.net/wordpress/51884/roleplaying-games/review-mothership-adventure-sphere
- Expanse game FAQ: https://the-expanse-transport-union.backerkit.com/faq
- Eclipse Phase licence: https://eclipsephase.github.io/en/license.html and https://eclipsephase.com/faq/
- Cepheus Engine SRD: https://github.com/orffen/cepheus-srd
- Paizo fan content: https://paizo.com/licenses/fancontent/faq
- Orion's Arm terms: https://orionsarm.com/page/554
- Frontier forum thread: https://forums.frontier.co.uk/threads/anyone-know-if-i-can-make-a-fan-site-and-what-content-i-can-use.537849
