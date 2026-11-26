# SSE4: the conserved chain, split by split (CC5)

Track CC. Written 26 Nov 2026 by the design thread, after the CC1 measurements (main at d726d46, home galaxy seed g3; report https://claude.ai/artifact/LtTgFC1uNKb7A2YcMEGtBm). Read with `SSE4-conserved-chain-design.md` (the accepted note) and `SSE4-conserved-chain-checks.md` (the check set).

State: **draft for the builders. Alex ruled on section 4 on 26 Nov 2026 (17:05 UTC): anchor at the campaign date.** The roster part (section 6A) is written from the CC4 prototype's report. Nothing here changes code. Each section is the spec its package reads before it starts.

The working rule holds throughout (Alex, 26 Nov): plausible, repeatable and explained, not exact. Where the measurements showed a join already closes to a few percent, this note says "leave it" and registers the tolerance instead of building machinery.

## 1. What the measurements changed

| Join | Measured | What it means |
|---|---|---|
| Ring to stars, by mass | Drawn stars weigh 0.92 to 0.97 of the ring's budget at every radius. A 10 pc cell strays 10 to 30% by the draw alone | The mass is nearly right already. Stage 3 of the note shrinks from "carry the mass" to "say which stars". The cell scatter is real clumpiness and stays |
| Ring to deaths | The ring books about twice the core-collapse deaths its drawn massive stars can make, and about eight times near the midplane | Two causes, both fixable at the root: companions never explode, and events sit in a 100 pc gas layer while young massive stars are drawn with old-star heights |
| Star frames | Frames are 0.1 Myr; 0 of 2,708 stars survive one frame | The star field at another date is a different field. Needs a ruling (section 4) |
| Arm and texture | The arm pattern averages exactly 1. Texture adds 3 to 5% per ring | One normalising constant per ring. Small |
| Belt to members | Resolved members carry 0.81 to 1.09 of the belt's metal share, mean 0.93 | Good enough under the working rule. Correct the mean; build no class ledger |
| Star to disc | Discs are about 5% of ring stellar mass. Mg/Si and Al-26 are fixed at 1.0 on the grown route; the stand-in Al-26 median is 0.3 | Disc is carved from the system's claim. Mg/Si and Al-26 come from the place (sections 3 and 5) |
| Place | 1 of 8 system routes inherits its place | This is GZ13's work; unchanged |
| Save size | 2.3 kB per charted star, 8 to 11 kB per system; the note's additions cost under 5% | Replaces the note's estimates |
| Neighbourhood stream | 12 ms for 10 pc, 234 ms for 50 pc, natively | 10 pc by default, 50 pc on request |
| Rings past now | Rings stop at now and hold flat | Needs a coasting rule (section 7) |

## 2. Ring to stars

**Keep the count tree.** `star_tree.rs` already hands a ring's stars down to cells by count and the totals land within 3 to 8% of the mass budget. Do not replace it with a mass-first partition.

**Fix the shortfall at its cause.** The counts are made from mass with `mean_star_msun: 0.5` (`envelope.rs:80`), which is a round number and not the mean of the stars actually drawn. Replace it with the mean system mass of the pack's own initial mass function and multiplicity rule, computed once from the same tables the draw uses. After that the ring-wide ratio should sit at 1.00 within the draw's noise.

**One initial mass function.** The rings quote `cc_per_mass 0.0118` and `return_fraction 0.4` as Chabrier numbers, while `stars.rs` draws from Kroupa. Every per-mass rate the rings use (core-collapse deaths, type Ia progenitors, returned fraction, yields per unit mass formed) becomes an integral over the pack's one function, computed in one place and read by both. This bears directly on the chemistry retune (CC3): do it first, because it moves the level of every element and is not a tuning knob.

**Texture.** Divide the texture field by its own ring average, so it moves stars about within a ring and adds none.

**Tests.**
- Closure: for each ring, drawn stellar mass over budgeted mass is within 1.00 plus or minus 0.02 over the whole ring. Absolute, not a ratio between two runs.
- Registered tolerance: a 10 pc cell may stray by the Poisson scatter of its count. The test checks that the scatter is the size Poisson predicts, so a future bias shows up.
- Indifference: the stars in a cell are the same whichever neighbouring cell was opened first and at whatever zoom.
- Seen red first: run the closure test with `mean_star_msun` put back to 0.5 and see it fail.

## 3. Birth clusters and deaths

Stars are born in groups, and the groups are where the interest is: a system's short-lived isotopes, its early close passes and its runaway siblings all come from its birth cluster. This is the "clumpy and lumpy" split Alex asked for, at the star level.

**The split.** For each cell of about 100 pc and each 10 Myr birth bin, the ring gives the mass of stars formed there. Draw clusters from a mass function falling as the inverse square of mass, between about 50 and 100,000 solar masses, until the bin's mass is used. The last cluster is trimmed to fit, so the bin closes exactly. Each cluster gets a centre, a birth date inside the bin and a size, all from its key `{cell}/cluster/{bin}/{k}`.

**Membership without a second star list.** The star draw stays as it is. A star belongs to a cluster by a keyed choice weighted by how near its birth position is to each cluster centre; the weights are normalised so the cell's total count does not change. For stars younger than a cluster's dispersal time (some tens of Myr; a pack number), the star's position is drawn inside its cluster. Older stars have long since dispersed and keep their field positions, but still know which cluster they came from. A cluster is therefore a stub with a key, a mass and a date, and costs nothing until someone asks.

**Deaths are stars.** A core-collapse supernova is the death of a drawn massive star, with the star's own key, date and place. The ring's supernova rate is no longer a second, independent number; it is what the one mass function says the cluster's stars will do, so both ends of the flow are the same event. Two root fixes make the counts agree:
- Companions get life lines. A massive secondary explodes like any other massive star.
- Young stars get a scale height that depends on their age, so massive stars sit in the thin gas layer where their deaths are booked. Age-dependent heights are being written up as their own package in `handover/packages/` by the Local Arm thread (after its bulge PR). CC6 points at that package and does not build a second version.

**What a system reads from its cluster.** Al-26 at birth is the yield of the cluster's massive siblings that died before the system's disc closed, diluted over the cluster's gas. Mg/Si is the ring's alpha-group value for that place and date with the data sheet's star-to-star scatter. Both replace the fixed 1.0 and the stand-in median.

**Tests.**
- Closure: clusters in a bin sum to the bin's formed mass exactly. Deaths counted from drawn stars over 100 Myr agree with the ring's booked deaths within 10% ring-wide, and within Poisson scatter per cell.
- Indifference: a star's cluster is the same whether the cluster or the star was looked at first.
- No stand-in: no system reads a constant Al-26 or Mg/Si.
- Seen red first: the death-count test fails on today's code (factor two).

## 4. The star field at other dates (ruled 26 Nov)

The measurement is blunt: the field is rebuilt every 0.1 Myr and no star is carried from one frame to the next. Within the working rule there are three honest choices.

1. **Anchor at the campaign's date (recommended).** At the campaign's defined date the stars within the home bubble are the cell fill, and they are promoted to the tracked set: each has a real orbit and is the same star at every date. For any other date the sky beyond the tracked set comes from the neighbourhood stream (section 5). Stream arrivals whose stay would overlap the anchor date are dropped, since the tracked set already holds those stars. Cost: 12 ms for a 10 pc bubble.
2. Orbits for every star in the galaxy. Correct and far too expensive; it also buys nothing a GM would see.
3. Leave it. Slide the clock a million years and the neighbours are strangers with no history. This breaks "same seed, same universe" for anything a GM has named.

Choice 1 keeps every star a GM has looked at fixed through time, and makes the rest plausible and repeatable. It is the one place the design says "hold on, that is enough" about the deep past of the general field.

**Ruling (Alex, 26 Nov 2026):** choice 1. The bubble radius is a pack number on the performance side: 10 pc by default, and 50 pc is to be timed against real star density; a few milliseconds is fine.

**What this means for motion.** Every star in the bubble has a determined velocity and follows its closed-form orbit at every date. The machinery exists: `stars.rs` already gives each drawn system an epicyclic orbit, and a promoted system (a pin) is the same system at any time asked in any order. What is missing is only that unopened fill stars are redrawn each frame, so their orbits are discarded. Anchoring promotes the bubble's fill at the campaign date, in effect making them pins without the cost of a full system resolve (a tracked star is its fill record: key, mass, birth, orbit).

## 5. The neighbourhood stream

The close-pass stream in `events.rs` already gives keyed encounters per site and time bin. Three changes make it the feeder for a local set of stars.

- **Wider.** Today it lists only passes close enough to stir comets (`shower_pc 0.05`). Widen the listing radius to the bubble (10 pc by default). The close passes are then the inner few of the same list, not a separate draw.
- **Arrivals are stars, not masses.** Today the passing star's mass is drawn fresh. Instead each arrival is a full star stub with its own key `{site}/encounter/{bin}/{j}`, drawn from the ring's stars at that place and date (so its age, metal content and birth cluster are consistent with where it is). Looking at it resolves a system like any other.
- **Rate from the books.** The arrival rate is the local star density from the ring times the bubble's cross-section times the local velocity spread. No free number.

Each arrival has an entry date, a closest approach, a relative velocity and an exit date, all from its key, so any date can be read directly without stepping from the start. Arrivals that overlap the anchor date are dropped in favour of the tracked set (section 4).

Runaways and strays ride the same stream. A star that loses its partner in a cluster or a supernova leaves as an arrival in someone else's stream with its origin recorded; a rogue planet thrown out during the violent phase of formation is listed by its parent system as a tracked object with a velocity. Neither needs new machinery, only that the leaving end is written when the event happens.

**Tests.**
- Closure: the time-averaged number of stream stars inside the bubble equals the ring's local density times the bubble volume within 5%.
- Indifference: the list for a date is the same read directly, or read after sliding there from any other date.
- Seen red first: the density test fails on today's stream (it lists only the 0.05 pc passes).

## 6. Belt to members

The belt's metal share came out at a mean of 0.93 across resolved members, range 0.81 to 1.09. Under the working rule that is close enough to leave the mechanism alone.

- **Correct the mean, keep the scatter.** `member.rs` scatters each member's makeup around the belt mean. Centre that scatter so its expected sum over members equals the belt's own share by group. Scatter between members stays; it is the lumpiness.
- **No class ledger.** Class (metal, stony, carbon-rich) stays a label read off a member's makeup, as the standing ruling says. There is no separate budget by class to debit.
- **Families only from events.** A family exists only where formation recorded a break-up. Its members' mass sums to the parent's, and the parent's layers go to members by the element rule already built for worlds (MP6): core pieces are metal-rich notables, mantle pieces stony. A bared core is one notable with a traced origin. This is where "Planet X has a massive platinum deposit after an old planetary core collided" comes from, and it needs no new physics, only the reuse of MP6 on pieces and on what an impact delivers to a site (CC9).
- **Gravel is a sum.** Members below the listing size are an amount by group and never individuals, unless a deed pins one.

## 6A. The roster (from the CC4 prototype)

The prototype (thread "Prototype the crater roster", branch `claude/project-thread-h51rzm` under `scratch/`, not for merging) proved the roster on belts of up to 10 billion named bodies. A crater reads back to its impactor, origin, cause and travel time in about 2.5 microseconds in wasm, the same in any order and at any date, bit for bit native and wasm. So the fallback is not needed for fates. The cost is in the mass books.

- **Names and fates for every body down to 1 km.** The roster is the seeded, invertible link between "the j-th departure of a size band in a time bin" and a member. Both ends read the same key.
- **Mass books in three tiers.** Exact, body by body, for about the largest 100,000 in each class (64 ms). Below that, exact counts at an average mass per size band. Gravel as one total by group. This is the registered tolerance of the belt join.
- **A deed leaves a hole.** A mined or moved body's slot stays empty and nothing moves up to replace it, so one deed removes one crater and changes nothing else. Saves cost about 100 bytes for the universe, 70 per deed and 275 per body looked at.
- **Listed craters are chosen by survival.** Picking the largest arrivals over a world's whole life fills the list with ancient scars. The surface's own erasure picks what is listed today; the roster supplies who made each.
- **The largest arrivals are giant impacts**, and belong to formation's violent phase, not the crater list. The roster hands them over above a size set by the target.
- **A world gets an arrivals line.** Each world's books gain a line for mass delivered by impacts, by group. This is where an impact-delivered ore deposit lives, and what "What is interesting here" reads.
- **A bared core is many bodies.** Formation shatters a core into dozens of iron bodies of similar size, and some fall into the star. "The bared core" is a family of notables with fates, of which the survivors are the interest.
- **Passing stars: real but rare.** In the prototype's system they sent 94 bodies of 1 km or more into solid worlds over its life and none made a crater list. The traced "why" should say so honestly when it applies and never be promised as common. A metal body from the comet cloud needs a metal class there, which follows from families (scattered core pieces), not from a new draw.

**Three decisions for the kernel owner before CC8, each with a decision record:**
1. The kernel mass unit rounds a 10 km asteroid to zero. Small bodies need a finer unit or a scaled one per reservoir; the books must still close in integers.
2. The prototype used a faster random scheme because the kernel's keyed draws are too slow at this count. Adopting it changes `draw_algorithm`; it must stay keyed, order-free and identical native and wasm.
3. The size above which an arrival is a giant impact.

**Tests,** as the prototype ran them: order independence and slider stability, both seen red when broken on purpose; one deed changes exactly one crater.

**Tests.**
- Closure: expected member sum equals the belt's share by group within 2% over many seeds; every family sums to its parent exactly.
- Indifference: the k-th largest member is the same whatever was opened first.
- Seen red first: the mean test fails on today's scatter (0.93).

## 7. Star to system, and rings past now

**The birth claim.** A system's disc is carved out of the mass its star slot was drawn with; it is not added on top. The claim is the star's drawn mass plus a disc fraction taken from the same slot, in six groups read from the ring at the birth place and date. Closure is then true by construction and the 5% in discs is an explained line, not a leak.

**Coasting.** Past the present the rings keep running the same equations with infall declining on its existing curve. When a ring's star formation falls below a pack threshold it is marked settled, and from then on it changes only as its stars' life lines end. This gives every galaxy the settled end state Alex asked for and removes the flat line after now.

## 8. Order for the builders

1. One initial mass function across rings and stars, and the real mean system mass (section 2). Before the chemistry retune.
2. The chemistry retune against the check set (CC3), after GZ12.
3. Clusters, deaths as stars, age-dependent heights from the Local Arm thread (section 3).
4. Tracked set and neighbourhood stream (sections 4 and 5), once Alex has ruled.
5. Belt mean and families (section 6); roster after CC4.
6. Coasting rule (section 7), any time after step 2.

Steps 1 to 4 move the star field and are the single golden change the hold list is waiting for. They should land as close together as the one-at-a-time golden lane allows.
