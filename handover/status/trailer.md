# Status: Trailer footage and side-by-side stills

Owner: thread "Capture footage for SSE4 trailers". Covers: the capture rig in `scripts/trailer/`, the shot list, the beauty shots and the "real photo, our version" stills.

Only the owner thread edits this file. Index and standing rules: `handover/START-HERE.md`.

## ON HOLD (Alex, 25 Nov 02:07 UTC)
Stop filming until all the colours have landed and the renderer is in better shape. Restart only when Alex or the coordinator says so. At minimum, wait for:
- climate PR 418: the giants' real colours;
- PR 488: Sizes labels and the giants' bands;
- the renderer's close-up package: a low orbit in True, Moon lighting, and HUD Off no longer pulling the camera back;
- the Moon's weathering and soil colours;
- postcards PC2.
The daily re-shoot task is disabled.

## Rules (Alex)
- Gather clips only as features finish. Never film or fake an unbuilt feature. Captions must be honest.
- Never call a sample or generated planet "Earth". The catalogued Earth has a measured orbit, size and field, but its continents are generated, so caption it so.
- Captions are burned in during the edit as a dark strip (`cut-preview.mjs`, from each clip's .srt). Film clean plates with `--no-captions`.
- Show True, HUD on and the Mag lines. Use Wireframe where it exists. Keep planet holds short.
- Land screen: wait for a grown world or a modelled Earth.

## Tools
| Tool | Use |
|---|---|
| `capture.mjs` | Films `shots/NN-*.mjs` (`--only`, `--no-captions`, `--base URL`, `--gpu --headed --channel chrome`). The cloud has no GPU, so True and globes film blank there. |
| `cut-preview.mjs` | Cuts a short preview under 10 MB with captions burned in (used for the Discord preview, 23 Nov). |
| `match-still.mjs` | Makes a 1920x1080 still: the photograph with its credit on the left, SSE4's frame (untouched) on the right. It reads a match folder (`ref.jpg`, `credit.txt`, `pose.json`). |

GPU takes run on Alex's PC (AC-ProArt) through Remote Control:
- the checkout is `C:\Development\sse4` on branch `trailer-gpu`;
- Playwright is reached through `NODE_PATH=C:\Development\dynamic-map-renderer-v2\node_modules`;
- takes go to `C:\Development\sse4-trailer-gpu\<date>`;
- film against `https://v4.starsystemx.com/`. A local build fails on Windows because the build fetches a Linux-only wasm-bindgen.

## State on 25 Nov
| Shot | State |
|---|---|
| 00 to 24 | Filmed in the cloud. The shot list with statuses is in the project files `sse4/trailer/SHOTLIST.md` (copied to Alex's PC with the project-files snapshot). |
| 06 3D fly-in, 26 True galaxy | GPU takes OK (25 Nov). |
| 25 True Solar System, 27 Sizes globes | GPU takes OK but **provisional**: the giants are grey and the Sizes labels are mangled. Reshoot after 418 and 488. |
| 28 Moon relief (beauty H1) | The shot works (it clicks the pins themselves and waits 30 s for the finer tiles). The picture isn't good enough yet: the Moon is small, evenly lit, with no relief. Waits on the close-up package. |
| Beauty H2 methane rainbow | The bow angles are built (`read_bows`, PR 406). Nothing draws the bow yet. Drop size is queued with climate. |
| Beauty H3 helium sea and ship lamps | Waits on climate (helium as a liquid, wave damping) and aurora/night lights (construct lamps), plus landing LR6 (the lander drawn on the ground). |
| Beauty H4 black hole tides | Next account. Needs XO1 part 2, XO2 to XO4, a hand-built system, and eccentric tides. Never fake the hole. |
| Side-by-side stills | The renderer thread gathered 54 references in project files `sse4/match/<id>/` (README has licences). First to film: saturn-portrait, jupiter-marble, blue-marble, galilean-moons, then the Visible-band moons. Check each licence before publishing: NASA is generally free with credit; ESA/Hubble/Webb are CC BY 4.0 with the exact credit; 67P is ESA copyright; Juno images may carry a processing credit. |

Storyboards A to I (intro, features, scale journey, physics demo, RPG series, system generation, teach it new physics, beauty shots, real photo vs ours) are in the project files `sse4/trailer/STORYBOARDS.md`.
