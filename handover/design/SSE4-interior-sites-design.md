# SSE4 sites you can go inside: caves, wrecks, mines and other interiors (track IS)

Status: **proposed 23 Nov 2026, not yet accepted.** Fourteen questions for Alex in section 13.
Mockup: `/mnt/project-files/sse4/IS-screens/site-survey-mockup.html` (artifact "SSE4 site survey", https://claude.ai/artifact/TPrbGRF7LBmyNHMpqgbR6V). Screenshots beside it in `IS-screens/`.

Asked for by Alex, 23 Nov 19:28 and 19:33: games in SSE4 are going to be "space trucker sims with horror, or frontier explorer with horror, or military team with horror", run inside SSE4 "even down to crashed spaceship wreckage and entering a lava tube". SSE4 supplies the exploring, mining and hauling, with no rule set built in. The example packs note (section 4.3A) listed "sites with an inside" as a general piece needing its own design. This is it.

Figures and papers below are from memory and marked for the data sheet thread's citation check, as with every design note.

## 1. The idea in six lines

1. **One thing: an interior.** A site with an inside is a set of **spaces** joined by **ways**, with real sizes, placed in the frame of whatever holds it. A lava tube, a cave, a crashed ship, a dead station, a mine and a bunker are all that one shape.
2. **Two sources, nothing special-cased.** Spaces in the ground come from the world's own history (lava, water, ice, faults). Spaces in a construct are the parts the generator already sized from physics (habitat, hold, tanks, reactor, spine). A wreck is a construct with a dated "wrecked" event.
3. **Conditions are computed, per space.** Air, temperature, radiation shelter, gravity, light, water, radio contact and how sound the roof is, each from the world and the route to the open. Dangers are shown in the same plain words as routes and landings. Nothing is refused and nothing is rolled.
4. **The survey map is the main view.** The plan-and-profile drawing cavers make, on every device. A 3D walk inside follows, lit only by what the party carries.
5. **Exploring inks the map.** The GM has the whole site in pencil. As the party goes in, what they see is inked onto their map. This is the "Known to" control with nothing added.
6. **What happened there is authored.** The GM, a pack or Minerva says why the ship came down and what is in the hold. SSE4 says what the place is like. Histories that leave wrecks by themselves are 4.1.

## 2. What already exists, and what each part becomes

| Exists | Where | In this design |
|---|---|---|
| Lava tube envelopes, and passages placed on approach (same tube from any site) | S14a parts 1 and 2, `survey_surface.tubes`, `survey_caves` | The first kind of ground interior. Its passage graph is the general shape; the other void makers (S14b) fill the same shape. |
| Eight void makers designed: lava tubes, solution caves, acid caves, Titan karst, ice caves, water lenses in ice shells, sea caves, fissures and talus | Rock 3 comparison 5.5 (accepted) | Unchanged. This note adds what is inside and how it is used, not new geology. |
| Flourishes that resolve on approach into addressable, nameable facts; recorded ones persist | surface design 4A.3 | A site is a recorded flourish with children. |
| Tech landmarks, wrecks among them; "ruins fall out for free" as a landmark with age applied | surface design 4A.5 | Kept and widened: a ruin or wreck is a construct with a state and a date, and its inside is its parts. |
| Ships generated from components sized by physics; plan views with call-outs; no deck plans | ship design rulings, renderer `hullFromShip`, `DesignRecipe` | The parts in the plan view are the spaces of the interior. Section 4 says what that is and is not. |
| "A construct is just a construct"; stations, ground facilities and megastructures on one rule set | future tech 6, megastructure ruling | So one interior rule covers them all. |
| Readiness and derelict status on constructs | TG9 | "Wrecked" and "derelict" are states read from dated events, not flags. |
| Land anywhere: pick a site from the map, autoroute down, wander a patch a few km across on foot or by low drone | SL1, `plan_descent` | An entrance is a thing in the patch. "Go inside" is the next step from it. |
| Hazards in one shape (subject, ratio to limit, chance with its basis, reasons) and plain difficulty words (routine, easy, moderate, hard, very hard) | `sse-nav` hazards, `ship_at_world`, ui-contract `DIFFICULTY_STEPS` | Site dangers use the same shape and the same words, so a game pack's numbers beside them apply here too. |
| Gas mixes and what to wear outside | Traveller X3d | Asked again per space: what to wear inside. |
| Dose along a route; ship shields in g/cm² | `sse-nav`, ship generator | Shelter under a roof is the same sum with rock in the way. |
| The band switch (Visible, Infrared, Ultraviolet, X-ray, Radio, Mag); a seat only gets bands its sensors carry | shared view controls | How hidden sites show up from orbit, and what a hand scanner shows inside. |
| "Known to": pencil and ink, one small eye, scope "this and what is inside it" | SV design (accepted in direction, 23 Nov) | The whole secrecy story for sites. No second control. |
| Caves and lava tubes as a habitat kind | life design | Real places for it. Plants and microbes are shown, animals are not, as ruled. |
| Notes, things and tables hung on places; roll on a table | record design, example packs 4.3A | What the GM puts in a space. |

## 3. The model

```
Interior { host: address,            a surface feature or a construct
           source: "ground" | "construct",
           spaces: [{ id, kind, shape: { length, width, height }, at: position in the host's frame,
                      floor_depth, made_by, dated, state }],
           ways:   [{ from, to, kind, clear: { width, height }, drop, state }],
           openings: [way ids that reach the outside] }
```

- **A space** is somewhere a person can be. Kinds are plain words: passage, chamber, shaft, skylight floor for ground; crew section, hold, tank, reactor room, drive section, spine, airlock, working, gallery for constructs. A pack can add kinds.
- **A way** joins two spaces or a space and the outside: an opening, a squeeze, a drop, a breach, a hatch, a collapse that can be dug. It has a clear width and height, so "will the drone fit" and "will the loader fit" are answers, not flags.
- **State** on a space or way is read from dated events: open, blocked, flooded, crushed, sealed, holed. Nothing is edited in place.
- Every space and way is an address under the site, so it can be named, noted, kept secret, pinned and asked "why is this here?".
- **Interiors nest.** A base built in a lava tube is a construct whose host is a space. A wreck that slid into a rille is the same. One rule: a space can host things.

Two special cases are deliberately absent:

- **An inside-out landscape is a surface, not an interior.** The inner ground of a spinning habitat or a ringworld is drawn and walked by the surface model in another frame. Its service decks, hull voids and hangars are an interior. The megastructure note (still to be written by the Traveller and kernel threads, as ruled) owns the first; this note gives it the second.
- **A crewed, working ship is not a site in 4.0.** Its parts could be walked by the same rule, but nothing needs it yet and it invites deck plans by the back door. See question 2.

## 4. What a construct's interior is, and is not

Alex ruled no interior deck plans: a ship's 2D is a top and side view with call-outs for where each major system sits. A wreck must not quietly break that.

**It is:** the generator's own parts, as spaces. The crew section with its real volume from crew count and endurance. The hold. Each tank, open to walk through if it is holed and empty. The reactor room behind its shadow shield. The spine that joins them, as long as the radiators and neutron dose made it. The airlocks. Parts are split further only where physics or the template already splits them: pressure compartments in a large habitat, the segments of a spin ring, the sections of a station. A small freighter comes to about 8 to 15 spaces; a big station to a few hundred, resolved a section at a time.

**It is not:** cabins, corridors, doors, bunks, consoles or furniture. The app never claims to know where the galley was. On the survey map a construct interior looks like its own plan view with the call-outs turned into rooms, broken where the wreck is broken.

That happens to be the map these games use anyway: a handful of named compartments joined by lines, with the GM describing what is in each. If a GM wants more, three ordinary tools already give it: attach their own plan as media on the site, add or split spaces by hand, or let a pack dress a kind of space from a table.

**Wrecked, derelict, ruined.** Any construct can be given one dated event:

```
kind: "construct.wrecked"   subject: construct address
payload: { cause: "crash" | "explosion" | "breakup" | "battle" | "abandoned" | "decay",
           where?: site or orbit,  speed?, angle?, energy?,  from?: which part }
```

- **Abandoned** breaks nothing. The ship is whole, cold and dark: a derelict.
- **The others** run a small cited damage block over the parts. Specific energy against each part's strength gives how broken it is (whole, holed, crushed, gone), the same disruption measure small bodies use. Speed and angle give the length and heading of the debris field, and parts thrown clear become things on the surface along it. This is a rule of thumb for a stage set, not a crash simulation, and it goes in the fudge register saying so.
- **Then time passes.** From the wreck's date to the clock's date, the world's own wearing-down terms apply, as they do to ruins: dust and sand bury it at the local rate, air corrodes it, ice covers it, and on an airless world it sits unchanged for a million years. A hot reactor cools by the decay law, so "the reactor room is still at 80 °C after forty years" is a computed line.
- **Every result can be overridden by hand**, part by part, as an authored pin. The derived wreck is a good default, never a constraint.
- **Any ship can become a wreck**, including one the players have flown past for six sessions. It keeps its address, name and history. Before the date on the time ruler it is still flying.

**Dug constructs.** A mine, a bunker, a vault and a buried base are ground facilities whose parts are spaces in rock.

- **A mine's workings are as big as what was taken out.** Resources are real amounts, so the dug volume is the amount removed over the ore's density and grade, laid along the ore body the surface model already has. A worked-out mine is large and empty for a reason the card can state. In 4.0 the amount taken is set by hand or by a pack; in 4.1 the economy digs.
- **A bunker's depth is the protection it was built for**, in g/cm² of cover, from the same shelter sum as section 6.

## 5. Finding a site

Nothing here is a "has cave" flag. A site is found because something about it can be measured, and each method is a sensor rule as data: what it reads, the smallest thing it resolves at a range, what it moves up the knowledge ladder.

| Method | Reads | Finds | Shown in band |
|---|---|---|---|
| Imagery | pits, pit chains and rilles; a fresh scar, a debris field, ejecta | skylights wider than the camera resolves; a crash younger than the surface's wear | Visible |
| Thermal | a pit or cave mouth is warm at night and cool by day against the ground round it (Cushing et al. 2007, Mars); a reactor or a power source still warm | entrances; live or recently live wrecks | Infrared |
| Gravity | the missing mass of a big void (Chappaz et al. 2017, GRAIL) | tubes and chambers hundreds of metres wide, with no entrance needed | a HUD overlay |
| Radar sounding | an echo from a roof and floor (Kaku et al. 2017, Kaguya; Carrer et al. 2024); bright metal on dull rock | intact tubes near the surface; buried wrecks and structures | Radio |
| Magnetics | iron and working machinery against the crust's own field | wrecks, mines, bunkers | Mag |
| A beacon, or leaking radio | whatever is transmitting | a site that wants to be found, or one that does not know it is shouting | Radio |
| Ground work | a drone in the patch; a thumper and a line of sensors | every entrance in the patch; depth to the roof | on the site map |

- At Orbit and Survey level the world already carries envelopes ("tubes about 280 m wide in this lava plain, about 40 skylights"). A survey method turns part of an envelope into a **found site**: a landmark on the world's map with a small "has an inside" mark and what is known so far ("a pit 90 m across, floor not seen").
- A found site is a landmark like a peak or a port, so the Land screen can pick it. A natural site needs no author: a keen GM can sift for "a moon with intact tubes wider than 300 m near the equator" and find it there.
- For a derelict in space, finding is the ordinary ship question: a dead cold hull is faint in every band, which is the right answer and a good scene.
- The GM sees everything in pencil. Each method writes the same `observe.grant` a press of the eye does, so "they ran the radar pass on day 14" is on the time ruler.

## 6. What it is like inside

One block per condition, each cited, each answering "why". All are worked out for a space when it is opened, in well under a millisecond, from the world's facts and the path to the open.

| Condition | From | What it gives |
|---|---|---|
| **Air** | the outside air if any way is open to it; denser gases settle in low spaces where a source feeds them (volcanic CO₂, as in Earth's caves); a sealed compartment keeps the ship's mix if it is whole | pressure and mix per space; "what to wear inside"; bad-air pockets with their reason |
| **Temperature** | day and year swings die out within a skin depth, √(κP/π), about half a metre for a day and a few metres for a year in rock, leaving the surface's long-term mean; then the world's heat flow with depth; cold air pooling under a single high entrance; a wreck's decay heat | a steady figure per space. A lunar pit sits near 17 °C while the surface swings from 100 °C to −170 °C (Horvath et al. 2022) |
| **Radiation shelter** | cover in g/cm² (roof thickness × density) against the world's surface dose, the same sum as a ship's shield; plus any source inside | dose rate per space. A few metres of rock is full shelter on the Moon (De Angelis et al. 2002). A wreck's reactor room can be worse than outside |
| **Gravity** | the world's; in a tumbling derelict, spin gravity by distance from the axis, from the tumble the event left | walking or floating; which way is down in each space |
| **Light** | the opening's size and the real sun angle at this place and date; nothing beyond the reach of the opening | "the pit floor is in sun for six hours either side of noon, at this latitude, this month"; otherwise dark |
| **Water and ice** | the water table and the season's run-off from the one climate; cold-trapped ice where the space never warms | flooded spaces and sumps; flood danger after rain; ice as a real minable amount |
| **Soundness** | roof span against roof thickness and gravity (the S14a stage); a wreck's holed and crushed parts | chance of a fall, with its basis |
| **Contact** | rock stops radio in a few metres, so contact is line of sight to an opening, or a relay, or a cable | where the party loses the ship, and where a relay would restore it |
| **Getting through** | each way's clear size and drop against the local gravity | squeezes, climbs and the speed of a fall from a drop; whether a drone or a vehicle fits |

- These are shown on an **Inside** card per space, built like the "At world" card: figures first, then dangers, each with a plain difficulty word and its reasons.
- **Danger is drama, never forbidden.** No way is greyed out. A flooded squeeze with bad air beyond is offered like any other route, with the odds.
- **No rolls and no skill checks.** The app gives the GM the information; the GM runs the table. Where a game pack defines numbers for the difficulty words, they show beside them here exactly as on a route.
- Sound is a small free extra: a space's echo follows from its size and its air, for when sound is on.

## 7. Getting there and going in

1. **Pick it.** A found site is a landmark on the Land screen. The ship's verdict for that ground is shown as for any site (a lava field is rough; a debris field is worse).
2. **Fly down.** The ship autoroutes; the descent plays or is skipped.
3. **Wander the patch.** Entrances and thrown-clear wreckage are things in the patch, with names. The low drone can look down a skylight.
4. **Go inside.** One action at an entrance. The view changes to the site's survey map with the party's marker on the first space.
5. **Move by intent.** Press a space: "go here". The kernel gives the route through known ways, its length, the time at a pace for this gravity and this kit, and what it passes through. Nobody steers. A drone is sent the same way and reports what it sees.
6. **Come out.** Back at the entrance the patch view returns. Leaving the world is back in the ship, as ruled.

A derelict in space is reached by an ordinary voyage leg that ends in a rendezvous. Matching a tumbling hull is a danger with reasons like any other. "Board" at the end of it is the same "Go inside".

Time inside runs on the one universe clock. The route time is real, so "three hours in, three hours out, on four hours of air" is visible before anyone commits. The air is the game pack's kept value if it has one; SSE4 supplies the hours.

## 8. How it is shown

**The survey map (first, every device).** A plan from above and a profile from the side, drawn the way cavers and salvage crews draw them: passage walls, cross-sections at intervals, drops marked with their depth, water hatched, breakdown stippled, a scale bar and a north arrow. A construct uses the same drawing with straight walls. It runs on the canvas tier, so a phone has all of it. The party's marker, the route and the Inside card sit on it.

**The cutaway.** The world's Cutaway view gains its void band (already planned), and a site's long profile is the same view close up.

**The walk view (after, by device).** The Surface view carried inside: a volume built only for the space the party is in and its neighbours.

- Ground spaces come from the passage graph's widths and heights; wall texture, rubble, ice and formations are decoration from the same seeds, below the fact floor.
- Construct spaces are plain volumes from the part's dimensions in the construct's style, holed and bent where the damage block says. Fittings are decoration and never facts.
- **Light is honest.** True style inside shows what the party's lamps and any openings light, and nothing else. That is the horror look, and it is free.
- The band switch works inside for whatever sensors the seat carries: Infrared shows what is warm.
- A phone gets the survey map plus a basic walk view of the one space it is in, and "Take photograph" for a still from the GM's machine.

**Charts and cards outside.** On a world's map a site is a landmark. Its card (from the envelope alone, no work for a click) says what kind of place, how big, how it formed or what it was, and what is known.

## 9. How much is worked out, and when

| Level | A cave or tube | A construct |
|---|---|---|
| Glance, Orbit, Survey | an envelope per cell: length of passage per km², width, depth, stage, cause. No passages | its record: template, state, date |
| A click or hover on the landmark | the card, from the envelope | the card, from the record |
| Site (approach, landing, the GM opens it) | the passage graph for the patch and what connects to it; leads that leave the patch end in "continues", with the envelope's expected length beyond | the parts as spaces with damage and age applied; a big station one section at a time |
| The party enters a space | its conditions; in the walk view, its volume and its neighbours' | the same |

- **The same site every time.** The graph is a keyed draw from the site's address and the code version, so every device and every visit gets the same passages. S14a already tests "the same tube from any site"; constructs get the same test.
- **Nothing is stored but the choice and the edits.** The resolve record, names, and the GM's dated changes (block this way, flood this level, add a chamber, put something here). Caches can be thrown away.
- **No heavy work for a click.** A site resolves on approach or when the GM opens it, never from a list or a hover. A weak device fetches the graph from the GM's machine.
- **Sites have dates.** A tube formed with its lava flow; a cave level with the river that cut it; the wreck arrived on its day. Scrub the ruler back and the wreck is not there yet.

## 10. Secrets

The SV design (accepted in direction, 23 Nov, with the eye as its mark) covers this with nothing new, which is a fair test of it.

- A site, each space, each way and each thing in a space carries the eye. The GM sees the whole site in pencil from the start.
- **Exploring inks the map.** When the party's marker enters a space, that space and the ways visible from it become Known to that crew: one dated grant, one undo. A lamp shows a short way; a drone or a mapper shows further, by line of sight.
- **Open leads are honest.** A way the party can see is drawn as cavers draw it, an open end marked "continues". A way they could not see (a buried hatch, a sump that hides a dry passage beyond) is not on their map at all, and no count or gap gives it away.
- **Heard of** is the half-open eye: "there is a wreck in the Tarn badlands", with a name and a place and nothing inside.
- **Cover stories** work on sites as on anything: the mine's card says "worked out, sealed 2291" to the table while the GM's says what is really down there.
- **Hand over** is the old survey: "give the table what the mining company knew of the lower levels", and their map inks in two levels they have not walked.
- Two crews in one site each have their own map.

## 11. What the GM, a pack and Minerva add

- **Add a site.** "Wreck a ship here" (pick a template or a ship already in the campaign, a cause and a date), "Sink a mine here", "Build in this tube". Each is a change through the ordinary intent path. Orac can set one up as a request the GM approves.
- **Put things in spaces.** A note, a body, a log, a cargo, a creature from a pack, a table to roll on when the party enters. Each has its own eye.
- **Change it.** Block, open, flood, collapse, seal: dated events, so the site as it was is still on the ruler.
- **Minerva writes what happened.** She is given the site's facts (which ship, what cause, which parts broke, how long ago, what it is like inside now) and writes an account that agrees with them: the truth for the GM, and what each audience would find.
- **Packs** supply site templates, kinds of space, how a kind is dressed, and tables. Placement by rule ("one worked-out mine per belt colony older than fifty years") is pack data, as the built layer already allows.
- **No wreck appears by itself in 4.0.** Natural sites come from the world's history. Built and wrecked ones come from a hand, a pack or an agent. This is the lighter touch.

## 12. Packages, who takes them, and what waits

| Id | What | Thread | Check |
|---|---|---|---|
| IS1 | The interior shape in the kernel: spaces, ways, addresses, states from dated events, nesting. `survey_caves` returns it; op `survey_site` for any site. Finishes S14a's "next" in this shape and carries S14b's other void makers into it. | Build SSE4 proven core | the same graph from any approach on any device; a blocked way is one undoable event |
| IS2 | Conditions inside: the nine blocks of section 6 and the Inside result in the one hazard shape. | Build SSE4 proven core, with the weather thread for air and water | a lunar pit, an Earth limestone cave and a Mars tube each within stated bounds of the measured values |
| IS3 | Construct interiors: parts as spaces from a recipe; the `construct.wrecked` event, the damage block, ageing; dug constructs sized from what was taken out. | Build kernel resolve and rules, with templates from Build Traveller pack and part sizes from the ship generator (renderer thread) | a stock freighter wrecked the same way twice is the same wreck; every part can be pinned by hand |
| IS4 | Finding: the survey methods as sensor rules; found sites as landmarks; band recipes that show them. | Build Traveller pack (sensor rules), Build photoreal renderer (bands), Build galaxy charts and travel screens (derelicts in space) | a skylight below the camera's resolution is not found; the thermal pass finds it at night |
| IS5 | The survey map view, the Inside card, the party marker and route. Site cards. | Make the SSE4 test drive work end to end (chart room) | desk, phone and Night chart screenshots; a route's time matches its length and the gravity |
| IS6 | "Go inside" from the Land patch and "Board" from a rendezvous. | Build photoreal renderer (Land screen), Build galaxy charts and travel screens | from orbit to the first space in four presses |
| IS7 | The walk view inside, honest light, the band switch inside; phone basic view. | Build photoreal renderer | a space looks the same on two devices; a phone holds its frame rate in one space |
| IS8 | Secrets inside: ink on entry, leads, sight lines. Needs SV1 and SV2. | kernel, then chart room | nothing about an unseen space reaches a seat |
| IS9 | Authoring: add a site, put things in spaces, change it; Orac requests; Minerva's account. | chart room; Build Orac, Minerva and Tars | a wreck made by Orac reads the same as one made by hand |
| IS10 | An example pack, "Deep places" (question 13), and trailer clips from it. | Build Traveller pack; Capture footage for SSE4 trailers | a new GM can open a site and run it without reading anything |

Order: IS1 and IS2 first (a real lava tube with real conditions on the survey map is already a demo), then IS5 and IS6, then IS3 and IS4, then the rest.

**Waits for 4.1:** wrecks, ruins and mines left behind by histories that grow by themselves; the economy digging mines out; creatures and cave ecologies beyond plants and microbes; knowledge of sites spreading by itself; structures that fail under load while the party is inside; air that moves between spaces; crews whose supplies run down unattended.

## 13. Questions for Alex

Each has my recommendation. Answer in the usual way ("yes to 1, 2 and 4").

1. **One model for every inside: spaces joined by ways, whether the ground made them or a builder did?** Recommend yes. It is the cave passage graph already being built, made general.
2. **A construct's interior is its generator parts as spaces and nothing finer: no cabins, corridors or furniture, and only for wrecks, derelicts and ground sites in 4.0, not for working ships?** Recommend yes. It keeps your no-deck-plans ruling intact, and it is the node map these games use anyway. A GM can still attach their own plan.
3. **A wreck is any construct given one dated "wrecked" event (cause, and speed and angle if it crashed), with the broken parts and the debris field derived by a small rule and every part overridable by hand?** Recommend yes. The other choice is marking damage part by part with nothing derived, which is more work for the GM and less consistent.
4. **Log that damage rule as a fudge (a rule of thumb for a stage set, not a crash simulation)?** Recommend yes.
5. **No wreck or built site appears by itself in 4.0; caves and tubes always come from the world's history?** Recommend yes, under the lighter touch. Packs may still place built sites by rule.
6. **The survey map (plan and profile) as the main view on every device, with the 3D walk view to follow and one space at a time on a phone?** Recommend yes. It is cheap, it reads at a glance, and it is the right drawing for the subject.
7. **Exploring inks the map: moving the party into a space makes it and what can be seen from it Known to that crew, in one undoable step?** Recommend yes. It is the "Known to" control used as designed.
8. **Conditions inside computed per space and shown on an Inside card, with dangers in the same plain words as routes and landings, nothing refused and nothing rolled?** Recommend yes.
9. **Sites are found by named real methods (imagery, thermal, gravity, radar sounding, magnetics, a beacon, ground work), each a sensor rule, each showing in its own band?** Recommend yes.
10. **Movement inside is by intent with a computed route and time; supplies such as suit air stay the game pack's values, shown beside the hours where a pack defines them?** Recommend yes.
11. **A mine's workings are sized from the amount actually taken out, set by hand in 4.0?** Recommend yes. It makes mines honest now and is exactly what the 4.1 economy will drive.
12. **Derelicts in space use the same "Go inside", reached by a rendezvous, with a real tumble giving spin gravity; built after ground sites?** Recommend yes.
13. **A free original example pack, "Deep places": a lava tube with a base in it, a crashed freighter, a dead station and a worked-out mine, also used for trailer clips?** Recommend yes. It contains no game's names or tables, so it does not wait on the Mothership licence.
14. **Walking the inner landscape of a habitat or ringworld is left to the megastructure note as a surface, with only its service spaces treated as an interior here?** Recommend yes.

## 14. Standing rules this design keeps

- Processes, not classes: no "has caves" or "can enter" flags; a site is there because something made it and it can be entered because the way is wide enough.
- One general mechanism: one interior shape, one hazard shape, one secrecy control, one style mechanism for ships and ground constructs.
- The same site at every level and on every device; no heavy work for a click.
- Physics never faked: every condition has a cited block and a "why"; the damage rule is an open fudge.
- Danger is drama, never forbidden. The GM judges; the app informs.
- No deck plans. Players never hand-pilot.
- The 4.0 lighter touch: what happened is authored and dated; what grows by itself waits for 4.1.
- Named settings stay out: nothing here uses a game's words.

## 15. Sources to check (data sheet thread)

- Cushing, Titus, Wynne and Christensen 2007, GRL: THEMIS thermal detection of possible cave skylights on Mars.
- Chappaz et al. 2017, GRL: large empty lava tubes on the Moon from GRAIL gravity.
- Kaku et al. 2017, GRL: intact lava tubes at Marius Hills from the Kaguya radar sounder.
- Carrer et al. 2024, Nature Astronomy: radar evidence of a cave conduit under the Mare Tranquillitatis pit.
- Horvath, Hayne and Paige 2022, GRL: thermal and illumination environments of lunar pits and caves (about 17 °C in shadow).
- De Angelis, Wilson, Clowdsley, Nealy, Humes and Clem 2002, J. Radiation Research: lunar lava tube radiation safety.
- Blair et al. 2017, Icarus; Sauro et al. 2020, Earth-Science Reviews: tube sizes and roof stability (already cited by S14a).
- Decay heat after shutdown: the Way and Wigner approximation, for the reactor that is still warm.
- Thermal skin depth and cave temperature near the surface mean: textbook heat conduction; a speleology source to be chosen.
- Benz and Asphaug 1999, Icarus: specific energy for disruption, as the measure behind the damage rule (a borrowed measure, logged as a fudge).
