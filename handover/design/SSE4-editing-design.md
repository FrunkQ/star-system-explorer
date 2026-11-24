# SSE4 editing: handles, strips and the chart (track ED)

Status: **proposed, 24 Nov 2026; Alex on first read, 14:42: "love this! what about atmospheres on the sliders? water -> liquids? SSE3 is our starting point there" (section 11A answers that). Waiting on his answers to the questions in section 16.**
Mockup: `/mnt/project-files/sse4/handover-screens/ED-screens/editing-mockup.html` (it works: drag the strip, drag the world on the chart). Screenshots beside it in `ED-screens/`: `desk-1-mass-strip`, `desk-2-what-followed`, `desk-3-drag-orbit`, `desk-4-star-kinds`, `desk-5-companion-night`, `desk-6-moon`, `phone-1-mass-strip`, `phone-2-drag-orbit-night`, `phone-3-star-kinds`, and for section 11A `desk-7-air`, `desk-8-liquids`, `phone-4-liquids-night`. The mockup's figures are stand-ins, not kernel output.

Asked for by Alex, 24 Nov: "Editing planets is a BIG need... Earlier versions went big into sliders, although we really like them because it encourages people to play with planets and know the range that they can play in. They do take up a lot of room, are relatively cumbersome, and the act of sliding them does tend to slam the game engine. We still want to encourage play... there's probably not that many things they can change now... generally there's only going to be a few sliders that matter. What we need to do is ensure when the user sits there and kind of goes, I really want a different type of star or something like this in the system, that they can add it and have a bit more control and are just allowed to do what they want and are guided if it's unfeasible." And a minute later: "dragging planets to move them is a great way to avoid a slider and do it fine... but consider moons and cut and paste, etc. We got a good set of tools in SSE3... maybe some cumbersome or awkward, so now is the time to fine tune and improve."

This note builds on what is planned and does not restate it: V3 gaps 1 (where things can orbit), 2 (Add here, built), 3 (grow with the four dials), 4 (pin a value), 5 (click a unit), 11 (mix editor), 12 (pin a class), 38 (press-and-hold menu), copy and paste (U20, built), the stability verdict (ST1, built) and "the science expects" for air. It gives them one shared surface.

## 1. The idea in six lines

1. **A handful of handles.** In SSE4 a world has about six things you can truly set, and a star has three. Everything else is worked out. The card shows the handles as one short row; nothing else on the card is a slider.
2. **The slider lives inside the number.** Press a handle and a **strip** opens from it: a range line with the feel of a slider, which shows where you are, where the physics would put you, where real worlds sit, and the exact points where something changes ("oceans boil here", "keeps hydrogen from here", "this is a star from here"). One strip open at a time. Closed, it takes no room.
3. **The chart is the first editor.** Orbits are changed by dragging the world. Moons the same, one level down. Adding, pasting and moving a body to a new parent all use the same ghost that follows the pointer and says how it would fare before you let go.
4. **Dragging is free.** While you move, nothing is written and the heavy physics does not run. The strip was sampled once when it opened, so the figures follow your hand from that. Letting go makes one change, one undo step, and the card shows **what followed**.
5. **Never refused, always told.** Every limit is a labelled mark you can pass, never a stop. A one-line verdict says what would happen and offers one way to make it work.
6. **For any result you want, two honest routes.** **Reach it** (change an input until the physics gives it, and it stays real) or **Pin it** (force it, and it carries the honesty mark). The app offers Reach it first.

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
| Makeup | `makeup.metal`, `.rock`, `.carbon`, `.ice`, `.gas` (shares) | one mix control (gap 11's mix editor), not five sliders |
| Liquids | `body.surface_water_oceans` today; any substance in section 11A | how much of each; which are liquid, and how much of the surface they cover, are results |
| Air | `atmosphere.surface_pressure`, `atmosphere.mix` | how much and what of; "the science expects" beside it; section 11A |
| Spin | `body.rotation_period`, `body.obliquity` | day length and tilt; expected values come from tides and history |

**Results, not handles** (all are V3 sliders today): radius, density, gravity, escape speed, year, surface temperature, albedo, greenhouse warming, tidal and inner heat, magnetic field, ocean coverage, class, a star's radius, temperature, luminosity and spectral type. To change one of these you use section 8.

Two honest notes on the kernel as it is:
- A world added or imported by hand carries mass **and** radius, both typed (`bodies_from_facts` reads both; "Something else" in Add here asks for both). A world that formation made gets its radius from its mass and makeup. This design wants the second behaviour for every world (question 3).
- A hand-added world gets a fixed core share (0.32) and no water share. Makeup needs to reach `bodies_from_facts` in full before the Makeup handle is honest for hand-made worlds (kernel ask K3).

So the count is 3 for a system, 1 for a star, 6 for a world. V3 had about 27 sliders across ten tabs for one body.

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

## 4. The chart is the first editor

On the Build desk, bodies drag (U14 says so already). This is what dragging does:

- **Drag a world** round its star: its orbit changes size. A ghost and its new orbit line follow the pointer; the world itself stays put until you let go. A small readout rides beside the pointer: the distance, the year, the surface the quick look gives, and the stability verdict (lasts, watch, unstable) from the ST1 blocks. The radial under the pointer shows the same tipping points and walls as the Orbit strip, and the "Where things can orbit" layer (gap 1, gap 40) turns on by itself for the length of the drag, so the neighbours' no-go bands and the zone lines are visible while you choose.
- **Drag the far point** of a selected orbit (a small handle at apoapsis) to change how stretched it is. Drag the handle round the ring to turn the ellipse.
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
7. **The chart redraws one layer.** During a drag only the ghost, its orbit line and the readout move.

Reading between samples is presentation of kernel results, not a physics shortcut: every sample is the real chain, and the settled figure after a pause or a release is exact. It still gets a line in the fudge register so it is on the record.

Budgets to add to the performance readout (PF1), as a chunk "Editing": drag frame 8 ms; strip usable 300 ms after opening on the standard tier; release to settled card 200 ms.

## 7. Guided, never refused

"Danger is drama, never forbidden" applies to building as much as to play.

- **Walls are marks.** The handle passes through every wall. The label on the far side says what the physics says happens there. A world dragged inside the star is "inside Kestrel: gone in days". A planet's mass pushed past the point where deuterium burns gets the label the atlas gives it there, and past 0.075 solar masses it is a star, on the same record (one body, B1).
- **The verdict offers one fix,** and the fix is always a move of a handle, never a refusal: "Move to 0.61 AU, where it lasts", "Thicken the air to 0.3 bar and the water stays liquid", "Make the companion 40 AU away and all six worlds last".
- **Things that do not go together** are said as advice (the atmospheres ruling), on the verdict line of whichever handle is open: "No free oxygen lasts under this much hydrogen."
- **Honesty marks set themselves** (question 4). A hand-set input inside the physics is *authored*: plain dated data. It becomes *impossible* only when it is past a wall where the model has no answer, or when it is a pinned result that contradicts the physics. The GM can still tick impossible by hand. *Fictional* comes from packs, as now.
- **The stability verdict is the same one everywhere:** in the drag readout, on the ghost, on the card, and under the Orbit strip, from the same ST1 blocks. "Unstable" always offers "Run it and see" (N-body on request).

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

## 11. Moons

- Everything in sections 3 to 8 applies to a moon unchanged; its Orbit strip runs from Roche to Hill.
- **Add a moon** today offers only "Something else", because moons are not sampled from formation yet (test drive rough edge 5). Ask K5 gives "What forms here" for moons. Until it lands, Kinds supplies three recipes sized from the planet by physics: a disc moon (about one ten-thousandth of the planet), an impact moon (up to about a hundredth, rocky worlds only), a captured body (small, tilted, far out, often backwards).
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
- **K5 moon formation sampling** for "What forms here" round a planet.
- **K6 honesty rule** of question 4: authored unless past a wall or contradicting the physics.
- **K7 authored air drives the climate**: pressure and mix feed the greenhouse and air optics for hand-made and catalogued worlds, and "what the science expects" is readable for both amount and mix.
- **K8 a surface inventory by substance** (`surface.inventory.<substance>`, with `body.surface_water_oceans` as its water row): phase from the Look up block, coverage from the ground's own height spread, vapour handed to the air.

Chart room (thread "Make the SSE4 test drive work end to end"), suggested packages, after navigation in stage 2:

- **ED1** the handle sheet, the handles row and the strip on the card, with exact answers on pause (no sweep yet). Walk step "Change a world" becomes: pick Mars, Build, drag Orbit.
- **ED2** the sweep behind the strip: tipping points, pencil figures, "what followed".
- **ED3** dragging on the chart: orbit size, shape, readout, magnetism, cancel.
- **ED4** the ghost for Add, Paste and Duplicate; drop on a parent; paste onto a body.
- **ED5** Reach it and Pin it (with gap 4 and gap 12).
- **ED6** Kinds and the star chooser; changing a star with worlds; a companion star.
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
8. **Kinds:** every atlas class carries a "make one" recipe, shared with the class gallery's sample, offered in Add, the star chooser and the class pages. *Recommend yes.*
9. **`sweep` as a kernel op** open to Orac and to "why?" curves, not private to the editor. *Recommend yes.*
10. **Light magnetism while dragging** (tipping points, round figures, resonances; Alt turns it off). *Recommend yes, weak, and named as it catches.*
11. **Reach it before Pin it:** on a result, the app offers the physical route first and the forced one second. *Recommend yes.*
12. **Editing several bodies at one time** (select four moons, move them together): *recommend waiting until after 4.0;* copy and paste covers most of it.
13. **Moons:** ask the kernel for formation sampling round a planet (K5), with the three physics-sized recipes as the stand-in until then. *Recommend yes.*
15. **Air is one handle:** an amount strip, one mix bar whose joins you drag, traces as chips in ppm, each gas with its own log strip on demand. *Recommend yes.*
16. **Drop SSE3's "ages or stays as authored" switch for air:** the verdict gives the loss rate and Look ahead applies it. *Recommend yes.*
17. **Liquids replaces Water:** you set how much of any substance; whether it is liquid, ice or vapour is a result, and the phase bars show why. *Recommend yes.*
18. **Coverage stays draggable** as marks on the amount strip, with a typed share as a Reach it. *Recommend yes.*
19. **Custom liquids are substances on the data sheet,** in a pack, marked fictional when they are; no separate liquids list. *Recommend yes.*
14. **Order:** ED1 to ED3 straight after the navigation items in stage 2, since "change a world" is in the stage 1 walk and is the first thing a tester tries; ED4 to ED7 after. *Recommend yes.*
