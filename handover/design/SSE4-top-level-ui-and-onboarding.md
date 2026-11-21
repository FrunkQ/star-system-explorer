# SSE4 top-level interface and onboarding

Status: accepted by Alex 21 Nov 2026 (the approach, and section 14 questions 1 to 6; see section 17). Proposed 20 Nov 2026. Alex, 21 Nov: "I like the approach", with additions that are now sections 8 to 13 (Explorers library and copy-paste, examples to break, an animated guide, undo and redo, time defaults per desk, how time reaches other systems, autopilots over time jumps). Nothing here is built. Mock-ups: the canvas "SSE4 top-level interface and onboarding" (four boards: start sheet, system desk, world desk, phone).

Asked for by Alex, 20 Nov: "not seeing much of the new functionality - time for a bit of a top end re-jig to surface UI stuff in a practical and 'makes sense' way. We can start thinking about onboarding now - how we layer features to make it approachable - revealing layers of depth. They should be able to 'see' all the core features up front - so they are clear what it is capable of. That fine line between learning and overwhelming."

This design keeps every standing rule: dynamic minimalism (`interface-style.md`), the chart room and Night chart, one fixed curated GM layout (handoff 8.8), lenses (8.6), seats that never get in the way (8.7), no modal wizards (8.5).

## 1. Why the new work is hard to see

Read from `main` at 011a328 and the latest screenshots. The live site could not be opened from the design sandbox, so this is from the code and the package screenshots, not from clicking through production.

1. **Eleven flat tabs that mix three kinds of thing.** Chart, Physics, Body, Sizes, Life, Explore, Screens, Player data, Symbols, Styles, Packs. Four are places to look, two are tools, five are setup. Setup takes half the bar, and a newcomer cannot tell which tab is "the app".
2. **The strongest features are 11 px links in the status line.** Make a system, Agents, What if, Look ahead, Campaign, Light, Sound and Share with players all sit beside the kernel version and two hash codes, and the line runs off the right edge at 1440 px.
3. **Features are nested two deep with no signpost.** Window, Galaxy, Cosmic web, Stories and Route are sections inside Explore. Cutaway, Take photograph, the recipe editor and the X-ray Sun are inside Body.
4. **Scale is a tab, not a direction.** Galaxy and Cosmic web are siblings of Route, so there is no sense of zooming out from a world to the web, although the handoff says the same chart grammar is used at every tier.
5. **It opens on a fixture.** The first thing seen is Kestrel with a "FIXTURE DATA" stamp and a card that says "Nothing selected". There is no statement of what the app is or what it can do.
6. **About a third of what was built has no screen at all.** See the table below. This is the largest part of "not seeing much".

### What is built, and where it can be reached today

| Capability | On screen today | How a user gets there |
|---|---|---|
| System chart, edit in place, why? | Yes | Chart tab |
| Time ruler, days to Gyr, campaign now | Yes | Always present |
| What if, Look ahead (age a system) | Yes | Small links in the status line |
| Make a system (V3 import, grow one) | Yes | Small link in the status line |
| Body globe, Take photograph, recipe editor | Yes | Body tab, among the facts list |
| Stars drawn from physics, visible and X-ray | Yes | Body tab with a star selected |
| Cutaway | Yes | Bottom of the Body tab |
| System 3D | Yes | 2D/3D switch on the chart |
| Sizes and Measure | Yes | Sizes tab; Measure on the chart |
| Life screen | Yes (example worlds) | Life tab |
| Route, Window, Galaxy, Cosmic web, Stories | Yes | Sections inside Explore |
| Physics flows and change requests | Yes | Physics tab, or "Open the flow" from a why? |
| Lenses | Yes | Dropdown in the bar |
| Light switch (Eye, Camera, SHO, single bands) | Yes | Status line |
| Sound | Yes | Status line |
| Agents (Tars, Orac, Minerva) | Yes | Small link in the status line |
| Players' screen, seats, requests | Yes | Status line |
| Campaign save and open | Yes | Status line |
| Packs, Screens, Player data, Symbols, Styles | Yes | Five tabs |

| Built in the engine, no screen yet | Kernel op | Where this design puts it |
|---|---|---|
| Spectrum of any object at an observer | `read_spectrum`, `read_light` | Light and spectrum instrument |
| Planet finding from afar (transit, wobble, imaging) | `survey_planets` | Survey from afar instrument |
| Nebulae from star deaths, patch glows | `read_nebulae` | Objects on the Local stars and Galaxy charts |
| Sensing and detection ranges | `detect` | Sky from here, and contacts on the Run desk |
| Stellar activity: spots, flares, ejections | `read_activity` (only the star picture uses it) | Events on the ruler, a star's card |
| Surface survey: plates, rivers, craters, climate, names, GM reshape | `survey_surface` | Layers on the Globe; Ground and plates instrument |
| Descent and landing | `plan_descent` | Landing instrument |
| Magnetic dynamo | flows | Cutaway and card now; field overlay on the Globe later |
| Notes, kept values, actors | `add_note`, `set_kept`, `add_actor`, `record` | "Notes and people" on every card |
| Triggers and rules awaiting a decision | `rules_*` | Run desk, the table panel |
| Programs (patrols, autopilots) | `programs*` | Run desk, the table panel |
| Stock ships, stations and sites | `templates`, `create` | "Add from stock" on the Build desk |
| Traveller world codes, hex map (views written, mounted nowhere) | `codes`, `import_sector` | Card section; the Local stars chart; Bring on the start sheet |
| Multiples as nested pairs | resolve | The chart (needs drawing) |

Also: the browser loads only the base-physics, base-time and Kestrel player-data packs (`stack.json`), so the Traveller pack is not in the running app. Weather is not on `main` yet.

## 2. The idea: four questions, four controls

Every screen in SSE4 answers the same four questions. Give each one control, always in the same place, and the eleven tabs disappear.

| Question | Control | Where | Replaces |
|---|---|---|---|
| **Where** am I? | The place line: Cosmic web › Milky Way › Local stars › Sol › Earth | Top left, it is the title | Chart and Body tabs; Galaxy and Cosmic web sections |
| **When** is it? | The time ruler | Bottom, unchanged | Nothing. What if and Look ahead move onto it |
| **How** am I looking? | The instrument rail | Left, with words | Body, Sizes, Life, Explore tabs and their hidden sections |
| **What** am I doing? | The desk: Explore, Build, Run | Top centre | Edit/View switch and the job lenses |

Plus two helpers on the right of the bar, **Find** and **Ask**, and one door, **Workshop**, for everything that changes the tool rather than the universe.

### 2.1 Where: the place line

The title becomes a line of places from the largest scale to the one in view. Each step is a link, and the last one is the big serif name. Double-clicking a body on any chart goes down a step; the line (or Esc, or pinching out) goes up. A small "8 worlds ▾" beside the name lists siblings and children for people who prefer a list.

Scales: Cosmic web, Galaxy, Local stars (the patch, and the hex map when the Traveller pack is stacked), System, World, and later Ground. This is the handoff's "same chart grammar at every tier" made into navigation. The galaxy and foam charts already exist and simply move from Explore into the line.

### 2.2 How: the instrument rail

A left rail lists, in words, every way of looking at the place in view. This is the answer to "see all the core features up front": the rail is the table of contents, and it is always open on a desktop.

- The rail changes with scale but keeps its groups in the same order: **Look**, **This world** (or Find, at system scale), **Travel**, **Understand**.
- At a system: Chart, Sizes, Light and spectrum, Sky from here; Survey from afar, Stories; Route; How it formed, The physics.
- At a world: Globe, Cutaway, Sizes, Light and spectrum, Sky from here; Ground and plates, Air and weather, Life; Route to here, Landing; Its history, The physics.
- At most ten entries per scale. If a scale needs more, the extra ones go behind "More" at the foot of a group.
- An instrument is a view module, as now. The rail is built from the registry by a `scales` field each view declares, so adding a view adds a rail entry and Orac, packs and player screens can use the same list. One mechanism.
- An instrument the device cannot run stays listed, dimmed, with the reason (the device-edge rule).
- Modifiers that apply to any instrument sit in a strip on the stage, not on the rail: 2D/3D, **Light** (eye, camera, X-ray and so on), Sound, **Measure**, Fit. Sizes is on the rail at every scale and Measure is on every chart, so the two most-loved V3 tools are never more than one click away.

### 2.3 What: three desks

Explore, Build, Run. The desk decides which verbs are armed, and nothing else.

- **Explore**: nothing can be changed by accident. Cards read as prose first. This is where a learner lives and where everyone starts.
- **Build**: values on cards become editable, bodies drag, and a short tool row appears (Undo, Redo, Add a moon, Reshape the ground, Add from stock, New system).
- **Run**: the table. Shows what players know, the Scanned button, the players' screen and seats, requests to approve, notes, triggers waiting for a ruling, programs. Editing clutter goes.

Under the hood a desk is a lens, the mechanism U9 already built: Explore is today's Basic/Advanced in View mode, Build is the Building lens in Edit mode, Run is the Running lens. Custom lenses, pack lenses and "as players see it" stay, reached from the desk's own small menu (Run offers "See as: the table ▾"). The Edit/View switch goes, because the desk says it.

### 2.4 Depth: Curious, Science, Engine

One dial, three stops, applied to the whole app. The names are the three depths "why?" already uses, so there is one idea of depth and not two.

| | Curious | Science | Engine |
|---|---|---|---|
| Cards | A sentence, four headline facts | Every value, set or derived, evidence stamps | Addresses, hashes, cost classes |
| Numbers | "about 15 °C", "365 days" | 288 K, 1 AU, honest precision | Full precision, units switchable |
| Instruments | The picture and one line on what it shows | Layers, tables, the model's limits | Recipes, budgets, flows, tiers |
| Why? | Opens at Curious | Opens at Science | Opens at Engine, "Open the flow" |
| Status line | Kept, seat | Adds device tier | Adds kernel, state and stack hashes |

Depth never hides an instrument, a scale or a desk. It only changes how much each one says. That is the line between learning and overwhelming: **all the doors are always visible, and each room starts tidy.**

A new visitor starts at Curious. The choice is remembered per device. A phone starts at Curious, as the handoff already says for Basic.

### 2.5 Find and Ask

- **Find** (key `/`): one box for worlds, places, instruments, tools and settings. Typing "photograph" offers "Take photograph, on the Globe". Typing "ocean worlds" runs a sifter and lists hits. It is the same `sift` mechanism Stories uses, so there is no second search.
- **Ask**: the agents, promoted from the status line to the bar. It opens next to the button with "What can I do here?", a short list made from the place, instrument and desk in view, and a box to type in. Tars answers from the help passages and the kernel's why? with no key, so Ask works for everyone on first run. Orac ("build it for me") and Minerva ("tell the story") appear in the same panel and ask for a key only when used.

### 2.6 Workshop

One door, in the More menu and at the foot of the rail, for everything that changes how the tool works: Physics flows and change requests, Packs, Player screens, Player data views, Symbols, Styles, Lenses, Sound and Light recipes. These are the five setup tabs plus Physics. Physics is also one click from any why? ("Open the flow") and from "The physics" on the rail, which opens the flows for the place in view, so the education path into the physics gets shorter, not longer.

More also holds Campaign (save, open, start fresh), Night chart, keyboard help and About this session.

### 2.7 Where everything goes

| Today | Proposed |
|---|---|
| Chart tab | System scale, Chart instrument |
| Body tab | World scale: Globe (with Take photograph and Light), Cutaway |
| Sizes tab | Sizes instrument at every scale; Measure on the stage strip |
| Life tab | Life instrument at world scale |
| Explore › Route | Route instrument (system), Route to here (world) |
| Explore › Window | Sky from here instrument, every scale |
| Explore › Galaxy, Cosmic web | The place line |
| Explore › Stories | Stories instrument; its sifters also power Find |
| Physics tab | Workshop; "The physics" on the rail; every why? |
| Screens, Player data, Symbols, Styles, Packs tabs | Workshop |
| Lens dropdown | Desk and depth; custom lenses in the desk menu |
| Edit/View | The desk |
| Chart room/Night chart switch | More menu, key N kept |
| What if, Look ahead | Buttons on the time ruler |
| Make a system | Start sheet; "New system" on the Build desk and at Local stars |
| Agents | Ask |
| Light, Sound | Stage strip |
| Campaign… | More menu; "Kept 11:02" stays in the status line |
| Share with players, seats, requests | Status line link as now; the whole table panel on the Run desk |
| Kernel version, hashes | Engine depth only |

## 3. Onboarding: layers, not a tutorial

### 3.1 The start sheet

The app opens on one chart-room sheet, not on a fixture and not in a wizard. It has three parts.

1. **Begin**: three doors. *Open the Solar System* (marked "start here"), *Grow a new system*, *Open or bring a campaign* (a campaign file, a V3 save, a Traveller sector). A returning user sees "Carry on: Kestrel Reach, kept today 11:02" above them and can set the app to skip the sheet.
2. **What it can do**: ten entries, laid out like an atlas contents page, each a verb: Age the Sun, Cut a world open, Take a photograph, See in other light, Follow life, Plan a trip, Look out of the window, Zoom all the way out, Ask why, Run a game. Each opens a two-minute tour on the real desk.
3. One line of reassurance: works in the browser, nothing to install, work is kept on this device.

The Solar System is the default first place because it is familiar, it exercises almost every instrument, and ageing it is Alex's own top wish. Kestrel stays as a sample campaign.

### 3.2 Tours are saved places

A **place mark** saves where, when, instrument, light, desk and depth. A **tour** is an ordered list of place marks, each with a caption of one or two sentences and, optionally, one thing to try ("drag the ruler to 7.6 billion years").

- A tour plays on the real desk. The caption sits beside the thing it describes and opens from it, as the interface style requires. Next, Back and Leave are always there. Nothing is blocked while a tour runs.
- Tours are data documents, so they ship in packs, Orac can draft one, and Tars can offer one as an answer.
- The same mechanism serves both audiences. For a learner, a tour is a lesson. For a GM, place marks are session prep ("the approach to Tarn at dusk") and a tour is a briefing to play on the players' screen. For support, a bug report can carry a place mark.

This is the one new mechanism in the design. Everything else is rearrangement.

### 3.3 Teaching in place

- **First visit to an instrument**: one line under its title saying what it shows, with "Show me (1 min)". It goes when dismissed or after the first real use, and "Show hints again" is in More.
- **Empty states teach**, as "Nothing selected" does today. Every empty state says what to do next and offers the one action that fills it.
- **Things that are not there yet say so**, in the app's honest voice ("Not yet modelled: flow and channels").
- **Ask is always there**, and "What can I do here?" needs no key.
- **Advanced things appear when they become relevant**, on the pattern seats already follow: the what-if stamp and Compare when a branch is open, the requests queue when a player asks, More seats after sharing.

### 3.4 Rules that hold the line

1. Every core feature has a visible, worded entry at Curious depth. Depth changes detail, never the list.
2. One new idea per screen on first run. A hint never stacks on another hint.
3. One primary action per instrument, drawn in solid ink. The rest are outlined.
4. No more than ten rail entries, three desks, three depths. If something does not fit, it belongs in Workshop or behind a card.
5. Words before numbers at Curious; the number is one tap away with its why.
6. Nothing is modal except a destructive confirmation.
7. No setting is needed before the first interesting thing happens. From a cold start to the Sun swelling into a red giant should be two clicks.

## 4. Phone

The same four controls, stacked: the place line as "‹ Sol · Earth", the desk as a three-part switch, the instrument rail as a row of chips that scrolls sideways and ends in "All 12 ›", the stage, the card as a sheet, the ruler at the foot. Depth, Find, Workshop and Campaign are in More. See board 4.

## 5. Suggested build order (for the coding threads, after Alex agrees)

Each step leaves the app usable. The first three are almost pure rearrangement of what exists.

1. **Shell frame**: place line (system and world only), instrument rail from the registry, stage strip, What if and Look ahead on the ruler, a slim status line, Workshop holding the six setup screens. No new features. This alone surfaces Window, Stories, Route, Cutaway, Take photograph, Light, Sound and Agents.
2. **Desks and depth**: Explore, Build, Run and Curious, Science, Engine as lens fields; Edit/View and the lens dropdown retire; cards gain the Curious prose form.
3. **Start sheet and the Solar System as the default place**, with Kestrel as a sample.
4. **Scale ladder upward**: Local stars, Galaxy and Cosmic web join the place line.
5. **Place marks and tours**, then the ten shipped tours.
6. **Find**, over places, instruments and sifters.
7. **Screens for engine-only work**, in the order that makes the best tours: Light and spectrum, Globe layers from `survey_surface`, Survey from afar, Notes and people on the card, Landing, the Run desk's table panel (triggers, programs, stock), nebulae on the wider charts, the Traveller hex map at Local stars.

## 6. Open questions, each with a recommendation

1. **Desk names.** Explore, Build, Run, or Learn, Make, Play, or something else? *Recommend Explore, Build, Run*: they suit both the astronomer and the GM, and Building and Running are already the lens names.
2. **Depth names.** Curious, Science, Engine (matching why?) or Basic, Advanced, Expert (matching the lens presets you named)? *Recommend Curious, Science, Engine*: one vocabulary for depth across the app, and "Curious" is kinder to a newcomer than "Basic".
3. **First place.** Solar System or Kestrel? *Recommend the Solar System* as the default, with Kestrel kept as a sample campaign. It does mean the Sol pack has to be good (for example the Sun cycle fix to 11 years).
4. **Start sheet every time, or only the first time?** *Recommend every time until a campaign exists*, then straight to the campaign with the sheet one click away on the "S" mark. A switch in More overrides either way.
5. **Should not-yet-screened instruments show on the rail?** *Recommend no in the product* (a dead entry teaches distrust), but yes in a "What is coming" list on the start sheet, since you want people to know what it is capable of. The hollow dots on the mock-ups are notes for you, not a proposed feature.
6. **Physics: Workshop or top level?** It is the heart of the education purpose. *Recommend Workshop plus "The physics" on every rail plus every why?*, which makes it reachable from more places than the tab did while keeping setup out of the main bar.
7. **Rail with words, or icons?** *Recommend words on desktop*, icons with words on hover only when the window is narrow. Words are what make the features visible.
8. **Tours as a general mechanism** (place marks, GM briefings, lessons, pack content)? *Recommend yes.* It is the only new mechanism here and it pays three times.

## 7. Not in this design

Player screens are untouched: themes, the screen composer and seats work as they do. Ground-level view, weather screens and the evolution screens will take rail entries when they exist. Night chart is unchanged apart from where its switch lives.

---

# Second pass, 21 Nov 2026

**Ruling (Alex, 21 Nov):** Explorers and the converter are "SSE native": they stay external tools with their own sites and repositories, and they integrate into SSE4 as needed. Sections 8 and 15 follow that.

Alex: bring the Explorers site in and integrate it better (maps now, and a home for physics packs, universe packs and star map packs); keep cut and paste of items and bodies from that site into SSE, under Build; onboarding with simple examples people can fiddle with, "encouraged to break it" and reset; an animated guide; where are undo and redo; the time scale at the bottom should default to what you are doing; how do the time sliders affect other systems and games; autopilots need a quick resolve over time jumps.

## 8. The Explorers library

Read from `C:\Development\starsystemx-creator-hub` (hub v0.56, `docs/contract-with-sse.md`, `docs/clips/`, `src/lib/openInSse.ts`, the R-20 map list note). What exists today, for V3:

- The site holds **campaigns** (star maps) and **single systems** as V3 bundles, with covers, tags, hearts, comments, credits and a moderation ledger keyed on content hashes.
- **Open in Star System Explorer** (a campaign) and **Add System to SSE** (a system, placed by the GM) are links built from a prefix the site keeps in its config.
- **Copy** on a body puts a text clip on the clipboard: the body, everything under it, and any custom rules it needs (liquids, engines, fuels). Pasting in SSE merges the rules, drops duplicates and reports what came with it.
- `GET /api/maps` gives a compact, sortable, taggable list with thumbnails, made for a panel inside the app. No account is needed to browse or download.

### 8.1 In the app

The library becomes a place in SSE4, called **Explorers**, reached from the start sheet (a fourth door, "Browse the Explorers library"), from More, and from **Add ▾** on the Build desk. It is the same list the website shows, read through `/api/maps`, drawn in the chart room style.

| Shelf | What is on it | What you can do |
|---|---|---|
| Campaigns and star maps | Whole campaigns, V3 and V4, labelled | Open to try (as an example you can break, section 9), or Open as my campaign |
| Systems | Single systems | Add to my campaign (pick the place on the Local stars chart), or Open to try |
| Bodies and things | Worlds, moons, ships, stations: anything that can be a clip | Copy, then paste on the Build desk |
| Universe packs | The Milky Way, a setting's galaxy and history | Stack it (campaign setup) |
| Physics packs | Alternative or extra physics: flows, atoms, class atlases | Stack it; the Why? of every changed value names the pack |
| Game packs | Traveller and later others | Stack it |
| Looks and tours | Themes, symbol sets, lenses, player screens, tours | Add to Workshop |

One shelf mechanism: every entry is a manifest with content hashes, a kind, a V3 or V4 label, a licence and a credit chain. That is what handoff 5 and build plan X2 already specify, widened from "manifests and packs" to every kind above. Packs are checked exactly as local packs are (one publisher per key, fictional labels, cost classes), and a physics pack from the library shows what it would change before it is stacked.

Publishing is in More › Campaign › Publish to Explorers, and on any pack in Workshop. It needs an Explorers account; browsing never does.

### 8.2 Copy and paste, kept

- **On the Build desk**, Copy and Cut work on any body, and take everything under it (a planet with its moons and stations). Paste places it where the pointer is, at the time on the ruler, as one authored event batch and one undo step. This is V3 parity row "cut, copy and paste anywhere".
- **From the Explorers site or the in-app shelf**, Copy puts the same clip on the clipboard and Paste in SSE4 takes it. A V3 clip goes through the V3 importer and shows its short report (what was kept, what was guessed, what was impossible). A V4 clip is authored facts plus the names of the packs it needs.
- **What a clip needs comes with it or is named.** V3 clips carry their custom rules; V4 clips name their packs, and Paste offers to stack a missing one, or to paste anyway with the affected values stamped IMPOSSIBLE and "make physically sound" on offer. Pasting the same thing twice never duplicates rules.
- **Credit travels.** The cartographer chain is a field on the authored events, shown on the card and carried forward if the result is published.
- Paste is refused on Explore, with one line saying to switch to Build. On Run it is allowed only for things from stock (a ship arriving), as an ordinary GM action.

### 8.3 The site itself

The website needs the V4 look or a skin to match (Alex). *Recommend a skin first*: the site has a central stylesheet (`src/app.css`), so the chart room tokens (ground, ink, one accent, the three typefaces, square corners, hard shadow) can go in there first, then page by page, without touching its moderation and account code, and the in-app shelf and the website then look like one product. A deeper rework can follow when V4 entries are common. The site is a separate repository from SSE4, so this is its own piece of work.

## 9. Examples you can break

Every tour and every "Open to try" opens as an **example**: the place is loaded on a branch, exactly the what-if mechanism already built, with a stamp on the stage that reads "EXAMPLE · change anything".

- All Build tools are armed, whatever desk the person was on, so they can drag Jupiter inward, make Earth twice as heavy or switch the Sun off.
- The consequences show at once, each with its why: "Mars's orbit is no longer stable · why?".
- **Reset** throws the branch away and reopens the example as it was. **Keep as my own** turns it into a campaign. Leaving asks nothing and loses nothing that mattered.
- Tours can end with a dare: "Now break it. How close can Jupiter come before Earth is lost?" This is the "left with a PhD" path: the physics teaches by being pushed.
- The ten shipped tours each run on a small example. *Recommend* the first three be: the Solar System (age it), one generated system of four worlds (move things), and one world (cut it open and change its air).

Nothing new is needed in the kernel: an example is a campaign file plus a branch.

## 10. The animated guide

A tour step can **show the gesture before asking for it**. An inked pointer, drawn in the chart's own line style, performs the drag or the click once on the real control, the chart responds for real, and the caption then says "Your turn". Replay is one click.

- It is a demonstration of a real action, so it obeys the interface style: motion explains a change of state, plays once, never loops, and can be interrupted by simply doing the thing.
- With reduced motion, or on a basic device, the same step is two still frames, before and after.
- Captions are in Tars's voice and follow his humour setting. *Recommend no mascot character*: an inked hand and a caption fit the chart room, and a cartoon guide would cut against it. Easy to revisit if you want a character.
- A step is data: target control, gesture, values, caption. So Orac can write a guide, and packs can ship them.

## 11. Undo and redo

They were only on the Build tool row in the first mock-ups, which was too hidden.

- **Undo and Redo sit at the top left of the stage, in the same spot on every screen.** They appear as soon as there is something to undo and stay dimmed when there is not. On Explore nothing can be changed, so they are absent, except in an example, where they are always shown beside Reset.
- Ctrl+Z and Ctrl+Shift+Z work everywhere, as now.
- Every change, including a paste or an accepted look-ahead, is one step.
- Each kept change is also an ink mark on the time ruler's history line. Undo steps back along it, so the ruler shows where you would return to. That is the handoff's "the ruler doubles as the history".

## 12. Time

### 12.1 What the ruler shows by default

The ruler opens at the span that suits the desk and the scale. Changing it is always one click, and the last choice is remembered per desk.

| Desk | At a world or system | At Local stars, the galaxy or the web |
|---|---|---|
| Run | Human time: days either side of the campaign's now, in the pack's calendar | Years |
| Explore | Human time, with "Whole life ›" beside it | The whole history |
| Build | The full ruler: birth to death of the place in view | The whole history |

A tour sets the span it needs (Age the Sun opens on the full ruler).

### 12.2 One clock, and what moving it does

This is the model I recommend. It differs slightly from what is built, and the difference is marked.

**There is one clock for the whole universe.** Universe time runs from the beginning to the far future (handoff 6.3). Every system, world and ship is a function of that one time. A star's ruler is not a separate timeline; it is the same clock, zoomed to that star's life, with the star's birth and death marked on it. Zoom out and you see the galaxy's history on the same line.

**Three different things can happen on the ruler, and they must look different:**

1. **Looking.** Dragging the ruler changes what you are looking at, nothing else. The system in view is worked out for that time. Other systems are not computed, but if you travel to one with the ruler still there, you see it at that time too. Players' screens do not move. Nothing is recorded.
2. **Looking ahead.** Look ahead runs the detailed future of the system in view (the engulfments, the orbit changes, labelled "one plausible future") on a branch. It shows only that system changing, because only that system has been worked out in detail. Nothing is kept unless you accept.
3. **Accepting.** There are two honest things "accept" can mean, and the dialog offers both by name:
   - **"Make this the campaign's now."** The clock moves for everything. Every other system is a billion years older too, and shows it when next looked at. Ships, people and programs are resolved or retired (section 13). This is the 4X and generational-saga move.
   - **"Make this system this old, today."** The campaign's now stays put. The system's birth is moved earlier instead, so that at today's date it has already lived that billion years. Only this system changes. This is the "I want a dying star next door to my present-day campaign" move, and it is what most GMs mean. If the new birth date is before the galaxy could have made such a star, it is allowed and stamped IMPOSSIBLE with the reason, as any other impossible edit.

**What is built today** is a third thing: a kept look-ahead writes a per-system "now" (`system.now_year`), so that one system runs on its own clock. *Recommend replacing that with the two choices above*, because a private clock per system has no clean answer to "what time is it for a ship that flies between them". This is a kernel change (T2's commit) and needs your yes before it goes to the kernel thread.

**Across games:** a campaign has exactly one now. Two campaigns in the same universe pack each have their own, and never affect each other. Players see only the campaign's now; the GM looking around in time is private.

**How far the ruler reaches:** the whole universe, Big Bang to heat death. On Build at a system it opens fitted to that star's life, and a small "Universe ›" at the end zooms out to the full line, where the star's life is one inked span among the galaxy's events.

## 13. Autopilots and ships over a time jump

Proposed for the game layer (track J). It belongs in `SSE4-record-logic-programs-design.md` once agreed.

A ship on a course does not need simulating: a course is a set of arcs and burns with a closed formula, so **where a ship is at any time on its current course is a look-up, not a calculation over the gap.** The cost is only in the decisions programs make along the way. So the rule is by size of jump, and the app says which one it used.

| Jump | What happens | Stamp |
|---|---|---|
| Up to the end of every ship's current leg (usually days) | Exact and instant. Positions from the course formula. No decisions were due. | none |
| Longer, while the number of program decisions fits the device's budget (days to a few years for a handful of ships) | Exact. Every decision runs in order: the patrol turns, the drone unloads, the pirate meets the freighter. Progress is shown and it can be cancelled. | none |
| Too long to run exactly (decades) | **Estimated.** Each program's loop is run exactly for a few cycles to measure its rates (ore per year, encounters per year, chance of loss), the totals are scaled to the gap with keyed dice, and each surviving ship is placed at a keyed point in its loop on the arrival date. | ESTIMATED, with the measured rates in its why |
| Beyond service lives (a template states one; centuries and more) | Ships and crews are retired with a note in the record ("lost to time", or a fate from the dice). Only physics carries on. | RETIRED |

- It is one general mechanism (measure a few cycles, then scale), not a rule per kind of program. It is deterministic: the same jump gives the same result on every device.
- The estimate is a documented fudge with a good reason, and goes in the fudge register.
- Before a long jump the dialog says what it will do: "90 years: 3 ships estimated, 2 retired, about 4 seconds". The GM can choose "run it exactly" when the device can afford it.
- Looking around in time (12.2, case 1) never runs programs. Ships are shown on their current course, or hidden past its end. Only moving the campaign's now resolves them.

## 14. Questions from this pass, each with a recommendation

1. **Accepting a look-ahead**: offer both "make this the campaign's now" and "make this system this old, today", and drop the per-system clock? *Recommend yes.* It needs a kernel change.
2. **Library name in the app**: Explorers, or Library? *Recommend Explorers*, the name people already know.
3. **Site look**: skin now, rework later? *Recommend yes.*
4. **Guide**: inked pointer and Tars's captions, no mascot? *Recommend yes.*
5. **Estimated jumps**: is "measure a few cycles and scale" acceptable for decades, with exact as an option? *Recommend yes.*
6. **Paste on Run** limited to stock items? *Recommend yes*, to keep the table safe from an accidental paste.

Build order additions (section 5): examples and Reset join step 5 with tours; the Explorers shelf (read-only, `/api/maps`, Open to try, Add system) comes after step 3; copy and paste with step 2's Build desk; publishing and pack shelves with build plan X2; the time changes in 12.2 and section 13 go to the kernel and game-layer threads.

## 15. Converters: one home, at convert.starsystemx.com

Alex, 21 Nov: SSE3 has importers for other software, replicated in the converter site (`C:\Development\starsystem-convert`, https://convert.starsystemx.com). Keep the import and export code there, use it from SSE4 through an interface instead of copying it, maintain it separately, give it the V4 look, and perhaps let it manage V3 to V4 conversion too.

What the converter is today (v0.2.0, read from its README and source): one page that converts between Universe Sandbox (`.ubox`), SpaceEngine (`.sc`, `.pak`) and SSE (`.json`, `.sse.zip`), with a real-sky import from star catalogues. It runs entirely in the browser ("a save never leaves the tab"), does no physics on purpose, and reports the losses that applied to each file. Its import code is a hand-kept copy of SSE3's, so fixes are already being made twice.

**Recommend yes, with one change of shape: a module, not a server API.**

- The converter publishes its engine as a versioned browser module (and the same code as an npm package). SSE4, the Explorers site and the converter's own page all load that one module. A file is still converted in the user's own tab, so the privacy promise holds, it works offline once loaded, and there is no upload service to run, secure or pay for. A server endpoint would break all three for no gain.
- The interface is small: `formats()`, `detect(file)`, `convert(file, to)` returning the result and the same loss report the page shows. SSE4 pins a version and its hash in the campaign manifest, so a re-import years later can say which converter made it.
- **SSE4 format becomes a fourth format in the converter.** Its output for V4 is authored facts only (what the file states), never derived values, which matches the converter's "no physics" rule and SSE4's own: the kernel works everything else out on arrival and tags what was guessed or impossible.
- **V3 to V4 lives there too, once.** SSE4 already has a V3 importer in the kernel (`sse-import-v3`, tested on three real saves). Rather than write a second one, the converter loads that same code as wasm. One implementation, hosted with the other formats. Pasting a V3 clip from Explorers uses the same path.
- **V4 to V3** reverses the earlier "one-way" ruling. *Recommend allowing it as a best-effort export in the converter only*, with a clear loss report (V3 cannot hold a V4 system's history, flows, light or record). SSE4 itself never promises a round trip.
- In the app this is one entry: **Bring** on the start sheet and **Add ▾ › Import a file** on the Build desk. Drop any supported file, see the report and a preview, then Open to try or Add to my campaign. Export is in More › Campaign › Export as.
- The converter keeps its own repository and release cycle, and takes the same chart room skin as the Explorers site, so the three sites read as one product.
- Traveller sector import stays where it is, in the Traveller pack, because it depends on that pack's code sheet.

Question 7 for section 14: module rather than server API, and V3 to V4 through the kernel's existing importer loaded by the converter? *Recommend yes to both.*

## 16. The opening line changes every visit

Alex, 21 Nov: the two tag lines on the start sheet should change each time, explain a little more about the software, and cover each core feature and use case. Push RPG use first, then learning, exploring for fun, worldbuilding, "no excuse for bad physics", "the universe is way weirder than you can do", and point to rarer uses (build a game on it, background screens for a film). A few expected ones to start, cycling through rarer ones later, to inspire uses and remind people of features they may not use. A few dozen.

### 16.1 How it behaves

- **It changes by itself (Alex, 21 Nov).** Nobody advances it by hand. A new line is chosen each time the start sheet opens, and while the sheet stays open the line gives way to the next with a slow fade about every twelve seconds. It holds still while the pointer or keyboard focus is on it, so "Show me" can always be reached, and with reduced motion it changes per visit only.
- **A line that names a feature links to it.** "Show me ›" opens the tour, example or screen. Lines that are only an idea (a use for a film, say) carry no link.
- **Order.** The first three visits show the three expected lines, role-playing first (lines 1 to 3). After that the pick is weighted: common lines often, rare ones seldom, never the same twice running, and a preference for lines about things this person has not opened yet. That preference uses a list kept on the device only; nothing is sent anywhere.
- **Honest.** A line is held back until the feature it names has a screen (the `needs` field). Every claim must be true of the build in front of the user.
- **Data, not code.** The lines live in one document (`start.lines.json`: id, headline, sentence, opens, weight, needs, audience tag). Packs can add their own, so the Traveller pack can greet a Traveller referee. Translating them later is a matter of translating one file.
- **Used twice.** The same lines feed "Have you tried…" at the foot of the Ask panel, one at a time, chosen for the place and desk in view.

### 16.2 Voice

Headline of eight words or fewer, as a question, a dare or a plain statement. One supporting sentence, concrete, naming what you will actually see. Second person. No sales words. British spelling.

### 16.3 The first set (42)

`○` marks a line that waits for a screen that is not built yet.

**Shown first, in this order**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 1 | Your sci-fi campaign, on real physics. | Build the systems your players fly through, then share a screen that shows only what they have found. | Run a game tour |
| 2 | A universe that works out its own physics. | Go anywhere, at any time, and ask why. | The Solar System |
| 3 | Come for the role-playing game. Leave with the astrophysics. | Every number opens to the physics behind it, and to the paper behind that. | Ask why tour |

**For the table**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 4 | Players see only what they have scanned. | You hold the whole chart. Their screen fills in as they explore. | Run desk, players' screen |
| 5 | How long to Mars, and will the crew survive it? | Courses compared by fuel, time, radiation dose and what they pass on the way. | Route |
| 6 | What does the sky look like from there? | Stand on any world and look up, through eyes, binoculars or a telescope. | Sky from here |
| 7 | Find tonight's story. | Ask for the places worth a session: a world about to lose its oceans, a moon nobody has surveyed. | Stories |
| 8 | Dress the bridge. | Give players a green-screen terminal, a glass cockpit or a paper chart, all showing the same system. | Player themes |
| 9 | A Traveller sector, turned into real systems. | Import the codes and get worlds whose physics agrees with them, with the ones that cannot flagged. | Traveller import |
| 10 | Run an empire game with real fog of war. | Each side sees only what it has discovered, on its own screen. | Seats |
| 11 | Give the pirates something to do. ○ | Patrols, raiders and drones carry on by their own rules while the players are elsewhere. | Run desk, programs |
| 12 | The same universe on every screen at the table. | No server holds your campaign. Each device works out the same result from the same seed. | Share with players |

**For worldbuilders**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 13 | Build a world. Then find out what it does. | Set a mass and an orbit. The air, the oceans and the seasons follow. | Build example |
| 14 | No excuse for bad physics. | Make your setting as strange as you like. Where it breaks the rules, it says so and tells you why. | Impossible stamp example |
| 15 | The universe is weirder than anything you would invent. | Tar seas, moons heated from inside, stars that swallow their planets. They turn up without being asked. | Stories, strange worlds |
| 16 | Grow a solar system from a cloud of gas. | Watch the worlds form, migrate and collide, with every stage of the history kept. | Grow a new system |
| 17 | Find a home. | Ask for an ocean world with one moon and a quiet star, and be shown where one could be. | Find |
| 18 | Design a planet nobody has thought of. | Place it on the class chart and see what kind of world that combination makes. | Class atlas |
| 19 | Writing a novel? Check the voyage. | How long it takes, what it costs in fuel, and what the crew sees on arrival. | Route and Sky from here |
| 20 | Borrow someone else's galaxy. | Open a campaign from the Explorers library, or take one world from it for your own. | Explorers |
| 21 | Bring your old maps. | Saves from version 3, Universe Sandbox and SpaceEngine come in with a report of what changed. | Bring |

**To learn**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 22 | Ten billion years on one slider. | Age the Sun and watch what happens to Earth. | Age the Sun tour |
| 23 | Cut a planet in half. | Core, mantle, crust, ocean and air, each with the reason it is there. | Cutaway |
| 24 | See it in X-rays. | Switch the light you look in and the same star becomes a different object. | Light |
| 25 | Why is this planet here at all? | Follow one world back through its history to the dust it formed from. | How it formed |
| 26 | Is anything alive down there? | How a living world got its air, and what lives on what. | Life |
| 27 | How big is it, really? | Put any worlds and stars side by side at true scale. | Sizes |
| 28 | What if Jupiter were closer? | Try it on a branch. Keep the result or throw it away. | What if example |
| 29 | Could we have found this planet from Earth? ○ | Try the real methods: the dip as it crosses its star, the wobble it gives it. | Survey from afar |
| 30 | Read a star's light. ○ | A spectrum of anything, with the lines labelled and what they tell you. | Light and spectrum |
| 31 | Change the physics. | Each rule is a small diagram you can open, edit and trace to its source. | The physics |
| 32 | Teach a class with it. | Make a tour of saved places with your own captions, and hand out the link. | Tours |
| 33 | Settle an argument. | What would Earth be like at the distance of Mars? Move it and see. | Build example |

**To play**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 34 | Just wander. | There is always another system, and nobody has looked at most of them. | Local stars |
| 35 | Break it. | Every example can be wrecked and put back with one button. | Any example |
| 36 | Zoom out until galaxies are dots. | From one world to the cosmic web, and back again. | Zoom out tour |
| 37 | Take its photograph. | A world in true colour, drawn from its own air and ground. | Globe |
| 38 | Listen to a solar system. | Orbits as sound, off until you ask for it. | Sound |
| 39 | Ask it anything. | Tars answers from the physics in front of you, and says where the answer came from. | Ask |
| 40 | Tell it what you want built. | Describe a screen, a lens or a rule in a sentence and Orac drafts it for you to check. | Ask, Orac |

**Rarer uses**

| # | Headline | Sentence | Opens |
|---|---|---|---|
| 41 | Build your own game on it. | The engine is open source and repeatable: the same seed gives the same universe on every device. | About, licence and source |
| 42 | Need a starship screen for your film? | Themed displays driven by a real system make convincing background screens. | Player themes |

Lines 1 to 3 are the "expected" ones. Weights: the table, worldbuilder and learn groups are common; play is medium; rarer uses are seldom. More are easy to add, and this list should grow with the features.

Answered 21 Nov: it changes by itself, with no manual control.

## 17. Decided, 21 Nov 2026 (Alex)

| Question | Ruling |
|---|---|
| Accepting a look-ahead (14.1) | Yes, simplify to the two named choices and drop the per-system clock. Plus the addition below. |
| Library name (14.2) | Explorers. |
| Site look (14.3) | Skin now, rework later. |
| Guide (14.4) | Yes: the box with the curved inked arrow on board 5, captions in Tars's voice, no cartoon character. He likes that it is minimal and disappears. |
| Estimated jumps (14.5) | Yes: measure a few cycles and scale for decades, exact as an option. |
| Paste on Run (14.6) | Yes, stock items only. |
| Opening line | Changes by itself, no manual control (section 16). |
| Explorers and the converter | "SSE native": external tools that integrate into SSE4 as needed. |

Also decided 21 Nov ("yes to both - make sense"): the converter is a browser module that SSE4 loads, not a server API; and V4 to V3 is allowed as a best-effort export in the converter with a loss report, which replaces the earlier one-way ruling (section 15).

### 17.1 "Make this system this old, today": the universe corrects itself

Alex: ageing one system to make it different probably means the star came from somewhere else. If the engine can work back a trajectory so the star could have been born at that age, do it: "we self correct the universe for a user tweak for their worldbuilding". Ancient stars beside young ones are possible.

So when a GM makes a system older (or younger) without moving the campaign's now:

1. The system's birth date moves, as in 12.2.
2. **The kernel then reconstructs an origin that fits.** Stars drift a long way from where they formed, and the galaxy track already gives every star an orbit through time (G5) and can solve a history around pinned facts (F9). The pins here are: where the star is now, how it is moving now, and its new age. The solve returns a plausible birthplace and path: for example "formed 9.1 billion years ago in the inner disc, 3 kpc nearer the centre, and has drifted out since". This is written to the system's history as reconstructed, with its why.
3. **Its make-up follows its age, with a choice.** An older star formed from gas with fewer heavy elements. The dialog offers "let its composition follow its new birthplace" (recommended; the planets are re-derived and the changes listed before accepting) or "keep its composition", which is allowed and marked as unusual for its age.
4. **IMPOSSIBLE is then rare.** It is stamped only when no origin fits: older than the galaxy could have made such a star, or a motion no path explains. The stamp says which.
5. The reconstruction is "one plausible past", labelled as such, deterministic, and replaced if the GM later pins a birthplace by hand.

This needs kernel work (a reverse solve on G5 orbits with age as a pin) and replaces T2's `system.now_year`. It goes to the kernel thread with this section as the brief.
