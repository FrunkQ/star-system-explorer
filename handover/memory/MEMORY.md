# SSE4 project memory

## HANDOVER (Alex 24 Nov)
- The dev window on this Claude account closes 25 Nov; the project moves to another account that gets only the repo and Alex's PC. Start at handover/START-HERE.md (memory copy, track map, golden lane, open questions; PR 424). Every thread: small PRs that merge green, nothing only in scratch, its row in handover/status/ kept true. Project files copied to C:\Development\sse4-project-files (13 zips, snapshot 17:14 UTC 24 Nov; top-up copy due late). Alex: "call me if there is a stop".
- Golden lane: Alex agreed to split the golden by area. Kernel thread first takes the op list (line 1) and unknown-op line (41) out of contract.jsonl, then per-area files; no CI edit needed. Until then one golden-changing PR at a time.

## What it is
- Star System Explorer (SSE) 4: rewrite on its own engine. Physics is editable, cited node flows ("lean into the physics, never fake it"). Procedural, deterministic, Big Bang to heat death. RPG/storytelling toolkit first, astrophysics education second.
- Kernel Rust, UI TypeScript and Svelte, renderer three.js WebGPU (WebGL2 on phones), data-only recipes. Apache-2.0 crediting Alex. Agents bring-your-own-key (Orac designer, Minerva storytelling, Tars support).
- Repo https://github.com/FrunkQ/star-system-explorer (main), live https://v4.starsystemx.com; every merge deploys. V3 is live at starsystemx.com; SSE4 is ALPHA and its opening page sends real use to V3.
- Alex's PC AC-ProArt, source under C:\Development (V3 = star-system-explorer-v2; sse4 checkout; sse4-real-saves; sse4-trailer-gpu; Mappadux VTT = dynamic-map-renderer-v2, github FrunkQ/dynamic-map-renderer-v2). V3 is not in this project's repos.
- Sister sites: Explorers (FrunkQ/starsystemx-creator-hub; database, storage or user data changes need Alex's yes) and converter (FrunkQ/starsystem-convert).

## How Alex wants work done
- Design and mockups on Fable 5.2 (never code). Coding on Opus (claude-opus-5-5).
- One builder thread per screen section or file area; each owns its folder and agrees shared seams by message. Split further when a builder becomes the queue.
- Each stage lands clean so another Claude can take over from the repo alone.
- Every open question numbered with Claude's recommendation; he answers "yes to 1, 2 and 4".
- SSE4 PRs merge when checks pass. CI workflow or merge check edits need his own yes. A step blocked by a session permission check needs his own yes, never a Claude's. No git history rewrites.
- No temporary stuff in the repo: screenshots and working files go to /mnt/project-files/sse4/handover-screens/<PKG>-screens/.
- One general mechanism over specials; "think wide". Don't copy V3, do it better.
- Physics over simplification where practical in a browser; every fudge in the fudge register; small editable blocks citing papers; block names in plain words. Material properties only via the data sheet's "Look up" block, no typed copies.
- Interface: "dynamic minimalism", tactile, not "AI slop"; common things a click away; newcomer friendly.
- Guide, don't leave them alone: suggest, never refuse; physics breaks shown to the GM only; typing always available.
- Cheap "cool physics wins" earn credibility: small correct touches a clever user can test.
- Orac is a tool to lean on ("ask Orac to create it"), fully trained on blocks, method and UI tools. Showcases: coloured white dwarf, then Astrophage and Taumoeba with Andy Weir's names (publish only after he has seen it).

## Design files (/mnt/project-files/sse4/, repo copies in handover/design/)
- SSE4-build-plan.md (start here) and the other SSE4-*.md notes, accepted unless marked. Topic memories: sse4-galaxy-histories, sse4-naming, sse4-new-physics-demo, sse4-element-inventory, sse4-coming-together, sse4-example-packs, sse4-future-tech, sse4-atomic-rockets, sse4-secrets-and-visibility, sse4-interior-sites, sse4-editing-design, sse4-playbook-switchboard, sse4-run-desk-beats, sse4-exotic-objects, sse4-paper-spacecraft, sse4-binaries.
- Tracks: FT future tech (accepted); RK Atomic Rockets (accepted, all 25; RK8 name, credit and links waits on the author's written reply); SV secrets, one "Known to" eye, pencil and ink (accepted); IS interior sites (PARKED to about 4.2, only natural caves in 4.0); ED editing (accepted 24 Nov: six handles, strips, chart grips, units engine, "Could be…" bands with the "Ammonia-breathing spider-rock" profile); PB Playbook and Switchboard (accepted 24 Nov: bookmarks = what, layouts = how it looks, playbook stores, screens = where; goes back into Mappadux later); XO exotic objects (accepted 24 Nov: real light bending from a kernel bend table, pulsars and glitches, Composite band on every view, a general "hazard map" approach diagram with Park and Dive marks for any hazardous body including radiation belts).
- SSE4-v3-gaps.md: SSE3 features not yet in SSE4 (navigation audit 24 Nov; renderer and "looks like the real thing" audit requested 24 Nov).
- SSE4-checklist.md: whole-project checklist (Claude Doc https://claude.ai/artifact/Xci8FXvxcfJkLighBrzS67).
- Next milestone: the landing sequence and an end-to-end run-through; gaps in handover/landing-run-through-gaps.md.

## Standing rulings
- Processes, not classes: class is a label read off results. Tags never feed physics. Resources are real amounts that can be mined out.
- Detail levels Glance, Orbit, Survey, Site; same world at every level.
- UI: light "chart room" plus dark "Night chart"; desks Explore, Build, Run; depth Curious, Science, Engine; honesty marks (authored, impossible, fictional) on GM view only. One universe clock. Library is "Explorers".
- Releases: 4.0 has Traveller as the only game pack (never the TAS logo) and hand-set things as plain dated data; 4.1 is "dynamic life" (richer aliens, civilisations, economy, anything that grows by itself); 4.2 is the RPG release (other game packs, site authoring, living-simulation adventures). Check with Alex before game-pack features for 4.0.
- Packs: named-setting packs are free unofficial fan work; paid packs via Stripe Connect (Alex keeps 20%, not surfaced yet). Gaia data only in a separate optional pack. A supporter page (Ko-fi or Patreon) is later, never tied to fan packs.
- Seats: GM offline means players read-only; GM streams only data. Players never hand-pilot; no dogfighting. Danger is drama, never forbidden: physics shows the odds, dice decide.
- Time is first-class; deep time is a GM function. Real sky backgrounds; Oort cloud is a sphere. Sound off by default.
- Formation: nine stages with a violent phase; n-body written fresh, never from REBOUND (GPL). Dark matter and dark energy "theoretical"; no MOND.
- Trailers: gather captioned clips as features finish; never call the sample planet "Earth" unless really modelled.

## Rulings of 21 to 24 Nov
- Bands: Visible, Infrared, Ultraviolet, X-ray, Radio, Mag, then Composite (mix several); players get bands through their sensors.
- Shared view controls at every level: View style (Chart 2D, Chart 3D, True, Wireframe), Seen from (re-picking resets the view), Scale, HUD, Band. Renderer priority: visible work first; True galaxy 3D, fly to surface, cosmic web, ship styles.
- Ships: hulls sized from components by physics; 3D model plus plan views; Explore and Run may change manifest, crew and fuel, the ship itself only on Build; picture selector remembers the last chosen. Stations and land constructs generated from their description (merged PR 408).
- Voyages: fuel physical; FTL abstract per setting; campaign clock by home time; roles are editable pack programs. Flight logs split and join at dated events.
- Navigation: SSE3-style object selector beside the place name at every level; press-and-hold menu on every object; click a planet flies to its close-up.
- Binaries (all yes): a barycentre always, no threshold, computed not stored; removing a body re-homes its moons to the host; Charon and the big moons in the default Solar System; binary evolution later.
- Orbital stability verdict on add, every check an inspectable block (merged PR 389).
- Seas (climate thread, in order): colour from the liquid's absorption; waves by liquid properties; whitecaps; cheap in shaders. Helium is one more liquid plus a "superfluid" line below 2.17 K; test is a rogue planet in a void with a helium sea that just works.
- Stars: glow, churning surface, visible flares and ejections.
- Galaxy histories: every merger leaves a dated mark; leading arms rare (about 1 in 100) from a backwards merger, fading. Known bug: a kernel sign error makes every galaxy's arms lead (fix with "Build galaxy arm fix and atmosphere advice").
- Cosmic web: close to real, void fraction toward 77% as a cited number; must read as a 3D spiderweb, not a "wrinkled rug".
- Atmospheres: mantle redox sets the outgassed mix; "the science expects" line; advice not refusal; "Suggest an atmosphere".
- Surface landing: pick a site anywhere, dangers shown; ship autoroutes down; wander a patch a few km across; phones get a basic walk view.
- Run desk: no Next; Playbook presets fired at roles; Switchboard routes every screen (built, PR 421); OBS out early.
- Megastructures: a construct with attached physics blocks; late. Territory overlays: one system across scales (design thread). Class gallery in Reference from SSE4's own imagery.

## Build state (check list_threads)
- Long-standing builders: kernel "Build kernel resolve and rules", chart room "Make the SSE4 test drive work end to end", data sheet, renderer, travel, light, climate, Traveller, spaceflight, manifest, Orac, selector, Run desk, editing. Started 24 Nov evening: exotic stars, star glow and flares, landing run-through, ship flow and power views, galaxy charts and histories, galaxy arm fix and atmosphere advice, surface time-lapse, cosmic web view, names gallery and life board, Paper spacecraft pack, handover.
- Galaxy charts and histories thread: memory sse4-galaxy-thread (boundaries with travel, cosmic web, selector).
- About 250k lines of code and tests. Approved tidy-ups: split sse-kernel/src/lib.rs; remove old Rust copies in sse-atoms/src/test/ once golden checks are trusted.
- Caveats: Pluto's air too thick; Mercury's field dies early and Earth's inner core too big (fudge register); stellar halo about 100x low until GH4.

- Exotic objects build (track XO): memory sse4-exotic-build (PR order 443, XO17, XO18 part 1; XO1 part 2 and XO18 part 2 need the golden lane).

## Waiting on Alex
- Asked 24 Nov: roughly what time on the 25th the window closes.
- Older items (class thresholds, hycean class, colour per class, Night chart accent) return as proper questions when work reaches them.
- Star glow and flares (track SG, thread "Build star glow and flares", 24 Nov): glare halo, churning cells, lumpy giants, flares and ejections in True, PR 439. Memory sse4-star-look.
- Visual polish (track VP, 24 Nov): design note SSE4-visual-polish-design.md accepted in full by Alex 24 Nov 18:57 (PR 462); packages by owner thread in its section 12. Memory sse4-visual-polish.
