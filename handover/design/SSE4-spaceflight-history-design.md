# SSE4: the history of spaceflight as a view, and those who died (track SH)

Status: **PROPOSED, 25 Nov 2026. Not accepted. Nothing here is built and no builder starts on it without Alex's yes** (questions in section 14). Design and mockup only. SH is a code only and is never shown in the product.

One part is already Alex's direction (25 Nov 01:04 UTC, "great idea - show the memorials that have been placed in space - very clever"): the memorial shows the real memorials where they really are. Section 8 is built on that.

Mockup, clickable: project files `sse4/handover-screens/SH-screens/spaceflight-history-mockup.html`, with desktop, night and phone screenshots beside it and its source in `SH-screens/source/`. Every name, date and figure in the mockup was typed by hand for the picture. The product types none of them: it reads them from the pack (section 10).

Alex's brief, in his words (25 Nov 01:00 and 01:03, in "Build real spaceflight history pack"): "does this mean you can go to any day in history and a crew roster of every who was in space on that day?" and "it makes this a cool history tool TBH! The whole history of spaceflight as a 'fun aside' - when it could be a 'thing by itself'. we can maybe build a cool custom UI around this and find a way to honour those who died trying... something respectful - names etched on a black monolith rotating in jupiter's orbit... or something better".

Read with: `SSE4-real-spaceflight-feasibility.md` (track RS, the data), `handover/status/spaceflight.md` and `handover/status/manifest.md` (people aboard, the Log), `SSE4-top-level-ui-and-onboarding.md` (rail 2.2, tours 3.2, the one clock 12.2), `SSE4-voyages-design.md` (flight logs), `SSE4-secrets-and-visibility-design.md` (Known to), `SSE4-postcards-design.md` (track PC, proposed), `handover/interface-style.md`.

## 1. The short answer

1. **Roll call is one new instrument.** Pick any date. It says who was in space that day, aboard what, and where each craft really was, drawn on the ordinary charts. It is a look-up over two things that already exist: flight logs and the manifest's "aboard" links. Nothing is simulated to answer it.
2. **It is general.** It reads whatever campaign is open. With the "Sol today" pack stacked it is the real history of spaceflight. In a GM's own setting it is that setting's fleet history, for free, with no extra work. There is no "real history mode".
3. **The count runs along the ruler.** A thin strip above the time ruler shows how many people were in space through time. The shape of sixty years is readable at a glance: the first single day in 1961, the gaps, and the unbroken line since late 2000.
4. **Records are questions, not typed facts.** "Most people in space at once", "last day with nobody up", "farthest anyone has been" are saved sifters over the logs, each showing how it was worked out. A clever user can test every one.
5. **It gets its own front door.** One address opens SSE4 straight into Roll call on today's date with the pack loaded, so the history tool can be linked and used by itself.
6. **Those who died are honoured three ways, none invented.** Their names stay in the roll call of the days they flew. One still page, **In memory**, lists every name in date order. And **Where their names are** shows the memorials that really exist in the Solar System, where they really are, on any date: the figure and plaque at Hadley, craters, hills and landing stations, and the asteroids that carry their names round the Sun tonight.
7. **The monolith's idea is kept and its object is not.** Alex's picture was names in orbit, far out, permanent. That is already true: seven asteroids carry the Challenger crew's names and seven the Columbia crew's, between Mars and Jupiter. SSE4 shows the real ones (section 9).
8. **No new mechanism.** It reuses the flight log, the manifest person, the one clock and its ruler, the rail, tours, sifters, Known to, the system chart and the Sky view.

## 2. Words

| Word | Means |
|---|---|
| **Roll call** | The instrument: who and what is in flight on the date in view. Proposed name (question 1) |
| **In flight** | A construct whose log puts it off the ground and not yet gone on that date |
| **In space** | In flight above the pack's "edge of space" (section 4.4) |
| **The count** | The strip above the ruler: people in space through time |
| **In memory** | The page of names of those who died (question 1) |
| **Where their names are** | The chart layer and tour of real memorials |
| **Carried name** | A real thing that bears a person's name: an object left, a named feature, a named small body |

## 3. What it reads

Three things, all present or already being built. This section is the whole mechanism.

| It needs | It reads | State |
|---|---|---|
| Where every craft is at time t | The flight log: find the piece covering t and evaluate it (`record.flight`, op `flight_log`) | Built for 1,232 real logs (RS3, RS8); the campaign side is the travel track's RS4 |
| Who is aboard what at time t | The manifest's `aboard.on` link with its dated history (MC1, MC5) | Built; real people arrive with RS9 |
| What is true of a person | The manifest person: kind of being, role, state (awake, cold sleep, missing, dead) | Built |

So Roll call for a date is: every construct in flight at t, and for each, everyone whose `aboard.on` points at it at t. Each row keeps the log's own `mark` (recorded or estimated) and `src`, so an estimated row is drawn in pencil and says why, exactly as the charts already do.

**Because it reads only these, it works in any campaign.** A Traveller GM who has run forty sessions can ask who was aboard what on day 112. A setting with its own lost ships has its own In memory page, with the GM-only "authored" honesty mark, and Known to decides what players see of it. The real pack is just the largest campaign anyone has loaded.

## 4. Roll call

Mockup screens 1 and 2.

### 4.1 The page

The stage keeps the ordinary chart. Roll call is a panel beside it (below it on a phone) and a strip on the ruler.

- **The date** is the big serif title: "20 July 1969". It is the clock's date, so dragging the ruler, typing a date, or pressing any dated thing anywhere changes it. Press the date to type one. Days are UTC, and the panel says so once, quietly.
- **One sentence under it**, computed: "3 people in space, aboard 2 craft. 9 more craft in flight with nobody aboard."
- **The roster, grouped by where.** Groups are places, nearest first: Around Earth, At the Moon, On the way to Mars, Around the Sun, Beyond the planets. Under each place its craft, and under each craft its people.

  ```
  AT THE MOON
    Columbia        in orbit, 111 km up
      Michael Collins          command module pilot     day 5 of 8
    Eagle           on the surface, Tranquility Base, since 20:17
      Neil Armstrong           commander                day 5 of 8
      Edwin "Buzz" Aldrin      lunar module pilot       day 5 of 8
  ```

- **A person row** shows the name as credited, role, and the day of their flight. No flags and no portraits: a name and what they were doing. Pressing it opens the person (section 6).
- **A craft row** opens the ordinary Ships and stations card with its Log, plans and manifest. Hovering a row inks that craft on the chart, and hovering a craft on the chart inks its row. Pressing "show" flies the chart to it.
- **"Also in flight"** lists craft with nobody aboard, folded to a count per place ("On the way to Mars: 2"). Earth-orbit satellites without logs are one line with a count from the catalogue, never drawn one by one (RS ruling, "wide and thin").
- **Kinds of being are never assumed human** (MC ruling). On 3 November 1957 the roll call reads "1 in space: Laika, dog, aboard Sputnik 2". That is the manifest's existing field doing its job (question 6).
- **An empty day says so in words**: "Nobody was in space on this day. The last person up was … who landed … The next was …", both names links. Before the first flight: "Nobody had been to space yet. … days to go."

### 4.2 The chart

Nothing new. The system chart, the Earth and Moon close-ups and the body globes already draw constructs from their logs at the date in view. Roll call only adds the linked highlight and a "people aboard" dot count on a craft's label when its HUD is on. View style, Seen from, Scale, HUD and Band are the shared controls. The place line picks the scale, so "Around Earth" in the roster is a link to Earth's own chart at that date.

### 4.3 Moving through time

- **‹ Event and Event ›** step the clock to the previous or next dated event in any log in view: launch, docking, landing, a split, an entry. Hold to keep stepping. This is SSE3's flight-log rewind made general.
- **‹ Day and Day ›** step one day.
- **Play** runs the clock at a chosen rate, as everywhere. In the real pack the campaign's now is today, so all of this is "Looking" in the one-clock model (top-level design 12.2): players' screens do not move, nothing is recorded.
- **"On this day"** opens a short list for the calendar day in view across every year: who was up on 25 November in each year, and what launched or landed on it. One press goes to that year.

### 4.4 The edge of space is a number

Whether a short hop counts as "in space" depends on where space starts. The pack carries one value, the edge of space, with its source: 100 km by default (the Kármán line, used by the FAI), with the 80 km line (50 miles, used by the United States) as the other common choice. A flight that crossed 80 km but not 100 km is listed with the plain words "by the 50-mile line". Change the value on the Build desk and the count, the records and the roster all follow. This is the SSE4 habit of making an argument into an editable, cited number rather than settling it quietly.

### 4.5 The count along the ruler

A strip one line high above the ruler: a stepped line of people in space through time, drawn for the span the ruler shows. In the real pack the ruler in Roll call opens on 1957 to today.

- It comes from a small precomputed step list in the pack (section 10, item 5), so drawing sixty years costs nothing and no logs are loaded for it.
- Hovering reads a date and a count; pressing goes there.
- Days someone died are marked on it with a small hollow mark in ink colour. Not red, no symbol. Pressing one goes to that day (section 8.4).
- The accent colour stays what it always is: the campaign's now.

### 4.6 Records and firsts

A "Records" fold at the foot of the panel. Each line is a saved sifter (the `sift` mechanism behind Find and Stories) with a why:

| Record | How it is worked out |
|---|---|
| Most people in space at once | The highest step in the count, with its dates |
| Last day with nobody in space | The last zero in the count |
| Longest single stay; most time in total | Longest `aboard` span off the ground; the sum per person |
| Farthest anyone has been from Earth | The greatest Earth distance of any crewed log piece, from the kernel's positions |
| People who have left Earth orbit | People aboard a craft whose log leaves Earth's sphere of influence |
| First of each kind | Earliest event matching a sifter (first docking, first landing on another body) |

None of these is typed into the pack, and the test suite never pins a "right answer" from a book. If the logs improve, the records move by themselves. Where a record rests on an estimated piece it says so. Packs and users add their own lines; Orac can write one from a sentence.

### 4.7 Tours

Tours are saved places with captions (top-level design 3.2), so the pack ships a few as plain data: "The first day" (one person, one orbit), "Eight days to the Moon", "The handshake" (two crews from two countries docked), "Never empty since" (the count from 2000), and "Where their names are" (section 8.3). A tour is how a newcomer meets the tool; nothing is blocked while one plays.

## 5. Where it lives

- **Rail.** Roll call is a screen document with a `rail` entry at `system` and `world` scale, group `travel`, label "Roll call", `needs` nothing: it is listed whenever the campaign has any flight log, and stays listed, dimmed, with the reason ("No flights are recorded here yet") when it has none. In memory is reached from Roll call and from a person, not from the rail, so the rail gains one entry (limit of ten per scale).
- **Desks.** Explore: read only. Run: the same panel reads the campaign clock, and "Show the roll call to the table" is one Switchboard source, filtered by Known to. Build: people and moves are edited where they already are, on the Ships and stations card.
- **Depth.** Curious shows the roster and the sentence. Science adds the mark and source on each row, orbit figures and the Records' workings. Engine adds addresses and the log pieces.
- **Find** knows people: typing a name offers the person; typing a date offers Roll call on that day.
- **Players.** A player sees the roll call of what their role knows. In the real pack everything is public, so it is simply shown.

## 6. A person

The manifest person card, with one added part when the person has any flight: **their flights as spans on a small ruler**, one line per flight, pressed to go to its launch day. Above it one computed sentence: "Three flights, 21 days 15 hours in space." For people in the real pack the card holds flight facts only: name, kind of being, who they flew for, flights, and for those who have died, the year of birth and of death. No biography, no portrait, nothing private (question 9). A person who has a carried name lists it: "His name is on: the plaque at Hadley; Komarov crater; asteroid 1836 Komarov", each a link that flies there.

## 7. Its own front door

"A thing by itself" needs an address that is about it and nothing else.

- **One address** (proposed `v4.starsystemx.com/spaceflight`) opens the app with Sol, fetches "Sol today" from Explorers, and lands in Roll call on today's date with the rail folded and the panel wide. One "Open the whole chart room" link leads to the rest. If Explorers cannot be reached it says so and offers the Solar System without the pack.
- **Every state is a link.** A date, a person and a carried name each have an address (a bookmark in the address bar), so "who was up the day I was born" can be sent to a friend. The pack stays on Explorers, not built in (Alex, 23 Nov): the door loads it, it does not bundle it.
- **The look is the chart room's.** "Custom UI" here means the view is designed for this job, not a new skin. A role or a kit (Atlas track, `sse.kit/1`) can restyle it later like any other view; a mission-control green screen of Roll call would be a kit, and needs no change to this design.
- **The start sheet** gains one card when the pack is stacked: "The history of spaceflight: who was up, any day since 1957."

## 8. Those who died

Mockup screens 3 and 4. Real people and real deaths: this section's rules are firmer than the rest of the note.

### 8.1 Rules

1. **Every name, date and place is sourced, twice.** A death enters the pack only with two independent sources, and the entry shows them. A name we cannot source is not guessed; the event says "and others whose names we have not been able to confirm".
2. **Nothing is invented.** No monument of our own in the real pack, no imagined last words, no reconstruction.
3. **Loss is never a spectacle.** No explosion, no breakup, no debris, no sound, no replay control. A lost craft's track ends where its record ends.
4. **Plain words.** One sentence of fact per event, in the register of an official record: what happened, when, where. No adjectives, no "tragically".
5. **People before deaths.** A person is shown first by what they flew. Their death is one line among their flights, not the headline.
6. **No decoration.** In memory has no pictures, no accent colour, no motion beyond a plain fade, in both chart room and Night chart. It is never a hold ground, a postcard backdrop, a joke target or an edition's margin note. The Guide edition says nothing on these pages.
7. **Corrections are welcome and easy.** A "Something wrong or missing?" link on the page opens a prepared report. A correction is a pack data change with its source.

### 8.2 Who is named

"Those who died trying" is wider than those who died in flight. The page has three parts, each plainly headed, and a person's entry says which (the pack's `during` field):

| Part | Who | Examples, to be confirmed from sources by the builder |
|---|---|---|
| **On a flight** | Died during a flight to or from space, or a flight test of a craft built for it | Soyuz 1, Soyuz 11, Challenger, Columbia; the X-15 and SpaceShipTwo test flights |
| **Preparing to fly** | Crew who died in training or a ground test | Apollo 1; the chamber fire of 1961; the training jet losses of the 1960s |
| **Working to send others** | Ground crews killed in launch site accidents | Baikonur 1960, Plesetsk 1980, Alcântara 2003, pad workers at Kennedy in 1981 |

Not included: people who flew and died later of something else. Recommended: all three parts in the first version (question 4), because the third is the one usually left out and "died trying" covers them. Where an accident's toll is disputed, the entry gives the sourced range and the names that are sourced.

Animals that died on flights (Laika and others) are in the roll call of their days as a matter of record. Whether they have a part on In memory is question 6; recommended: a short separate part at the foot, plainly headed, facts only.

### 8.3 In memory, the page

- One quiet column. A computed first line: "*N* people have died flying to space, preparing to, or working to send others. These are their names."
- Names in date order under each event's date and one sentence. Serif, generous spacing, ink on the ground colour.
- A name opens the person: their flights first (section 6), then where their name is carried.
- An event's date goes to that day in Roll call.
- The page is not filtered by the clock. It is the same whatever date is in view.
- At the foot: sources for every entry, the correction link, and "Where their names are ›".

### 8.4 The day itself

On the day of a loss the roll call lists the crew like any other day's, with their craft, and under the craft one plain line with the time: "Lost 73 seconds after launch, 16:39 UTC." and a link "In memory". The craft's track on the chart ends at its last recorded piece with the same small hollow mark the count uses. Play does not stop by itself (nothing ever blocks), but Event › stops on it like any event. The next day's roll call simply no longer has them, and the empty-day sentence, if it applies, is left as it is.

### 8.5 Where their names are

The memorials are already out there, and each is an ordinary dated thing SSE4 can already hold. **This is the part Alex has endorsed.**

| Kind | In SSE4 it is | Examples, each to be confirmed from its source before it enters the pack |
|---|---|---|
| **An object carried and left** | Manifest cargo that came aboard a craft and left it at a dated place, so the craft's own Log shows it going | The Fallen Astronaut figure and its plaque of 14 names, left at Hadley by Apollo 15 in 1971. Also the mementoes Apollo 11 left for Apollo 1's crew and for Gagarin and Komarov |
| **A named feature** | A name on a site of a body, with the date it was approved | Craters on the Moon's far side named for the Apollo 1, Soyuz 11, Challenger and Columbia crews; the Columbia Hills and Columbia Memorial Station at Spirit's landing site; Challenger Memorial Station at Opportunity's |
| **A named small body** | A small body on its real orbit, named on a date | Asteroids 3350 to 3356 for the Challenger crew and 51823 to 51829 for the Columbia crew; asteroids for Gagarin, Komarov and the Soyuz 11 crew |
| **Remains carried** | Manifest cargo, same as the first row | Offered by the pack thread: Eugene Shoemaker's ashes on Lunar Prospector, Clyde Tombaugh's on New Horizons. These are people honoured in space, not people who died trying, so they are shown under their own heading |

How it is shown:

- **A layer, "Names carried",** on the system chart and the Moon and Mars globes. Off by default; on in the tour and when arrived at from In memory. Each carried name is a small ringed mark with the name in the serif face.
- **Each exists only from its date.** Scrub to 1970 and Hadley is bare; the figure arrives with Apollo 15 on the day it was set down. The asteroids are on the chart always, since the rock was there, but take their names on the day they were named. This is the one clock doing what it does.
- **"Tonight".** The asteroids are drawn where they really are on the date in view, from JPL's orbits, and the Sky view can point to them from where you stand: "3352 McAuliffe is in Taurus tonight, too faint for the eye." A small true thing a user can check with a planetarium.
- **At Hadley** the Survey and Site levels show the figure and the plaque as an object at its real coordinates, with the plaque's names as text beside it in the order the plaque gives them. It is drawn as a small plain marker, not a model of the sculpture, until Alex says otherwise (question 8).
- **The tour "Where their names are"** visits them in date order with one sentence each, and ends on tonight's sky.

Earth's own memorials (the Space Mirror at Kennedy, the wall at the Kremlin, Star City) are named on the In memory page as text with their sources. They become sites on the globe only when Earth has authored sites (question 10).

## 9. The monolith, answered straight

Alex offered "names etched on a black monolith rotating in Jupiter's orbit... or something better". What the picture gets right: the names are out among the planets, permanent, quiet, turning in the dark. What argues against building it:

- It would be the one invented object in a pack whose whole worth is that everything in it happened. It would need the "authored" mark, on a memorial.
- A black monolith at Jupiter is another story's image (*2001*), not spaceflight's own.
- Jupiter has no tie to these people. The Moon, Mars and the asteroid belt do, because that is where others chose to put their names.

And the better thing is the same picture made true: fourteen rocks between Mars and Jupiter already carry fourteen of the names, and SSE4 can show where they are tonight. Recommended: no invented monument in the real pack (question 5). If Alex still wants one of SSE4's own, the honest form is a separate, optional, plainly "authored" object outside the pack, and the note would place it at the Sun–Jupiter L4 point, where things really do stay.

## 10. What the pack must hold (seam with the spaceflight pack thread)

Agreed by message with "Build real spaceflight history pack" on 25 Nov; it is building RS9 from McDowell's astronaut tables (planet4589 `astro`, `rides`, `segments`, `evas`), with born and died dates. Its files stay its own.

| # | Data | Shape | State (pack thread's answer, 25 Nov 01:09 UTC) |
|---|---|---|---|
| 1 | People | Manifest persons: id, name as credited, kind of being, who they flew for (text), sources (McDowell id, Wikidata id). Flight facts only for the living | Built in RS9 part 1: `store-packs/sol-today/crews.json` (id, name, short name, `being`, citizenship, `sources.mcdowell`), written by `scripts/spaceflight/build_crews.py`. Wikidata ids to come. McDowell lists primates only, so Laika and other animals need GCAT's payload notes |
| 2 | Moves | Dated `aboard.on` changes with `mark` and `src`. A Moon walker stays aboard the lander; the lander is on the surface | Built as legs (person, `from_s`, `to_s`, craft, `on` = the log's construct id, role, mark, src). Turning legs into `aboard.on` actor changes needs the manifest thread |
| 3 | Deaths | On the person: a dated fact with `where`, `during` (`flight`, `training`, `ground_test`, `ground_work`), one sentence, two sources. The instant in UTC and the date the record itself uses (a landing at night in Kazakhstan is two calendar days). People with no flight (training and ground deaths) are persons too | `died_in_flight` is set for those killed in flight. Training and ground deaths, the `during` field and second sources are RS9 part 2 |
| 4 | Loss on a log | An ordinary dated event and a `gone` state with `how: "lost"` beside today's `reentered` and `impacted`. No special case in the reader | RS9 part 2 |
| 5 | The count | A step list of (instant, people in space), computed by the build script from 2, checked by a test against the look-up | Built: `in_space`, a list of `[t, count]` steps, checked against the legs by a test |
| 6 | The edge of space | One pack value with its source (100 km), not baked into 2 or 5 beyond a recorded peak height per short flight | The file says `edge_of_space_km` 80 (McDowell's line) and each leg has a `suborbital` flag. Peak heights per short flight come in part 2, so a 100 km value can mark the 80 km flights. Which line is the default is question 11 |
| 7 | Carried names | `memorials.json`: kind (object left, named feature, named small body, remains carried), where (craft and site, or body and coordinates, or orbit elements from JPL), since-date, the people named, source (NASA mission report, USGS Gazetteer of Planetary Nomenclature, the Minor Planet Center's citation) | asked; the pack thread offered the cargo ones (Hadley, Shoemaker, Tombaugh, the Celestis flights) |
| 8 | Tours and record sifters | Plain data in the pack | this track writes them |

The pack thread's log for this is `handover/packages/RS9.md` (arrives with its PR). It found none of the asks awkward.

Sources of record, in order of trust for this track: mission reports and agency histories; McDowell's GCAT and astronaut tables (CC BY 4.0, credited); the USGS Gazetteer and the Minor Planet Center for names; Wikidata as a cross-check, never alone.

## 11. Honesty and fudges

- Flight positions inherit the RS fudge (big burns only, stable orbits between) and its measured size. A Roll call row built on an estimated piece is in pencil with its why.
- The count is exact to the moves it is given; its only choice is the edge of space, which is on the page.
- "Day" is the UTC day. A person who launched at 23:50 UTC is in that day's roll call.
- Nothing in Roll call, In memory or the Names carried layer feeds physics or the state hash.
- Fudge register entries arrive with the builder's PRs, not with this note.

## 12. Phone

One column: date, sentence, the count as a strip you drag, the roster. The chart is a fold above the roster, shut by default at phone width so the roster is the first thing read. In memory is already one column. Free view rules for the chart are the standing ones.

## 13. Packages, if accepted

| Package | What | Owner |
|---|---|---|
| SH1 | `rollCall(campaign, t)`: in-flight constructs and who is aboard, grouped by place, marks kept. Pure function in `ui-contract`, tested on the real pack and on a made-up campaign | chart room or a new thread "Build roll call and In memory" (Opus) |
| SH2 | The Roll call screen: panel, linked highlight, date typing, Day and Event steps, empty-day words, rail entry | same |
| SH3 | The count strip on the ruler, from the pack's step list; hollow marks | same, with the chart room (the ruler is theirs) |
| SH4 | The person card's flights; Find knows people and dates | same, with the manifest thread |
| SH5 | Records as sifters; On this day | same |
| SH6 | In memory, the page; the day of a loss; `how: "lost"` in the log reader | same, after data items 3 and 4 |
| SH7 | Names carried: the layer, dated appearance, Hadley at Survey and Site, the asteroids on their orbits, "tonight" in the Sky view | same, with the renderer and the sky owners; after data item 7 |
| SH8 | The front door address, deep links, start sheet card | chart room |
| SH9 | Tours and record sifters as pack data; the edge of space as a Build value | spaceflight pack thread and this track |
| RS9 and its follow-ups | Data items 1 to 7 | spaceflight pack thread |

SH1 and SH2 need only what is on main plus RS9. Nothing here changes a golden hash.

## 14. Questions for Alex, each with a recommendation

1. **Names: "Roll call" for the view, "In memory" for the page, "Names carried" for the layer.** Recommended: yes. (Alternatives: "Who was up"; "Remembered".)
2. **Roll call is general: it works in any campaign, and the real pack is one use of it.** Recommended: yes. This is the one-mechanism rule, and it gives every GM a fleet history.
3. **A front door of its own** at an address such as `/spaceflight`, loading the pack from Explorers, with every date and person linkable. Recommended: yes.
4. **In memory names three groups: died on a flight, died preparing to fly, died working to send others (ground crews).** Recommended: yes, all three, each plainly headed.
5. **No invented monument in the real pack; the real memorials carry it** (your 01:04 message already leans this way). Recommended: yes. Say if you still want an authored one as a separate optional object.
6. **Animals:** in the roll call of their days as fact (Laika aboard Sputnik 2), and a short separate part at the foot of In memory. Recommended: yes to the first; yes to the second, but it is your call on tone.
7. **The rules of restraint in 8.1 are standing rules for this track** (two sources, nothing invented, no spectacle, plain words, no decoration, never a backdrop or a joke). Recommended: yes.
8. **The Fallen Astronaut is drawn as a plain marker with the plaque's names as text, not a model of the sculpture.** The sculpture is a living artist's work (Paul Van Hoeydonck) and its image has a history of dispute. Recommended: yes, marker and text only.
9. **People in the real pack carry flight facts only:** no portraits, no biography, nothing private; year of birth and death only for those who have died. Recommended: yes.
10. **Earth's own memorials are text with sources for now,** sites on the globe later. Recommended: yes.
11. **The edge of space is one editable pack value, 100 km by default, with 80 km flights marked.** The pack's source (McDowell) uses 80 km, so the other honest choice is 80 km by default with the 100 km line as the option. Recommended: 100 km, the line most people know.
12. **Release:** Roll call, the count, In memory and Names carried are 4.0 work that follows RS9; kits that restyle it, and Earth memorial sites, are later. Recommended: yes.
13. **Who builds it:** a new Opus thread "Build roll call and In memory" owning `shell/src/rollcall/`, taking SH1 to SH8. Recommended: yes.

## 15. Not in this design

- The pack's data work itself (track RS).
- Spacewalks as moves, mission audio and transcripts, photographs: later, each needs its own rights check.
- A kit or edition that restyles Roll call (Atlas track).
- Any change to the kernel, the clock or the log format beyond `how: "lost"`.
