# SSE4 project memory

## Handover
- Dev window on this account closes 25 Nov 7pm London (Alex: "final close out"; some API credit if we slip). The project moves to another account that gets only the repo and Alex's PC. Start at handover/START-HERE.md. Every thread: small PRs that merge green, nothing only in scratch, its row in handover/status/ true. Project files copied to C:\Development\sse4-project-files (snapshot 24 Nov 17:14 UTC; top-up due late). "Call me if there is a stop".
- Golden lane (kernel PR 536, 25 Nov): tests/contract/requests.json closed at 380 cases; new cases in tests/contract/areas/<area>.json, golden tests/golden/areas/<area>.jsonl. Area-only PRs (plus sorted contract-reach lines) merge in parallel; core contract.jsonl line changes go one at a time, the coordinator hands out the slot (25 Nov midday: 417 proven core, then light, then kernel GZ12).
- Deploy: Cloudflare Workers Builds (`star-system-explorer`) on every merge to main, 4 to 10 min; ci.yml is the only workflow. Healthy 25 Nov.

## What it is
- Star System Explorer 4: rewrite on its own engine; editable, cited physics node flows ("lean into the physics, never fake it"); procedural, deterministic, Big Bang to heat death. RPG and storytelling toolkit first, astrophysics education second.
- Kernel Rust, UI TypeScript and Svelte, renderer three.js WebGPU (WebGL2 on phones), data-only recipes. Apache-2.0 crediting Alex. Agents bring-your-own-key (Orac designer, Minerva storytelling, Tars support).
- Repo github.com/FrunkQ/star-system-explorer (main), live v4.starsystemx.com. V3 live at starsystemx.com; SSE4 is ALPHA and sends real use to V3.
- Alex's PC AC-ProArt, source under C:\Development (V3 = star-system-explorer-v2; sse4; sse4-real-saves; sse4-trailer-gpu; Mappadux VTT = dynamic-map-renderer-v2).
- Sister sites: Explorers (FrunkQ/starsystemx-creator-hub; database, storage or user data changes need Alex's yes), converter (FrunkQ/starsystem-convert).

## How Alex wants work done
- Design and mockups on Fable 5.2 (never code); coding on Opus (claude-opus-5-5).
- One builder thread per screen section or file area; shared seams agreed by message; split when a builder becomes the queue. Each stage lands clean so another Claude can continue from the repo alone.
- Open questions numbered with a recommendation; he answers "yes to 1, 2 and 4".
- PRs merge when checks pass; never merge red. CI or merge-check edits need his own yes, except the standing yes (25 Nov): any thread may add its own test line to ci.yml, nothing else. A step blocked by a permission check needs his own yes. No git history rewrites. "Test locally, push bigger chunks" (memory sse4-light-track-local-ci).
- No temporary files in the repo: screenshots go to /mnt/project-files/sse4/handover-screens/<PKG>-screens/.
- One general mechanism over specials, "think wide"; don't copy V3, do it better.
- Physics over simplification where practical; every fudge in the fudge register; small editable blocks citing papers, named in plain words; material properties only via the data sheet's "Look up" block.
- Interface: "dynamic minimalism", tactile, not "AI slop", newcomer friendly; suggest, never refuse; physics breaks shown to the GM only.
- Cheap "cool physics wins" earn credibility. Orac is a tool to lean on; showcases: coloured white dwarf, then Astrophage and Taumoeba (publish only after Andy Weir has seen it).

## Design files (/mnt/project-files/sse4/, repo copies in handover/design/)
- SSE4-build-plan.md (start here) and the other SSE4-*.md notes, accepted unless marked; SSE4-v3-gaps.md; SSE4-checklist.md (Claude Doc https://claude.ai/artifact/Xci8FXvxcfJkLighBrzS67). Topic memories: sse4-galaxy-histories, sse4-naming, sse4-new-physics-demo, sse4-element-inventory, sse4-coming-together, sse4-example-packs, sse4-future-tech, sse4-atomic-rockets, sse4-secrets-and-visibility, sse4-interior-sites, sse4-editing-design, sse4-playbook-switchboard, sse4-run-desk-beats, sse4-exotic-objects, sse4-paper-spacecraft, sse4-binaries, sse4-galaxy-thread, sse4-exotic-build, traveller-pr400-batch, sse4-true-surface, sse4-star-look, sse4-visual-polish, sse4-aurora-drawing, sse4-atlas-and-editions, sse4-atlas-rulings, sse3-source-repo, sse4-outside-models, sse4-postcards, sse4-spaceflight-history, sse4-grand-zoom.
- Accepted tracks: FT future tech; RK Atomic Rockets (RK8 name and credit waits on the author's reply); SV secrets ("Known to" eye, pencil and ink); ED editing; PB Playbook and Switchboard; XO exotic objects; VP visual polish; AT Atlas and editions; PC postcards; SH spaceflight history and In memory ("one place we don't joke"). IS interior sites PARKED to about 4.2. OM outside models BANKED, nothing queued.
- FH from inside worlds (SSE4-from-here-design.md): ACCEPTED 25 Nov, builder opens on the new account. Aids toggleable, folded under HUD, remembered; descent shown as a transition; uploaded audio appears under "Your own…"; shipped clips are real NASA audio and fresh readings of public-domain text, never film, radio-play or musical clips.
- GZ grand zoom (SSE4-grand-zoom-design.md, thread "Design the grand zoom"): ACCEPTED 25 Nov with changes: tie-up of levels first, then Local Group rebuild, then the zoom; gazetteer; "Motion" HUD layer. Stage A (GZ0, 12, 13, 2, 17, 15, 14) with the kernel thread; Stage B (GZ5 to 7) with "Build galaxy charts and histories" (builder only, design stays with GZ); Stage C builder "Build travel between places" not opened, for the new account.
- Three modes (thread "Design each display in three modes", 25 Nov): what every level shows in Explore, Edit and Play; not yet accepted.

## Standing rulings
- Processes, not classes: class is a label read off results. Tags never feed physics. Resources are real amounts.
- Every level sits in the one above (Alex 25 Nov): no system without a star slot, no star slot without a cell, no cell without a galaxy, no galaxy without a halo in the web; each level drives the physics below. Places and names are cached once at the defined date; only in-system motion is computed live.
- Detail levels Glance, Orbit, Survey, Site; same world at every level.
- UI: light "chart room" plus dark "Night chart"; desks Explore, Build, Run (Alex 25 Nov says Explore, Edit, Play: Play is clean, visual controls only; names to settle in the three-modes design); depth Curious, Science, Engine; honesty marks on GM view only. One universe clock. Library is "Explorers".
- Shared view controls at every level: View style (Chart 2D, Chart 3D, True, Wireframe), Seen from, Scale, HUD, Band. Bands: Visible, Infrared, Ultraviolet, X-ray, Radio, Mag, Composite; players get bands through sensors.
- Releases: 4.0 Traveller is the only game pack (never the TAS logo); 4.1 "dynamic life"; 4.2 the RPG release. Check with Alex before game-pack features for 4.0.
- Packs: named-setting packs are free unofficial fan work; paid packs via Stripe Connect (Alex keeps 20%); Gaia data only as a separate optional pack.
- Seats: GM offline means players read-only; players never hand-pilot; no dogfighting. Danger is drama: physics shows the odds, dice decide.
- Time is first-class; deep time is a GM function. Real sky backgrounds; Oort cloud a sphere. Sound off by default.
- Formation: nine stages with a violent phase; n-body written fresh, never from REBOUND (GPL). Dark matter and dark energy "theoretical"; no MOND.
- Ships: hulls sized from components by physics; ship itself edited only on Build. Voyages: fuel physical, FTL abstract per setting. Navigation: object selector beside the place name, press-and-hold menu, click a planet flies to its close-up.
- Binaries: a barycentre always, computed not stored. Orbital stability verdict on add.
- Seas: colour from the liquid's absorption, waves by liquid properties; helium sea with a superfluid line works (PR 538).
- Galaxy histories: mergers leave dated marks; leading arms rare. Arm sign fixed (PR 463). Alex yes 25 Nov: add the Local Arm from measured positions (Reid 2019).
- Cosmic web: void fraction toward 77%, must read as a 3D spiderweb. Atmospheres: mantle redox sets the mix; advice not refusal.
- Run desk: no Next; Playbook presets fired at roles; Switchboard routes every screen.
- Trailers: captioned clips as features finish; never call the sample planet "Earth" unless really modelled. Filming PAUSED by Alex 25 Nov until colours and renderer are in better shape.

## Build state (check list_threads)
- Builders: kernel "Build kernel resolve and rules", chart room "Make the SSE4 test drive work end to end", proven core, data sheet, renderer, detail ladder, travel, light, climate, giant bands, plumes, aurorae, Traveller, roll call, Orac, Run desk, editing, Atlas, postcards, exotic, galaxy charts, galaxy arm fix, quality tiers, handover.
- About 250k lines. Approved tidy-ups: split sse-kernel/src/lib.rs; remove old Rust copies in sse-atoms/src/test/ once golden checks are trusted.
- Caveats: Pluto's air too thick; Mercury's field dies early and Earth's inner core too big; stellar halo about 100x low until GH4.
- Renderer matching (25 Nov): 11 worlds beside famous photos in handover-screens/MATCH1-screens/. Alex unimpressed ("bruised potatoes", lacking colour, a step back from the first test render). Causes: sheet shot full-face before fine tiles loaded (renderer: three-quarter light on fly-in, faster fine tiles, re-shoot side-lit); one tan palette on airless worlds (PR 572); basins as flat dark patches and icy worlds built as rock (proven core, after PR 417); Io sulphur has no optics rows; Venus cloud structure owed.

## Waiting on Alex (25 Nov 12:50)
- Postcard seed travelling with a sent card (recommended keep).
- Quality tiers thread needs its PC connection approved to read his bench report at C:\Development\sse4\docs\perf-reports\ac-proart\sse4-bench-device.json.
- Older items (class thresholds, hycean class, colour per class, Night chart accent) return as questions when work reaches them.
- Three desks (track TM, 25 Nov): ACCEPTED 13:30 UTC, yes to all 15; desk names stay Explore, Build, Run (Alex says Explore, Edit, Play); every new screen says what it shows on each desk; builder "Build the three desks" (Opus) to open with TM1. Memory sse4-three-desks.
