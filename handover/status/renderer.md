# Status: Photoreal renderer

Owner thread: "Build photoreal renderer". Covers: R, the drawing side of Y5 and MF14a.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| R1 pass library, render recipes, degrade order, governor, quality badge (the Body globe runs on recipes: GPU, canvas, words) | done (renderer track) | packages/R1.md |
| V3 gap item 14 part 1: ship hulls and drive plumes from the ship record's physics (tanks, shield, thrust, jet power, words, Chart paths, lathe outline) | done | packages/SHIP14.md |
| V3 gap item 14 stage 2: ships designed from their parts (base-ships pack rules and styles, side and top views with call-outs, remix, hand overrides, ShipPlans view) | done | packages/SHIP14.md |
| V3 gap item 14 stage 3: the 3D model from the same layout (volumes and radiator area checked), panel textures and looks per style, glowing hot radiators, Chart/3D through the shared ViewControls in ShipPlans | done | packages/SHIP14.md |
| V3 gap item 14 stage 4: abilities shape the ship from the kernel's ship_at_world (heat shield, wings or lifting body, legs or wheels, parachute, scoop), drawn in plans and model, verdicts in ShipPlans | done; stage 5 (styles per nation or culture) next | packages/SHIP14.md |
| V3 gap item 14 stage 4b: the designed ship on the Voyages At world card, "Fit it for" the world, and "Watch the pass" (shield at radiative balance, bow shock per Billig) | done | packages/SHIP14.md |
| V10a: True on the system chart, one press away: worlds in their own surfaces (level-0 tiles), lit by their star, star in its own colour; Scale (Readable/Real) and HUD (Off/On) in the one ViewControls | done; living star, bands, other screens carrying the controls next | packages/V10a.md |
| V10b: Band in the one ViewControls (Visible, Infrared, Ultraviolet, X-ray, Radio, Mag); a view declares the bands it can show in its spec, the rest read "not seen in … here yet"; kept in the saved view choice | done; views wire their bands next (chart room moves Band out of LightControl) | this row |
| V10c: a living star in True: the kernel's star (limb darkening, spots from the cycle, flares from the flare list) turning on its spin on its own clock, six hours a second; StarDisc and True read one `readLiveStar` | done; X-ray band through True next | packages/V10c.md |
| OV1: pack overlays under the HUD switch: overlays as pack data (a hex grid or fixed layers), placed by campaign anchors, drawn by detail of zoom; first on Voyages between stars | done; Traveller pack entry, turning and dragging, galaxy chart next | packages/OV1.md |
| V10d: bands through True: X-ray (the star's corona and flares; worlds dark) and Infrared (10 µm, each body by its own heat, mapped grey); the Window's sky follows the one Band | done | packages/V10d.md |
| V10e: Mag through 3D and True: every world's field lines and the wind's bubble round it at system scale, from read_field; Real at true size, Readable cut short of neighbours | done | packages/V10e.md |
| OV2: territory map layers, the contract: `areas` on the overlay spec (any source: relation, tag group, choice, fact or bands, charted, reach), area layers drawn as outlines, map palette slots with patterns and slot assignment, frames body-latlon and web-mpc, the `Holding` shape agreed with the kernel's OV3 | done; OV4 (Atlas look, palette in both themes) after OV3 lands | packages/OV2.md |
| V9t: the galaxy in 3D and True: the galaxy chart in the one set of view controls; True draws the kernel's rings, arms, bar and bulge in light in every band (old stars by mass, young by Kennicutt, dust from the gas), as layers bent by the collisions' marks; 3D turns the diagram | done; edge-on dust lane, thick disc and halo glows, nearby galaxies and the web in the same looks next | packages/V9t.md |
| VC1: one page of the view controls at every scale (`docs/view-controls.md`), made from the screens' own specs by `tests/ui/view-controls-page.ts` and checked by a test, so it cannot drift | done; it is the list of gaps to close: True and 3D for the web, nearby galaxies, sector and voyages; bands on the globe | docs/view-controls.md |
| GH9: galaxies' collision marks drawn from GH3's dated marks (warp, ripples, tilt, rings with emptied middles, lopsided and driven arms, knots, thick disc), face-on layer and edge-on strip in the galaxy chart | done; True drawing done in V9t | packages/GH9.md |

## Done so far and what is next

R1, R2a (air from kernel facts, physical lighting, True colour), R3 (still tier, "Take photograph"), R4 part 1 (checked recipe edits, hand editor) and R5a (stars: photosphere, spots, X-ray corona, granules from O10 cells), R6 (time-lapse layers, `packages/R6-timelapse.md`) and R2b part 1 (today's clouds, haze and storms from the weather op, `packages/R2b-weather.md`), Y5 parts 1 and 2 (the stars beyond, and behind the Window's lights, `packages/Y5.md`) and MF14a part 1 (the Mag layer's shared tracer, Chart cut and 3D lines, `packages/MF14a.md`) done; air from the kernel's read_air_optics; R2b part 2 (cloud shell at height on the GPU tiers), MF14a part 2 (3D tubes and True band emissions), R5 light-lens views, R4 part 2 with Orac (A2)
