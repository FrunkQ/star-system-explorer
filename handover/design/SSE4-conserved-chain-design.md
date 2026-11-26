# SSE4: the conserved budget chain (track CC)

Status: **ACCEPTED by Alex, 26 Nov 2026 14:52 UTC** ("Yes to all - make sense"), with his rulings in sections 1, 10 and 11, which win where they differ from the rest. Design only; nothing here is built by this note. What must be settled before any build is in section 13; the stages are in section 9. CC is a code only, never shown in the product.

Written by the thread "Design the conserved budget chain" (Fable, design only) on 26 Nov 2026, from Alex's brief of 13:55 UTC and his messages up to 14:52 UTC. The working copy Alex reads and comments on is the document https://claude.ai/artifact/G3kghHdZC91GEK2smFVmxr; this file is its text at acceptance. Times in the text are London time, which was UTC on that day.

Read with: the audit "Grand zoom: where the joins stand" (https://claude.ai/artifact/Mh4vmc9CB4c36j5pSThCbn, 26 Nov), `SSE4-design-handoff.md` sections 5 and 6.1 (the rule this note completes), `SSE4-galaxy-design.md` sections 8 and 9, `SSE4-grand-zoom-design.md` section 6 (the tie-up of places), `SSE4-system-formation-design.md` section 6, `SSE4-postcards-design.md` section 12A.

**The standing rule this note adds (build plan section 1):** no join between levels passes a description without an amount, and every join has three tests: closure, indifference to the order and date of looking, and no stand-in place.

## 1. Status: accepted by Alex, 26 Nov 2026, 2:52 PM

"Yes to all - make sense." Every question in this note (1 to 14) is answered yes, with the changes recorded in the sections "Alex's additions", "Local sets" and below. Where an earlier section differs from a later ruling, the ruling wins.

- **Question 3, postcards.** "We can only do what they do." If the sender has changed their universe, or the view shows a vehicle or ship, the receiver's device likely cannot rebuild it. The card warns the receiver and still sends the picture. So a card always carries its still, the live view is rebuilt from the seed where it can be, and the deeds on the card's chain are sent when they are small. No promise beyond that.
- **Question 5, existing universes.** "Nothing we have now is important - you can freely blow away the universe. No one is using it. We have no legacy to protect." So stage 3 need not preserve anything recorded in today's alpha universes, and no old star code is kept alive. The algorithm version is still stamped from then on, for the day there is a legacy.
- **Question 8, interest.** "Keep it REAL and physical." A GM viewing any map gets a **"What is interesting here"** button that lists the odd and rare things at that level, each with its physical reason. His example: "Planet X has a massive platinum deposit after an old planetary core collided."
- **Fudge.** Plausible, repeatable and explained, not exact (2:45 PM). This is what-if territory.

### "What is interesting here": is it doable?

Yes, in three steps of rising reach.

1. **Now, on what exists.** A system's formation already records its shattered worlds, bared cores, giant impacts, moon-making collisions, captured and lost bodies, and the kernel already lists bared cores as notables with a reason. A first list can be built from those records alone.
2. **With belts by class and the roster (stage 5).** The list gains traced things: which crater came from which body, what it was made of, what loosened it.
3. **With the rule for where elements go (question 12).** Only then can the list honestly say "platinum". A core fragment is rich in platinum and gold because those metals followed the iron down when its parent melted. Until that rule is in, the line reads "a large metal deposit from an old planetary core", which is true and still a good hook.

The list is ranked by rarity, using the measure the query engine already has, and each line links to the thing and to its "why". It is the same list at every level: a galaxy's might hold a recent merger, a stream of torn-apart stars, a young cluster about to lose its first star to a supernova. On the Run desk it shows only what the players have been granted.

## 2. The verdict

Yes, the idea works, with one change that matters: **nothing is debited when you look. Everything is allocated by the seed, and looking reveals it.** The rest of your picture stands: named stubs, one mechanism at every level, budgets split by category, causes traced back from a crater.

The change is needed because a true debit depends on what you looked at first. If looking at asteroid A lowers the metal left for asteroid B, then B is a different rock on a device that looked at B first. That breaks "same seed, same universe", the golden tests and postcards. So the share each child gets has to be fixed by the parent's seed and address before anyone looks. Looking then works out a share that was already set aside. The "budget remaining" figure still falls as you explore, and it is still true, but it is a readout and never an input.

You can't tell the difference from the chair. It is "the answer was always there" instead of "the answer was made when I looked", and the first is the only kind two devices can agree on.

Four more things I would change or sharpen:

1. **A budget is a history, not a number.** Every object is a whole life line (made, changed, ended) and every movement of mass is a dated transfer. Any moment on the slider is a read of those. This is what answers your time worry.
2. **Causes are never invented backwards.** The passing star, the comet shower and the impact all exist in the sketch from the start, each keyed by the seed. Looking at the crater reads that chain upstream. If the cause were made up at the moment of looking, two craters could invent two stories that disagree.
3. **The two ends of an event must be the same event.** Today a crater is drawn from an impact rate and a belt member from the belt, with no link. The design needs one keyed roster per belt that says which member leaves, when and to where, read forwards by the crater and backwards by the asteroid.
4. **Only what the GM or the players do is a true stateful change.** Mining an asteroid out, pinning a star, adding a merger: these are dated events in the campaign record, they change things from their date on, and they travel with the campaign.

Most of this is not new. The design handoff of 18 Nov states it as rules (section 6.1, rules 2, 3 and 6), and the crate `sse-resolve` implements it with tests. Belts and the inside of a system use it. It was dropped at two joins when the live star code replaced the first patch, and it was never extended to events or to composition. So this is a repair and a completion, not a rewrite. My estimate is that about four fifths of the physics code stays as it is.

Nothing in the repo was changed for this note. I read the code and the notes; I did not run anything or measure the size of any mismatch.

## 3. The one mechanism

Every level, from a web cell to a boulder field, is the same four things. A builder who learns it once can apply it anywhere.

| Part | What it is | Cost when nobody is looking |
| --- | --- | --- |
| **Reservoir** | A parent's stock at each moment, by category, in whole units. A ring's gas and stars by six element groups; a belt's mass by class, size band and element group | A few numbers per time step. This is your light sketch from year 0 to the end |
| **Split** | The rule that divides a reservoir among its children, drawn from the parent's seed and address alone, so child 7 never needs children 1 to 6 | Nothing until asked; then one cheap sum |
| **Stub** | A child nobody has looked at. It is an address, which gives it a name, a place and its share. Nothing is stored for it | Zero bytes. It can always be worked out again |
| **Flow** | A dated transfer between two reservoirs: gas into stars, a dead star's ejecta back into gas, a belt member onto a planet. Keyed by its two ends and its time bin, so both ends see the same event | A rate per time bin |

Looking at, naming or visiting a stub **resolves** it: the proper maths runs, the child gets its whole life line and its own reservoirs for the level below, and the result is recorded so it never shifts. The parent's "remaining" readout is its total minus its recorded children, category by category.

Three rules hold everywhere and each gets a test:

- **Closure.** A reservoir's children, plus what is unresolved, plus what has flowed out, equal what it started with plus what flowed in. Exactly, in whole units, at every moment.
- **Indifference.** Resolving things in any order, on any date, from any screen, gives the same universe. `now` is never an input to a resolve.
- **One event, two ends.** Anything that leaves one reservoir arrives in another as the same keyed event, with the same mass, date and makeup.

### Categories, and your asteroid belt

You are right that a belt is not an average. Today it nearly is: a belt's total mass splits exactly into its members by size rank, but each member's makeup is a random scatter around the belt's mean (`sse-small/member.rs`). So ten resolved iron asteroids do not lower the iron left, and the members' metals need not add up to the belt's.

The fix is one more level of split, made once when the system forms. The belt's mass by element group is divided into **classes** that formation already knows about: fragments of cores, of mantles and of crusts from worlds the violent phase broke, primitive bodies that never melted, icy bodies from beyond the snow line. Each class has its own budget by element group, its own size law and its own members. A metal asteroid is a member of the core-fragment class and its mass comes off that class. The class budgets add up to the belt's, group by group, so the books close.

The spectral letter (M, S, C) stays what it is today: a reading taken from the makeup, never a second source.

## 4. Time

Your leaning is right: sketch the whole timeline lightly, add detail on looking. What makes it hold together is that a resolved thing is never a snapshot of the moment you looked. It is its whole life.

**A resolved object is a life line.** When you resolve a star you get its birth, every phase and its end in one go (`sse-formation/lifeline.rs` already does this, and says so in its first line). An asteroid gets where it formed, when it was knocked loose, and when and where it ended. The date you happened to be looking at is not recorded as anything physical. The chronicle already keeps three separate times for every event (when it happened, when its effects begin, when the session noticed it) and the handoff forbids the third from touching physics.

**A debit is dated when it physically happened.** Take your crater. You look at it today. The impactor's mass does not leave the belt "today" or "when you looked". It left the belt at the date the roster says it was knocked loose, and arrived on the planet at the impact date. Slide back before the impact and the crater is gone, the planet is lighter by that rock, and the rock is on its orbit in the belt, where you can go and look at it. Slide back further and the passing star has not arrived yet. Every one of those is a read of the same record.

**The sketch is the reservoirs and their flow rates through time.** Much of it exists: each ring's gas, stars and element groups in 50 Myr steps; each star's life line; a system's formation history as stored slices; a world's impact chronology. What is missing is the same sketch for small-body reservoirs (a belt and a comet cloud draining over billions of years), and the rule that an event rate is the outflow of a reservoir. Today a world's cratering rate is the Moon's curve scaled to the world, plus spikes from the violent phase. It is not tied to how much the system's own belt and cloud hold.

### Where time stays hard

I would be misleading you if I said every case closes.

- **The anonymous star field over deep time.** Stars drift tens of parsecs per million years, so "the stars near here" is a different set in each era. The galaxy design (section 8.3) chose this rule: a looked-at star is permanent, and everything else is redrawn each time frame from that era's statistics. So if you slide 50 million years, the stars you had not looked at are not the same stars moved. They are an honest redraw with the right numbers, and the design labels them so. Total mass still closes in every frame. Following every star's orbit through all time would remove this limit, and I do not think a browser can afford it. This is the one place where the full promise ("the same universe at every time") is not met for things nobody has looked at.
- **A past edit.** If the GM adds a merger five billion years ago, things already resolved after it stay as they are. They are checked against the new history and flagged if they no longer fit. The galaxy design already rules this.
- **The far future.** I did not check how far forward each ledger's sketch runs. The star life lines and the cosmology run to the end; I believe the rings stop at the campaign's present. "To the end of time" needs each reservoir to have a rule for coasting forward.

## 5. Same seed, same universe

There are three ways to keep the books. I recommend the third.

|  | A. Allocated by seed | B. A true ledger | C. Seed for nature, ledger for people (recommended) |
| --- | --- | --- | --- |
| How a share is set | By the parent's seed and address, before anyone looks | By what is left when you look | As A for everything generated; as B only for what the GM and players do |
| Does looking change anything | No | Yes, for everything not yet looked at | No |
| Same universe on every device | Yes, from the seed and packs alone | Only if every device replays the same looks in the same order | Yes for the universe; a campaign also needs its record of deeds |
| Golden tests | Work as now | Need a scripted order of looks; any new screen that looks at something moves them | Work as now |
| A postcard carries | Seed, packs, address, date | The whole ledger up to that look | Seed, packs, address, date |
| Can children react to each other | Only through what the split encodes | Freely | Only through the split, plus dated deeds |
| Cost | Each level needs a split that is cheap and can be read at any child | Simple to code, level by level | As A |

**What A and C give up.** A child cannot depend on its siblings' details. Asteroid 40 cannot be "whatever metal the first 39 left". Its share is fixed by the split. This is a real constraint on the maths at each level, and it is why the split is the hard design at every join. Three splits already in the code show it can be done cheaply: a size-rank law for belts (the k-th largest has a closed form), a tree that halves a region's star count down to 10 pc cells, and draw-until-full for a patch's birth mass.

**What C adds over A.** People do change the universe, and their changes are ordered. A ship strips an asteroid; the GM pins a star where there was none. These are dated events in the campaign record, as edits already are. They take effect from their own date, never earlier. A pin claims its share first, and the generated children give way from the tail end, so pinning never reshuffles what was already there. If pins ask for more than the parent holds, they are honoured and the parent is marked as holding an impossible input. `sse-resolve` does this today.

### Two places where looking does leave a mark

To be straight about it, C is not perfectly clean.

1. **A looked-at star outlives its time frame.** Inside the frame it was drawn in, a promoted star is identical to the stub, so looking changed nothing. In a later frame it is still there, in place of one anonymous star. So after a deep-time slide, two GMs with the same seed who promoted different stars see neighbourhoods that differ by those stars. I recommend treating this as it is: any star can be worked out again by anyone from its address, and its staying put across frames is a fact of the campaign.
2. **Recorded results are kept even though they could be recomputed.** That is on purpose (handoff 5.5): if a later version of a pack changes a formula, what a campaign already resolved does not shift under it.

### The tests that would have caught this

The breaks in the audit happened because nothing failed when the budget rule was dropped. Three standing tests per join would stop it happening again: a closure test (children plus remainder equal the parent, in whole units), an indifference test (resolve a sample in two orders and at two dates, compare the state hash) and a no-stand-in test (already ruled on 25 Nov: a test fails if any op falls back to a stand-in place). Per the house rule, each is first run against the present code and seen to fail.

## 6. What is saved, and how it travels

Under the recommended option the universe itself costs nothing to save. It is the seed and the list of packs. Only what people did, and what they have looked at, takes room.

| Layer | Holds | Can it be rebuilt | Size (my estimate, not measured) | Travels with |
| --- | --- | --- | --- | --- |
| Universe | The seed, the pack list with their hashes, the algorithm version | It is the source | Under 1 kB | Everything: a campaign, a postcard, a link |
| Deeds | Every GM edit and play event, dated: pins, renames, a mined-out asteroid, an added merger | No. This is the only part that cannot be worked out again | Tens of bytes to about 1 kB per deed | The campaign |
| Resolved record | Each looked-at thing's facts and life line, so it never shifts | Yes, from the two layers above with the same packs | About 0.5 to 2 kB per star or small body; a formed system is larger | The campaign |
| Gazetteer | Names and places at the campaign's date, ready to hand (ruled 25 Nov) | Yes | Small | The campaign, as a cache |
| Derived cache | Textures, tiles, flow results | Yes | Large | Nothing. Never saved |

A campaign that has charted a few hundred stars, formed twenty systems and named a few thousand small bodies and events should come to a few megabytes before compression, and well under one after. A stub is not in the file at all. Its name comes from its address through the naming track, so every galaxy in view is named while only the looked-at ones take room.

**Postcards.** Your decision that the seed travels with a postcard is what makes this cheap, and it only works under options A or C. A postcard stays what was ruled on 25 Nov: an address, a date, a camera and named packs, well under 1 kB. The receiver's device works the view out from the seed. Under option B the card would have to carry the sender's ledger.

One consequence to decide (question 3 below): a postcard then shows the **universe**, as a fresh visitor would find it, not the sender's **campaign**. A crater's impactor and its passing star arrive intact, because they come from the seed. An asteroid the sender's players mined out arrives whole, unless that campaign or world is published and the card names it. That matches the existing ruling that an authored world is published before a card can show it.

**Two devices at one table.** The players' page receives the rows it is allowed to know as data, as now. Nothing here changes that; it only guarantees that if a player's device does work something out for itself, it gets the GM's answer.

## 7. Conservation, join by join

The three ledgers that close stay as they are. The work is at the joins between them, and at two breaks the audit did not list. Top of the universe first.

| Join | Today | The fix | The hard part |
| --- | --- | --- | --- |
| Web cell to its haloes | Closes | Keep | None. The cell's report uses a typical halo per mass (fudge F48); leave it |
| Web to our galaxy | Not joined: a typical halo placed at the origin | Home is chosen from the web's own catalogue when a universe opens. Already ruled on 25 Nov (GZ12) and first in the kernel's queue | Moves the core golden once, on purpose |
| Halo to galaxy to rings | Closes, per ring per 50 Myr, six element groups | Keep | See the note on rates below |
| Ring to a place | The arm and the local texture multiply the gas and take it from nowhere | Make both a reshuffle: more gas in the arm means less between arms, the ring's total unchanged | Small. One normalising factor per ring |
| Place to stars | A random count from the local density; no mass claimed | Hand the ring's star mass down the existing tree by mass, then fill each 10 pc cell with stars until its share is used. A pin claims mass, not a slot | Every unlooked star changes. See the staged path |
| Star to its system | The disc's mass is a share of the star's, taken from nowhere. Only "Grow a system here" inherits even the recipe | A system is one named line in a debit the ring has already made: the ring turned so much gas into stars in that step, and this system is so much of it. Star, disc and what stays as planets come from one birth claim, in the six groups | Two recipe items (magnesium to silicon, aluminium-26) are constants and should come from the place. A star slot for every system is already ruled (GZ13) |
| Inside a system | Closes exactly, with a record of where each part came from | Keep | None |
| Belt to its members | Mass closes; makeup does not | Classes with their own budgets (above) | The class shares must be worked out from what formation did, group by group, without going negative |
| Belt and comet cloud to craters | Not joined. Craters come from a lunar-type curve | Craters are the arrivals of the system's own reservoirs, through the roster (next section) | The largest piece of new design |
| Stars to their deaths (not in the audit) | A cell's supernovae are drawn from a rate. The cell's massive stars are drawn separately. They are not the same objects | A core-collapse supernova is the death of a particular massive star of that birth group, so the star you charted is the one that explodes | Slow kinds (type Ia, neutron-star mergers) go off billions of years after birth, far from where they were born. Those stay drawn from a rate, with the dead pair made to fit, and the fudge is registered |
| Stars to close passes (not in the audit) | The passing star's mass is a fresh draw. It is not a star of the field | Inside the current time frame a close pass is worked out from the real neighbours' orbits. Further back it is drawn from the rate, as now, and creates its star for that era | Cost of the neighbour search; bounded by the cell size |
| Deaths back to the gas | The ring already takes back what dying stars return, in bulk | Keep the bulk entry. A resolved death is a named line in it, as a birth is | None for mass. No state is needed |
| Events to a system | The code exists; nothing in the app calls it | Wire it: every campaign system reads its site's event history from its own star slot | Needs the star slot first |

### Clearing the gas round a star

Your test case from the audit can be met without any stored state. The gas at a place becomes the ring's value, times the arm reshuffle, times the dents that nearby events make: a supernova's bubble for about a million years, a young cluster's cavity. Those events are keyed by the seed, so the dent is the same for everyone and nothing needs saving. What is pushed out of a bubble is piled on its rim, so the total holds.

### A conserved chain carries mistakes faithfully

The Local Arm thread found this morning that the inner galaxy's gas comes out at +0.5 to +0.8 in iron, because the rings' radial winds were tuned to measurements beyond about 5 kpc and keep steepening inward where the real gradient flattens. That ledger closes perfectly and is still wrong. Once every star and disc claims its recipe and amount from its ring, an error like that reaches every inner-galaxy system as too much dust, too many giant planets and too much ore. So each ledger needs checks against measured things as well as closure, and the upstream rates should be right before the downstream joins are tied. That thread's fix is on the path of this design, and I would let it land first.

### What I would leave out

Mass and the six element groups are the chain. I would carry angular momentum across one join only (birth cloud to disc, where it sets the disc's size) and not build an energy ledger across levels. The audit found none, and I can't name a question a GM would ask that needs one.

## 8. The magic trick: from a crater back to a passing star

The trick works if the story runs forwards in the model and is only read backwards on screen. Here is your example as the design would hold it. Every step exists from the moment the universe is opened, as a keyed entry nobody has worked out yet.

1. **A star passes.** The system's close passes are a series keyed by the system's address and the time bin. One pass comes inside the comet cloud. The code for this exists (`sse-cosmos/events.rs`) and already marks such a pass as a shower. Change: the star is a real star of the field where it can be, not a fresh draw.
2. **The comet cloud is shaken.** The pass raises the cloud's outflow for the next few million years. The size of the rise comes from the star's mass, speed and distance. New.
3. **Members leave the cloud.** The cloud's roster says which members leave in each time bin and where each goes: onto which world, into the star, or out of the system. The raised outflow means more names are called in those bins. New, and the heart of it.
4. **One of them hits the planet.** It is a particular member of a particular class, so it has a size, a makeup, an orbit and a fall time. Its mass leaves the cloud at the date it was knocked loose and joins the planet at the date it lands. New join; the crater's size, place and shape rules exist (`sse-surface/impacts.rs`).
5. **The crater, and what it left.** The site gets the impactor's makeup: a core fragment leaves metal. The ore is a line in the planet's own record of where its parts came from, which exists inside a system today.

When you look at the crater, the app reads 5, 4, 3, 2, 1 and tells it as one sentence with dates. The chronicle's events already carry a `causes` field for this, and every surface feature already carries a "why is this here" line. What changes is that the line is traced, not written.

### The roster

This is the one new mechanism, so it is worth saying how it stays cheap and order-free.

A reservoir with millions of members cannot be walked to find who hits which world when. And if craters were drawn from a rate on the planet's side while members' fates were drawn on the belt's side, the two would never be the same objects, which is today's state. The roster is a seeded shuffle of each class's members within a size band. The sketch says how many leave that band in each time bin and how they are shared among the sinks. The shuffle says which members those are. It can be read both ways at the cost of a few sums: "the third arrival on this world in this bin is member 48,112" and "member 48,112 leaves in this bin for this world". The crater reads it one way and the asteroid the other, and they meet at the same event. Nobody has to look at members 1 to 48,111.

The same shape serves a ring's star births, a belt's collisions and a cloud's comets. It is the tree that already shares a region's star count down to its cells, with a shuffle on the end.

### What the trick will and will not claim

- **It claims** a consistent, repeatable history in which the masses add up, the dates respect travel time, and the same crater tells the same story on every device.
- **It does not claim** that this is what a full simulation of ten billion comets would have produced. The handoff has a label for exactly this, "consistent reconstruction", and it should show on the card.
- **Where the chain ends.** A cause is traced until it reaches a reservoir's ordinary background rate. Most craters will say "a stray from the belt, loosened by the giant's resonance", because that is the truth of it. The passing star is the rare, good story, and it is rare for a physical reason: a pass inside the comet cloud's reach happens on the order of once per tens of millions of years. I would not turn that rate up to make the story more common.

## 9. What is kept, and the path there

Your feeling that it grew by goodwill is half right. The architecture was written down and built first; then the screens were built from a system upwards while the kernel was built from the web downwards, and where they met, stand-ins went in so both could keep moving. The stand-ins are each in the fudge register. What was missing was a test that fails when a join passes a description without an amount.

| Code | Verdict | What changes |
| --- | --- | --- |
| `sse-resolve` (envelopes, splits, pins, the record) | Keep as is | It gains the roster and a class level as general tools |
| `sse-cosmos` web, haloes, galaxy, rings | Keep | Checks against measured gradients; the wind fix from the Local Arm thread |
| `sse-cosmos/patch.rs` (the first patch, unused by the app) | Revive | It is the birth-group level the live path skipped |
| `sse-cosmos/star_tree.rs`, `stars.rs` | Keep the tree and the orbits; change what the tree shares out | Mass in place of a count; pins claim mass |
| `sse-cosmos/envelope.rs` (arm, texture, local gas) | Keep; normalise | Reshuffle in place of multiply; dents from events |
| `sse-cosmos/events.rs` | Keep its keyed-by-cell design | Massive stars' deaths come from the stars; close passes from the field; wired to systems |
| `sse-formation` (disc, growth, violent phase, leftovers, life lines) | Keep | The birth claim comes from the place; belts gain classes and a draining sketch |
| `sse-small` (a belt member up close) | Keep | Makeup comes from the member's class, not a scatter |
| `sse-surface/impacts.rs` | Keep the crater rules | Arrivals come from the system's reservoirs through the roster |
| `sse-chronicle` | Keep as is | Its `causes` field starts being filled |

### Stages

Each stage is a few work packages, one at a time, checks green before the next. Alex's additions fold in as follows: clumps with stage 3 (clusters) and stage 5 (families), notables and gravel-as-a-sum with stage 5, the settled-state rule as a check in stage 1 and a rule for every later stage, and the GM's list of interesting things with stage 7. None needs the app taken down.

1. **Measure, change nothing.** Add the closure and indifference tests in a report-only form and a ledger read-out for each join, and record how far off each one is today. No golden moves. This tells us how big the star-field change in stage 3 really is, which nobody has measured.
2. **Finish the tie-up of places** already ruled on 25 Nov: home from the web's catalogue (GZ12) and a star slot for every system (GZ13). Core golden, already queued. Let the Local Arm thread's metals fix land in the same window.
3. **Tie mass through the middle.** Ring to stars by mass, star to system as a named birth claim, the two missing recipe items. This is the big golden move: every unlooked star changes.
4. **Wire events to systems,** with massive stars as their own supernovae and close passes from the field.
5. **Belts by class, the draining sketch and the roster.** Craters become arrivals. The crater's "why" becomes a traced chain.
6. **Local gas as a reshuffle with dents,** and deaths as named lines in the ring's returns.
7. **Show it.** A budget read-out on each level's existing card (total, resolved, remaining, by category) and the cause chain on a feature's "why". No new screen, so no new rail entry.

### Not breaking the alpha

- **What is recorded stays.** A campaign's promoted stars, formed systems and resolved bodies are in its record, and the record wins over a recomputation. They become pins in the new split.
- **What was never looked at will shift once,** at stage 3, for every seed. I recommend accepting that during alpha and saying so in the alpha notice, over keeping the old star code alive beside the new. The handoff's stream versions (`planets.count.v1`) and the run manifest's algorithm version are the tool for freezing old universes later, and stage 3 is a good moment to start stamping that version into every saved universe.
- **V3 imports and hand-built systems** keep their authored facts. They gain a star slot and claim their mass as pins; where they overdraw their birthplace the card says "unusual for its birthplace", as already ruled.

## 10. Alex's additions, 26 Nov, and what they change

Alex read the sections above and added eight points (2:33 PM). They agree with the design and sharpen it in five places. Where this section differs from the text above, this section wins.

### How this differs from the original design

The core is the same as the design handoff of 18 Nov: shares set up front from the parent's seed, pins first, exact closure, one life line per object. What this note adds or changes:

|  | Original design | This note |
| --- | --- | --- |
| Composition | Budgets by element group at each level | The same, plus classes inside a reservoir, so a metal asteroid draws on a metal stock |
| Events | A rate at a place; an event changes the neighbours' stored state from then on | An event is one keyed entry with two ends. Nothing stored: its effect on a place is worked out from the seed |
| Rates and stocks | Separate. A cratering rate and a belt's mass are unrelated numbers | A rate is the outflow of a stock. The roster links the two ends |
| Causes | Each event has a `causes` field | A rule for filling it: forwards in the model, read backwards on screen |
| Looking and doing | One word, "resolve", plus edits | Looking reveals and never changes. Only deeds change |
| Guard | A one-off proof at the start (budget closure on samples) | Standing tests at every join, so a join cannot drop the rule quietly again |
| Scope | The rule stated once, for all levels | Each join's split written out, with what it costs |

So the original was right and thin. It gave the rule and left each join to its builder, and two builders took a shortcut.

### 1. A split must be clumpy

Agreed, and it corrects something above. An even or smoothly random split is wrong physics. Stars form in clusters, and belts hold families from particular break-ups. The galaxy design already asks for this (section 8.1: stars born in clusters, each cluster's mass shared out among its stars, siblings sharing a recipe). The live code only gives each star a cluster mass as a label. No two stars are in fact siblings.

So every split has a clump level: ring to clusters to stars, and belt to families to members. A clump is a child like any other, with an address, a share and a name. This also serves the story: "these five stars were born together" is traced, as a crater's cause is.

### 2. Special things exist before anyone looks

Your Psyche case is in the code today, in exactly your form. When the violent phase shatters a world, the bared iron core is the largest member of that impact's debris, and it is listed as a **notable** with its reason before anyone resolves it (`sse-small/member.rs`, `notables`).

The design makes that general. Any event in a reservoir's history may leave a notable: a bared core, a captured moon, a runaway star thrown out of its cluster, a comet on its first fall. A notable is a full object with its own claim on the budget, shown to the GM and hidden from players until they find it. Notables come from events, so they are as rare as the physics makes them, and a prospector's chance of finding one among the S-type debris is real.

### 3. Gravel is a sum

Agreed, and simpler than what I wrote. Below a size worth tracking, nothing is ever resolved by rank. Mining "some asteroid" is a deed that draws a stated mass from a class's unresolved stock, by element group. If the players should be able to return, the deed also creates an object ("mined asteroid", or the name they give it), which is a pin that claims its mass from that class. This is the route by which small things become real.

It is also the precise statement of your rule: **two devices with the same seed are identical until someone interferes, and differ only by their deeds.**

### 4. Gold, at any level

Yes. The six groups (`alpha`, `fe`, `cn`, `s`, `r`, `k40`) are each a shared way of being made, so any single element is its group's amount times that group's fixed pattern. Gold is in the rapid-capture group. "How much gold is in this belt, system, ring or galaxy" is then a read of the ledger at that level, with no extra accounting. The honest limit: it assumes each group's internal pattern is the same everywhere, so it is a sound estimate and is labelled as one.

### 5. Pointing at the sky

Yes, by two routes that already exist. A star in the sky is found through the same tree and the same keys as the star in its cell, so the star you point at is the one you arrive at. A galaxy in the sky is a catalogued halo of the web, with its own address, so it can be named and pinned. The audit's "a galaxy is in two places" (about 7 Mpc apart) has to be closed first; that is package GZ15, already ruled.

### 6. The grand zoom

Yes, this is what makes the zoom safe to build. Every level's children have an address and a place before they are drawn, so the journey can read ahead and nothing can turn out differently on arrival. One caution: "set" means fixed, not already worked out. The first approach to a new galaxy or system still costs its resolve, which the journey has to hide by reading ahead.

### 7. A storytelling tool: show the GM what is interesting

This was missing above. When anything is resolved (a galaxy, a system, a belt), the result carries a short list of its notables and its rarest facts, each with its one-line reason, shown to the GM at once. The query engine already ranks outcomes by how unusual they are. So the rates stay real, and the rare things are surfaced, not made more common. That answers question 8 better than my text did.

### 8. Edits, and postcards

You are right that a past edit ends "everyone sees the same" for that campaign, and that a postcard must then say what changed. Deeds are small, so the fix is cheap: **a postcard carries the deeds that lie on its own chain of places** (its galaxy, ring, system, world, site, and the thing itself). For most cards that is nothing. For a card from an edited campaign it is a few hundred bytes, and the receiver's device applies them before drawing. A deed that needs an unpublished authored world still needs that world published, as ruled.

### 9. Every timeline runs to a settled state

Agreed, and it replaces my question 9. The rule: **every life line ends in a settled state, after which it is a formula.** A system runs through its star's swelling and death, the loss of the inner planets, the spreading of the orbits and any reshuffle of the survivors, until nothing more will change for billions of years. From there, only cooling is worked out, by one sum for the date asked. A galaxy fades as its stars die until it goes dark, and then holds. Nothing is simulated past the point where nothing happens, and little is stored for it.

Ageing the Solar System this way was the acceptance scenario of the original design (handoff 6.3). I have not checked how much of it runs today; stage 1 should find out.

### 10. Following the local set through its whole life

Alex, 2:37 PM: can the travel of a system and its local stars, and the same for the local group of galaxies, be simulated through their lives? Yes, for a bounded set, and cheaply.

- **Stars.** A permanent star's path round the galaxy is a formula, not a simulation (`stars.rs`: a guiding circle with a radial and a vertical wobble). So the home system and, say, the thousand stars near it at the campaign's date can each be placed at any moment from birth to death for the cost of a thousand small sums. That set becomes **the tracked set**: promoted together when a campaign is made, and the same stars at every time.
- **Galaxies.** The Local Group is a few dozen bodies. Each halo's merger tree already fixes who falls in and when, and the members already carry their own orbits. Following them from first infall to the last merger is affordable on any device.

What this does and does not buy:

- You can ask where any tracked star was or will be, watch the home system's companions drift apart, and get close passes between tracked stars from the paths themselves.
- Today's neighbours are not yesterday's. Stars near each other now differ in speed by tens of kilometres a second, so 100 million years ago they were spread across thousands of parsecs. The tracked set tells you where your neighbours went. Who was beside the home system back then is mostly other stars, and those are still the era's redraw unless they are tracked too. The set can be widened by promoting the neighbours at a few chosen eras, at a cost that grows only with the number of stars.
- The paths are smooth approximations. Over billions of years real stars are nudged by arms and clouds, so a tracked path is one plausible path, repeatable and labelled.

So the deep-time limit shrinks from "all unlooked stars" to "stars outside the tracked set". I recommend it as part of stage 3.

### Where the questions stand

Taken as answered by this message: 1 and 2 (yes), 3 (changed: a card carries the deeds on its chain), 4 (yes, deterministic now and refined later), 8 (real rates, with the interesting shown first) and 9 (every level, to a settled state). Still open: 5, 7, 10 and 11, and the three new ones (12 to 14). Question 6 was answered at 2:43 PM (see "How much fudge").

## 11. Local sets: my planet, my star, my galaxy, and their neighbours

Alex, 2:43 PM: three levels are special because their members can truly interact. My planet and the local planets; my star and the local stars; my galaxy and the local galaxies. Each could have a feeder in and a feeder out, run on a beat, to see what runs away and what is arriving. I agree, and it fits the mechanism without a new one.

A **local set** is a small group of real objects that carry positions and velocities and act on each other. The level above feeds it: things arrive through its edge at the rate the parent's stock gives, and things that leave go back to being part of the parent's stock. The beat is how long it takes to cross the set, which differs a great deal by level.

| Local set | Members | What they do to each other | Fed from | Beat | State today |
| --- | --- | --- | --- | --- | --- |
| A system | Planets, moons, the belts' notables | Resonances, scattering, collisions, capture | Its comet cloud and belts; passing stars | Years to millions of years | Built: orbits, the violent phase, late-life stepping |
| A birth cluster | A few dozen to thousands of sibling stars, for the first 10 to 100 Myr | Close passes strip discs, a massive sibling floods the others with light and fresh aluminium-26, one explodes | The ring's gas at that place and date | About 1 Myr | Not built. A cluster is only a label on each star |
| The stars near home, after the cluster has scattered | A few hundred stars within about 10 pc | Close passes, comet showers, a nearby supernova | The ring's star field, through the edge of the bubble | About 0.3 Myr. At 30 km/s a star crosses 10 pc in a third of a million years | Half built: close passes are drawn this way for a small radius |
| The Local Group | A few dozen galaxies | Mergers, stripped gas, stirred discs, starbursts | The halo's merger tree and the web | 100 Myr to 1 Gyr | Mostly built: each tree member has an orbit and its mass is in the ledger |

### How the star-level feeder works without stepping through all of time

Your beat of ten million years suits a birth cluster or a wide stream of stars moving together. For the ordinary neighbourhood it is too long: the whole set changes thirty times over in that span. That does not make it expensive, because of a property the code already has.

A system's close passes are drawn today as a stream keyed by the system's address and the time bin. Each pass has a time, a distance, a speed and a mass, and any stretch of time can be asked for without working out the stretches before it (`events.rs`). **The neighbourhood is the same stream with the radius opened from a twentieth of a parsec to about ten parsecs.** Every star that will ever come within reach of home is then an arrival in that stream, with a path in and a path out. "Who is near home at this date" is the arrivals of the last crossing time or so that have not yet left. A leaver stays a real star on its path for as long as anyone cares to follow it.

This gives what your feeder asks for, and it closes two gaps from above:

- The local map at any date is the same for everyone, straight from the seed. Nothing has to be stepped from the beginning, and nothing depends on what was looked at. The "deep-time limit" disappears for the home neighbourhood.
- A passing star that shook the comet cloud is a member of that stream, so "that star, over there, now six parsecs away" becomes true, where I had called it too far.

Its limits, each to go in the fudge register:

- **It is centred on a home.** Two campaign systems within reach of each other see each other truly, as both are permanent. But an anonymous star in one home's stream is not the same object as the galaxy map's fill at that spot. At the campaign's date the stream wins inside the bubble and the map's fill stands outside it.
- **Mass closes on average.** The bubble is a window on the ring's stock of stars, not a stock of its own. Arrivals match the ring's density and speeds over time; a given bubble at a given date does not have to hold its exact share.
- **Paths are smooth.** A pass bends neither star's path round the galaxy. It acts on what is loosely held: comet clouds, wide companions, outer planets.

### Runaways and strays

Alex, 2:44 PM: the occasional rogue planet to track, and stars losing partners. Both fall out of the same rule: **whatever leaves a local set is a real object on a path, and whatever arrives came from somewhere.**

- A planet thrown out in a system's violent phase leaves with a speed and a direction. It becomes a notable of the neighbourhood, and its mass comes off the system's books and joins the ring's stock of rogues.
- A wide companion prised loose by a close pass leaves the same way. The system records a lost partner with a date and a cause, and the lost star is in the stream outward.
- In the other direction, the neighbourhood stream carries a small share of rogue planets and lone remnants beside its stars, at the rate the galaxy's ejections give. The galaxy design already lists rogue planets as a population. One drifting through the outer system is a rare arrival, and a traced one.

These are the things to put at the top of the GM's list of what is interesting. Alex, 2:45 PM: "we just want INTEREST within the bounds of feasibility and good physics." That is the test for every item in this note: a rate stays real, and the design's job is to make sure the rare thing is found, named and explained when it happens.

### Energy

This also answers where energy goes without an energy ledger across levels. Inside a local set the members carry velocities, so momentum and energy are simply what the motion is. Between levels only mass and the six groups are passed.

### How much fudge

Alex: "we can fudge to a degree, this is party trick territory", and it is fine to say "hold on, that is enough" to someone testing the big bang. Taken as the answer to question 6: exact books from the ring down wherever a thing is named, registered tolerances above that and for anonymous crowds, and the fidelity label on screen says which is which.

Alex, 2:45 PM: "we are not slavish to absolute accuracy for this type of thing, it gets too much. This is what-if territory." So the working rule for runaways, strays, local sets and deep-time stories is: **plausible, repeatable and explained, not exact.** Exact books are kept only where they are already built or nearly free (inside a system, a belt's members, a birth claim). Nothing in this note should grow a new exact ledger to serve a party trick, and where the roster or a split proves costly, the simpler fallback is the right answer, not a failure.

## 12. Where the spec stands: solid, unproven, and too far

The chain of mass and six element groups from the web to a named crater is possible for generated systems, on code that mostly exists. Four parts are unproven, and five things the spec could be read as promising are fudges too far that I would not promise. One gap matters most for your ore deposit.

### Solid: built or a short step from built

| Part | Why I am confident |
| --- | --- |
| Shares set by the seed, pins first, exact closure | `sse-resolve`, with tests; in use for belts and inside a system |
| A life line per object, any date a read | Stars and formed systems do this now |
| Deeds as dated events that travel with the campaign | The chronicle and the edit rules exist |
| Notables that exist before anyone looks | The bared-core case is built; widening it is more of the same |
| Gravel as a sum, with a pin made when players want to return | A draw on a remainder plus an ordinary pin |
| Home galaxy from the web; a star slot for every system | Ruled and queued (GZ12, GZ13) |
| Arms and texture as a reshuffle | One normalising factor per ring |
| Star death, swelling and engulfment on the slider | Stepping through a star's late life exists in `sse-time`. I did not run it, and how far the planets' reshuffle goes afterwards is unchecked |
| Totals of any element at any level | A read of the group ledgers times a fixed pattern |
| A galaxy or star you can point at | Same keys in the sky and in the cell |

### Unproven: possible, but each needs a prototype before a promise

1. **The roster.** It is the heart of the crater trick and exists only on paper. Its cost on a belt of millions, and how it behaves when a deed removes a member that was due to hit something later, need a working prototype. If it fails, the fallback is honest but weaker: only notables and the largest few hundred members of each class are real objects with fates, and smaller impacts are a sum.
2. **Stars by mass, in clusters.** Sharing a ring's star mass down the tree is a modest change. Making it close per birth era needs the tree to split by age first; it already holds the age bins to do so. Real siblings are the hard part, because the star field is drawn by where stars are now, not where they were born. See "too far" below.
3. **Core-collapse supernovae as the deaths of charted stars.** Massive stars die within about 40 Myr and a few hundred parsecs of birth, so this should work across neighbouring cells. Nobody has tried it.
4. **Imported and hand-built systems.** V3 is the one in real use, so most systems people care about arrive with no history. The formation design already says the answer: fit a plausible history around the real bodies as pins and label it reconstructed (section 6). I do not know how much of that fitting is built. Without it, the crater trick works only for generated systems.

### Too far: I would not promise these

| Reading of the spec | Why it is too far | What to offer instead |
| --- | --- | --- |
| Every crater traces to a named impactor | A world has millions of small craters, drawn tile by tile | Trace the listed craters (the large ones the surface code already keeps as a global list). Small ones are gravel: a sum |
| Every star has real siblings and the cluster's books close | Old stars have long since scattered across the galaxy; finding who was born with whom means following every orbit | Young clusters, still together, are real objects with members. For an old star the birth cluster is a described fact, and closure holds by ring and birth era |
| "That star over there is the one that passed" | True only for a recent pass inside the current time frame, or for a star someone promoted | Recent passes come from real neighbours. Older ones name a star of that era, labelled a reconstruction |
| Supernova bubbles carved into the gas along every line of sight | The sky's dust is summed through thousands of cells | Dents within a few hundred parsecs of where you stand; the far sky keeps the smooth value. Registered |
| Exact closure at the very top, and for slow supernovae | A web cell holds hundreds of haloes; slow supernovae go off billions of years and many kiloparsecs from birth | Statistical there, exact from the ring down |

### The gap that matters most: where an element ends up

This one is mine, and it bears directly on "a metal asteroid leaves a massive ore deposit".

The six groups say how an element was **made**. They do not say where it **goes** when a world melts and separates. Gold and platinum follow iron into a core. Uranium and potassium stay in the rock. Water and carbon leave with the volatiles. So "how much gold is in this system" is a sound read of the ledger, but "how much gold is in this asteroid" depends on which layer of which parent it came from. The data for this was collected on 23 Nov (data sheet package MP24, "where each element goes": condensation temperatures, and Earth's core, mantle and crust concentrations). Its own log marks every value as an unverified reproduction, and the rules that would use it (MP6 to MP8) are not built. I found no code that reads it.

Without it, the ore deposit is a fudge too far: the crater would know a metal body hit, and not what the metal holds. With it, the prize is large. A core fragment is rich in gold and platinum for a physical reason, a crust is poor in them, and a prospector's map follows from the history. It needs that table verified against its sources and one rule applied wherever a body separates into layers. It adds no new ledger.

### Weak spots in the plan itself

- **Nothing is measured.** How far today's stars are from their ring's mass, how big a save is, what a resolve costs on a phone: all unknown. Stage 1 exists to find out, and its results may reorder the rest.
- **Stage 3 holds the core golden lane.** Every star changes in one move, and other threads' work queues behind it. It should be short, announced, and done when the lane is quiet.
- **The far future is one plausible future.** After a star dies, what its planets do is chaotic. The slider will show a repeatable outcome, and the handoff already requires it to be labelled as one of many.
- **A postcard from an edited campaign.** Carrying the deeds on the card's own chain misses a deed elsewhere that still shows, such as a star pinned in that sky. The safe addition is a fingerprint of the campaign's deeds on every card, so the receiver's screen can say "from an edited universe; some details may differ" when it does not hold them.
- **A conserved chain needs calibrated rates.** This morning's finding on inner-galaxy metals is the first example. Each ledger needs a small set of checks against measured things, and those checks do not exist as a set yet.

### Three more questions

12. **Verify the table of where elements go (MP24), and build the rule that uses it?** Recommend yes, as a data sheet package ahead of stage 5. It is what makes deposits real.
13. **Trace only listed craters, and treat the rest as a sum?** Recommend yes.
14. **Give imported systems a reconstructed history, so the trick works on V3 systems?** Recommend yes, after stage 5, once we know how much of the fitting exists.

## 13. Before development starts

Six things need settling first, and three pieces of work in flight should hold until they are. Everything else in the project can carry on.

### Sort first

|  | What | Why it comes first | Who | Size |
| --- | --- | --- | --- | --- |
| 1 | **Your answers** to the open questions (5, 7, 10 to 14) and a yes to local sets | They fix the scope | Alex | One reply |
| 2 | **This note into `handover/design/`**, with one standing rule in the build plan: no join passes a description without an amount, and every join has its three tests | Other threads are adding joins now. The rule stops a third shortcut | This thread, on your yes | An hour |
| 3 | **Measure, change nothing** (stage 1): closure and indifference tests in report-only form, a ledger read-out per join, save sizes, resolve costs | Every later estimate rests on numbers nobody has | Kernel builder | A few packages; no golden moves |
| 4 | **Calibrate the chemistry upstream.** The Local Arm thread has now found the mid-disc gas (3.6 to 6.6 kpc) at +0.3 to +0.46 in iron, about 0.15 to 0.2 above measured gas clouds, from ring yields marked "tuned". With the inner winds found this morning, that is two faults in the one ledger everything below will inherit from | Tying the joins first would hand every system a wrong recipe exactly | A builder, against a check set this thread writes | See below |
| 5 | **Prove the roster** in a scratch prototype outside the kernel, on a belt of millions, including a member removed by a deed | It is the one new mechanism and exists only on paper. If it fails, stage 5 shrinks to notables and the largest members | Kernel or small-bodies builder | A few days; nothing merged |
| 6 | **One short spec per split**, before its package: ring to clusters to stars; belt to classes to families to members; the neighbourhood stream; where elements go when a body separates | The splits are where the design is hard, and where the last shortcut was taken | This thread | One note each |

### On the chemistry (item 4)

The Local Arm thread's card to you recommends handing the recalibration to this thread. It does belong on this list, ahead of stage 3. I would split it in two. This thread writes the **check set**: the handful of measured things each ledger must match (the iron gradient with radius from young stars and gas clouds, its flattening inward, the age and metal relation near the Sun, the alpha and iron trend, the gas and star surface densities), each with its source and tolerance. A builder thread then retunes the yields and winds against that set, because that is code and golden work, and design threads change no code. The check set is useful beyond this one fix: it is the "checks against measured things" that a conserved chain needs at every ledger.

### Hold until the star field is settled (stage 3)

- **The Local stars rung of the grand zoom,** and anything that draws or names particular nearby stars.
- **The gazetteer's star rows** (GZ17). Its galaxy rows are safe.
- **Postcards or fixtures that depend on particular generated stars.** They will redraw themselves, but any golden or stored picture of them will move.

### Safe to carry on

The places tie-up (GZ12, GZ13, GZ15), which this design needs anyway; the Local Group view; everything inside a system; surfaces, weather, light, the renderer, editing, the desks, the Atlas and the Run desk. None of them reads the parts that change.

### Two small things to do now, because they are cheap now

- **Stamp the draw algorithm's version into every saved universe.** The kernel already reports a `draw_algorithm` and a `kernel_version`; I did not check that a saved campaign records them. After stage 3 this stamp is the only way to tell an old universe from a new one.
- **Say in the alpha notice** that the unexplored parts of a universe may change once while this lands, and that anything explored and saved will stay.

## 14. Recommendation, and what only you can answer

Build option C by the seven stages, starting with the stage that measures and changes nothing. Keep the code. Treat the splits and the roster as the design work that still needs doing at each join, and write each one as a short note before its package starts.

The questions, each with my recommendation, in the form you asked for ("yes to 1, 2 and 4").

1. **Allocated by the seed and revealed by looking, in place of debited when looked at?** Recommend yes. It is the only form that keeps same seed, same universe.
2. **Only deeds change the universe?** Looking, naming and visiting record what was found. Mining, pinning, editing and play are the dated changes. Recommend yes.
3. **A postcard shows the universe, not the sender's campaign?** A receiver sees what the seed gives, plus any published world the card names. Recommend yes; it matches the 25 Nov ruling.
4. **Accept the deep-time limit on the anonymous star field?** Unlooked stars are redrawn each time frame with the right totals; looked-at stars are permanent. Recommend yes for 4.0, labelled on screen.
5. **Let unlooked parts of alpha universes shift once at stage 3?** Recorded things stay. Recommend yes, with a line in the alpha notice, and stamp the algorithm version from then on.
6. **Exact where a thing is named, a registered tolerance where it is not?** Births, core-collapse deaths, belt members and impacts close in whole units. Slow supernovae and the web cell's report stay statistical, each in the fudge register. Recommend yes.
7. **Mass and the six element groups only?** Angular momentum at the birth of a disc; no energy ledger across levels. Recommend yes.
8. **Keep real rates even where a better story is rare?** Most craters will trace to a dull cause and few to a passing star. Recommend yes. A GM who wants the good story can pin it, and it is marked as chosen.
9. **How far forward must the sketch run for 4.0?** To the end of the home star's life, or to the end of the universe at every level? Recommend the first now, the second as each reservoir gains a coasting rule. I have not checked how far the rings run today.
10. **Where in the order?** Recommend stages 1 and 2 now, since GZ12 and GZ13 already lead the kernel's queue, then stage 3 before the grand zoom's journey is built on top of a star field that is about to change.
11. **May this note go into `handover/design/` as `SSE4-conserved-chain-design.md`,** with a status file for the track? Recommend yes. I have not added it; no code or repo file was changed.

### What I checked, and what I did not

- **Read in code on `main`:** the split and pin rules in `sse-resolve`; the star fill, pins and promotion (`stars.rs`, `star_tree.rs`, the kernel's `promote_star`); how events are drawn, including where a passing star's mass comes from (`events.rs`); how a belt splits and how a member gets its makeup (`leftovers.rs`, `sse-small/member.rs`); where craters come from (`impacts.rs`); the ring model's groups and returns (`rings.rs`).
- **Read in notes:** the audit, the design handoff sections 5 and 6, the galaxy design section 8, the grand zoom note section 6 and its rulings, the postcards rulings, the final handover, and the Local Arm thread's finding of this morning.
- **Not done:** nothing was run. The sizes in the saving table are estimates. I did not measure how far any cell's stars are from its ring's mass, read the formation code's angular momentum and energy, or check how far forward each sketch runs. The roster is a design, not a prototype; its cost needs proving on a real belt before stage 5 is promised.
