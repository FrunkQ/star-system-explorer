# SSE4: the grand zoom, and travelling between places (track GZ)

Status: **PROPOSED 25 Nov 2026, not yet accepted.** Twelve questions for Alex in section 14, each with a recommendation. Design and mockup only; nothing here is built by this note. Packages for builders are in section 13. GZ is a code only, never shown in the product.

Mockup, clickable: project files `sse4/handover-screens/GZ-screens/grand-zoom-mockup.html` (also published privately to Alex as the artifact https://claude.ai/artifact/662RbMkMjyz2CZSEm5CKAb), with 17 desktop and phone screenshots beside it: the seven stops, two seams, a pressed galaxy, and the proposed Galaxies nearby view at both reaches and both scales. The pictures in the mockup are painted stand-ins, not SSE4 output; the mockup says so.

Alex's brief, in his words (25 Nov 12:05 UTC): "what is the capability of doing the grand zoom to show off the true scope of the software? ... when you build a postcard or a bookmark and you use them, you'll have the opportunity to zoom to location. And then you can choose where you start ... At each stage, pause, show the HUD, and on a click, it zooms down to the next level ... it could be one long, smooth zoom from top to bottom with no text at all, or it could be it goes each level, pauses, shows you what's around, allows you to click on them to see what they are, and then zoom down to the next level ... effectively from galactic filament to lava tube on a planet ... we can do a little bit of fudging at the transition, but because everything should align, in theory, this should work. Or am I whistling Dixie?" And: "Our level between galactic filament and our galaxy, the local galaxy group, is pretty poor at the moment and will need a lot of work."

Read with: `SSE4-top-level-ui-and-onboarding.md` (the place line in 2.1, tours in 3.2), `SSE4-playbook-and-switchboard-design.md` (bookmarks in 4A, presets in 4.2, who steers in 5.3), `SSE4-postcards-design.md` (the camera on a bookmark), `SSE4-galaxy-design.md` and `SSE4-galaxy-histories-design.md` (haloes, galaxies, satellites), `SSE4-visual-polish-design.md`, `interface-style.md` (rules 7 to 12), `SSE4-secrets-and-visibility-design.md`.

## 1. The short answer

1. **It is feasible.** The two ends exist and most of the middle does. A bookmark already holds the place, the date, the instrument, the view and (since the postcards work) the exact camera. The place line already runs cosmic web, galaxies nearby, galaxy, system, world, and a place link already carries a landing site. One kernel and one clock answer every level.
2. **What is new is small in idea: a change of place can be travelled instead of cut.** Going to a bookmark today is a cut. This note adds one property to the *use* of a bookmark, the **way in**: where the journey starts, whether it stops at each level, and what is shown on the way. The grand zoom is the longest way in. The same mechanism, in a short form, makes every step on the place line a zoom rather than a swap.
3. **It cannot be one 3D scene, and does not need to be.** The range is about 10²² to 1 in size. Each level keeps its own view and its own renderer. At each change of level the outgoing view zooms onto the target, the incoming view opens already aimed the same way with the target on the same pixel, and the two cross-fade while both keep zooming. That hand-off is the one new fudge, and it is registered (section 9).
4. **A stop is the real screen, held.** At a stop the level is the ordinary view of that level, so everything in it can be pressed and opens its ordinary card. Nothing is a film.
5. **From the system down, everything truly lines up. Above it, it lines up by stand-ins that are already in the fudge register.** Our galaxy is not yet a galaxy of the web, and a system has no real place in its galaxy unless it was generated there. Section 6 says what to do about each.
6. **The Local Group level needs rebuilding, and the facts for it are already worked out.** The kernel already gives every neighbour galaxy a broad type, a size, a tilt, a colour and its star formation, and gives our own satellites their orbits. Only the night sky reads them today. Section 7 turns "Galaxies nearby" into a 3D view drawn from those facts, with real histories for the biggest two or three.
7. **About twelve packages**, in an order where the first three already give a zoom from system to site that is true all the way (section 13).

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
3. **Our galaxy is not in the web.** The campaign's galaxy is the median halo of its mass, set at the centre of cell 0, and any web halo within 0.3 Mpc is taken to be us (fudge F37). The chart of "the galaxy" is that median halo (fudge register, `galaxy_chart` halo).
4. **A system has no place in its galaxy unless it was generated there.** An imported or hand-made system is put at the Sun's radius, azimuth 0 (`reconstruct_origin` place). "Your systems" already refuses to invent a place: it lists such systems beside the map.
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

### 5.9 Reduced motion, small screens, no GPU

- `prefers-reduced-motion`: no zoom. Each rung arrives with the plain fade and holds. "One smooth zoom" becomes a sequence of held rungs of about two seconds each.
- A device with no GPU, or one the speed probe places too low for True, travels in **Chart** style: the flat charts (web slice, galaxies nearby, galaxy chart, system chart, the world's map) zoom and cross-fade the same way. The mechanism is the same; only the port's answers differ.
- Phones: the place line is already two rows; Go on is a thumb-sized button at the bottom edge at a stop; the caption is a sheet that slides up from the target's side.

## 6. Making the top half line up

From the system down the chain is true today: a world's place in its system comes from the kernel's orbits at the date, its pole from its tilt, the site from latitude and longitude. Above the system:

1. **One read for the whole chain.** A new kernel read, `place_chain {address, t}`, returns for each rung the target's position, natural size and orientation in its parent's frame, and an honesty mark where a stand-in was used. Every seat gets the same answer, and the stand-ins stay in one place. It is a read only and goes in the kernel's own golden area.
2. **Home in the web.** Keep F37 as it is for now, so the sky, the galaxies nearby and the journey all agree: home at the centre of cell 0, the web's axes taken as the galaxy's. Then let "Find a home" finish its job: when the GM picks a web galaxy whose history fits, that halo **becomes the campaign's home** at every level, the galaxy chart charts that halo's own history, and F37 retires for that campaign. That is the "home halo drawn in the web" that the register already names as F37's replacement (question 6).
3. **A system's place in its galaxy.** A generated system has a real place (its star cell and slot). An imported one may have a map position (Your systems reads a V3 starmap's). One with neither is travelled to at the stand-in place, and the GM's caption carries the honesty mark: "placed at the Sun's distance from the centre as a stand-in". The players' caption simply does not name a place in the galaxy. Giving it a real place is one press: "Find it a place" on Your systems picks a drawn star of fitting mass and age nearby (this wants the kernel's `from_star`, already queued).
4. **Orientation.** The journey needs each frame's turn relative to its parent: the galaxy's axis in the web (F37's convention today), the system's plane in the galaxy (the convention `sky_from_here` already uses for galactic longitude and latitude), the world's pole in the system. `place_chain` returns them; GZ0 checks they are the same ones the sky uses, so the Milky Way's band in a world's sky is where the journey came from.

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
- **Play** on the universe clock, at this level, runs the satellites along their stored orbits: a dwarf spiralling in, stripping, and joining the disc. It is the galaxy's battle-worn history seen from outside, and every frame of it is the kernel's own orbit.
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
| (F37, `reconstruct_origin`, `galaxy_chart` halo) | home in the web, a system's place | Existing stand-ins, read through `place_chain` with their marks | See section 6 | "Find a home" making the halo the campaign's; a real place for each system |

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
- **Dynamic minimalism.** No new chrome: the place line is the progress bar, Go on is one button, the options are four switches behind one ▾. Rule 7's 120 to 240 ms is for interface motion; a journey is content, like Play on the ruler, and is always interruptible. Rules 8 and 9 hold.
- **Seats.** Players never steer the journey unless their role may; the GM streams only data; a role sees only what it knows.
- **One universe clock.** The journey never moves it.
- **Sound off by default.**
- **Design on Fable, code on Opus.** This note and its mockup are design; the packages below are for Opus builders.

## 13. Packages (proposed)

Order matters: GZ1 to GZ3 give a true zoom from system to site with nothing fudged above it. The Local Group rebuild (GZ5, GZ6) is worth having with or without the journey and can run beside them.

| Package | What | Owner (proposed) | Needs |
|---|---|---|---|
| **GZ0** | Checks and timings, no feature: cold cost of every read in section 10 on desktop, the phone stand-in and a real phone; whether a halo is at the same place in the web cube and the cell; that the orientation chain is the sky's; what one neighbour's full chart and satellite walk cost | a new thread "Build travel between places" | nothing |
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
| **GZ12** | Home in the web: "Make this home" from Find a home, the campaign's own halo at every level, F37 retired for that campaign | kernel thread and "Build galaxy charts and histories" | GZ2, GZ5 |

A cave's inside is not here. Interior sites are parked to about 4.2, so in 4.0 the grand zoom ends at the tube's mouth, looking in.

## 14. Questions for Alex

1. **What it is called in the product.** The control reads "Zoom in from ▾" beside "Go there"; the full-length shipped preset and trailer shot are "The grand zoom"; "way in" and "journey" stay words for the notes. *Recommend yes.*
2. **One mechanism for all travel between places**, so every step on the place line zooms in a short form (about 0.6 s) and the grand zoom is the long form. *Recommend yes.* It is what makes the work pay every day.
3. **Two new steps on the place line: "Local stars" between Galaxy and System, and "Site" below World.** *Recommend yes.*
4. **Galaxies nearby is rebuilt centred on a galaxy, as a sphere with two reaches (the neighbourhood, 12 Mpc, and the group, about 1.5 Mpc), in Chart, 3D and True.** Today's cube-by-cube catalogue stays inside Chart. *Recommend yes.*
5. **Broad types for every galaxy, real histories for home and the two or three biggest neighbours read in the background, and our own satellites on their real orbits with Play.** *Recommend yes.*
6. **Home in the web.** Keep the registered stand-in for now, and make "Find a home" able to set the campaign's own galaxy at every level (GZ12). *Recommend yes.*
7. **The default way in when "Zoom in from" is chosen:** True, a stop at each level, HUD and captions at stops, go on by a press. "One smooth zoom" is one switch that turns stops, HUD and words off together. *Recommend yes.*
8. **Pace:** Brisk (about 25 s for the whole grand zoom), Stately (about 45 s, the default) and Slow (about 90 s, for film). *Recommend yes.*
9. **At the table:** by default the GM's Go on moves every screen of the role together, and a role travels only from as high as it knows. *Recommend yes.*
10. **A sent postcard may arrive by zooming in**, as the sender's choice, off by default; three of the shipped base postcards have it on as showpieces. *Recommend yes.*
11. **The one-line width readout while moving** ("30 million light years across · 10²³ m"), part of the HUD. *Recommend yes.*
12. **For 4.0.** The whole of it is a showcase and reuses what 4.0 already has. *Recommend* GZ0 to GZ11 in 4.0, in the order of section 13, with GZ12 when the kernel's lane allows, and the cave's inside left to 4.2 with interior sites.

## 15. Picking this up without the chat

- State of the track: `handover/status/grand-zoom.md`. Nothing is built. The first package is GZ0 and needs no answer from Alex.
- The claims in section 3 were read from main at 69d7dca (25 Nov 2026). Check a file name before relying on it.
- Two things were not verified and are GZ0's first jobs: whether the web cube and the halo cell put a galaxy in the same place, and what the reads in section 10 marked "to measure" really cost.
- The mockup shows the mechanism in two dimensions with painted pictures. It is there for pace, the seam and the stop's layout, not for how any level will look.
