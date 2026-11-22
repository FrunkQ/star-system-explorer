# SSE4 voyages between stars: the voyage finder and the autopilot

Status: proposed 21 Nov 2026, second UI track ("Build galaxy charts and travel screens"). It builds on the matter-pack design's section 7A.8 (ratings and counts, MP16), the programs design's section 6 (autopilot is a seat driven by a flow, J6), the night sky design (catalogue, then resolve, track Y) and NAV (`sse-nav`, T3).

## 1. What Alex asked for (21 Nov, 23:11 to 23:13)

- "Autopilot will be able to do jumps/FTL/relativistic speeds - if capable."
- "An explorer ship gradually scanning systems spiralling outwards."
- "Or set a destination and it charts a course - refuels on route (distant surveys to gas skim) and detailed surveys to destination."

The standing rule still holds: players never hand-pilot. They tell the ship's computer where to go and what to do, and they read its plan.

## 2. The pieces

| Piece | What it does | Where it lives |
|---|---|---|
| **Drive facts** | What one ship can do: thrust, exhaust speed, propellant, a jump rating, another FTL rating | on the ship (template facts), read by NAV's `Ship` |
| **Leg** | One move between two star systems by one means: jump, other FTL, or flying there for real (relativistic or not) | `sse-nav` `voyage.rs` |
| **Voyage finder** | The cheapest chain of legs between systems for one ship, with fuel carried and refuel stops; or the nearest unvisited system it can reach | `sse-nav` `voyage.rs`, kernel op `plan_voyage` |
| **Stops** | What happens at a system: refuel (skim a gas giant, ice or water, buy at a port), survey (from afar, or detailed) | pack rules and the survey ladder |
| **Autopilot program** | Chains legs and stops: "explore outward", "go to X, refuel on the way, survey there" | track J programs (J6 part 2, kernel thread) using the finder, plus this track's screens |

## 3. Legs: three ways to cross between stars

**Jump** (a pack's FTL, marked fictional on the GM view). A ship rated Jump-N may make one jump of up to N parsecs. The pack's rule gives the fuel for a jump and the time it takes (Traveller: fuel of 10% of the hull per parsec of the jump; about a week whatever the length). On a drawn route the count the GM typed wins (MP16); off the drawn routes the pack's default rule gives it from the real distance.

**Other FTL** (warp factors, gate classes). The same as a jump in shape: a rating on the ship, compared with a need on the leg, the cost and time from the pack's rule. Until a pack declares such a rule the finder refuses the leg and says which rule is missing.

**Flying there for real** (physics, never marked fictional). A sustained drive burns at constant proper acceleration, turns over and brakes. The finder works it with special relativity: proper time aboard and time at home both, the peak speed, and the propellant from the relativistic rocket equation (the rapidity change times c over the exhaust speed gives the log of the mass ratio). If the propellant cannot reach the turnover speed, the ship coasts in the middle at the best speed it can afford. At low speeds this is the same answer NAV's torch courses give. A ship without a sustained drive, or whose propellant cannot stop it at the far end, is not offered the leg, and the refusal says so ("the Kestrel Wren cannot brake at Tau Ceti: 0.4% of the Δv it needs").

### 3.1 In-system legs, and how the two scales join (Alex, 23:27: "This all has to work in system too ... Hohmann transfers and asteroid belts")

There is one finder and one kind of voyage. What changes with scale is only what a **place** is and which **leg-maker** prices the move:

| Place | Example | Leg-maker between two places of this kind |
|---|---|---|
| A body | Mars, Ceres, a moon, a station on a body | NAV's courses (`plan_route`, T3): Hohmann and Lambert transfers with launch windows for coasting ships, brachistochrone torch courses for sustained drives, patched-conic escape and capture, hazards and difficulty |
| A belt region | a stretch of the main belt or a Kuiper belt band, from the belt clouds the kernel already resolves | the same courses, to the region's mean orbit; the bodies in it resolve when surveyed (night sky rule) |
| A star system | Tau Ceti | this design's interstellar legs: jump, other FTL, relativistic flight |

The two scales join at a system's **edge**. An interstellar leg leaves from and arrives at the system as a whole (for a jump, the pack's jump distance from the star; for a flight, the system's barycentre). Getting from a body out to that edge, and from the far edge in to a body, is an in-system leg like any other. So "Mars to a moon of a gas giant at Tau Ceti" is one chain: Mars to the edge (NAV course), the jump or flight, the edge in to the gas giant (NAV course), skim, then on to the moon. The finder searches across all of it at once with the same weights, the same carried fuel and the same refusals.

For a hard-science game with no FTL the search simply never leaves the system: a miner's or a patrol's whole life is in-system legs, with real launch windows, so "wait 7 months for the window to Jupiter" is part of the plan and its time at home. Refuel stops in-system are the same pack rules (skim a giant, crack ice on a moon or belt body, buy at a port), and asteroid belts are both places to go (mining) and something courses are checked against (the hazards NAV already flags).

Each leg carries its distance in parsecs (physics, with "Why?"), its count where a counted measure applies ("2 jumps"), its time at home, its time aboard, its fuel, and whether it is fictional.

## 4. The voyage finder

One search over one graph, as MP16 asks: nodes are star systems (from the star cells around the start, Y7a, and later the sector maps and drawn routes), edges are legs the ship can make, and the weight is what the GM or the program chose: **time at home**, **time aboard**, **fuel**, **number of legs**, or a **counted measure** (fewest jumps).

- **Fuel is carried.** The finder tracks the fuel left after each leg, refuels to full at a stop that offers fuel the crew knows about, and never plans a leg the tank cannot pay for. A tank is drawn as the number of jumps or parsecs it holds.
- **Refuel stops are what the crew knows, not what the GM knows.** A system offers fuel if its crew's knowledge (the program seat's audience, as J6 senses) shows a gas giant to skim, water or ice to crack, or a port that sells it. From afar that comes from the remote survey (night sky catalogue and O12 to O15: a gas giant found by its wobble or transit with this ship's instruments). An unsurveyed system is assumed dry (question 3).
- **Refusals name the missing capability.** "Needs Jump-3, the Kestrel Wren is Jump-2"; "no known fuel within Jump-2 of Barnard's Star"; "a relativistic leg needs a sustained drive".
- **Nearest unvisited.** Given a ship, a start, its fuel and the list of systems already visited, the finder answers "the nearest system this ship can reach that it has not visited", with the chain of legs to it. This is what the explorer program calls at each step.

## 5. The two programs Alex described

**Explore outward.** Repeat: ask the finder for the nearest unvisited reachable system (ties go to the one farther from home, so the path spirals out instead of doubling back); fly there; survey it (resolve it the night sky way, recorded with who found it); if the tank is low, refuel where the survey found fuel; go on. Stops when nothing is reachable, after N systems, or when a GM-set boundary is reached.

**Go to a destination.** Ask the finder for the chain of legs to the destination with refuel stops. At each refuel stop: a quick survey to find and skim the gas giant, then go on. At the destination: a detailed survey of every world (surface surveys where the ship's instruments allow). The plan is shown before it runs, as the Route console already shows courses: legs, stops, fuel left, time at home and aboard.

Both are programs (track J): they act through intents within the GM-set scope, sense only through their crew's audience, and advance the campaign clock by home time; each leg ends a tick, as J6 part 2 plans for NAV travel.

### 5.1 The other roles (Alex, 23:15: "Mining, trading (economy driven eventually), patrolling, pirating, etc.")

These are not new machinery. Every role is a program (J6's states, picks and intents) and every move between places is a voyage leg from the same finder, so a role is pack data, not code:

| Role | Picks (from what its crew knows) | Acts at each stop | Already exists |
|---|---|---|---|
| Explorer | nearest unvisited reachable system | survey, refuel when low | the finder (this design) |
| Courier or passage | the destination it was given | refuel, deliver | the finder |
| Miner | the richest body it has surveyed within reach | mine (real amounts from physics, which run out), return, unload | J6's miner fixture, in-system |
| Trader | the best margin between ports it knows, net of fuel and time | buy, sell | new: prices from a pack table now, the economy pack later through the same kept values |
| Patrol | the next waypoint of its beat; a sensed suspect | inspect, pursue, report | J6 part 2's patrol fixture |
| Pirate | the weakest ship it has sensed on a lane it watches | intercept, demand, flee | J6 part 2's pirate fixture |

What this track adds for all of them: the finder as the one way any role moves between systems (in-system courses stay NAV's `plan_route`), and the screens that show any role's plan before it runs. Trading needs prices; until the economy pack exists they are a pack table of base prices per port (the "placeholder" that the economy replaces without changing the trader program).

### 5.2 What V3's autopilot teaches (read 22 Nov from V3's `docs/autopilot-guide.md` and `src/lib/transit/autopilot*.ts`)

V3 already flies standing routes in a system, and much of it is right. Kept as ideas, not structure:

- **The itinerary is the state.** Everything (position, fuel, cargo, log) is derived from the clock, so scrubbing time gives the same answer every time. SSE4 gets this from the chronicle and J6 programs for free; keep it a rule.
- **Five verbs cover the roles:** transport (place to place), patrol (go and hold; hold 0 is a flyby), mine (search for the best source of a resource), explore (search for somewhere not yet visited), escort (meet a moving ship and keep formation, which breaks honestly when the charge out-thrusts it). Trading and piracy are compositions of these plus a pick rule, which is how section 5.1 already frames them.
- **One search scores candidates** by richness, closeness from where the ship will be, and a refuel bonus that grows sharply when the tanks are below half. SSE4's finder makes this exact: the cost is the real voyage, refuel stops are part of the plan, and the pick rule is pack data.
- **Plan families and a "drive" choice:** most efficient (may wait for a window), efficient now, direct burn, flyby assist; and a cap on any leg's whole time including the wait ("prevents the zero-fuel fifty-year crawl"). SSE4: the finder's measure plus a `max_leg_days` cap in the program's parameters; NAV already offers the families.
- **Traversal:** in order, best order (reordered by real transfer quotes at the projected times, with a staleness bonus so no stop starves), and as needed. SSE4: a program state's pick, with the finder's quotes as the cost.
- **Honest stops:** a red "stuck" with the exact reason, orange "needs your decision", green "done". SSE4: the program's halt reason, and approval requests (question 7).
- **Logs and totals:** work events at stops (loaded, mined, refuelled), incoming visits shown on stations, tonnes a year. SSE4: these are chronicle events, so any data view can read them.
- **Self-fuel while mining** (a fuel's refuel tags), an **avoid list** that explicit orders override, **discipline** (slack added only to stopped time, from the owner), and **disengage** choices (finish the leg, cut thrust and drift, emergency stop).

Where V3 was overcomplicated: many separate sliders and special cases per verb, and its own scheduler. In SSE4 each knob is a program parameter with a default in the ship-roles pack (editable by people or Orac), the five verbs are states of one program language, and every move is the one finder.

## 6. Screens

- **Route console, "Between stars"**: pick a ship and a destination (or "nearest unvisited"), choose what to minimise, see the legs on a small star map with fuel left after each, refuel stops marked, times at home and aboard, and every refusal in words.
- **Galaxies nearby and the hex map** get "plan a voyage to here" on a star.
- **The player console**: the crew asks the ship's computer for a voyage and reads its plan; they never pick legs by hand. The GM's approval rules apply as for any intent.

## 7. Packages

| Package | What | Owner |
|---|---|---|
| V8 part 3 | `sse-nav` voyage legs (jump, relativistic; other FTL refused until a pack rule), the finder with fuel and refuel stops, nearest unvisited with the outward tie-break; kernel op `plan_voyage` over star cells; Route console "Between stars" on the GM screen | this track (starts now) |
| V8 part 3b | In-system voyages through the same finder: bodies and belt regions as places, NAV courses as the leg-maker (windows included), refuel and mining stops; the join at each system's edge so a voyage can run body to body across stars | this track, straight after part 3 |
| V8 part 3c | Fuel by ship, engine and fuel type: engines name their fuel, ships list refuel capabilities (scoop, ice processor, docking), places list what they hold, exotic fuels only at ports that stock them; refusal when a ship cannot climb out of a gravity well; FTL rules evaluated per star system (questions 1 and 2) | this track |
| V8 part 4 | Counted jumps on drawn routes and "fewest jumps" (needs MP16's declarations and kept values); "plan a voyage to here" from the maps | this track, after the kernel's MP16 part |
| J6 part 2 (voyage) | The `voyage` intent and the two programs as pack data; ticks at each leg's end; survey at stops; the clock advances by home time | kernel thread, with this track's finder |
| V8 part 5 | The player console's voyage requests and a seat's preview of what its autopilot will do | this track, after J6 part 2 |

## 8. Questions for Alex (with recommendations)

1. **Fuel.** Answered 22 Nov 00:40: "fuel is determined by ship and engine and fuel type - all physically modelled. FLT is abstract by nature - and need \"per system\" rules ... Real travel & relativistic are always available IF it can break the gravitational field." Plan (V8 part 3c): real and relativistic drives already burn propellant by the rocket equation from the ship's mass, engine exhaust speed and tanks; each engine now also names its fuel type (hydrogen, water, deuterium and tritium, deuterium and helium-3, antimatter), and a leg is refused when the ship cannot climb out of where it starts (thrust against the local gravity off a surface; the escape Δv from its orbit, and from the star, for a flight between stars), with the reason in words. FTL stays a setting's pack rule, read as per game setting; the rule is evaluated against each star system it touches, so a setting can say "jump only beyond 100 diameters of any body" or "no jumps inside a gravity well", and the jump distance then depends on that system's own masses.
2. **What counts as a refuel stop.** Answered 22 Nov 00:40: "Refuelling is a ship capability - it may need a port and exotic fuel." Plan (V8 part 3c): a stop offers fuel only when three things agree: what the place holds (a giant's hydrogen and helium-3, ice that cracks to hydrogen and oxygen, deuterium from water, a port's stock list), what the ship can do (a scoop and a hull that can skim, a processor that can crack ice, docking), and the fuel its engines burn. Antimatter and other exotic fuels come only from ports that list them. Each refuel kind is pack data, so a setting or Orac can add one. V3 did the same with refuel tags on each fuel and instant refuelling at ports and depots, at a rate when harvesting.
3. **Unsurveyed systems.** Answered 23:26: an informed risk, not a blind gamble. Built: each unsurveyed system has odds of holding fuel, from the ship's sensors when it has sensed the system, otherwise from its star (giant-planet odds rise with the star's mass and metallicity, Johnson et al. 2010, editable in the pack). Gambles are a GM switch; a plan takes one only when the odds reach a pack floor (50% to start), and the plan shows each gamble's odds and the chance they all pay off. Original recommendation: assumed dry, so a plan never depends on fuel nobody has seen. A GM switch "allow gambles" lets a program jump to an unsurveyed system hoping for fuel, and such a leg is marked as a gamble.
4. **The explorer's order.** Settled by V3: its Explore verb picks the best-scored place not yet in the ship's log, so the log growing by one each circuit drives the outward sweep, and it raises "needs your decision" when nothing new is left. SSE4 does the same with the real voyage as the score (nearest unvisited by the chosen measure, ties to the one farther from home), as built.
5. **Relativistic time.** Answered 23:26: yes, and each crew member's own age matters (plot points). Recommended: the campaign clock moves by time at home; each crew member ages by time aboard (kept on the ship), and the plan shows both. Messages home take the light time, as the night sky already honours.
6. **What a survey does at each stop.** Partly settled by V3 ("go and survey it, then push on"; refuelling and other work at a stop are logged events). Still open: how deep. Recommended: a system survey on arrival at every stop (every body resolved and recorded with who found it); the detailed surface survey only at the destination or where the program names it, because it is the costly one. A program parameter changes either.
7. **Approval.** Mostly settled by V3: a ship on autopilot runs on its own and stops itself with a red "stuck" and the exact reason, or an orange "needs your decision" when its search finds nothing; no step asks first. SSE4 takes the same rule: an autopilot runs within its program's parameters (avoid list, gambles allowed and the odds they need, a cap on any leg's time), and the GM is asked only at those two flags. Still open: whether leaving surveyed space should also raise "needs your decision" by default. Recommended: no, it is a parameter (default off) so explorers run unattended.
8. **Roles as starter programs.** Answered 23:26: yes, with as much as possible editable by people or by Orac (every rule above is pack data for that reason). Recommended: explorer, courier, miner, trader, patrol and pirate ship as starter programs in one "ship roles" pack, each editable and copyable like any pack data, with no role written in code. V3's roles are a checklist of what to cover, not a copy.
