# SSE4: the Playbook and the Switchboard (track PB)

Status: **ACCEPTED by Alex, 24 Nov 2026 16:34** ("Agree with all your recs above", all 22 questions of section 14; and on who steers a screen: "This is a bit 'suck it and see' to be honest - to see how it works out", so section 5.3 is built to be tried and changed). The direction was accepted earlier the same day (15:39, memory `sse4-run-desk-beats`). Design and mockup only; nothing here is built by this note. Packages for the builder are in section 15. A picture of how the pieces fit, for newcomers and for us: `/mnt/project-files/sse4/handover-screens/PB-screens/concept-map.html`.

Mockup, clickable: `/mnt/project-files/sse4/handover-screens/PB-screens/switchboard-mockup.html`, with desktop and phone screenshots beside it; live at https://claude.ai/artifact/FJYtkbakTj4kbi55PdczU2 . The mockup shows the Switchboard on a desktop and a phone, light and dark: sending, dragging, holding, the signal dials on the boxes, the line card, Hold all, folding sections, adding a preset, the Everyone layer and Lift, Undo and Redo, the Bookmarks list, the layout list and the new-layout builder made of bookmarks, picture and sound presets, Keep and Again. The role card, Add a screen and the page picker are described here and not mocked.

Alex's brief, in his words (24 Nov): "a gm must always be able to adapt and remix... presets... are great to have 'in hand' but live is the key"; "no need for next - just let the GM build up a 'playbook'... chapters and pages and sections - each can be a preset that is fired at various windows"; "a routing diagram showing what everyone sees - gm can pause any one by clicking on the line"; "a mini video routing package"; "the routing screen should be generally available and does remove the need for a bunch of other smaller interfaces - we corral the user to the 'nerve centre'".

## 1. The short answer

1. **Everything the GM can show is a source. Everything that can show it is a destination. A line joins them.** That one sentence is the whole Switchboard. The GM's live view is a source. Each preset on the open Playbook page is a source. A camera is a source. A destination is a screen, a role (all its screens) or Everyone (all roles).
2. **Sending is making a line.** Press a preset and its line appears to the role it names. Drag from any source to any destination to send it somewhere else. "Send this view" is a line from Live. Nothing is sent any other way.
3. **Nothing sent destroys what it replaced.** A send to Everyone is a layer over the set-up that lifts off again; a fired preset's button un-presses to go back; Undo and Redo sit at the top.
4. **Holding is breaking a line.** Click a line and it breaks: that screen gets a holding card ("Awaiting transmission", "Signal lost", or a joke). Click again and the picture comes back. "Screens off" is a hold with a blank card, so it is the same thing and not a second control.
5. **A preset is a saved send.** It holds exactly what a live press would send, plus the name of a role. So firing a preset and doing it live go down one path, and "keep what I just did" is free.
6. **The Playbook is pages of presets**, in chapters, with section headings. The open page is the GM's hand. There is no order, no Next and no script.
7. **Roles are named in the Playbook; screens are decided at the table.** A preset says "Bridge". Which glass is Bridge tonight is set by dragging on the Switchboard, and is never saved in the Playbook.
8. **Bookmarks are the only thing made elsewhere.** A Bookmark button beside Undo, on every desk, saves where the GM is and how they are looking at it. Everything players get on top of that (who sees it, look, signal, delivery, arrangement) is added in the Switchboard and nowhere else.
9. **It replaces the Seats dialog, the seat rows, the "Table sees" bar, the share link line and the planned Looks seat rows.** What stays on the chart is one Send button and one small "on air" chip that opens the Switchboard.

## 2. Words

Seven words, used the same way everywhere.

| Word | Means | Today |
|---|---|---|
| **Screen** | One piece of glass showing a players' page: a tab, a phone, a wall TV, an OBS source | a tab that opened a seat link |
| **Role** | A named place to send things: "Bridge", "Telescope". It carries whose knowledge it shows, its sensors, its screen layout and its look | a `Seat` (`ui-contract/src/seat.ts`). The builder confirms a seat is already a role in effect: any number of screens may open one seat link |
| **Everyone** | Every role at once | the table-wide `shown` |
| **Source** | Something that can be shown: Live, a preset, a camera | a `Shown` value |
| **Line** | A source reaching a destination, with its state: live, driven, held | `shown` per table and per seat |
| **Bookmark** | Where the GM is and how they are looking at it, saved. Nothing about players | the "place mark" of the top-level design (section 3.2 there), not built |
| **Preset** | A bookmark (or a landing, words, an asset) plus where it goes and how it arrives | not built |
| **Layout** | Bookmarks arranged on one screen, with a look | `sse.screen/1`, the player screen document |
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

**Screens on this computer.** Alex, 24 Nov 16:37 to 16:38: "how do we add screens on our PC - I suggest we have a 'screen' as a physical screen connected to a PC or a subdivision - a simple dual/3/quad layout... so a physical screen could be a composite screen made up of 2-4 screens - useful on big 4k displays."

- **A screen is a display attached to this computer, or a part of one.** "Add a screen", "On this computer" lists the displays the computer has (the second monitor, the projector, the wall TV on HDMI), each drawn as a small rectangle in its real position. Press one and SSE opens a full-screen window on it. Where the browser cannot list displays, the same button opens a window to drag there and press F for full screen.
- **Split, kept out of the way.** Alex, 16:39: "most of the time it will be a network display or a full screen, so this subdivision is an advanced feature and can be a bit hidden." A display is Whole unless changed, and nothing on the board mentions splitting. Under "More" on a local display's box there is one choice: **Whole, Two, Three or Four**. Two is side by side; Three is one large and two small; Four is two by two. Each part is a screen in its own right: its own box on the board, its own role, line, dial and hold. So one 4K television can be the Bridge on the left and the Telescope on the right, or four roles at once.
- **A split display gets its own heading on the board, with its parts beneath it.** Alex, 16:40: "effectively it would create multiple displays under a heading. Fed into the heading it's the full display - feed into one part it's that part of that display." So "Wall TV" becomes a heading with "left" and "right" under it, each part a screen box with its own preview, dial and hold, and each part naming the role it belongs to in a small word that can be changed.
  - **A line to the heading fills the whole display.** It lies over the parts as a layer, exactly as Everyone lies over the set-up, and lifting it gives the parts back as they were.
  - **A line to a part fills that part.** A send to a role reaches whichever parts belong to that role.
  - Dragging one part onto another swaps them on the glass.
- **A split is not a layout.** A layout arranges boxes inside one screen for one role (one audience, one look). A split makes several screens out of one piece of glass, each of which then has its role's layout. The GM only needs to know: split the glass first, then treat each part as a screen.
- The split belongs to the table set-up on this computer, never to the Playbook, like every other fact about screens.
- A phone or tablet that joined by link is always Whole.

### 3.3 Between: the lines

A line is drawn from a source to whatever it was sent to. Its look says its state without words:

| Line | Means |
|---|---|
| solid ink | live: the destination is showing this source |
| solid accent | driven: a read-only screen tracking the GM's stage (section 5.3) |
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
| often | take the last send back | press the preset's button again; **Lift** on Everyone; **Undo** at the top (section 5.4) |
| often | turn the signal dial | the dial on the role's heading (all its screens) or on one screen's box |
| sometimes | choose the holding card | click the line's dot: the **line card** opens on the line itself |
| sometimes | move a screen to another role | drag the screen onto the role's heading |
| sometimes | swap two screens | drag one screen onto another: they trade roles |
| sometimes | add a preset | **Keep** on Live, "+ Add" in a section (from a bookmark), or Keep on the "just now" strip (section 4.4) |
| sometimes | fold a section out of the way | press its heading |
| rarely | change a role's knowledge, layout, look or brand | the role card |

Every drag has a press-only twin, for touch and for keyboards: press a source's dot, then press a destination.

### 3.5 Where it lives

- **Its own screen document** (`switchboard.gm.json`) with a `rail` entry, as CLAUDE.md requires. The entry is called **Switchboard** and sits in a rail group named **Table**, present at every scale and on every desk (question 3). No tab is added.
- **The on-air chip.** While a session is shared, the top bar carries one small chip, for example "On air · 4 screens · 1 held". It opens the Switchboard from where it sits. It replaces the "Table sees" line, the share link and the request count on the status line.
- **One Send button stays on the chart**, on the Run desk only, so the commonest live act is still one press from the chart. It sends to Everyone; its small arrow lists the roles. It is the Live source's Send, shown in a second place, and it goes down the same path.
- **Nothing else is added anywhere else.** No "send to" or "keep as preset" entries on object menus, no look or signal controls on the chart. Alex, 24 Nov 16:11: "This is the ONLY place we actually do all this filter adding, etc. Keep it all in one contained place - not distributed." The one thing the rest of the app offers is the Bookmark button (section 4A).
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
| **Show** | a bookmark: a place and how it is looked at (section 4A); optionally a layout | partly: `Shown {scope, title, screen}`; the bookmark is new |
| **Land** | a landing at a site | yes: `Shown.landing` (`LandingShow`) |
| **Reveal** | a Known to change for the role's audience | yes: the `set_known` intent |
| **Read aloud** | words for the GM to read; optionally also a card on the screens | new `Shown` kind |
| **Play** | a picture, a sound or a film (section 4.7) | new `Shown` kind |
| **Look** | a look for the role, a signal level, or both | Looks not built yet |
| **Hold** | a holding card (blank is "screens off") | new `Shown` kind, replacing `{off: true}` |

Rules that keep it one mechanism:

- **A preset knows nothing about screens.** It names a role, or Everyone. It never names a device.
- **A preset holds no knowledge of its own.** A Show step says "this bookmark", not "these facts". What a screen draws is always its own role's knowledge through its own sensors, exactly as a live send. So sending a preset to a different role than it was written for can never leak anything.
- **Only a Reveal step changes what anyone knows**, and a preset with one wears the small eye from the secrets design, so the GM can see before pressing that this one tells the table something. A Reveal goes to the audience of the role it is fired at.
- **A band the role cannot see** (Infrared sent to a role with no infrared sensor) shows the nearest band it has. The preset row says so in a title ("Bridge has no Infrared: will show Visible").
- **No conditions, no timers, no "then"** in 4.0. Steps in one preset go together, at once.

### 4.3 Read aloud

A Read aloud step shows its words in the GM's monitor, large enough to read at arm's length. By default nothing goes to any screen: firing it marks it read, so it can sit in the hand among the sends. A switch on the step, "also show on screens", sends the words to the role as a plain card that arrives by the look's delivery (Typewriter suits it). This is the first use of documents on a player screen that are not a place.

### 4.4 Adding a preset

Alex, 24 Nov 15:53: "how do we define new items to add to the playbook?" The rule is **bookmark it, then add the gloss here** (section 4A). A preset is never built from a blank form or from pickers.

1. **Keep on Live.** Beside Send. Bookmarks where the GM is and puts it on the open page as a preset. The fastest way to prepare: fly somewhere, frame it, Keep.
2. **"+ Add to …" at the foot of each section.** One row of kinds: A bookmark, A landing, A reveal, Read aloud, Picture, sound or film, Look or signal, A hold. "A bookmark" lists the campaign's bookmarks to choose from. A landing and a reveal also start from a bookmark (the site, the thing). Read aloud opens a box to type in. A hold asks only for the card.
3. **Keep on the "just now" strip.** Anything sent live in the last few minutes, kept after the fact.

A new preset arrives in the section it was added to, already selected, with its name ready to type over and a "goes to" choice beside it (Everyone, or a role). Enter or Done finishes it. Its look and signal come from the role it goes to unless the preset says otherwise.

### 4.5 Sections fold

Alex, same message: sections "like 'Approach' as collapsible". Yes.

- Pressing a section's heading folds it to one line that keeps its name and count ("The beacon · 3 presets").
- Folding is the GM's own, kept on the device. It never changes the Playbook and nothing folds by itself, since a section that closed when its presets had been fired would be a script by the back door.
- Keys 1 to 9 count only the presets in view.
- A page's heading in the picker works the same way one level up.
- "+ Section" sits at the foot of the page, and "+ Page" and "+ Chapter" in the picker.

### 4.6 Edit, reorder

- **Edit brings it back to the top.** Alex, 24 Nov 16:35: "clicking on the item at the bottom - we need an edit option that brings back the top screen... eg to let a sound change its screen from incoming transmission to distress call or whatever the GM types." Selecting a preset, in the Playbook or on the "just now" strip at the foot, shows it in the monitor at the top exactly as its screens will see it, with an **Edit** button on the monitor. Edit turns the monitor into the editor for that one thing, and every change shows there as it is made:
  - its name and where it goes;
  - what its kind allows. A sound: the words on its transmission card (typed over "Incoming transmission…", so "Distress call, automated, repeating" or anything the GM writes), which card style, and whether its name shows. Read aloud: the words. A bookmark: "Update from live", and the layout it brings. A hold: the card;
  - a note to self.
  Done or Escape puts the monitor back on Live. Editing never sends: a preset that is on screens now keeps showing what it was sent as until it is fired again, and the monitor says so ("on Bridge as sent at 21:43; fire again to update").
- On Run the editor opens only when asked for, so a session is not an editing screen.
- **Reorder by dragging the row** by its body, within a section, onto another section's heading, or onto a page name in the picker.
- **"Update from live"** on a selected Show preset replaces its bookmark with where the GM is now.
- **Again** on the "just now" strip repeats a send.

### 4.7 Pictures, sound and film

Alex, 24 Nov 16:03: "we could 'do nothing' to let the GM read - but here would also be useful to load an audiofile, image or video to play... those assets can come with some games... eg: show a map". Yes. A **Play** step shows or plays one asset, and it is a source like any other: it takes a line, it can be held, it lifts.

- **Read aloud keeps its plain form**: words on the GM's monitor and nothing sent. It may also carry one asset, so a read-aloud card can come with its handout or its recorded voice.
- **A picture** (a map, a handout, a portrait) fills the role's screens through the role's look, so a map on a green terminal looks like it belongs there. The look can be switched off for one picture.
- **A sound** plays **here**, on the GM's computer, by default, because at a table that is where the speakers are. Sent to a role it plays on that role's screens, and only on screens whose owner has turned sound on (sound is off by default, as ruled, and browsers insist on it).
- **A sound sent to screens has a face.** Alex, 16:34: "for an audio file have a default screen which says 'incoming transmission...' and have it able to be a 'look' - eg: distress calls." While a sound plays on a role's screens they show a **transmission card**: "Incoming transmission…", the sound's name if the GM wants it shown, and a moving trace of the sound itself. It is drawn through the role's look, so on a green terminal it is a green terminal's incoming call. A look may carry its own transmission card (a distress call with a red border and a repeating beacon mark; a calm ship's computer; a pirate hail), and a Play preset may name one. The card comes down when the sound ends and the screen returns to what it showed. Transmission cards are the same kind of pack data as holding cards (section 6.3), in the same file.
- **A film** plays on the screens it is sent to, with the same sound rule.
- **Where assets come from.** (a) A pack: a game or adventure pack carries its maps, handouts and sounds, and presets name them. Every screen already has the pack, so only the asset's name is sent. (b) The GM's own file, dropped on the Playbook: kept with the campaign. (c) A web address, which each screen loads for itself.
- **"The GM streams only data" still holds**, with one line drawn. A pack asset or a web address is a name. A GM's own picture or short sound is a file sent once, not a stream, and is allowed up to a small size (question 16). A GM's own film is too big to send that way: it plays on this computer's screens, or goes by web address or in a pack.
- **Whose work it is.** Assets in a pack follow the pack rules already set (our own or licensed; named-setting fan packs free). A GM's own files stay in their campaign and are not uploaded to Explorers with it unless they say so.

### 4.8 Another program as a source (later)

Alex, 16:03: "eventually - show the VTT - as we integrate Mappadux". This needs no new mechanism, only one more kind of source: a **web page**. A source that is a web address is shown in a panel on each screen, and each screen loads it for itself, so nothing is streamed from the GM. A virtual tabletop that offers a players' page by link becomes a source the moment its address is pasted in, and it then takes lines, holds and layers like everything else. It is the mirror of what SSE4 already does the other way (a seat shown inside a tabletop). Marked later, with Mappadux: what the two programs tell each other (which map, where the party is) is its own design.

### 4.9 Where the Playbook is kept

In the campaign, as one document (`sse.playbook/1`), exported and imported with it. A pack may carry a playbook (an adventure pack's scenes), which appears as extra chapters marked with the pack's name and can be copied into the campaign to change. Sketch in section 11.

## 4A. Bookmarks: the one thing made at source

**The whole model in four words** (Alex, 24 Nov 16:13): "bookmarks define 'what', layouts define 'how it looks' and playbooks store them. These are fired at screens with dynamic fog, which define 'where it is shown'."

| Word | Answers | Holds |
|---|---|---|
| **Bookmark** | what | a place, a date, an instrument and a way of looking |
| **Layout** | how it looks | bookmarks arranged on one screen, with a look |
| **Playbook** | where they are kept | presets in chapters, pages and sections, each a bookmark or layout with the role it is for |
| **Screen** (through its role) | where it is shown, and how much of it | the fog: each screen draws only what its role knows, with that role's sensors, worked out at the moment it is shown |

"Dynamic fog" is read here as the knowledge filter: the same bookmark fired at the Bridge and at the Scout shows each of them a different amount, and shows more the moment something is revealed, with nothing re-sent. It is never stored in a bookmark, a layout or a preset (question 21).

Alex, 24 Nov 16:11, on the layout builder: "where do you get the context of what you are looking at/from... lots of drop down pickers? ...perhaps we define 'bookmarks' at source and those become layouts when the extra gloss is added here. So effectively the GM can have anything on screen (they create a bookmark) and that is what is selected here in a new layout... next to undo/redo we have a bookmark that lets the user 'come back to this page' any time AND also use in 'looks' in this."

Accepted, and it replaces the earlier draft's pickers and its scattered "keep as preset" entries. It is also the "place mark" the top-level design already has (tours are ordered place marks), so this is one mechanism finding its third use, not a new one.

**What a bookmark is.** Where you are (the place and the date on the ruler), which instrument you are using (Chart, Globe, Cutaway, Landing, a card), and how you are looking (view style, seen from, scale, band, HUD, what is selected). It holds nothing about players: no role, no look, no knowledge. So it is safe and useful to everyone, GM or not.

**Where it is made.** One **Bookmarks** button beside Undo and Redo, in the same spot on every desk and every instrument. It opens a short list with "+ Bookmark where I am" at the top. Choosing a bookmark takes you back there, exactly as you left it. That is its first job and it needs no Switchboard: a learner uses it to come back to a page.

**What the Switchboard does with it.** Everything else:

| A bookmark, plus | is | made |
|---|---|---|
| who it goes to, and how it arrives | a **preset** | "+ Add" in a section, "A bookmark" |
| other bookmarks beside it, an arrangement and a look | a **layout** | "+ New layout…" on a role |
| a look or a signal fault being tuned | the **preview** for the look card | "New look…" on a role |
| a caption and an order | a **tour** stop | the tours design, unchanged |

So there are no pickers in the Switchboard for "which world, which instrument, which band". The GM goes there, gets it looking right, and bookmarks it. If what is wanted is not in the list, the hint says how: "Go there on the chart and press Bookmarks."

**Live is the unsaved bookmark.** The Live source is simply where the GM is now. Keep on Live bookmarks it and puts it on the open page in one press.

**Kept with** the campaign, and exported with it. A pack can carry bookmarks (an adventure's key places). Outside a campaign they are kept on the device.

## 5. Sending, in detail

### 5.1 What a destination shows

Each destination has exactly one current source. If an Everyone layer is on (section 5.4), every screen shows that. Otherwise a screen with a line of its own uses that, or else its role's, and a role with no line of its own shows the table's floor: the last thing Everyone was sent before any role had its own. This is today's rule (`Seat.shown` over the table's `shown`) with one more level and with the wipe removed.

### 5.2 Arrival

A new picture arrives by the role's look's **screen delivery** (Fade, Scene wipes, Power on, Static burst, Typewriter, Scan in), from the screen before or from nothing, as the Looks note rules. Until Looks is built everything arrives by Fade.

### 5.3 Who steers a screen

Alex, 24 Nov 16:34: "Users will be able to control their view and access the standard controls on the screen the GM has allowed. Unless it is a 'read only' screen - which means it follows the GM - when that is enabled then clicking on a display would then make it 'live' for the GM and they can change the viewpoint and change the screen as they wish and player view would track. Otherwise the player will have access to the controls they have on that screen and can't move off it." And: "This is a bit 'suck it and see'... to see how it works out." So this section is the first version to try, not a ruling to defend.

Every screen is in one of two states, shown as a small word on its box and switched with one press:

| State | Who moves the picture | What the player can do |
|---|---|---|
| **Theirs** (the default) | the player | Use the controls its layout gives it (turn, zoom, view style, band, the panels on that layout, whatever the role card allows). They cannot leave what was sent: no place line, no other world, no other layout. |
| **Read only** | the GM | Watch. Nothing on the screen responds except sound on and off. |

- **What was sent is the fence.** A bookmark sent to a screen that is Theirs is where that screen starts, and its place is as far as it goes. The player may look around it; a new send from the GM moves them on.
- **Driving a read-only screen.** Press the preview of a read-only screen on the board and it becomes the GM's Live: the GM's stage opens on that screen's picture, and as the GM flies, zooms, changes band or changes the layout, the screen tracks. The line turns accent colour and the on-air chip says "You are driving Wall TV". Press Live's own heading, or another preview, to let go; the screen stays where it was left. One screen, a role or Everyone can be driven.
- **It is drawn through their knowledge, not the GM's.** A driven screen goes where the GM goes but shows only what its role knows, with its role's sensors. The GM flying past the secret base shows the table empty space.
- **This replaces the "Following" latch** of the earlier draft. A read-only screen being driven is a following line; there is one idea, not two.
- **A line from Live to a screen that is Theirs is a snapshot**, as today: the GM sends where they are and is then free to look elsewhere in private.
- A role's heading sets the state for all its screens; one screen can differ. A preset may say which state it wants ("Landing at Hollin Flats", read only).
- "Players never hand-pilot" is untouched: these are view controls. Nothing here flies a ship.

**Reveals reach every view at once.** Alex: "The GM can unhide objects, etc. and reveal facts... but that would work whichever view you are in. It should be dynamic and show on all views." Revealing is a change to what a role knows, so it is not a send and has no line: every screen in that role redraws where it stands, whatever it is showing and whoever is steering it, and the GM can do it from any desk or instrument with the eye. This is the "dynamic fog" of section 4A.

### 5.4 Going back: press again, Lift, Undo

Alex, 24 Nov 15:58: "when you have an 'everyone', second click on that should go back to the last state... unclicking resumes old setup. Maybe an undo/redo at the top?" Yes to both. The rule is that **nothing sent ever destroys the set-up it replaced**, so there is always a way back, in three sizes.

**Everyone is a layer, not a wipe.** Today a table-wide send clears every seat's own send. That goes. A send to Everyone sits over the set-up: every screen shows it, and the lines to each role and screen stay where they were, drawn faint underneath. **Lift** on the Everyone bar takes the layer off and every screen returns to exactly what it had. A second Everyone send stacks on the first, and Lift goes back one. Anything sent to a role or a screen while a layer is on goes underneath it, ready for when it lifts.

**A preset's button is a switch while its picture is up.** Once fired, the button stays pressed for as long as that preset is what its destination shows. Pressing it again takes it back: an Everyone preset lifts, a role preset returns that role to what it had before, "Signal breaks up" puts the dial back, "Screens dark" releases. So "hold transmission" then un-click resumes the old set-up, as asked. A preset that is no longer showing fires afresh.

**Hold all remembers.** It notes which lines were already held. Release all puts each line back as it was: lines held before stay held.

**Undo and Redo, at the top beside Hold all** (Ctrl Z, Ctrl Shift Z). They step through tonight's changes to what screens show: sends, lifts, holds, dial moves, a screen changing role. This is the Switchboard's own history, separate from the Build desk's undo of edits to the world, and it is not saved. Undo never takes back a Reveal, because the table has already seen it: the eye's own "take it back" does that, on purpose. Undo says so when it steps over one.

**Back** on a line card stays as the one-step return for a single destination.

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

`sse.holding-cards/1`, in the base presentation pack: each card has its words and a kind. The GM edits the list on the look card (a fifth row, "Holding cards", so a look can carry its own in-fiction set), which opens in the Switchboard (section 7.4). A pack adds cards; a setting pack can replace the base jokes with ones that fit its world. First set, in the voice of the opening lines (short, dry, second person where it helps, British spelling):

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
- **Only the level is live.** Alex, 24 Nov 16:15 and 16:17: the digital noise and analogue noise controls of the old picker "need to be brought over too", and they are "defined in a look - we don't need to dynamically change it. Only dynamic controls (like level of interference) should be on a slider at all times." So the kind of fault (Analogue: tearing, a rolling bar, snow, ghosting, ending in static. Digital: blocks, frozen patches, colour smears, dropped frames, ending in blocks and black. Or both) and its finer settings are part of the look, set on the look card's Signal fault row, which opens here (section 7.4). The dial names the kind in small words beside it and does not change it.
- **The dial is there at all times**, on every role heading and every screen box. A look that saves no fault uses plain analogue interference and starts the dial at clear. This withdraws the Looks note's rule that the dial appears only when the look has a fault (question 22).
- **The rule for the board in general:** what changes during play (the level, hold, what is showing) is a control in reach at all times; what is chosen once in preparation (kind of fault, filters, palette, delivery) is in the look.
- The line into a role or a screen roughens as its dial rises, so the whole board shows at a glance who has a bad picture.
- "Tune in" is a held line released with the dial at the far end and drawn down.
- A Look step in a preset ("Signal breaks up") sets the role's dial, and the GM's hand can take it from there.

### 7.2 The look belongs to the role

A role's heading shows its look in words ("Bridge · Green terminal"). Pressing those words lists the campaign's looks. The Looks ruling holds: a look the GM gives holds for that role, with "players may change it" as a switch on the role card, and accessibility settings on the player's device always win.

### 7.3 The role card

Opens from a role's heading, and grows from it. It holds, in order of use:

1. **Name**, and **Look**.
2. **Knows what**: the audience (whose knowledge), with a link to the table ledger from the secrets design. The Switchboard never edits knowledge itself; "Known to" stays the one control for that.
3. **Layout**: which layout this role's screens open (section 7.4).
4. **Sensors**: the bands this role can be sent.
5. **More**, closed by default: what the role may ask for and what applies without asking; brand and logo; tabletops it may sit inside; sealed (for competitive play); the quick switches ("hide labels").

This is today's Seats dialog, field for field, rearranged by how often each is touched. One role today can list several layouts, each meant as its own window. That becomes one layout per role: a seat with two screens becomes two roles that share an audience (question 4).

### 7.4 Layouts: bookmarks, arranged, with a look

Alex, 24 Nov 16:02: "how do you actually define a setup... a wizard to help build a player screen makes a lot of sense. But also... make one in situ... you as a GM get a cool screen". And 16:11: the context comes from bookmarks made at source, with the gloss added here and only here (section 4A).

A **layout** is what a player screen is made of. It is today's screen document (`sse.screen/1`: Bridge, Tarn survey report, Bridge in 3D, Landing). "Layout" is the word used here so that "screen" can mean the piece of glass.

1. **Listed here.** Each role's heading names its layout ("Bridge ▾"). Pressing it lists every layout the campaign has: shipped, from packs, and made. Choosing one changes that role's screens at once. A preset may also name a layout, so "Landing at Hollin Flats" can bring the Landing layout with it and going back returns the role to its own.
2. **Edited here, in one editor.** Alex, 24 Nov 16:36: "New layout is actually edit layout - the inputs being a bookmark or a file/obs link", and "we need to try and simplify as we go. And order dragging for the boxes." So there is no separate wizard for a new layout. "Edit this layout…" and "+ New layout…" in that list open the same editor in place, with a live preview of the screen beside it; new is the editor on a blank copy. It has four lines, each optional:
   - **Arrangement:** One picture, Picture and side panels, Two by two, Report page, Blank.
   - **On it.** Each box takes one input, and there are only three kinds: **a bookmark** (anything in SSE: a place, an instrument, a card), **a file** (a picture, a sound, a film, from a pack or the GM's own), or **a link** (a web address: an OBS or camera feed on this computer, another program, a tabletop). "+ Where I am now" bookmarks the GM's view and puts it on. One special input, **"Whatever is sent"**, is the box that shows what a send or a preset brings; a layout with that box works with every preset, and the boxes beside it stay put.
   - **Look:** one of the campaign's looks, or "New look…".
   - **Name**, and Save gives it to the role it was opened from.
   - **Boxes are reordered and resized by dragging them in the preview.** Dragging one box onto another swaps them.
   - **Orac** can do the arranging from a sentence ("the wreck big, the survey card beside it, amber"), choosing among the inputs that exist.

**Simplify as we go** is a rule for this track from here on: one editor per thing (the monitor edits a preset, this editor edits a layout, the look card edits a look), a new one is the same editor on a blank copy, and anything shown in a list or a box can be dragged into another order (presets, sections, roles, screens within a role, boxes on a layout). An earlier draft's separate "+ From where I am now" entry and the Following latch are both gone under this rule.

A layout holds bookmarks and never the GM's knowledge: on a player's screen each panel is drawn from what that role knows, without the GM's pencil marks.

**Looks are tuned here too.** "This is the ONLY place we do all this filter adding." So the look card the Looks note designed (four rows: Filters, Palette, Signal fault, Screen delivery, with Holding cards as a fifth) opens from a role's look in the Switchboard, with the monitor as its preview on any bookmark. Workshop keeps a "Looks" entry that opens the same card here, so nothing is lost and nothing is in two places (question 19). Choosing a saved look for your own view stays in the View style switch, as ruled, because that is using a look, not making one.

The full screen composer in Workshop › Player screens stays for fine work on a layout's document; the builder here is the front door.

### 7.5 The pinned preview

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
| Workshop › Looks, the look card (designed, not built) | example packs 4D | opens in the Switchboard from a role's look; Workshop keeps a link |

After this, `SeatsPanel.svelte` and the table half of the status line are deleted, not hidden.

**Not absorbed**, on purpose: Known to and the eye (knowledge is one control, and it lives on the thing known); the table ledger; the party line on the players' own header.

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
// A bookmark: made anywhere, by the Bookmarks button. Nothing about players. (The top-level design's place mark.)
{ "schema": "sse.bookmark/1", "id": "b2", "name": "Tarn IV in infrared", "scope": "u:…/tarn/iv", "at_yr": null,
  "instrument": "sse.view.globe", "view": { "style": "true", "seen_from": "free", "scale": "readable", "band": "infrared", "hud": false, "pose": … },
  "selected": "u:…/tarn/iv/b" }

// A source: what a live send, a preset step and a line all carry. Extends today's Shown.
{ "show":  { "bookmark": "b2", "title": "Tarn IV close up", "layout": "players.survey.screen.json" } }   // layout optional
{ "land":  { /* LandingShow, unchanged */ } }
{ "say":   { "text": "The beacon repeats every ninety seconds…", "on_screens": false } }
{ "hold":  { "card": "blank" | "plain" | "joke" | { "text": "…" } } }   // replaces { off: true }
{ "play":  { "kind": "picture" | "sound" | "film", "asset": "pack:kestrel/maps/hollin-2" | "file:…" | "https://…", "look": true } }
{ "page":  { "address": "https://…" } }                                   // later: another program's players' page
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
{ "over": [ { "source": …, "state": "live" } ],                     // Everyone layers, top last; empty when none
  "lines": { "everyone": { "source": …, "state": "live" },             // the floor
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
- **Players never hand-pilot.** A screen that is Theirs has view controls only (section 5.3). Nothing here flies a ship.
- **Danger is drama.** Nothing is refused: a preset for a role with no screen, a band a role lacks and a hold on a line mid-landing are all allowed and all say what will happen.
- **One mechanism.** Sources, destinations and lines. Screens off, per-seat send, seat rows, the preview, the dial and OBS out are each a case of it.
- **Sealed roles** (competitive play) work the same from the Switchboard. The builder notes a per-seat hold is not yet honoured for sealed seats; PB1 fixes that as part of moving to lines.

## 13. Phone, light and dark, motion

- **Phone, as the GM.** No lines are drawn. The screens are a list, each row a preview with its role and "showing …" in words. The open page is a strip of presets along the bottom, in thumb reach; press one to fire it where it was written to go, press and hold to choose where. Press a screen's row for its sheet: Hold, the dial at full width, Send live here, Move to. Hold all stays at the top.
- **Light chart room and Night chart** both, from the tokens. Previews are always dark glass, since they are pictures of screens.
- **Motion** follows `interface-style.md`. A line draws itself from source to destination in about 200 ms when a send is made, which is the one flourish and it explains the change. A hold snaps the line apart. The role card and the dial grow from what opened them. Reduced motion makes all of it instant.
- **Cost.** One preview is one scene build, and screens in one role with one source share it, so the cost follows roles, not screens. Previews are drawn in Chart style and only when something changes. The pinned or selected preview may draw True (the builder's recommendation).

## 14. Questions, all answered yes

1. **Sources, destinations and lines as the one mechanism**, with "Screens off" becoming a hold with a blank card and per-seat send becoming a line? Recommended: yes.
2. **Say "role" and "screen" on the GM's side, and drop "seat" from the interface** (the code keeps it)? Recommended: yes. A seat already behaves as a role, and "screen" is the word you used.
3. **The Switchboard as a rail entry in a new last rail group, "Table"**, on every desk and at every scale, plus the on-air chip in the top bar and one Send button on the chart? Recommended: yes. The alternative is to reach it only from the chip, which hides it before a session is shared, when presets are being prepared.
4. **One layout per role**, so today's seat with two screens becomes two roles sharing one audience ("Bridge" and "Telescope", both the Kestrel's crew)? Recommended: yes. It is what makes "fire at Telescope" mean one thing.
5. **A preset row is two parts: the body previews it in the GM's monitor, the fire button sends it**, with no confirmation? Recommended: yes. One press to send, and browsing the hand is safe.
6. **Following: a line from Live may be latched to follow the GM**, off by default, shown in accent on the line and named on the chip? Recommended: yes, off by default. Without it a GM narrating a fly-through must press Send at every stop.
7. **Going back in three sizes** (your message of 15:58, taken as yes): a fired preset's button un-presses, Everyone is a layer with Lift, and Undo and Redo sit at the top; none of them takes back a Reveal? Recommended: yes.
8. **A role with no screen tonight can be dropped onto another role** as tonight's redirect, never saved in the Playbook? Recommended: yes.
9. **Read aloud stays on the GM's monitor unless its "also show on screens" switch is on?** Recommended: yes.
10. **Presets never move the universe clock in 4.0.** A Show step may say "as it was at this date" for the picture only. Recommended: yes. Moving the clock changes the world for everyone and should stay a deliberate act on the ruler; a preset that does it waits for 4.2 with triggers and timers.
11. **Holding cards as pack data with the first 24 above, editable in Workshop › Looks as a fifth row, and a look may carry its own set?** Recommended: yes. Strike any joke you do not like.
12. **Delete the Seats dialog and the table half of the status line once PB7 lands**, with no old path kept beside the new one? Recommended: yes. Keeping both is how small interfaces come back.
13. **OBS out in 4.0 as an ordinary screen on an ordinary role, with the transparent background option; camera in marked later; NDI parked?** Recommended: yes. This is the agreed position with the order made explicit.
14. **Build order: PB1 to PB3 first** (lines, the Switchboard with send and hold, the Playbook with fire and keep), because those three are what the run-through needs? Recommended: yes.
15. **Bookmarks at source, gloss here** (sections 4A and 7.4, your message of 16:11, taken as yes): one Bookmarks button beside Undo on every desk; presets and layouts are made from bookmarks in the Switchboard; no "send to" or "keep" entries anywhere else? Recommended: yes. It is the top-level design's place mark, so tours, presets and layouts share one saved thing.
16. **A GM's own picture or short sound may be sent to remote screens as a file, once, up to about 5 MB**; films only on this computer, by web address or in a pack? Recommended: yes. It is a file and not a stream, so I read it as inside "the GM streams only data", but it is your rule to read.
17. **Sound plays on the GM's computer by default**, and on a role's screens only where that screen's owner has turned sound on? Recommended: yes.
18. **Pictures and sound in 4.0 as one late package (PB12); film, web-page sources and the tabletop wait for 4.2 with Mappadux?** Recommended: yes. A map on the table is useful to the run-through; the rest is RPG release work.
19. **The look card opens in the Switchboard, not as its own Workshop screen** (Workshop keeps a link that opens it here)? Recommended: yes. It follows from "the only place we do all this filter adding"; it changes where the accepted Workshop › Looks card lives, not what is on it.
20. **Keep the one Send button on the chart on the Run desk?** It adds no gloss, it only sends where you are to Everyone, and it is the commonest live act. Recommended: yes, as the single exception to "all in one place". If you would rather have none, the on-air chip opens the Switchboard in one press and Send is the first button there.
21. **"Dynamic fog" means the knowledge filter at the screen** (section 4A): what a role knows and which bands its sensors have decide how much of a bookmark its screens draw, live, and a reveal lifts it without a re-send? Recommended: yes. If you also meant a visible fog (unknown parts of a map or system drawn as mist rather than left out), that is a drawing style for the same filter and I would add it as a choice on the look.
22. **The interference dial always present on every role and screen box; the kind of fault (analogue, digital, both) set in the look and not changeable live** (section 7.1, your messages of 16:15 and 16:17, taken as yes)? Recommended: yes. It withdraws the Looks note's rule that the dial shows only when the look has a fault saved.

**Answers.** Alex, 24 Nov 16:34: "Agree with all your recs above." All 22 are yes as recommended. On 21, "dynamic fog" is the knowledge filter at the screen, and reveals show on every view at once (section 5.3). Question 6 (Following) is answered by section 5.3: a read-only screen that the GM drives.

## 15. Packages (for "Build Run desk playbook and players' side")

Files in `packages/shell/src/run/` and `packages/ui-contract/src/`. One at a time, checks passing before the next. Screenshots to `/mnt/project-files/sse4/handover-screens/<PKG>-screens/`.

| Pkg | What | Needs | For the run-through? |
|---|---|---|---|
| **PB1** | Contract: `Source` (show with view, land, hold; `{off}` read as a blank hold), lines for Everyone (as layers over the set-up), role and screen with `before`, the board's undo history, the screen hello (id, name, wanted role) and its "showing" report. Sealed roles honour a hold. Old `shown` and `Seat.shown` read and upgraded. | nothing | yes |
| **PB2** | The Switchboard screen: Live source, roles and screens as a list with lines, Send, click to hold, Hold all that remembers, Lift, Undo and Redo, drag and press-then-press, move and swap screens. The on-air chip and the chart's Send button. Rail entry. The status line's table half goes. | PB1 | yes |
| **PB2b** | Bookmarks: `sse.bookmark/1`; the Bookmarks button beside Undo on every desk; go back to one; kept in the campaign (on the device outside one). Shared with tours. | nothing; chart room owns the top bar | yes |
| **PB3** | The Playbook: `sse.playbook/1` in the campaign; chapter and page picker; folding sections; fire, and press again to go back; keys 1 to 9; Keep on Live and "+ Add" with its kinds, "A bookmark" first; the "just now" strip with Keep and Again; Edit on the monitor for the selected preset (from the Playbook or the "just now" strip); reorder; Update from live; Reveal steps with the eye. | PB2 | yes |
| **PB3b** | Who steers: each screen Theirs or Read only (role heading sets all its screens); a screen that is Theirs keeps its layout's controls and cannot leave what was sent; press a read-only preview to drive it from the GM's stage, drawn through that role's knowledge; reveals redraw every screen of the role where it stands. Built to be tried and changed. | PB2 | yes |
| **PB4** | Previews: a Chart preview per role and source, shared; the confirmed tick; Pin to the chart; See full size. | PB2 | helps |
| **PB5** | Holding cards: `sse.holding-cards/1` in the base pack, the 24 lines, kinds, chosen on the line card, no repeat twice running. | PB2 | small, yes |
| **PB6** | Read aloud: the `say` source, the GM monitor reading card, "also show on screens" as a player document. | PB3 | yes if the run-through has narration |
| **PB7** | The role card: the Seats dialog's fields moved in; one layout per role with the upgrade of two-screen seats; redirects; requests on the screen that asked. `SeatsPanel.svelte` deleted. | PB2 | no |
| **PB8** | Clean output link: `?clean=1`, fixed 16:9, transparent option, marked "Output" on the board. | PB2; renderer for the transparent ground | no, but small |
| **PB9** | Phone layout; "Open in its own window"; "On this computer": the attached displays listed, a full-screen window on the chosen one, and Split (Whole, Two, Three, Four) making each part a screen of its own. | PB2 | no |
| **PB10** | Looks on the board: the look card opening here with a bookmark as its preview; look per role, the dials on role headings and screen boxes always present, the kind of fault read from the look, deliveries on arrival and hold; the transmission card as part of a look. | renderer's looks (filters, faults, deliveries) | no |
| **PB11** | Later: camera in, on local screens only. | PB9 | no |
| **PB12** | Play: pictures and sound as a source, from a pack, a file or a web address; "Here" as a sound destination; one asset on a Read aloud; the "Incoming transmission…" card while a sound plays on screens. | PB6 | no, but wanted |
| **PB13** | Layouts on the board: the layout list on a role; one layout editor for new and existing, with preview; each box a bookmark, a file or a link, or "Whatever is sent"; boxes reordered by dragging; a layout named by a preset. Orac's line when the agents track is ready. | PB2b, PB7; the existing screen composer | no |
| **PB14** | Later (4.2): film; a web page as a source; the tabletop with Mappadux. | PB12 | no |

Asks of other threads: the renderer for the cheap Chart preview at thumbnail size, the transparent ground and the looks pieces already assigned to it; the secrets work (SV) for the eye on Reveal steps; the chart room for the Bookmarks button in the top bar.

## 16. For Mappadux, when it is rethought

Alex, 24 Nov 16:04 to 16:05: "This routing system is going back into Mappadux as that is rethought. The ultimate space VTT... jot down notes for this - I will get to Mappadux again one day and we need to carry these over." Also confirmed in the same message: assets are loaded here, can be included in packs, and "the GM can throw them up where they want".

Mappadux lives at `C:\Development\dynamic-map-renderer-v2` on Alex's machine and at https://github.com/FrunkQ/dynamic-map-renderer-v2 (Alex, 16:05). Its code was not read for this note.

What to carry over, written so it stands without the rest of this note:

1. **Sources, destinations, lines.** Everything the GM can show is a source; everything that can show it is a destination (a screen, a role, Everyone); sending draws a line. One mechanism covers send, per-player send, screens off and previews.
2. **Roles in the prepared material, screens at the table.** Prepared scenes name a role. Which device is which role is set on the night by dragging, and is never saved with the scenes.
3. **A preset is a saved send**, so doing it live and firing it prepared are one path, and "keep what I just did" costs nothing.
4. **Nothing sent destroys what it replaced.** Everyone is a layer that lifts; a fired preset's button un-presses; Hold all remembers what was already held; Undo and Redo for the board.
5. **Holding is a send, not a cover**, so a held screen has nothing behind its card. Holding cards are editable data with jokes.
6. **Knowledge belongs to the role a screen is in**, never to the thing sent. Routing can then never leak a secret.
7. **The GM sends data, and each screen draws its own picture.** Previews are drawn on the GM's machine and say they are what the screen should be showing.
8. **Assets** (pictures, sound, film) come from a pack by name, from the GM's own files, or from a web address, and are sources like any other.
9. **Another program is a source by its web address.** This is the seam between SSE4 and Mappadux in both directions: a Mappadux table as a source on an SSE4 screen, and an SSE4 role's clean output as a source on a Mappadux table.
10. **A clean output link per role** gives streaming for free.
11. **Bookmarks at source, gloss in one place.** The only thing made out in the app is a bookmark (where you are, how you are looking). Who sees it, its look, its arrangement and its routing are added in the one routing screen and nowhere else.
12. **Each screen is either the player's to steer, inside what was sent, or read only and driven by the GM.** Reveals are a change to what a role knows, so they reach every screen at once without a send.
13. **Controls placed by how often they are used**: the dial on the box it changes, Hold all always in one corner, rare things on a card that opens from what it describes.

Open for that day, not designed here: what the two programs tell each other (which map, where the party is, whose turn), and whether they share one Switchboard or each have their own.

## 17. Not in 4.0

Kept out so the run-through stays small; most belong to 4.2, the RPG release.

- Triggers, timers and conditions that fire presets by themselves; presets that move the clock.
- A co-GM or Minerva drafting pages from the campaign's record.
- Sound and music cues as steps.
- Dice, checks or any game rule on a preset.
- Players asking for a picture ("put the scope on that moon") as a request line into the Switchboard.
- Picture-in-picture and split screens on one destination.
- Film, web-page sources and showing a virtual tabletop (section 4.8).
- NDI.
