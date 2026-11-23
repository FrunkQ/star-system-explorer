# SSE4 territory overlays: politics and other maps of "who or what, where", at every scale

Status: **proposed**, 23 Nov 2026. Written in the thread "Design territory overlays across all scales". Track **OV** (it continues OV1, the pack overlays of PR 271). Fifteen questions for Alex are in section 12, each with a recommendation. Mockup: the artifact "SSE4 Territory overlays mockup" (https://claude.ai/artifact/6fUw6HgRsmXVegPR1iMox9), its page in `OV-screens/territory-overlays-mockup.html` and four stills beside it.

## 1. What Alex asked for (23 Nov, 15:48)

"An overlay to show politics, etc across planet surfaces (countries) or galaxy maps (areas of interest). We did that in tags before ... but to create cool maps which can reflect them I guess we can design that system better. Again a basic SSE3 concept we can scale up to something that actually works across all boundaries."

## 2. The short answer

1. **One capability, not a politics feature.** A **map layer** colours places by any value that has a small number of answers: who holds it, which trade league it belongs to, how dangerous it is, what the party has charted, which language is spoken. Politics is the first layer, not the mechanism.
2. **Territory is said about places, never drawn as loose shapes.** A power holds *addresses* (a galaxy region, a system, a world, a belt, a country, a city). Every map works out its shapes from where those places are, at the date on the clock. That is why one statement shows up correctly on the galaxy chart, the sector map, the system chart and the globe, and why borders keep up when stars and worlds move.
3. **Two rules carry it across scales.** Holding a place means holding everything inside it unless something more specific says otherwise (the naming design's "inherits down the tree"). Looking from further out, a place takes the colour of whoever holds most of it, or shows as divided.
4. **Borders have dates.** A holding is a dated record, so the time ruler redraws the map, a conquest is one event, and history can be replayed.
5. **Maps can disagree.** A holding is one of *holds*, *claims* or *influences*, and a claim belongs to whoever makes it. So there is a map "as it is", a map "as the Crown draws it" and a map "as the players know it", from the same records.
6. **It extends the overlay slot that just landed.** Same pack entry, same Layers list, same HUD rule, same campaign anchors. One new kind of content (areas) beside grids and fixed lines.
7. **Still only labels.** No physics flow can read a layer. Physics can be read *by* a layer, in one direction: a border may settle on a coast, a Hill sphere or a nebula's edge because the physics already knows where those are.

## 3. What exists today

- **V3** hung allegiance and ownership on tags, coloured markers by tag, and let a GM pin a picture of borders behind the stars (V3 gaps 18). Nothing connected a country on a surface to a sphere on a star map, nothing had dates, and a border was a picture.
- **Tags** (accepted 21 Nov): a label on an address, present or absent, a dated event when authored. Exclusive groups exist ("at most one of the group per object"). The space opera pack already declares `so.faction` this way and its dressing rules put a `faction` on the bases they place.
- **The record** (track J): actors are addressable (`/actor:vesh/actor:third-fleet`, so powers nest), and relations between an actor and a place (`owner`, `allegiance`) are kept values of shape `ref`, dated, filtered by data views.
- **Lenses** (handoff 8.6) already list `layers` and a `colour_by`, and name a Political lens. The surface design (4A.6) already says countries are "a region layer over the surface ... with dates so borders move on the time ruler".
- **Pack overlays** (OV1, PR 271): `provides.other.overlays: [{ id, label, frame, grid?, layers? }]`, drawn only while HUD is on, listed in the view's Layers tool, placed by `workspace.overlay_anchors`. Frames `galaxy-plane-pc`, `cluster-pc`, `system-au`. Content so far: a hex grid, or fixed lines, hexes and labels.
- **Naming** already depends on this design: "a polity or species that carries a scheme supplies the Local layer for places inside its territory". Nothing yet answers "inside whose territory is this place?".

So the subjects, the dates, the visibility rules and the drawing slot all exist. What is missing is the piece between them: how a statement about places becomes an area on a map, at any scale.

## 4. The model

### 4.1 A map layer

A map layer is pack data. It says what value it shows, how the gaps between places are filled, and how it is drawn.

```
provides.other.overlays: [{
  id: "so.powers", label: "Powers of the Reach",
  areas: {
    by:      { relation: "holds" }            what is read at each place (4.2)
    answers: { from: "actors", under: "so" }  the set of answers, and so the key
    level:   "auto"                           how far down the answer tree to split, by zoom (4.4)
    fill:    { between_stars: { reach: { value: 4, unit: "pc" } },
               in_system: "hill", on_surface: "regions" }      (4.5)
    weigh_by: "population"                    what "most of it" means when rolling up (4.4)
    look:    "sse.look.atlas"                 a drawing recipe (section 6)
    tier:    "obvious"                        default disclosure, as for a tag
  }
}]
```

The entry has no `frame`. A layer of areas is about places, so it is drawn by every view that shows places, in that view's own frame. A grid or a fixed drawing still names its frame, as today.

### 4.2 What a layer can read

| Source | Example layers | Notes |
|---|---|---|
| A relation to an actor (`ref` kept value) | Powers, trade leagues, faiths, corporate concessions, patrol districts | The full version: status, strength, nesting, dates, "according to whom" (4.3). |
| An exclusive tag group | Tech era, port grade, anything a GM tags by hand | What V3 did, kept: a tag group with a colour per tag becomes a map with no further setup. Tags are dated events, so it moves with the ruler too. |
| A `choice` kept value | Travel zones (green, amber, red), law level bands, quarantine | |
| A fact with few answers, or a number cut into bands | Biome, plate, climate band, class label; population, danger from radiation, resource richness | Read-only from physics. This is the same key and the same drawing as a political map, so the two can be read together. |
| What an audience knows | Survey coverage ("charted by the party"), fog of war | From the knowledge state the data views already keep. |
| A reach that the layer computes | Jump range from here, sensor cover, the **news horizon** (how far word of an event has travelled at light speed by the date on the clock) | A centre, a radius, and optionally a rate. The radius may come from a fact (a ship's range) or from the clock. |

One sentence covers all of them: *a layer gives each place at most one answer from a short list, and the map colours places by their answer.* Two layers can be on at once (4.6).

### 4.3 A holding

For the relation source, the record gains one relation kind with a little more in it than `owner`:

```
kind: "record.set"   subject: [place address, actor address]
payload: { relation: "holds",
           status: "holds" | "claims" | "influences",
           strength?: 0..1,              for influence; drawn as tint depth
           extent?: Extent,              only when the place itself is not enough (below)
           by?: actor,                   who asserts it; absent means it is simply so
           tier, audience }              as any kept value
```

- **Holds** is control. One holder per place per layer at a time.
- **Claims** is an assertion by `by`. Any number may overlap. Where a claim covers a place someone else holds, the map shows it as disputed.
- **Influences** is a soft presence with a strength: a sphere of influence, a trade network's pull, a faith's share.
- It starts at its event's date and ends when a later event replaces or withdraws it. Nothing is edited in place, as with every record.
- It is a kept value, so data views, tiers, per-object overrides ("secretly holds this station") and audiences apply with nothing added.

**Extents: when "this place" is not enough.** Most holdings need only an address. Three more ways to say where, all anchored to an address so they move with it:

| Extent | Says | Used for |
|---|---|---|
| (none) | the whole address | a system, a world, a country, a patch of the galaxy |
| `reach` | everything within a distance (or a travel cost) of the address | a sphere of influence round a base, a danger zone round a pulsar, a Hill sphere |
| `band` | between two distances from the address | "the inner system", a belt, a shell of the halo |
| `outline` | a drawn shape in the address's own frame | a hand-drawn border on a continent, an area of interest on the galaxy chart |

An outline is the only place a drawn shape appears, and it is stored as points in the frame of the thing it is drawn on (latitude and longitude on a body, parsecs on the galactic plane), never in screen or picture coordinates.

### 4.4 Across scales: inherit down, roll up

**Down.** A holding covers everything inside its place. The Crown holds the Tarn system, so it holds Tarn's worlds, moons and stations without a record on each. A more specific holding wins inside its own place: the League's station in Tarn's belt is the League's. This is the rule naming settings already use.

**Up.** Seen from further out, a place shows the answer that holds most of it, measured by the layer's `weigh_by` (population where the campaign keeps one, otherwise area or count). If nobody has 60 percent, it is drawn **divided**: the mark for the place is split by share, and its area takes the two largest in stripes. So a world with five countries is a ring in five parts on the system chart and a striped or single-coloured dot on the sector map.

**The answer tree.** Actors nest (a kingdom inside a league inside a species' sphere), and so can any list of answers. `level` chooses how far down to split. On "auto" a view shows the deepest level whose areas are still big enough to read (the same detail-by-zoom idea as OV1's hexes), so the galaxy chart shows the three great spheres, the sector map shows the powers inside one, and the globe shows countries. The Layers row has a small step control to force a level ("Spheres, Powers, Countries"). A member's colour is a shade of its parent's, so stepping down reads as subdivision.

### 4.5 Filling the space between places

Systems are points and space is empty, so a map needs a rule for the gaps. The rule is per scale, written once in the kernel, and the views draw what it returns.

| Scale | A place is | Between places | Natural edges the physics offers |
|---|---|---|---|
| Cosmic web | a galaxy or group | reach in megaparsecs | void walls, filaments |
| Galaxy | a patch cell (the octree), a named region | rolled-up cells; outlines for areas of interest | arm edges, the bar, the disc's edge, the halo |
| Local stars, sector | a system | **reach, met halfway**: each held system colours space out to the layer's reach, and where two meet the border runs midway | gaps wider than a jump, a nebula's edge |
| the same, with a pack grid placed | a hex | borders follow hex edges | the grid |
| System | a body, belt or station | Hill spheres round bodies, bands between orbits | Hill sphere, belt edges, the frost line, the heliopause |
| World | a surface region or feature | regions tile the surface; or grown from capitals over travel cost (surface design 4A.6) | coasts, rivers, range crests, plate edges, the ice line |

- **Reach is why a map never paints the void.** Space further than the reach from anything held stays unclaimed paper.
- **Three dimensions.** The rule uses true distance. A flat chart draws the slice through the plane it shows. See question 13 for 3D.
- **Natural edges are offered, not imposed.** On the Build desk a drawn border snaps to them, and a fill rule may name one ("in_system: hill"). Physics is read; nothing is written back.
- **A point can be asked.** `who_holds(point or address, layer, date, audience)` is a kernel question with a "Why?" (the holding, the rule, the inherit or roll-up step). Drawing and asking use the same rule, with a check that they agree.

### 4.6 More than one layer

One layer fills areas with colour. A second layer on top draws as **edges and hatching only** (a danger zone's hatched outline over the political tints), and a third as marks on places. The Layers list is ordered and the top area layer gets the fill. This keeps any stack readable and stops colour mixing into mud.

### 4.7 What may read a layer, and what may not

May read `who_holds` and "is inside": conditions in rules, triggers and programs ("when a ship enters League space"), the finder ("nearest Crown port"), sifters (border worlds, enclaves, choke points, a world that changed hands three times), naming (the Local layer), Minerva, cards and the ruler.

May not: any physics flow, RESOLVE, formation, weather, life. The loader refuses it, as for tags. The one bridge stays the explicit, asked and marked `set_pin`.

## 5. Time

- **The clock draws the map.** Every holding has a date, so the map at any date is a fold over the record, like everything else. Look-ahead and what-if branches include it.
- **Places move, borders follow.** Stars drift round the galaxy and worlds round their star. Holdings are on addresses, so over deep time a sphere of influence shears with galactic rotation by itself. Nothing stores a shape between stars.
- **The ruler shows changes.** With a layer on, the ruler carries a tick for each border change in view. A card has "History of this border".
- **Motion.** Crossing a change, the affected area fades from one answer to the other in 240 ms (a plain cut under reduced motion). That fade is presentation, and nothing in between is claimed to be a real state (interface rule 10).
- **A changing reach.** A reach may have a rate. The news horizon grows at light speed from its event; a blockade shrinks. The radius comes from the clock, with no animation data.
- **Play it back.** "Play the history" on the Layers row runs the ruler from the first change to the campaign's now.

## 6. How it looks

The aim is a map people want to print: an atlas plate, not a heat map.

- **Atlas look (default).** Each territory is a band of colour along the inside of its border, fading towards the middle, over the untouched chart. Stars, coasts and orbits stay readable because the middle of a territory is nearly bare. This is the old hand-tinted atlas method and it works on dark ground as well as on paper.
- **Lines say status.** Solid: holds. Dashed: claims. Dotted, with tint by strength: influences. Disputed ground is hatched in the claimant's colour over the holder's tint. Divided places are striped.
- **Names are lettered, not labelled.** A territory's name runs along its long axis in spaced italic capitals, sized to the area, and drops out when the area is too small to carry it (the key still lists it).
- **Colour.** Each theme gains a **map palette**: eight colours tuned to that theme's ground, checked by the existing colour-vision check, each paired with a pattern for themes and eyes that need it. A power may state a preferred colour; the map gives it the nearest free slot. Neighbours never share a slot. With more than eight answers in view, slots are reused for answers that do not touch.
- **Looks are data.** `sse.look.atlas` first; `sse.look.cells` (flat fills on hexes, the tabletop look) with the Traveller pack; contour lines for influence; in-universe looks later with the view styles. A look is a recipe like any render recipe, so Orac and packs can add one.
- **HUD.** Layers draw only while HUD is on, as OV1 says. HUD off is still a photograph. In True with HUD on, borders lie on the globe as they would from orbit.
- **The key** sits with the Layers list: one row per answer in view, with its colour, its share of what is in view, and a press to open the actor's card. Pointing at a row lifts that territory and quietens the rest.
- **Touch answers.** Pointing at a territory inks its edge and shows, beside the pointer: "Tarn. Held by the Meridian Crown since 2291. Claimed by the League." Pressing opens the card of the place, which has the holding on it.
- **Phone.** The Layers list and key are a bottom sheet. The key collapses to colour chips. Lettering thins out first, borders last.
- **Honesty marks.** A political map is authored from edge to edge, so inking every border in the accent would mean nothing. On the GM view the mark sits on the key row and on the holding's line on the card: "drawn by you", "from pack: Meridian Reach", "by rule, why?". Players never see them.

## 7. Making and changing territory

Common things are one press away; the rest is in a menu that already has the context.

- **On any card (Build and Run):** a line "Held by: Meridian Crown, since 2291" with a picker. Changing it writes one dated event at the campaign's now. "As of an earlier date" is in its menu, for backfilling history.
- **Paint (Build):** pick an answer from the key, then press or drag across systems, hexes, bodies or surface regions.
- **Draw (Build):** an outline on a surface or the galaxy chart, snapping to natural edges and to existing borders.
- **Grow (Build):** "grow from here over travel cost until it meets a neighbour or a limit", which gives borders on ranges and rivers on a surface and along jump routes between stars. The result is ordinary holdings the GM can then edit.
- **At the table (Run):** "Tarn falls to the League" is the same card line. Showing a holding to the players is its tier, one press, as for any fact.
- **Orac:** "the Vesh expand rimward two parsecs a decade from 2200 until they meet the League" becomes dated holdings the GM approves as one change.
- **Programs:** two effects, `claim` and `cede`, under the existing approval policy, so a faction program can take a world and the GM sees it asked.

## 8. Whose map

One switch on the Layers row, "According to":

- **As it is** (GM only): holds in tint, claims hatched, secrets shown and marked.
- **As <a power> draws it**: that power's claims drawn as if held. A handout that lies in the way the Crown's own atlas would.
- **As the players know it**: only holdings their data view discloses. This is what their screen shows, and it is the existing "as players see it" lens applied to the layer.

On a player's map, unclaimed space and a secret holding both look like bare paper, so nothing leaks by its absence. Space the party has not charted keeps the existing unsurveyed hatching. A seat tied to a polity (handoff 8.7) sees what that polity knows.

## 9. Packs

- **A universe pack carries its layers:** the layer entries, its actors as templates, its colours and its look.
- **Territory by rule.** A procedural universe has no fixed addresses for a pack to name, so a pack states territory the way it states bases: **claim rules**, cousins of dressing rules. "A system with a League base is the League's, and so is anything within four parsecs of two of them." They are deterministic, lazy, explained, and arrive as **candidates**: ghost borders on the GM's chart until adopted, per system, per region, or on first visit. The GM can hold one off. Swapping the pack changes every border not yet adopted.
- **Territory by address** for packs that ship a fixed map (a published sector with named systems): plain holdings, valid only in that pack's own universe.
- **Grown histories (later).** A rule may also have a start and a pace: powers spread from seed worlds along travel cost and stop where they meet. From the universe seed that gives a whole political past, replayable on the ruler, the way the galaxy now has an assembly history. See question 12.
- **Explorers:** a layer set is part of a universe pack or a small pack by itself ("tag set" already exists as a kind). Nothing new on the site.
- **V3 import:** the importer offers "treat this tag group as powers": each tag becomes an actor and each tagged place a holding from the save's date. A pinned border picture stays a pinned picture (V3 gap 18).

## 10. How this extends OV1

| OV1 today | With this design |
|---|---|
| `OverlaySpec { id, label, frame, grid?, layers? }` | adds `areas?` (4.1); `frame` optional when only `areas` is given |
| `OverlayLayer` kinds: lines, hexes, labels | adds `areas`: `{ kind: "areas", items: [{ key, status, strength?, rings: point lists, label?, label_turn? }] }` |
| `OverlayInk`: faint, ink, accent | adds `{ slot: 1..8, pattern? }` from the theme's map palette |
| Frames: galaxy-plane-pc, cluster-pc, system-au | adds `body-latlon` and `web-mpc` |
| Anchors in `workspace.overlay_anchors` | unchanged for grids and pictures; holdings live in the chronicle (they are the campaign's record, and players' maps need them) |
| A placed grid | a layer whose look follows the grid snaps its borders to the hexes |
| Drawn under HUD, listed in Layers | unchanged; the Layers row gains the key, the level step and "According to" |

A lens names layers by overlay id, so "Political" is a lens that turns the Powers layer on, sets the chart's body colours to plain ink and puts holdings first on cards. No new control anywhere: Layers for one layer, a lens for a bundle.

## 11. Build packages

Owners are suggestions for the coordinator. Stage is from the coming-together plan.

| Id | Package | Owner thread | Stage | Checks |
|---|---|---|---|---|
| OV2 | Contract: `areas` in the overlay spec and layer kinds, palette slots, new frames | Build photoreal renderer (owns `overlay.ts`) | 2 | spec round-trips; OV1 tests unchanged |
| OV3 | Kernel: relation `holds` with status and extent; inherit and roll-up; `who_holds` and `holdings_in(bounds, layer, date, audience)` with "Why?"; the reach rule | Build kernel resolve and rules | 2 | same answer on two devices and in any order; a holding on a system answers for its moons; a divided world rolls up divided; no flow can read it (loader test); goldens untouched |
| OV4 | Drawing: areas in chart-grammar with the Atlas look, lettering, key, pointing and pressing; map palette in both themes | Build photoreal renderer with the chart room thread | 2 | screens on desk and phone, light and Night chart; colour-vision check passes; drawn border agrees with `who_holds` at 1,000 sample points |
| OV5 | "Your systems" map, sector map and galaxy chart carry area layers; Layers row with level step | Build galaxy charts and travel screens | 2 to 4 | a layer on at three scales shows the same holdings |
| OV6 | Card line "Held by", Paint, and reveal on Run | chart room thread | 5 | one press changes a holder; undo restores it; a player screen updates |
| OV7 | System scale: body rings, Hill spheres, bands | chart room thread | 5 | a five-country world shows a five-part ring |
| OV8 | Time: ruler ticks, border history, play it back, a reach with a rate | chart room thread | 5 | scrubbing across a change redraws; reduced motion cuts |
| OV9 | "According to", and layers on the players' page | chart room thread | 5 | a secret holding never reaches a player device (wire test) |
| OV10 | Claim rules and candidates; layers in the space opera and near-future packs; Traveller allegiance and travel zones as layers on the hex map | kernel thread, then Build Traveller pack | 5 | fixture pack gives the same ghost borders on two devices; adopt, swap the pack, adopted borders stay |
| OV11 | Surface: outlines drawn on the globe, snapped to coasts; later, grown over travel cost on the surface atlas | Build photoreal renderer | 5, then with M6c | a drawn country survives turning the globe and a change of view style |
| OV12 | Readers: conditions, finder, sifters, naming's Local layer | kernel thread, matter data sheet thread (naming) | 5 | "nearest Crown port" finds one; a world inside Vesh space gets its Vesh name |
| OV13 | V3 import of tag groups as powers | SSE4 converter | 5 | a real V3 save with faction tags opens with a Powers layer |
| OV14 | Grown histories (question 12) | kernel thread | after 5 | design note first |

## 12. Questions for Alex, each with a recommendation

1. **One mechanism: a map layer colours places by any value with few answers, and politics is the first layer?** Recommend yes. Trade zones, danger, survey coverage, languages, faiths, biomes and jump range then cost a pack entry each, not a feature each.
2. **Territory is held on addresses and maps work out the shapes, with drawn outlines only where nothing else will do?** Recommend yes. It is what makes one statement appear on every scale and keeps borders right as things move.
3. **Extend OV1 (same pack entry, Layers list, HUD rule) with an `areas` kind, and no new switch?** Recommend yes.
4. **Holdings are the record's relations to actors, with status holds, claims or influences; a plain exclusive tag group can still drive a simple map?** Recommend yes. The tag route keeps V3's ease; the relation route adds dates per power, disputes and nesting.
5. **Inherit down, roll up, and "divided" below a 60 percent share?** Recommend yes, with the share measured by population where there is one.
6. **Between stars: each held system colours space out to a reach and borders run midway; with a pack grid placed, borders follow its hexes?** Recommend yes, default reach 4 pc, set per layer.
7. **Offer the physics' own edges (Hill spheres, belt edges, coasts, rivers, ranges, nebula edges) as snap targets and fill rules, read-only?** Recommend yes. It is the "real terrain reasons behind its borders" the surface design promised, at every scale.
8. **"According to": as it is, as a power draws it, as the players know it?** Recommend yes. A map that lies on purpose is a handout no other tool makes.
9. **A map palette of eight colours with patterns per theme, as the one exception to "one accent", only inside map layers?** Recommend yes. Neighbours never share a colour; a power's own colour is a preference.
10. **The Atlas look (colour along the inside of the border, bare middle, lettered names) as the default, with the flat hex look for tabletop packs?** Recommend yes. See the mockup.
11. **Honesty marks on the key and the card, not on every border?** Recommend yes, GM view only as always.
12. **Grown histories: powers spreading from seed worlds over travel cost through time, by pack rule, so a universe has a political past from its seed?** Recommend: design reaches it now (9), build it after stage 5 with its own short note. It is the most distinctive thing here and the easiest to get wrong if rushed.
13. **In 3D charts: draw a territory as its footprint on the reference plane with each star's stem in its holder's colour, and leave see-through volumes for True later?** Recommend yes. Volumes look good in a trailer and read badly on a working chart.
14. **Surface countries in two steps: hand-drawn outlines on the globe at stage 5, grown borders when the surface atlas has regions (M6c)?** Recommend yes, so the "RPG country app" is usable before the full ground detail exists.
15. **First layers to ship as proof it is general: Powers, Travel zones, Charted by the party, and the News horizon?** Recommend yes. The last is cheap and shows time and physics in a political map: the rim does not know the capital has fallen yet.

## 13. Standing rules this design keeps

Tags and layers are labels and never feed physics. One general mechanism. Time is first-class. Players see only what their data view allows. Honesty marks on the GM view only. HUD off is a photograph. Looking promotes; nothing is worked out for the whole galaxy in advance. Deterministic on every device. Packs carry it; the GM always wins.
