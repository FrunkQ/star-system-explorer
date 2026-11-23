# How SSE4 comes together

Date 22 Nov 2026. Status: **accepted by Alex, 23 Nov 2026 ("yes to all" on the seven questions of section 7).** It sits beside `SSE4-build-plan.md`, which stays the list of every package. This file is the order in which the pieces are joined up, and how that stays tidy. Read from main, the open pull requests and each build thread's checklist on 22 Nov at about 16:30 UTC.

## 1. The idea

- The engine is far ahead of the app. Most tracks are built in the kernel and reach the screen only in part. The work now is mostly joining up, not inventing.
- Stages order the **joining up**: what reaches a person's hands, and what Alex is asked to look at. They do not stop a thread building underneath for a later stage.
- A stage ends with one walk through the app that a person can do, and a short list of known rough edges. It does not have to be fully working. It has to be honest about what is rough.
- Changes that touch many blocks at once go one at a time, in one lane, with one owner (section 4).
- Alex gets one numbered list a day at most, each question with a recommendation (section 6).

### The test every joined-up screen must pass (Alex, 23 Nov)

In his words: "we need to stitch everything together so it feels intuitive and approachable. Most common things a click away. Rarely touched things in menus that have relevant context."

- **Common things are one click away** from where a person already is: on the thing itself, its card, the place line or the instrument rail. If a tester does it in most sessions, it is not in a menu.
- **Rare things live in a menu that belongs to what they act on**: this world's menu, this chart's menu, this pack's menu. Never one global list of everything, and never a setting far from the thing it changes.
- **Each stage's walk is also the click count.** When a walk is written into the check, each step records how many clicks it took from the step before. A common step that needs more than one is a rough edge and goes on the list.
- When a screen is joined up, its owner sorts its actions into "every session", "sometimes" and "rarely", and places them by that. New capability arrives where its subject already is (a field on the body, a formula on its block), not as a new top-level place.

## 2. Where things stand on main

| Area | In the app today | Built underneath, not yet on screen | Biggest gap |
|---|---|---|---|
| Start and one system | Start sheet, Open the Solar System, grow a system, import a V3 save, edit on the chart, why on every number, undo, copy and paste, save in the browser | | Formation story screen: only the replay exists |
| A world | Body globe with time lapse, cutaway, climate Why, Mag lines, sizes, 3D system | Surface chain to caves and lava tubes, weather, descent, icy worlds, core heat history (PR 142 open) | Life screen shows example worlds, not the chart's own; cutaway has no ice zones yet |
| Time | One ruler from Gyr to hours, campaign now, look-ahead, what-if branches | Ice ages, field history | Campaign now is not yet in the intent log |
| Out to the galaxy | Galaxy chart, galaxies nearby, cosmic web, voyages, route console | Night sky (stars, glow, other galaxies), nebulae, assembly tree (PR 178 open), rotation curve | **Getting back down to a system.** Place line being built now |
| Reference | Constants and Units; Elements in PR 187 | Phases of water, constants with a history, Cosmos group | Substances, phase diagram, calendars, editing |
| Physics you can read | Flow editor changes parameters; Look up block; 499 typed copies of constants left of 615 | Boxes, logic blocks | Editor cannot add or wire nodes; no Equation block yet |
| Light | Band switch on stubbed spectra | Spectra, body light, shifts, sensing, planet finding | Screens still read the stub; no gas has a colour |
| Running a game | Players page, seats, themes, requests and approval, lenses, sound | Campaign record, programs part 1, tags TG1 to TG5 and TG12, Traveller codes, sectors, sensors, ships | Traveller has no screens in the chart room; programs part 2 |
| Agents | Orac change requests, Minerva, Tars, own key | | Orac cannot take a paper or add blocks |
| Sharing | Converter live; Explorers linked by copy and paste | | Explorers cannot publish or fetch V4 packs |
| Performance | Readout in PR 193 | | Bench and minimum spec table |

## 3. The natural order: six stages

Each stage names what a person can do at its end, what it needs, and who does it.

### Stage 1: Test drive (a day or two)

**You can:** open the Solar System or grow your own, edit a world and ask why, open it as a globe, a cutaway and its magnetic field, run time forward, go out to the galaxy and the cosmic web, come straight back down to your world from anywhere, and look up a constant or an element.

Needs:
1. The place line: breadcrumb up, "nested below" drop-down down, at every scale (chart room thread, then one pull request from the galaxy charts thread).
2. The five open pull requests merged: MF1 (142), GH1 (178), Elements (187), typed copies part 2b (188), performance readout (193).
3. **The walk as a check.** The route above is added to the existing shell check, on desktop and phone, so nothing merged later can break it without failing. No workflow change is needed.
4. A "known rough edges" list in `handover/test-drive.md`, written by the chart room thread and kept current, so a tester knows what is unfinished on purpose.

Rule for this stage only: a fix to the walk goes ahead of any new screen.

### Stage 2: One system, told in full

**You can:** open any system on the chart and read how it formed as one chart of distance against time; see where each world's material came from; watch a world's field switch on and die on the time ruler; see the ice zones in a cutaway; read the chart's own worlds on the Life screen; open a substance's phase diagram.

Needs, in order: the formation story screen (chart, cards, open for a system already on the chart: two packages, chart room). Reference finished (Substances, phases, calendars). Mag in the band switch. Field ledger and history (MF2, MF3) and phase profile (MP19, MP19b). Habitats on adopted worlds so Life reads them. Performance bench and minimum spec. In parallel in the kernel: branching a formation run from a stored moment, so "Change the story" and Orac's request can follow in stage 3 or 4 without waiting.

### Stage 3: One language underneath

**You can:** open any flow and follow it from the block names alone, see every formula on its block with a line to each letter's source, add and wire a block yourself, and see a star's or a planet's colour come from its gas.

This is the cross-cutting stage. Needs: maths blocks and the Equation block (ND3). Atoms moved into named Equations family by family (ND3b). The typed-copies ledger to zero (MP22). Light through gas with the spectrum on a wire (ND2, with O4). Atmosphere facts on any body (ND1). The flow editor adds and wires nodes. Screens switch from stubbed light to the kernel's. The order inside it is section 4.

### Stage 4: The galaxy has a past

**You can:** open a galaxy and see its mergers, satellites, streams and globular clusters, each with a date and a cause; read its rotation curve and dark matter ledger; stand on a world and see that history in its sky.

Needs: DM2 to DM5, GH2 to GH8 (kernel, then the galaxy chart to the accepted mockup), the rest of the night sky (Y4 part 3, Y5 to Y9), the voyage join at a system's edge. Much of this is already running underneath; it joins up here.

### Stage 5: Run a game

**You can:** import a subsector, open a world as a full system, hand players their screens, set a ship's autopilot and look ahead, mine a rock out, load a setting pack, and fetch someone else's pack from Explorers.

Needs: Traveller screens in the chart room. Programs part 2 (J6). Tags TG6, TG8, TG9, TG11, TG13 to TG15. The chart room on the kernel's intent registry (J5). Explorers publishing and fetching V4 packs (X2), which touches its storage and so needs Alex's own yes.

### Stage 6: Show it

**You can:** hand Orac a paper and get a pack; switch a view to True; follow a tour; watch the trailers.

Needs: ND4 to ND10, view styles and True views, tours, help text for every block (X4), the films. Then 4.1.

## 4. Changes that touch many blocks: one lane each

| Change | Owner | Rule so it does not collide |
|---|---|---|
| Typed copies to zero (MP22) | Data sheet thread | Sweeps one crate at a time and says which. Other threads still remove copies only in files they already touch. |
| Atoms into Equations (ND3b) | Kernel thread | A family moves only after MP22 has swept its crate, so each file is rewritten once. Results and hashes unchanged, 5% timing rule. One family per pull request. |
| Light through gas (ND2) | Light thread, rows with the data sheet thread | Starts after ND3 lands, so its formulas are written once, as Equations. Row shape agreed before any rows are filled. |
| Golden hashes changed on purpose (DM4, GH3, GH4, later S and W stages) | Whoever's package it is | One such pull request open at a time. Its owner posts "golden window open" to the other build threads, merges fast, posts "closed". |
| Place contract | Chart room thread | The shell owns the bar, history and links. A view only says where it is and what is below it. |
| Free view on every diagram (rule 11) | Each view's owner | As each diagram is next touched. No sweep. |
| Package status | Each thread, its own file | Done 23 Nov: `handover/status/<track>.md`, one file per track, edited only by its owner thread. `handover/START-HERE.md` is the index and standing rules and takes no package rows. |

## 5. Parked on purpose

| Parked | Until | Why |
|---|---|---|
| View styles, True views, in-universe looks | Stage 6 | Alex: design only until it makes sense. They sit on top of light through gas and the renderer. |
| The Orac paper film (ND10) and its rehearsal | Stage 6 | It must be real, so everything it shows has to exist first. |
| Tours | Stage 6 | A tour of a screen that is still moving is rework. |
| "Change the story" and Orac's request on the formation chart | After the kernel branching package | The reading half is useful by itself and needs no kernel change. |
| Ground detail (S9 to S13), later weather (W10 to W12), sound beyond N1, later field stages (MF11 to MF13, MF15) | As their threads reach them | Depth, not joining up. None blocks a stage. |
| Paid packs in the app, the real-sky (Gaia) pack, sealed turns (J8) | After stage 5 | Each needs the sharing path working first. |
| Economy | 4.1 | Already ruled. |

## 6. Keeping it tidy

- **Who owns what.** Chart room thread: the shell, every screen's frame, the walk check, the rough-edges list. Kernel thread: resolve, rules, the Equation work, galaxy histories. Proven core: surfaces, interiors, fields at the source. Light thread: light, sky, field effects. Data sheet thread: the sheet, citations, the typed-copies sweep. Galaxy charts thread: galaxy and travel screens. Others as named in the build plan. A thread that needs something in another's area asks that thread; it does not reach in.
- **No new build threads** for a stage that has not come. Finished threads (agents, renderer, weather) restart when their stage does.
- **Handover.** Every package still ends with its log, its status line and checks green. Each stage adds one thing: the walk for that stage, written into the check, so the next Claude can see what "working" means by running it.
- **Asking Alex.** Threads send questions to the coordinator, not to Alex. The coordinator posts one numbered list a day at most, each with a recommendation, answered as "yes to 1, 2 and 4". Only something that cannot be undone, or that stops a stage, comes sooner.
- **Stage end.** One short message: the walk to try, the rough edges, what the next stage is. Nothing else needs reading.

## 7. Questions for Alex (all seven answered yes, 23 Nov)

1. **The six stages in this order?** Recommended: yes. Stages 4 and 5 could swap if you want a table game before the galaxy's past; the kernel work for both runs underneath either way.
2. **Stage 1 is the walk in section 3, and a fix to it goes ahead of new screens until it passes?** Recommended: yes. It costs about a day and gives a floor that later merges cannot break.
3. **Formation story screen: the reading half (chart, cards, open for an existing system) first in stage 2, kernel branching alongside, "Change the story" after?** Recommended: yes.
4. **The class atlas joins Reference as a "Names and classes" group, read pages there, one editor?** Recommended: yes. Classes are labels read off results, so they belong with the other things a setting names, and one editor avoids a second place to learn.
5. **Split START-HERE into one status file per track with a one-page index?** Recommended: yes. It removes the most common merge clash and makes "where does this track stand" one small file.
6. **Questions reach you as one numbered list a day through the coordinator?** Recommended: yes.
7. **Sweep order in section 4: typed copies, then Equations per crate, light through gas after the Equation block?** Recommended: yes.
