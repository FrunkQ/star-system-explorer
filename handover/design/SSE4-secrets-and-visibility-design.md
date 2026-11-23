# SSE4 secrets and visibility: one "Known to" control (track SV)

Status: **accepted in direction by Alex, 23 Nov 2026** ("just the small dot by the name was not intuitive but apart from that... this is a work of GENIUS!"), with one change he asked for: the mark is **a small eye**, not a dot, and it stays a symbol, not words ("WORDS too much... maybe just a small eye"). The ten questions in section 12 are taken as yes with that change unless he says otherwise.
Mockup: `/mnt/project-files/sse4/SV-screens/known-to-mockup.html` (artifact "SSE4 Known to control", https://claude.ai/artifact/DiBSjywKnQGqcSRzWAHCeC). Screenshots beside it in `SV-screens/`.

Asked for by Alex, 23 Nov: "Secrets we generally kept with a small visibility icon next to a piece of information in the previous version. We probably need to improve that, make it a bit more intuitive and feature it in far more places. Subtle yet not overwhelming." It came up over secret links between stars (space highways), and it replaces the plain eye proposed for V3 gap 21.

This note adds no second system. Everything underneath is already designed and mostly built: disclosure tiers and player data views (handoff 6.7, `sse-observe`), audiences and seats (handoff 8.7), the record's audiences and need-to-know wire rule (record design 2.3 and 8A), "See as" lenses, the Scanned button, and "According to" on territory maps. What is missing is the surface: one control, one mark and one set of words, used the same way everywhere.

## 1. The idea in five lines

1. **One question.** Every thing and every fact can be asked "who knows this?", and the answer is always shown and changed the same way. The control is called **Known to**.
2. **Pencil and ink.** On the GM's chart, what the table knows is drawn in ink and what only the GM knows is drawn in pencil (thin, dotted, quieter). Revealing something inks it in. This is the chart room's own language: an edit is already "inked onto the map".
3. **One small mark.** A small eye: closed for GM only, half open for partly known, open for known to the table. It sits in the same place on everything and it is also the button. (First drawn as a dot; Alex found the dot by the name unclear and chose the eye, 23 Nov.)
4. **One press to reveal.** Press the eye and the table knows. Everything rarer (one crew only, heard of but no details, a cover story, a date) is in the popup that opens from the eye.
5. **Nothing leaks by absence.** A player's screen never shows a gap, a count or a blank line where a secret is. This is the territory design's "bare paper" rule, made general.

Why this is better than V3's eye, while keeping the eye people already know: V3 put an eye on every line, it only said yes or no, it could not say "to whom", and it was not visible on the map at all. Here pencil and ink is seen at a glance over a whole chart without adding a single icon, the eye has three states and opens onto "to whom", and it only appears where a decision has been made: one by the name, and one on a line with an exception.

## 2. What already exists, and what each part becomes

| Exists | Where | In this design |
|---|---|---|
| Disclosure tiers, data views, an override per address inherited by children | handoff 6.7, `sse-observe` `effective_view`, `grant` | The rungs a thing can stand on for an audience. Unchanged. |
| The Scanned button (flip a body between the campaign's "before" and "after" views) | handoff 6.7, `scan_toggle` | Pressing the eye **is** this action, on anything. The separate button goes. |
| `observe.grant` events naming an audience | chronicle | Every reveal and every hide is one of these, dated, so "when did they learn it" stays history. |
| Audiences and seats ("everyone", a crew, an empire by tag) | handoff 8.7 | The rows of the Known to popup. With one audience there are no rows, only the eye. |
| Notes with an audience list; kept values with a tier; `misc` fields secret unless stated | record 2.1, 2.3 | The same eye on a note, a stock, an owner, a holding. |
| Need-to-know on the wire: a hidden thing is not on the player's device | record 8A | The guarantee behind every eye. Unchanged, and tested on the wire. |
| "See as: the table" lens; "not known to the table", "long range", "rumour" under names on the Run desk | handoff 8.6, U14 | Kept. Pencil and ink carries the same information without the words; the words move to hover and to the selected body. |
| "According to: as it is, as a power draws it, as the players know it" | territory design 8 | Unchanged. Holdings use the same eye. "As a power draws it" is the model for cover stories (section 6). |
| Honesty marks (authored, impossible, fictional) | handoff 8.6 | A different question (is this real physics?) with a different mark (a stamp). The two never share a symbol. |
| Remote survey and sensor rules that move a body up the ladder | handoff section 11, record 5 | Unchanged. They write the same grants a press does. |

## 3. The rungs

For one audience, a thing stands on one rung of a short ladder. The ladder is the pack's own (handoff 6.7), so a body under the Traveller pack might have Hidden, Rumour, Long range, Scanned, Surveyed. Every ladder has the same three landmarks, and those are what the eye shows:

| Eye | Name | Meaning | On the player's screen |
|---|---|---|---|
| closed | **GM only** | not known to exist | nothing at all, and nothing where it would be |
| half open | **Heard of** | known to exist, with a name and a place, details held back | the name, a position, and the lines the rung allows |
| open | **Known** | everything the audience's data view shows at its top rung | as today |

Things without physics to survey (a link, a note, a holding, a map layer) have just these three. A thing with a survey ladder shows its rung name beside the eye in the popup ("Long range") and the eye is half open until the top rung.

A **fact** on a thing (one card line) follows its thing's rung by default: the pack already says population needs Surveyed and colour needs only Long range. A line gets an eye of its own only when the GM makes an exception: show this one fact early, or keep this one fact back. So a card normally carries one eye, at its title, and a few more only where the GM has chosen something.

## 4. The mark, and where it sits

- **On a card:** one small eye at the left of the title. Lines the table cannot see are in pencil: quieter ink and a dotted rule under the value. Hover or focus a line and its eye appears in the left margin; press it to make an exception. Lines with an exception keep their eye showing.
- **On a chart or map:** a thing the table does not know is drawn in pencil: a thin dotted outline, a dotted line for a link or a border, the name in quieter italic. Heard of is a thin solid outline with no fill. Known is ordinary ink. No icons are added to the chart.
- **In a list or table:** a narrow eye column on the left, sortable, so "show me what they don't know" is one press on the column head.
- **On the Layers row:** an eye beside each layer. A whole layer (patrol districts, a smugglers' web of links) can be GM only.
- **On the time ruler:** an event the table has not learnt is a pencil tick.
- **On a note:** the eye replaces the note's "audience" field.

When the marks show:

- **Explore desk:** never. Someone using SSE4 as an astronomy tool never meets secrecy.
- **Build desk:** the eyes and pencil show, so secrets are set while building.
- **Run desk:** the same, plus the table ledger (section 7).
- **Players' screens:** never. A player sees ink only. What they do get is a moment of motion when something is revealed (section 8).
- Until a campaign has been shared or something has been hidden, Build shows no eyes either. The control appears when it becomes relevant, as seats and requests already do.

One accent colour still holds. Pencil is the existing muted ink with a dotted stroke, and the eye is drawn in ordinary ink, its three states told apart by shape. Nothing here uses colour to carry meaning, so it works in the Night chart and for colour-blind users.

## 5. The control

**Press the eye:** reveal to the table. The thing inks itself in (a 180 ms draw, an instant change under reduced motion), and a small slip opens from the eye: "Shown to the table. Undo". Press an open eye and it goes back to GM only, with the same slip.

While players are connected, a reveal waits two seconds before it is sent, so Undo within that time truly means they never saw it. After that, hiding still works (the thing leaves their screens), and the slip says plainly that the table has seen it.

**Hold the eye, right-click it, or press the small caret beside it:** the **Known to** popup opens from the eye. On a phone it is a sheet from the foot of the screen.

```
Known to                                    Tarn Deep (base)
The table        closed: GM only   half: Heard of   open: Known
  Kestrel crew   ·          ·           ·        (rows appear only with more than one audience)
  The Vesh       ·          ·           ·
Exceptions       Garrison strength: kept back
Told instead     Purpose: "ice mining station"       (section 6)
From             now ▾        (as of an earlier date; from a later date)
Also             what is inside it ▾   everything like it here ▾
```

- With one audience, the popup is one row of three steps and most GMs will never open it.
- "From a later date" is the record's existing `activated_at`: the news is not on the players' devices until that date on the campaign clock. It is set by hand.
- Every change is one dated `observe.grant` event through the ordinary intent path, so it undoes, it shows on the time ruler, and scrubbing the ruler back shows the chart as the table knew it then.

**Scope, for revealing many things at once** (the "Also" line, and the same three choices on a selection):

- **This.** The default.
- **This and what is inside it.** A system with its worlds, a world with its bases, a power with its holdings. Data views already inherit down the address tree.
- **Everything like this here.** All links of this kind in view, all the Crown's stations in this sector.

Bulk reveal on a chart: drag a box or lasso, or pick rows in a list, then one "Reveal to the table" (or "Known to..." for anything finer). Orac can do the same as one change request the GM approves ("show the table everything the Imperial survey had on file in 2290").

**Hand over.** One audience can be given what another knows about a place: "Give the table what the Vesh know of this sector." It is a single dated action that copies rungs from one audience to the other, and it is the general form of a bought chart, a captured database, a treasure map or a defector. It is done by hand; nothing spreads knowledge by itself in 4.0.

## 6. Partial knowledge, rumour and cover stories

- **Heard of** is the half-open eye: the base is on the map with a name, and the card says only what that rung allows. It is the fixture's "rumour" view made a standard landmark.
- **A cover story** is a fact the audience is shown in place of the true one: the base's purpose reads "ice mining station" to the table while the GM's card says "listening post". In the popup it is "Told instead". On the GM's card the true value stays in ink with the told value in pencil beneath it ("the table is told: ice mining station"). On the player's screen the told value looks like any other fact, with no mark.
- Underneath, a cover is an authored value addressed to an audience, the same shape as a claim in the territory design ("as the Crown draws it") and as names held per audience. A data view override gains one more precision beside exact, rounded and band: **told**. The true value is never sent.
- Revealing the truth supersedes the cover with a dated event, so the campaign remembers "they believed it was a mining station until day 212", and Minerva can use it.
- Programs sense through their owner's audience (record 6), so the same control decides what a patrol or a pirate knows. "The pirates do not know this lane" is one eye. A program fooled by a cover story costs nothing extra.

## 7. What the table knows, at a glance

Three views of the same knowledge, each one press away on the Run desk:

1. **The chart itself.** Ink and pencil show it over the whole view with no reading.
2. **Peek.** Hold the "See as" control (or the P key) and the chart shows exactly what the table sees; release and it returns. Choosing "See as: the table" keeps it. This is the existing lens with a faster hand.
3. **The table ledger.** A panel on the Run desk under the rail entry "The table": a line per place ("Kestrel: 4 of 10 worlds known, 2 things heard of, 3 secrets"), the reveals of this session newest first with Undo, and anything waiting on a date. With several audiences it is one column each. It is a `rail` entry in a screen document, not a tab.

## 8. On the players' side

- Nothing marks a secret, because there is nothing there.
- When something is revealed, it inks itself in on their chart and its card line counts in, once. A short "Newly learnt" list on the players' page holds the last few so a reveal made while someone looked away is not lost.
- With the GM offline, players keep what they knew, read-only, as ruled.
- Counts, lists, search, the voyage finder and travel times are all built from the audience's own knowledge, so "10 bodies" never betrays a hidden eleventh and a route never runs along a link the crew has not learnt.

## 9. Defaults

- **Generated things** (stars, worlds, belts) follow the campaign's data views, as now.
- **A new thing the GM makes** starts as known as the place it is in: a base on a world the table has surveyed is known, a base on a world they have not found is GM only. Its eye shows at once, so the choice is visible and one press changes it.
- **GM notes and `misc` fields** always start GM only.
- **Packs are public** (record 8A): a pack's "secret" base is only secret from players who have not read the pack. A pack marks such things to start GM only, and that is honest at an RPG table. Scenario secrets that must hold are made on the GM's machine.
- **Exports:** "Export for players" and the players' page carry only what the table knows. A campaign shared to Explorers is a GM's file and carries everything, with its secrets still marked.

## 10. What this does not do (4.0)

- Knowledge does not spread by itself. No gossip, no news travelling along links, no spies. Those arrive with 4.1's living civilisations, and they will write the same grants.
- No per-player secrets within a crew unless the GM makes more seats. One table, one eye, is the default.
- No strict enforcement beyond what is ruled: ordinary seats hold the seed, so generated worlds are hidden on trust; authored secrets and the whole record are never sent (record 8A).

## 11. Work packages and who takes them

| Id | What | Thread | Check |
|---|---|---|---|
| SV1 | One kernel answer, `known_to(address, field?)`: for each audience the rung, the landmark (GM only, Heard of, Known), exceptions and any cover. One intent, `set_known`, with a scope (this, with children, a list) that writes `observe.grant` events. Per-thing exceptions for one field. Counts and lists built from the audience's knowledge. | Build kernel resolve and rules | a hidden thing and a kept-back field are absent from every byte sent to a seat; a scoped reveal is one undo; counts never include hidden things |
| SV2 | The eye (three states), pencil styles and the Known to popup as one shared component with tokens; on the card title, card lines, notes and lists; the two-second hold while players are connected. Closes V3 gap 21. | Make the SSE4 test drive work end to end (chart room) | desktop, phone and Night chart screenshots; reveal and undo by keyboard alone; reduced motion |
| SV3 | Pencil and ink on charts: the system chart, galaxy and sector charts, links (HW3), territory layers (OV6 and OV9 use this control and add none of their own), the Layers row, time ruler ticks. A shared draw style in the renderer's view kit. | Build photoreal renderer (shared view kit), Build galaxy charts and travel screens (links, voyages, sector charts) | the same secret reads the same on every chart; HUD off still hides nothing from the GM that matters |
| SV4 | Run desk: the table ledger, peek, box and lasso reveal, Hand over. | chart room | "what does the table know of Kestrel" answered without opening a card |
| SV5 | Cover stories: precision `told` in data views, "Told instead" in the popup, the GM's two-line display, supersede on reveal. | kernel, then chart room | the true value never reaches a seat; the cover and its end are both dated events |
| SV6 | Players' page: reveal motion and "Newly learnt". Export for players. | chart room; Integrate Explorers site with SSE4 for exports | a reveal made while the page was closed shows on return |
| SV7 | Agents and packs: Orac bulk reveals as change requests; Minerva writes from covers for the audience and from the truth for the GM; the Traveller pack names its rungs and maps its sensor rules onto them. | Build Orac, Minerva and Tars; Build Traveller pack | Minerva never states a kept-back fact to a seat |

Order: SV1 and SV2 first (they give links, bases and notes a working secret), then SV3 and SV4, then the rest. The highways plan (HW1) stores who knows on the link and meets this control at SV3.

## 12. Questions for Alex

Each has my recommendation. Answer in the usual way ("yes to 1, 2 and 4").

1. **Pencil and ink as the way secrets look on the GM's chart, with one mark that is also the button?** Yes; Alex chose a small eye for the mark, 23 Nov.
2. **The name "Known to", with the three landmarks GM only, Heard of, Known?** Recommend yes. "The table" stays the word for all players.
3. **One press on the eye reveals to the table; everything finer is in the popup?** Recommend yes. It is the Scanned button made general, and the Scanned button itself goes.
4. **Hold a reveal for two seconds while players are connected, so Undo truly unsends it?** Recommend yes. A mis-press on a phone is the one mistake here that cannot be taken back.
5. **A fact follows its thing unless the GM makes an exception, so a card normally shows one eye?** Recommend yes. The alternative, an eye on every line, is V3 again.
6. **A new GM-made thing starts as known as its place; notes always start GM only?** Recommend yes. The other choice is "everything new starts GM only", safer but more pressing.
7. **Cover stories ("Told instead") in 4.0, as package SV5?** Recommend yes. It is set by hand, it reuses the claim shape, and no other tool gives a GM a map that lies on purpose with the truth kept beside it.
8. **Hand over (give one audience what another knows of a place) in 4.0?** Recommend yes. One action covers bought charts, captured databases and treasure maps.
9. **Replace the words under each name on the Run desk ("not known to the table") with pencil and ink, keeping the words on hover and on the selected body?** Recommend yes. The chart gets calmer and says the same thing.
10. **No marks at all on the Explore desk, and none on Build until the campaign is shared or something is hidden?** Recommend yes.
