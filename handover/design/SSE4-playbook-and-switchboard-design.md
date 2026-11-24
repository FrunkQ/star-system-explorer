# SSE4: the Playbook and the Switchboard (track PB)

Status: **PROPOSED 24 Nov 2026**, for Alex. The direction was accepted by Alex on 24 Nov 15:39 ("agree... on all counts", six questions, in memory `sse4-run-desk-beats`); this note designs it in detail. Design and mockup only; nothing here is built by this note. The 14 questions are in section 14, each with a recommendation.

Mockup, clickable: `/mnt/project-files/sse4/handover-screens/PB-screens/switchboard-mockup.html`, with desktop and phone screenshots beside it; live at https://claude.ai/artifact/FJYtkbakTj4kbi55PdczU2 . The mockup shows the Switchboard on a desktop and a phone, light and dark: sending, dragging, holding, the signal dials on the boxes, the line card, Hold all, folding sections, adding a preset, Keep and Again. The role card, Add a screen and the page picker are described here and not mocked.

Alex's brief, in his words (24 Nov): "a gm must always be able to adapt and remix... presets... are great to have 'in hand' but live is the key"; "no need for next - just let the GM build up a 'playbook'... chapters and pages and sections - each can be a preset that is fired at various windows"; "a routing diagram showing what everyone sees - gm can pause any one by clicking on the line"; "a mini video routing package"; "the routing screen should be generally available and does remove the need for a bunch of other smaller interfaces - we corral the user to the 'nerve centre'".

## 1. The short answer

1. **Everything the GM can show is a source. Everything that can show it is a destination. A line joins them.** That one sentence is the whole Switchboard. The GM's live view is a source. Each preset on the open Playbook page is a source. A camera is a source. A destination is a screen, a role (all its screens) or Everyone (all roles).
2. **Sending is making a line.** Press a preset and its line appears to the role it names. Drag from any source to any destination to send it somewhere else. "Send this view" is a line from Live. Nothing is sent any other way.
3. **Holding is breaking a line.** Click a line and it breaks: that screen gets a holding card ("Awaiting transmission", "Signal lost", or a joke). Click again and the picture comes back. "Screens off" is a hold with a blank card, so it is the same thing and not a second control.
4. **A preset is a saved send.** It holds exactly what a live press would send, plus the name of a role. So firing a preset and doing it live go down one path, and "keep what I just did" is free.
5. **The Playbook is pages of presets**, in chapters, with section headings. The open page is the GM's hand. There is no order, no Next and no script.
6. **Roles are named in the Playbook; screens are decided at the table.** A preset says "Bridge". Which glass is Bridge tonight is set by dragging on the Switchboard, and is never saved in the Playbook.
7. **It replaces the Seats dialog, the seat rows, the "Table sees" bar, the share link line and the planned Looks seat rows.** What stays on the chart is one Send button and one small "on air" chip that opens the Switchboard.

## 2. Words

Seven words, used the same way everywhere.

| Word | Means | Today |
|---|---|---|
| **Screen** | One piece of glass showing a players' page: a tab, a phone, a wall TV, an OBS source | a tab that opened a seat link |
| **Role** | A named place to send things: "Bridge", "Telescope". It carries whose knowledge it shows, its sensors, its screen layout and its look | a `Seat` (`ui-contract/src/seat.ts`). The builder confirms a seat is already a role in effect: any number of screens may open one seat link |
| **Everyone** | Every role at once | the table-wide `shown` |
| **Source** | Something that can be shown: Live, a preset, a camera | a `Shown` value |
| **Line** | A source reaching a destination, with its state: live, following, held | `shown` per table and per seat |
| **Preset** | A saved send with a role name | not built |
| **Playbook** | The campaign's presets, as chapters, pages and sections | not built |

"Seat" stays in the code where it is. On screen the GM reads "role" and "screen", because those are the two things the GM actually thinks about (question 2).

## 3. The Switchboard screen

One screen, three parts, read left to right the way a signal travels.

```
 PLAYBOOK (sources)                              SCREENS (destinations)
 ┌─────────────────────────┐                     Bridge · green terminal
 │ Live: Tarn system        ●━━━━━━━━━━━━━━━━━━━▶ ┌────────┐ ┌────────┐
 │ [the GM's own view]      │        ╲            │ Wall TV│ │ Mira   │
 ├─────────────────────────┤         ╲           └────────┘ └────────┘
 │ Ch 2 · Arrival at Tarn ▾ │          ╲          Telescope
 │ APPROACH                 │           ━━━━━━━━▶ ┌────────┐
 │  Tarn IV close up  ▸Tel  ●━━━ ╳ ━━━━━━━━━━━━▶  │ Jon    │  (held)
 │  Survey report     ▸Tel  │                     └────────┘
 │ GOING DOWN               │                    Stream
 │  Landing, Hollin   ▸Brg  │                     ┌────────┐
 └─────────────────────────┘                     │ Output │
  Hold all                    just now: ...       └────────┘
```

### 3.1 Left: the sources

- **Live** is always first. It is a small monitor of the GM's own working view, with its place name. It is the live half of "live is the key": whatever the GM is looking at can be sent with one press or one drag.
- **The open Playbook page** below it: a chapter and page picker in one line, then the page's presets under their section headings.
- **A preset row** has two parts. The body selects it, which shows it in the monitor so the GM can check it before anyone else sees it. The **fire button** at its right edge, which reads "▸ Bridge", sends it. One press, no confirmation. This split is so that looking through the hand never fires anything by accident.
- **Camera** appears as a source only when the GM has added one (section 10).

### 3.2 Right: the destinations

- Screens are grouped under their role's name. Each screen is a small live preview with its name ("Wall TV", "Mira's tablet") and one line saying what it shows.
- **The preview is drawn on the GM's machine** from what the screen was sent, through its role's knowledge and look. It is what the screen should be showing, and the caption says so, as the Looks note already ruled. Each screen also reports back one short data line (which place and layout it is showing, and whether reduced motion is on), so a preview has a small tick when the screen confirms it and a "not answering" mark when it has gone quiet.
- **Role headings are destinations too.** So is the "Everyone" bar at the top of the column.
- **Add a screen** sits at the foot of the column: the join link, a QR code, "Open a screen on this computer", and "Clean output link" (section 9). A new screen arrives in the role its link named, or in the first role if it used the plain link.
- **A role's heading opens the role card** (section 7.3). It is the old "Add a seat" form, opened from the thing it describes.

### 3.3 Between: the lines

A line is drawn from a source to whatever it was sent to. Its look says its state without words:

| Line | Means |
|---|---|
| solid ink | live: the destination is showing this source |
| solid accent | following: the destination follows the GM's live view as it moves (section 5.3) |
| broken, with a small lozenge in the gap | held: the destination shows a holding card |
| dashed more and more coarsely | the signal dial is turned up on the box this line reaches |
| thin and quiet | a role's line that one of its screens has left for a line of its own |

There is one line per role by default. A screen gets its own line only when the GM sends something to that screen alone ("one player's phone can be its own destination"). "Back with Bridge" on that screen removes its own line.

### 3.4 What the GM does, and how many presses

Placed by how often it is done (Alex's rule, 23 Nov).

| How often | Action | How |
|---|---|---|
| all the time | fire a preset where it was written to go | press its fire button (or keys 1 to 9 for the first nine on the page) |
| all the time | send the live view | press Send on Live (goes to Everyone), or the Send button on the chart |
| often | send something somewhere else | drag from the source's dot to a screen, a role or Everyone |
| often | hold or release one line | click the line |
| often | hold everything | **Hold all**, always in the same corner, key H |
| often | take the last send back | **Back** on the line: the destination returns to what it showed before |
| often | turn the signal dial | the dial on the role's heading (all its screens) or on one screen's box |
| sometimes | choose the holding card, latch Following | click the line's dot: the **line card** opens on the line itself |
| sometimes | move a screen to another role | drag the screen onto the role's heading |
| sometimes | swap two screens | drag one screen onto another: they trade roles |
| sometimes | add a preset | **Keep** on Live, "+ Add" in a section, or Keep on the "just now" strip (section 4.4) |
| sometimes | fold a section out of the way | press its heading |
| rarely | change a role's knowledge, layout, look or brand | the role card |

Every drag has a press-only twin, for touch and for keyboards: press a source's dot, then press a destination.

### 3.5 Where it lives

- **Its own screen document** (`switchboard.gm.json`) with a `rail` entry, as CLAUDE.md requires. The entry is called **Switchboard** and sits in a rail group named **Table**, present at every scale and on every desk (question 3). No tab is added.
- **The on-air chip.** While a session is shared, the top bar carries one small chip, for example "On air · 4 screens · 1 held". It opens the Switchboard from where it sits. It replaces the "Table sees" line, the share link and the request count on the status line.
- **One Send button stays on the chart**, on the Run desk only, so the commonest live act is still one press from the chart. It sends to Everyone; its small arrow lists the roles. It is the Live source's Send, shown in a second place, and it goes down the same path.
- **Every object's menu** (NV1's one action list) gains two entries: "Send to ▸" with the roles, and "Keep as a preset".
- **Generally available.** The Switchboard opens on Explore and Build as well as Run. On Build with nobody connected it is where presets are prepared: the right column shows each role with one stand-in preview, "what Bridge would see".
- **In its own window.** "Open in its own window" puts the Switchboard on a second monitor while the chart keeps the first. This is the two-screen GM set-up Alex describes; on a one-screen night it is a rail entry like any other.

## 4. The Playbook

### 4.1 Shape

- **Chapter**, then **page**, then **section**, then **preset**. A chapter is a session or a part of the story. A page is a scene: what the GM wants in hand at one moment. A section is only a heading on a page.
- The page picker is one line at the top of the hand ("Ch 2 · Arrival at Tarn ▾"). It opens a two-level list. Changing page changes nothing on any screen.
- A preset that has been fired this session carries a quiet tick and the table time. It can be fired again. Nothing is ever locked, greyed out or skipped for the GM.

### 4.2 What a preset holds

A preset is a short list of **steps**, each one a thing the GM could do live, plus a role name. Most presets have one step.

| Step | Sends | Exists today? |
|---|---|---|
| **Show** | a place, with its view (layout, view style, seen from, band, HUD) | yes: `Shown {scope, title, screen}`; the view fields are new |
| **Land** | a landing at a site | yes: `Shown.landing` (`LandingShow`) |
| **Reveal** | a Known to change for the role's audience | yes: the `set_known` intent |
| **Read aloud** | words for the GM to read; optionally also a card on the screens | new `Shown` kind |
| **Look** | a look for the role, a signal level, or both | Looks not built yet |
| **Hold** | a holding card (blank is "screens off") | new `Shown` kind, replacing `{off: true}` |

Rules that keep it one mechanism:

- **A preset knows nothing about screens.** It names a role, or Everyone. It never names a device.
- **A preset holds no knowledge of its own.** A Show step says "this place", not "these facts". What a screen draws is always its own role's knowledge through its own sensors, exactly as a live send. So sending a preset to a different role than it was written for can never leak anything.
- **Only a Reveal step changes what anyone knows**, and a preset with one wears the small eye from the secrets design, so the GM can see before pressing that this one tells the table something. A Reveal goes to the audience of the role it is fired at.
- **A band the role cannot see** (Infrared sent to a role with no infrared sensor) shows the nearest band it has. The preset row says so in a title ("Bridge has no Infrared: will show Visible").
- **No conditions, no timers, no "then"** in 4.0. Steps in one preset go together, at once.

### 4.3 Read aloud

A Read aloud step shows its words in the GM's monitor, large enough to read at arm's length. By default nothing goes to any screen: firing it marks it read, so it can sit in the hand among the sends. A switch on the step, "also show on screens", sends the words to the role as a plain card that arrives by the look's delivery (Typewriter suits it). This is the first use of documents on a player screen that are not a place.

### 4.4 Adding a preset

Alex, 24 Nov 15:53: "how do we define new items to add to the playbook?" The rule is **show it, then keep it**. A new preset is never built from a blank form: it takes what the GM is looking at or has selected, and the GM only names it and says where it goes. Four doors, most used first:

1. **Keep on Live.** Beside Send. Whatever is on the GM's view (place, layout, view style, band, a landing in progress) becomes a Show or Land preset on the open page. This is the fastest way to prepare: fly somewhere, frame it, Keep.
2. **"+ Add to …" at the foot of each section.** It opens one row of six kinds: This view, A landing, A reveal, Read aloud, Look or signal, A hold. The first three take the GM's current view, picked site or selected object. Read aloud opens a box to type in. Look or signal takes the role's current look and dial. A hold asks only for the card.
3. **Keep on the "just now" strip.** Anything sent live in the last few minutes, kept after the fact.
4. **"Keep as a preset" in any object's menu**, on any desk, so a preset can be made from the chart without opening the Switchboard.

A new preset arrives in the section it was added to, already selected, with its name ready to type over and a "goes to" choice beside it (Everyone, or a role). Enter or Done finishes it. Nothing else is required.

### 4.5 Sections fold

Alex, same message: sections "like 'Approach' as collapsible". Yes.

- Pressing a section's heading folds it to one line that keeps its name and count ("The beacon · 3 presets").
- Folding is the GM's own, kept on the device. It never changes the Playbook and nothing folds by itself, since a section that closed when its presets had been fired would be a script by the back door.
- Keys 1 to 9 count only the presets in view.
- A page's heading in the picker works the same way one level up.
- "+ Section" sits at the foot of the page, and "+ Page" and "+ Chapter" in the picker.

### 4.6 Edit, reorder

- **Edit in place.** A selected preset's pencil opens the same name and "goes to" row it was made with, plus its steps and a note to self. On Run the row opens only when asked for, so a session is not an editing screen.
- **Reorder by dragging the row** by its body, within a section, onto another section's heading, or onto a page name in the picker.
- **"Update from live"** on a selected Show preset replaces its place and view with what the GM is looking at now.
- **Again** on the "just now" strip repeats a send.

### 4.7 Where the Playbook is kept

In the campaign, as one document (`sse.playbook/1`), exported and imported with it. A pack may carry a playbook (an adventure pack's scenes), which appears as extra chapters marked with the pack's name and can be copied into the campaign to change. Sketch in section 11.

## 5. Sending, in detail

### 5.1 What a destination shows

Each destination has exactly one current source. A screen with a line of its own uses that; otherwise it shows its role's. A role with no line of its own shows Everyone's. This is today's rule (`Seat.shown` over the table's `shown`) with one more level.

### 5.2 Arrival

A new picture arrives by the role's look's **screen delivery** (Fade, Scene wipes, Power on, Static burst, Typewriter, Scan in), from the screen before or from nothing, as the Looks note rules. Until Looks is built everything arrives by Fade.

### 5.3 Following

A line from Live is a snapshot by default: the destination gets the place the GM was at when the line was made, and the GM is then free to look elsewhere in private. This is today's behaviour and it protects secrets.

A line from Live may be switched to **following** (a small latch on the line). The destination then goes where the GM goes, drawn through its own knowledge. The line turns accent colour and the on-air chip says "Bridge is following you", because a GM who forgets this will show the table where the secret base is by going to look at it (question 6).

### 5.4 Back

Each destination remembers the one source it had before. **Back** on its line returns to it. A Reveal is not undone by Back: the eye's own "take it back" does that, on purpose, in the place where knowledge is managed.

### 5.5 Roles with no screen tonight

A preset aimed at a role with no screen is not an error. Its fire button shows the role name hollow, with the reason as its title. The GM can:

- fire it anyway (the role remembers it, and a screen that joins that role later shows it), or
- drag the role's heading onto another role: "Telescope is on Bridge's screens tonight". Presets for Telescope then go to Bridge's screens until the GM undoes it. The screens keep their own role's knowledge and sensors.

This is the "runtime layer that redirects" Alex asked for. It lives in tonight's table set-up and never in the Playbook.

### 5.6 What is remembered between sessions

- **The campaign** keeps roles and the Playbook.
- **Each screen** remembers the role it last had and its own name, on that device, and asks for that role again when it rejoins. The GM sees it arrive where it was last week and moves it if that is wrong.
- **Tonight's lines, holds and redirects** are live state on the GM's machine. They are not saved and they do not travel in the campaign file.

## 6. Holding

### 6.1 One mechanism

Clicking a line holds it. The destination's screens get a **holding card** by the look's delivery (Static burst is the natural one) and stop receiving the picture. The builder's point is accepted: a hold must be a send, so that a held screen truly has nothing behind the card, and a player cannot peek under it.

While a line is held the GM can do anything to it: change its source, fire presets at it, turn its dial. The line shows what is waiting. Releasing it delivers the latest, once.

**Hold all** holds every line. It is the one control that never moves, because it is what a GM reaches for when something has gone wrong.

### 6.2 The cards

A holding card is one line of words on the role's look. Three kinds:

- **Blank**: a dark screen. This is "Screens off".
- **Plain**: "Awaiting transmission", "Signal lost", "Stand by", "No carrier".
- **Jokes**: drawn at random from a long list, never the same one twice running.

A click uses the kind the GM used last. The line card (the line's dot) chooses the kind, a particular card, or words typed for this moment.

### 6.3 The list is pack data

`sse.holding-cards/1`, in the base presentation pack: each card has its words and a kind. The GM edits the list in Workshop › Looks (a fifth row on the look card, "Holding cards", so a look can carry its own in-fiction set). A pack adds cards; a setting pack can replace the base jokes with ones that fit its world. First set, in the voice of the opening lines (short, dry, second person where it helps, British spelling):

1. Hold on: tracking down the missing gold atoms. I think the black hole swallowed them. *(Alex)*
2. Awaiting transmission. Light is doing its best.
3. Signal lost. It was last seen heading for the Oort cloud.
4. Recalculating. Someone moved the barycentre.
5. Please wait. The star is being asked nicely to hold still.
6. Stand by. Counting the moons again. There was one more a minute ago.
7. Your picture is in a queue behind 4.2 light-years of other pictures.
8. No carrier. The carrier says the same about you.
9. Tuning. This frequency is mostly hydrogen.
10. One moment. The navigator has dropped the slide rule.
11. Signal lost. Have you tried turning the antenna toward the thing?
12. Hold on. Waiting for the photons to finish their random walk out of the core.
13. Buffering. About eight minutes at this distance.
14. Stand by. The ship's computer is having a quiet word with itself.
15. Transmission delayed by solar weather. Bring a coat.
16. Please wait. Re-deriving the orbit from first principles, in pencil.
17. Interference. Possibly a pulsar. Possibly the microwave in the galley.
18. Signal lost. The relay has gone behind a planet. It does this.
19. Hold on. Somebody divided by the eccentricity.
20. Stand by for picture. The picture is standing by for you.
21. Nothing to see here yet. Space is like that.
22. Aligning the dish. Left a bit. No, your left.
23. Awaiting transmission. The message is redshifted and in a mood.
24. Please wait. The tide is out.

All original. None quotes a film, a book or a game.

### 6.4 Care for the people watching

The Looks rulings carry over unchanged: no fault or card flashes the whole screen more than three times a second, and a player's reduced-motion setting turns any delivery into a plain fade, whatever the GM does.

## 7. Looks and roles on the Switchboard

### 7.1 The signal dial sits on the boxes

Alex, 24 Nov 15:52: "the 'interference slider' should probably make it to the relevant right hand boxes". It does. The first draft put the dial on a card behind the line's dot, which hid a control that is used often.

- **Every screen's box carries its own dial**, under its preview, so the hand is beside the picture it changes: clear at one end, lost at the other, a tick at the look's saved level.
- **A role's heading carries one dial for all its screens.** Moving it moves every screen in the role together and their own dials follow. Moving one screen's dial afterwards changes only that screen.
- A dial is there only when the role's look has a signal fault, as ruled.
- The line into a role or a screen roughens as its dial rises, so the whole board shows at a glance who has a bad picture.
- "Tune in" is a held line released with the dial at the far end and drawn down.
- A Look step in a preset ("Signal breaks up") sets the role's dial, and the GM's hand can take it from there.

### 7.2 The look belongs to the role

A role's heading shows its look in words ("Bridge · Green terminal"). Pressing those words lists the campaign's looks. The Looks ruling holds: a look the GM gives holds for that role, with "players may change it" as a switch on the role card, and accessibility settings on the player's device always win.

### 7.3 The role card

Opens from a role's heading, and grows from it. It holds, in order of use:

1. **Name**, and **Look**.
2. **Knows what**: the audience (whose knowledge), with a link to the table ledger from the secrets design. The Switchboard never edits knowledge itself; "Known to" stays the one control for that.
3. **Layout**: which screen document this role opens (Bridge, Survey report, Bridge in 3D, Landing).
4. **Sensors**: the bands this role can be sent.
5. **More**, closed by default: what the role may ask for and what applies without asking; brand and logo; tabletops it may sit inside; sealed (for competitive play); the quick switches ("hide labels").

This is today's Seats dialog, field for field, rearranged by how often each is touched. One role today can list several layouts, each meant as its own window. That becomes one layout per role: a seat with two screens becomes two roles that share an audience (question 4).

### 7.4 The pinned preview

Alex called the pinned preview "genius", so it stays, and it is now the same picture as the Switchboard's: **Pin** on any screen's preview pins it to the corner of the chart, so the GM can work on the chart and watch one screen. Pressing another screen's Pin moves it. For a full-size look, a preview's own "See full size" replaces the list in the desk's "See as" menu.

## 8. What the Switchboard absorbs

Found by reading main (`packages/shell/src`) and from the builder's account (`/mnt/project-files/sse4/plans/switchboard-what-exists.md`).

| Today | Where | Becomes |
|---|---|---|
| "Table sees: …" line | App.svelte status line | the on-air chip; in full, the lines |
| "Send this view" (table) | status line | Send on Live; the one Send button on the chart |
| "Screens off" (table) | status line | Hold all, with the blank card |
| Share link | status line | Add a screen |
| Requests waiting, approve and refuse | status line | a badge on the screen that asked, with the same two buttons; the chip carries the count |
| Seats dialog: add a seat, fog of war, screens, brand, logo, tabletops, sealed | `SeatsPanel.svelte` | the role card |
| Seat rows: join links per screen | `SeatsPanel.svelte` | Add a screen |
| Seat rows: Send, Screens off, Back with the table | `SeatsPanel.svelte` | a line to that role; hold; removing its own line |
| Seat rows: quick switches | `SeatsPanel.svelte` | role card, More |
| Seat rows: mark as discovered | `SeatsPanel.svelte` | already going to the eye (SV); not carried here |
| Looks seat rows: Look, signal dial, Send (designed, not built) | example packs 4D | role heading; a dial on each role heading and each screen's box |
| Pinned preview (designed, not built) | example packs 4D | Pin on any preview |
| "See as: the table ▾" | desk menu | "See full size" on a preview; the menu keeps one entry, "As players see it", which opens the Switchboard |
| "Tell the table?" on arrival | `arrival.ts`, uncommitted | a suggested send on the "just now" strip, with Send and Keep |
| Show the landing to the table | `ws.sendLanding` | a Land step; live, a line from Live while landing |

After this, `SeatsPanel.svelte` and the table half of the status line are deleted, not hidden.

**Not absorbed**, on purpose: Known to and the eye (knowledge is one control, and it lives on the thing known); the table ledger; Workshop › Looks (building a look is preparation); the party line on the players' own header.

## 9. OBS out: the clean output link

Accepted for 4.0. Each role offers a **clean output link**: the role's picture with no buttons, no pointer, no header, at a fixed 16:9, and optionally with no background so the chart or the ship floats over the streamer's own scene.

- It is **a screen like any other**: it joins the role, appears in the right column marked "Output", and takes lines, holds, looks and the dial. A streamer makes a role called "Stream", gives it the audience they want viewers to have, and points an OBS browser source at its link. No special case.
- Because it is a role, **viewers see only what that role knows**. A stream cannot leak the GM's secrets.
- It receives data and draws its own picture, like every screen. Nothing is streamed from the GM.
- Sound is off by default, as everywhere.
- **NDI** stays parked. A browser cannot speak NDI. Someone who needs it today can get it by running the output link through OBS, which has an NDI plug-in.

## 10. OBS in: a camera as a source (later)

Accepted in principle, with the limit already agreed: it never leaves the GM's machine.

- **Add a camera** on the sources side lets the GM pick any camera the browser offers. OBS's virtual camera is one of them, so anything OBS can compose (a face cam, a map, a video clip, another program's window) becomes a source.
- **Its lines can end only on screens open on this computer** (a wall TV on the GM's second output, a local window). A remote screen dims when a camera line is dragged over it, with the reason: "A camera picture stays on this computer. The GM sends only data."
- It is the one source that is not data, so it is also the one source that cannot be kept as a preset that works on someone else's machine. A preset may name it ("Camera 1"); on a machine without it the preset holds instead.
- Marked **later**: it is not needed for the run-through, and it needs the local-window work of package PB9 first.

## 11. Data shapes (sketches for the builder)

Names are proposals. Everything is plain data; nothing here feeds physics or RESOLVE.

```jsonc
// A source: what a live send, a preset step and a line all carry. Extends today's Shown.
{ "show":  { "scope": "u:…/tarn/iv", "title": "Tarn IV close up", "layout": "players.survey.screen.json",
             "view": { "style": "chart", "seen_from": "tilted", "band": "visible", "hud": true } } }
{ "land":  { /* LandingShow, unchanged */ } }
{ "say":   { "text": "The beacon repeats every ninety seconds…", "on_screens": false } }
{ "hold":  { "card": "blank" | "plain" | "joke" | { "text": "…" } } }   // replaces { off: true }
{ "camera": { "name": "Camera 1" } }                                     // GM machine only

// A preset: steps plus a role name. Knows nothing of screens.
{ "id": "tarn-iv-close", "title": "Tarn IV close up", "to": "telescope" | "everyone",
  "steps": [ { "show": { … } }, { "reveal": { "address": "…/wreck", "level": "known" } },
             { "look": { "look": "green-terminal", "signal": 0.6 } } ],
  "note": "wait for them to ask about the moon" }

// The Playbook, in the campaign.
{ "schema": "sse.playbook/1",
  "chapters": [ { "title": "Arrival at Tarn", "pages": [ { "title": "Tarn IV",
      "sections": [ { "title": "Approach", "presets": [ … ] } ] } ] } ] }

// Tonight, on the GM's machine only (host state; replaces `shown` and `Seat.shown`).
{ "lines": { "everyone": { "source": …, "state": "live" },
             "role:bridge": { "source": …, "state": "held", "card": …, "before": … },
             "screen:k3f9": { "source": …, "state": "following" } },
  "screens": { "k3f9": { "name": "Mira's tablet", "role": "bridge", "local": false, "showing": … } },
  "signal": { "role:bridge": 0.52, "screen:k3f9": 0.12 },   // a screen's own level wins over its role's
  "redirects": { "telescope": "bridge" } }
```

A screen's join gains one step: it says hello with a screen id it keeps on the device, its name and the role it would like. The GM's machine answers with the role it has.

## 12. Standing rulings, checked

- **GM offline means players read-only.** Lines, holds and redirects live on the GM's machine. If it goes away, every screen keeps its last picture or card and shows the usual "GM away" mark. When the GM returns the lines are sent again.
- **The GM streams only data.** Sources, lines and cards are data. Previews are drawn locally. A camera never leaves the machine.
- **Seats get only what is Known to them and only their sensors' bands.** Knowledge and sensors belong to the role a screen is in, never to a preset or a line. Moving a screen to another role is the only routing act that changes what it may know, and the drop says so ("Mira's tablet will see what Telescope knows").
- **Honesty marks on the GM view only.** The monitor on the left is the GM's view and has them. The previews on the right are players' views and have none.
- **Players never hand-pilot.** Nothing here gives a screen controls.
- **Danger is drama.** Nothing is refused: a preset for a role with no screen, a band a role lacks and a hold on a line mid-landing are all allowed and all say what will happen.
- **One mechanism.** Sources, destinations and lines. Screens off, per-seat send, seat rows, the preview, the dial and OBS out are each a case of it.
- **Sealed roles** (competitive play) work the same from the Switchboard. The builder notes a per-seat hold is not yet honoured for sealed seats; PB1 fixes that as part of moving to lines.

## 13. Phone, light and dark, motion

- **Phone, as the GM.** No lines are drawn. The screens are a list, each row a preview with its role and "showing …" in words. The open page is a strip of presets along the bottom, in thumb reach; press one to fire it where it was written to go, press and hold to choose where. Press a screen's row for its sheet: Hold, the dial at full width, Send live here, Move to. Hold all stays at the top.
- **Light chart room and Night chart** both, from the tokens. Previews are always dark glass, since they are pictures of screens.
- **Motion** follows `interface-style.md`. A line draws itself from source to destination in about 200 ms when a send is made, which is the one flourish and it explains the change. A hold snaps the line apart. The role card and the dial grow from what opened them. Reduced motion makes all of it instant.
- **Cost.** One preview is one scene build, and screens in one role with one source share it, so the cost follows roles, not screens. Previews are drawn in Chart style and only when something changes. The pinned or selected preview may draw True (the builder's recommendation).

## 14. Questions for Alex

1. **Sources, destinations and lines as the one mechanism**, with "Screens off" becoming a hold with a blank card and per-seat send becoming a line? Recommended: yes.
2. **Say "role" and "screen" on the GM's side, and drop "seat" from the interface** (the code keeps it)? Recommended: yes. A seat already behaves as a role, and "screen" is the word you used.
3. **The Switchboard as a rail entry in a new last rail group, "Table"**, on every desk and at every scale, plus the on-air chip in the top bar and one Send button on the chart? Recommended: yes. The alternative is to reach it only from the chip, which hides it before a session is shared, when presets are being prepared.
4. **One layout per role**, so today's seat with two screens becomes two roles sharing one audience ("Bridge" and "Telescope", both the Kestrel's crew)? Recommended: yes. It is what makes "fire at Telescope" mean one thing.
5. **A preset row is two parts: the body previews it in the GM's monitor, the fire button sends it**, with no confirmation? Recommended: yes. One press to send, and browsing the hand is safe.
6. **Following: a line from Live may be latched to follow the GM**, off by default, shown in accent on the line and named on the chip? Recommended: yes, off by default. Without it a GM narrating a fly-through must press Send at every stop.
7. **Back, one step per destination**, which never undoes a Reveal? Recommended: yes.
8. **A role with no screen tonight can be dropped onto another role** as tonight's redirect, never saved in the Playbook? Recommended: yes.
9. **Read aloud stays on the GM's monitor unless its "also show on screens" switch is on?** Recommended: yes.
10. **Presets never move the universe clock in 4.0.** A Show step may say "as it was at this date" for the picture only. Recommended: yes. Moving the clock changes the world for everyone and should stay a deliberate act on the ruler; a preset that does it waits for 4.2 with triggers and timers.
11. **Holding cards as pack data with the first 24 above, editable in Workshop › Looks as a fifth row, and a look may carry its own set?** Recommended: yes. Strike any joke you do not like.
12. **Delete the Seats dialog and the table half of the status line once PB7 lands**, with no old path kept beside the new one? Recommended: yes. Keeping both is how small interfaces come back.
13. **OBS out in 4.0 as an ordinary screen on an ordinary role, with the transparent background option; camera in marked later; NDI parked?** Recommended: yes. This is the agreed position with the order made explicit.
14. **Build order: PB1 to PB3 first** (lines, the Switchboard with send and hold, the Playbook with fire and keep), because those three are what the run-through needs? Recommended: yes.

## 15. Packages (for "Build Run desk playbook and players' side")

Files in `packages/shell/src/run/` and `packages/ui-contract/src/`. One at a time, checks passing before the next. Screenshots to `/mnt/project-files/sse4/handover-screens/<PKG>-screens/`.

| Pkg | What | Needs | For the run-through? |
|---|---|---|---|
| **PB1** | Contract: `Source` (show with view, land, hold; `{off}` read as a blank hold), lines for Everyone, role and screen with `before`, the screen hello (id, name, wanted role) and its "showing" report. Sealed roles honour a hold. Old `shown` and `Seat.shown` read and upgraded. | nothing | yes |
| **PB2** | The Switchboard screen: Live source, roles and screens as a list with lines, Send, click to hold, Hold all, Back, drag and press-then-press, move and swap screens. The on-air chip and the chart's Send button. Rail entry. The status line's table half goes. | PB1 | yes |
| **PB3** | The Playbook: `sse.playbook/1` in the campaign; chapter and page picker; folding sections; fire; keys 1 to 9; Keep on Live and "+ Add" with its six kinds; the "just now" strip with Keep and Again; edit row; reorder; Update from live; Reveal steps with the eye. | PB2 | yes |
| **PB4** | Previews: a Chart preview per role and source, shared; the confirmed tick; Pin to the chart; See full size. | PB2 | helps |
| **PB5** | Holding cards: `sse.holding-cards/1` in the base pack, the 24 lines, kinds, chosen on the line card, no repeat twice running. | PB2 | small, yes |
| **PB6** | Read aloud: the `say` source, the GM monitor reading card, "also show on screens" as a player document. | PB3 | yes if the run-through has narration |
| **PB7** | The role card: the Seats dialog's fields moved in; one layout per role with the upgrade of two-screen seats; redirects; requests on the screen that asked. `SeatsPanel.svelte` deleted. | PB2 | no |
| **PB8** | Clean output link: `?clean=1`, fixed 16:9, transparent option, marked "Output" on the board. | PB2; renderer for the transparent ground | no, but small |
| **PB9** | Phone layout; "Open in its own window"; "Open a screen on this computer" keeping a link to its opener. | PB2 | no |
| **PB10** | Looks on the board: look per role, the dials on role headings and screen boxes, deliveries on arrival and hold, Following latch. | renderer's looks (filters, faults, deliveries) | no |
| **PB11** | Later: camera in, on local screens only. | PB9 | no |

Asks of other threads: the renderer for the cheap Chart preview at thumbnail size, the transparent ground and the looks pieces already assigned to it; the secrets work (SV) for the eye on Reveal steps; navigation (NV1) for the two object-menu entries.

## 16. Not in 4.0

Kept out so the run-through stays small; most belong to 4.2, the RPG release.

- Triggers, timers and conditions that fire presets by themselves; presets that move the clock.
- A co-GM or Minerva drafting pages from the campaign's record.
- Sound and music cues as steps.
- Dice, checks or any game rule on a preset.
- Players asking for a picture ("put the scope on that moon") as a request line into the Switchboard.
- Picture-in-picture and split screens on one destination.
- NDI.
