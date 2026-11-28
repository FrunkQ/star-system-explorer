# SSE4: stand-in pictures while things move, the proper picture at rest (track FL)

Status: **PROPOSAL, 28 Nov 2026. Not accepted. Nothing here is built.** Written at the coordinator's word for Alex's ask of 28 Nov 13:14 UTC, after he chose stills for the stirred clouds: "can we 'fake movement' and when they stop, resettle ... A simple shader, rather than real maths. Even mobiles have decent shaders for this type of thing. We need to build this in as a standard feature to reuse. The ability to 'fake it if time moves too fast' and fall back to 'proper render' when you stop ... maybe you can still animate but limit to 1 fps for cloud updates ... Is this a 'visual flourish' system we need to have designed to reuse when this type of thing crops up (eg: transitions between the grand zoom)". Design only. FL is a code only, never shown in the product.

Read with: `SSE4-visual-polish-design.md` section 3 (the three grades every drawn thing has) and 8 (tiers, the governor, sharpen at rest); `SSE4-stirred-clouds-design.md` sections 4 and 11A and `packages/WS5.md` (the measured cost); `packages/CLK1.md` (the picture clock); `SSE4-grand-zoom-design.md` sections 5.3, 5.4 and 10 and `packages/GZ8a-first-seam.md`; `SSE4-time-scales-design.md` section 9 (pace); `interface-style.md` rules 7 to 10.

**On sources.** The paper search is out until 1 Dec 2026. The two sources in section 12 are from memory and **unverified**. The hand checks in section 5 use figures already in the repo.

---

## 1. The short answer

1. **Yes, it is one system, and we already have six half-copies of it.** Orbits that run on between the kernel's answers, a giant's rows sliding at their jets' speed, star cells that blur at speed, sharpen at rest, the grand zoom's cross-fade and its "Reading the galaxy…" hold: each is "show something cheap while things move, and the proper picture when they stop", each written by itself (section 2). This note makes them one mechanism with one rule.

2. **The rule.** A picture that is dear to work out is a **proper picture**: a pure function of the date, the place and the view, drawn from the kernel's numbers. While the clock, the camera or the place is moving faster than the device can rebuild it, a **stand-in** is shown. When the movement stops, the proper picture is built without being asked and the view **settles** onto it. What you see at rest is always the proper picture.

3. **It does not break "never fake it", if the line is drawn here** (section 3): a stand-in never feeds the kernel, is never saved, never moves a tracked thing, is never what a press is answered from, and is never left on screen at rest. It is said in the HUD's note while it shows, and each use is in the fudge register. That line is for Alex to confirm (question 1).

4. **Most stand-ins are not fakes at all.** There are six kinds (section 4.2), tried in order of honesty. The first four are true things shown more plainly: a coarser picture, the last picture held, the last picture carried on by the real winds, and a long exposure. Only the last kind is decoration, and nothing needs it yet.

5. **The "simple shader" is one small kit** (section 4.4): a carry that slides and swirls the last proper picture along a small table of the kernel's own winds (three texture reads a pixel, fine on a phone), a long-exposure blur, and one fade. The same three pieces serve clouds, seas, a star's face, belts, a galaxy's turning and the zoom's seams.

6. **Alex's "1 fps" is the rebuild cap** (section 4.3): while the clock runs, a world's proper picture is rebuilt at most once a second, in the frame's spare time, and the carry fills in between. When the device cannot keep even that, it carries on from the last one and settles when the clock stops.

7. **Hand check: Alex's hunch is right.** Jupiter's cloud only changes visibly once the clock passes about 8 hours a second, and by then the planet is turning most of a turn every second. Past about 6 days a second it turns faster than the pictures. So a stirred giant is nearly always seen at rest or as a blur, and the stand-in covers a narrow window (section 5).

8. **The grand zoom is the same thing with the place moving in place of the clock.** A level that is not read yet travels on its coarse picture and settles at the stop, in place of holding (question 6). Film, photographs and postcards never use a stand-in.

9. **Seven small packages** (section 8). None touches the kernel or any golden. The first is a list of what exists; the mechanism is two files.

---

## 2. What the code does today (main at `30574a2a`)

Each row is this pattern, written once for its own case.

| Where | While moving | At rest | Said to the user? | Register |
|---|---|---|---|---|
| Worlds on their orbits (`System3D`, CLK1) | phases run on at each body's period between the kernel's ten answers a second | the kernel's answer | no | "Orbits run on between the kernel's answers" |
| A giant's face (`true-air.ts` `flowFace`) | rows slide at their jet's speed; storms turn in two blended phases | the face as read | no | yes |
| A giant's stirred still (PR 739, draft) | the still slid by `flowFace`; rebuilt when the clock stops or the cloud has aged a tenth of its renewal | the stirred still | planned | planned |
| A star's cells (`granuleBlur`) | blur toward their average as a picture stands for more of their lives | sharp cells | no | yes |
| A fast moon (`tooFast`) | drawn where it truly is; may seem to jump | the same | yes, the note names it | "Motion faster than the pictures" |
| Sharpen at rest (`still.ts` `RestClock`, VP5) | the recipe's moving passes | 16 to 32 finer frames averaged after half a second | the badge | no (not a fudge) |
| The governor (`governor.ts`) | walks a recipe's degrade order down under load | walks back up slowly | the badge, in plain words | no |
| The zoom's seam (`Journey.svelte`, GZ8a) | two levels cross-faded over about 0.9 s | one level | no | GZ-F1 |
| The zoom's wait (`readyHold`) | holds on the upper level with "Reading galaxies nearby…" | goes on when read | yes | no |
| A world's finer tiles; the unstirred face before a still | the coarser one shows, the finer fades in | the finer one | no | no |

What is missing is not the idea but the one place for it. Six different rules decide "when is it moving", three different fades settle, two tell the user and four do not, and a new case (the stirred still) is about to write a seventh.

---

## 3. The line: what a stand-in may never do

The polish design (section 3) already sorts every drawn detail into **fact**, **drawn from the numbers**, or **camera effect**. A stand-in is not a fourth grade. It is a statement about **when**: this picture is between two proper ones. These seven rules hold for every use, and FL1's tests hold them.

1. **Presentation only.** A stand-in never reaches the kernel. Nothing reads it back: no save, pack, postcard, report, rule or hazard. It lives in the device's memory and is thrown away.
2. **Tracked things are always true.** Bodies, named storms, shadows, eclipses, the terminator, ships: anything on a card or in a list is placed from the kernel every picture, moving or not. A stand-in covers only the texture between them.
3. **A press is answered by the kernel at the date in view**, never from the stand-in's pixels. "Why does it look like this here?" is the kernel's read and is right whichever picture is up.
4. **At rest, the proper picture, unasked.** When the clock, the camera and the place have been still for half a second (the rest clock's figure), the proper picture is built and the view settles onto it. No press. A device too slow to build it shows the coarser proper picture it can build, and says so.
5. **Never used to save effort.** If the device can rebuild the proper picture as fast as it changes, it does, and there is no stand-in. The scheduler decides from measured cost, never from a tier's name.
6. **On the campaign's clock.** A stand-in is a function of the last proper picture and the clock seconds since. No wall-clock rate. Stopped means it stands still. The one thing that then changes on screen is the settle (question 2).
7. **In plain sight.** While a stand-in shows, the HUD's note says so in one clause. Each use is a row in the fudge register and an entry in one declared list, and a test fails if either is missing (FL4). Its sizes, speeds and directions come from kernel numbers or it is not drawn (polish rule 3).

**How this reads against the standing rules.**

| Rule | How it is kept |
|---|---|
| "Lean into the physics, never fake it" | Read as: never fake a fact, and never leave a stand-in at rest. The physics is still worked out in full; the stand-in only covers the time the device needs to draw it |
| Plain sight | Rule 7. The app can still say why a thing looks as it does from the numbers that drew it, because at rest those numbers did draw it |
| The picture clock | Rule 6 |
| Always a picture, no click | Rule 4. There is a picture at every instant: a stand-in while moving, the proper one at rest |
| Low power renders once as a still, in memory only | The still is the proper picture; the cache rules of stirred clouds 11A are taken over unchanged (section 4.5) |
| One general mechanism over specials | Section 4. The six half-copies of section 2 fold into it (FL6) |
| A pass is never the only way to see a fact (polish 8.3) | Rule 2 |

---

## 4. The mechanism

### 4.1 What a view declares

A layer whose picture is dear declares one small record. Nothing else is special to it.

| Field | Meaning | Giant's cloud | A level in the zoom |
|---|---|---|---|
| `id` | its name in the declared list and the register | `giant.cloud` | `journey.nearby` |
| `proper(date, view, size)` | builds the proper picture, in pieces over several frames | the stirred still (`stirred-gpu.ts`) | the level's own reads and drawing |
| `moves_with` | which movements make it stale: clock, camera, place | clock | place |
| `own_time_s` | how long before the picture has changed enough to see | a tenth of the deck's renewal time (kernel) | none |
| `cycle_s` | the shortest turn of what the picture rides on, if any | the world's day (kernel) | none |
| `stand_in` | the kinds it offers, from 4.2, in order | carried, then exposure | coarse, then held |
| `needs` | kernel numbers the stand-in reads | the wind table | the level's coarse catalogue |
| `says` | the note's clause, through the figures door | "carried by its winds between pictures" | "drawn coarsely on the way" |

`own_time_s` comes from the kernel, per world. Where the time pipeline's pace ladder (TP4, op `pace_ladder`) already names the process, it is read from there, so the same number that paces play decides rebuilds.

### 4.2 The six kinds of stand-in, most honest first

A use takes the first kind that looks right. A later kind needs a reason in its register row.

| Kind | What is shown | Grade (polish 3) | Example |
|---|---|---|---|
| **Coarse** | the same derivation at less detail: a smaller face, fewer steps, a smaller cube | fact, coarser, and said | a 128-wide stirred still; a 32³ web cube; a world's 256 face |
| **Held** | the last proper picture, right for its own date or pose | fact, of an earlier date, and said | the last still while a new one builds |
| **Carried** | the held picture moved on by the simple part of the real motion, from kernel numbers | drawn from the numbers | rows slid by the jets; cloud swirled by a table of the winds; orbits run on at their periods |
| **Exposure** | the average over the stretch of time one picture stands for, as a camera with its shutter open would see | drawn from the numbers | bands blurred along their winds; star cells toward their mean; a fast moon as the arc it sweeps |
| **Seam** | two pictures that agree at the target, cross-faded | camera effect | the zoom's level change (GZ-F1); every settle |
| **Hint** | decoration whose size, speed and direction come from kernel numbers, but which follows no real path | drawn from the numbers, moving only | none proposed. Kept as a named kind so that if one is ever wanted it arrives through this door, with its own register row and Alex's yes |

The kind Alex described ("fake movement, a simple shader") is **Carried**. It is less of a fake than it sounds: sliding a row at its jet's speed is exactly what the jet does, and swirling by the winds as they stood at the still's date is right to first order for a short time. What it leaves out is the folding that builds up, and that is what the settle puts back.

### 4.3 When to rebuild: one schedule

One pure schedule (no clock of its own, as `RestClock` is) takes: what moved, the clock's rate, the layer's `own_time_s`, the measured build time at the size on screen, and the governor's spare time. It gives one of four states.

| State | When | What is drawn |
|---|---|---|
| **Rest** | nothing in `moves_with` has changed for half a second | the proper picture for this exact date and view; the settle fade onto it |
| **Live** | a build takes less clock time than `own_time_s` (rate × build time < own time), within the cap | the proper picture, rebuilt as it changes, with the stand-in filling the frames between. The viewer cannot tell it from stepping every frame |
| **Carried on** | builds cannot keep up, but one picture still covers less than `own_time_s` | the stand-in from the last proper picture; a rebuild whenever one can be afforded |
| **Too fast** | one picture covers more than `own_time_s`, or more than half of `cycle_s` ("faster than the pictures", CLK's own test of under two pictures a turn) | the Exposure kind where the use has one, otherwise Held. No rebuilds until the rate drops |

Details the builder needs:

- **The cap.** At most one rebuild a second for each layer while anything moves (Alex's figure), and only in the frame's spare time: a build is cut into pieces, a piece is run only when the governor is under budget, and the governor's degrade order drops rebuilds first of all. The stirred pass already works this way (one step a draw, state kept between draws).
- **Built at the size it is shown.** A build is asked for at the layer's size on screen, from a ladder of sizes (64, 128, 256, 512, 1024 wide). A giant twenty pixels across costs a few hundredths of a full face, so it is always Live. This is Alex's "it only needs to be stirred when close", for free. The biggest thing on screen is built first.
- **Rebuild dates sit on a lattice.** While the clock runs, proper pictures are built for dates on a fixed lattice anchored at the campaign's epoch, spaced `own_time_s` apart (a coarser power-of-two multiple on a slow device). Scrubbing back and forth then finds pictures already in memory, and two devices on the same spacing show the same thing. At rest the picture is built for the exact date.
- **A drag on the ruler** is a movement of unknown rate. Within one lattice step of a kept picture, carry it (the carry runs backwards as well as forwards). Further, show the nearest kept picture if memory has one, else the Too fast picture. On release, Rest.
- **The settle.** The stand-in and the new proper picture are both brought to the present date and cross-faded, opacity only, over the interface's 240 ms. So nothing jumps. Under reduced motion it is one plain fade, and the Carried and Hint kinds are off (Held shows).
- **No GPU** (`canvas`, `text`): Held and the settle only. The proper picture is still built at rest, on the processor, as stirred clouds 11A already says.

### 4.4 The kit: three small shader pieces

Written once as renderer nodes for WebGPU and WebGL2, each with a processor twin it is held to by test.

| Piece | Does | Cost a pixel |
|---|---|---|
| **Carry** | Reads the held picture at a place shifted back along a small **wind table** by the clock seconds since it was made. The table has two parts: a steady part by row (the jets, exact for any length of time) and a turning part (storms and eddies as they stood), which is run as two overlapping short loops and blended so nothing stretches. The loop's length is set from the numbers: the shortest of the band's shear time and a radian of the fastest eddy's turn | three texture reads |
| **Exposure** | Blurs along the same table by speed × the time one picture covers. At the limit it is the mean along the row, which is one column worked out once per picture | up to eight reads; one at the limit |
| **Fade** | The settle and the seam: two pictures, one weight, aligned first | two reads |

The wind table is the one new thing a use has to supply. For a giant it is a small bake of the carry table the kernel already gives (`want: "carry"`): one sum of the winds per cell at 256 × 128, not the thousands of steps a pixel the proper picture takes. `flowFace` is this piece already, written for giants alone; it becomes the kit's first user.

The same Carry serves a world's cloud faces and sea, a star's face, a belt's shear and a galaxy's turning, each with its own table from its own kernel read. None of those is built here; they are the reason it is a kit.

### 4.5 Memory, capture and the players

- **Memory only.** Proper pictures are kept on the device while memory allows, oldest dropped first, under the limit stirred clouds 11A set. Never in a file, save, pack or postcard.
- **Photographs, postcards and film never show a stand-in.** "Take photograph" and a postcard's picture build the proper picture first. Film capture (the trailer rig asks for a frame at a journey time or a date) builds the proper picture for every frame, however long that takes.
- **Seats.** At rest every seat shows the same picture for the same date. While moving, two seats may differ in the stretch between proper pictures, because one device rebuilds more often than the other. The lattice keeps that small (question 7).
- **Run desk and players.** Nothing extra to press. The note's clause is in the plain words the players' side already uses.

### 4.6 What the user reads

One clause is added to the motion note the picture clock already writes (`motionWords`), only while a stand-in shows:

> "Jupiter's clouds are carried along by its winds between pictures. The full picture settles when the clock stops."

> "The bands are blurred as a long exposure: the clouds change faster than the screen can show."

> "The galaxies nearby are drawn coarsely on the way down. The full picture settles when the zoom stops."

When the settle is done the clause goes. The performance readout gains one line for each layer: its state, its last build time and its size.

---

## 5. Hand checks

Jupiter, from figures in the repo: renewal 32 days (WS3b), so `own_time_s` is 3.2 days; a day of 9.93 hours; a 512-wide still in about 0.1 s on AC-ProArt and 1 to 2 s on a phone (WS5's estimate, unverified on a real GPU); 30 pictures a second.

| Clock rate | Cloud ages 3.2 days in | Jupiter turns | State (desktop / phone) | What you see |
|---|---|---|---|---|
| stopped | never | still | Rest | the stirred still |
| an hour a second (the ruler's slowest) | 77 s | once in 10 s | Live / Live | bands sliding; the still renewed about once a minute |
| 8 hours a second | 10 s | 0.8 times a second | Live / Live | the first rate at which the cloud visibly changes |
| a day a second | 3.2 s | 2.4 times a second | Live / Live | a rebuild every 3 s, carried between |
| 3 days a second | 1.1 s | 7 times a second | Live at the cap / Carried on | the phone falls behind here |
| 6 days a second and up | under 0.5 s | faster than the pictures (under two pictures a turn) | Too fast | bands blurred to their mean; a long exposure of a spinning giant is exactly this |

- The window in which the stand-in is doing real work is from about 8 hours to 6 days a second. All through it the planet is spinning too fast to study. That is Alex's "you will rarely see it animated close up anyway", in numbers.
- The one view where the cloud could be watched changing is one that turns with the planet. That view gets Carried with a rebuild a second, which is his "limit to 1 fps for cloud updates".
- Saturn: renewal 336 days, a still about ten times dearer. Its phone falls behind near 2 days a second, the same order as Jupiter.
- The carry's loop on Jupiter: band shear time 16 hours, fastest eddy a radian in 0.9 days, so the loop is 16 hours. Jets of 150 m/s move cloud 8,600 km in that, about a band's width, 10 pixels of a 512 face.
- Cost of the Carry on a phone: three reads a pixel on a face of 512 × 256 is under half a million reads a frame, well inside what the speed probe's fill scene measures.

---

## 6. The grand zoom, as the second user

In a journey the camera and the place move and the clock does not (grand zoom 5.7). The same four states apply with `moves_with: place`.

| Today | With this |
|---|---|
| If the level below is not read at its seam, the journey holds with "Reading galaxies nearby…" (8.7 s on a desktop, 37 s on the phone stand-in, GZ0) | The level declares a **Coarse** picture from what is cheap to read (the catalogue's galaxies as type pictures, a 32³ cube, a world's 256 face). The journey travels on it and the level settles at the next stop or at the end. It still holds if not even the coarse picture is ready |
| The seam's cross-fade is its own code (`SEAM_FADE_DECADES`) | It is the kit's Fade, with the same alignment rule. GZ-F1 stays its register row |
| The view sharpens at a stop by VP5 | The same Rest state does both: sharpen and settle are one event |
| A zoom played for film reads everything first | Kept: capture never uses a stand-in (4.5) |

Alex's rule of 27 Nov ("nothing worse than spotting the adventure because it is the only bit written in detail") binds the Coarse picture: it must show **the same things in the same places** as the proper one, more plainly. A coarse picture with fewer galaxies, or an empty one, is not allowed; the journey holds as it does today.

---

## 7. What changes for whom

- **Explore, Edit, Play:** nothing new to press and no new screen. Pictures stop stalling or popping when time runs or a zoom travels, and settle when things stop. One clause in the note says when a stand-in is up.
- **A builder with a dear picture:** declares the record of 4.1, supplies a wind table or a coarse picture, and gets the schedule, the fade, the note, the cache and the tests. No new rule for "is it moving".
- **The stirred clouds thread:** PR 739 lands as it is, with `flowFace` between rebuilds. FL3 then moves it onto the schedule and the kit. Nothing in WS5 has to wait.
- **The fudge register:** each use is a row written from one template (what is shown while moving; which kernel numbers set it; what the proper picture is; how long the settle takes on the reference devices).
- **Phones:** the Carry is cheap enough for tier B, so a phone's giant moves between stills where today's plan has it slide rows only.

---

## 8. Packages

Builder threads are the coordinator's to name (section 11). Each leaves main green. **None touches the kernel, a pack or any golden.**

| Package | What | Needs | Size |
|---|---|---|---|
| **FL0** the list | Docs and one data file: every row of section 2 written as a declared use (`id`, kind, what moves it, its numbers, its register row, its words). Register rows added for the four that have none. No behaviour changes | nothing | S |
| **FL1** the schedule | `ui-contract/src/settle.ts`, pure: the record of 4.1, the four states, the cap, the size ladder, the lattice of dates, the drag rule. Tests only; nothing drawn | nothing | S |
| **FL2** the kit | Renderer nodes Carry, Exposure and Fade with processor twins; the wind table's layout; a bench row for each. Checked on AC-ProArt on WebGPU and WebGL2 | FL1 | M |
| **FL3** giants | The stirred still on the schedule: built at the size shown, on lattice dates, capped; `flowFace` replaced by Carry over a table baked from `want: "carry"`; Exposure at speed; the settle. With the stirred clouds thread, after PR 739 | FL2, PR 739 | M |
| **FL4** said and checked | The note's clause from each use's words; the performance readout's line; a test that every declared use has a register row and every register row marked as a stand-in has a declared use | FL1 | S |
| **FL5** the zoom | The journey port's "ready" answers proper, coarse or not yet; the journey travels on coarse and settles at a stop; the seam on the kit's Fade; capture waits for proper. With the crash zoom thread | FL2 | M |
| **FL6** folding in | One at a time, each its own small PR: sharpen at rest takes its "moved" from the schedule (so the clock counts as movement in one place); orbits between answers; star cells; a world's finer tiles | FL1 | S each |

**Later, not queued:** the time exposure for fast bodies (CLK3 already wants it; it is the Exposure kind on a body's arc); a world's cloud faces and sea on the Carry; belts and rings by their shear; a galaxy's turning at play; a view that turns with a giant so its cloud can be watched.

Red-first tests (each seen red with its fix removed; at least one absolute):

- FL1: at rest the state is Rest whatever came before (absolute); the same date reached by play, by a drag and by a jump asks for the same proper picture; with a build time of zero the state is never anything but Live or Rest (rule 5); no more than one rebuild a second is asked for; a stopped clock asks for exactly one build and then none.
- FL2: each node agrees with its processor twin within one pixel on a 64-wide patch; the Carry at zero seconds returns the picture untouched, to the bit; a pure jet carried for ten days matches the rows slid exactly; the Exposure at the limit equals the row mean.
- FL3: a stopped clock shows pixels equal to PR 739's still at that date (absolute: the stand-in leaves nothing behind); a named storm's drawn centre is the kernel's at every picture while playing (rule 2); the still built for a giant 64 pixels across costs under a twentieth of the 512 one.
- FL4: removing a register row turns the check red; the clause shows only while a stand-in shows.
- FL5: with the lower level's reads held back the journey travels on the coarse picture and settles at the stop; a coarse picture with fewer galaxies than the proper one makes the journey hold (the filled-in rule); a capture never contains a stand-in frame.

---

## 9. What could be wrong

1. **The build times are estimates.** 0.1 s on a desktop and 1 to 2 s on a phone are WS5's sums, not measurements on a real GPU. If a phone's still takes 10 s, phones sit in Carried on for longer and the settle is slower. The mechanism is the same.
2. **The two-loop carry can show a faint pulse** where the two loops disagree most, as every flow map does. A third loop is the cheap cure.
3. **The settle may be seen as a jump** when the stand-in has drifted far from the truth (a long run on a slow phone, then stop). The fade hides position, not a different pattern. Shorter lattice steps cost more builds.
4. **A long exposure of a banded giant looks plain.** That is correct, but it may read as "the detail went away". The note's clause is the answer; a Hint is the temptation to resist.
5. **Folding in the old copies (FL6) touches views that work.** Each is its own small PR for that reason, and none is needed for FL3 or FL5.
6. **Two seats differ while moving.** Small, and gone at rest, but a GM streaming a table could notice.
7. **A new knob to misuse.** "It settles at rest" could become the excuse to ship a poor moving picture. Rule 5 and the Live state are the guard: the stand-in is only ever the device's shortfall, measured.

---

## 10. Questions for Alex

1. **Is this the line?** A stand-in is presentation only; tracked things are always true; what you see at rest is always the proper picture; it is said in the note and listed in the register. "Never fake it" then reads as "never fake a fact, and never leave a stand-in at rest". *Recommend yes.*

2. **May the picture change once while the clock is stopped, to settle?** The picture clock says stopped means still. The settle is one fade of about a quarter of a second after the proper picture is built, as sharpen at rest already does. *Recommend yes, as the one allowed exception, never a loop.*

3. **Rebuild cap: at most one proper rebuild a second for each world while time runs, in the frame's spare time?** The alternative is no rebuilds until the clock stops, which is simpler and looks staler. *Recommend the cap of one a second.*

4. **Honest kinds first?** Coarse, held, carried by the real winds, and long exposure are tried in that order, and pure decoration (a Hint) is kept as a named kind that nothing uses until a case needs it and you say yes to that case. Your "simple shader" is the carried kind. *Recommend yes.*

5. **At speed, show a long exposure instead of unrelated pictures one after another?** The accepted clouds design shows each picture right for its own instant, so at speed successive pictures share nothing. A long exposure blurs the bands to their mean, which is what a camera would record. Tracked bodies keep the picture clock's rule (drawn where they truly are). *Recommend yes for textures.*

6. **In a zoom, travel on a coarse picture instead of holding on "Reading…"?** Only when the coarse picture shows the same things in the same places; otherwise it holds as now. *Recommend yes.*

7. **Accept that two seats may differ slightly while moving, and agree at rest?** *Recommend yes.*

---

## 11. Routine choices, for the coordinator

- Track code FL; status in `handover/status/flourish.md`. The design thread owns the note; builders add their own rows.
- Builders, proposed: FL1, FL2, FL4 and FL6's sharpen-at-rest with "Build quality tiers and speed probe" (it owns `governor.ts`, `true-run.ts` and `still.ts`); FL3 with the stirred clouds thread; FL5 with "Build the crash zoom between places"; FL0 with whichever starts first. Code on Opus.
- PR 739 is not held by this note. If it is accepted, the only ask of that thread is the one already made: keep the motion between rebuilds simple and in one place.
- FL2's and FL3's screens on AC-ProArt under the standing permission, to `sse4/handover-screens/FL2-screens/` and `FL3-screens/`.
- No CI change beyond each package's own test line.
- The two sources go through the citations check after 1 Dec.

---

## 12. Sources, unverified until 1 Dec 2026

- Max and Becker 1995, "Flow visualization using moving textures" (ICASE/LaRC Symposium on Visualizing Time-Varying Data): a texture carried along a flow in overlapping loops and blended.
- Vlachos 2010, "Water flow in Portal 2" (SIGGRAPH course notes): the two-loop flow map as used in games.

---

## 13. Picking this up without the chat

- Nothing is built. Start with FL0 only after Alex accepts; his answers go in a section 10A here and win where they differ.
- The whole idea is section 1 points 2 and 3, the seven rules of section 3 and the table in 4.3.
- The first real user is the stirred still (FL3); read `packages/WS5.md` for its traps on the GPU (no loops round the wind, series for sine and cosine, one step a draw).
