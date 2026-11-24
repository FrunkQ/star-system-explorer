# SSE4 trailer storyboards (outline, 21 Nov 2026)

Both trailers lead with role-playing, then learning, exploring and worldbuilding. Look: chart room paper and
ink, calm screens, motion that explains a change. Captions are set in the chart room type, short, on paper-
coloured cards; no voice-over needed. Marked `[have]` where a clip exists, `[wait]` where the feature is not built.

## A. Intro trailer (about 60 s): what SSE4 is

1. Black. Type on paper: "I came here for my sci-fi RPG..." (2 s)
2. The chart room opens; the pointer peeks a world, the card grows from it. [have 01]
3. Caption: "Build the systems your players fly through." Players' page fills in as they scan. [have 18]
4. Route: three courses to Vault; "How long, and will the crew survive it?" [have 13]
5. "Find tonight's story." Story finder hits. [have 15]
6. Turn: "...and left with a PhD in astrophysics."
7. Fast cuts, one idea each, 2 s apiece: why? with the science and its paper [have 02]; drag a world and it
   cools [have 03]; the Sun in X-rays [have 10]; a planet cut in half [have 08]; true-scale Jupiter beside a
   neutron star [have 07]; the star ages and dies [weak 05].
8. Zoom out beat: galaxy rings [have 11] to the cosmic web growing [have 12]. Caption: "From one world to the
   cosmic web."
9. The hero globe: a world in true colour, from its own air and ground. [needs GPU 09]
10. "Why is this here?" A crater traced to the impactor that made it, or a lava tube. [wait S14 to S16]
11. Title: Star System Explorer 4. "No Man's Sky, the astrophysicist's version." v4.starsystemx.com

## B. Features trailer (about 2 to 3 min): chapters

Each chapter opens on a one-line caption card taken from the start-sheet lines.

1. **Run a game** (RPG first): chart room [01], players see only what they scanned [18], share with players on
   two screens [to film], route [13], stories [15], player themes on the bridge [to film], Traveller sector to
   real systems [wait for screen].
2. **Ask why**: edit and why [02], open the flow and change the physics [to film], the paper behind it.
3. **Build**: drag orbits [03], grow a system [16], import a V3 save [09 setup], sizes [07], Night chart [17].
4. **Time**: a year by the week [04], ten billion years on one slider [05, re-shoot when the one clock lands].
5. **Look closer**: cutaway [08], globe [needs GPU], star in X-rays [10], light switch [to film], life [to film],
   weather [wait].
6. **Why is this here?** The signature chapter: a crater's source, a coal seam's swamp, a lava tube's flow, each
   traced back through the history. [wait S14 to S16; the strongest reason to hold this trailer]
7. **Out to the galaxy**: galaxy and an inked merger [11], cosmic web [12].
8. End card with both pitch lines.

## Timing recommendation

Cut the intro trailer first, once the start sheet and the GPU globe are filmed (it needs neither surface detail
nor the one clock). Hold the features trailer for chapter 6.

---

# Added 21 Nov 2026 (Alex): three more videos

Marks: `[have NN]` a clip exists; `[GPU NN]` filmed on AC-ProArt's graphics card (clips/gpu/);
`[to film]` the feature is built but has no shot yet; `[wait X]` not built, no footage until it lands.

## C. Scale journey (about 60 to 90 s): from the filaments to a cave

One continuous move down through every scale, no cuts if the app can carry it; cross-dissolves until it can.
The pitch: "Zoom out until galaxies are dots. Then all the way back down, and underground."

| # | Leg | Status |
|---|---|---|
| 1 | Cosmic web: filaments, played from early times to today | [have 12] |
| 2 | Into one halo: galaxies nearby (cluster map) | [to film] cluster map exists ("Galaxies nearby") |
| 3 | The galaxy: arms, rings, the green band | [have 11] |
| 4 | Into the local stars around one system | [wait] Local stars chart is not built |
| 5 | The system chart, then 3D | [have 01] [GPU 06] |
| 6 | The planet globe, turning | [GPU 09] |
| 7 | Through the atmosphere: entry heating, haze, cloud deck | [wait] the descent scene exists in the kernel (S5 `plan_descent`) but has no view; atmosphere and clouds in the renderer wait on R2b |
| 8 | Onto the surface: ground-level view | [wait] ground-level rendering, detail levels D0 to D3 (S16a) |
| 9 | Underground: a lava tube or cave, with "why is this here?" | [wait] S14 lava tubes and caves |

Today only legs 1, 3, 5 and 6 are real, as separate clips. The continuous camera move between scales does
not exist in the app (each scale is its own screen), so the first cut would be dissolves. Recommendation: hold
this video until legs 7 to 9 land; film legs as they arrive. It is also the strongest closing sequence for
the intro trailer.

## D. The physics system (about 90 s): how it works, looks and is edited

| # | Beat | Status |
|---|---|---|
| 1 | A world's card: words first, numbers with honest precision | [have 01] |
| 2 | "Why?" on a value: the formula, its inputs, the papers behind it | [have 02] |
| 3 | Depth switch: Curious, Science, Engine, the same fact told three ways | [have 19] (on camera) |
| 4 | Open the flow: the rule as a small diagram of nodes, opened at the node that made the value | [have 19] |
| 5 | Change a parameter to a bad value: refused, with the reason | [have 19] |
| 6 | Change it to a good one: checked, its effect on every world listed, adopt | [have 19] |
| 7 | The world now says "Changed physics" on its card | [to film] (19 ends at adopt) |
| 8 | Change a world (mass, orbit) and the derived values follow | [have 02] [have 03] |
| 9 | Impossible edits allowed but stamped, with why | [to film] |
| 10 | Orac drafts a rule change from a sentence, checked by SSE's own validators | [to film] needs an agent key on the filming machine |
| 10b | Climate: why is it like this? Belts, reasons, assumptions | [have 20] |
| 11 | Big blocks with their own UI: geology, weather, life, stellar evolution | [have 08] cutaway, [have 05] star ageing; weather and life views [to film or wait] |

Most of this is filmable now. Recommendation: cut this one first, after the next re-shoot adds beats 7 and 9.

## E. RPG use cases: a short series (30 to 45 s each)

Role-playing comes first, so a series of short clips, one use each, reads better than one long video.
Ranked by how well each sells SSE4 to a GM:

1. **Prep a session in five minutes.** Grow a system [have 16], Stories finds the hook [have 15], drop a note
   on a world [to film: notes, J1, needs a screen]. Status: mostly filmable.
2. **Players see only what they have scanned.** The GM's full chart beside the players' screen filling in as
   they scan; fog of war per seat. Two browser windows side by side. [have 18] for the player half; split
   screen [to film] (seats V10 and share are built).
3. **How long to Vault, and will the crew survive it?** Astrogation: courses compared by fuel, time, dose
   [have 13], then the window view on arrival [GPU 14, weak: mostly empty sky]. Status: filmable.
4. **Dress the bridge.** The same system as green phosphor, glass cockpit, paper chart [to film] (V2 themes built).
5. **A Traveller sector into real systems.** Import codes, worlds that agree, impossible ones flagged.
   [wait] the hex map and import exist in the kernel but are not mounted in the app.
6. **Time jump.** Six months pass: worlds move, the campaign's now moves [have 04]; long jumps with autopilots
   and patrols stamped ESTIMATED [wait J6b, one universe clock].
7. **Pirates and patrols carry on without you.** Programs on the Run desk [wait: programs have no screen].
8. **Minerva tells it.** A scene told from only what the players know, numbers checked [to film] needs an agent
   key; filmable on AC-ProArt if you have one set up there.

Recommendation: film 1 to 4 next (all built), then 8 if a key is available on AC-ProArt; 5 to 7 wait.

---

## F. How a system is made (explainer, about 2 to 3 min). Asked for by Alex, 22 Nov

What the video argues: nothing is painted on. A system grows from a cloud through nine stages, and then its
worlds go on changing, with climate, volcanism, ice, rock and weather pushing on each other. Each chapter
ends on a "why?" that traces one feature back through the history.

| # | Chapter | What we show | Status |
|---|---|---|---|
| 1 | The cloud and the star | A seed, a cloud core, the star's mass and metals | [have 21] as a dated list; [have 22] the replay's first stage |
| 2 | The disc | Gas and dust bands, the water ice line creeping inward | [have 22] the disc drawn with its five condensation lines |
| 3 | Growth | Pebbles and planetesimals growing into worlds | [have 22] embryos with their ledgers, played forward |
| 4 | The violent phase | Giants migrate, orbits scatter, collisions and ejections (Nice-like) | [have 22] the violent phase with its events in words |
| 5 | Leftovers | Belts, Oort cloud, Trojans, moons and rings | [have 22] the run's worlds at stage 9; [have 01] belts on the chart; [wait] Oort cloud drawn as a shell |
| 6 | The star ages | Main sequence, giant, white dwarf, and what it does to the worlds | [have 05] look ahead; [have 21] the dated list |
| 7 | One world, many processes | Plates and volcanoes heat and resurface it; the climate sets rain and ice; rivers and glaciers wear it; ice ages follow the orbit's cycles; weather on top | [have 08] cutaway; [have 20] climate belts; [built, not filmed] time-lapse layers and their controls on the Body globe (U22), waiting for a modelled or honestly named world and a GPU take; [wait] layers on the globe beyond what U22 draws, and a time-lapse of them interacting (the kernel has the models: plates, volcanism, shared climate, rivers, glaciers, ice ages; the screen does not yet) |
| 8 | Why is this here? | A glacier valley, a lava tube, a crater, each traced to its cause | [wait] feature "why" on the globe; lava tubes have envelopes in the kernel only |
| 9 | Realism | The papers behind each step; edit a rule and watch the world answer | [have 02] [have 19] |

Filmable today: chapters 1, 2 and 6 as text and charts, 5 in part, 9 fully, and the cutaway and climate
belts for 7. The heart of the video (3, 4, 7 and 8, the parts that move) needs views that do not exist yet:
a formation replay, and climate, ice, volcanism and weather layers on the globe over time. Recommendation:
ask the UI and renderer threads for those two views; they would also serve the scale journey (C).

Update 22 Nov: the formation replay (U19) landed and is filmed as shot 22, so chapters 1 to 5 now have
moving pictures. The globe time-lapse controls (U22) landed too; chapter 7 waits only on an honest world to film
and a GPU take.

---

## G. Teach it new physics (about 2 to 3 min). Asked for by Alex, 22 Nov. Film near the end

The core capability, end to end: hand Orac a paper, read the result as a row of simple, cited, editable
blocks, save it as a pack, load it on a device with no key. Pitch: "From a paper to physics you can see, check
and change." Alex: "the paper is not disappearing into a hidden codebase, it is surfaced as simple blocks", and
it "turns SSE from an RPG tool into something that could be used for academic what-ifs".
Test case: coloured white dwarfs (Hollands et al. 2026, doi:10.1093/rasti/rzag069). Full plan, feasibility and
what each shot waits on: `/mnt/project-files/sse4/SSE4-new-physics-demo.md` (track ND in the build plan).

| # | Shot | Status |
|---|---|---|
| 1 | An aged system; its white dwarf is a plain blue-white dot; Why? says "a blackbody" | [to film] built today |
| 2 | The paper dropped onto Orac: "Bring this in." | [wait ND4] |
| 3 | Orac's plan in words, with what it cannot do; the cost ticker runs | [wait ND4] |
| 4 | A draft refused by the kernel, then fixed | [have 19] shows the refusal by hand; Orac's version [wait ND5] |
| 5 | **Central shot.** The chain as blocks: what fell in, atmosphere, broad absorbers, spectrum, colour to the eye. Click one for its formula and paper. Curious, Science, Engine | [wait ND1, ND2, ND5] |
| 6 | Reproduction report: the paper's stars beside the pack's, with tolerance and fudge list | [wait ND6] |
| 7 | Accept; which stars change, before and after | [wait ND5] |
| 8 | The white dwarf turns green; disc with a tint at the limb; spectrum with the sodium trough; Why? leads to the day its asteroids fell in | [wait ND1, ND2, star disc, spectrum chart] needs GPU |
| 9 | By hand: drag sodium up, green slides to blue; change what fell in | [wait ND5] |
| 10 | Save as a pack; the listing shows sources, fudges and "cost to make, once" | [wait ND7] |
| 11 | Second browser, no key: fetch from Explorers, same star, same green, same hash | [wait ND8] two contexts side by side |
| 12 | End card | |

Rules for this one: Orac really does the job on camera with a real key (AC-ProArt), the real cost is shown,
time is compressed with the clock visible, and the pack in the film is the pack published. Never staged.
