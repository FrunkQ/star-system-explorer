# SSE4 project memory

## What it is
- Star System Explorer (SSE) 4: total rewrite on its own engine. Physics is editable, cited node flows ("lean into the physics, never fake it"). Procedural, deterministic, Big Bang to heat death, no particle simulation. RPG/storytelling toolkit first, astrophysics education second. Pitch: "No Man's Sky, the astrophysicist's version".
- Kernel Rust, UI TypeScript and Svelte. Apache-2.0 crediting Alex. Agents are bring-your-own-key (Orac designer, Minerva storytelling, Tars support); keys never leave the browser.
- Repo https://github.com/FrunkQ/star-system-explorer (main), live at https://v4.starsystemx.com; every merge deploys. Chart room `/`, players `/players.html`, device check `/harness/`.
- V3 is live at starsystemx.com (Alex 24 Nov). SSE4 is ALPHA: the opening page says so and sends real use to V3 (Alex 24 Nov).
- Alex's machine AC-ProArt: source under C:\Development (V3 = star-system-explorer-v2, code in star-system-generator subfolder; sse4 checkout; real V3 saves in sse4-real-saves; trailer clips in sse4-trailer-gpu). V3 is not in this project's repos.
- Sister sites deploy on merge: Explorers (repo FrunkQ/starsystemx-creator-hub; database, storage or user data changes need Alex's yes) and converter (FrunkQ/starsystem-convert, merges when green).

## Design files (/mnt/project-files/sse4/, repo copies in handover/design/)
- SSE4-build-plan.md (start here) plus every other SSE4-*.md design note there, all accepted unless marked. Detail in memories sse4-galaxy-histories, sse4-naming, sse4-new-physics-demo, sse4-element-inventory, sse4-coming-together.
- SSE4-example-packs.md (accepted 23 Nov): an example pack per Explorers shelf; Traveller is the only 4.0 game pack, the rest parked for 4.2; Looks (filters, palettes, signal faults, screen delivery) accepted 20:08. Detail in memory sse4-example-packs.
- SSE4-future-tech-design.md (track FT, accepted 23 Nov): tech is how far ten physical limits are pushed; ladder per setting; capability on a civilisation, dated. Detail in memory sse4-future-tech.
- SSE4-atomic-rockets-plan.md (track RK, accepted 23 Nov, questions 1 to 25 all yes): Atomic Rockets as base data plus a free "Paper spacecraft" pack; one power model (four buses); one Bands table; weapons as in-range sums. Detail in memory sse4-atomic-rockets.
- SSE4-secrets-and-visibility-design.md (track SV, direction accepted 23 Nov): one "Known to" control; pencil (GM only) and ink (table knows) on the GM chart; the mark is a small three-state eye (Alex: not a dot, not words). Detail in memory sse4-secrets-and-visibility.
- SSE4-interior-sites-design.md (track IS, designed 23 Nov, PARKED by Alex for about 4.2): sites a party can go inside (caves, lava tubes, wrecks, mines); only S14 natural caves stay in 4.0. Detail in memory sse4-interior-sites.
- SSE4-editing-design.md (track ED, ACCEPTED by Alex 24 Nov 15:21; Solar System build experiment in 10B, rerun on the built UI as ED0b; units engine 3C; survivability bands 11C with the "Ammonia-breathing spider-rock" counter-profile): editing by six handles each with a box of parts, strips, chart grips, typing always, pairs always (no barycentre threshold), "Could be…" suggestions with requirement profiles, one editing grammar. Detail in memory sse4-editing-design.
- SSE4-v3-gaps.md: SSE3 features not yet in SSE4; second audit 24 Nov (items 37 to 42 are navigation: selector, press-and-hold menu, system hover, zone lines, find by tag, notes).
- 4.2 is the RPG release (Alex 23 Nov 19:50): Traveller is the 4.0 showcase; other game packs, by-hand site authoring and play pieces wait for 4.2. Check with Alex before game-pack features for 4.0.
- Later, not yet (Alex, 23 Nov): a Ko-fi or Patreon supporter page with a Discord badge, for all his RPG tools (Ovid more than SSE4), made from existing assets when there is good material; never tied to named-setting fan packs (they stay free; tell Mongoose donations are voluntary).
- Next milestone (Alex, 23 Nov 22:16): the landing sequence and an end-to-end run-through, the GM plotting the players' experience of the events; chart room thread writes the gap list.
- 4.2 idea (Alex, 23 Nov 19:47): dynamic adventures in a living simulation, a co-GM making random encounters and events, pirate skirmishes as a side channel leaving wrecks to investigate.
- trailer/ (SHOTLIST.md, STORYBOARDS.md, clips); ship pictures in ships/ (styles mockup: styles-and-tech-mockup.html, artifact "SSE4 ship styles and tech levels").

## How Alex wants work done
- Design and mockups on Fable 5.2 (limited weekly allowance, never for code). Coding on Opus (claude-opus-5-5).
- One builder thread per screen section for contained context and speed (Alex 24 Nov, "split things up and speed up dev"): chart room, "Build object selector and menus", "Build Run desk playbook and players' side", "Build planet and star editing"; shell/src/App.svelte is being split into section folders so they don't collide.
- Each stage lands clean so another Claude can take over from the repo alone (handover/; status rows per track in handover/status/, START-HERE.md index).
- Every open question numbered with Claude's own recommendation; he answers "yes to 1, 2 and 4". Questions as one numbered list a day unless something can't be undone or stops a stage. No release date pressure.
- SSE4 PRs merge when checks pass ("main is entirely yours"). CI workflow or merge check edits need his own yes. One golden-changing PR open at a time.
- No temporary stuff in the repo (Alex 24 Nov: "From now on DON'T save temp stuff like this in the repo"): screenshots and other working artefacts go to /mnt/project-files/sse4/handover-screens/<PKG>-screens/ or project files, never committed (PR 338). The old screenshots stay in git history; Alex is fine with that, no history rewrite.
- One general mechanism over specials. Don't copy V3; do it better. "Think wide": behind an edge case, find the general, extensible capability and its other uses.
- Physics over simplification where practical in a browser; every fudge in the fudge register; small editable blocks citing papers. Config designed for AI (Orac) first. Block names in plain words a keen amateur can follow.
- Interface is "dynamic minimalism", tactile, not "AI slop". Common things a click away, rare things in menus with context. Place each control by how often it is used; easy flow, newcomer friendly (Alex 23 Nov).

- Orac (Alex 24 Nov): a tool to lean on, e.g. "ask Orac to create it" when a gas is not defined; like in Ovid it must be fully trained on the blocks, the dev methodology and the UI tools at its disposal. Showcases: coloured white dwarf first, then Astrophage and Taumoeba using Andy Weir's names (publish that pack only after he has seen it).
- Guide, don't leave them alone (Alex 24 Nov): suggest what a world could be, each step optional; physics breaks shown to the GM only; typing always available for real data. Memory feedback-guide-not-leave-alone.

## Standing rulings
- Processes, not classes: no regime gates; class is a label read off results.
- Detail levels Glance, Orbit, Survey, Site; same world at every level; no heavy work for a click.
- UI: light "chart room" plus dark "Night chart"; Explore, Build, Run desks; depth Curious, Science, Engine; honesty marks (authored, impossible, fictional) on GM view only. One universe clock. Library is "Explorers".
- Tags are only labels, never feed physics. Resources are real amounts from physics that can be mined out.
- Universe packs add bases and labels to fitting worlds. Named-setting packs are free unofficial fan work, never paid. Paid packs via Stripe Connect, Alex keeps 20%, not surfaced yet. Economy is a separate 4.1 pack.
- Richer alien life is 4.1; 4.0 keeps life sockets and lineage ready (sse4/SSE4-checklist.md section 3).
- 4.1 headline: "bringing dynamic life to the simulation": civilisations defined from evolution and history, which drive their culture, tech and aims.
- 4.0 lighter touch (Alex 23 Nov 17:49): hand-set things are plain dated data; anything that grows by itself (OV14, tech trees, research, culture) waits for 4.1.
- The 4.1 economy drives the autopilots; 4.0 leaves role programs, minable resources, routes and holdings ready.
- Trailers: gather captioned clips as features finish; never call the sample planet "Earth" unless really modelled.
- Seats: GM offline means players read-only; GM streams only data. Players never hand-pilot; no dogfighting. First game pack is "Traveller pack" (never the TAS logo).
- Game layer: record computed on GM machine; dice use a GM-only secret; competitive packs need sealed seats.
- Danger is drama, never forbidden: physics shows the odds of a risky act; dice decide; the app never filters the option out.
- Time is first-class; deep time is a GM function. Real sky backgrounds; Oort cloud is a sphere.
- Rendering: three.js WebGPU, data-only recipes; phone uses WebGL2. Heavy physics ships as tables behind swappable blocks.
- Formation: nine stages with a violent phase; n-body written fresh, never from REBOUND (GPL).
- Surfaces deterministic from history; life from energy gradients. Remote survey by real methods. Sound off by default.
- Gaia data is non-commercial, so only in a separate optional pack.

## Rulings of 21 to 24 Nov (detail in the design files)
- Master data sheet (pack `base-reference`): one cited sheet; one "Look up" block is the only door to material properties; CI checks: sheet out of date, no typed copies.
- Magnetic fields: "Mag" is a band in the band switch (Visible, Infrared, Ultraviolet, X-ray, Radio, Mag); it replaces the picture; players get it through their lens like any sensor.
- Voyages: fuel physical; FTL abstract with rules per setting; campaign clock by home time, crews age by ship time; roles (explorer, courier, miner, trader, patrol, pirate) are editable pack programs.
- Ships: hulls sized from components by physics; 3D model plus plan views, no deck plans; aerobraking and landing computed per ship per world.
- Ships by desk (Alex 24 Nov): Explore and Run allow manifest, crew and fuel changes but not changes to the ship itself, its picture or 3D model; those are Build only. UI is context sensitive. Picture selector (own images, 2D plans, 3D) remembers the last chosen. Fuel gauge with top-up: yes.
- Stations and land constructs (Alex 24 Nov, renderer thread): generated from their description by a simple self-contained procgen builder, about two dozen base shapes sized to population with bolt-ons, docking points by size, and a cut-off where it becomes a megaconstruct. Generator merged 24 Nov, PR 408.
- In-system selector (Alex 24 Nov, chart room): adopt SSE3's object selector at the top beside the current place, reused at every level, showing the current level, one above and one below. First version merged 24 Nov (PR 399) with click-to-fly close-up.
- Future tech: skipped physics always shown, marked fictional on GM view, logged as fudges; availability tied to civilisation capability. Size rule 5.2 and "keeping the books" (section 15) accepted 23 Nov.
- Orbital stability (Alex 23 Nov 22:03, yes to all six): card verdict on add (Hill spacing, AMD with e and i, Kozai, resonances, binary limits; N-body on request), every check an inspectable block. Merged 24 Nov, PR 389.
- Megastructures: a construct is just a construct; mega ones carry attached physics blocks. Late stage 5, design note first.
- Surface landing (all accepted): pick a site from the world's map or anywhere, dangers shown; ship autoroutes down; free wander over a patch a few km across; facilities only where a pack, story or GM puts them; phones get a basic walk view.
- Shared view controls (renderer, every level): View style (Chart 2D, Chart 3D, True, Wireframe), Seen from, Scale (Readable or Real), HUD (pack overlays), Band (recipes as data; seats get only their sensors' bands). Size comparison gets 3D, True and Wireframe with slow spin (renderer, started 24 Nov).
- Territory overlays (asked 23 Nov, thread "Design territory overlays across all scales"): politics and areas of interest on planet surfaces and galaxy maps as one system across scales; design in progress.
- Renderer priority (Alex, 23 Nov): visible renderer work first; True galaxy in 3D, then fly to the surface, then cosmic web, then ship styles.
- Class gallery in Reference from SSE4's own imagery; gas giant colours keep V3's rainbow range from cloud chemistry. Dropped for now: Toytown spacing, GM skins, low-power switch.
- Galaxy histories: galaxies resolve from a merger history; the Milky Way should look battle-worn; every collision leaves a dated mark the renderer draws. Dark matter and dark energy on the sheet as "theoretical"; no MOND mode. Leading arms (Alex yes, 24 Nov): rare (about 1 in 100), from a backwards merger, fading with time (section 9.4). Known bug: a kernel sign error makes every galaxy's arms lead; fix queued in the golden lane.
- New-physics demo (accepted in full): Orac brings in a paper (coloured white dwarfs) as visible blocks and saves it as a pack; claim is "no new code for this paper".
- Integration: six stages (test drive; one system in full; one language underneath; galaxy has a past; run a game; show it), one lane per cross-cutting change.
- Cosmic web (Alex 23 Nov 22:14): stay close to real; void fraction toward the measured 77% as a cited editable number; the view must read as the 3D spiderweb of filaments, nodes and voids, not a "wrinkled rug". Tides and waves: yes. 4.1 scope questions 7 to 9: yes.
- Atmospheres (Alex 23 Nov 22:27, queued behind the run-through): mantle redox state sets the outgassed mix; "the science expects" line beside hand-set air; "doesn't go together" note as advice; "Suggest an atmosphere" button; UV photochemistry later.

## Build state (check list_threads for current)
- Build threads (Opus) are listed by list_threads; the kernel one is "Build kernel resolve and rules", chart room "Make the SSE4 test drive work end to end", data sheet "Build the master matter data sheet".
- Design threads (Fable), 23 Nov: "Choose example packs for every shelf", "Design territory overlays across all scales", "Design future tech as problems solved".
- Long-running: "Capture footage for SSE4 trailers" (daily clip run; GPU shots on AC-ProArt).
- 23 Nov: constants sweep done; kernel typed-copy ledger to 0 in PR 326; 71 physics blocks are named Equations (PR 266), 14 stay in Rust. Element inventory under way (kernel MP7 next).
- Code size 23 Nov: about 250k lines of code and tests (thread "Measure SSE4 code base size"). Tidy-ups Alex approved: split sse-kernel/src/lib.rs and shell/src/App.svelte (App.svelte split in checks 24 Nov); remove the old Rust copies in sse-atoms/src/test/ once golden checks are trusted.
- Caveats: Pluto's air too thick; Mercury's field dies early and Earth's inner core too big (fudge register); stellar halo ~100x low until GH4.

- Playbook and Switchboard design (track PB, ACCEPTED by Alex 24 Nov 16:34; bookmarks = what, layouts = how it looks, playbook stores, screens = where; he is refining it in the thread "Design playbook and switchboard"): SSE4-playbook-and-switchboard-design.md, mockup in handover-screens/PB-screens/, PR 411. Sources, destinations, lines; packages PB1 to PB11. Memory sse4-playbook-switchboard. Alex 24 Nov: the routing system goes back into Mappadux (his VTT) when that is rethought, "the ultimate space VTT"; keep notes that can carry over.
- Run desk (Alex 24 Nov 15:28-15:39): no Next; a Playbook of presets fired at roles; a Switchboard "nerve centre" routing what every screen sees, generally available, replacing Seats dialog and seat rows; OBS out early, OBS in GM-side only. Fable design first. Memory sse4-run-desk-beats.

## Waiting on Alex
- Asked 24 Nov 17:10 (thread "Design exotic stars and black holes"): 11 questions on the exotic objects design (track XO, SSE4-exotic-objects-design.md, PR 420; real light bending from a kernel bend table, pulsars, Wolf-Rayet). Memory sse4-exotic-objects.
- Asked 24 Nov 15:00 (kernel thread): barycentre-always for binaries, question 1 confirmed, 2 to 5 open (memory sse4-binaries).
- Asked 24 Nov 15:50: start a Fable design for exotic stars and black holes on screen (disc and real light bending first, then pulsars and stripped giants), with star glow, churn and visible flares for the renderer queue.
- Whole-project checklist (phases, wanted-not-planned, 4.1): /mnt/project-files/sse4/SSE4-checklist.md and Claude Doc https://claude.ai/artifact/Xci8FXvxcfJkLighBrzS67 (23 Nov).
- Older items (class thresholds, hycean class, colour per class, Night chart accent) return as proper questions when work reaches them.