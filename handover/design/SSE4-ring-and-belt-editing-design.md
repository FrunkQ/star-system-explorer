# SSE4: editing rings and belts (track RB)

Status: **PROPOSED 26 Nov 2026, not accepted.** Eight questions for Alex in section 11, each with a recommendation. Design only; nothing here is built. RB is a code only, never shown in the product.

Readable copy with drawn mock-ups: https://claude.ai/artifact/1YxqDTEaSyXXPqU43Gk81j

Alex's brief, in his words (26 Nov 2026, 17:43 to 17:47 UTC, while looking at Saturn's rings in the shadows thread):

- "rings with real structure should be '1 ring! but it can describe all its own subrings?"
- "ring and belt editing will be an intersting UI :)"
- "sggeston is we 'generate plausible' and they can piuck the one they want - in initial gen - we just pick the one our generator says to be consitent"
- "or do we 'fine design' a ring and let the user do it?"
- "any way for a crazy cult to paint a ring of suturn red (thinkoing John Varley) - is this even plasible in SSE?"
- "it is a weird 'edge case' but would let an advanced culture beautify their rings ;)"
- "just think those slightly crazy things as we deign... implausible is not a limit..."

This is the "later note" the editing design promised (section 11: "Rings and belts are placed with the same ghost and have two handles, where and how much; their detail is a later note") and the fix for its finding F2 (Saturn's rings could not be made to full precision).

Read with: `SSE4-editing-design.md` (sections 3, 3B, 5, 7, 8, 13), `SSE4-three-modes-design.md` (section 1), `SSE4-conserved-chain-design.md`, `SSE4-real-catalogues-design.md` (sections 6 and 7, proposed), `status/small-bodies.md`, `interface-style.md`.

---

## 1. The short answer

1. **A ring and a belt are the same thing to edit:** a crowd of small bodies spread over a range of distance from a parent. One record, one box, one set of handles. A ring's parent is a world and its sculptors are moons. A belt's parent is a star and its sculptors are planets. Nothing else differs.
2. **One ring per world, one belt per region.** It describes its own sub-rings as a list of **zones** inside it. Saturn is one ring with named zones (D, C, B, Cassini Division, A), not five objects.
3. **Pick or fine design: both, and they are one mechanism at two depths.** The structure is always worked out from three layers: a base shape that history decided, the marks the moons make, and anything the GM drew by hand. "Pick a plausible one" changes the first layer with one press. "Fine design" adds to the third. The moons' marks always come from the moons.
4. **Shaping never changes how much there is.** The zones are shares of the ring's mass. Making one zone denser trades against the rest, the way the air mix already works. Only the "How much" handle changes the total, and that change is booked against a named source.
5. **A gap wants a cause.** Asking for a gap offers the two honest routes the editing design already has: Reach it (add the moon that would hold it open) or Pin it (draw it anyway, with the honesty mark on the GM's view).
6. **Colour is never typed.** A ring is the colour of what it is made of. To redden a ring you add a red material, as a coating or mixed through, and the app says how much that takes, where it could come from and how long it lasts. Painting the B ring red comes out as a real but very large job (section 8).
7. **Implausible is not a limit.** Every wall is a mark the handle passes through. The physics says what would happen and what it would take. Breaks are shown to the GM only.

---

## 2. What exists today (read from main at 5023104, 26 Nov 2026)

| Thing | State |
|---|---|
| Generated belts, clouds and rings | **Built in the kernel.** `sse_formation::leftovers::Cloud`: kind, parent, inner and outer edge, spread of orbit shapes and tilts, mass by six element groups, largest member, count over 1 km. `Cloud::resolve(seed, k)` gives the k-th largest member. A giant keeps a massive ring one time in four, with real mass taken from its moon-forming disc |
| Generated rings on screen | **Missing.** The kernel ring never reaches a view |
| The Solar System's rings | Nine separate regions in `packs/base-sol`, five of them Saturn's, each with edges and one flat optical depth. No mass |
| The Solar System's belts | Three regions, edges only, no mass (the real catalogues note lists this as a gap) |
| Structure from moons | In the drawing code only (`renderer/src/ring-structure.ts`, VP-R3): resonance edges, dips, gaps round embedded moons |
| Editing | None for rings or belts. "ring or belt" exists as a scale class in the units engine. `make_small_body` makes one member out of the nearest belt and debits it |
| Being built now | Thread "Give planetary rings real structure": one ring per world as a kernel answer, with sub-rings, gaps from moon resonances and embedded moons, and optical depth that varies with radius. Seam in section 9 |

---

## 3. One record: the ring or belt

The kernel's `Cloud` is already the general record. Editing treats every kind of it the same way. What a GM can truly set, as the editing design asks ("a handful of handles"):

| Handle | What it sets | What follows from it |
|---|---|---|
| **Where** | inner edge and outer edge | which resonances fall inside it, how long it lasts there, what it looks like from the parent |
| **How much** | total mass, a real amount | how opaque it is, how many members over any size, the largest member |
| **Made of** | shares of ice, rock, carbon and metal (the Makeup bar, unchanged) | colour, brightness, density of the grains, what a prospector finds |
| **Grains** | the size ladder: the largest member and how steeply numbers rise toward small sizes | opacity for a given mass, how it scatters light, the list the Detail control cuts |
| **Stir**, at Science depth | how thick the sheet is and how stretched and tilted the members' orbits are | a calm thin ring or a puffed-up hot belt; collision speeds |

**Results, not handles:** optical depth, brightness, colour, the count of members, every zone the moons make, how long the ring lasts, and what it weighs in each element. To change one of these, use Reach it or Pin it (editing design section 8).

The row on the card, on Build:

`Where 74,500 to 136,800 km · How much 0.4 of Mimas · Made of ice · Grains up to 10 m · Structure 5 zones, 3 held by moons`

"Structure" is the fifth chip. It opens the profile (section 4). Curious depth names the handles in plain words (Where, How much, Made of, Lumps). Science adds Stir and the size ladder's slope. Engine adds field names and the blocks each feeds.

**Walls are marks, as everywhere:**

- On Where, round a world: the world's cloud tops, the distance inside which tides stop grains gathering into moons, each moon's orbit, the edge of the world's reach. A ring drawn outside the tide limit is allowed. The verdict says "outside the tide limit: gathers into moons in about 10,000 years. *Keep it as a ring by pinning it*, or *let it gather*."
- On Where, round a star: each planet's reach, the strong resonances with each giant, the snow line.
- On How much: the mass at which the ring's own gravity clumps it; the mass the parent's leftover stock could supply (section 6).

**Kinds** are chips that set several handles at once and say what they set, as for a star. Each is an origin that formation already knows, so nothing new is modelled:

| For a ring | For a belt |
|---|---|
| A moon pulled apart as it spiralled in (massive, icy) | Leftovers that never grew (between a rocky world and a giant) |
| A comet or small moon shattered recently (young, bright) | Icy outer belt beyond the last giant |
| Dust knocked off small moons (faint, thin) | Debris of a shattered world (holds a bared core) |
| Narrow rings held between pairs of small moons | Bodies sharing a giant's orbit |
| | A far cloud of comets (a sphere: Where and How much only) |

**Adding one** uses the ghost the editing design already has (section 5). The ghost is a hoop that follows the pointer; a second press sets the other edge. The panel beside it shows the Kinds and the verdict.

---

## 4. Structure: three layers, one profile

The profile is the ring seen edge to edge as a graph: distance from the parent across, how much is there upward. It is always the product of three layers:

| Layer | Who decides it | How the GM changes it |
|---|---|---|
| **1. The base shape** | History. Where the broken moon's ice landed, how long the ring has spread, how lumpy it began. The physics does not fix this, so the generator draws it from the seed | **"Others like this"**: pick another plausible draw |
| **2. The sculptors' marks** | The moons (or planets). Edges held by strong resonances, dips at weaker ones, gaps round moons inside the ring, wave trains | Move, add or remove the moon. The marks follow |
| **3. Hand-drawn zones** | The GM | Drag a join, raise or lower a zone, name it |

### 4.1 Pick: "Others like this"

Layer 1 is one whole number, the **variant**. Variant 0 is the one the generator picked as consistent, and it is what first generation uses, as Alex said. Every other variant is another plausible ring for the same world, the same mass and the same moons.

In the Structure box the first row is five small strips, each a picture of one variant's profile with a line of words ("Dense inner half, broad faint outer half", "Three bright bands, thin dark lanes between"). Hover or touch shows it on the world in pencil. A press sets it: one change, one Undo. "More" deals five more. On Explore it is a try; on Build it is kept.

The moons' marks are the same in every variant, because the moons have not moved. A Kind may add moons (narrow rings need their pairs of small moons) and says so.

### 4.2 Fine design: zones

Below the row is the profile itself, with the zones as spans along it.

- **Drag a join** sideways to move an edge between two zones.
- **Drag a zone up or down** to make it denser or thinner. The rest give way in proportion, so the total mass does not change (section 6). The figure beside it reads in both ways: its share of the ring and its optical depth.
- **Press between joins** to split a zone. Name it by typing on its label.
- **Type** any edge or amount in any unit. "Type it all in" lists every zone as a row, so Saturn's measured bands paste in from a table (editing design 3A).
- **Each zone shows its cause** as a small tag: *Mimas 2:1*, *cleared by Pan*, *as generated*, *measured*, *drawn by you*.
- A hand-drawn zone has **"Let physics decide"** in its menu, which removes it.

This needs one new part in the editing grammar, the **Profile**: an amount along a line, with joins that drag sideways and spans that drag up and down, always summing to the whole. It is the Bar with a second direction. It is general: the same part would edit a disc's gas with distance, an air's layers with height, or a galaxy's gas with radius. Question 4.

### 4.3 Moons and their marks

- A mark that a moon makes is drawn with a thin line to that moon's tick on the same axis, so the cause is visible.
- **Move the moon and the marks move.** While a moon is dragged on the world's chart, the ring's marks follow in pencil from the sweep; letting go settles them. No extra step.
- **Drawing against a moon is allowed.** Fill in a gap that a moon holds open and the verdict says: "Pan clears this in about 2,000 years. *Remove Pan*, or *Pin it*." Pinning it adds the honesty mark on the GM's view.
- **Asking for a gap with no cause** offers both routes in the zone's verdict: "Nothing holds this gap open; it fills in about 500 years. *Reach it: add a moon of about 5 × 10^15 kg at 133,600 km*, or *Pin it*." Reach it adds the moon as one change with the ring.
- "Could be: a gap here", "Could be: a sharp outer edge" and "Could be: rings" (already on the editing design's list) are the same recipe run from the other side.

For a belt, read "planet" for "moon": gaps at a giant's resonances, edges where a planet's reach begins. The real catalogues note found that generated belts have no resonance gaps today; RB6 gives belts the same sculpting answer that rings get.

### 4.4 How long things last

Every hand-made feature carries a "lasts about" figure from the same physics that spreads, darkens and drains the ring: a pinned gap fills, a ring outside the tide limit gathers, a coating gets buried. The figure is on the zone's verdict and in the GM's "What does not add up" list. It never stops the edit. With "From this date" (editing design section 13) the feature becomes an event on the ruler, and sliding time forward shows it fade.

---

## 5. Mock-up sketches

Sketches in text; the readable copy has them drawn. Figures are stand-ins, not kernel output.

### 5.1 The card's row and the Structure box (Build desk, a world's close-up)

```
 Saturn's ring                                                       [authored ·]
 Where 74,500–136,800 km · How much 0.4 Mimas · Made of ice · Grains to 10 m · Structure 5 zones
                                                                      ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾
 ┌─ Structure ────────────────────────────────────────────────────────────────────┐
 │ Others like this                                                               │
 │ [▂▅▇▇▂▆▆] [▆▆▂▇▇▇▃] [▃▃▇▂▇▂▇] [▇▇▇▅▃▂▁] [▂▇▂▇▂▇▂]   More                       │
 │  as generated                                                                  │
 │                                                                                │
 │ how much                                                                       │
 │  ▲            ┌────────────┐                                                   │
 │  │            │     B      │            ┌───────┐                              │
 │  │   ┌────────┤            │            │   A   ╎│                             │
 │  │   │   C    │            │  Cassini   │       ╎│                             │
 │  │ D │        │            ├────────────┤       ╎│                             │
 │  └───┴────────┴────────────┴────────────┴───────╎┴──────▶ distance from Saturn │
 │      74,658   91,975       117,507      122,340 ╎ 136,780 km                   │
 │                            ╎ Mimas 2:1          ╎ cleared by Pan   ╎ Janus 7:6 │
 │  moons ──────────────────────────────────────────●Pan──●Atlas───●Janus──●Mimas │
 │                                                                                │
 │ B ring · 55% of the ring · optical depth about 1.5 · measured                  │
 │ Lasts. Outer edge held by Mimas.                                               │
 │ Reset · Type it all in · From this date · Why?                                 │
 └────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 A gap with no cause

```
 │  New gap · 133,400 to 133,800 km · drawn by you                                │
 │  Nothing holds this open: it fills in about 500 years.                         │
 │  Reach it: add a moon of about 5 × 10^15 kg here  ·  Pin it                    │
```

### 5.3 On the chart: edges are grips

```
            .  ·  ˙  ˙  ·  .
        ·  ◜‾‾‾‾‾‾‾‾‾‾‾‾‾‾◝  ·          ◇ inner edge grip      ◆ outer edge grip
      ·  ◜   .─────────.   ◝  ·         ┆ tide limit (a mark, not a stop)
     ◆  │   (  Saturn   )   │◇ ┆        The readout rides beside the pointer:
      ·  ◟   `─────────´   ◞  ·         "Outer edge 136,800 km · inside the tide limit · lasts"
        ·  ◟______________◞  ·
            ˙  ·  .  .  ·  ˙
```

One grip changes one thing. Dragging a zone's join on the chart is the same join as in the box.

### 5.4 A belt, same box (system level)

```
 Kestrel's inner belt
 Where 2.1–3.3 AU · How much 4% of the Moon · Made of rock and carbon · Grains to 940 km · Structure 4 zones
 ┌─ Structure ───────────────────────────────────────────────────────────┐
 │ Others like this  [▅▇▂▆▃] [▇▅▅▂▂] [▂▆▇▆▂] …                          │
 │  ▲     ┌─────┐   ┌────────┐  ┌─────┐                                  │
 │  │ ┌───┤     │   │        │  │     ├──┐                               │
 │  └─┴───┴─────┴───┴────────┴──┴─────┴──┴──▶ distance from Kestrel      │
 │           ╎ 3:1 with Tarn   ╎ 5:2      ╎ 2:1 (outer edge)             │
 │ Detail: down to 10 km · 1,840 bodies of about 2.1 million             │
 └───────────────────────────────────────────────────────────────────────┘
```

### 5.5 "What it would take" (painting a zone)

```
 │ B ring · Made of: ice 99.9% · + iron oxide, as a coating                       │
 │ Colour: rust red (was pale tan)                                                │
 │ What it would take                                                             │
 │   About 2 × 10^15 kg of iron oxide: a metal-rich asteroid about 9 km across.   │
 │   Take it from…  [ a body in this system ▾ ]                                   │
 │   Lasts about …: collisions fold the coat into the grains; infalling dust      │
 │   darkens it.                                                                  │
 │ Do this from this date · Pin the colour instead                                │
```

---

## 6. Keeping the books (the conserved budget chain)

The chain's rules: mass and the six element groups are real amounts; no join passes a description without an amount; looking changes nothing and only deeds do; plausible, repeatable and explained, not exact.

1. **A ring or belt is a reservoir.** Its mass by element group is its stock. Zones are a split of that stock, so their masses add up to the ring's, group by group. The Profile part enforces it in the page and the kernel renormalises on the way in.
2. **Shaping is free.** Variants, joins and zone heights move mass about inside the ring. No source is needed and nothing shows on the GM's list.
3. **"How much" is a claim on a source.** Raising a ring's mass takes it from somewhere, and the handle's verdict says where:
   - by default, **the parent's leftover stock** (the giant's moon-forming disc, or the system's leftover solids), which the formation budget already tracks and closes;
   - or **a named body**: "Take it from…" a moon, an asteroid, a comet. This is what `make_small_body` already does in the other direction (`made_from`).
   - If the stock cannot cover it, the edit still happens. It is marked authored, and the GM's "What does not add up" list says "Tarn's ring holds 3 times what its disc had: 2 × 10^19 kg from no source", with "Take it from a moon" as the offered fix. No new ledger is built for this; it is one line against the system's birth claim.
4. **Lowering "How much"** returns the mass to the same stock, or names where it went ("fell into the planet", "gathered into a moon").
5. **Two acts make the big changes honest, and both conserve mass exactly:**
   - **Break up**: a moon inside (or dragged inside) the tide limit becomes a ring; a world becomes a debris belt. The moon's makeup becomes the ring's. Formation already makes rings this way.
   - **Gather**: a ring, or a zone of one outside the tide limit, becomes a moon of the same mass.
6. **Members stay consistent.** The size ladder and the mass fix every member's size by rank, as today. A member the GM named, mined or built on is a kept body: it holds its rank and its mass is taken out of the remainder first (the chain's pins).
7. **A deed is dated.** An edit made "from this date" is an event; before that date the ring is as it was. A postcard from the campaign carries it, as the chain note already rules for deeds.
8. **A measured ring gives opacity, not mass.** For a catalogued zone the mass is worked out from its optical depth and the size ladder, and is graded "estimated" on the card.

---

## 7. The three desks, depth and the phone

| | Explore | Build | Run |
|---|---|---|---|
| The row and the Structure box | Shown. The first change opens a what-if by itself (as TM4 does for worlds) | Shown. A change is kept, one Undo | Not shown |
| Others like this | A try | Kept | No |
| Grips on the chart, the Add hoop | A try | Kept | No |
| Break up, Gather, a coating | A try, with the ruler free to slide forward and watch it fade | Kept, "as it has always been" or "from this date" | No. A ring changed in play is a Build edit dated "from this date" (question 8) |
| "What does not add up", honesty marks, "Known to" | GM only | GM only | GM only |
| What players see | | | The picture. A zone's makeup or a hidden moon is known through sensors and the "Known to" eye, as for any body |

The ring or belt's `rail` entry is the one its parent's card already has; no tab is added.

**Phone.** The row scrolls sideways. The Structure box docks across the full width above the ruler; the profile takes the whole width and a zone is picked by touching it, then dragged with a thumb-sized mark.

**Light on the engine.** A drag writes nothing. The profile is one kernel answer worked out when the box opens, and the five thumbnails are five more of the same, each a few hundred numbers. While a join or a zone moves, the page does the arithmetic (shares and a redraw of one strip); on a pause the kernel gives the exact answer on a scratch branch; on letting go, one change.

---

## 8. The test case: a cult paints Saturn's B ring red

Alex's question, used as the proving case for "implausible is not a limit".

**Is it possible in SSE?** Yes, by the general mechanism, with nothing special for paint:

1. The GM opens the ring, presses the B zone, opens **Made of** for that zone and adds a material from the data sheet (iron oxide, or the red organic sludge that already tints outer worlds), choosing "as a coating" or "mixed through".
2. Colour is a result. It comes from the data sheet's optics for the mix, through the "Look up" block, as every material property must.
3. Or the GM goes the other way: on the zone's colour, **Reach it** ("Reach [red] by adding…") offers the materials that would do it and the amount each needs.
4. The verdict is the "What it would take" card (5.5): the amount, a source, how long it lasts.
5. Or **Pin the colour** with a reason ("the Ringpainters"). It then carries the honesty mark on the GM's view and no amount.

**What the physics would say** (my arithmetic from the catalogued edges in `base-sol`; pigment figures are estimates from memory and **unverified**, since the paper search is out of allowance until 1 Dec):

| | Figure |
|---|---|
| Face of the B ring, one side | about 1.7 × 10^16 m², 33 times the Earth's surface |
| A coat 10 micrometres thick on both faces only | about 3 × 10^11 m³, 2 × 10^15 kg of iron oxide: a body about 9 km across |
| The same coat on every grain's whole surface (the grains tumble and the ring is many grains deep) | about 1 × 10^12 m³, 5 × 10^15 kg: a body about 12 km across |
| Against the rings' own mass (about 1.5 × 10^19 kg, **unverified**) | a few parts in ten thousand |
| How long it lasts | Short on a ring's timescale. Grains collide every few hours and churn their surfaces, so a coat on the faces is folded in quickly; infalling dust darkens what is left. The kernel should give the figure; I would not guess it here |

So: one mid-sized asteroid ground to dust and spread evenly over 33 Earths of area. Far beyond us, plausible for an advanced culture, and it needs upkeep. That is the answer the app should give, as a card with amounts, never a refusal. It also gives the GM three hooks for free: where the pigment was mined, who keeps repainting, and what the ring looks like half faded.

The same mechanism covers the other "slightly crazy" cases without new parts: a ring swept clear in one zone (a hand-drawn gap, pinned, with what it would take to remove the mass), a ring thickened for shade, a belt mined out (How much lowered, with where the mass went), a ring built round a rocky world from a broken moon (Break up), a ring of something that is not ice at all (Made of).

---

## 9. Seams with other tracks

| With | Seam |
|---|---|
| **"Give planetary rings real structure"** (building now) | **Agreed by message, 26 Nov 2026 17:48 UTC.** Its kernel answer is op `ring_structure`, answer `ring_structure_is`, in crate `sse-small`, module `ring.rs`. Ask, as a pure function: `key`, `variant` (0 is the generator's pick), `host {mass_kg, radius_m, j2?}`, `sculptors [{id, name, a_m, mass_kg}]` (moons today, planets round a star later; nothing ring-specific), `ring {inner_m, outer_m, mass_kg?, makeup?, grain_radius_m?, contaminant?, zones? [{name?, inner_m, outer_m, tau?, albedo?, share?, makeup?, source: catalogued or authored}]}`, `samples?`, `window?`. Or by place: `{system, world, variant?, branch?}`. Answer: `inner_m, outer_m, mass_kg`, `zones [{inner_m, outer_m, mass_kg, tau, sigma_kg_m2, albedo, rgb, name?, cause {kind: base, seeded, resonance, embedded, catalogued or authored; sculptor?; m?}}]`, `marks` (edge, dip, gap, wave, each with its sculptor), `profile {lo_m, hi_m, tau[], rgb[]}`, `why[]`. Zone masses sum to `mass_kg`. Moon-made marks are the same in every variant; only the seeded base changes. Authored zones with a share and a makeup are accepted and renormalised. **Not in that job, so left to RB1:** `branch` is reserved and wired only if cheap; thickness (Stir) is not read; the size ladder is one `grain_radius_m`; no editor. This note does not redesign that job's physics. The ring shadows of PR 617 read the same profile and are unaffected; a ring's colour may change freely |
| **Editing** (ED) | Uses its parts unchanged, adds one (Profile). New rows in `handles.json` for the scale class "ring or belt". The editing-grammar check learns the Profile |
| **Three desks** (TM) | Section 7 is this screen's row on each desk. Each new control declares its desks |
| **Conserved chain** (CC) | Section 6. Belts by class, the draining sketch and the roster (chain stage 5) are the chain's; this note only sets what an edit books and where |
| **Real catalogues and the Detail control** (RW, proposed) | Section 10 |
| **Small bodies** (SB) | Members, notables and `make_small_body` are theirs and are used as built. Belt sculpting by planets (RB6) is offered to that thread |
| **Formation screen** | Break up and Gather are events it can show on a world's track later; nothing asked now |
| **Data sheet** | Coatings need optics rows for the pigments (iron oxide and the red organics); the sulphur rows are already on their way for Io |

---

## 10. How this fits the Detail control

The real catalogues note (proposed, PR 629) makes every population a list ranked by size and puts one Detail control on every view to cut it. For "a world" its measure is "diameter of its moons and ring members".

1. **A ring or belt is one population.** The handles here are the knobs of that list: How much and Grains set every rank's size; Made of sets every member's makeup; Where and Stir set their orbits. Change a handle and the Detail readout changes with it ("down to 10 km · 1,840 bodies").
2. **Detail cuts members, never structure.** The profile is the sum of everything too small to show ("gravel is a sum"), so the zones are drawn at every Detail setting, and get finer as the view zooms in, not as Detail rises. Members above the cut are drawn as bodies on top: shepherd moons, moonlets inside the ring, the big asteroids. Question 7.
3. **The box shows the ladder.** At Science depth the Grains strip is the size ladder itself with the Detail cut marked on it, so the GM sees how much of the crowd the cut is hiding.
4. **Kept members always pass the cut,** as that note's "always shown" rule says: anything named, mined, built on, or made by Break up.
5. **Catalogued zones and catalogued members are claims** in that note's sense. Saturn's measured bands are zones with the cause "measured". A measured moonlet is a member at its rank.
6. **The zone list in the box** shows named, measured and hand-drawn zones, then the widest of the rest, with "Show finer". A real ring has thousands of ringlets and the list should not try to hold them.

If the catalogues note is not accepted, only points 1 and 3 change wording; nothing here depends on it.

---

## 11. Questions for Alex

1. **Pick and fine design are both there, layered.** "Others like this" is one press at the top of the Structure box; the profile for hand shaping sits under it; first generation uses the generator's own pick; the moons' marks are the same in every alternative. *Recommend yes.*
2. **Shaping keeps the mass.** Raising one zone thins the others; only "How much" changes the total. *Recommend yes.* It is what keeps the books closed without the GM thinking about them.
3. **Colour is never typed.** You add a material and the colour follows, or you pin the colour and it carries the honesty mark. The app answers "what would it take" with an amount, a source and how long it lasts. *Recommend yes.* It follows the rule that material properties come only from the data sheet.
4. **One new editing part, the Profile** (an amount along a line), added to the editing grammar and usable elsewhere. *Recommend yes.*
5. **Extra mass may come from nowhere, marked.** If the parent's leftover stock cannot cover a bigger ring, the edit still happens and the GM's list says how much has no source, offering "Take it from a moon". *Recommend yes*, in line with "plausible, repeatable and explained, not exact".
6. **Break up and Gather** as two acts (moon to ring, world to belt, and back), each conserving mass. *Recommend yes, built after the basics.*
7. **Detail thins the members, never the sub-rings.** Zones are always drawn and sharpen with zoom. *Recommend yes.*
8. **A ring changed during play is a Build edit "from this date",** not a Run desk act, for 4.0. *Recommend yes.* A Playbook step that fires such an edit can come later.

---

## 12. Packages, in build order

Each is sized for one agent. Code on Opus.

| Package | What | Needs | Owner (proposed) |
|---|---|---|---|
| **RB0** | This note, the status file, the row in START-HERE | | this thread (done) |
| **RB1** | Kernel: a ring or belt's inputs as authored fields (edges, mass by group, makeup, size ladder, stir, `variant`, zones), read by `ring_structure`'s place form; `branch` on it if not already wired; the size ladder and thickness read; its own golden area | the ring structure job merged; Alex's yes | the ring structure thread, or kernel |
| **RB2** | The row and boxes: handle sheet rows for "ring or belt" (Where with two marks, How much, Made of, Grains, Stir), walls and reference dots, typing, "Type it all in"; Explore tries, Build keeps | RB1 | planet and star editing |
| **RB3** | The Profile part in `ui-contract` and `views/editing`, read-only first: zones, cause tags, the moons' axis; then joins and heights with the trade that keeps the sum; the grammar check learns it | RB1; question 4 | planet and star editing |
| **RB4** | "Others like this": five thumbnails from variants, pencil preview on the world, pick as one change; Kinds; the Add hoop and edge grips on the chart | RB2, RB3 | planet and star editing, with the chart room |
| **RB5** | The books: "How much" as a claim on the parent's stock or a named body; the overdraft line in "What does not add up"; Break up and Gather | RB1; questions 5 and 6 | kernel |
| **RB6** | Sculptors: authored zones against moons (verdicts with "lasts about"), Reach it adding the moon for a gap or an edge, the three "Could be" records; planets sculpting belts through the same answer | RB3, RB5 | small bodies, with editing |
| **RB7** | Coatings and the "What it would take" card: a zone's own makeup, coating or mixed through, colour as a result, Reach it on colour, "from this date" with the fade. Proven on the red B ring, a generated ring, and a belt | RB3, RB5; pigment optics rows on the data sheet | editing, with the data sheet |
| **RB8** | The Detail fit: the size ladder with the cut on the Grains strip, kept members, the zone list's "Show finer" | RW4, RW5 (if accepted) | small bodies |
| **RB9** | `base-sol`: Saturn as one ring with measured zones, the other giants' rings the same way, masses graded estimated; belts' masses come with RW7 | RB1 | data sheet |

RB1 to RB4 give a GM a ring they can pick and shape. RB5 to RB7 make it honest and let the red ring be painted. RB8 and RB9 follow other tracks.

---

## 13. What I checked, and what I did not

- **Read on main at 5023104:** `leftovers.rs` (the `Cloud` record, how a ring gets its mass), `packs/base-sol` regions, `renderer/src/ring-structure.ts`, the editing design and status, the three desks note, the conserved chain note, the real catalogues note, the small bodies status.
- **Not run:** no code was run or changed. The mock-ups are sketches with stand-in figures.
- **Arithmetic:** the B ring's area is from `base-sol`'s edges. The pigment thickness, its density, the rings' total mass and every "lasts about" figure in the sketches are from memory or invented for the sketch, and are unverified.
- **The seam** with the ring structure job was agreed by message (section 9). Its op was not on main when this was written, so the names are that thread's word, not read from code.
- **Not designed here:** how a ring is lit and drawn (the ring structure and shadows threads), families and the roster inside a belt (the chain), sites on a ring member.

## 14. Picking this up without the chat

State of the track: `handover/status/ring-and-belt-editing.md`. Nothing is built. No builder starts until Alex answers section 11; his answers go in a section 11A here and in `decisions.md`.
