# SSE4: the grand zoom, and travelling between places (track GZ)

Status: **ACCEPTED by Alex, 25 Nov 2026 12:45 UTC** ("All your suggestions are good - see no real issues apart from discussion above - so yes to all... once you have manipulated them based on above"). His changes are in section 14A, which wins where it differs from the rest: the tie-up of the levels and the Local Group come first, the gazetteer of worked-out places is kept with the campaign, and the Run desk gets the plain, clean form of this zoom. Design and mockup only; nothing here is built by this note. Packages are in section 13, in the new order. GZ is a code only, never shown in the product.

Mockup, clickable: project files `sse4/handover-screens/GZ-screens/grand-zoom-mockup.html` (also published privately to Alex as the artifact https://claude.ai/artifact/662RbMkMjyz2CZSEm5CKAb), with 17 desktop and phone screenshots beside it: the seven stops, two seams, a pressed galaxy, and the proposed Galaxies nearby view at both reaches and both scales. The pictures in the mockup are painted stand-ins, not SSE4 output; the mockup says so.

Alex's brief, in his words (25 Nov 12:05 UTC): "what is the capability of doing the grand zoom to show off the true scope of the software? ... when you build a postcard or a bookmark and you use them, you'll have the opportunity to zoom to location. And then you can choose where you start ... At each stage, pause, show the HUD, and on a click, it zooms down to the next level ... it could be one long, smooth zoom from top to bottom with no text at all, or it could be it goes each level, pauses, shows you what's around, allows you to click on them to see what they are, and then zoom down to the next level ... effectively from galactic filament to lava tube on a planet ... we can do a little bit of fudging at the transition, but because everything should align, in theory, this should work. Or am I whistling Dixie?" And: "Our level between galactic filament and our galaxy, the local galaxy group, is pretty poor at the moment and will need a lot of work."

Read with: `SSE4-top-level-ui-and-onboarding.md` (the place line in 2.1, tours in 3.2), `SSE4-playbook-and-switchboard-design.md` (bookmarks in 4A, presets in 4.2, who steers in 5.3), `SSE4-postcards-design.md` (the camera on a bookmark), `SSE4-galaxy-design.md` and `SSE4-galaxy-histories-design.md` (haloes, galaxies, satellites), `SSE4-visual-polish-design.md`, `interface-style.md` (rules 7 to 12), `SSE4-secrets-and-visibility-design.md`.

## 1. The short answer

1. **It is feasible.** The two ends exist and most of the middle does. A bookmark already holds the place, the date, the instrument, the view and (since the postcards work) the exact camera. The place line already runs cosmic web, galaxies nearby, galaxy, system, world, and a place link already carries a landing site. One kernel and one clock answer every level.
2. **What is new is small in idea: a change of place can be travelled instead of cut.** Going to a bookmark today is a cut. This note adds one property to the *use* of a bookmark, the **way in**: where the journey starts, whether it stops at each level, and what is shown on the way. The grand zoom is the longest way in. The same mechanism, in a short form, makes every step on the place line a zoom rather than a swap.
3. **It cannot be one 3D scene, and does not need to be.** The range is about 10²² to 1 in size. Each level keeps its own view and its own renderer. At each change of level the outgoing view zooms onto the target, the incoming view opens already aimed the same way with the target on the same pixel, and the two cross-fade while both keep zooming. That hand-off is the one new fudge, and it is registered (section 9).
4. **A stop is the real screen, held.** At a stop the level is the ordinary view of that level, so everything in it can be pressed and opens its ordinary card. Nothing is a film.
5. **The levels are tied together, with three gaps.** Web, halo, galaxy, star cell, star and formation are one computed chain: a star's birth date and metals come from its own ring of its own galaxy, and formation inherits them. The gaps are that our galaxy is not yet one of the web's own galaxies, that systems not grown from a drawn star have no cell, and that the screens do not pass the place along. Section 6 closes all three, computed once.
6. **The Local Group level needs rebuilding, and the facts for it are already worked out.** The kernel already gives every neighbour galaxy a broad type, a size, a tilt, a colour and its star formation, and gives our own satellites their orbits. Only the night sky reads them today. Section 7 turns "Galaxies nearby" into a 3D view drawn from those facts, with real histories for the biggest two or three.
7. **A Motion layer on the HUD** (Alex's ask): velocity and pull as arrows with the speed said, on every level, from one mechanism (section 8A).
8. **About sixteen packages**, in an order where the first three already give a zoom from system to site that is true all the way (section 13).

## 2. Words

| Word | Means |
|---|---|
| **Way in** | How a place is arrived at: cut to it (today), or travel to it. A small piece of data beside a bookmark where it is used. In the product the control reads **Zoom in from** |
| **Journey** | One played way in: the list of levels passed, the camera's path through each and the hand-offs between them |
| **Rung** | One level of the place line that the journey passes through |
| **Stop** | A rung where the journey holds until told to go on |
| **Seam** | The hand-off between two rungs: both drawn, cross-fading, both zooming |
| **Target** | The thing the journey is going to, at each rung: the web's knot holding our galaxy, our galaxy among its neighbours, the star in its galaxy, the world in its system, the site on its world |
| **Grand zoom** | The journey from the cosmic web to a site. The name of the shipped preset and of the trailer shot, not of the mechanism |

## 3. What already lines up (checked on main, 25 Nov 2026)

| Thing | Where | State |
|---|---|---|
| The ladder of places and its deep link | `ui-contract/src/place.ts` (`PLACE_SCALES`: web, cluster, galaxy, system, world; `placeHash` with `~surface?site=lat,lon&time=&lander=`) | Built. No "local stars" rung and no "site" rung yet (the site rides on the world's link) |
| The end point | `ui-contract/src/bookmark.ts` (`sse.bookmark/1`: scope, date, instrument, scale, view, selected, `pose`), `pose.ts` (eye direction and distance in the body's frame, up, field of view, exposure, kept box), `postcard.ts` | Built (PC1, PR 509). The globe and the 3D system chart keep their camera |
| One clock, one kernel | every view reads the same session at the campaign's now | Built |
| The shared view controls | `ui-contract/src/view-controls.ts` (View style, Seen from, Scale, HUD, Band; a view declares what it draws) | Built. This is what "show the HUD or not" switches |
| Cosmic web in 3D and True | `views/foam-map/src/Web3D.svelte`, `renderer/web-cube.ts`, kernel `web_slice` with `cube`, `haloes` and a `centre` | Built (V9w to V9w3). A cube takes about 4 to 8 s to read at 48³. The view is centred on the origin today |
| Galaxies nearby | `views/cluster-map` on `halo_cell` (one 8 Mpc cube) | Built as a flat chart only: circles by halo mass, height flattened, stepped cube by cube. Section 7 |
| Each neighbour galaxy's look | kernel `galaxies_seen` (`sse-kernel/src/galaxies.rs`): morphology (dwarf, irregular, disc, barred, lenticular, elliptical), stellar mass, star formation, half-light radius, axis ratio, tilt, colours, magnitudes, for catalogued haloes within 12 Mpc of home; `satellites_seen`: our own satellites from the assembly tree, each on its orbit under sinking, tidal stripping, gas stripping and disc shocks | Built for the sky (Y4 parts 1 and 2). Not drawn at the galaxies nearby level |
| The galaxy in 3D and True | `views/galaxy-3d/Galaxy3D.svelte`, `renderer/galaxy-true.ts`, `galaxy-marks.ts` (warp, ripples, scars); HUD draws the Sun's orbit | Built. Draws the campaign's galaxy or any catalogued halo |
| Stars around home | `views/galaxy-chart/HomeCell.svelte` on `star_cell` (the 10 pc cell: each star's place in parsecs, mass, companions, metals); the voyages star map (`views/voyage/Stars.svelte`) | Built, flat, inside "Your systems". Not a rung |
| System to world | `views/system-3d/System3D.svelte` `flyTo`: a press on a body flies in with two eased moves, and the frame changes from the star's to the body's while the eye is close | Built. This is the first seam, already working |
| World close up | `renderer/close-up.ts`: True comes down to 3 km over the ground and tips toward the horizon; the detail ladder brings finer tiles | Built (25 Nov) |
| World to ground | `views/surface/Surface.svelte`: choose a site, the kernel flies the lander down (`plan_descent`), then the ground at eye height over a few km; lava tube openings cut into the ground from `survey_caves` (`renderer/cave-mouths.ts`) | Built. Going inside a cave is parked to about 4.2 (interior sites) |
| How a screen arrives | `ui-contract/src/delivery.ts`, `chart-grammar/deliver.ts`: a look's deliveries for five moments, a full and a short form, reduced motion | Built (AT15a). The same full and short idea is used here |
| Firing at players | Playbook presets with a Show step naming a bookmark; each screen draws on its own kernel as its role knows it | Built (PB3, PR 421) |
| A clean picture for film | the clean output link per role (Playbook design section 9), the capture rig `scripts/trailer/` | Built |
| The idea itself | Top-level design 3.2 (a tour is an ordered list of place marks) and its "Zoom all the way out" tour; trailer storyboard C, "Scale journey: from the filaments to a cave" | Designed, not built. Tours do not exist in code yet |

## 4. What is missing

1. **Nothing carries a camera across a change of level.** Each level is a screen with a camera of its own, and changing level swaps the screen.
2. **No single answer to "where is this place at every level".** The pieces are in different ops and in the shell. A journey needs, for one address and date, the target's position, size and orientation in each parent's frame.
3. **Our galaxy is not one of the web's galaxies.** The campaign's galaxy is the median halo of its mass (`named_halo`: "milky-way-like", 10¹² suns), grown and fed properly, but set at the centre of cell 0, and any web halo within 0.3 Mpc is taken to be us (fudge F37).
4. **A system that was not grown from a drawn star has no cell.** The shipped sample system, V3 imports and hand-built systems carry no galactic place; when one is re-aged it is put at the Sun's radius, azimuth 0 (`reconstruct_origin` place). "Your systems" already refuses to invent a place: it lists such systems beside the map. A system grown from a drawn star (`from_star`, PR 546) has a real one.
5. **Galaxies nearby is flat and stepped by cube.** No 3D, no True, no types, not centred on us (section 7).
6. **Two rungs are missing from the place line:** the local stars between galaxy and system, and the site below the world.
7. **Two views may disagree about where a galaxy is.** The web cube moves each halo by the cube's own truncated Zel'dovich step; the cluster map draws `position_mpc` from the halo cell. If those differ, galaxies would jump at the seam. To be checked in GZ0 and, if they differ, settled as one position per galaxy at every level.
8. **Tours are not built**, so there is nowhere yet to keep a journey with authored words.

## 5. The mechanism

### 5.1 The way in, as data

One small shape, `sse.approach/1`. It sits **beside** a bookmark wherever a bookmark is used, never inside it, so "bookmarks define what" stays true.

```
approach: {
  from: "web" | "cluster" | "galaxy" | "stars" | "system" | "world" | "here",
  stops: "each" | "none" | [rungs],         // stop at every rung, at none, or at the named ones
  go_on: "press" | { after_s: number },     // at a stop: wait for a press, or go on by itself
  hud: "at_stops" | "always" | "never",
  words: "captions" | "names" | "none",
  style: "true" | "chart" | "viewer",       // the view style travelled in
  pace: "brisk" | "stately" | "slow" | { total_s: number },
  ends: "camera" | "landing"                // the camera simply arrives, or the last leg is the kernel's flown descent
}
```

Left out, everything takes the default: from the rung above, no stops, HUD as the viewer has it, the short form. That default is the everyday zoom between two levels (5.6).

Where it can be attached:

| Place | How | For whom |
|---|---|---|
| A bookmark's row, a postcard's back, a picker row | "Go there" keeps its one press. Beside it, **Zoom in from ▾** lists the rungs above the place, then a line of four switches (Stops, HUD, Words, Pace). The last choice is kept on the device | Anyone |
| A Playbook Show step | `{ show: { bookmark, layout?, approach? } }` | The GM firing it at a role |
| A postcard | an optional `approach` on `sse.postcard/1`, set by the sender ("Arrive by zooming in") | Whoever opens the card or its link |
| A place link | `&from=web&stops=each` after the place | A shared link, a tour, the start sheet |
| A tour step (when tours are built) | a step is a place mark; its way in is this shape, with its own caption per rung | Lessons, briefings |

### 5.2 The ladder

The place line gains two rungs (question 3). In full, top to bottom, with what is drawn and roughly how wide the picture is at each:

| Rung | Words | View | Picture width |
|---|---|---|---|
| web | Cosmic web | the web cube, centred on home | 400 to 100 million parsecs |
| cluster | Galaxies nearby | the rebuilt view of section 7, two reaches | 24 Mpc, then 3 Mpc |
| galaxy | Galaxy | Galaxy 3D | about 60 thousand parsecs |
| stars (new) | Local stars | the home star cell in 3D (section 8) | about 20 parsecs |
| system | System | System 3D | tens to hundreds of au |
| world | World | the globe, then the close-up | a few radii, down to 3 km up |
| site (new) | Site | the Land screen's ground | a few km, down to eye height |

A journey between any two places runs **up to the lowest rung they share and down again**. From a cave on one world to a sea on another in the same system it rises to the system and descends. The grand zoom is the case that starts at the top. Zooming out is the same journey played the other way.

### 5.3 The camera

- The journey is a **pure function of journey time**: given the plan and a time, it returns each view's camera. Nothing in it reads a clock of its own. So it can be paused, stepped back, played at any frame rate, and filmed frame by frame (the trailer rig asks for time t and gets exactly that frame).
- Within a rung the camera zooms at a **constant rate in powers of ten**: the picture's width shrinks by the same factor every second. "Stately" is about two seconds for each factor of ten. Web to site is about 22 factors of ten, so about 45 s of motion.
- The line of sight eases, during each rung, from that rung's good viewing angle to the angle the next rung will open at, so there is never a jump in direction at a seam.
- At the bottom the camera ends **exactly on the bookmark's pose**. A postcard's journey therefore ends on the picture on its front.
- Each view that can be travelled through answers three questions through one small addition to `ViewModule` (the "journey port"): *put your camera here* (target, width, direction), *are you ready* (its reads are in), and *what is at the middle of the frame* (for the caption and the place line). Views keep their own renderers and their own camera code.

### 5.4 The seam

When the outgoing rung's picture has narrowed to the seam width for that pair of rungs:

1. The incoming view has been mounted underneath and read ahead. Its camera is set so the target sits on the same pixel and the line of sight is the same.
2. Over about a third of a second, and never stopping the zoom, the outgoing canvas fades out over the incoming one. Only `opacity` and `transform` are animated (interface rule 8), and the two views are two canvases, so no renderer has to be merged with another.
3. The outgoing view is let go once the seam is past. Stepping back up makes it again from kept data.

If the incoming view is not ready when the seam comes, the journey **holds on the outgoing rung** as if it were a stop, with one quiet line ("Reading the galaxy…"), and goes on when it can. It never stalls half-faded. A journey played for film reads everything first.

### 5.5 A stop

At a stop:

- The zoom eases to rest and the HUD comes up (names, outlines, the scale bar), unless the way in says never.
- The **target is ringed** in the accent, so "where am I going" is always plain. With HUD and words off there is no ring: only the picture.
- A **caption** opens beside the target, one or two sentences made from the level's facts through the one figures door: "The galaxies nearby. Forty-one within five million light years. Yours is the second largest, and its big neighbour is falling toward it: they first meet in about four billion years." A tour or a preset may carry its own words per rung instead, and Minerva can draft them.
- **Everything is live.** It is the ordinary view of that level. A press on a galaxy, a star or a moon opens its ordinary card, with "Why?" and the rest. Drag turns the view (Free). Nothing is blocked.
- **The place line is the progress bar.** Its steps are the rungs: those passed in ink, the one in view in the accent, those to come pencilled (the line already draws steps "below" quietly). No new chrome.
- **Go on** is one accent button at the end of the place line, and Space or Enter. **Back up** is the step to its left. Pressing any step of the place line goes to that rung. Esc, or any ordinary navigation, leaves the journey and leaves the viewer where they are, on the ordinary screen.
- If the view was turned, going on first eases the camera back onto the line (under half a second).

"On a click it zooms down" is therefore a press on Go on, not on empty space, because at a stop a press on a thing means "what is this?".

### 5.6 The short form: every change of level travels

The place line, the picker and "Chart this galaxy" all change level by swapping the screen. With the journey port in place they play the same seam in a short form: about 0.6 s, one rung, no stop, no words. It is the existing `flyTo` of System 3D made general. Reduced motion turns it into the plain fade used everywhere else. This is where the mechanism pays every day, not only in a showpiece (question 2).

### 5.7 During the zoom

- **One line of HUD** while moving (when HUD is on): the width of the picture, counting down, in the viewer's units and as a power of ten: "30 million light years across · 10²³ m". It is the cheap correct touch a clever user can check against the scale bar at each stop.
- **The universe clock does not move.** The journey happens at the bookmark's date, or the campaign's now. Views that turn by themselves (the web cube's slow turn) are held to the journey's line.
- **Scale.** The journey travels at Real scale wherever the view draws Real, because exaggerated sizes cannot line up across a seam. At a stop a view may ease to Readable if the viewer has it chosen, and says so on its scale chip, as it does today.
- **Exposure.** Each True view is metered by itself, as now. At a seam the incoming view's exposure is set so the frame's mean brightness matches, then eases to its own within a second.
- **Sound** stays off by default. A later package may give a rung change a tone from the sound design.

### 5.8 Players, knowledge and who presses

- A preset fired at a role sends the bookmark and the way in, nothing else. **Each screen plays the journey on its own kernel**, as its role knows the world. The GM streams only data, as ruled.
- **A role travels only from as high as it knows.** If the table does not know where in the galaxy it is (a misjump), a journey fired "from the cosmic web" starts for them at the system. The GM's preview says so before firing: "Starts at System for the Crew: they do not know where this system lies." This is the secrets design doing its ordinary work.
- **Who goes on.** By default the GM's Go on moves every screen of the role together (one small message naming the rung). The role card may instead let each screen go on by itself, where the role steers its own screen ("Theirs" in Playbook 5.3). At a stop a player may press things if their role may; what a card shows is what the role knows.
- Pressing the preset again while it plays takes it back, as with every preset.
- **OBS and trailers**: the clean output link plays the same journey with no chrome. Words none, HUD never, pace slow, read everything first: that is storyboard C in one take.

### 5.9 The three desks

Alex, 25 Nov 12:45 UTC: "Explore I see as a 'messing about' and exploring... what ifs. Edit is getting games set up and worldbuilding. Play is about keeping interfaces clean to support the GM at play time (and it will be the most visually arresting) - being able to use portions of this zoom to step up and down the hierarchy would be truly lovely - feel as if you are spanning the universe with a few clicks." And: "in Play the UI is a lot cleaner. Just visual controls rather than aging the universe or additional editing stuff." (His Explore, Edit and Play are the desks the product calls Explore, Build and Run.)

For travel between places that means:

| Desk | What a level shows | Travel |
|---|---|---|
| **Explore** | The level with its time controls and what-ifs: the ruler, Play on the clock, branches | The short form on every step; "Zoom in from" with all its options |
| **Build** | The level with its editing tools: add, move, make this home, find it a place, the gazetteer's names editable | The short form on every step; stops are where things are placed |
| **Run** | **The level plain.** The picture, the place line, the shared view controls (View style, Seen from, Scale, HUD with Motion, Band) and nothing else: no ruler for deep time, no editing, no ledgers. Everything above the system comes from the gazetteer, so nothing waits | **Stepping up and down the place line is the main way to move**, each step the short zoom, a held press on a step the long form with a stop. One press spans a level; a few span the universe. Firing the same at the table is a preset |

The Run desk's version is the default look of a stop on a journey: what section 5.5 calls "the real screen, held" is the Run desk's plain level.

What every display looks like on each of the three desks is wider than this track. It is its own design task, the thread "Design each display in three modes" (opened by the coordinator 25 Nov 12:46 UTC); this table is this track's input to it, and the seam (what Run shows at each stop, how a short zoom steps between levels) is agreed with that thread by message. **Its note, `SSE4-three-modes-design.md` section 4 (the level tables), is the authority on what a desk shows**; where it and this table differ, it wins. Agreed with it on 25 Nov 12:49 UTC, none changing the mechanism: Run still carries what the Playbook put there (the on-air chip, Send, Bookmarks, a card on a press); Run has a small campaign clock at System, World and Site and a date that does not move above them; on Explore "Make this home" and "Find it a place" are a try in pencil on a what-if and are kept only on Build; and, if Alex agrees to view choices remembered per desk, a journey's `style` defaults to the desk's remembered style.

### 5.10 Reduced motion, small screens, no GPU

- `prefers-reduced-motion`: no zoom. Each rung arrives with the plain fade and holds. "One smooth zoom" becomes a sequence of held rungs of about two seconds each.
- A device with no GPU, or one the speed probe places too low for True, travels in **Chart** style: the flat charts (web slice, galaxies nearby, galaxy chart, system chart, the world's map) zoom and cross-fade the same way. The mechanism is the same; only the port's answers differ.
- Phones: the place line is already two rows; Go on is a thumb-sized button at the bottom edge at a stop; the caption is a sheet that slides up from the target's side.

## 6. The chain of places: what is tied today, and the tie-up

Alex, 25 Nov 12:22 UTC, on reading "a system has no real place in its galaxy": "where does it get its initial components to accrete, etc? I thought we HAD tied these levels together - each is a point in the map above effectively - if not - we need a rethink on how to achieve this. Preferably without too much 'fudging' and be scientifically literate (short cuts are allowed... we are simulating the universe in a browser - we will get some slack!) But these things only need to be calculated once."

He is right in the main, and the first draft of this note put it too strongly. Read from the kernel:

### 6.1 What is tied today

| Link | How it is computed | Where |
|---|---|---|
| Web to halo | Haloes are drawn cell by cell on the web's own density, each with its mass, place, spin and merger tree | `sse-cosmos/haloes.rs` (G2), `assembly.rs` (GH1) |
| Halo to galaxy | The galaxy is grown on the halo's growth history and fed by its tree's members, each on its own orbit | `galaxy.rs` (G3), `feed.rs`, `infall.rs` (GH2, GH3) |
| Galaxy to place in it | The disc evolves ring by ring: each radius has its own star formation history and gas chemistry through time. The envelope gives, at any radius, azimuth, height and time: gas and star densities and the gas's [Fe/H], alpha, C/O | `rings.rs`, `envelope.rs` (G4) |
| Place to star | Space is cut into 10 pc cells. Each star in a cell is drawn from the envelope there: birth time from the ring's own history, mass, companions, [Fe/H], alpha, C/O, r-process and potassium-40 from the gas at birth plus scatter, its birth cluster's mass, and an orbit round the galaxy. A looked-at star is pinned and never redrawn. A halo star knows the torn-apart galaxy it was born in | `stars.rs` (G5, GH4), ops `star_cell`, `sky_from_here` |
| Star to system | Formation inherits the star's birth (population, mass, age, metals) and the system takes the star's address | `Stars::birth`, `from_star` on `run_formation` and `adopt` (PR 546, merged 25 Nov) |
| System to world to site | Orbits at the date, the world's pole, latitude and longitude, the kernel's tiles | resolve, `orbits_at`, `survey_surface` |

So a system grown from a drawn star is a point in a cell, the cell a point in a ring of a galaxy, and its starting ingredients are that ring's gas at its birth.

### 6.2 The gaps: three, and a fourth found on 25 Nov

1. **Our galaxy is not one of the web's galaxies** (section 4, item 3). The chain is whole from the galaxy down and whole from the web down to any catalogued galaxy; ours is the one galaxy that is not in the catalogue.
2. **Systems not grown from a drawn star have no cell** (item 4): the shipped sample, V3 imports, hand-built systems.
3. **The screens do not pass the place along**: the web view opens on the origin, Galaxies nearby on cell 0.0.0, and "Grow a system here" on a drawn star is not yet wired to `from_star`.
4. **A galaxy is in two places** (found by reading the code with the galaxy charts thread, 25 Nov 12:49 UTC; the size is not measured). `halo_cell` and `galaxies_seen` use a halo's starting point, never moved (`sse-kernel/galaxies.rs`, the `h.position` lines), so the cluster map and the sky agree with each other. The web cube moves each halo by the Zel'dovich step at the cube's time (`sse-cosmos/web_light.rs`, `web.displaced`). So between the web and the levels below it a galaxy jumps by its own displacement, which can be several Mpc.

### 6.3 The tie-up, each part computed once

1. **Home is chosen from the web's own catalogue when a universe is opened, and stored.** The kernel searches the cells outward from the origin for the catalogued halo that best fits the universe pack's "home wish" (by default the Milky Way wish that "Find a home" already uses and cites: mass, a quiet recent history, an early big merger, a bound large neighbour) and writes its address as a fact of the universe. Every op that today asks for the median "milky-way-like" halo (`galaxy_chart`'s default, `with_world`, `reconstruct_origin`, the sky, `star_cell`) reads that halo instead. F37 and the two `reconstruct_origin` and `galaxy_chart` stand-ins retire. It is one search of a few dozen cells, done once per universe; the answer is the same on every device. The GM can still choose another ("Make this home" on Find a home), and a Milky Way pack (G7) can pin the real one. This moves the core golden on purpose (the default galaxy changes) and goes through the core lane.
2. **Every system has a star slot in a cell.** A generated system has one already. For an authored or imported system the kernel finds one at adoption: if the system carries a map position (a V3 starmap), the cell there; otherwise the nearest cell to home with a drawn star of fitting mass, age and companions. The system becomes a pin in that cell and claims its share of the count, by the pin rule that exists, so nothing is counted twice. Its authored facts stay authored. Where they differ from what that place would have made (a metal-rich star in a metal-poor ring), the card says "unusual for its birthplace" through the check `reconstruct_origin` already has, as advice, never a refusal. Done once, at adoption; the GM can move it on Your systems.
3. **One read for the whole chain.** `place_chain {address, t}` returns, for each rung, the target's position, velocity, natural size and orientation in its parent's frame, with an honesty mark wherever something was authored over what the place would have given. The journey, the Motion layer (section 8A), the place line's names and "centre this view on where I am" all read it.
4. **The screens centre on the chain.** The web cube opens centred on home's cell, Galaxies nearby on home, the galaxy chart on the home halo with the system's place marked, Local stars on the system's cell.
5. **A galaxy's tilt from the web's tides.** Today each galaxy's tilt is a keyed isotropic draw. The web already computes the tidal tensor at any point. Tidal torque theory and the surveys agree on the trend: the spins of small haloes lie along their filament and those of massive haloes across it. The shortcut: draw the spin axis about the tensor's axes with a mass-dependent preference, as a cited editable block. It costs three eigenvectors per halo, once. It is also a small correct touch a clever user can test: discs along a thread tend to line up. (The citations for the trend are from memory and go through the citation check before the block is written.)
6. **Every galaxy's drift from the web's own displacement.** In the Zel'dovich step the kernel already takes, a parcel's velocity is its displacement times the growth rate and the expansion rate: one multiplication. That gives every halo a peculiar velocity, retires F37's "Hubble flow alone, so the nearest galaxies all recede", lets a near neighbour be blueshifted in the sky as Andromeda is, and feeds the Motion arrows at the two top levels.
7. **One place for each halo.** The displaced position at the asked time is the halo's place, and every level reads it: `halo_cell`, `galaxies_seen`, the web cube and `place_chain` return the same point (the starting point stays in the answer as a second field, for the catalogue's keys). It is the same displacement as point 6, so it is one package with it (GZ15), it is computed once per halo and time and kept in the gazetteer, and it moves the sky's golden on purpose. "Find a home" then ranks by the displaced places with no change to its screen.

### 6.4 The gazetteer: worked out once, kept, ready to roll

Alex, 25 Nov 12:45 UTC: "Most of the heavyweight processing is during the creation and sliding the time slider ... for stuff worked out, we can have placeholders in the database ready to roll, because we do need to name nearby galaxies even if we don't know what they look like (i.e. we cache the positions of everything at the current defined time... and at those timescales only 'human time' matters and that is all 'in system' and easily calced on the fly."

So the chain's upper levels are **worked out when a universe is made or its date is set, and kept with the campaign** as a gazetteer:

- **What it holds**, at the campaign's defined time: the home halo and the web cell it sits in; every catalogued galaxy of the neighbourhood (12 Mpc) with its address, name, place, drift and halo mass; home's satellites with their places; the home galaxy's place for each campaign system, and the stars of each system's own cell and its neighbours with their names, places and motions. A row is a **placeholder**: a name and a place are always there; type, look and history fill in when that thing is first looked at, and then stay.
- **Names come at once.** A galaxy is named by the naming track's scheme when its row is made, so "Varrow" exists on the map, in the picker and in a caption before anyone has charted it.
- **When it is redone.** Above the system nothing moves in human time: over a thousand years a galaxy drifts a few light-months and a nearby star a fraction of a light-year. So the gazetteer is good for the whole of a campaign's play. It is redone only when the GM moves the universe's date by deep time (the time slider on the Explore and Build desks), in the kernel's worker, with progress. Inside a system everything is worked out on the fly from the orbits, as now.
- **What it is not.** It is a cache of the kernel's own answers, keyed by the universe's seed, pack stack and date, never a second source of truth: thrown away, it comes back identical. A GM's authored change (a renamed galaxy, a moved system) is a fact in the record, not a cache row.
- **What it buys.** The Run desk never waits for the web or a galaxy; a journey's read-ahead (section 10) is a lookup above the system; the picker and Find can search galaxies and stars by name; the players' screens receive the rows they are allowed to know as data.

### 6.5 Each level drives the one below

Alex, 25 Nov 12:45 UTC: "that system has to be in a star system group, has to be in a galaxy, has to be in a galaxy group, has to be in a filament or node... each *should* drive the physics of the one below - so how have we managed to get this far without that link? It is concerning. (eg passing star and supernova and life impact etc... needs to know where we are to work)."

The physics does run downward today, and from a place:

- Supernovae, kilonovae, gamma-ray bursts, nucleus episodes, close passes by other stars and spiral-arm crossings are drawn **per cell of the galaxy and per bin of time** from the galaxy's own rates there (`sse-cosmos/events.rs`, G6). An event belongs to its cell, so two neighbouring systems see the same supernova at the same place, each after its own light-travel time. The life track reads that history.
- The rates are the ring's, the ring is the galaxy's, the galaxy is fed by its halo's tree, and the halo sits on the web's density.

How it got this far with a gap: the kernel was built from the top down (web, haloes, galaxies, rings, stars) and the app from a system up (the sample system, V3 imports, the chart room). The two met at "the campaign's galaxy", which was given a stand-in so both could go on, and at "the system's place", which defaults to the Sun's radius. Both stand-ins were written in the fudge register at the time (F37 and its neighbours) with "until G7" beside them. So a system that was not grown from a drawn star does get passing stars and supernovae, but those of the Sun's radius of a median galaxy, not of a place of its own. That is the real cost of the gap, and it is what GZ12 and GZ13 remove. From then on the rule is stated and tested: **no system without a star slot, no star slot without a cell, no cell without a galaxy, no galaxy without a halo in the web.** A test fails if any op falls back to a stand-in place.

One thing to confirm in GZ0 rather than assume: exactly which place each consumer (life history, the sky, the events op) passes for a campaign system today. This note read the events model and the defaults, not every caller.

### 6.6 What stays a shortcut (each in the register)

- The home galaxy is the best fit in a few dozen cells, not a constrained realisation of the real Local Group. The Milky Way pack replaces it with pins.
- An authored system's slot is the best-fitting drawn star, not a derivation of that system from the galaxy.
- The spin rule is a statistical preference about the tidal axes, not a torque integrated through the halo's growth.
- Velocities are linear (Zel'dovich); inside a group the bound-pair orbit and the satellites' stored orbits take over.
- A system's own plane within the galaxy stays a keyed isotropic draw. That matches what is observed.

## 7. The galaxies nearby, rebuilt

### 7.1 What is wrong today

The view is one 8 Mpc cube of the catalogue seen from above: circles sized by the cube root of halo mass, every galaxy's height thrown away, the web as one slice behind, and "next cube" buttons on x, y and z. It is an honest catalogue page. It does not look like a group of galaxies, it is not centred on us, and a galaxy on it has no look of its own.

### 7.2 What it becomes

One view, **centred on the galaxy in question** (home by default), drawn as a sphere of space, with all five shared view controls:

| Control | Here |
|---|---|
| View style | **Chart**: today's flat map, kept, now centred on the galaxy with the cube steps folded into a "Catalogue" disclosure. **3D**: inked marks in space with stalks to a ground plane (the same stalks as Your systems). **True**: each galaxy in its own light against black |
| Seen from | Above, Tilted, Edge-on (along the home galaxy's disc), Free |
| Scale | **Real**: galaxies at their true size against their true separations, which makes most of them points of light with a glow; that is the truth and worth seeing. **Readable**: each galaxy drawn larger by one stated factor so its shape reads; the chip says "galaxies drawn 8 times size" |
| HUD | Names; the dark halo of each big galaxy as a faint ring at its r200; a dashed line between a bound pair with "meet in about 4 Gyr"; the edge of the group, where its gravity gives way to the expansion; the threads of the web through the neighbourhood; a scale bar in light years |
| Band | Visible first. The kernel already gives magnitudes per band, so Infrared and Ultraviolet change each galaxy's brightness and colour honestly |

Two **reaches**, one press apart, and both rungs of a journey: **the neighbourhood** (12 Mpc, the catalogue's own reach, where the web's threads and the next groups show) and **the group** (about 1.5 Mpc, or the edge of the bound group once it is worked out).

### 7.3 How a galaxy is drawn

Three steps of detail, the same world at each, chosen by how big the galaxy is on screen:

1. **A point**: its colour from the kernel's colours, its brightness from its magnitude.
2. **A type picture**: a small picture made by rule from the kernel's facts: the broad type (dwarf, irregular, disc, barred, lenticular, elliptical), half-light radius, axis ratio, tilt and turn, colour, and star formation as blue knots. Class stays a label read off results: the type is what the kernel's `classify` already says of that galaxy's state. Two galaxies of one type differ by their own numbers and by a keyed pattern.
3. **The real galaxy**: Galaxy 3D's own disc from that halo's `galaxy_chart`, with its warp, ripples and scars. Used for home at once, for the biggest two or three neighbours when their charts have been read in the background, and for any galaxy the viewer presses "Chart this galaxy" on. The view says how far it has got: "Histories read: 2 of 3".

This is Alex's guess made exact: **broad types for everyone, battle-worn for the few that matter**, with the swap happening by itself as the reads finish. GZ0 measures what one neighbour's full chart costs; GH0 measured about 0.1 s on a laptop and 0.5 s on a phone for the galaxies of one tree at Chart level, so two or three big neighbours look affordable.

The tilt needs one care. The kernel draws each galaxy's tilt and turn as seen from home. The 3D view needs an axis in space. An axis that reproduces the same tilt and turn from home is chosen by rule, so the sky and this view agree and no golden moves.

### 7.4 Our own satellites, and theirs

- **Home's satellites** come from `satellites_seen`: the survivors as dwarfs at their places on their orbits; those torn apart as streams and shells (counted today, to be drawn faintly as arcs along their orbits under HUD); those that merged listed in the card as history.
- **Play** on the universe clock, at this level (on the Explore and Build desks; the Run desk has no clock above the system, section 5.9), runs the satellites along their stored orbits: a dwarf spiralling in, stripping, and joining the disc. It is the galaxy's battle-worn history seen from outside, and every frame of it is the kernel's own orbit.
- **The big neighbours' satellites** are not walked today (the sky looks only at ours). The same walk can run for the two or three biggest in the background. Until it has, a big neighbour is drawn without companions and its card says "satellites not worked out yet".
- **Faint dwarfs below the catalogue's floor** (10¹⁰ suns of halo) are counted, not placed. They stay a number in the caption ("and about sixty smaller ones"), never invented dots.

### 7.5 A galaxy's card

Pressing a galaxy opens the ordinary card: its name, its type in words, stellar and halo mass, distance from home and the light's travel time, whether it is bound to us and when the two first meet, how many mergers and when the last big one was, and "Chart this galaxy", "See our galaxy from there", "Make this home" (Build desk). Names come from the naming track's schemes; a catalogue address such as "halo 17" is never the name on show.

### 7.6 What it still will not do (registered)

- Galaxies move only on their bound-pair approach and, for home's satellites, their orbits. There are no flows from the web yet (F37 says so: Hubble flow only).
- Light-travel time is not applied: the view is a map at one now, where the sky op shows each galaxy as its light left it. The chip says "as it is now".
- The neighbourhood is the web's own, not chosen to hold a Milky Way and an Andromeda, until "Find a home" or the Milky Way pack sets it.

## 8. The local stars rung

Between the galaxy (tens of thousands of parsecs) and a system (a thousandth of a parsec) there is nothing today but a flat inset inside Your systems. The rung is:

- **The home star cell in 3D**: every star `star_cell` draws within the 10 pc cell at its real place, coloured by its temperature and sized by its light; the campaign's systems filled; routes between them (the links Your systems already draws); the target ringed. Chart is the flat map that exists. True is points of light against the sky.
- **The dive through the disc.** From the whole galaxy to the cell is a factor of three thousand with no stars drawn one by one in between. On the way down the galaxy's glow layers thin out and the cell's stars brighten in, and around them the **sky as seen from the target** comes up behind (the renderer's night sky, from `sky_from_here`), so the galaxy just left becomes the band across the sky. This is the longest seam and it is registered.
- Sector and hex maps are a pack's way of drawing the same rung. Where a pack with a hex map is stacked (Traveller), the Chart style of this rung is that map.

## 8A. Motion: velocity and pull as arrows, on every level

Alex, 25 Nov 12:22 UTC: "Another option for the HUD is velocity... vector arrows on velocity/accel - (and speed shown) to show relative movement of stars, galaxies and local system (ships), etc."

One HUD layer, **Motion**, the same on every level and every view that has a HUD. It sits in the HUD and Band chip beside the HUD switch: **Motion: Off, Velocity, Velocity and pull.**

- **An arrow is drawn by one mark** in the chart grammar, used by every view. Its length is the speed on a stated key shown beside the scale bar ("this long = 200 km/s"). Where speeds on one screen span more than a factor of about thirty the key is logarithmic and says so. The speed is written at the arrow's head, through the figures door, in the viewer's units.
- **Pull** is a second, thinner arrow in the field colour: the acceleration, with its own key ("this long = 1 mm/s²", or in g for a ship under thrust).
- **Relative to** is always said, and can be changed: by default the thing at the centre of the picture (so at Galaxies nearby the arrows are motions relative to home), or the level's own rest frame (the galaxy's centre, the system's barycentre), or the expansion (the web's comoving frame). Pressing an arrow opens a line: "Varrow: 110 km/s toward home, 40 km/s across. Relative to your galaxy."
- **Every arrow is a number the kernel already has**, read through `place_chain` and the level's own op:

| Level | Velocity | Pull |
|---|---|---|
| Cosmic web | each galaxy's drift from the web's displacement (6.3 point 6): matter streaming out of voids, along threads, into knots. An option adds the expansion itself | the gradient of the web's potential, which the kernel has |
| Galaxies nearby | drift relative to home; the bound pair closing; satellites along their stored orbits | toward each dark halo |
| Galaxy | the rotation curve from the halo structure block, as arrows round the disc; the target star's own motion against it | toward the centre |
| Local stars | each star's velocity from its orbit round the galaxy, relative to the target: who is passing, who is coming | too small to draw; said in the press line |
| System | orbital velocities of worlds, moons, small bodies and ships at the date | toward the star, or the planet for a moon; a ship's thrust as its own arrow, in g |
| World | the ground's speed from the spin at the site; moons, stations and ships in orbit | surface gravity |
| Site | the wind from the weather track, drawn with the same mark | none |

- **Play agrees with the arrows.** Where a level can run on the clock, things move along their arrows; the arrows are the first step of what Play shows.
- **On a journey** the Motion layer is part of the HUD, so it comes up at stops when the way in says HUD at stops.
- **Players** see Motion like any HUD layer, as their role knows the world. A later refinement (question 18): a table's sensors measure speed along the line of sight at once and the rest only by watching, so a player's arrow could start as a line-of-sight arrow and fill in. Not in the first package.
- **Night chart, phones, colour vision**: the arrow uses the ink and field tokens with shape, never colour alone; on a phone, arrows show for named things only until one is pressed.

## 9. Fudges to register

Each goes in `docs/fudges.md` in the pull request that builds it. None changes a number the kernel gives.

| # | Where | The fudge | Why it exists | What real thing would replace it |
|---|---|---|---|---|
| GZ-F1 | every seam | Two levels drawn by two renderers are cross-faded over about a third of a second. They agree at the target and in direction; things away from the target need not line up exactly | Each level's view is built for its own range of sizes; one scene across 22 powers of ten would need a renderer written for that alone | One nested scene with camera-relative coordinates per level |
| GZ-F2 | every seam | The exposure is re-metered at each level and eased across the seam | A galaxy's glow and a sunlit world differ by many powers of ten in brightness | Nothing: a camera would do the same. Registered so the caption can say it |
| GZ-F3 | galaxy to local stars | Between the whole disc and the home cell no stars are drawn one by one; glow gives way to the cell's stars and the target's sky | Billions of stars cannot be drawn; the kernel counts them by a count tree and draws only cells that are looked at | Stars drawn from the count tree along the line of descent |
| GZ-F4 | every rung | The journey travels at Real scale even if the viewer keeps Readable, and eases back at a stop | Exaggerated sizes cannot meet across a seam | Nothing; a rule, said on the scale chip |
| GZ-F5 | web, galaxies nearby, galaxy | Everything is shown at one now, with no light-travel delay | It is a map. The sky op shows the delayed view | A "seen from here, as the light arrives" option on these views |
| GZ-F6 | world to site | From 3 km over the globe to the ground patch is a seam between the globe and the Land screen's ground, whose detail below the kernel's fact floor is decoration (already said by that view) | The globe and the ground are two renderers | The detail ladder carried down to the ground |
| GZ-F7 | the camera | The camera crosses the picture far faster than light. Nothing travels: it is a change of map scale | So that the journey takes a minute | Nothing. The one-line HUD names it a zoom, never a flight |
| (F37, `reconstruct_origin`, `galaxy_chart` halo) | home in the web, a system's place | Existing stand-ins. They **retire** with GZ12 and GZ13 (section 6.3) | See section 6 | Home chosen from the web's catalogue; a star slot for every system |
| GZ-F8 | home | Home is the best-fitting catalogued galaxy within a few dozen cells of the origin, not a constrained copy of the real Local Group | A constrained realisation is far beyond a browser | The Milky Way pack's pins (G7) |
| GZ-F9 | authored systems | An authored or imported system takes the slot of the best-fitting drawn star near its place; its authored facts are kept and marked where they differ | The system was not derived from the galaxy | Nothing: it is authored. The mark says so |
| GZ-F10 | galaxy tilt | A spin axis drawn about the web's tidal axes with a mass-dependent preference | Integrating tidal torque through a halo's growth is too dear | A torque integral along the merger tree |
| GZ-F11 | galaxy drift | Peculiar velocities are linear (Zel'dovich); bound pairs and satellites use their own orbits | The web is a Zel'dovich web | 2LPT or an n-body step |

## 10. Speed

- **Drawing.** Only one level is drawn except in a seam, where two are, for about a third of a second. Each level already runs under the frame-time governor. The journey asks for the governor's "moving" quality while it zooms and lets the view sharpen at rest at a stop (both exist: VP4, VP5).
- **Reading.** The slow parts are kernel reads, and they are read ahead in the order the journey needs them, in the kernel's worker, while the upper rungs play. A one-minute journey gives the lower rungs most of a minute.

| Read | Cost today (desktop) | When |
|---|---|---|
| Web cube, 48³ at 100 Mpc, with its galaxies | about 4 to 8 s | before the journey starts; kept for the session (it is deterministic, so it may be kept on the device) |
| The neighbourhood's galaxies (`galaxies_seen`: a few dozen cell catalogues) and home's satellites | to measure in GZ0; the galaxy and web budget is 1 s an op | during the web rung |
| The home galaxy's chart | within the 1 s budget | during the web rung |
| Two or three neighbours' full charts | to measure in GZ0 | in the background; never waited for |
| The home star cell | small | during the galaxy rung |
| The system's scene | within 100 ms | during the galaxy rung |
| The world's faces and finer tiles (about 0.2 s a tile) | 1 to 3 s | during the system rung |
| The site's ground and caves | within the 2 s budget | during the world rung |

- **WebGPU desktops**: comfortable. A cold grand zoom should be ready to start in under ten seconds, most of it the web cube, and that wait is spent on the postcard's stamp or the first frame with one line of progress.
- **Phones on WebGL2**: every level already has a WebGL2 or canvas path. The risks are the web cube's read (the phone stand-in runs the kernel about four times slower, so 16 to 32 s), True's fill rate on the galaxy's layers, and memory with two levels alive in a seam. So on the basic tier the journey reads a 32³ cube, draws at the governor's lowest moving quality, and, if the speed probe says even that will not hold, travels in Chart style. GZ0 measures all three on the phone stand-in and on a real phone before anything is promised.
- **No heavy work for a press**: Go on never starts a read. If the next rung is not ready the button says "Getting ready…" with the same quiet line as 5.4.
- New rows on the drawing bench and in the performance chunk list: "Journey seam" and "Journey read-ahead".

## 11. What it is for

| Use | How |
|---|---|
| "Where am I in the bigger picture" | Zoom in from, with stops, on any bookmark or place |
| The scope of the software in one minute | The shipped preset "The grand zoom" on the start sheet's "Zoom all the way out" entry, played both ways, ending at a lava tube's mouth on a generated world |
| A GM's establishing shot | A preset: Show the bookmark, way in from Galaxy with no stops, fired at the Crew; or from as high as they know |
| A postcard that arrives | The sender ticks "Arrive by zooming in"; the link opens on the stamp, gets ready, and descends to the picture |
| Trailers | The clean output link, words none, read first, stepped by journey time for a perfect take on Alex's GPU. Storyboard C stops being nine clips and becomes one |
| Teaching | A tour whose steps are rungs with a teacher's own captions |

## 12. Standing rulings, checked

- **One general mechanism.** One shape (`approach`), one port on views, one player. The grand zoom, the everyday zoom between levels, a postcard's arrival, a preset's establishing shot and the trailer shot are all the same thing.
- **Don't copy SSE3.** SSE3 had no travel between levels.
- **Physics never faked.** The journey changes no number. Every stand-in is registered (section 9) and marked for the GM.
- **Processes, not classes.** A galaxy's type is the label the kernel reads off its state.
- **Same world at every level.** A stop is the real view.
- **Little fudging, computed once** (Alex, 25 Nov). Home, each system's slot, each galaxy's tilt and drift are each worked out once and stored or keyed; four new shortcuts, all registered (GZ-F8 to F11), and three old stand-ins retire.
- **Dynamic minimalism.** No new chrome: the place line is the progress bar, Go on is one button, the options are four switches behind one ▾. Rule 7's 120 to 240 ms is for interface motion; a journey is content, like Play on the ruler, and is always interruptible. Rules 8 and 9 hold.
- **Seats.** Players never steer the journey unless their role may; the GM streams only data; a role sees only what it knows.
- **One universe clock.** The journey never moves it.
- **Sound off by default.**
- **Design on Fable, code on Opus.** This note and its mockup are design; the packages below are for Opus builders.

## 13. Packages (proposed)

**Order, as Alex ruled on 25 Nov 12:45 UTC ("we need to sort local group before this is feasible. Also we need to tie everything together - probably the ideal time to do it"):**

1. **Stage A, the tie-up:** GZ0, then GZ12 (home in the web), GZ13 (a star slot for every system), GZ2 (`place_chain`), GZ17 (the gazetteer), then GZ15 and GZ14.
2. **Stage B, the Local Group:** GZ5 and GZ6, beside stage A from the moment GZ12 names the home halo; GZ7 (Local stars).
3. **Stage C, travel:** GZ1, GZ3, GZ4 (the short form, which is the Run desk's stepping), GZ16 (Motion), GZ8, GZ9, GZ10, GZ11.

The contract (GZ1) and the lower journey port (GZ3) touch nothing in stage A and may start beside it if a builder is free; the whole grand zoom (GZ8) waits for stages A and B.

| Package | What | Owner (proposed) | Needs |
|---|---|---|---|
| **GZ0** | Checks and timings, no feature: cold cost of every read in section 10 on desktop, the phone stand-in and a real phone; how far a halo's place in the web cube is from its place in the cell (they differ, 6.2 point 4: measure the size); that the orientation chain is the sky's; what one neighbour's full chart and satellite walk cost | a new thread "Build travel between places" | nothing |
| **GZ1** | The contract: `ui-contract/src/approach.ts` (`sse.approach/1`, defaults, its link form), the two new rungs on `PLACE_SCALES`, the pure plan (`journeyPlan`) and camera (`journeyAt`), with tests. No view code | same | GZ0 |
| **GZ2** | Kernel read `place_chain` with honesty marks; its own golden area | kernel thread | GZ0 |
| **GZ3** | The journey port on System 3D, the globe and the Land ground; the player in the shell (two stages, the seam, the place line as progress, Go on, Back up, Esc, reduced motion). Delivers "Zoom in from System" to a site, with and without stops | "Build travel between places", with the chart room and renderer threads for their files | GZ1, GZ2 |
| **GZ4** | The short form on the place line, the picker and "Chart this galaxy" (5.6) | same, with the selector thread | GZ3 |
| **GZ5** | Galaxies nearby part 1: centred on a galaxy, two reaches, 3D and True, points and type pictures from `galaxies_seen`, Real and Readable, HUD (names, haloes, bound pairs, scale bar), the galaxy's card | "Build galaxy charts and histories", with the renderer thread for the type pictures | GZ0 |
| **GZ6** | Galaxies nearby part 2: home's satellites on their orbits with Play, streams under HUD, real charts for the biggest neighbours read in the background, the web's threads behind, the edge of the group | same, with the kernel thread | GZ5 |
| **GZ7** | The local stars rung: the home cell in 3D and True, on the place line; the dive through the disc with the target's sky behind | "Build galaxy charts and histories" and the renderer thread | GZ1 |
| **GZ8** | The upper ports: the web cube centred on home, galaxies nearby, Galaxy 3D with the target's place marked, local stars. Delivers the whole grand zoom | "Build travel between places", "Build cosmic web view" | GZ3, GZ5, GZ7 |
| **GZ9** | Stops in full: captions from facts and authored words per rung, the width line, the target ring, free look and return | "Build travel between places" | GZ3 |
| **GZ10** | Where it is offered: bookmark rows, the postcard's back and link, the Show step's way in on the Playbook, the players' side (one kernel each, the knowledge ceiling, who goes on), the clean output link | Run desk thread and postcards thread | GZ3, GZ9 |
| **GZ11** | Small devices and film: the basic tier's lighter journey and the Chart-style journey, bench rows, capture by journey time, the shipped preset "The grand zoom" and the start sheet entry; "ends with the landing" | "Build travel between places", quality tiers thread, trailer thread | GZ8 |
| **GZ12** | Home in the web (6.3 point 1): the kernel chooses the home halo from the catalogue when a universe opens and stores it; every default-galaxy op reads it; "Make this home" on Find a home; F37 and the two stand-ins retired. Core golden changes on purpose | kernel thread ("Build kernel resolve and rules"), core lane; "Build galaxy charts and histories" for the button | GZ0 |
| **GZ13** | A star slot for every system (6.3 point 2): at adoption and import, by map position or best fit; the pin claims its count; "unusual for its birthplace" advice; move it on Your systems; "Grow a system here" wired to `from_star` | kernel thread or its hand-off thread ("Build galaxy arm fix and atmosphere advice"); "Build galaxy charts and histories" for Your systems | GZ12 |
| **GZ14** | Tilt from the web's tides (6.3 point 5): eigenvectors of the tidal tensor, the spin block with its citations checked first, the sky and Galaxies nearby reading one axis. Moves the sky's golden on purpose | kernel thread, citations thread | GZ12 |
| **GZ15** | Drift and one place per halo, from the web's displacement (6.3 points 6 and 7): `halo_cell`, `galaxies_seen` and the web cube report the same displaced position; a velocity per halo, redshift with peculiar motion in `galaxies_seen`, velocities in `place_chain`. Moves the sky's golden on purpose | kernel thread | GZ2 |
| **GZ16** | The Motion layer (section 8A): the arrow mark and its key in the chart grammar, the Motion switch in the shared view controls, System 3D and the system chart first (their numbers exist today), then each level as its velocities arrive | "Build travel between places" for the mark and switch; each view's owner for its level | GZ1; GZ15 for the top two levels |

| **GZ17** | The gazetteer (6.4): rows worked out at universe creation and when the date is set, kept with the campaign (home, the neighbourhood's galaxies with names, places and drift, home's satellites, each system's cell and its neighbours' stars); placeholders that fill in when looked at; redone in the worker on a deep-time change; the picker and Find read it; players get the rows they may know | kernel thread for the read, the Run desk thread for the campaign store and the players' side, the names thread for galaxy names | GZ12, GZ13, GZ2 |

GZ12 to GZ15 are kernel work that changes goldens, so they queue in the coordinator's lane order, first in line by Alex's ruling.

### 13A. Where each package lands, and what it reuses

Alex, 25 Nov 12:47 UTC: "the grand zoom and Build galaxy charts and histories - may be doing similar things - or designing at least." The split, so nothing is designed or built twice: **this note is design only and its thread writes no feature code. "Build galaxy charts and histories" is the builder that owns the Galaxies nearby and Your systems screens**, and what it has built is extended, not redrawn.

| Folder and owner | Packages landing there | Already built there, to reuse |
|---|---|---|
| `views/cluster-map`, `views/galaxy-chart` (Your systems, HomeCell), `views/galaxy-3d` use, `ui-contract/galaxy.ts`, `home-finder.ts`, `your-systems.ts`: **"Build galaxy charts and histories"** | GZ5, GZ6 (Galaxies nearby in 3D and True), GZ7 (Local stars), the screen halves of GZ12 ("Make this home") and GZ13 ("Find it a place", moving a system), the reading side of GZ17 on these screens | `ClusterMap` (kept as the Chart style and its catalogue), `FindHome` and the cited Milky Way wish (becomes the kernel's home wish in GZ12), `cellHalo`, `projectHalo`, `haloRadius`; Your systems' map, height stalks, routes, pencil and ink, map layers; `HomeCell` on `star_cell` (becomes the Local stars rung's Chart style); `systemSummary`; **"Grow a system here" on `from_star`** (built on its branch, held on two kernel bugs), which is GZ13's generated case, finished rather than redesigned. Your systems keeps its list of systems with no place, beside the map, for anything still unplaced; it never invents a place |
| `crates/sse-kernel` (`cosmos.rs`, `galaxies.rs`), `crates/sse-cosmos`: **the kernel thread** ("Build kernel resolve and rules") or its hand-off thread | GZ0's kernel checks, GZ12, GZ13, GZ2, GZ14, GZ15, the kernel read of GZ17 | `halo_cell`, `galaxies_seen`, `satellites_seen`, `star_cell`, `from_star`, `reconstruct_origin`, the web's tidal tensor and displacement |
| `views/foam-map/Web3D.svelte`, `renderer/web-cube.ts`: **"Build cosmic web view"** | the web's part of GZ8 (centred on home, its journey port), the web's arrows in GZ16 | the cube in 3D and True, light from haloes |
| `renderer/` galaxy and sprite files: **the renderer thread** | the type pictures of GZ5, the dive of GZ7, ports for the globe and the ground in GZ3 | `galaxy-true.ts`, `galaxy-marks.ts`, `close-up.ts`, the night sky |
| `views/voyage`, travel panels: **"Build galaxy charts and travel screens"** | nothing new; its star map (`Stars.svelte`) and the Local stars rung read the same `star_cell`, and its flight arrows adopt the Motion mark of GZ16 when it exists | the voyages star map |
| `shell/src/run`, postcards: **Run desk and postcards threads** | GZ10, the campaign store of GZ17 | bookmarks, presets, the Switchboard, postcards |
| new: `ui-contract/approach.ts`, the journey player in the shell, the arrow mark in `chart-grammar`: **"Build travel between places"** (to open) | GZ1, GZ3, GZ4, GZ8, GZ9, GZ11, GZ16 | `flyTo` in System 3D, `delivery.ts`, `place.ts` |

Section 7 is therefore a brief for the galaxy charts thread, not a second design of its screen: where its own notes or its accepted galaxy chart mockup already settle a point, they stand, and that thread says so by message.

A cave's inside is not here. Interior sites are parked to about 4.2, so in 4.0 the grand zoom ends at the tube's mouth, looking in.

## 14. Questions for Alex

1. **What it is called in the product.** The control reads "Zoom in from ▾" beside "Go there"; the full-length shipped preset and trailer shot are "The grand zoom"; "way in" and "journey" stay words for the notes. *Recommend yes.*
2. **One mechanism for all travel between places**, so every step on the place line zooms in a short form (about 0.6 s) and the grand zoom is the long form. *Recommend yes.* It is what makes the work pay every day.
3. **Two new steps on the place line: "Local stars" between Galaxy and System, and "Site" below World.** *Recommend yes.*
4. **Galaxies nearby is rebuilt centred on a galaxy, as a sphere with two reaches (the neighbourhood, 12 Mpc, and the group, about 1.5 Mpc), in Chart, 3D and True.** Today's cube-by-cube catalogue stays inside Chart. *Recommend yes.*
5. **Broad types for every galaxy, real histories for home and the two or three biggest neighbours read in the background, and our own satellites on their real orbits with Play.** *Recommend yes.*
6. **Home in the web.** Superseded by question 13 after Alex's follow-up: home becomes one of the web's own galaxies by default, not only when the GM uses Find a home.
7. **The default way in when "Zoom in from" is chosen:** True, a stop at each level, HUD and captions at stops, go on by a press. "One smooth zoom" is one switch that turns stops, HUD and words off together. *Recommend yes.*
8. **Pace:** Brisk (about 25 s for the whole grand zoom), Stately (about 45 s, the default) and Slow (about 90 s, for film). *Recommend yes.*
9. **At the table:** by default the GM's Go on moves every screen of the role together, and a role travels only from as high as it knows. *Recommend yes.*
10. **A sent postcard may arrive by zooming in**, as the sender's choice, off by default; three of the shipped base postcards have it on as showpieces. *Recommend yes.*
11. **The one-line width readout while moving** ("30 million light years across · 10²³ m"), part of the HUD. *Recommend yes.*
12. **For 4.0.** The whole of it is a showcase and reuses what 4.0 already has. *Recommend* GZ0 to GZ11 and GZ16 in 4.0, in the order of section 13, with GZ12 to GZ15 as the kernel's lane allows, and the cave's inside left to 4.2 with interior sites.

Added after Alex's follow-up of 12:22 UTC (sections 6 and 8A):

13. **Home is one of the web's own galaxies, chosen once by the kernel when a universe is opened** (best fit to the pack's home wish, stored, the same on every device), and the GM may choose another. This changes the default galaxy, so existing campaigns' galaxy charts and skies change once. *Recommend yes.* It closes the one broken link.
14. **Every authored or imported system is given a real star slot in a cell at adoption, automatically**, by its map position or the best-fitting drawn star near home, keeping its authored facts and saying where they are unusual for that birthplace. The GM can move it. *Recommend yes, automatic,* so no system is ever without a place.
15. **A galaxy's tilt from the web's tides** (small haloes along their thread, massive ones across it) as a cited editable block, replacing the isotropic draw. *Recommend yes,* after the citations are checked.
16. **Every galaxy's drift from the web's own displacement**, so near neighbours can approach and the sky's redshifts include it. *Recommend yes.*
17. **The Motion layer as section 8A has it:** Off, Velocity, Velocity and pull; one arrow mark with a stated key and the speed written; "relative to" always said and changeable; the system level first. *Recommend yes.*
18. **Players' arrows from what their sensors can measure** (line-of-sight speed at once, the rest by watching). *Recommend later,* with the sensors work, not in the first package.

## 14A. Alex's rulings, 25 Nov 2026 12:45 UTC

"All your suggestions are good - see no real issues apart from discussion above - so yes to all... once you have manipulated them based on above." So questions 1 to 5 and 7 to 18 are yes, with these changes, which win where they differ:

1. **The Local Group first.** "Looks like we need to sort local group before this is feasible." Section 7 (GZ5, GZ6) moves ahead of the journey.
2. **Tie everything together now.** "Probably the ideal time to do it." Questions 13 and 14 are not "as the kernel's lane allows": GZ12 and GZ13 lead. The rule of section 6.5 (no system without a star slot, and so on up) is a standing rule from here.
3. **Each level drives the one below.** Already how the events and chemistry work (6.5); the stand-ins that hid it go.
4. **A gazetteer kept with the campaign** (6.4, GZ17): places and names worked out once at the defined time, placeholders for what has not been looked at, only in-system motion worked out on the fly.
5. **Three desks.** Explore is messing about and what-ifs, Build ("Edit") is setting games up and worldbuilding, Run ("Play") is clean, visual controls only, and the most visually arresting; on Run, stepping up and down the hierarchy by this zoom is the way to move (5.9). A separate design task sets out every display on each desk.
6. Question 6 stays replaced by 13. Question 12 (release): unchanged, all of it for 4.0, now in the order of section 13. Question 18 (players' arrows by sensor): later, as recommended.

## 15. Picking this up without the chat

- State of the track: `handover/status/grand-zoom.md`. Nothing is built. Accepted; start with stage A of section 13 (GZ0, then GZ12).
- The claims in section 3 were read from main at 69d7dca (25 Nov 2026). Check a file name before relying on it.
- Not verified, GZ0's first jobs: how big the jump of 6.2 point 4 is (the two places differ by the code; nobody has measured by how much), and what the reads in section 10 marked "to measure" really cost.
- The mockup shows the mechanism in two dimensions with painted pictures. It is there for pace, the seam and the stop's layout, not for how any level will look.
