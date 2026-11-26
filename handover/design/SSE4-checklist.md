# SSE4 checklist: where we are

Phone copy: https://claude.ai/artifact/Xci8FXvxcfJkLighBrzS67

Read from the repo on 23 Nov 2026 at about 17:30 UTC: `handover/START-HERE.md`, every file in `handover/status/`, the design notes, merged and open pull requests, and project memory. No dates anywhere, on purpose.

Marks: ✅ done and on main · 🔶 started, part done · ⬜ not started · ⏸ parked on purpose

---

## Next milestone: the landing sequence, end to end

Alex, 23 Nov: "landing sequence and end to end run through for next week's level. The GM plotting the players' experience of the events." One walk from the GM's plan to what the players see: the GM plots a landing and its events, and the run-through plays it to the players' screens. The chart room thread is writing the gap list for it.

---

## 0. What needs you now

Short answer: nothing is blocked on you today. These are waiting when you have a moment, each with my recommendation.

1. **Atomic Rockets plan** (`SSE4-atomic-rockets-plan.md`, still proposed): 11 questions in its section 10. Recommend: yes to all as written (build from primary sources whatever the author says, leave weapons out for now, send a short Mastodon hello first).
2. **Mothership licence email**: drafted, yours to send. Recommend: send it; the retro look pack is built either way, only the game-side compatibility waits.
3. **Cosmic web void fraction**: keep the method at 8% voids and log it as a fudge, or raise the threshold toward the measured 77%. Recommend: raise the threshold, since the measured figure is the honest target.
4. **Old reviews still marked open in the status files** (none blocks anything): the K0 source-reading documents, T2 "age the Solar System", the class thresholds and the hycean class (C2), the System Lab licence line (F2: your own GPL code ported under Apache-2.0 with credit), and the environment classifier question in DM3. Recommend: tell me "close them as built" unless you want to look.
5. **Cloudflare**: START-HERE still says to switch the production branch to `main`. Every merge does deploy, so this is very likely done and the line is stale. Recommend: say "done" and the line goes.
6. **Part 2 below**: decide when each wanted item joins the plan.

Under your ruling that 4.1 life carries culture and technology (lighter touch in 4.0), three packages could go either way. **Answered yes to all three (Alex, 23 Nov 22:15: "we have the tools now - the GM can use them as they need and later they get computer help to improve it"):**

7. **Borders grown over travel cost on a world's surface** (the later half of OV11: countries spreading from capitals across the surface map). Ruled: move it to 4.1 with grown histories; hand-drawn outlines on the globe stay in 4.0.
8. **Life's capability graph** (built, L1b): it already dates when a world's life could reach tool use and language. Ruled: keep it as it is, a dated label, with no more work on it in 4.0.
9. **Role programs** (pirate, patrol, trader, miner, courier, explorer, J6 part 2): ships acting on their own by a pack program. Ruled: keep in 4.0, because they are ship automation a GM sets up, not a civilisation growing; their goals stay hand-set.

---

## 1. The six stages: what is done, what is left

The stages order the **joining up** of work into the app. Threads keep building underneath for later stages.

### Stage 1. Test drive ✅

**You can today** (https://v4.starsystemx.com): open the Solar System or grow a system, pick a world and ask why, change Mars's orbit and watch its surface warm, add a world with Add here, open Earth as a globe, its magnetic field and its cutaway, press Play and watch days pass, go out to the cosmic web, come straight back down, and look up iron. The walk runs as a check on desktop and phone for every pull request, counting presses.

Also done for this stage: colour-blind check on every theme, the Help door with What's new, Your data and recovery when a campaign will not open, one Open door for any file, Find in Look up.

**Rough edges left (13, listed in `handover/test-drive.md`).** The ones you would notice:
- ⬜ The Solar System's Earth has no greenhouse: it reads −18 °C, bare ground, no liquid. This is the most visible wrong thing in the app.
- ⬜ Only five Solar System worlds have a cutaway.
- ⬜ Going up from a system opens the last galaxy you charted, not the system's own (systems are not yet placed in galaxies).
- ⬜ "Why?" on a catalogued value does not name the catalogue.
- ⬜ On a phone: the place name is cut short, view controls take two rows.

### Stage 2. One system told in full 🔶

**You can today:** watch a grown system form (replay of its nine stages), read Constants, Units, Elements (periodic table by origin) and Names in Reference, see Mag on a world and every world's magnetosphere at system scale, use the place line at every scale, read a world's climate Why and scrub it through time.

**Still to do:**
- ⬜ Formation story screen: one chart of distance against time with cards (only the replay exists). The kernel's "branch a formation run" package alongside it.
- ⬜ Reference finished: Substances and the phase diagram, Calendars, editing.
- ⬜ Field ledger and field history (MF2, MF3): a world's field switching on and dying on the time ruler.
- ⬜ Ice zones in the cutaway (MP19, MP19b).
- ✅ The Life screen reading the chart's own worlds (L3, 26 Nov: today's state held through the history).
- ⬜ V3 gaps for this stage: where things can orbit (Hill spheres, L points), grow with a star you choose and four dials, pin a value or class with a reason, change a unit by clicking it, next eclipse, one mix editor for any gas mix, the eye and GM notes on every card, a "Your systems" map.
- ⬜ The campaign's "now" kept in the intent log.

### Stage 3. One language underneath 🔶

**You can today:** open a flow and see 71 physics blocks that are now written as named formulas in packs (results bit for bit the same), and see a pack's own sources, fudges and reproduction checks before you add it.

**Still to do:**
- 🔶 Typed copies of constants: 12 left, all in the old atom blocks; they go as the last atoms move.
- 🔶 Atoms into Equations: 71 moved, 14 stay in Rust with stated reasons.
- ⬜ Light through gas (ND2 with O4): a star's or a planet's colour from its gases, a spectrum on a wire. This also brings gas giants in every colour (V3 gap 36) and worlds coloured by their own daylight (gap 9).
- ⬜ Atmosphere facts on any body, white dwarfs included (ND1, inside B3).
- ⬜ The flow editor adds and wires blocks (today it only changes values).
- ⬜ Screens move off the stubbed light onto the kernel's.

### Stage 4. The galaxy has a past 🔶

**You can today:** chart a galaxy and see what other galaxies brought it and the marks each collision left; galaxies nearby; the cosmic web with time scrub; dark matter and dark energy on the data sheet as theoretical; the rotation curve from a dark matter profile; satellites on real orbits; the matter ledger of a cell.

**Open now:** GH4 streams and shells (PR 285); the galaxy in 3D and True (PR 290).

**Still to do:**
- ⬜ GH5 to GH8: globular clusters, the Milky Way's pinned history, the galaxy chart to its mockup (the mockup itself is still to come from a design thread).
- ⬜ The rest of the night sky: Y4 part 3, Y6 to Y9 (Y8 is the optional real-sky pack, parked).
- ⬜ Systems placed in their galaxy, and the voyage join at a system's edge.
- ⬜ Stars that shed shells on the chart (gap 19 part 2).

### Stage 5. Run a game 🔶

**You can today:** import a subsector onto a hex map with Travellermap, read world codes, put ships and stations in a system from templates, plan voyages between and inside systems with fuel and refuelling, see whether a ship can aerobrake or land at a world, see a ship designed from its parts in plan and 3D, hand players seats with their own knowledge, brand and join words, show a seat inside a virtual tabletop, ask Minerva to describe a world, read a ship's log, stack two original universe packs, overlay the Traveller sector grid, and fetch a pack from Explorers by link (hash checked).

**Open now:** Traveller standard ships and World hooks (PR 289); base-tech pack (PR 286).

**Still to do:**
- ⬜ Programs part 2: autopilot roles that run (explorer, courier, miner, trader, patrol, pirate).
- ⬜ The chart room on the kernel's intent registry (J5).
- ⬜ Tags: V3 tag import (TG6), real resources you can mine out (TG11), ghost markers and the rest (TG10, TG14, TG15).
- ⬜ Explorers: publishing a V4 pack end to end (needs your yes, it touches storage).
- ⬜ Territory overlays OV3 to OV13 (politics and other "who holds what" maps at every scale). OV1 and OV2 done.
- ⬜ Counted jumps and "fewest jumps" (MP16), the player's voyage requests.
- ⬜ Report screens (GM briefing, player survey sheet, world dossier), your own pictures with credit, credit for the maker.
- ⬜ Ship styles per nation or culture.
- ⬜ Future tech FT2 to FT8 and the example packs (see part 2: accepted, not yet in the build plan).

### Stage 6. Show it 🔶

**You can today:** turn the system chart to True (worlds in their own surfaces, a living star turning on its spin), and look through X-ray, infrared and Mag bands there.

**Still to do:**
- ⬜ Orac takes a paper, adds and wires blocks, saves it as a pack; the rehearsal and the film (ND4, ND5, ND7 to ND10).
- ⏸ View styles beyond True (Wireframe, in-universe looks such as a green CRT): designed, held by you.
- ⏸ Tours: wait until the screens stop moving.
- ⬜ Help text at three depths for every block (X4).
- ⬜ Stand here on a world (gap 8), the class gallery (gap 35), uploaded 3D models (gap 14, later).
- 🔶 Trailer clips: gathered daily as features finish.

---

## 1A. Track by track (what is underneath)

| Track | Done | Next |
|---|---|---|
| Kernel foundations (hashes, record, flows, packs, API) | ✅ all of K0 to K10 | |
| Kernel rules: classes, rules, story finder, seats, galaxy and stars | ✅ C1 to C3, K7 to K9, G0 to G6, galactic nuclei | G7 |
| Campaign record, logic, boxes, programs | ✅ J1 to J5, J7, J6 part 1 | J6 part 2 (programs run voyages and roles) |
| Life and evolution | ✅ L0 to L2, E0 to E4, E5 (cut), E6 part 1, E8 | E6 part 2, E7 |
| Formation and long life | ✅ F0 to F7, F9 | F8 |
| Surfaces | ✅ S0 to S7, S5 descent, S3b, S0b part 1 | 🔶 S16a detail levels, S14a lava tubes, SF1 time-lapse frames; ⏸ S9 to S13 |
| One body (stars and planets as one mass axis) | 🔶 B1 parts a to d part 1 | rest of B1, B3 |
| Climate and weather | ✅ W1 to W9, W7c part 1 | W7c part 2, W8 part 3, W10 to W12 |
| Light, sky, field effects | ✅ O1, O2, O3 slice, O7, O9, O10, O12, O13, Y2, Y3, MF6, MF7, MF9 part 1 | O4 and ND2 (light through gas), Y part 3 onward |
| Magnetic fields at the source | ✅ MF0, MF1 | MF2, MF3 |
| Galaxy histories, dark matter | ✅ DM1 to DM4, DM6, GH0 to GH3, GH9 | GH4 (PR open), GH5 to GH8 |
| Data sheet | ✅ constants, units, elements, water's 16 ices, fuels, the catalogued Sol | typed copies to 0, element inventory (waits on that) |
| Naming | ✅ NM0 to NM3d, NM6 parts 1 to 3 | NM4, NM5 |
| Chart room | ✅ U0 to U25 bar U7 part 2, V1 to V6, V10 to V14, T1 to T7 | Substances page, formation story, players' page view controls |
| Galaxy charts and travel | ✅ U6 parts 1 to 2c, V9, V8 parts 1 to 3c, ship at a world, U8 part 1 | V8 parts 4 and 5, U6 part 2b |
| Renderer | ✅ R1 to R6 (parts), SHIP14 stages 1 to 4b, V10a to V10e, OV1, OV2, GH9 | V9t (PR open), ship styles, OV4 |
| Agents | ✅ A1, A3, A4; A2 parts 1 to 4 | A2 triggers and rules, ND4, ND5 |
| Traveller pack | ✅ X3a to X3f, TG8 part 1, TG9, TG13, gaps 16, 18, 23, 25, 26 | standard ships (PR open), TG11 resources |
| Performance | ✅ readout, bench, minimum spec table | a phone report from the app |
| Explorers site | ✅ V4 shelves beside V3, Contains filter, sha header | publish end to end (your yes) |
| Converter | ✅ live, V3 importer as a browser module | in-tab conversion once its API is confirmed |

### Where the status files and reality disagree

- **Chart room status**: its "Done so far and what is next" paragraph still says copy and paste, the start sheet and Find are next. All three are done (U20, U21, T7). The table rows are right.
- **Physics status**: its closing paragraph says "next B1 one body", while the rows show B1, S16a, S14a and SF1 all in progress.
- **Across-tracks status**: the magnetic fields row says "MF0 and MF1 done … next MF1". The new-physics row says "next ND3b", but the kernel file shows ND3b parts 1 and 2 done.
- **Kernel status**: the J row says "J6 next"; the paragraph below says J6 part 1 is done.
- **Test-drive rough edges**: item 10 says True is held, but True on the system chart has shipped (V10a). Item 9 (galaxy charts flat) will be out of date once PR 290 merges.
- **Coming-together plan, section 2** (written 22 Nov) is behind: it says Traveller has no screens in the chart room (it has Sector map, Codes, Ships and stations) and that the bench and minimum spec are missing (both done).
- **Voyages design** still says "proposed" in its header, though its packages are built and its rulings are in memory.
- **Future tech and example packs** are accepted but have no track in the build plan (territory overlays got its track OV; these did not).
- **Project memory** lists "whether the size comparison joins the Scale switch" as waiting on you; you said yes (PR 283), so that line goes.

I have only reported these. Each owner thread fixes its own file.

---

## 2. Wanted, but not yet in the build plan

For you to decide when each joins. "Accepted" means you ruled yes on the design; "designed" means a design exists but not your yes; "idea" means only talked about.

| Item | State | What it is | Waits on |
|---|---|---|---|
| Future tech (track FT, FT1 to FT8) | Accepted 23 Nov | Technology as the physical limits it pushes, a tech ladder per setting at ten times the size per level short, capability on a civilisation, availability never says no. In 4.0 a civilisation's level and advances are set by hand or by a pack (Develop it is one dated event); nothing grows by itself until 4.1. FT1 is in PR 286. | Track FT added (PR 292); FT2 in the kernel next |
| Example packs, one per Explorers shelf (16 pieces) | Accepted 23 Nov | Traveller standard ships, World hooks, Sol today, small data packs, sky overlays, Pulp solar system, What if G drifts, Retro future look, Bridge console look, formula FTL rules, A working Sol, two tours, Megastructures. Game-side packs for other systems (Mothership compatibility, Eclipse Phase, 2300AD) move to 4.2 | Track EP added (PR 292); licence email for Mothership; view styles for the two looks |
| FTL rules as six plain fields, start-and-end conditions as data | Accepted 23 Nov | Duration in home and ship time, fuel, where a leg may start and end (clear of a body, gravity level, L points, gates, anywhere), drawn under HUD | Travel thread package |
| Atomic Rockets ("Paper spacecraft", RK1 to RK8) | Designed, proposed | The site's physics into the base packs, its design studies as flyable ships, power plants as parts, slow spiral courses, mission tables as test fixtures | Your answers to its 11 questions |
| Megastructures | Accepted direction | A construct carrying physics blocks (thrust for stellar engines; a swarm dims the star) | A short design note first (Traveller and kernel threads) |
| Surface landing and free wander | Accepted | Pick a site, ship autoroutes down, walk or low drone over a few km patch, phones get a basic walk view | Packages; the ground scale |
| Construct styles for ground facilities | Accepted | Same styles as ships, for bases and sites | After ship styles |
| Element inventory | Accepted 23 Nov | Every element per star and per body (patchy gold, Psyche-like cores), feeding mining | Typed copies at 0 and atoms moved first |
| Formation "Change the story" | Accepted, parked | Change a formation and Orac's request on the story chart | The kernel's branching package |
| Weapons (ranges and heat only, no dogfighting) | Idea | Revisit as its own note when a game pack wants it | A game pack asking |
| Grown territory histories (OV14) | Moved to 4.1 (Alex, 23 Nov) | Borders that grow over time by themselves | 4.1, with culture and technology in life |
| Paid packs in the app | Built on Explorers, switched off | Stripe, 20% cut, not surfaced | After stage 5 |
| Real-sky pack (Gaia, non-commercial) | In plan, parked | Real stars round Sol | After stage 5 |
| Uploaded 3D models (GLB, OBJ, STL) | Accepted for later | Your own ship models in a pack with credit | Packs carrying files |
| Ground detail (S9 to S13), later exogeology (S17b to S21), later weather, sound beyond the engine, later field stages | In plan, parked | Depth, not joining up | Their threads |
| Low-power switch, Toytown spacing, GM skins | Dropped for now | | Testers asking |
| Explorers site look rework | Accepted for later | Skin now, rework later | |
| Supporter page (Ko-fi or Patreon) with a Discord supporter badge | Idea, for later (Alex, 23 Nov) | For all of Alex's RPG tools (Ovid more than SSE4), made by Claude from existing assets when there is good material to show. Caution: support is never tied to named-setting fan packs, which stay free; worth one sentence to Mongoose that the project takes voluntary donations | Good material to show; Alex's word |

---

## 3. Version 4.1 and later

**Headline (Alex, 23 Nov): "bringing dynamic life to the simulation."** Alien life that grows its own culture and technology, an economy that drives the autopilots, and histories that grow by themselves. 4.0 sets and records these by hand; 4.1 makes them move.

**The core idea (Alex, 23 Nov): "a deep system that defines civilisations based on their evolution and history, and that will in turn drive the civilisation's direction."** A species' biology, its world and its history shape its culture, its technology choices and its aims, and those aims steer what it does next. It builds on the evolution design (`SSE4-evolution-design.md`: gradients, traits, lineage) and on life's existing record of when a world's life could reach tools and language (L1b).

1. **Richer alien life, with culture and technology as part of life** (Alex, 23 Nov): a new evolutionary biology in which culture and technology grow out of the living things, possibly driving automated tech trees. It takes in the evolution design's later horizons (what creatures look like, simulated windows from Evolution Lab, open evolution) and the handoff's technosphere stage. Alex, 23 Nov 17:49: "So all these systems can be extended then and we can have a lighter touch." So in 4.0, whatever a GM or a pack sets by hand stays, as plain dated data: a civilisation's tech level and advances, its FTL rule, what it holds on the map and its border history, names, calendar, units, styles. Anything that grows by itself waits for 4.1: grown political histories (OV14), automated tech trees and research, culture.
2. **Economy pack** (ruled 21 Nov): goods priced by scarcity and carrying cost, trade ships, players buying and hauling. Alex, 23 Nov: "The economy will drive the system needed to run autopilots better... A reason to move resources (and promote conflict)." So in 4.1 the economy gives the role programs their reasons: miners mine what is scarce, traders haul it, pirates and patrols follow the cargo. Its high-level design starts once resources, readiness, programs and Traveller trade data have landed.
3. **Sealed turns** (J8): downtime weeks and turn-based play with sealed orders. Marked "later" in the plan; it fits 4.1 with the economy.

Alex, 23 Nov: "Now we can manually set and record stuff; 4.1 brings in systems to manage them dynamically and can add the story." That is the rule for both 4.1 items: 4.0 records by hand, 4.1 runs it.

### What 4.0 should leave ready for the economy to drive the autopilots

| Leave ready in 4.0 | Why the 4.1 economy needs it | State |
|---|---|---|
| Role programs that take their goals from data (what to fetch, where to take it, whom to follow), not from code | The economy only has to write those goals; the programs already know how to fly them | ⬜ J6 part 2; goals hand-set in 4.0 |
| Resources as real amounts that can be mined out, stock on any address, transfers that conserve | Scarcity is read off real inventories, never a table of prices | ⬜ TG11, element inventory |
| Routes, fuel and travel time from the voyage finder, with carrying cost | What it costs to move a cargo is already physics | 🔶 V8 parts 1 to 3c done; counted jumps (MP16) and parts 4 and 5 to do |
| Ship readiness and discipline as kept values | Who will take a risky run, and who turns pirate | ✅ TG9 |
| Holdings and actors with relations | Who owns the cargo, who is hostile, where conflict starts | ⬜ OV3; ✅ J1 actors and relations |
| The `economy.` names and the `trade` tag group reserved | The pack adds its state without changing 4.0 results | ✅ TG1 |

### What 4.0 must have in place so 4.1 life slots in

| Must be in 4.0 | Why 4.1 needs it | State |
|---|---|---|
| Fixed life sockets: each life box swappable without changing what reads it | New biology replaces boxes; screens, sifters and Minerva cannot tell which stage made a fact except by its label | ✅ L0 |
| Energy gradients and chemistries as data, the shared catalogue format with Evolution Lab | New biology starts from what a world offers | ✅ E0, E1; ⬜ E7 further chemistries (ammonia water, hydrocarbon, sulphuric acid) |
| Functional traits under selection, and a **kept lineage tree** with trait values | Appearance is derived from traits and lineage, so nothing in 4.0 is redone | 🔶 E5 cut to what E6 needs; check the lineage tree is stored, not only used |
| Soil, cover and biomes from climate | The ground a richer biology lives on | 🔶 E4, E6 part 1; ⬜ E6 part 2 |
| Life on the chart's own worlds | Without habitats on adopted worlds, nothing runs on real campaigns | ✅ L3 (history over time next) |
| Exporter to Evolution Lab and a socket for one simulated window with a fidelity contract | Lab mechanisms are proven there, then promoted | ⬜ E9 (waits on the Lab's own two sessions) |
| The capability graph reaching tool use and language | The bridge from biology to a civilisation | ✅ L1b (built; stays as it is, no more work in 4.0, ruled 23 Nov) |
| Capability on a civilisation as plain dated data, inherited down the tree | A 4.1 tech tree writes the same dated advances a GM writes by hand today, so nothing is redone | ⬜ FT2 (kernel), lighter: set by hand or by a pack, no hooks for growth in 4.0 |
| Tech ladders as pack data | A grown tree climbs the same ladder an authored one does | 🔶 FT1 (PR 286) |
| Holdings with dates and status | Grown civilisations claim space through the same territory model | ⬜ OV3; grown histories (OV14) are now 4.1 itself |
| Actors and relations in the record, programs that act | Civilisations as actors with goals | ✅ J1, J6 part 1; ⬜ J6 part 2 |
| Real element and resource inventory | Biology and technology both need materials | ⬜ element inventory, TG11 |
| Naming and construct styles per culture, set by hand | A grown species gets its own names and look | ✅ naming layers; ⬜ ship styles per culture |
| Evidence labels on every fact (observed to fictional) | Speculative biology is marked as such | ✅ |
| Own keyed draw streams per model | Adding 4.1 biology must not change 4.0 results or golden hashes | ✅ mechanism exists; recommend reserving `life.lineage`, `civ.` and `tech.` names now, as was done for `economy.` |

---

## 4. Version 4.2 and later

**Headline (Alex, 23 Nov 19:47): dynamic adventures in a living, breathing simulation.** A co-GM that makes random encounters and interesting events to bury your clever plot in, runs pirate skirmishes as a side channel, and leaves wrecks to investigate.

**4.2 is where SSE4 goes hard into RPG support** (Alex, 23 Nov 19:50: "4.2 is where this turns from an SF VTT"). Game packs other than Traveller wait until then. **Traveller is the showcase pack** for 4.0 and 4.1.

| Item | State | What it is | Waits on |
|---|---|---|---|
| Sites you can go inside (track IS, `SSE4-interior-sites-design.md`) | Designed, proposed, then parked to about 4.2 | Wrecks, built sites, the Inside card, exploring that inks the map, an example pack; its 14 questions are not live until it is picked up | 4.2, and the 4.1 economy and autopilots that make the skirmishes and wrecks |
| The co-GM: random encounters, events, side-channel skirmishes | Idea | Runs on 4.1's dynamic life and economy | 4.1 |
| RPG support for game systems beyond Traveller | Ruled for 4.2 (Alex, 23 Nov 19:50) | Mothership compatibility and its play features (manifest, crew, character sheet, table rolls, difficulty numbers for other systems); the Eclipse Phase and 2300AD fan packs; any other game-system pack | 4.2 |
| Generic game layer for a 4X pack ("On the Expansion of Space") | Idea, now 4.2 | Turns, sealed orders, empire ledgers as blocks | Sealed turns (J8), the other author's agreement |

Stays in 4.0 as already accepted: natural caves and lava tubes from a world's own history (S14a, with their survey map), because the world makes them and nobody has to author them.
