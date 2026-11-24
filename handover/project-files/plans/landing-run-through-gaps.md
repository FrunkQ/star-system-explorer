# Next week's target: the GM plots the players' experience, ending in a landing

Alex, 23 Nov 22:16: "landing sequence and end to end run through for next week's level. The GM plotting the Players experience of the events."

**What this means.** The GM sets up a short run of events and steps the table through them:

1. The party's ship flies a voyage to a world.
2. It arrives, and the world is revealed.
3. The GM picks a site, and the party lands there.
4. The party walks about on the ground.

Players connected from their own seats see each step happen on their screens. Nothing dead-ends along the way.

**How this list was made.** I walked the path on main (fccd80ac) as GM in one window and as a joined player in another, on 23 Nov at 22:20 UTC, using the Solar System and the Run desk. Screenshots are next to this file.

## What works today

**GM side**
- Open the Solar System and take the Run desk.
- Share with players. A player joins, and the hash agrees.
- Plan a voyage on Voyages.
- On Earth, open Land.
- Pick a landmark or any spot on the map. Dangers and a verdict are shown.
- Land: the descent is flown with the kernel's log beside it.
- Skip to the ground, walk about, then take off.

## Gaps on the path, by owner

The owner of each gap is in brackets.

### Blockers: without these there is no run-through

1. **The players' page never follows the GM** [chart room, mine]. It always loads the shipped Kestrel facts (`kestrel.player-facts.json`), whatever system the GM is in.
   - With the GM in the Solar System, the player still sees Kestrel and Tarn (`11-pl-down.png`).
   - Needed: the players' scene follows the campaign's system, and the place the GM shows.
2. **The GM has no way to "show the table this"** [chart room, mine; Looks 4C and 4D seat rows].
   - The Seats dialog is the old one: add a seat, fog of war, a list of screens.
   - Needed: the seat rows on the Run desk, with "Send this view" and "Screens off" for every seat. This is a smaller first cut of the accepted Looks seat rows. The look and the signal dial can come after.
3. **Players never see a voyage or a landing** [renderer, for the Land view on the players' page; chart room, for the screen document; travel, for streaming the descent's state].
   - Land runs only on the GM's machine. There is no player screen for "in flight", "descending" or "on the ground".
   - Needed:
     - a players' Land screen that plays the same descent from the kernel's log, at the GM's pace;
     - the ground view for walking about. Does the GM walk for everyone, or does each player walk? I recommend the GM leads.
4. **No party ship ties the steps together** [travel].
   - Voyages plans from the first world in the system (it opened on "Mercury to Venus"), not from where a ship of the campaign is.
   - Land offers a lander picker, not the party's ship.
   - Needed: pick a campaign ship (Ships and stations), fly it on a voyage, and on arrival land that same ship.
   - Its crew comes from the Aboard panel, now in PR 318.
5. **Nothing for plotting "the events"** [design needed: chart room plus travel; one question for Alex below].
   - Today the GM can do each step by hand, but there is no run of beats to set up beforehand and step through.

### Needed for it to feel right

6. **The Known to eye is not live yet** [kernel SV1 `known_to`, then mine]. Arrival reveals ("now they know Mars has air") depend on it. The Scanned stand-in still does the job meanwhile.
7. **Earth reads as airless on Land: "Airless lander", "No air"** [renderer, PR 328 open, fixes it].
8. **The ground in Chart style is noise** [renderer]. Chart draws the ground as a 3D diagram that reads as static (`11-gm-down.png`). True or 3D should be the default on the ground, or Chart should get a contour diagram.
9. **Voyages labels overlap**: "Earth", "Main asteroid belt", "Ceres", "Uranus", "Kuiper belt", "Pluto" [travel].
   - Picking Earth on the chart doesn't make Earth the start.
10. **Run desk card loses Mass** at Curious depth [chart room, mine; checking whether that's the lens's intent].

### Done tonight

- The view bar is one row on desktop. HUD and Band now sit together in a "Show" chip (PR 335).
- Run-desk chart names the table doesn't know are drawn in pencil instead of words (PR 335).
- The ship card's buttons are agreed with the manifest thread: one "Aboard" button, with room left for Power (PR 318).

## Order I'd take

1. **Blocker 1, players follow the GM** (mine). This comes first because everything else shows through it.
2. **Blocker 2, Send this view and Screens off** (mine). This uses blocker 1.
3. **Blocker 4, the party ship through voyage and land** (travel). This can run alongside 1 and 2.
4. **Blocker 3, the players' Land screen** (renderer, with me for the screen document). This comes after 1 and 4.
5. **Blocker 5, the run of events**. This comes once its question is answered.

## Question for Alex

**Q1. How does the GM plot the events?** I recommend (a).

- (a) **A run sheet on the Run desk.** It is an ordered list of beats, each one a thing SSE4 already does:
  - fly the ship to X;
  - reveal Y;
  - send this view;
  - land at site Z;
  - a note to read aloud.

  The GM presses Next to step through it. Players see what each beat sends.
  - Beats are plain dated data, so they fit 4.0's "lighter touch": set by hand, recorded, and replayable.
  - Minerva could draft a run sheet later.
- (b) **No sheet.** The GM does each step live, and SSE4 only makes sure every step shows on the players' screens.
