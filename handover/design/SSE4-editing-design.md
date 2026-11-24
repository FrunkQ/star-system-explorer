# SSE4 editing: handles, strips and the chart (track ED)

Status: **accepted by Alex, 24 Nov 2026 15:21** ("apart from those notes YES to all"; section 10B is the Solar System experiment he asked for, on paper, with eleven findings folded in). Earlier the same day, 15:13: (yes to 1 to 11, 14 to 22, 24, 26, 28, 31 onwards; 12 dropped; 25 and 27 changed; 13, 23, 29 and 30 explained again and waiting; see section 16). History: proposed 24 Nov 2026; Alex on first read, 14:42: "love this! what about atmospheres on the sliders? water -> liquids? SSE3 is our starting point there" (section 11A answers that). Then: makeup "is surely more than 1 slider now we have complex internal makeups"; how do you put in a subsurface ocean; "think through... what a worldbuilder may want to do to their world to make it right for their universe. We let them and point out things that break physics (kept to GM)"; "look at ALL the editors under settings"; "we *should suggest*: this world *could* be human habitable with an atmosphere... and that in turn may need a mag field to protect it... and a user can do one and not the other"; alien compatibility hooked in now, built in 4.1; and "we guide better now rather than leave them alone" (sections 11B to 11E); and "orbit is also more than 1 slider... eccentricity, where on the orbit, inclination, rotation, retrograde, oblateness, etc" (section 11F); "Users must ALWAYS be able to type an entry: it's how they enter real data" (section 3A); and binaries: "make sure we are coping with binaries properly... Pluto/Charon should run on a barycentre... maybe we can just always assume it... adding and removing bodies needs to be well managed chaos" (section 10A). And: SSE3's "editing was a bit all over the shop as it evolved, but now we can adopt a consistent and extensible look and feel" (section 3B). And a units engine: "clicking on a unit changes it... (k)m -> miles -> AU (within each unit will handle kilo mega giga peta... picks the most appropriate and adds exponents as needed). Have a consistent function (maybe editable) that expresses values in the most relatable and readable way", remembered per class of object (section 3C). Waiting on his answers to the questions in section 16.**
Mockup: `/mnt/project-files/sse4/handover-screens/ED-screens/editing-mockup.html` (it works: drag the strip, drag the world on the chart). Screenshots beside it in `ED-screens/`: `desk-1-mass-strip`, `desk-2-what-followed`, `desk-3-drag-orbit`, `desk-4-star-kinds`, `desk-5-companion-night`, `desk-6-moon`, `phone-1-mass-strip`, `phone-2-drag-orbit-night`, `phone-3-star-kinds`, and for section 11A `desk-7-air`, `desk-8-liquids`, `phone-4-liquids-night`, and for 11B and 11C `desk-9-makeup`, `desk-10-could-be`, `phone-5-could-be`, and for 11F `desk-11-orbit-box`, `desk-12-spin-box`, and for 10A `desk-13-family-tree`. The mockup's figures are stand-ins, not kernel output.

Asked for by Alex, 24 Nov: "Editing planets is a BIG need... Earlier versions went big into sliders, although we really like them because it encourages people to play with planets and know the range that they can play in. They do take up a lot of room, are relatively cumbersome, and the act of sliding them does tend to slam the game engine. We still want to encourage play... there's probably not that many things they can change now... generally there's only going to be a few sliders that matter. What we need to do is ensure when the user sits there and kind of goes, I really want a different type of star or something like this in the system, that they can add it and have a bit more control and are just allowed to do what they want and are guided if it's unfeasible." And a minute later: "dragging planets to move them is a great way to avoid a slider and do it fine... but consider moons and cut and paste, etc. We got a good set of tools in SSE3... maybe some cumbersome or awkward, so now is the time to fine tune and improve."

This note builds on what is planned and does not restate it: V3 gaps 1 (where things can orbit), 2 (Add here, built), 3 (grow with the four dials), 4 (pin a value), 5 (click a unit), 11 (mix editor), 12 (pin a class), 38 (press-and-hold menu), copy and paste (U20, built), the stability verdict (ST1, built) and "the science expects" for air. It gives them one shared surface.

## 1. The idea in six lines

1. **A handful of handles.** In SSE4 a world has about six things you can truly set, and a star has three. Everything else is worked out. The card shows the handles as one short row; nothing else on the card is a slider. **Alex 24 Nov 15:13: yes.**
2. **The slider lives inside the number.** Press a handle and a **strip** opens from it: a range line with the feel of a slider, which shows where you are, where the physics would put you, where real worlds sit, and the exact points where something changes ("oceans boil here", "keeps hydrogen from here", "this is a star from here"). One strip open at a time. Closed, it takes no room. **Alex 24 Nov 15:13: yes.**
3. **The chart is the first editor.** Orbits are changed by dragging the world. Moons the same, one level down. Adding, pasting and moving a body to a new parent all use the same ghost that follows the pointer and says how it would fare before you let go. **Alex 24 Nov 15:13: yes.**
4. **Dragging is free.** While you move, nothing is written and the heavy physics does not run. The strip was sampled once when it opened, so the figures follow your hand from that. Letting go makes one change, one undo step, and the card shows **what followed**. **Alex 24 Nov 15:13: yes.**
5. **Never refused, always told.** Every limit is a labelled mark you can pass, never a stop. A one-line verdict says what would happen and offers one way to make it work. **Alex 24 Nov 15:13: yes.**
6. **For any result you want, two honest routes.** **Reach it** (change an input until the physics gives it, and it stays real) or **Pin it** (force it, and it carries the honesty mark). The app offers Reach it first. **Alex 24 Nov 15:13: yes.**

## 2. What is truly an input

Read from the kernel on `main`, 24 Nov (`sse-formation` manifest, inheritance and star pins, `WorldStart`; `sse-kernel/src/adopt.rs` `spec_from_facts` and `bodies_from_facts`; the authored fields used by Add here and the clipboard).

**A system** (set once, on the system's own card):

| Handle | Field today | What follows from it |
|---|---|---|
| Age | `system.age` (`set_system_age`) | every star's phase, size and light; every world's cooling, air loss, field |
| Metals | `star.metallicity_z` (formation pin `feh`) | star life and colour; how much solid there was to build worlds |
| Birth chemistry, at Science depth | formation pins `c_o`, `mg_si`, `al26_rel`, `cluster_members`, `fuv_g0` | what worlds are made of, how wet, how the disc ended |

**A star:** one handle of its own, **Mass** (`body.mass`, formation pin `primary_mass`). With the system's Age and Metals that is the whole star: radius, temperature, luminosity, colour, phase, lifetime, activity and class all come out of the stellar fits. A companion star adds its orbit.

**A world or moon:**

| Handle | Fields today | Notes |
|---|---|---|
| Orbit | `orbit.semi_major_axis`, `orbit.eccentricity`, `orbit.host`; inclination and the three angles at Science depth | dragged on the chart first |
| Mass | `body.mass` | |
| Makeup | `makeup.metal`, `.rock`, `.carbon`, `.ice`, `.gas` (shares); at Science depth heat (`k40_rel`, `r_process_rel`) and how oxidised (`redox_iw`) | one box with a bulk bar and a cutaway, section 11B |
| Liquids | `body.surface_water_oceans` today; any substance in section 11A | how much of each; which are liquid, and how much of the surface they cover, are results |
| Air | `atmosphere.surface_pressure`, `atmosphere.mix` | how much and what of; "the science expects" beside it; section 11A |
| Spin | `body.rotation_period`, `body.obliquity` | day length and tilt; expected values come from tides and history |

**Results, not handles** (all are V3 sliders today): radius, density, gravity, escape speed, year, surface temperature, albedo, greenhouse warming, tidal and inner heat, magnetic field, ocean coverage, class, a star's radius, temperature, luminosity and spectral type. To change one of these you use section 8.

Two honest notes on the kernel as it is:
- A world added or imported by hand carries mass **and** radius, both typed (`bodies_from_facts` reads both; "Something else" in Add here asks for both). A world that formation made gets its radius from its mass and makeup. This design wants the second behaviour for every world (question 3).
- A hand-added world gets a fixed core share (0.32) and no water share. Makeup needs to reach `bodies_from_facts` in full before the Makeup handle is honest for hand-made worlds (kernel ask K3).

So the row is 3 handles for a system, 1 for a star, 6 for a world. **A handle is a subject, not a single number** (Alex, 24 Nov: makeup and orbit are each "more than 1 slider"). Each opens a box with a few parts, set out in sections 11A, 11B and 11F: about twenty true inputs for a world in all, grouped six ways, with the commonest part of each on top. V3 had about 27 sliders across ten tabs for one body, most of them results.

## 3. The handle and its strip

A **handle** is an input shown as a figure with a faint underline on the Build desk. The card's first row on Build is the handles alone:

`Orbit 0.52 AU · Mass 0.82 M⊕ · Makeup rock and iron · Liquids 1.1 oceans · Air 1.1 bar · Day 31 h, tilt 23°`

Three ways to move one, all the same mechanism underneath:

- **Drag the figure sideways.** The figure is the slider. The strip opens under it as you start.
- **Press it.** The strip opens from the figure (interface rule 6) and stays until you press elsewhere. Drag the mark, press anywhere on the line, or use the arrow keys (Shift for bigger steps).
- **Type.** Any unit of that kind ("1.2 AU", "180 million km", "300 d" for an orbit: a period is turned into a distance). This is today's edit in place.

**What the strip draws**, on one line, usually on a log scale because these inputs span many decades:

| Mark | What it is | Where it comes from |
|---|---|---|
| The mark you hold | your value, in the accent colour | |
| Open diamond | **the science expects**: what the physics would give if you had not set it | the resolve with this field left unset |
| Tipping points | short labelled ticks where a result changes its label: "oceans boil", "air is lost", "keeps hydrogen", "torn apart by tides", "fusion starts" | the sweep (section 6): found by the physics, never typed |
| Bands | usual, unusual, unheard of (V3's banded slider, wording kept: "possible, just unlikely") | formation samples at this spot, and catalogues |
| Reference dots | Mars, Earth, Neptune and the like at Curious depth; this system's other worlds at any depth | the data sheet (`base-reference`) and the system |
| Walls | real limits: the star's surface, the Roche limit, the edge of the Hill sphere, break-up spin | the stability and tide blocks |

Under the line sits the **verdict**, one sentence in plain words, worst news first, with at most one offered fix. Examples: "Lasts. Surface about 255 K, thin ice at the poles." "Inside the Roche limit: tides pull this apart in about 40,000 years. *Move to 0.011 AU, where it holds.*"

A strip's small menu (the rare things, in context): Reset to what the science expects, Change units (gap 5), "From this date" (question 6), Why? (the flow behind it), and at Engine depth the field name and the blocks it feeds.

**Why this beats a row of sliders.** It takes no room until wanted. It shows the playable range better than V3 did, because the marks are consequences and not just two end numbers. Its range is the real one for this body in this place, so the interesting part is never squeezed into a few pixels. And there is one of it, so a phone can give it the whole width.

**Handles are data.** Each handle is a row in a small handle sheet in the base pack: the field, its name at each depth, its quantity kind, log or linear, the block that gives its range and walls, its reference marks, and which desks may move it. A pack can add a handle for a field of its own, Orac can read the sheet to know what it may set, and the same strip then works for every numeric input in SSE4 (section 12).

## 3A. Typing is always there

Alex, 24 Nov: "Users must ALWAYS be able to type an entry: it's how they enter real data. Eyeball is great for experimenting but some users like to bring in other work." A standing rule for every editing surface:

- **Every figure that can be dragged can be typed,** in place, with one press. That includes every part of every box (shape, lean, tilt, each gas, each makeup share), not only the six on the row. A grip on the chart always has a figure that moves with it, and that figure takes typing.
- **Typed values are taken exactly as typed.** No magnetism, no rounding to the strip's steps, full precision kept and shown ("0.01671022"). Any unit of the right kind is accepted and converted ("149.6 million km", "365.256 d" for an orbit's size, "23d 26m" for an angle).
- **Past the end of a strip is fine.** A strip's range is where play is interesting, not a limit on entry. A typed value beyond it stretches the strip to include it.
- **The sheet.** Each body has "Type it all in": one plain table of every input it has, by handle, with its unit, and beside each the value the physics would give. Tab moves down it. It takes a pasted row or column from a spreadsheet or a paper's table, shows what it understood before applying, and applies as one change. It is where catalogue data goes in, and it can mark the entry "catalogued" with its source (the evidence kind the kernel already has for the start sheet's Sol), which is distinct from authored.
- **Typing gets the same guidance as dragging:** the verdict, tipping points and "what followed" appear for a typed value exactly as for a dragged one, after Enter.

## 3B. One editing grammar

SSE3's editors grew one at a time: ten tabs on a body, each laid out its own way; sliders of four kinds (plain, banded, log, dual); locks on some values and not others; an Overrides tab apart from the values it overrode; seven definition editors under Settings, each its own dialog; a right-click menu with its own list of actions. SSE4 has the chance to make all editing out of a few parts that always look and behave the same. These are the parts, and nothing in this note uses anything else.

| Part | What it is | Always |
|---|---|---|
| **Handle** | a subject you can set, as a chip on the row | label above, value below, inked underline when you have set it |
| **Box** | what a handle opens, under the row | most-used part on top; one box open at a time; a verdict line; a menu row last |
| **Strip** | a number on a line | your mark in the accent; the diamond for what the science expects; ticks where something changes; green for usual; hatching for a neighbour's reach; the figure top right, typeable |
| **Bar** | shares of a whole (air, makeup, any mixture) | drag a join; always sums to the whole; small shares become chips |
| **Ring** | an angle or a place round a circle | drag the dot; catches named in words |
| **Chips** | named choices and shortcuts (Kinds, "in step with", "locked") | a chip sets handles and says what it set; never a hidden mode |
| **Verdict** | one sentence under any control | worst news first, plain words, at most one offered fix as a link |
| **Menu row** | the rare things for this control | Reset first, Why? last, the unit and "From this date" between |
| **Grip** | a control on the chart itself | one grip changes one thing; a readout beside the pointer; the box's figure moves with it |
| **Ghost** | a thing not yet placed | follows the pointer, carries a verdict, placed with a press, Esc cancels |
| **Tree** | who goes round what | brackets round pairs; before and after for any structural edit |
| **Recipe** | steps toward something a world could be | each step separate, with what it leans on and what skipping costs |
| **Pencil and ink** | pencil (muted italic) is a preview; ink is settled | nothing in pencil is saved |
| **Sheet** | every input as a typeable table | paste accepted; the physics' own value beside each |

How it stays consistent as SSE4 grows:

- **A new editor is data, not a new screen.** A pack declares handles, their parts, and which part type each uses (strip, bar, ring, chips) in the handle sheet. The shell draws them. A ship's drive, a calendar, a gas on the data sheet, a tech level and a weather setting all get the same boxes with no new component.
- **The same words everywhere:** Reset, Why?, Kinds, Could be, Reach it, Pin it, From this date, What followed, What does not add up. Each is defined once in this note.
- **The same three colours of meaning:** accent for what you set and are holding, green for where the physics is comfortable, the warm warning tone for strain. Honesty marks stay on the GM view only.
- **A check, like the walk check:** `tests/ui` gains an editing-grammar check that fails a pull request which adds a slider, dial or dialog for a value outside these parts.

## 3C. One units engine: every figure said the same way

Alex, 24 Nov. This finishes what is planned in pieces: the Units table on the data sheet (MP1c, built), unit schemes and a setting's own units (MP15, designed), and click a unit (gap 5, planned). What was missing is the one function that turns a value into words, so that every screen says a figure the same way.

**One function, used everywhere.** `say(value, kind, about)` takes a value in SI (all the kernel ever holds), the kind of quantity (length, mass, time span, temperature, pressure, speed, angle, and so on) and what it is about (a planet's orbit, a moon's orbit, a world's size, a star's size, a gap between stars, a ship's range), and gives back the text. Cards, strips, the chart readout, the sheet, "what followed", the players' pages and the agents all call it. No screen formats its own numbers; the editing-grammar check (3B) fails one that does.

**Families and rungs.** Each kind has **families** of units, and each family is a ladder of **rungs**. Alex's rule (24 Nov 14:56): "use SENSIBLE units, scientific ones that are established for that type of value", across every kind (densities, volumes, masses and the rest), and "we entertain imperial but avoid it". So there are three families and no more unless a setting adds its own:

- **Scientific** (the default): the unit the field itself uses for that kind of thing.
- **SI**: plain metric with prefixes, for anyone who wants it straight.
- **Imperial**: miles, feet, pounds, °F. It is there for those who want it, it is last in the click order, it is never a default in any scheme we ship, and the app never shows it unasked. **A ton is a tonne** (1,000 kg): there is one "ton", spelt "tonnes", in every family; no short or long tons.

The established unit depends on what the value is about, which is why "about" is an argument:

| Kind | Scientific (by what it is about) | SI | Imperial |
|---|---|---|---|
| Length | world size: km or Earth radii; giant: Jupiter radii; star: Sun radii; orbit: AU; moon's orbit: thousand km or planet radii; between stars: light years or parsecs; landform: m, km | m, km | feet, miles |
| Mass | rocky world: Earth masses; giant: Jupiter masses; star: Sun masses; small body: kg with exponent; ship, cargo: tonnes | kg, tonnes | pounds, tonnes |
| Density | g/cm³ (water is 1) | kg/m³ | lb/ft³ |
| Volume | world: Earth volumes or km³; sea: km³; tank, hold: m³, litres | m³ | cubic feet, gallons |
| Area | km²; a world's surface as a share of Earth's | m², km² | square miles |
| Gravity | g (Earth is 1), m/s² beside it | m/s² | g |
| Pressure | air: bar, millibar; inside a world: GPa | Pa | psi |
| Temperature | K; °C for places people stand | K | °F |
| Time span | s, hours, days, years, Myr, Gyr; local days and years | s | the same |
| Speed | km/s; m/s on a surface; share of light speed when fast | m/s | mph |
| Energy, power | J, W with prefixes; star: Sun luminosities; blast: tonnes of TNT | J, W | the same |
| Light received | Earth's sunlight = 1, W/m² beside it | W/m² | the same |
| Magnetic field | gauss or microtesla for worlds, tesla for stars and machines | T | the same |
| Angle | degrees | degrees | degrees |
| Amounts of stuff | Earth oceans for water; bar-equivalent for air; kg with exponent; share by mass in % or ppm | kg | pounds, tonnes |

**Reference bodies are rungs you can click through** (Alex, 24 Nov 15:00). Each scale class has its own comparison body first and its neighbours after: a moon defaults to Moon radii and Moon masses, then Earth's; a planet to Earth's; a giant to Jupiter's, then Earth's; a star to the Sun's, then Jupiter's for the smallest. So Jupiter can still be read as 318 Earth masses, or in gigatonnes with an exponent, by clicking on.

**Do not climb the prefix ladder where nobody does** (Alex: "if you show in km, no need for m or megametres"). Each unit lists the prefixes people really use with it. Distances in space stay in km, with "thousand", "million" or an exponent as they grow, and never become megametres or gigametres; metres appear only for things you could stand beside. Tonnes take kilo, mega and giga. Watts, joules and pascals take the full ladder.

This table is the first cut of the "How figures are said" data, to be checked line by line against the sheet's Units table when ED1a is built; where a field has two established units (gauss and tesla) both are rungs of the Scientific family.

- **Clicking the unit steps to the next family** (Scientific, SI, Imperial, the setting's if it has one, and round again); a long press lists them. This is gap 5's click.
- **Inside a family the engine picks the rung by itself,** so the number reads comfortably (by default between 0.1 and 1,000): 384 thousand km, then 1.2 million km, then 0.52 AU as a value grows. A little stickiness at each boundary stops it flickering, and during one drag the rung is held so the figure does not jump; it re-picks on release.
- **Prefixes and exponents, as the depth suits.** Curious says "million km" in words. Science may use the SI prefixes where they are familiar (km, MW, GPa) and words otherwise. Engine, and any value no rung fits, gets an exponent: 3.2 × 10¹⁹ kg. Kilo to quetta and milli to quecto are all known to the engine (so any of them can be typed), but it only writes the ones listed for that unit.
- **The scientific units are also the relatable ones** (Earths, Suns, AU, g), so they are the default at every depth: a world is 0.82 Earth masses before it is 4.9 × 10²⁴ kg. A figure's peek adds a plain comparison where the sheet has one: "0.38 g, like Mars".

**Remembered for that class of object.** Alex, 24 Nov 14:58: a class here is a standard *scale* of thing, not each object: "stars will remember what the user last used on a star, planets, moons, constructs... megaconstructs and constructs are different classes". So the choice is kept per kind of quantity and per scale class, and applies to every object of that class at once. The scale classes are a short list on the "How figures are said" table: star, giant planet, planet, moon, small body, ring or belt, construct (ships and stations), megaconstruct, surface site, system, stellar neighbourhood, galaxy. A pack can add one. Each class also carries its own default scientific units (the "about" column above), which is what makes a megaconstruct read in thousand km and a ship in metres without anyone choosing. Choose AU for planets' orbits and thousand km for moons' orbits and both stick, for every planet and every moon. It is a preference saved with the campaign as an override on its unit scheme (Astronomer's, SI, Everyday, Traveller's, or the setting's own). Player seats inherit the GM's unless their lens chooses otherwise. "Use the scheme's" puts one back.

**Honest precision.** The number of figures comes from the value, not the screen: a computed value shows three, a catalogued one what its source gave, a typed one exactly what was typed. Curious adds "about". Nothing gains digits by being converted.

**The rules are data, and editable.** Reference gains "How figures are said" beside Units: for each kind and what it is about, the families, their rungs, where each rung takes over, and the words used. A scheme is a named set of these. A setting can add a unit ("klick", "standard day"), a family, or change where "million km" gives way to AU. The engine is one small pure function over that table, so the same value always reads the same on every device.

**Typing uses the same table backwards.** Anything `say` can write, the entry box can read, plus any other unit of the kind, prefixes and words included ("1.2 Gm", "750 thousand miles", "0.005 AU").

**Kinds that need care:** time spans climb seconds, minutes, hours, days, years, thousand, million and billion years, and have rungs for the world's own days and years ("117 local days"), tied to its calendar. Temperature converts with an offset (K, °C, °F). Compound units (km/s, g/cm³, W/m²) are built from their parts so a family change is consistent. Angles offer degrees or degrees, minutes and seconds. **Counted measures** (jumps, per the matter pack's 7A.8) are never converted and never get a prefix.

## 4. The chart is the first editor

On the Build desk, bodies drag (U14 says so already). This is what dragging does:

- **Drag a world** round its star: its orbit changes size. A ghost and its new orbit line follow the pointer; the world itself stays put until you let go. A small readout rides beside the pointer: the distance, the year, the surface the quick look gives, and the stability verdict (lasts, watch, unstable) from the ST1 blocks. The radial under the pointer shows the same tipping points and walls as the Orbit strip, and the "Where things can orbit" layer (gap 1, gap 40) turns on by itself for the length of the drag, so the neighbours' no-go bands and the zone lines are visible while you choose.
- **One grip, one thing** (Alex, 24 Nov: "the drag only has 1 aspect... the distance from parent"). Dragging the world in or out changes only its distance. The other parts of an orbit each have their own grip, which appear on the orbit once the world is selected, so nothing is hidden behind a modifier key:
  - **Along the line:** drag the world round its own orbit to set where it is now. The drag locks to "in and out" or "round" from the way it first moves, as a drawing tool does; Shift frees both.
  - **The far point:** a small grip at the far end of the orbit. Pull it out or in to stretch or round the orbit; carry it round to turn the ellipse.
  - **The lean:** in a tilted or edge-on view, a grip at the orbit's highest point lifts or lowers it. Past upright it is backwards, and says so.
  - Each grip shows the same readout, tipping points and verdict as its part of the Orbit box (section 11F), and the box's figure moves with it.
- **Fine control.** The drag is geared: moving the pointer far from the orbit line makes the same hand movement count for less, the way a scrub bar on a video does. Shift is ten times finer. The ghost is lightly magnetic to tipping points, round figures and resonances with neighbours, each named as it catches; Alt turns that off (question 10).
- **Moons** work the same at the moon level of the chart ("Moons of Tarn"). The range runs from the Roche limit to the edge of the Hill sphere, with the stable part marked (ST1's moon limit).
- **Drop on a new parent.** Drag a body onto another body and it lights as a target; let go and it orbits that body. Drag a moon out past its planet's Hill sphere and the ghost says "would escape: now orbits Kestrel" and that is what dropping does. This replaces V3's "Re-home to" list and its co-orbital picker (drop into a Lagrange zone of the gap 1 layer).
- **A star** drags too. In a pair, dragging one changes the separation; dragging the pair's centre moves the pair. The worlds that would be lost turn amber on the chart before you let go (section 10).
- **Touch.** Long press picks a body up (the same press that opens the menu in gap 38: holding still opens the menu, moving starts the drag). One finger then drags; the readout sits above the finger.
- **Esc** or dragging back to the start cancels. Nothing was written, so there is nothing to undo.

## 5. One way to place anything

Add, Paste, Duplicate and Move are the same act: a **ghost** on the chart that follows the pointer, carries a verdict, and is dropped with a press.

- **Add** (the + in the Build tools, A, or Add here in the press-and-hold menu). Today's Add here opens a dialog and asks for an orbit as a number. Change: the ghost comes first. Move over the chart and the small panel beside it keeps up: "What forms here" (the existing formation sampling, cached by distance band so it does not rerun per pixel), then "Kinds" (section 9), then "Something else". Pressing drops it. Typing a distance still works.
- **Paste** (U20, built, where pointer placement is "still to come"). The clip becomes the ghost, with everything orbiting it, and the verdict covers the whole clip ("Vault and its 4 moons: lasts here; its outer moon would be lost").
- **Paste onto a body** offers three things in a small menu at the drop: Replace it, keeping its orbit and name; Add as its moon; Put beside it.
- **Duplicate** (Ctrl+D): a copy becomes the ghost at once.
- **Cut** leaves a pencil outline where the body was until it is pasted or the cut is dropped.
- **Delete** is the kernel's `withdraw`: the body leaves from now on, history keeps it, Undo brings it back.
- A file dropped on the chart (the Open door, T5) that turns out to be a clip becomes the ghost under the pointer.

Every one of these is one change and one Undo, as U20 already does with `set_authored_batch`.

## 6. Light on the engine

V3 recomputed the body on every slider tick. SSE4's rule is "no heavy work for a click"; a drag is hundreds of ticks. The design:

1. **A drag writes nothing.** The moving value is a pencil value held by the view. No chronicle event, no state hash change, nothing sent to players.
2. **The strip is sampled once, when it opens.** One kernel job, a **sweep**: about 33 values across the strip's range, and for each only the results the card shows, the class label (`classify_preview`) and the stability verdict (`orbit_stability`). It runs in the kernel's worker in slices (the job mechanism that exists), outward from the present value, so the strip fills in as it arrives and can be cancelled. Where the label differs between two neighbouring samples, the job closes in on the exact point by halving. Those are the tipping points.
3. **While moving, figures come from the sweep.** Reading between samples is arithmetic in the page: no kernel call per frame, so it holds 60 frames a second on the phone tier. Figures shown this way are drawn in pencil (muted, italic) so nobody takes them for the settled answer.
4. **On a pause of 150 ms, one exact answer.** A `derive` for the pencil value on a scratch branch (`branch`, `branch_set`, `discard` exist). Budget: the 100 ms of "working out a system".
5. **On letting go, one change.** One `set_authored`, one full resolve, one undo step. The changed figures count to their new values (interface rule 7) and the card lists **what followed**.
6. **Heavy readers wait.** Weather, globe tiles, light and spectra, formation and N-body never run during a drag. After the change, each shows "from before the change" until its screen is next looked at, then redoes itself. This is how they are fetched already; the label is the new part.
7. **The chart redraws one layer.** During a drag only the ghost, its orbit line and the readout move. **Alex 24 Nov 15:13: yes.**

Reading between samples is presentation of kernel results, not a physics shortcut: every sample is the real chain, and the settled figure after a pause or a release is exact. It still gets a line in the fudge register so it is on the record.

Budgets to add to the performance readout (PF1), as a chunk "Editing": drag frame 8 ms; strip usable 300 ms after opening on the standard tier; release to settled card 200 ms.

## 7. Guided, never refused

"Danger is drama, never forbidden" applies to building as much as to play.

- **Walls are marks.** The handle passes through every wall. The label on the far side says what the physics says happens there. A world dragged inside the star is "inside Kestrel: gone in days". A planet's mass pushed past the point where deuterium burns gets the label the atlas gives it there, and past 0.075 solar masses it is a star, on the same record (one body, B1).
- **The verdict offers one fix,** and the fix is always a move of a handle, never a refusal: "Move to 0.61 AU, where it lasts", "Thicken the air to 0.3 bar and the water stays liquid", "Make the companion 40 AU away and all six worlds last".
- **Things that do not go together** are said as advice (the atmospheres ruling), on the verdict line of whichever handle is open: "No free oxygen lasts under this much hydrogen."
- **Honesty marks set themselves** (question 4). A hand-set input inside the physics is *authored*: plain dated data. It becomes *impossible* only when it is past a wall where the model has no answer, or when it is a pinned result that contradicts the physics. The GM can still tick impossible by hand. *Fictional* comes from packs, as now.
- **The stability verdict is the same one everywhere:** in the drag readout, on the ghost, on the card, and under the Orbit strip, from the same ST1 blocks. "Unstable" always offers "Run it and see" (N-body on request).

**A mark always says what it means** (Alex, 24 Nov 15:04: "clicking on authored and impossible should reveal what is being referred to"). The honesty marks (authored, catalogued, impossible, fictional) are never bare labels. Clicking one on a body's card opens a short list of exactly what earned it, each line a value in plain words with who set it and when: "Authored: mass 2.1 Earths (you, today), air 0.4 bar (pasted, 3 days ago)"; "Impossible: liquid water at 38 K. The science expects ice." Hovering a line lights the handle or the grip on the chart that it refers to; clicking a line opens that handle's strip at the value, where "Let physics decide" withdraws it. A mark on a single figure opens that figure's own line directly. A mark on a system or a pack entry lists the bodies that carry it, and each opens in turn. This is the same list as "What does not add up", filtered to one mark, so there is one mechanism; like the marks themselves it is on the GM's view only.

## 8. Changing a result: Reach it or Pin it

Results have no strip. Pressing a result's "why?" on Build shows the flow (as now) and two buttons.

- **Reach it.** "Get a surface gravity of 1 g by changing… Mass · Makeup." Choosing one runs a sweep of that handle and sets it where the result lands on your figure. The world stays wholly physical and gets no mark. If no value of the handle gets there, it says how close it can come and offers Pin it. Further uses: "make the year 365 days", "make it tidally locked", "make it warm enough for liquid water" (which offers Orbit, Air or the star's Mass).
- **Pin it.** Gap 4 as planned: the figure is fed into the flow so everything downstream follows, an optional reason is asked for, and the honesty mark shows on the GM view. Reset removes the pin. A pinned class is gap 12, the same button on the class line.

This replaces V3's Overrides tab, its Temperature tab of "+K" sliders, and the lock mass, lock radius, lock density switches (which were a way of saying which of three numbers should give way; Reach it asks that in words).

## 9. "I want a different kind of star": Kinds

Classes are labels read off results, so there is no list of types to pick from. But people think in kinds, and they should be able to ask for one.

- **Every class in the atlas has a "make one" recipe:** the input values of its representative sample (the same sample world or star the class gallery of gap 35 draws its picture from). A recipe is data in the pack that owns the class.
- **Kinds appear in three places:** the Add ghost's panel, the star chooser, and each class page in "Names and classes" ("Add one to this system").
- **Choosing a kind moves the handles** to the recipe and says so: "Red giant: 1.2 solar masses, 7 billion years old." From there you are free to move them, and the label changes as you go.
- **A kind that cannot hold where you put it is still offered.** "An ocean world at 0.1 AU: the sea boils off in about 2 million years. *Move it to 0.6 AU*, or *Add it anyway*" (the water is then pinned and marked).
- **Rare and hypothetical kinds** (the star census) are in the list when their pack is on, marked hypothetical.

**The star chooser** opens from a star's Mass handle, from Add, and from "New system". It is two strips, and it is the HR diagram of gap 3 made small:

1. **Mass**, a strip from 0.01 to 150 solar masses drawn in the real colour of the star at each mass, with the letters M K G F A B O under it and walls at "fusion starts", "ends as a white dwarf below here", "ends as a neutron star", "ends as a black hole".
2. **Age**, a life bar for the chosen mass: forming, main sequence, subgiant, giant, then the remnant, with the system's present age marked. Dragging it is `set_system_age` (the Look ahead ruling applies: "make this system this old, today").

Kinds chips above the strips (Red dwarf, Sun-like, Blue giant, Red giant, White dwarf, Neutron star, Black hole, more) move the two marks. At Science depth the full HR diagram is one press away with the star's track on it, and the same point can be dragged there.

**Changing a star that already has worlds** asks one question, once, with the cheap answer first:

- **Keep the worlds** (default). They stay where they are and answer to the new star: temperatures, air loss and stability are worked out again, and the chart shows which worlds changed class or would be lost.
- **Grow the system again round this star.** Runs formation (heavy, with a progress bar) on a what-if branch, shows the two side by side with Compare, and keeps nothing until you say so. Pinned worlds are honoured (the F9 pin search exists).

## 10. A second star

Adding a companion is the Add ghost with a star on it.

- As the ghost moves, the chart shades where worlds cannot last (Holman and Wiegert, both kinds, in ST1) and the worlds inside the shading turn amber. The readout counts: "4 of 6 worlds last."
- Two places catch the ghost: **close pair** (inside the innermost world: the worlds then circle both stars) and **wide companion** (well outside the outermost). Anywhere between is allowed and told.
- The verdict offers "Move it to 62 AU, where all six worlds last" or "Keep it here and move the worlds to safe orbits" (one batch, one Undo).
- Up to three stars as nested pairs, as formation makes them (`StarPins.count`); a fourth and beyond is allowed and gets ST1's verdict.

## 10A. Pairs, always: binaries, barycentres and managed chaos

Alex, 24 Nov: binaries "caught us out BIG time trying to bring them in later... Pluto/Charon should run on a barycentre. Also assume users will add stars and planets and trip binary adoption. I had an 8% mass to trigger a barycentre, but maybe we can just always assume it with its parent and have the counter wobble always. Leave those decisions to you... adding and removing bodies needs to be well managed chaos."

**What the kernel thread reports (24 Nov 14:55).** (a) An orbit is facts on the orbiting body (`orbit.host` plus elements); there is no barycentre object, and V3's barycentre nodes import as `bary:N` addresses nothing reads. (b) Nothing wobbles: `orbits_at` places a body about its host's centre with μ = G(M+m) and the host never moves; NAV uses G·M and has no moons. (c) Formation's chains of up to three stars are right inside formation, but the facts give every companion and planet `orbit.host` = star 0, so a third star or a world round both ignores the inner companion. (d) The default Solar System has no Charon and only one moon. (e) `set_authored` does not check hosts and `withdraw` cascades by address only, so hosts can dangle; a dangling body is silently left unplaced and dropped from the stability checks. ST1 knows a binary only from `body.kind` = star, which V3 imports lack. (f) There is no 8% rule anywhere.

**Its proposal, which this design adopts:** barycentre always, worked out down the hierarchy (Jacobi), no threshold and no barycentre objects. Each body orbits the centre of mass of its host plus everything inside its own orbit about that host, and every host wobbles. This makes decision 4 below simpler than first written: "round both stars" needs no pair as a parent. A world whose host is star A and whose orbit lies outside B's simply goes round the centre of A and B. So `orbit.host` stays one body, and R3's "round A" or "round both" is only inside or outside the companion's orbit. The one case it does not cover is a world round the *lighter* member only, which is just host = B. The kernel thread also proposes that removing a body re-homes what orbited it to its host keeping current motion, with "remove them too" as the other choice (this design's R4 and the remove row; the default between the two is question 38), and that every add or remove reruns the stability verdict (R1).

**What I read earlier.** Formation builds multiples as nested pairs about their barycentres to any depth, with the Mardling and Aarseth check on every nested pair (`sse-formation/src/hierarchy.rs`, B1c), and ST1 has Holman and Wiegert for worlds in and around pairs. But the record that editing writes to names one body as a parent (`orbit.host`), and the resolve from facts looks for one star with no parent. So today a generated triple is right inside formation and flattened where editing happens. This section is the model editing needs; its kernel side is ask K13.

### The decisions

1. **Always a pair. No threshold.** Every orbit is two sides going round their common centre. The lighter side's path is the big one and the heavier side's is the small counter-wobble, always computed (it is one multiplication). There is no 8% switch and no "binary mode" to trip. The Sun wobbles for Jupiter, Earth for the Moon, Pluto for Charon, by the same rule.
2. **"Binary" is a label read off the result,** like every class: a pair is *drawn and named* as a pair when its centre lies outside the heavier body's surface. That is a real boundary, not a typed number (Pluto and Charon: outside, so a double planet. Earth and Moon: inside. Sun and Jupiter: just outside, which is true and worth a line in "why?").
3. **The centre is never a thing the user makes or manages.** It is a point the chart draws when it matters. Nobody adds, names, moves or deletes a barycentre.
4. **What can be orbited: a body, or a pair.** A world goes round one star of a pair (inside), or round both (outside). Nix goes round Pluto and Charon. A pair needs no record of its own: it is named by its lighter member's orbit, so "round Pluto and Charon" is "round Charon's orbit". Nesting to any depth falls out of this, matching formation's tree.
5. **Addresses never change** when a system gains or loses a star or a pair becomes a single. Names are suggested ("Kestrel" to "Kestrel A" and "Kestrel B", per the naming design) and never forced.
6. **The heavier one is "the primary" only as a label.** If an edit makes the lighter side the heavier, nothing is restructured; the label and the drawing follow the masses.

### Adding, removing and moving: the rules that keep it managed

Every edit that changes who goes round what follows the same five rules.

- **R1. Show the family tree before it happens.** The ghost's panel carries a small before-and-after tree of the system, with every body that would be affected listed and its verdict (lasts, watch, unstable, lost). Nothing structural is ever a surprise.
- **R2. Nobody else moves unless you say so.** Adding a star does not re-parent or shift any existing world. Each keeps its parent and its numbers; only its verdict is worked out again. The tree offers, per affected world and for all of them together, the move that would save it ("round both", "round A, closer in", "hand to B").
- **R3. Where you drop decides what it goes round,** and the ghost says so in words before you let go: inside a body's hold, "round Kestrel A"; outside a pair's outer stable edge, "round both stars"; between, the nearer, with the verdict's warning.
- **R4. A move keeps one of two things, your choice at the drop:** *a tidy orbit here* (round, at this distance: the default for Add, Paste and a drag) or *where it is and how it is moving right now* (the same position and velocity, re-expressed round the new parent: the default for an escape, a capture or a removed parent, because it is what would physically happen).
- **R5. One change, one Undo, dated.** However many bodies a structural edit touches, it is one step, and like any edit it is "as it has always been" or "from this date" (a rogue star captured in play).

Applied:

| You do this | What happens | The tree asks |
|---|---|---|
| Add a star far out | it and the old star are a wide pair; worlds stay round the old star | worlds beyond the stable edge: move in, go round both, or leave to their fate |
| Add a star close in | a close pair | worlds: go round both (default offered), or stay round one |
| Add a world round both stars | parent is the pair | nothing |
| Raise a planet's mass until it is a star | nothing structural: the pair was always a pair. Label, drawing and verdicts change | as for adding a star |
| Add a heavy moon (Charon) | the planet and it are drawn as a double planet once the centre leaves the planet | existing moons: stay round the planet, or go round both |
| Remove a body with things round it | asks once | take them with it (default) · leave them, moving as they are, round its parent · hand them to a named body |
| Remove one star of a pair | the pair is gone; its worlds follow R4 | its own worlds: go with it, become rogues, or stay with the other star. Worlds that went round both now go round the one that is left |
| Drag a world from star A to star B | re-parented, tidy orbit | nothing unless it had moons (they come along) |
| Drag a moon past its planet's hold | "would escape": it goes round the planet's parent, moving as it was | nothing |
| Paste a clip that holds a pair | the pair arrives whole | as Add |

### Editing a pair: two orbits, and the centre stays hidden

Alex, 24 Nov 15:13: barycentre handling "was BAD in every version I have made... you need to be able to adjust the planets' orbit to each other and the barycentre orbit of that pair to its parent... the handles you have now may be the key to managing this cleanly and allowing us to effectively hide the barycentre, which confuses people. Also this way EVERYTHING can be a barycentre, the maths just rounds down to zero."

The handles do it, because a pair has exactly two orbits and each is an ordinary Orbit part. Nobody ever sees, selects or drags a centre.

- **"Round each other"**: how far apart the two are, the shape of that orbit and its lean. One orbit, shared by both members. It is the first part of the Orbit box on *either* member's card (open Pluto or Charon and it is the same strip, with the same value). Its strip runs from touching to the edge of the pair's hold on each other.
- **"Together round X"**: the path of the two as one round whatever they orbit (the Sun). It is the second part of the Orbit box, again on either card, and it is the ordinary orbit strip of section 4 with both masses counted. A top-level pair of stars has no second part.
- **A single body is the same thing with a partner too small to matter.** Its Orbit box shows one part, "round X". The moment something heavy enough goes round it that the pair is drawn as a pair, the box gains the "round each other" part by itself. Nothing is converted, because underneath every body already has both (the maths rounds the wobble to nothing).
- **On the chart** the same two things are two lines. Drag either member's small ellipse and both ellipses grow or shrink together about the hidden centre, in the right proportion for their masses: that is "round each other". Drag the pair's one big orbit line and both members move as one: that is "together round X". The centre is at most a faint cross, and it is not a thing you can pick.
- **Change a mass** and the two small ellipses re-share themselves (the heavier one's shrinks); the separation and the big orbit stay as you set them. That is the rule that makes mass edits calm: you set distances, the physics shares out the wobble.
- **A third body** is added with the ghost as ever, and R3 says in words which of the two it joins: "round Charon", "round Pluto", or "round both".
- **Typing** real data: the sheet takes the catalogue's own form (separation, period, both masses, the pair's orbit) since that is how binaries are published.

**And it feeds the stability verdict.** The blocks already use both masses (Hill spacing, Holman and Wiegert's stable edges inside and outside a pair, Mardling and Aarseth for a pair within a pair). What "always a pair" adds is that they apply to every pair by mass ratio with no "is this a binary" test first, so a heavy moon or a brown dwarf companion gets the same treatment as a second star. The kernel thread reports ST1 spots a binary today only by `body.kind` = star; K13 asks for it to go by mass.

**Seeing it.** The object selector on the place line (gap 37) is the tree, with a bracket drawn round each pair. Rows drag in it to re-parent, which is the reliable way on a phone and for anything too close to pick on the chart. The chart draws each pair in its own frame round its centre (the rendering rules' nested frames), with the centre as a small cross when the pair is drawn as a pair; the wobble of a heavy side is visible at Real scale or when zoomed to it. "Seen from" gains "the centre" and either member.

**What must be right underneath** (and is cheap): an orbit's period uses the mass of both sides; positions at a time come from the pair tree (`orbits_at`); the light on a world comes from every star by distance (B1's companion light); the stability verdict uses the pair-aware blocks.

### Build out the default Solar System first

Alex's suggestion, and the right test: the start sheet's Solar System becomes the fixture that proves pairs, before any editing package ships.

- **Pluto and Charon** on their common centre, with Nix, Hydra, Kerberos and Styx round the pair.
- **The Sun's wobble** from Jupiter and the rest; **Earth and Moon** round theirs.
- Two more catalogued starter systems, so the cases are all in hand from the start sheet: **Alpha Centauri** (a close pair with Proxima as a wide third: a nested pair) and **Kepler-16** (a world round both stars).
- **A checked list of structural edits** run against them in CI, each asserting the tree, the addresses and one Undo: add a wide star, add a close star, promote Jupiter to a star, remove Charon, remove Alpha Centauri B, drag a world from A to B, cut and paste the Pluto pair to another system.
- A walk step for the test drive: "Open Pluto: watch both circle the point between them."

## 10B. The experiment: build the Solar System to full precision with this UI

Alex, 24 Nov 15:21: "try an experiment: build the solar system to total precision using the UI you build. What worked, what was too hard, what broke?"

**What this is, honestly.** The editing UI is a design and a mockup; it is not built, so nobody can press it yet. What follows is a paper walk: each step of building the real Solar System from an empty system, taken through the design as written, to see where it holds. It found eleven things, and they are fixed in the design below. **The same experiment is then the acceptance test for the built UI** (package ED0b): a builder makes the Solar System through the UI alone and the result must match `packs/base-sol` (SOL1, the cited catalogue copy already in the repo) value for value, with a written log of what was awkward. Alex's three questions are the log's headings.

### The walk

1. Empty system. Add a star, type "1 Sun mass", set Age 4.57 billion years and Metals.
2. Eight planets: for each, Add here, then type six orbit numbers (size, shape, lean, the two angles that place the ellipse, where it was on a date), mass, makeup, day, tilt, air, liquids.
3. The Moon round Earth; the four big moons of Jupiter; Titan and Saturn's mid-sized six; Uranus's five; Triton and Nereid; Phobos and Deimos.
4. Pluto, then Charon round it, then Nix, Styx, Kerberos and Hydra round both.
5. Ceres, Vesta, Eris, Haumea, Makemake; a comet (Halley).
6. Rings for the four giants; the asteroid belt, the Kuiper belt, the Oort cloud.
7. The other 270 or so small moons.

### What worked

- **The star in three typed values**, and its light, size and colour follow. The life bar shows where 4.57 billion years falls.
- **Typing with units** (3A, 3C): "149.6 million km", "23.934 h", "23d 26m" go straight in as published. Nothing needed converting by hand.
- **Odd worlds need no special cases:** Venus spins backwards (one chip), Uranus lies on its side (tilt 97.8°), Triton orbits backwards (one chip), Mercury is mostly iron (drag the bulk bar), Halley's orbit is a long thin ellipse (shape 0.967). Each produced a sensible line in "what followed".
- **Pluto and Charon** (10A): add Charon, type the separation in "round each other", and the pair is drawn as a pair because the centre is outside Pluto. Nix "round both" is just an orbit outside Charon's. Catalogues publish the Pluto system and the Earth and Moon system by their shared centre, which is exactly the "together round the Sun" part, so the published numbers go in unchanged.
- **Catalogued beside computed:** the sheet shows the model's value next to the typed one, so the build doubles as a test of the physics (Earth's radius from mass and makeup against 6,371 km).

### What was too hard

- **F1. One body at a time is far too slow.** Eight planets are about 110 typed values; with the 20 main moons and dwarf planets about 400; with every known moon over 3,000. The per-body sheet does not scale. **Fix: a system sheet.** "Type it all in" at system level is one table, a row per body and a column per input, that takes a whole pasted table or a file (a spreadsheet, CSV, or the standard ephemeris tables), matches columns by name and unit, shows what it understood, and applies as one change. Steps 2 to 7 become one paste each.
- **F2. Seven tiny rings and belts.** The design gives rings and belts "where and how much" and defers the rest. Saturn's rings to full precision (the gaps, the Cassini division, the shepherd moons) cannot be made. **Fix: none in this note;** it confirms the rings and belts note is needed before "full precision" is claimed. Belts and the Oort cloud come from `base-sol` as graded regions meanwhile.
- **F3. The family tree for a big paste.** R1 shows every affected body before a structural edit; for 300 moons that is unreadable. **Fix:** the tree groups by verdict ("294 last, 4 to watch, 2 unstable") and opens only the ones that need a decision.

### What broke

- **F4. Which plane, which centre, which date?** Published orbits come in several conventions: leans measured from the ecliptic, from the planet's equator (most moons), or from a local plane; centred on the parent or on the shared centre; true for one date. The design had one "lean" figure and said nothing. A typed 5.1° for the Moon means something different in each. **Fix:** every sheet has a header line of three choices, **measured from** (the system's plane, the parent's equator, the sky as seen from Earth), **centred on** (the parent, the shared centre) and **true on** (a date), with sensible defaults per class; the kernel converts on the way in and the chart's grips always work in the system's own plane. This is the most important finding, because wrong-plane data looks plausible and is silently wrong.
- **F5. A star had one handle.** The Sun's 25-day spin and 7.25° tilt had nowhere to go. **Fix:** a star gets the Spin box too (Day, Tilt), which the magnetic and flare physics already read.
- **F6. Measured values that disagree with the model looked like mistakes.** Typing Earth's measured radius over the computed one would sit in "What does not add up" beside real impossibilities. That is the wrong way round: when a catalogued value and the physics disagree, the suspect is the model. **Fix:** the GM's list has two parts, **"You set this against the physics"** and **"The physics misses a measured value"** (with the size of the miss). The second part is the fudge register's intake: Mercury's field, Earth's inner core and Pluto's air would each appear here by themselves.
- **F7. Earth's air does not add up without life.** 21% oxygen is not what the geology expects; life made it. With the life sockets empty, Earth would be flagged. **Fix:** "the science expects" for air takes the world's life into account when it has any, and a mix can carry the reason "kept by life" (a chip), which is the 4.0 socket for what 4.1 computes.
- **F8. Pluto is flagged unstable.** It crosses Neptune's orbit and is safe only because of their 3:2 rhythm; ST1's resonance protection is listed as next, not done. Jupiter's Trojans, and Saturn's Janus and Epimetheus which swap orbits, fail the spacing check for the same reason: they share an orbit. **Fix:** kernel ask K15, resonance and shared-orbit protection in the verdict; the Solar System is its test. Until then the verdict would say "watch" where the real answer is "lasts".
- **F9. "Locked to" could only name the parent.** Pluto is locked to Charon, its lighter partner, and Mercury turns three times for every two orbits. **Fix:** the lock chip names any partner ("locked to Charon") and offers the common rhythms (1:1, 3:2).
- **F10. Precise on the day, drifting after.** Orbits are fixed ellipses. The Moon's orbit really swings round every 18.6 and 8.85 years, Mercury's creeps, so a build exact on its date is visibly off within months for the Moon. **Fix:** not a typed rate. The slow turning of an orbit follows from physics the kernel has (the parent's flattening, the pull of the Sun and neighbours), so K16 asks for it as a result applied by `orbits_at`. Until then the card states honestly "exact on 1 Jan 2000; positions drift after".
- **F11. Not everything is a ball.** Haumea is an egg and Phobos a potato; the design has one radius. **Fix:** small bodies below the size where gravity rounds them (a result) take three typed radii on the sheet, shown as a shape; no handle, since nobody drags that.

### What it changes

New: the system sheet (F1), the three-choice header on every sheet (F4), Spin on stars (F5), the two-part GM list (F6), "kept by life" (F7), lock to any partner (F9), three radii for small bodies (F11), grouped tree (F3). Kernel asks K15 (resonance and shared orbits in the verdict) and K16 (slow turning of orbits). Waiting on its own note: rings and belts (F2). Package **ED0b** is the experiment run for real on the built UI, with its log kept in project files.

## 11. Moons

- Everything in sections 3 to 8 applies to a moon unchanged; its Orbit strip runs from Roche to Hill.
- **Add a moon.** Corrected 24 Nov after Alex asked ("I thought the plan was to build moons from leftovers"): that is built. Formation's leftovers stage (F5, `sse-formation/src/leftovers.rs`) already gives every giant its regular moons from a disc of one ten-thousandth of its mass (Canup and Ward 2006), keeps the big moons made by giant impacts (F4), and scatters small captured ones far out, half of them backwards (Jewitt and Haghighipour 2007). So a *generated* system has its moons from the physics. The only thing missing is small: **Add here** can ask formation "what forms at this orbit round a star" but not "what would form round this planet", so adding a moon *by hand* offers only "Something else". Ask K5 is that one question, answered by the same F5 rules run for the selected planet (its mass, its leftover solids, its history of impacts), not a new model. Until it lands the same three rules are offered as fixed recipes: a disc moon, an impact moon (rocky worlds only), a captured body. F5's own note lists one real gap to close with it: small rocky planets get no captured moons yet (Phobos and Deimos).
- A moon's effect on its planet (tides, day length, tilt holding steady) is in "what followed" on the planet's card.
- Rings and belts are placed with the same ghost and have two handles, where and how much; their detail is a later note.

## 11A. Air and Liquids (added after Alex's first read)

SSE3 is the starting point for both: its Atmosphere tab (a pressure slider, a log slider for each gas with ppm below 1%, presets, a greenhouse readout) and its Hydrosphere tab (pick a primary liquid from an editable list, a coverage slider, derived fluid layers). Both keep their reach. What changes is that each becomes one handle with one box, and that the physics does more of the work.

**One idea underneath.** What a GM sets is an **inventory**: how much of each volatile substance the world has at its surface. Whether a substance is gas, liquid or ice there is a result of the temperature and pressure, read from its phase row on the data sheet through the Look up block. So Air and Liquids are two faces of one inventory, and each tells you when the other is affected: put 1.1 oceans of water on a world at 516 K and the Liquids verdict says "none of this is liquid here: it is 290 bar of steam", and the Air handle shows it.

**Air** opens a box with two parts:

1. **How much:** a strip of surface pressure on a log scale, like any other. Tipping points found by the sweep: "water can be liquid above", "people breathe unaided above" (a pack's rule; the Traveller pack adds its atmosphere codes as marks), "the sea boils away above". The diamond is what the science expects from outgassing and loss (the redox work, F-redox). Reference dots: Mars, Earth, Venus.
2. **What it is made of:** one **mix bar**, the whole air as a stacked bar. Drag a join to trade one gas against its neighbour; the rest keep their proportions, so it always sums to 100% with nothing to fix up by hand. Gases under 1% sit below as **trace chips** in ppm. Pressing any gas or chip opens that gas's own strip in place of the amount strip: its share on a log scale across eight decades (SSE3's log slider, kept), with its own tipping points: "fires burn above", "toxic to people above", "freezes out at the poles above" for carbon dioxide on a cold world.
   - **Add a gas** lists the data sheet's gases in three groups: what this world's rock gives off (expected), what it can hold (allowed), and what escapes ("hydrogen leaves in about 30 million years": still allowed, and told).
   - The verdict carries the things-that-do-not-go-together advice: "Oxygen with methane lasts only while something keeps making them."
   - Results that follow, shown in "what followed": greenhouse warming, surface temperature, sky colour (air optics), what rains, whether people can breathe it.

The box's menu holds "Reset to what the science expects" (this is the planned "Suggest an atmosphere" button), **Kinds** for air (Earth-like, Mars-like, Venus-like, Titan-like, first hydrogen air; packs add their own), "From this date" and Why?.

SSE3's "ages or stays as authored" switch goes. A hand-set air is dated data; the verdict says how fast it is being lost ("thins by half in 400 million years"), and Look ahead applies that. Nothing erodes behind the GM's back and nothing is frozen against the physics.

**Liquids** replaces the Water handle. Its box has two parts:

1. **How much:** a strip of the amount of the main liquid (in oceans, metres of global layer, or kg; click the unit). Its tipping points come from the world's own ground: "first seas", "half covered", "last land drowns", "ice forms on the sea floor". **Coverage is a result but it is on the same line**, so dragging to "half covered" is dragging to 50%. SSE3's coverage slider is therefore still there, as marks on the amount strip, and "Cover a share instead" in the menu takes a typed percentage (a Reach it).
2. **What can be liquid here:** a short list of substances, each drawn as a **phase bar** along temperature (solid, liquid, gas at this world's pressure) with the world's own surface range laid across them all. At a glance: water is liquid here, methane is gas, sulphur is solid. The ones in use are marked. This is the guide SSE3's liquid picker lacked: it shows why before you choose.
   - **Add a liquid** adds another substance to the inventory (methane and ethane on a cold moon; ammonia in water as antifreeze). Choosing one that cannot be liquid here is allowed: it goes in as ice or vapour, the verdict says so, and offers the move that would melt or condense it.
   - **Salts** and other dissolved things use the mix editor (gap 11), and shift the freezing point as the data sheet says.
   - **New substance** opens the data sheet's substance editor (Reference): melting, boiling, triple and critical points, density, colour, whether life can use it as a solvent. This is SSE3's Edit Liquids dialog, moved to where every other part of SSE4 can read the result. A made-up liquid lives in a pack and carries the fictional mark.

Molten rock, seas under ice shells and deep fluid layers are not set here. They are results, and the cutaway shows them (the phase profile). To get one, change what causes it (Reach it offers the handles), or pin it.

**The kernel today:** pressure and a gas mix are authored fields, but an authored air does not yet change the greenhouse (the surface chain uses a pack constant; test drive rough edge 1 is the same gap). Surface liquid is water only, in oceans, with a yes or no for "has oceans". Moving ices (W8) already handle carbon dioxide, nitrogen and methane frost. So this section needs asks K7 and K8 below, and until they land the Air and Liquids boxes can ship showing amount and mix with the honest note "does not yet change the climate".

## 11B. Makeup and the inside of a world

Makeup is one handle on the row, but its box is the richest of the six, because SSE4's interiors are real: a core that freezes over time, a mantle that convects or stalls, ice phases, shells and buried seas. The box has three parts, and the **cutaway is the editor**.

1. **What it is built from:** one bulk bar of the five materials formation uses (iron and metal, rock, carbon, ices, gas), by mass. Drag a join; the others keep their proportions. Tipping points sit on the joins and come from the sweep: "core as big as Mercury's", "enough ice for a sea with no floor of rock", "gas past 1%: the radius swells". Radius, density and gravity follow in pencil as you drag, which is the mass and radius relation made visible.
2. **What that makes inside:** a cutaway wedge beside the bar, redrawn from the sweep as you drag: inner and outer core, mantle, crust, any buried sea, ice shell, deep air. **Layers are results.** Press a layer for its why?, with Reach it and Pin it like any result ("make the ice shell 5 km thick").
3. **What drives it** (Science depth), two strips:
   - **Heat from the rock:** how radioactive the rock is, against Earth's (`k40_rel`, `r_process_rel`). Tipping points: "the mantle stalls below", "the core's dynamo dies below", "a buried sea freezes below", "volcanoes everywhere above". Heat from tides is shown on the same line as a second mark that you cannot drag, labelled "from its orbit", because it is a result of the orbit and the neighbours; pressing it takes you to the Orbit handle.
   - **How oxidised:** the mantle's redox state (`redox_iw`). It sets how much iron went to the core and what the rock breathes out (F-redox), so its tipping points are "air comes out as hydrogen and methane below" and "as water and carbon dioxide above".

**Kinds for makeup** at any depth: Earth-like rock, iron world, water world, ice moon, carbon world, small Neptune, gas giant. Each sets the bar.

**Where research is divided,** the kernel already carries a choice (a regime's `dial`: for example whether a heavy rocky world has moving plates or one stiff lid). That shows as a small choice on the result's why?: "Research is divided. Using: stiff lid. Switch to: moving plates." Choosing is not a pin and carries no mark.

**Riches.** What a world holds to mine is a real amount from its makeup and history (the element inventory, MP6). A worldbuilder who wants "rich in heavy metals" changes the system's birth chemistry or this world's heat-producing and heavy element shares at Science depth. Placing one particular lode by hand is site authoring, which is parked for 4.2.

**So how do you put in a subsurface ocean?** It is a natural occurrence, and you ask for it. A buried sea exists when there is water, a lid of ice, and enough heat or antifreeze to keep the water under the lid liquid. The app shows how close the world is and what would close the gap. That is section 11C.

## 11C. "Could be…": the app suggests, the worldbuilder chooses

Alex: "we *should suggest*... a user can do one and not the other" and "we guide better now rather than leave them alone". So guidance is not only a verdict on what you just did. Each world's card carries one line that looks ahead:

> **Could be** home to humans · have a buried sea · have a magnetic field · more ▾

Pressing one opens a short **recipe**: the steps that would make it true for this world, worked out by the physics, each with its own button, in order of what depends on what.

*Home to humans, for a cold, thin-aired world:*
1. **Thicken the air** to 0.6 bar with 21% oxygen. `Do this`
2. **Give it a magnetic field,** or the star strips that air: half gone in 80 million years. Needs a liquid iron core that is still stirring: raise the heat from the rock to 1.3 times Earth's. `Do this` · `Pin a field instead`
3. **Warm it** by 14 K: the thicker air does 9 of those; move in to 0.49 AU for the rest. `Do this`

The rules for a recipe:

- **Every step is optional and separate.** Do one and not the others. The line then reads what you chose to live with: "Home to humans, **but** the air is being lost (half in 80 million years)". That is a story hook, not an error.
- **Each step is a Reach it** where the physics can get there, and offers **Pin it** where the worldbuilder would rather just say so. A pinned step is marked on the GM view and nowhere else.
- **Steps know what they lean on.** A step that only matters because of another is drawn under it ("needs a field *to keep* the air"), so the chain Alex describes (air, then a field to protect it) is visible before anything is changed.
- **The smallest change first.** Where several handles could do it, the recipe offers the one that moves least from where the world is, and lists the others under "or".
- **Nothing is applied until pressed,** and each press is one undo step. "Do all" exists and is one step.
- **It is suggestion, not nagging.** One line on the card, closed by default at Science depth, and a Curious-depth sentence in words ("With thicker air this could be a second home"). A world can be told "stop suggesting this".

**What can be suggested** is data: each "could be" is a small record in a pack with a name, a test over results (process strengths, amounts, labels) and nothing else. The recipe is not written by hand; it is the solver of Reach it run on that test across the handles. The base pack ships these: home to a profile (below), have open seas, have a buried sea, have a magnetic field, have moving plates, have volcanoes, be locked to its star, have seasons, keep its air, have rings, hold a large moon. A game pack adds its own (the Traveller pack: "be a Garden world", "have a breathable-with-filter air").

**Who it could be home to: requirement profiles.** "Home to humans" is a test against a **profile**: the ranges a kind of life needs. Temperature, pressure, the gases it breathes and the ones that poison it, gravity, radiation at the ground, the liquid it is built on, light. The life design already describes this profile for habitats (its requirement profile, handoff 11A); this uses the same shape for a visitor or settler.

- **A profile is a set of survivability bands, one per thing that can hurt** (Alex, 24 Nov 15:20, replacing the first draft's "three grades": "surely we can say yes, but needs protection from lack of oxygen (breather), cold (warm or heated clothing for ranges), heat, radiation (foil suit, underground habitat and infrequent surface visits)... 'can live here' and 'can live here uncomfortably because'... run the physics for a human body... a survivability band"). Each band is a strip like any other, over one physical quantity the kernel already works out, with four stretches: **comfortable**, **tolerable** (uncomfortable, with the reason and for how long), **needs protection** (and which), **nothing saves you**.

| What can hurt | Read from | Protection, by how far out of band |
|---|---|---|
| Too little oxygen | oxygen partial pressure | breather, then pressure suit |
| Too much oxygen, or poison gases (CO₂, CO, SO₂, H₂S, ammonia, chlorine and the rest) | each gas's partial pressure against its own limit | filter mask, then sealed suit |
| Total pressure low or high | surface pressure | pressure suit; hard suit; pressurised habitat |
| Cold | air temperature with wind, by day, night and season | warm clothing, heated clothing, heated suit, by range |
| Heat | wet-bulb temperature (heat and humidity together) | shade and rest, cooling garment, cooled suit |
| Radiation | dose at the ground (MF7, built) from the star, flares and cosmic rays | hours outside per day, shielded suit, living underground with short surface visits |
| Ultraviolet | UV at the ground through the air and ozone | covered skin and eyes, then suit |
| Gravity | surface gravity at the site's latitude | none: tolerable with fatigue, then harm over years, then braces and short stays |
| Day and night length, light level | the sun's day, light received | lighting and kept hours; purely comfort |
| Water to drink, air to breathe on site | liquids and their salts | none needed, treat it, bring it |

- **The verdict is put together from the bands, with reasons:** "Humans can live here." "Humans can live here, uncomfortably: thin air like 4,000 m, nights of −30 °C (heated clothing)." "Humans can live here with a breather and a radiation shelter; two hours a day outside." "Humans need a sealed suit: CO₂ at 30 times the limit." The worst band sets the headline; every band out of comfort is listed. A band's line opens its strip with the world's value marked on it, so the GM sees how far off it is and which handle would move it, which is exactly what "Could be…" then offers.
- **It varies over the world and the year,** because the quantities do: the same card can say "at the equator by day: a breather only; at the poles: heated suit", and a surface site gets its own verdict for its place and season. Altitude counts (thin air on a mountain, thick air in a deep basin).
- **Protections are data too:** a short list of gear and shelter, each stating which band it widens and by how much (a breather supplies oxygen down to a stated total pressure; heated clothing holds to a stated temperature; two metres of rock cuts dose by a stated factor). A setting's better suits are a copy with wider reach, and the tech ladder (track FT) can date them.
- **The human bands are cited physiology,** on the data sheet like any other numbers: oxygen and CO₂ limits, the pressure limits (the Armstrong limit, nitrogen narcosis), wet-bulb heat limit, cold exposure tables, dose limits, long-term gravity as "research is divided". This is "running the physics for a human body": a body is a requirement record the kernel tests against a place.
- **Nothing in a profile is human by default** (Alex, 24 Nov 15:22: "oxygen is human; have Rocky in the back of your mind from Project Hail Mary... we need his spaceship atmosphere; use that as the counter-human... it shows us what we need to get ready for 4.1"). The table above is the *human* filling of a general record. The record itself has slots, not fixed rows: **what it breathes** (any gas, with its band of partial pressure), **what poisons it** (any gases, each with a limit), total pressure, temperature, gravity, radiation, **what it senses by** (light, sound, none), and the liquid it is built on. The counter-human used to test the shape, in the design and in the checks, is a being from a hot, crushing ammonia world as that book describes: it breathes ammonia at about 29 bar and about 210 °C, lives under about twice Earth's gravity, has no use for light, and **oxygen is one of its poisons**. Put the two profiles against the same places and the design has to cope with all of these:
  - Earth reads "Humans: can live here" and, for the other, "needs a sealed, heated, pressurised suit: far too cold, a thirtieth of the pressure, oxygen at poisonous levels". Its own world reads the reverse. So no band, word or protection may assume oxygen, 1 bar, 20 °C or daylight.
  - **Protections are per profile:** a human's "breather" supplies oxygen; the other's supplies ammonia and heat. The protections list is keyed by which band of which profile it widens.
  - **A ship's inside is a place too.** A construct carries "the air inside" as the same Air box (amount, mix, temperature), set by its builders' profile, and the profile test runs against it: a human aboard the other's ship gets the same composed verdict as on a planet. This is the hook the Paper spacecraft data (track RK) needs: a hull holding 29 bar at 210 °C is a different hull.
  - **Two profiles side by side:** a card can show where two kinds could both stand unprotected (here, nowhere: they meet through a wall), which is a story fact a GM wants and the first thing 4.1's species will be asked.
  - Naming: the base pack ships this counter-profile as a plainly named example, "Ammonia breather (hot, high pressure)", not under the book's names; a fan pack may do otherwise under the named-settings rule.
- 4.0 ships **Humans** and that one example in the base pack. It is editable like any pack data (a setting's hardier colonists are a copy with wider bands).
- **The same shape is what exobiology supplies:** a microbe's or an alien's bands are different numbers over the same quantities (or its own solvent and gases), which is why the life design's habitat profile and this one are one record type.
- The card's line names the profile: "Could be home to **humans**". A campaign chooses which profiles it cares about; the line lists those.
- **The hook for 4.1:** a species is a record that carries a profile. When the aliens update adds species that come out of evolution, each brings its profile and every world answers "could be home to the Vesh" with no new mechanism. A GM can already write a profile by hand in 4.0 for a setting's aliens; what 4.1 adds is profiles that follow from a species' own history.
- Players never see suggestions. What a crew learns about habitability comes through their sensors and lens, as now.

**What does not add up (GM only).** The other half of "we let them and point out things that break physics". Each world's card, on the GM view only, has one list of everything on it that strains or breaks the physics: pins, values past a wall, unstable orbits, mixes that should not last, steps skipped in a recipe. Each line says what, by how much, and has two buttons: go to it, and "make it add up" (the Reach it that would mend it). The system card sums its worlds. Nothing here is ever shown to a player seat, and nothing here blocks anything.

## 11D. What a worldbuilder wants to do, and how

Thought through from the worldbuilder's side. "Could be" means it is offered as a suggestion; "allowed" under the last column is always yes, so the column says what the GM-only list will note.

| I want this world to… | What you do | What the GM list may note |
|---|---|---|
| have Earth gravity | Reach it on gravity (offers Mass or Makeup) | nothing |
| be warmer, colder, temperate | drag Orbit; or Air; or the star's Mass; "Could be: open seas" | nothing |
| have air people can breathe | Air box, or "Could be home to humans" | oxygen with nothing making it; air being lost |
| be a desert, an ocean world, an archipelago | Liquids amount, with coverage marks | nothing |
| have seas of methane, ammonia, something invented | Liquids, Add a liquid; New substance for an invented one | fictional substance |
| have a buried sea | "Could be: a buried sea" (ice, heat, antifreeze, or tides from a neighbour) | pinned sea with too little heat |
| be volcanic, or dead and cratered | Makeup, heat from the rock; or an eccentric orbit for tides | nothing |
| have continents that drift | "Could be: moving plates"; or choose where research is divided | nothing |
| shield its surface from radiation | "Could be: a magnetic field"; or thicker air | pinned field without a stirring core |
| have a long day, no day, a steep tilt | Spin handle; "Could be: locked to its star" | spin faster than break-up |
| have big tides, a huge moon, two moons | Add a moon with the ghost; Kinds | moon past the stable limit |
| have rings | "Could be: rings" (a moon inside the Roche limit, or a ring placed by hand) | ring that should have spread away |
| sit round a red giant, a pulsar, two suns | star chooser, Kinds; Add a companion | worlds that should not have survived |
| be rich in something to mine | Makeup at Science depth; system birth chemistry | amounts no birth cloud gives |
| have been different once (dead world, terraformed, shattered) | "From this date" on any handle makes it an event in the world's history | nothing: dated changes are plain data |
| be a rogue with no sun, a twin world, a trojan | drop on no parent; drop on a body; drop in a Lagrange zone | nothing |
| carry life | 4.0: life follows from energy and chemistry, and "Could be home to" a profile; richer control is 4.1 | life pinned where no energy source exists |
| be flat out impossible for the story | Pin it, with a reason ("Precursor engineering") | the pin, in full |
| look a certain way (colour, clouds, sky) | results of air and ground; pin a look only in a Looks pack | nothing |
| have a name, a story, secrets, a picture | the card, Known to, notes (already designed) | nothing |
| be built, hollow, artificial | a construct with physics blocks (megastructures note) | per that note |

## 11E. SSE3's Settings editors, each with its SSE4 home

SSE3 keeps its definition editors under Settings: Tag categories, Time and Calendars, Fuel and Drives, Sensors, Atmospheres and Mixes, Liquids, Biospheres, the Skin editor and colour palette, plus LLM settings, the world gallery, the relay, Your data, memory, player connections, diagnostics and the danger zone.

The SSE4 rule for all of them: **a definition is a record in a pack, and you edit it with the same card, handles and strips as a world.** A change goes into the campaign's own pack, the base value is shown beside it with Reset, and an invented thing carries the fictional mark. **Changed by Alex, 24 Nov 15:13:** definitions are *not* edited from a world's card. "Gases are defined in the depths of SSE4; here you can only use what has been defined. We don't want to complicate this interface any more." So a world's boxes only *pick* from what exists. Every picker (gases, liquids, and by the same rule drives, sensors, calendars) ends with one quiet line, **"Can't find it?"**, with two ways on: **"Define it in Reference"** (opens the data sheet's page for that shelf, in Workshop under "Your setting's data") and, when an agent is connected, **"Ask Orac to make it"** (Orac writes the record as a pack entry, shows it for a yes, and it then appears in the picker). For that, Orac must know the blocks, the data sheet's shape, the build method and the UI tools it can drive, as it does in Ovid; that is a standing need on the agents track, not a new one here.

| SSE3 editor | What it edits | SSE4 home | Better how |
|---|---|---|---|
| Atmospheres and Mixes | per gas: molar mass, melting and boiling, greenhouse factor, Rayleigh scattering, absorption and aurora bands, colour, clouds; mixes with pressure limits, "occurs on" and spawn weight | Gases are substance rows on the data sheet (Reference). Mixes are Kinds for air. | Greenhouse and colour come from the absorption bands, not a typed factor. "Occurs on" and spawn weight go: the physics decides where a mix arises. Bands are edited on the spectrum chart. |
| Liquids | melting, boiling, triple and critical points, density, colour, cloud look, biosolvent, conducting | the same substance rows (section 11A) | one row serves air, sea, ice, interior and life |
| Biospheres | biosphere types and their needs | life chemistries as pack data (E0), and requirement profiles (11C) | profiles also answer "could be home to"; richer life is 4.1 |
| Fuel and Drives | fuel density; drive type, thrust, exhaust speed, power, use in air, FTL, where refuelled, availability | Paper spacecraft data (track RK) and the tech ladder (track FT) | availability is a civilisation's capability with a date, not a typed list; each number is a handle with "can land here" as a tipping point |
| Sensors | sensor definitions: range, what it finds | sensor records with real bands and reach (`sse-sense`, `detect`) | range follows from aperture and band; seats get their sensors' bands through the lens |
| Time and Calendars | calendar name, epoch, year length, leap rules, weekdays, format | Reference, Units and calendars (MP15; the kernel's calendars exist) | a calendar can be read off a world's own day and year, then adjusted |
| Tag categories | categories, colours, player-facing names, automated rules with chances | Tags in Workshop (tags design) | tags are labels read off results by rules, never chance, and never feed physics |
| Skin editor, palette | GM skins and single colours | Looks packs and player themes; GM skins were dropped | as ruled |
| Rendered world gallery | pictures of world types | the class gallery in Names and classes (gap 35) | drawn from the physics; "Add one to this system" on each (Kinds) |
| LLM settings | model and key | Agents (bring your own key) | as built |
| Map display, units | map options, unit system | view controls (rule 12); click a unit (gap 5) | per viewer, in context |
| Relay, player connections | network for remote players | Seats and Share with players | as built |
| Your data, memory, diagnostics, danger zone | storage, export, reports, wipe | Your data in More; the diagnostic file through the Open door | as built |

Not yet anywhere in SSE4 and needed for this: a substance editor on Reference (MP10 part 3 covers the page; editing is MP9 and MP12), the calendar editor, and the "Your setting's data" entry in Workshop that lists a campaign pack's own records by shelf. These join the asks below.

## 11F. The Orbit and Spin boxes in full

**Orbit.** The chart stays the first editor (section 4). The Orbit box is the same things as figures and strips, for when you want a number, are on a phone, or want a part the flat chart cannot show. It opens with a small picture of the orbit from above and from the side, whose points drag like the chart's.

| Part | Field | The control | Tipping points and marks |
|---|---|---|---|
| Size | `orbit.semi_major_axis` | the strip of section 3; type a distance or a period | boil and freeze lines, frost line, neighbours' reach, Roche, the star |
| Shape | `orbit.eccentricity` | strip from round to 1 | "seasons from distance begin", "leaves the temperate zone at its far point", "crosses Sable's orbit", "tides heat the inside above" (moons), and the wall at 1: "leaves the system" |
| Where it is now | `orbit.mean_anomaly_at_epoch` | a ring: drag the world round its orbit | catches at closest and farthest point, and at "lined up with", "opposite" and "square to" each neighbour; this sets where it is at the campaign's now |
| Lean | `orbit.inclination` | strip from 0 to 180 degrees | "wild swings with the companion begin" (Kozai), "no longer crosses the star's face as seen from Sable"; **past 90 is backwards**, written on the strip |
| Backwards | the same field | one chip, **Retrograde**, which flips the lean to its mirror | the verdict says what a backwards orbit means here ("a captured body: tides draw it inward", for a moon) |
| Turn of the ellipse, and the line where it crosses | `orbit.argument_of_periapsis`, `orbit.ascending_node` | two small rings, at Science depth | which season falls at the closest point |
| In step with a neighbour | none: it is a size | chips: "2:1 with Sable", "3:2 with Vault" | sets Size to the resonance and says whether it holds |
| Round what | `orbit.host` | drop on a parent (section 4); or a list | none |

Results shown under it: year, closest and farthest distance, speed at each, how much the sunlight swings through the year, the stability verdict and why. In a tilted or edge-on view of the chart the lean is dragged directly by lifting the orbit; in the flat chart it is the strip.

**Spin** is the sixth handle and gets the same treatment.

| Part | Field | The control | Tipping points and marks |
|---|---|---|---|
| Day | `body.rotation_period` | strip on a log scale, from break-up to years | "flies apart faster than this", "weather splits into more bands below", "locked to its star here" (it catches), "three turns per two orbits" (Mercury's state); the diamond is what tides and its birth would have left |
| Backwards | the same field, signed | one chip, **Retrograde** | "the sun rises in the west" |
| Tilt | `body.obliquity` | strip from 0 to 180 degrees | "no seasons below", "the poles get more sun than the equator above", "held steady by its moon" or "wanders without a large moon" |
| When midsummer falls | season phase | a small ring, at Science depth | against the closest point of the orbit |
| Locked | none: it is a day | chips: "Locked to its star", "Locked to its planet" | sets Day to the year or month, and says whether tides would have done this by now |

Results: length of the sun's day against the star's day, days in a year, **oblateness** (how flattened the spin and makeup leave it), how the spin axis wobbles and over how long, and tide heating. Oblateness is a result and only ever shown, with no control of its own (Alex, 24 Nov 15:13: "no selection, just show... they can reduce it by turning down the spin"); its line says so ("flattened 6%: a 10 hour day does that"). Earlier draft, withdrawn: Reach it offers Day and Makeup, and Pin it remains. It matters downstream (a flattened planet turns its moons' orbits), so it is on the card at Science depth with its why?.

**Feeling lighter from spin** (Alex, 24 Nov 15:15). Yes, and it is a result worth showing. At the equator the spin takes away a share of your weight equal to (shortest possible day ÷ actual day)², where the shortest possible day is the one at which the equator would lift off (about 1.4 hours for Earth). Earth today: 0.3% lighter at the equator than at the poles, 0.5% with its flattening counted. An Earth with a 14 hour day: 1%. A 4.5 hour day: 10%. A 2 hour day: half. Saturn's equator already has about a quarter less pull than its poles. So the Spin box shows **"gravity at the equator and at the poles"** as a result once they differ by 1% or more, the Day strip carries two tipping points found by the sweep, **"noticeably lighter at the equator"** (10%) and **"the equator lifts off: the world sheds material or pulls into an egg shape"** (the wall, passable like any other, Haumea being the real example), and a surface site's gravity is the figure for its latitude, which ships, landing and the players' pages read.

**The kernel today** has every orbit element as an authored field, a signed rotation period and obliquity (weather reads them, W7b). It has no season phase as an authored field and no oblateness as a result on the card; both join the asks (K12).

## 12. Thinking wide: what else this is

The general capability is "any input is a handle with a strip, and any strip can be swept". Other uses, none of them a special:

- **Age is a handle.** "Ten billion years on one slider" (tour 22) is the Age strip with the star's life as its tipping points.
- **"Why?" gains a curve.** Any result can show how it responds to one input, from the same sweep. This is the teaching use: "how does temperature fall with distance" is one press.
- **Orac** gets `sweep` and Reach it as tools: "at what mass does this world keep an ocean?" is one call, with the answer traced.
- **The class atlas** can draw each class's edges from the physics instead of a typed table.
- **Ships and stations** (Build only for the ship itself, per the 24 Nov ruling): thrust, tank size and reactor power are handles with strips whose tipping points are "can land here", "can reach orbit".
- **Workshop constants:** "what if gravity were stronger" is a handle on a constant.
- **Future tech ladders,** light controls, weather settings: any numeric field with a row in the handle sheet.

## 13. Desks, depth and the phone

- **Build** moves handles. **Explore** shows them as plain figures; pressing one opens the strip read-only as an explanation of where this world sits, with one button, "Try changing it", which opens an example branch ("EXAMPLE · change anything", onboarding section 12) where it moves freely and nothing is kept unless asked (question 7). **Run** shows no handles; only pack-declared live fields (fuel, crew, cargo) move there.
- **Curious** names handles in plain words (How far out, How heavy, What it is made of, Seas, Air, Day and tilt), shows tipping points and reference worlds, hides bands. **Science** adds eccentricity, inclination, makeup shares, bands, Reach it and Pin it. **Engine** adds the angles, field names, the blocks each handle feeds and the sweep's timings.
- **Phone.** The handles row scrolls sideways under the body's name. The open strip docks across the full width above the time ruler with a thumb-sized mark; the verdict sits over it. Dragging on the chart is long press, then drag.
- **Dated edits.** A change on Build is "as it has always been" unless the strip's menu says "From this date", which makes it an event at the ruler's time (terraforming, a moon captured in play). See question 6.
- **Undo** names what it takes back ("Undo: Tarn's orbit 0.52 to 0.61 AU") and Reset on a body returns every handle to what the science expects, as one step.

## 14. V3's editing tools, one by one

Read from V3's source on 24 Nov (`BodySidePanel` and its ten tabs, `BandedSlider`, `OrbitalSlider`, `BodyOverridesTab`, `AddBodyTypeModal`, `ContextMenu`, `UndoPill`).

| V3 tool | Verdict | In SSE4 |
|---|---|---|
| Ten tabs per body, about 27 sliders | Replace | Six handles in one row; one strip at a time |
| Banded slider (green, amber, red; "possible, just unlikely") | Keep the idea, widen it | Bands on the strip, plus tipping points and walls found by the physics |
| Mass, radius, density sliders with locks | Replace | Mass and Makeup are handles; radius and density are results; Reach it for a target |
| "This slider covers part of the full range" zoom | Drop | The strip's range is already the real one for this body |
| Orbit tab sliders | Replace | Drag on the chart; figures for typing; angles at Science depth |
| "Re-home to" list, co-orbital picker | Replace | Drop on a new parent; drop in a Lagrange zone |
| Binary: distance of the pair and separation as two sliders | Replace | Drag a star, or drag the pair's centre |
| Star tab: spectral type list plus separate mass, radius, temperature, luminosity sliders | Replace | One Mass handle and the system's Age and Metals; Kinds; Pin it for the rest |
| Type and image picker, "reset to type", "let physics derive the type" switch | Replace | Kinds set inputs; class is always read off; Pin a class (gap 12) |
| Temperature tab (albedo, greenhouse, tidal, inner heat as "+K") | Replace | Results with "why?"; Reach it or Pin it |
| Hydrosphere tab: pick a primary liquid, coverage slider, derived fluid layers | Keep the reach, change the footing | Liquids handle (11A): any substance, by amount; phase and coverage are results; coverage can still be dragged |
| Edit Liquids dialog (melting and boiling points, critical point, colour, biosolvent) | Move | A substance row on the data sheet, in a pack; marked fictional when it is |
| Atmosphere tab: pressure slider, a log slider per gas with ppm, presets, greenhouse readout | Keep, tighten | Air handle (11A): one amount strip, one mix bar, traces as chips; presets are Kinds |
| Atmosphere "ages or stays as authored" switch | Drop | The verdict gives the loss rate; Look ahead applies it; hand-set air is dated data |
| Overrides tab with reasons | As planned | Pin it (gap 4), on every result, no tab |
| Add body type dialog with gates as switches | Done better already | Add here; now ghost first, with Kinds |
| Right-click menu | As planned | Gap 38, from the same list of actions as the Build tools |
| Cut, copy, paste | Built (U20) | Gains the ghost, paste onto a body, Duplicate |
| Undo pill | Keep | Undo and Redo on Build, naming the change; "what followed" beside it |
| Generation dials and wizard | As planned | Gap 3; the dials are handles on the system card, with strips |
| Biosphere tab | Wait | Life is 4.1; sockets stay |

## 15. What it asks of the kernel and the chart room

Kernel (thread "Build kernel resolve and rules"):

- **K1 `sweep`**: for one address and field, a range and a count, return the asked-for results, the class label and the stability verdict at each value, with tipping points refined; as a cancellable job. Also offered to the agents.
- **K2 unset as a read**: "what the science expects" for a field that is set (the resolve with that one field left out). `branch_set` with an unset may already do it.
- **K3 makeup all the way through** for hand-made worlds, and **radius from mass and makeup** when radius is not set (question 3).
- **K4 pin a result** (gap 4) and Reach it's solve, which is a sweep plus a root find.
- **K5 "what forms round this planet"**: F5's existing moon rules (disc, impact, captured) exposed for one chosen planet, so Add here can offer them; plus captured moons for small rocky planets.
- **K6 honesty rule** of question 4: authored unless past a wall or contradicting the physics.
- **K7 authored air drives the climate**: pressure and mix feed the greenhouse and air optics for hand-made and catalogued worlds, and "what the science expects" is readable for both amount and mix.
- **K9 interior inputs as authored fields** for any world: heat-producing elements, redox, core light elements, and the regime `dial` choice, with layers readable per sweep sample for the live cutaway.
- **K10 `could_be`**: given a world and a test from a pack, return the steps (handle, target value, what it leans on, what skipping costs). It is Reach it over several handles. Also offered to Orac.
- **K11 requirement profiles** as pack records, with the Humans profile in the base pack, shaped so a 4.1 species record can carry one.
- **K12 season phase as an authored field, and oblateness, gravity by latitude and axis wobble as results** readable per sweep sample.
- **K14 test a place against a profile:** one op that takes a body (or a site and a time) and a profile and returns each band's stretch, the value, and the protection that would cover it; wet-bulb temperature and UV at the ground as results if not already there.
- **K15 resonance and shared-orbit protection in the stability verdict** (10B, F8): Pluto with Neptune, Trojans, co-orbital moons.
- **K16 slow turning of orbits as a result** (10B, F10): apsidal and nodal precession from the parent's flattening and neighbours' pull, applied by `orbits_at`.
- **K17 conventions on the way in** (10B, F4): convert typed elements from a stated plane, centre and date.
- **K13 pairs in the record** (section 10A): a parent may be a pair; every orbit two-sided about its centre with both masses; "binary" as a derived label; a re-parent op with the two keeps of R4; `withdraw` with the three choices for what orbited it; a preview op that returns the tree an edit would make, with verdicts; Pluto and Charon, Alpha Centauri and Kepler-16 as catalogued fixtures. The kernel thread answered on 24 Nov (see 10A): it proposes Jacobi hierarchy with `orbit.host` kept as one body, so "a parent may be a pair" reduces to "host plus what lies inside"; hosts must be checked on write, and no body may be silently unplaced.
- **K8 a surface inventory by substance** (`surface.inventory.<substance>`, with `body.surface_water_oceans` as its water row): phase from the Look up block, coverage from the ground's own height spread, vapour handed to the air.

Chart room (thread "Make the SSE4 test drive work end to end"), suggested packages, after navigation in stage 2:

- **ED0b the Solar System experiment, for real:** once ED1 to ED13 exist, build the Solar System through the UI alone, compare with `packs/base-sol` value for value, and log what worked, what was too hard and what broke (10B).
- **ED13a** the system sheet: a row per body, paste a table or a file, the measured-from, centred-on and true-on header.
- **ED1a** the units engine: `say` and its parser in `ui-contract`, reading the sheet's Units and the new "How figures are said" table; every existing figure moved onto it; click a unit; the choice remembered per kind and what it is about.
- **ED10a** survivability bands: the Humans profile and the protections list as base pack data, the composed verdict on a world's card and on a surface site (kernel K14).
- **ED1** the handle sheet, the handles row and the strip on the card, with exact answers on pause (no sweep yet). Walk step "Change a world" becomes: pick Mars, Build, drag Orbit.
- **ED2** the sweep behind the strip: tipping points, pencil figures, "what followed".
- **ED3** dragging on the chart: orbit size, shape, readout, magnetism, cancel.
- **ED4** the ghost for Add, Paste and Duplicate; drop on a parent; paste onto a body.
- **ED5** Reach it and Pin it (with gap 4 and gap 12).
- **ED6** Kinds and the star chooser; changing a star with worlds; a companion star.
- **ED0, before the rest:** the Solar System fixture with pairs and the checked list of structural edits (kernel K13 first), then the family tree in the ghost's panel and in the selector.
- **ED13** the sheet, "Type it all in", with paste and the catalogued mark.
- **ED12** the Orbit and Spin boxes in full: shape, position ring, lean, retrograde, resonance chips, day, tilt, lock chips.
- **ED9** the Makeup box: bulk bar, live cutaway wedge, heat and redox strips, research-is-divided choices.
- **ED10** "Could be…" recipes, requirement profiles (Humans), and the GM-only "What does not add up" list.
- **ED11** "Your setting's data" in Workshop and Edit-in-context for definitions (with the data sheet and track RK owners).
- **ED8** the Air and Liquids boxes: mix bar, trace chips, phase bars (after K7 and K8, or before them with the honest note).
- **ED7** moons in full; Explore's read-only strip and "Try changing it"; phone dock.

## 16. Open questions, each with a recommendation

1. **The short list.** A world shows six handles (Orbit, Mass, Makeup, Liquids, Air, Spin), a star one (Mass), a system three (Age, Metals, birth chemistry at Science depth), and everything else is changed through Reach it or Pin it. *Recommend yes.*
2. **The strip replaces standing sliders:** it opens from the figure, one at a time, and takes no room when closed. *Recommend yes.*
3. **Radius becomes a result** of mass and makeup for every world, hand-made ones too (today a hand-made world takes a typed mass and a typed radius). A typed radius is then a pin. Needs kernel work. *Recommend yes.*
4. **Honesty marks set themselves:** a hand-set input inside the physics is "authored"; "impossible" is only for values past a wall or pinned results that contradict the physics. Today "Something else" in Add here marks every hand-made world impossible. *Recommend yes; it stops the mark meaning "a person touched this".*
5. **Changing a star that has worlds** keeps the worlds by default and offers "Grow the system again" on a what-if branch. *Recommend yes.*
6. **Edits on Build mean "as it has always been"** unless you choose "From this date" in the strip's menu. *Recommend yes; "from this date" is the rarer, story-driven case.*
7. **On Explore, a figure opens its strip read-only** with "Try changing it", which opens an example branch. *Recommend yes; it lets a newcomer play without being on Build and without risk to a campaign.*
8. **Kinds:** every atlas class carries a "make one" recipe, shared with the class gallery's sample, offered in Add, the star chooser and the class pages. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
9. **`sweep` as a kernel op** open to Orac and to "why?" curves, not private to the editor. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
10. **Light magnetism while dragging** (tipping points, round figures, resonances; Alt turns it off). *Recommend yes, weak, and named as it catches.* **Alex 24 Nov 15:13: yes.**
11. **Reach it before Pin it:** on a result, the app offers the physical route first and the forced one second. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
12. **Editing several bodies at one time** (select four moons, move them together): *recommend waiting until after 4.0;* copy and paste covers most of it. **Alex: dropped, no use case. Not planned.**
13. **Moons:** generated systems already get their moons from formation's leftovers (F5). The ask is only that Add a moon can put the same question for one chosen planet (K5), with the three rules as fixed recipes until then. *Recommend yes.* **Alex queried this 24 Nov 15:16; corrected and re-asked.** **Alex 24 Nov 15:21: yes.**
15. **Air is one handle:** an amount strip, one mix bar whose joins you drag, traces as chips in ppm, each gas with its own log strip on demand. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
16. **Drop SSE3's "ages or stays as authored" switch for air:** the verdict gives the loss rate and Look ahead applies it. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
17. **Liquids replaces Water:** you set how much of any substance; whether it is liquid, ice or vapour is a result, and the phase bars show why. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
18. **Coverage stays draggable** as marks on the amount strip, with a typed share as a Reach it. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
19. **Custom liquids are substances on the data sheet,** in a pack, marked fictional when they are; no separate liquids list. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
20. **Makeup is one handle whose box holds a bulk bar, a live cutaway, and heat and redox strips at Science depth;** layers are results. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
21. **A buried sea (and plates, a field, volcanoes) is asked for through "Could be…",** never painted in as a layer; Pin it remains for those who just want it so. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
22. **"Could be…" on every world's card:** separate optional steps, dependencies shown, what skipping costs stated, one line, can be silenced per world. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
23. **Requirement profiles as survivability bands** (section 11C): one band per hazard with comfortable, tolerable, needs protection and nothing saves you; the verdict composed with reasons and the protection named; protections as data; Humans shipped in 4.0 with cited physiology; 4.1 species bring their own bands. *Recommend yes.* **Alex 24 Nov 15:20 asked for this in place of "three grades"; the hazard list is his to add to.** **Alex 24 Nov 15:21: yes.**
24. **One GM-only list per world, "What does not add up",** with a mend button on each line. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
25. **Definitions (gases, liquids, drives, sensors, calendars, tags) are edited as records with the same handles,** in context and under one Workshop entry, "Your setting's data". *Recommend yes.* **Alex: changed. A world's boxes only pick what is defined; "Can't find it?" leads to Reference or to Orac (section 11E).**
26. **A handle is a subject with a box of parts:** the row stays six, and Orbit and Spin get the parts in section 11F (shape, where it is now, lean, retrograde, in step with a neighbour; day, tilt, locked). *Recommend yes.* **Alex 24 Nov 15:13: yes.**
27. **Oblateness is a result** shown at Science depth with Reach it and Pin it, not a handle. *Recommend yes.* **Alex: a result, shown only. No Reach it, no Pin it; turn the spin down to reduce it.**
28. **Typing is a standing rule:** every draggable figure types, exactly as typed, past a strip's end if need be; plus a per-body sheet that takes pasted data and can mark it catalogued. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
29. **Always a pair, no 8% threshold:** every orbit is two-sided about its centre, and "binary" or "double planet" is a label for when the centre lies outside the heavier body. *Recommend yes.* **Alex: "?". Explained in the thread; waiting.** **Alex 24 Nov 15:21: yes.**
30. **A world can go round one body or round both of a pair,** done as the kernel thread proposes (host stays one body; the centre includes whatever lies inside the orbit), and the centre is never something the user creates or manages. *Recommend yes.* **Alex: wants the centre hidden and both orbits adjustable. Answered by "Editing a pair" in 10A; waiting on his read.** **Alex 24 Nov 15:21: yes.**
31. **The five rules for structural edits** (show the tree first; nobody else moves unless you say; the drop decides the parent; tidy orbit or keep its motion; one dated Undo). *Recommend yes.* **Alex 24 Nov 15:13: yes.**
32. **Pairs are proved on the default Solar System first** (Pluto and Charon, plus Alpha Centauri and Kepler-16 as starters, with a checked list of structural edits in CI) before editing packages ship. *Recommend yes. CI gains tests only; no workflow change.* **Alex 24 Nov 15:13: yes.**
33. **One editing grammar** (section 3B): every editor is built from the same fourteen parts, a new editor is data in the handle sheet, and a UI check fails anything that adds its own slider or dialog. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
34. **One units engine:** a single `say` function for every figure on every screen, with a parser that reads the same forms back. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
35. **Click steps the family** (Scientific, SI, Imperial last, the setting's), the engine picks the rung inside it, and the choice is remembered per kind and per class of object, saved with the campaign. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
36. **The rules are an editable table on Reference,** "How figures are said", and schemes are named sets of it. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
37. **Scientific by default at every depth,** per the table in 3C; words before prefixes, exponents only when nothing else fits. Imperial is never a default and a ton is always a tonne. *Settled by Alex 24 Nov 14:56; the table itself is for him to correct.*
38. **Removing a body that has things round it:** this note's default is "take them with it" (a deleted planet does not leave stray moons); the kernel thread's is "leave them, moving as they are". Both are always offered. *Recommend take them with it for a plain delete, leave them when the removal is a dated event in play (it was destroyed).* **Alex 24 Nov 15:13: yes.**
39. **Marks open what they refer to:** clicking authored, catalogued, impossible or fictional lists the exact values behind it, and each line leads to its handle. *Settled by Alex 24 Nov 15:04.*
14. **Order:** ED1 to ED3 straight after the navigation items in stage 2, since "change a world" is in the stage 1 walk and is the first thing a tester tries; ED4 to ED7 after. *Recommend yes.* **Alex 24 Nov 15:13: yes.**
