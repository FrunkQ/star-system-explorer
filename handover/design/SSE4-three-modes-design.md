# SSE4: every display on the three desks (track TM)

Status: **ACCEPTED by Alex 25 Nov 2026 13:30 UTC** ("All looks good. We are using those functions better now.. it will help with future UI decisions"): yes to all fifteen questions of section 13, so each recommendation there is a ruling (section 13A). Proposed the same day. Design and mockup only; nothing here is built by this note. Packages for builders are in section 12. TM is a code only, never shown in the product.

Mockup, clickable: project files `sse4/handover-screens/TM-screens/three-modes-mockup.html`, with screenshots beside it (the System, World and Galaxies nearby levels on each desk, and the Run desk on a phone). The pictures in the mockup are painted stand-ins, not SSE4 output; the mockup says so.

Alex's brief, in his words (25 Nov 2026 12:45 UTC, in the grand zoom thread): "Edit explore mode will have the time options and let you change things but the play would be a plainer view and kinda show this zoom as it is at each layer. So in Play the UI is a lot cleaner. Just visual controls rather than aging the universe or additional editing stuff. We need a design task to clearly outline what each display looks like under the 3 modes. Explore I see as a 'messing about' and exploring... what ifs. Edit is getting games set up and worldbuilding. Play is about keeping interfaces clean to support the GM at play time (and it will be the most visually arresting) - being able to use portions of this zoom to step up and down the hierarchy would be truly lovely - feel as if you are spanning the universe with a few clicks."

**Words.** Alex says Explore, Edit, Play. The product, the code and every accepted note say **Explore, Build, Run** (top-level design 2.3, his yes of 21 Nov). This note uses Explore, Build, Run and calls them desks; "mode" is his word for the same thing. Question 1 asked which names stand; his yes keeps **Explore, Build, Run**.

Read with: `SSE4-top-level-ui-and-onboarding.md` (the four controls in section 2, time in section 12), `SSE4-grand-zoom-design.md` (5.5 a stop, 5.6 the short form, 5.9 the three desks, 6.4 the gazetteer, 8A Motion), `SSE4-editing-design.md` (section 13), `SSE4-playbook-and-switchboard-design.md` (3.5, 4A, 5.3), `SSE4-secrets-and-visibility-design.md` (section 4), `SSE4-from-here-design.md` (section 10), `SSE4-atlas-and-reports-design.md` (11A.4, controls that switch), `interface-style.md`.

## 1. The short answer

1. **Today a desk arms verbs and nothing else.** That was the design ("the desk decides which verbs are armed, and nothing else", top-level 2.3) and it is what the code does. On all three desks the screen is the same: the same rail, the same ruler with What if and Look ahead, the same view controls, the same card. Section 2 lists the few places the desk is read.
2. **Alex now asks for more: the desk also decides what time can do and how much is on screen.** So a desk becomes three answers, the same three on every display:

| | Explore | Build | Run |
|---|---|---|---|
| **Where does a change go?** | Onto a what-if. Tried, not kept | Into the campaign. Kept, one undo step | Only the acts of play are possible: the clock, a reveal, a send, fuel, crew and cargo |
| **What can time do?** | Look anywhere, at any speed. Nothing is recorded | Look anywhere, and set dates: a birth, "from this date", the campaign's now | The campaign's now, moving in human time. No deep time |
| **How much is on screen?** | Every door, worded. Cards read as prose | Every door, plus tools, handles, the "Known to" eyes and the ledgers | The picture, the place line, the view controls, the clock and Send. A card only when a thing is pressed |

3. **One line each.** Explore: *try anything, nothing is kept.* Build: *what you change is the campaign.* Run: *the picture, the place and the clock.*
4. **It is one mechanism, and mostly one that exists.** Actions and handles already declare the desks they belong to. This note gives the same field to rail entries, the shell's own parts and each view's controls, so "what is on screen on this desk" is data read in one place (section 10). The Run desk is then the GM's own screen with most controls switched off, which is the same idea as the controls that switch on a player's screen (AT13).
5. **Stepping up and down the levels is the way to move on Run.** The place line shows all seven steps and each press travels by the grand zoom's short form. The mechanism is the grand zoom track's; this note says what is on screen at each step (section 4) and where the stepper sits (3.2). The seam was agreed by message on 25 Nov and is in section 9.
6. **Nothing on Run waits.** Above the system the Run desk reads the gazetteer (grand zoom 6.4): places and names worked out once at the campaign's date, placeholders for what has not been looked at. Heavy work (growing, ageing, working a galaxy's history out) belongs to Explore and Build.
7. **Standing rulings kept:** the five shared view controls on every level and every desk; depth (Curious, Science, Engine) on every desk; deep time is the GM's; one clock; players' screens are separate and are not a desk; no tab is added; nothing about players is added anywhere but the Switchboard.

## 2. What the desk changes today (read from main at 40901c2, 25 Nov 2026)

The shell (`packages/shell/src/App.svelte`) holds the desk and passes it to the stage and the views. It is read in these places and no others:

| Where | What the desk does today |
|---|---|
| The lens (`deskLensDefault`) | Explore opens the device's default lens, Build the Building lens, Run the Running lens. The lens chooses chart layers, card fields and tools. "See as" overrides it per desk |
| Edit or view (`DESK_MODE`) | Build is "edit": card values type, bodies drag. Explore and Run are "view" |
| Stage tools (`chart/Stage.svelte`) | Undo and Redo on Build, or while a what-if is open. Copy, Add here and Cut on Build |
| Handles (`views/editing/Handles.svelte`) | Move on Build or inside a what-if. On Explore a handle offers "Try changing it", which opens an example branch. None on Run |
| Object menu (`ui-contract/actions.ts`) | Each action lists its desks; the menu shows this desk's actions and one quiet line per other desk |
| Paste | Build only. Refused on Explore with a line. Stock items only on Run |
| "Known to" eyes and pencil (`known.ts`, card, chart, Ships and stations, Aboard, Your systems) | Build and Run, once a campaign is shared or something is hidden. Never on Explore |
| The on-air chip | In the top bar on Run; on the status line on the other desks |

Everything else is identical on the three desks: the place line, the full rail (Look, Find, This world, Travel, Understand, Table, the Workshop door), the five view controls, the whole ruler with What if, Look ahead, Light and Sound, the depth dial, See as, Ask, More, the status line, and every instrument's own buttons. The ruler does not yet open at a span chosen by desk (top-level 12.1 is designed, not built).

## 3. The frame on each desk

The frame is everything round the picture. It is the same at every level, so it is set out once here and the level tables in section 4 only say what differs.

### 3.1 Part by part

| Part | Today | Explore | Build | Run |
|---|---|---|---|---|
| **Place line** | Full, top left | As today | As today | **The stepper.** All seven steps always drawn; Out and In at its end; each press travels (3.2) |
| **Line under the name** (subtitle and date) | System's subtitle and the ruler's date | As today | As today | The campaign's date only, in the pack's calendar |
| **Desk switch** | Explore, Build, Run | Same | Same | Same. Always in view: it is the way out of Run |
| **See as** (lens) | Every lens, on every desk | Every lens | Every lens | Three kinds only: Game master, The table, each role. "What do they see?" is the one lens question asked at the table |
| **Depth** | Dial in the bar | Dial in the bar | Dial in the bar | In More. Run remembers its own depth and starts at Curious (question 9) |
| **Ask** | Bar | Bar | Bar | Bar. At the table Minerva's "describe this" is one press |
| **More** | Bar | As today | As today, with Copy, Cut, Paste, Add | Campaign, Workshop, Depth, Night chart, Help. No edit group |
| **Rail** | Open, every group | Open: Look, Find or This world, Travel, Understand, Table, the Workshop door | Open: the same, with the instruments that set a game up (5) | **Folded to one button** at the head of the view controls that names the instrument in view ("Globe ▾"). It opens the rail as a list in words with only the table's instruments (5). The picture gets the width |
| **View controls** | View style, Seen from, Scale, then the "HUD · Band" chip | As today | As today | As today: all five, same order, same words. Motion and Aids sit under the HUD chip where their notes put them |
| **Stage tools** | Select; Undo, Redo, Copy, Add here, Cut on Build; Bookmarks; Keyboard shortcuts | Select, Measure where the view has it, Bookmarks. Undo and Redo while a what-if is open | Select, Undo, Redo, Bookmarks, Copy, Add here, Cut, Measure | Bookmarks, Measure, **Send** (the one Send button the Playbook design leaves on the chart) |
| **Card** | Always there ("Nothing selected") | There; prose first; handles live as a try (6) | There; the handles row, boxes, the eye, notes and people | **Closed until a thing is pressed.** It opens from the thing and goes back to it (6) |
| **What-if stamp and dashed frame** | When a what-if is open | Appears by itself with the first change (question 3) | Only if the GM opens a what-if | Never |
| **Ruler** | The whole ruler; What if…, Look ahead | The ruler, opening on human time with "Whole life ›"; Play on the clock; Look ahead; the what-if's Keep it, Compare, Throw away | The full ruler, birth to death of the place in view, "Universe ›"; Look ahead with its two named accepts; "From this date"; the history marks that Undo steps along | **The clock** (7.3): one line. No ruler, no What if, no Look ahead |
| **Light and Sound** | On the ruler's side | As today | As today | Sound stays, beside the clock. Light is the Band control |
| **Status line** | Seat, Kept, the table's chip, engine line by depth | As today | As today | Kept only. The on-air chip is in the bar |
| **Honesty marks** | GM view | As today | As today | As today, on the card. Over the picture they are part of the HUD, so HUD off is a clean picture |
| **Fill the screen** | F, on a window the GM opened for a display | - | - | F on the Run desk itself: the picture alone. The bar comes back when the pointer goes to the top edge (question 11) |

### 3.2 The stepper

On Run the place line is the main way to move.

- **All seven steps are drawn:** Cosmic web, Galaxies nearby, Galaxy, Local stars, System, World, Site. The one in view is in the accent. Steps below the one in view are pencilled and named when the GM has been there (the trail the place line already keeps); otherwise they read as the level's word and a press opens the picker for that level.
- **A press on a step travels to it** by the short form of the grand zoom (about 0.6 s a level, grand zoom 5.6). A held press travels the long way with a stop at each level between (5.5). Reduced motion gives the plain fade.
- **Out and In** sit at the end of the line as two buttons with two keys (the builder proposes the keys; Page Up and Page Down are free). Out goes up one level. In goes down onto the thing selected, or onto the target the trail remembers.
- **A press on a thing in the picture opens its card. Going into it is the gesture that exists** (double press, or the card's "Open ›").
- **Zooming past the edge** (question 13): keep zooming out when the view is at its widest and, after a short hold so that it is deliberate, the view steps out a level. The same going in on the thing under the pointer.
- On Explore and Build the place line looks as it does today, and each step travels by the same short form. The stepper form (every step drawn, Out and In) is the Run desk's.

## 4. The seven levels

"Today" is the same on every desk unless it says otherwise. Explore, Build and Run say what is on screen *in addition to the frame* of section 3. Where a level's rebuilt view comes from the grand zoom note, the section is given.

### 4.1 Cosmic web

| | On screen |
|---|---|
| **Today** | The web slice as a chart, the cube in 3D and True (`views/foam-map`), a Density switch, the whole-history ruler. It opens on the origin, not on home |
| **Explore** | Centred on home (grand zoom 6.3). Density and the layers. The ruler over the whole history, and Play: the web growing. Press a knot, wall or void for its card and its why. Motion arrows under HUD: matter leaving voids, running along threads |
| **Build** | The same, plus: the names of knots, walls and voids typed over (they are gazetteer names, grand zoom 6.4); notes and bookmarks; the universe's date set here by the ruler's accept ("Make this the campaign's now"), which redoes the gazetteer with progress |
| **Run** | The picture, centred on home. With HUD on: home's knot ringed, names, a scale bar. Press for a card: the name, what it is, how far. Nothing moves and nothing is worked out: it is read from the gazetteer |

### 4.2 Galaxies nearby

| | On screen |
|---|---|
| **Today** | One 8 Mpc cube of haloes as a flat chart, circles by mass, stepped cube by cube; "Chart this galaxy"; Find a home with "Search nearby cells" (`views/cluster-map`) |
| **Explore** | The rebuilt view of grand zoom section 7: centred on home, two reaches (the neighbourhood and the group), Chart, 3D and True. Play runs our satellites along their orbits and the bound pair closing. "See our galaxy from there". "Make this home" as a try: the sky and the charts change on the what-if only |
| **Build** | Make this home, kept. Find a home (the search). Rename a galaxy. "Chart this galaxy" works a neighbour's history out and fills its gazetteer row. The catalogue disclosure under Chart |
| **Run** | The picture. Every galaxy has a name and a place at once; one not yet looked at is drawn as its type picture or a point (a placeholder), never a wait. Press for the card. No Play, no search, no catalogue |

### 4.3 Galaxy

| | On screen |
|---|---|
| **Today** | Galaxy chart: the rings through time, metals, stars, supernovae, where life could start, edge-on, scars, "Ink it" and "Back to the galaxy as grown" for an authored history (`views/galaxy-chart`). Galaxy 3D and True (`views/galaxy-3d`). Your systems (the campaign's systems on one map, with the flat home cell inset). Sector map when a game pack is stacked, with import and export |
| **Explore** | All of the charts and ledgers. The ruler over the whole history with Play: arms turning, mergers arriving, each leaving its dated mark. A merger tried as a what-if. Your systems and the Sector map to look at |
| **Build** | "Ink it": the authored history. Your systems as the place where a system is put, moved or found a slot (grand zoom 6.3 point 2); "Grow a system here". Sector import and export. Territory overlays drawn. The eyes on systems and links |
| **Run** | Galaxy in 3D or True with the campaign's systems marked, the party's place ringed, and routes. Your systems and the Sector map as the two other instruments. No rings, ledgers or history charts |

### 4.4 Local stars (a new step, grand zoom section 8)

| | On screen |
|---|---|
| **Today** | Not a level. The home cell is a flat inset inside Your systems; the voyages star map draws the same stars |
| **Explore** | The cell in 3D: every drawn star at its place, the campaign's systems filled. Press a star for its card. Play over thousands of years with Motion on: who is passing, who is coming (the passing-star question Alex raised). "Grow a system here" as a try |
| **Build** | Grow a system here, kept. Move a campaign system to another slot. Name a star. Draw and remove the links between systems. The pack's hex map edited where one is stacked |
| **Run** | Points of light with the campaign's systems named, links, the party's ship. Where a pack with a hex map is stacked, Chart style here is that map. Press for a card; In goes to the system |

### 4.5 System

| | On screen |
|---|---|
| **Today** | The chart (`views/system-chart`, `system-3d`): Chart, 3D and True, layers by lens, Measure, range rings, the formation replay. The card with handles (Build moves them; Explore offers "Try changing it"). The tool row and the ruler as in section 2 |
| **Explore** | The chart with every layer the lens gives. Measure. Handles and the chart's grips live as a try: drag a world and the verdict line says what follows, on the what-if. The stability verdict as advice. The ruler on human time with "Whole life ›", Play, Look ahead, the formation replay |
| **Build** | The tool row. Handles, boxes, grips, the ghost for adding and pasting, the tree for pairs. The eyes and pencil once the campaign is shared. Dated edits. The full ruler with the two named accepts |
| **Run** | The picture, in the style the Run desk remembers. Ships and the party marked. Range rings and Motion under HUD. Measure ("how far is it?" is asked at every table). The clock. Press a world for its card; In flies to it |

### 4.6 World

| | On screen |
|---|---|
| **Today** | The globe (`views/body`): True with layers, bands and Mag, the timeline with "Back to today", weather, Take photograph, Keep as postcard, the recipe editor ("Keep as my recipe"); the close-up down to 3 km. The card with handles |
| **Explore** | The globe with its layers (plates, rivers, climate, life), every band, the time-lapse. Take photograph, Make a postcard. Handles as a try: change the air, the seas, the tilt and see what follows. "Could be…" suggestions |
| **Build** | Handles and their boxes (Air, Liquids, Makeup, Orbit, Spin). Reshape the ground. Names on features. Landmarks and sites placed. The eye on each. The look recipe kept |
| **Run** | The globe, True with HUD off where the device can. Ships in orbit. With HUD on: the landmarks the table knows in ink, the rest in pencil. The clock (the terminator moves with it). The card on a press, with "Land ›". No layers panel, recipe editor or timeline |

### 4.7 Site

| | On screen |
|---|---|
| **Today** | Land (`views/surface`): pick a landmark or any spot, the kernel flies the descent ("Skip to the ground"), walk a patch a few km across, Time of day, Take photograph, Take off. The standing view's Aids, exposure stops and sounding line are designed and accepted (From here), not built |
| **Explore** | Pick anywhere, with the dangers shown. The descent with its odds. Time of day free. Exposure and the four Aids (off until asked for, remembered). Lamp. Take photograph, Make a postcard |
| **Build** | Place and name landmarks and sites. Reshape the ground here. Cave mouths. What is secret at this site. Notes and people |
| **Run** | The standing view, clean. Time of day follows the clock. Aids under the HUD chip as the GM left them. The bottom bar keeps turn, up and down, Take off; Send puts it on the table, and a landing is watched there as an event. Dangers give the odds; the dice decide |

## 5. The other instruments

A rail entry is on Run if it answers a question asked at the table: where are we, what is it like, how do we get there, what do they see. It is off Run if it explains, sifts or sets up. Off Run means not listed on the folded rail; Find and the desk switch still reach it. Each entry declares its desks as data (section 10), so moving one is a one-word change.

| Instrument | Today (every desk) | Explore | Build | Run |
|---|---|---|---|---|
| **Sizes** | Line-up, reference objects, Surprise me | Yes | Yes | Not listed |
| **Sky from here** | The sky from a world or place | Yes, any date | Yes | Yes, at the campaign's now |
| **Cutaway** | Shells from core to sky | Yes | Yes, with Makeup's handles | Yes, as a picture: no boxes |
| **Life** | Board, Story, the flow, "Back to the pack's values" | Yes; values changed as a try | Yes; values kept | Not listed |
| **Codes** (game pack) | The world in the pack's codes | Yes | Yes, "Write these codes" | Yes, read only |
| **Route**, **Route to here** | Plan a trip: time, fuel, dose | Yes, any date | Yes | Yes, from the campaign's now |
| **Voyages** | A voyage with stops; the flight log | Planned as a try | Planned and kept | Yes: plan, depart, arrive. The voyage moves the clock by the ordinary dialog |
| **Ships and stations** (list; Picture, plans, Aboard, Power and flow) | List, Add from stock, edit, "Take it out of play"; Aboard: Bring aboard, Load it, Top up, Move | Look at everything. Changes are a try (question 5) | The ship itself: hull, components, Remix, Add from stock, Take it out of play | Picture and plans to look at. Aboard is live: crew, cargo and fuel change and are kept. A ship arriving from stock (the paste ruling) |
| **Roll call**, **In memory** | Who was in space on any day | Yes | Yes | Not listed |
| **Stories** | Sift the campaign for the unusual | Yes | Yes | Not listed |
| **How it formed** (the replay, inside the chart) | Replay | Yes | Yes | No |
| **The physics** | The flows for the place in view | Yes | Yes | Not listed. "Why?" on a card still opens, without "Open the flow" |
| **Look up** | Constants, elements, units, names, classes | Yes | Yes | Not listed |
| **Atlas** | The book of the place, editions, print | Yes | Yes, with editions and print | Yes: it is what a GM reads from and sends |
| **Postcards** | The wall, Send | Yes | Yes | Not listed. A postcard is a source on the Switchboard |
| **Switchboard** and the **Playbook** | Rail group Table, every scale, every desk | Yes | Yes: where presets, roles and layouts are prepared | Yes, and the on-air chip opens it. "Open in its own window" for a second monitor |
| **Workshop** (Physics flows, Packs, Player screens, Player data, Symbols, Styles, Reference) | Door at the foot of the rail and in More | Door on the rail | Door on the rail | In More only |
| **Bookmarks** | Button after Redo, key B | Yes | Yes | Yes |
| **Explorers** | Start sheet, More, Add | Browse; "Open to try" | Add to my campaign, copy and paste, publish | No |
| **Start sheet** | Opens the app | A new visitor lands on Explore | "Carry on" returns to the desk last used | The "Run a game" tour opens here |

The Workshop changes the tool, not the universe, so it stays outside the three answers of section 1: its screens are armed on every desk, as now.

## 6. The card

One card for any object, as now. The desk changes what it leads with.

| | Explore | Build | Run |
|---|---|---|---|
| **Opens** | With the selection, as today | With the selection | Only on a press, from the thing; Esc or a press elsewhere closes it |
| **Leads with** | The sentence and headline facts (by depth) | The handles row, then the facts | The sentence and four facts, then what the table knows |
| **Handles** | Live. The first move opens a what-if by itself; the verdict line and "What followed" show on it. Keep it, Compare and Throw away are on the ruler, as now | Live and kept; Reach it, Pin it, Reset, "From this date" | None. Pack-declared live fields only (fuel, crew, cargo) |
| **"Known to" eye** | Never (the ruling) | Yes, once it matters | Yes: one press reveals. The lines the table does not know are in pencil |
| **Actions** | Look, Copy, Open in the Atlas, Send to the table if shared | Copy, Cut, Paste here, Add here, Take it out of play | Send, Reveal, Voyage from, Land on, Open ›. (The object menu's one-line "On Build: …" stays, as now) |
| **Why?** | Opens at the depth; "Open the flow" | The same | Opens at the depth, without "Open the flow" |
| **Notes and people** | Read | Write | Read and add a note (a thing said at the table) |

## 7. Time on each desk

One clock for the universe, as ruled. The desk decides what the GM can do to it. The three kinds of thing that can happen on the ruler (looking, looking ahead, accepting: top-level 12.2) map onto the desks cleanly, which they did not before.

### 7.1 Explore: look

- The ruler opens on human time at a system, world or site, with "Whole life ›" one press away, and on the whole history above the system (top-level 12.1, unchanged).
- Dragging it, or Play, changes what is looked at and nothing else. Nothing is recorded and players' screens do not move.
- Look ahead runs the system's detailed future on a branch. On Explore its result can be looked at and thrown away. Keeping it is a Build act: the button reads "Keep on Build…", switches desk and offers the two named accepts.
- Deep time above the system redoes the gazetteer for the date looked at, with progress, as a view. The campaign's own gazetteer is untouched.

### 7.2 Build: set

- The full ruler: birth to death of the place in view, "Universe ›" at its end.
- Everything Explore has, plus the acts that write: "Make this the campaign's now", "Make this system this old, today" (the two accepts of top-level 12.2 and 17.1), a birth date, "From this date" on a handle, a dated event.
- Each kept change is an ink mark on the ruler's history line and Undo steps along it.

### 7.3 Run: the clock

The ruler's place is taken by one line:

- **The date and time** in the pack's calendar, large.
- **Step chips**, from the pack's time units (by default + 1 hour, + 1 day, + 1 week), and **Run ▸** with a rate, which lets human time flow: worlds turn, ships move along their courses.
- **What is due**: the next few dated things as ticks on a short line to the right, a few days wide (an arrival, a window opening, a trigger waiting for a ruling). A press on a tick offers "Go to then".
- **"Jump to a date…"** for longer steps (weeks in jump space, a year of downtime). It is the ordinary jump with the dialog that says what it will do ("3 ships estimated, 2 retired, about 4 seconds", top-level section 13).
- **Not there:** deep time, What if, Look ahead, the history line. Ageing a star is one press on the desk switch away, and coming back to Run returns to the campaign's now.
- **Above the system** the clock shows the date and has no steps: nothing there moves in human time, and the Run desk reads the gazetteer.

So "deep time is a GM function" holds as ruled, and within the GM's screens it lives on Explore and Build.

## 8. Phone

The same three answers. The frame stacks as top-level section 4 has it: the place line as "‹ Sol · Earth", the desk as a three-part switch, the rail as chips. On Run the chips fold to the one instrument button, the place line's two rows become the stepper (the steps scroll sideways, Out and In are thumb-sized at the row's ends), the card is a sheet that comes up only on a press, and the clock is one row at the foot with the date and one step chip, the rest under its "…". A phone on Run is mostly a GM's remote for a bigger screen; it starts on Chart, as phones do.

## 9. Players' screens, the Switchboard and the grand zoom

- **Players' screens are not a desk.** They are separate pages made of parts, each with its controls switched on or off by the role's layout and kit (AT13, AT14), fed through the Switchboard. Nothing in this note changes them. "See as: The table" on Run is how the GM checks one from the chart.
- **Nothing about players is added outside the Switchboard** (Alex, 24 Nov). The Run desk's chart keeps exactly what that ruling left: one Send button and the on-air chip. The Playbook's presets are not put on the chart (question 14).
- **The same idea underneath.** A player's part with every control off is a picture. The Run desk is the GM's screen with most of its own controls off. Both read one kind of declaration (section 10), so a later "GM kit" from a pack is possible without new machinery. Not proposed for 4.0.
- **The seam with the grand zoom** (agreed by message with "Design the grand zoom", 25 Nov 2026; its note's 5.9 names this note's section 4 as the authority for what a desk shows):
  1. On Run a level is shown plain, and a stop on a journey is exactly that Run level, held. On Explore and Build a stop is that desk's level.
  2. The place line is the stepper: a press is the short zoom, a held press the long form with stops, "Go on" and "Back up" at its end. The mechanism (the way in, the seam, the camera) is the grand zoom's.
  3. The ladder has seven steps; Local stars and Site are new.
  4. Above the system the Run desk reads the gazetteer; time controls above the system belong to Explore and Build. Play on the clock for satellites at Galaxies nearby is Explore and Build only.
  5. "Make this home" and finding a system a place are a try on Explore and kept on Build.
  6. If Alex agrees to question 9, a journey's style defaults to the desk's remembered view style.
  7. A galaxy's place is being made one place at every level (GZ15), so no display here is designed round a jump between the web and Galaxies nearby.

## 10. The mechanism: one declaration

Today four things already say which desks they belong to: an action (`actions.ts`, `desks`), a handle (`handles.ts`, `desks`), the lens a desk opens with, and the secrecy marks (`eyesShow`). The rest is `if desk === …` in the shell and in six views. The proposal is to finish the job with the one field:

- **`desks`** (a list; left out means all three) on: a rail entry (`ScreenDoc.rail`), a part's control (`PartControl`, the AT13 shape, extended from players' parts to the GM's instruments), and the shell's own parts (a small table in ui-contract: ruler, clock, card, tools, status line, each with its desks).
- **One function** answers "is this on screen?": `shows(thing, desk)`. The shell and the views call it; no view tests the desk by name. A `tests/ui` check counts `desk ===` outside ui-contract the way the figures ledger counts `toFixed`: the count may go down, never up.
- **A desk's three answers are a small record**, `sse.desk/1`: `{ id, label, writes: "try" | "campaign" | "play", time: "look" | "set" | "clock", lens }`. The three shipped desks are three records. The desk switch reads its words from them, so question 1 is a change of data whichever way it goes.
- **Remembered per desk on the device** (question 9): the view choices (View style, Seen from, Scale, HUD, Band, Motion, Aids), the depth and the ruler's span. The key is the desk and the level.
- **A view declares once, in words, what each control is for**, and that list is also what the Reference "Parts" page and Orac read. Orac can then answer "what can I do here on Run?" from data.

Nothing here touches the kernel. Explore's try is the what-if branch that exists (`ws.branch`); Run's clock is the campaign's now that exists.

## 11. Standing rulings, checked

- **Shared view controls at every level:** on every desk, same order and words. Run folds nothing of them away.
- **Depth:** on every desk. On Explore and Build it never hides an instrument, a scale or a desk (top-level 2.4). On Run it is the desk, not the depth, that lists fewer instruments.
- **"All the doors are always visible"** (top-level 3.4 rule 1) holds on Explore, where everyone starts, and on Build. Run is the one place a GM chooses to see less; the desk switch is always in view.
- **Deep time is the GM's:** section 7.
- **Dynamic minimalism:** Run is the calm surface taken furthest. Nothing new is added to any desk; this note only says where existing things show. Things open from what they describe (the Run card). Motion: only the short zoom and the card's open, both existing kinds.
- **One mechanism over specials:** section 10.
- **Guide, never refuse:** on Explore nothing is refused, it is tried. An act that belongs to another desk is said in a line with the switch, as the object menu does now.
- **Secrets:** unchanged. Eyes on Build and Run, never on Explore, never on players' screens.
- **Players never hand-pilot; the GM streams only data; one clock; sound off by default:** untouched.
- **A new GM screen gives itself a rail entry, never a tab:** unchanged; the entry gains `desks`.
- **Changed by this note, each a question:** "Explore: nothing can be changed by accident" becomes "nothing can be *kept* by accident" (question 3); "Explore and Run may change manifest, crew and fuel" becomes Run and Build (question 5); "a capable PC still starts on Chart" stays for Explore and Build and changes for Run (question 10).

## 12. Packages

Each leaves the app usable. TM1 and TM2 are rearrangement; nothing new is drawn until TM4.

| Package | What | Owner (proposed) | Needs |
|---|---|---|---|
| **TM1** | The contract: `sse.desk/1` and the three records, `desks` on rail entries, shell parts and part controls, `shows()`, the per-desk memory keys, the desk-test ledger check. No screen changes | a new thread "Build the three desks" (Opus), with a note to the chart room | Alex's yes |
| **TM2** | The frame by desk (section 3.1): the shell's parts read `shows()`; Run's folded rail, bar, tools and status line; the rail entries given their desks (section 5) | "Build the three desks"; shared-shell changes one small PR at a time | TM1 |
| **TM3** | Run's clock (7.3) in place of the ruler; the ruler's span by desk (top-level 12.1); "Keep on Build…" for a look ahead on Explore | "Build the three desks", with the chart room for the ruler | TM2 |
| **TM4** | Explore tries at once: handles and grips live, the what-if opening by itself, paste and add as a try, Keep it and Throw away as now | "Build planet and star editing" | TM1 |
| **TM5** | The Run card: closed until pressed, opens from the thing, its lead and actions (section 6) | chart room | TM2 |
| **TM6** | Remembered per desk: view choices, depth, ruler span; Run's first-visit True on a capable device | "Build the three desks", with quality tiers for the probe | TM1 |
| **TM7** | The stepper on Run (3.2): all seven steps drawn, Out and In. Travels by a plain swap until the grand zoom's short form (GZ4) lands, then by that | "Build the three desks", with the selector thread and "Build travel between places" | TM2; GZ1 for the two new steps |
| **TM8** | The level pass: each level's and instrument's view declares its controls' desks (sections 4 and 5), one small PR per view owner | each view's owner thread | TM1 |
| **TM9** | Fill the screen on Run; the phone's Run frame (section 8); tours and the start sheet's lines checked against the new Explore | "Build the three desks", onboarding | TM2 |

Ships: the manifest change of question 5 is one line of TM8 for "Build ship manifest and crew roster".

## 13. Questions for Alex

1. **The names.** You wrote Explore, Edit, Play; the product says Explore, Build, Run (your yes of 21 Nov). *Recommend keeping Explore, Build, Run.* "Play" is already the word for running the clock and is one letter from "players", whose screens are a different thing; "Run" has neither problem, and "Build" covers setting a game up as well as editing a world. If you prefer your words it is a change of three labels (section 10) and I will rename the note.
2. **A desk is three answers** (where a change goes, what time can do, how much is on screen), declared as data, with one `desks` field on everything that can be on screen. *Recommend yes.*
3. **Explore tries at once.** On Explore every handle and grip is live; the first change opens a what-if by itself with the stamp and dashed frame, and nothing reaches the campaign unless "Keep it" is pressed. Paste and Add work there as a try too. This replaces the "Try changing it" button and "Paste works on the Build desk". *Recommend yes:* it is "messing about" with no step in the way, and nothing can be kept by accident.
4. **Run has a clock, not a ruler** (7.3): the date, step chips, Run ▸, what is due, "Jump to a date…". No deep time, What if or Look ahead on Run. *Recommend yes.*
5. **Manifest, crew and fuel are kept on Run and Build; on Explore they are a try like everything else.** This changes the ruling that Explore may change them. *Recommend yes,* so Explore has one rule with no exception.
6. **On Run the rail folds to one button** that names the instrument in view and opens the list in words. *Recommend yes.* It costs one extra press to change instrument and gives the picture the width.
7. **Which instruments Run lists** (section 5): Sky from here, Cutaway, Codes, Route, Voyages, Ships and stations, Atlas, Switchboard, and each level's own picture. Not listed: Sizes, Life, Stories, Roll call, The physics, Look up, Postcards. *Recommend yes;* each is one word of data to move.
8. **The Run card is closed until a thing is pressed.** *Recommend yes.*
9. **Each desk remembers its own view choices, depth and ruler span on the device.** So Run can sit on True with HUD off at Curious while Build sits on Chart at Science, and changing desk does not mean resetting five switches. *Recommend yes.*
10. **Run starts on True with HUD off on a device the speed probe places high enough, and on Chart elsewhere.** This changes "a capable PC still starts on Chart" for the Run desk only. *Recommend yes:* you want Run to be the most visually arresting, and the probe already knows which devices can.
11. **Fill the screen on Run** (F): the picture alone; the bar returns at the top edge. *Recommend yes.*
12. **The stepper** (3.2): on Run all seven steps of the place line are always drawn, with Out and In and two keys. *Recommend yes.*
13. **Zooming past the edge of a view steps a level,** after a short hold, on every desk. *Recommend yes,* built with the grand zoom's short form. It is the cheapest way to make the levels feel like one place.
14. **The Playbook's presets stay off the chart on Run.** One Send button and the on-air chip, as you ruled on 24 Nov; the Switchboard in its own window is the answer for a second monitor. *Recommend yes (no change).*
15. **For 4.0, in the order of section 12,** by a new builder thread "Build the three desks" on Opus, with TM4, TM5 and TM8 taken by the threads that own those files. *Recommend yes.*

## 13A. Alex's ruling, 25 Nov 2026 13:30 UTC

In his words: "All looks good. We are using those functions better now.. it will help with future UI decisions."

- **Yes to all 15**, so each recommendation in section 13 is a ruling. He gave no changes.
- **The names stay Explore, Build, Run** (question 1).
- **Three earlier rulings change, as section 11 lists:** Explore tries at once, so nothing can be *kept* by accident (question 3); manifest, crew and fuel are kept on Run and Build and are a try on Explore (question 5); the Run desk starts on True with HUD off on a capable device, while Explore and Build still start on Chart (question 10).
- **For future interface decisions** (his remark): a new screen or control says which desks it shows on, by the `desks` field of section 10, and is placed by the three answers of section 1. A design note for a new screen gives its Explore, Build and Run rows.
- **Builders may start.** "Build the three desks" (Opus) opens with TM1.

## 14. Picking this up without the chat

- State of the track: `handover/status/three-desks.md`. Nothing is built. The note is accepted (section 13A); TM1 is the first package.
- Section 2 was read from main at 40901c2 (25 Nov 2026). Check a file name before relying on it.
- The level rows marked with a grand zoom section describe views that track will build; this note only says what each desk shows of them. Where the two notes differ on what a desk shows, this one wins (agreed with that thread); on how travel works, that one wins.
- The mockup shows the frame and what comes and goes per desk on painted pictures. It is there for the layout of the Run desk and the three-way comparison, not for how any level will look.
